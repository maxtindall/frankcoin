use anchor_lang::prelude::*;
use crate::constants::MAX_TITLE_LEN;

/// The organization's record. One per deployment. Holds only the rules of the
/// vote — no treasury, no admin power, no way to mint a vote.
#[account]
#[derive(InitSpace)]
pub struct Dao {
    pub bump: u8,
    /// Who founded it (recorded; grants no special power).
    pub founder: Pubkey,
    pub genesis_ts: i64,
    pub voting_period: i64,
    pub member_count: u64,
    pub proposal_count: u64,
    /// Forward space so the singleton can grow without a migration.
    pub reserved: [u8; 64],
}

/// A question put to the membership. Full text lives off-chain; the chain keeps
/// a title and the hash that pins it. Tallies are weighted sums.
#[account]
#[derive(InitSpace)]
pub struct Proposal {
    pub bump: u8,
    pub id: u64,
    pub proposer: Pubkey,
    #[max_len(MAX_TITLE_LEN)]
    pub title: String,
    pub body_hash: [u8; 32],
    pub created_ts: i64,
    pub closes_ts: i64,
    pub yes: u64,
    pub no: u64,
    pub abstain: u64,
    /// Members enrolled at the moment this opened — a stable denominator for
    /// turnout, independent of who joins later.
    pub electorate_at_open: u64,
    /// If this is a spending proposal, the recipient of the treasury franks and
    /// the amount (base units). A default (zero) recipient with amount 0 marks
    /// an ordinary proposal that moves no funds. When such a proposal passes,
    /// anyone may execute the frankcoin `treasury_withdraw` it authorizes.
    pub spend_recipient: Pubkey,
    pub spend_amount: u64,
}

/// One ballot per member per proposal. Its existence prevents a second vote, and
/// it records the weight that was applied for a full audit trail.
#[account]
#[derive(InitSpace)]
pub struct Ballot {
    pub bump: u8,
    pub proposal: Pubkey,
    pub member: Pubkey,
    pub choice: u8, // 0 = no, 1 = yes, 2 = abstain
    pub weight: u64,
    pub cast_ts: i64,
}
