use anchor_lang::prelude::*;
use crate::{constants::*, error::DaoError, state::{Citizen, Proposal, Ballot}};

/// Cast one vote. One citizen, one vote — the ballot account can only be
/// created once per (citizen, proposal), so a second attempt fails at account
/// creation, not at a check that could be forgotten.
#[derive(Accounts)]
pub struct Vote<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,

    #[account(
        mut,
        seeds = [CITIZEN_SEED, voter.key().as_ref()],
        bump = citizen.bump,
    )]
    pub citizen: Account<'info, Citizen>,

    #[account(mut, seeds = [PROPOSAL_SEED, proposal.id.to_le_bytes().as_ref()], bump = proposal.bump)]
    pub proposal: Account<'info, Proposal>,

    #[account(
        init,
        payer = voter,
        space = 8 + Ballot::INIT_SPACE,
        seeds = [BALLOT_SEED, proposal.key().as_ref(), citizen.key().as_ref()],
        bump
    )]
    pub ballot: Account<'info, Ballot>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Vote>, choice: u8) -> Result<()> {
    require!(choice <= 2, DaoError::BadChoice);

    let clock = Clock::get()?;
    let p = &mut ctx.accounts.proposal;
    require!(clock.unix_timestamp < p.closes_ts, DaoError::VotingClosed);

    // Every citizen counts for exactly one. No weighting, by design.
    match choice {
        1 => p.yes += 1,
        0 => p.no += 1,
        _ => p.abstain += 1,
    }

    let ballot = &mut ctx.accounts.ballot;
    ballot.bump = ctx.bumps.ballot;
    ballot.proposal = p.key();
    ballot.citizen = ctx.accounts.citizen.key();
    ballot.choice = choice;
    ballot.cast_ts = clock.unix_timestamp;

    ctx.accounts.citizen.votes_cast += 1;
    Ok(())
}
