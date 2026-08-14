use anchor_lang::prelude::*;
use frankcoin::state::Proof;

use crate::{constants::*, error::DaoError, state::{Proposal, Ballot}};

/// Cast one vote. Membership is automatic: having mined frankcoin (a Proof with
/// at least MIN_PROOFS_TO_JOIN accepted proofs) is the whole qualification — no
/// join, no roster. Weight is derived live from that Proof: sub-linear in franks
/// mined and decaying with time since the last mine. The ballot account is
/// created once per (proposal, voter), so a second vote fails at creation.
#[derive(Accounts)]
pub struct Vote<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,

    /// The voter's frankcoin Proof — their membership card and their weight.
    /// Genuinely owned by frankcoin and tied to the voter by seed.
    #[account(
        seeds = [frankcoin::constants::PROOF_SEED, voter.key().as_ref()],
        bump = proof.bump,
        seeds::program = FRANKCOIN_PROGRAM,
        constraint = proof.miner == voter.key() @ DaoError::ProofOwnerMismatch,
    )]
    pub proof: Account<'info, Proof>,

    #[account(mut, seeds = [PROPOSAL_SEED, proposal.id.to_le_bytes().as_ref()], bump = proposal.bump)]
    pub proposal: Account<'info, Proposal>,

    #[account(
        init,
        payer = voter,
        space = 8 + Ballot::INIT_SPACE,
        seeds = [BALLOT_SEED, proposal.key().as_ref(), voter.key().as_ref()],
        bump
    )]
    pub ballot: Account<'info, Ballot>,

    pub system_program: Program<'info, System>,
}

/// Integer square root (Newton's method).
fn isqrt(n: u64) -> u64 {
    if n < 2 { return n; }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

/// Voting weight: base 1 (equal floor for every member), plus the integer square
/// root of *active* mined franks — total mined, halved once per HALF_LIFE of
/// silence since the last accepted proof, so stopping mining decays weight back
/// toward the floor.
fn weight_for(proof: &Proof, now: i64) -> u64 {
    let whole = proof.total_mined / ONE_FRANK;
    let idle = now.saturating_sub(proof.last_claim_ts).max(0);
    let halvings = (idle / HALF_LIFE_SECS).min(63) as u32;
    let active = whole >> halvings;
    1 + isqrt(active)
}

pub fn handler(ctx: Context<Vote>, choice: u8) -> Result<()> {
    require!(choice <= 2, DaoError::BadChoice);
    // Membership is automatic — you just have to have mined.
    require!(ctx.accounts.proof.count >= MIN_PROOFS_TO_JOIN, DaoError::InsufficientLabour);

    let clock = Clock::get()?;
    let weight = weight_for(&ctx.accounts.proof, clock.unix_timestamp);

    let p = &mut ctx.accounts.proposal;
    require!(clock.unix_timestamp < p.closes_ts, DaoError::VotingClosed);

    match choice {
        1 => p.yes += weight,
        0 => p.no += weight,
        _ => p.abstain += weight,
    }

    let ballot = &mut ctx.accounts.ballot;
    ballot.bump = ctx.bumps.ballot;
    ballot.proposal = p.key();
    ballot.member = ctx.accounts.voter.key();
    ballot.choice = choice;
    ballot.weight = weight;
    ballot.cast_ts = clock.unix_timestamp;
    Ok(())
}
