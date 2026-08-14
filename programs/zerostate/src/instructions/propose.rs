use anchor_lang::prelude::*;
use frankcoin::state::Proof;
use crate::{constants::*, error::DaoError, state::{Dao, Proposal}};

/// Put a question to the membership. Members only — membership is automatic:
/// the proposer's frankcoin Proof (having mined) is the qualification.
#[derive(Accounts)]
pub struct Propose<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,

    #[account(mut, seeds = [DAO_SEED], bump = dao.bump)]
    pub dao: Account<'info, Dao>,

    /// The proposer's frankcoin Proof — proof of membership (having mined).
    #[account(
        seeds = [frankcoin::constants::PROOF_SEED, proposer.key().as_ref()],
        bump = proof.bump,
        seeds::program = FRANKCOIN_PROGRAM,
        constraint = proof.miner == proposer.key() @ DaoError::ProofOwnerMismatch,
    )]
    pub proof: Account<'info, Proof>,

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

pub fn handler(
    ctx: Context<Propose>,
    title: String,
    body_hash: [u8; 32],
    spend_recipient: Pubkey,
    spend_amount: u64,
) -> Result<()> {
    require!(title.len() <= MAX_TITLE_LEN, DaoError::TitleTooLong);
    require!(ctx.accounts.proof.count >= MIN_PROOFS_TO_JOIN, DaoError::InsufficientLabour);

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
    p.electorate_at_open = dao.member_count;
    p.spend_recipient = spend_recipient;
    p.spend_amount = spend_amount;

    dao.proposal_count += 1;
    Ok(())
}
