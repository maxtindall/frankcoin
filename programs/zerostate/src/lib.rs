pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("BPu5i6U3T69a16TY62J2HBWk7DJMHrU4UHH1Z1GCGmY9");

/// 0state — an autonomous organization governed by the miners of frankcoin.
///
/// franks are a currency: earned, held, and freely traded. The franchise is
/// something else. Membership is acquired only by MINING and is non-transferable,
/// so no accumulation of capital can acquire control of the organization. Money
/// and the vote are held separately.
///
/// Membership is automatic — having mined frankcoin (a Proof account) is the
/// entire qualification; there is no join step and no admitting authority.
/// Voting weight reflects a member's mining, tempered sub-linearly and decaying
/// with inactivity, so influence tracks recent labour, not wealth. Proposals are
/// decided first past the post. This program is the VOTING layer only; it holds
/// no funds.
#[program]
pub mod zerostate {
    use super::*;

    /// Found the organization. Once.
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize::handler(ctx)
    }

    /// Put a question to the membership. Members only. For an ordinary proposal
    /// pass a default recipient and amount 0; for a spending proposal pass the
    /// treasury recipient and amount — if it passes, anyone may execute the
    /// frankcoin `treasury_withdraw` it authorizes.
    pub fn propose(
        ctx: Context<Propose>,
        title: String,
        body_hash: [u8; 32],
        spend_recipient: Pubkey,
        spend_amount: u64,
    ) -> Result<()> {
        instructions::propose::handler(ctx, title, body_hash, spend_recipient, spend_amount)
    }

    /// Cast a vote: 0 no, 1 yes, 2 abstain. Weight is derived from the member's
    /// (decaying) mining, read live from their frankcoin Proof.
    pub fn vote(ctx: Context<Vote>, choice: u8) -> Result<()> {
        instructions::vote::handler(ctx, choice)
    }
}
