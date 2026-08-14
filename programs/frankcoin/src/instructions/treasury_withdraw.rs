use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use crate::{constants::*, error::FrankError, state::Spent};

/// Execute a treasury spend authorized by a passed 0state proposal.
///
/// The treasury is owned by a program PDA with no private key, so franks can
/// only leave via this instruction, and only when a genuine 0state spending
/// proposal has passed. Execution is permissionless — anyone may enact the
/// membership's decision — because the recipient and amount are fixed in the
/// proposal, leaving no discretion. A per-proposal `Spent` marker prevents the
/// same proposal from being executed twice.
///
/// frankcoin decodes the proposal by hand rather than depending on the zerostate
/// crate, keeping the two programs decoupled; it verifies the account is owned
/// by the 0state program and carries the Proposal discriminator first.
#[derive(Accounts)]
pub struct TreasuryWithdraw<'info> {
    #[account(mut)]
    pub caller: Signer<'info>,

    #[account(seeds = [MINT_SEED], bump)]
    pub mint: Account<'info, Mint>,

    /// CHECK: the treasury PDA (token authority). Seeds enforce the address; it
    /// signs the transfer via its bump.
    #[account(seeds = [TREASURY_SEED], bump)]
    pub treasury: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = treasury
    )]
    pub treasury_ata: Account<'info, TokenAccount>,

    /// CHECK: the recipient wallet; must equal the proposal's spend_recipient.
    pub recipient: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = caller,
        associated_token::mint = mint,
        associated_token::authority = recipient
    )]
    pub recipient_ata: Account<'info, TokenAccount>,

    /// CHECK: the 0state proposal authorizing the spend. Verified to be owned by
    /// the 0state program, then decoded and checked by hand in the handler.
    #[account(owner = ZEROSTATE_PROGRAM @ FrankError::NotAZerostateProposal)]
    pub proposal: UncheckedAccount<'info>,

    /// Replay guard: created here, so a second execution of the same proposal
    /// fails at account creation.
    #[account(
        init,
        payer = caller,
        space = 8 + Spent::INIT_SPACE,
        seeds = [SPENT_SEED, proposal.key().as_ref()],
        bump
    )]
    pub spent: Account<'info, Spent>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

fn read_u64(d: &[u8], o: usize) -> u64 {
    u64::from_le_bytes(d[o..o + 8].try_into().unwrap())
}
fn read_i64(d: &[u8], o: usize) -> i64 {
    i64::from_le_bytes(d[o..o + 8].try_into().unwrap())
}

pub fn handler(ctx: Context<TreasuryWithdraw>) -> Result<()> {
    let clock = Clock::get()?;

    // --- decode the 0state Proposal account by hand ---
    let data = ctx.accounts.proposal.try_borrow_data()?;
    require!(data.len() >= 8, FrankError::NotAZerostateProposal);
    require!(data[0..8] == PROPOSAL_DISCRIMINATOR, FrankError::NotAZerostateProposal);

    // layout: disc(8) bump(1) id(8) proposer(32) title(4+len) body_hash(32)
    //         created_ts(8) closes_ts(8) yes(8) no(8) abstain(8)
    //         electorate_at_open(8) spend_recipient(32) spend_amount(8)
    let mut o = 8 + 1 + 8 + 32;
    require!(o + 4 <= data.len(), FrankError::NotAZerostateProposal);
    let tlen = u32::from_le_bytes(data[o..o + 4].try_into().unwrap()) as usize;
    o += 4 + tlen + 32 + 8; // title, body_hash, created_ts
    require!(o + 8 + 8 + 8 + 8 + 8 + 32 + 8 <= data.len(), FrankError::NotASpendProposal);
    let closes_ts = read_i64(&data, o); o += 8;
    let yes = read_u64(&data, o); o += 8;
    let no = read_u64(&data, o); o += 8;
    o += 8 + 8; // abstain, electorate_at_open
    let spend_recipient = Pubkey::try_from(&data[o..o + 32]).unwrap(); o += 32;
    let spend_amount = read_u64(&data, o);

    // --- verify it is a passed spending proposal for this recipient ---
    require!(spend_amount > 0, FrankError::NotASpendProposal);
    require!(clock.unix_timestamp >= closes_ts, FrankError::ProposalNotPassed); // voting closed
    require!(yes > no, FrankError::ProposalNotPassed);
    require!(spend_recipient == ctx.accounts.recipient.key(), FrankError::RecipientMismatch);
    require!(ctx.accounts.treasury_ata.amount >= spend_amount, FrankError::InsufficientTreasury);

    // --- transfer from the treasury, signed by its PDA ---
    let signer: &[&[&[u8]]] = &[&[TREASURY_SEED, &[ctx.bumps.treasury]]];
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            Transfer {
                from: ctx.accounts.treasury_ata.to_account_info(),
                to: ctx.accounts.recipient_ata.to_account_info(),
                authority: ctx.accounts.treasury.to_account_info(),
            },
            signer,
        ),
        spend_amount,
    )?;

    let spent = &mut ctx.accounts.spent;
    spent.bump = ctx.bumps.spent;
    spent.proposal = ctx.accounts.proposal.key();
    spent.amount = spend_amount;
    spent.ts = clock.unix_timestamp;
    Ok(())
}
