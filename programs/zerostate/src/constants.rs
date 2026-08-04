use anchor_lang::prelude::*;

/// The frankcoin program. A citizen proves their labour by presenting a Proof
/// account owned by this program; that is the only qualification to vote.
pub const FRANKCOIN_PROGRAM: Pubkey = pubkey!("61yBp4FQSXq6qxS1Scny8LRBNDLDoNQBKupofVSyyHL8");

// PDA seeds
pub const DAO_SEED: &[u8] = b"dao";
pub const CITIZEN_SEED: &[u8] = b"citizen";
pub const PROPOSAL_SEED: &[u8] = b"proposal";
pub const BALLOT_SEED: &[u8] = b"ballot";

/// The floor of labour that makes a miner a citizen: at least one accepted
/// proof. You must have actually mined — holding franks you were given is not
/// enough, and cannot be, because the gate reads the Proof account, not a token
/// balance. Power comes from work, never from wealth.
pub const MIN_PROOFS_TO_JOIN: u64 = 1;

/// A proposal is open for this long. Kept in the config so the DAO could, in
/// principle, propose to change its own clock later.
pub const DEFAULT_VOTING_PERIOD: i64 = 3 * 24 * 60 * 60; // three days

/// Bounds, so a proposal cannot carry an unbounded amount of on-chain text.
/// The body itself lives off-chain; the chain keeps a title and a hash of it.
pub const MAX_TITLE_LEN: usize = 96;
