use anchor_lang::prelude::*;
use frankcoin::state::Proof;

use crate::{constants::*, error::DaoError, state::{Dao, Citizen}};

/// Admit a member. A closed commune: entry is the admit authority's act, not a
/// permissionless self-join. Mining is still the floor — the authority can only
/// admit a wallet that has genuinely mined, verified against its real frankcoin
/// Proof account. So a member is always someone who did the work AND was let in.
#[derive(Accounts)]
pub struct Admit<'info> {
    #[account(mut)]
    pub admit_authority: Signer<'info>,

    #[account(mut, seeds = [DAO_SEED], bump = dao.bump, has_one = admit_authority @ DaoError::NotTheAuthority)]
    pub dao: Account<'info, Dao>,

    /// CHECK: the wallet being admitted. Only its key is used — to seed the
    /// citizen and to verify the proof belongs to it. Not a signer; admission
    /// is the authority's act, and the member consents off-chain.
    pub member: UncheckedAccount<'info>,

    /// The member's frankcoin Proof. Anchor's Account<Proof> only accepts an
    /// account genuinely owned by frankcoin, and the seed ties it to `member`.
    #[account(
        seeds = [frankcoin::constants::PROOF_SEED, member.key().as_ref()],
        bump = proof.bump,
        seeds::program = FRANKCOIN_PROGRAM,
        constraint = proof.miner == member.key() @ DaoError::ProofOwnerMismatch,
    )]
    pub proof: Account<'info, Proof>,

    #[account(
        init,
        payer = admit_authority,
        space = 8 + Citizen::INIT_SPACE,
        seeds = [CITIZEN_SEED, member.key().as_ref()],
        bump
    )]
    pub citizen: Account<'info, Citizen>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Admit>) -> Result<()> {
    let proof = &ctx.accounts.proof;
    require!(proof.count >= MIN_PROOFS_TO_JOIN, DaoError::InsufficientLabour);

    let clock = Clock::get()?;
    let citizen = &mut ctx.accounts.citizen;
    citizen.bump = ctx.bumps.citizen;
    citizen.wallet = ctx.accounts.member.key();
    citizen.admitted_by = ctx.accounts.admit_authority.key();
    citizen.joined_ts = clock.unix_timestamp;
    citizen.proofs_at_join = proof.count;
    citizen.votes_cast = 0;

    ctx.accounts.dao.citizen_count += 1;
    Ok(())
}
