use anchor_lang::prelude::*;
use crate::{constants::*, error::DaoError, state::{Dao, Citizen, Proposal}};

/// Put a question to the citizens. Any citizen may propose — proposing is
/// itself an equal right, not something reserved to a proposer class.
#[derive(Accounts)]
pub struct Propose<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,

    #[account(mut, seeds = [DAO_SEED], bump = dao.bump)]
    pub dao: Account<'info, Dao>,

    /// Only a citizen may propose. The seed ties the citizenship to the signer.
    #[account(
        seeds = [CITIZEN_SEED, proposer.key().as_ref()],
        bump = citizen.bump,
    )]
    pub citizen: Account<'info, Citizen>,

    #[account(
        init,
        payer = proposer,
        space = 8 + Proposal::INIT_SPACE,
        seeds = [PROPOSAL_SEED, dao.proposal_count.to_le_bytes().as_ref()],
        bump
    )]
    pub proposal: Account<'info, Proposal>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Propose>, title: String, body_hash: [u8; 32]) -> Result<()> {
    require!(title.len() <= MAX_TITLE_LEN, DaoError::TitleTooLong);

    let clock = Clock::get()?;
    let dao = &mut ctx.accounts.dao;
    let p = &mut ctx.accounts.proposal;
    p.bump = ctx.bumps.proposal;
    p.id = dao.proposal_count;
    p.proposer = ctx.accounts.proposer.key();
    p.title = title;
    p.body_hash = body_hash;
    p.created_ts = clock.unix_timestamp;
    p.closes_ts = clock.unix_timestamp + dao.voting_period;
    p.yes = 0;
    p.no = 0;
    p.abstain = 0;
    // Fix the electorate at the moment the question opens, so quorum/turnout has
    // a stable denominator regardless of who is admitted or revoked later.
    p.electorate_at_open = dao.citizen_count;

    dao.proposal_count += 1;
    Ok(())
}
