use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};

use crate::{constants::*, error::FrankError, state::Config};

/// The General Secretary's **one and only** mint. This is the single founder allocation — a
/// premine performed in the open, once, by the sitting General Secretary — after which
/// `genesis_minted` latches true and every future frank can come only from
/// proof-of-work. There is deliberately no second path: this is what lets the
/// coin be listed and traded credibly, because "renounce after launch" is
/// enforced by the code, not promised in a tweet.
///
/// Constraints, all checked here or by Anchor:
///  * only the sitting General Secretary may call it (`has_one = general_secretary`, and `general_secretary` signs);
///  * it may run at most once in the life of the program (`genesis_minted`);
///  * it mints to a General Secretary-named recipient — it cannot touch anyone's balance,
///    only add newly-minted franks to an account of the General Secretary's choosing.
#[derive(Accounts)]
pub struct GensecGenesisMint<'info> {
    #[account(mut)]
    pub general_secretary: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG_SEED],
        bump = config.authority_bump,
        has_one = general_secretary @ FrankError::NotTheSecretary
    )]
    pub config: Account<'info, Config>,

    #[account(mut, seeds = [MINT_SEED], bump = config.mint_bump)]
    pub mint: Account<'info, Mint>,

    /// CHECK: the wallet that will receive the genesis allocation. Only its key
    /// is used, to derive the destination ATA.
    pub recipient: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = general_secretary,
        associated_token::mint = mint,
        associated_token::authority = recipient
    )]
    pub recipient_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<GensecGenesisMint>, amount: u64) -> Result<()> {
    // The latch: once true, never again — not even for this same General Secretary.
    require!(!ctx.accounts.config.genesis_minted, FrankError::GenesisAlreadyMinted);

    let authority_bump = ctx.accounts.config.authority_bump;
    let signer: &[&[&[u8]]] = &[&[CONFIG_SEED, &[authority_bump]]];
    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.recipient_ata.to_account_info(),
                authority: ctx.accounts.config.to_account_info(),
            },
            signer,
        ),
        amount,
    )?;

    let cfg = &mut ctx.accounts.config;
    cfg.total_minted = cfg.total_minted.checked_add(amount).ok_or(FrankError::Overflow)?;
    cfg.genesis_minted = true; // the one mint is spent; PoW-only from here forever
    msg!("general_secretary genesis mint: {} base units to {}; the mint is now proof-of-work only",
        amount, ctx.accounts.recipient.key());
    Ok(())
}
