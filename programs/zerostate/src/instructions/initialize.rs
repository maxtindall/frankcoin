use anchor_lang::prelude::*;
use crate::{constants::*, state::Dao};

/// Found the organization. Once. Sets the clock and nothing that grants power.
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub founder: Signer<'info>,

    #[account(
        init,
        payer = founder,
        space = 8 + Dao::INIT_SPACE,
        seeds = [DAO_SEED],
        bump
    )]
    pub dao: Account<'info, Dao>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    let clock = Clock::get()?;
    let dao = &mut ctx.accounts.dao;
    dao.bump = ctx.bumps.dao;
    dao.founder = ctx.accounts.founder.key();
    dao.genesis_ts = clock.unix_timestamp;
    dao.voting_period = DEFAULT_VOTING_PERIOD;
    dao.member_count = 0;
    dao.proposal_count = 0;
    Ok(())
}
