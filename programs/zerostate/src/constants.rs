use anchor_lang::prelude::*;

/// The frankcoin program. Membership and voting weight are proven by presenting
/// a Proof account owned by this program — mining is the only qualification.
pub const FRANKCOIN_PROGRAM: Pubkey = pubkey!("61yBp4FQSXq6qxS1Scny8LRBNDLDoNQBKupofVSyyHL8");

/// Base units in one frank (frankcoin has 9 decimals).
pub const ONE_FRANK: u64 = 1_000_000_000;

// PDA seeds
pub const DAO_SEED: &[u8] = b"dao";
pub const PROPOSAL_SEED: &[u8] = b"proposal";
pub const BALLOT_SEED: &[u8] = b"ballot";

/// The floor of labour to join: at least one accepted proof. You must have
/// actually mined — the gate reads the Proof account, never a token balance, so
/// holding or being gifted franks cannot buy the franchise.
pub const MIN_PROOFS_TO_JOIN: u64 = 1;

/// How long a member's mining weight takes to halve once they stop mining.
/// Standing tracks *recent* labour: keep mining and your weight holds; stop and
/// it decays toward the base vote. 90 days.
pub const HALF_LIFE_SECS: i64 = 90 * 24 * 60 * 60;

/// A proposal is open for this long.
pub const DEFAULT_VOTING_PERIOD: i64 = 3 * 24 * 60 * 60; // three days

/// Max on-chain title length; the body lives off-chain, pinned by its hash.
pub const MAX_TITLE_LEN: usize = 96;
