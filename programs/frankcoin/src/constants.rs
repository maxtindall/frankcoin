use anchor_lang::prelude::*;

/// franks has 9 decimals (SPL standard).
#[constant]
pub const DECIMALS: u8 = 9;

/// Base units in one frank: 10^9.
pub const ONE_FRANK: u64 = 1_000_000_000;

/// The size of the distribution phase, in base units (1,000,000,000 franks).
/// This is NOT a cap — mining never stops. It only shapes the halving schedule:
/// the reward halves across tranches that together span this much supply, after
/// which emission settles to a perpetual tail. Roughly a billion franks are
/// issued on the steep part of the curve; then issuance continues forever at
/// TAIL_REWARD.
pub const DISTRIBUTION_PHASE: u64 = 1_000_000_000 * ONE_FRANK;

/// Genesis reward per accepted proof, in base units (500 franks). Halves each
/// supply tranche across the distribution phase, then floors at TAIL_REWARD.
pub const INITIAL_REWARD: u64 = 500 * ONE_FRANK;

/// The perpetual tail: once the halving schedule decays to it, every accepted
/// proof mints exactly this much — 1 frank — forever. Emission never stops, but
/// because the tail is a *fixed absolute* amount, percentage inflation falls
/// toward zero as total supply grows (Monero's model). This is what lets an
/// uncapped memecoin stay sound: asymptotically-zero inflation plus a
/// proof-of-work production cost under the price.
pub const TAIL_REWARD: u64 = 1 * ONE_FRANK;

// ---- Difficulty retargeting -------------------------------------------------
// Difficulty floats toward a target issuance *pace*, so franks are minted on a
// predictable schedule regardless of how much hashpower shows up, and the
// marginal cost to produce one rises as miners compete.

/// Target seconds between accepted proofs, network-wide. Retargeting nudges
/// difficulty so the observed pace tracks this.
pub const TARGET_INTERVAL_SECS: i64 = 60;

/// Retarget every this many accepted proofs (one difficulty-adjustment window).
pub const RETARGET_INTERVAL: u64 = 20;

/// Difficulty moves by at most ±1 bit per window, and only when the observed
/// pace is off target by more than 2×, so it can't oscillate wildly. It never
/// falls below the genesis difficulty (stored as `min_difficulty`) and never
/// rises past this ceiling (a valid nonce stays findable within a u64).
pub const MAX_DIFFICULTY: u8 = 56;

// ---- The General Secretary's bounded dials -----------------------------------------------
// The General Secretary may tune the mine's *pace*, never its money. These bounds
// are hard-coded so no General Secretary — however capricious — can brick mining or turn the
// dials into a covert emission lever. The Council of code binds the General Secretary.

/// A General Secretary-set cooldown must fall within [MIN_COOLDOWN, MAX_COOLDOWN] seconds.
pub const MIN_COOLDOWN: i64 = 0;
pub const MAX_COOLDOWN: i64 = 24 * 60 * 60; // one day

/// A General Secretary-set target interval must fall within these bounds (seconds/proof).
pub const MIN_TARGET_INTERVAL: i64 = 1;
pub const MAX_TARGET_INTERVAL: i64 = 60 * 60; // one hour

/// A General Secretary-set retarget window must fall within these bounds (proofs).
pub const MIN_RETARGET_INTERVAL: u64 = 1;
pub const MAX_RETARGET_INTERVAL: u64 = 10_000;

// PDA seeds
pub const CONFIG_SEED: &[u8] = b"config";
pub const MINT_SEED: &[u8] = b"mint";
pub const PROOF_SEED: &[u8] = b"proof";

/// The 0state governance program. The treasury can only be spent by executing a
/// passed spending proposal owned by this program.
pub const ZEROSTATE_PROGRAM: Pubkey = pubkey!("BPu5i6U3T69a16TY62J2HBWk7DJMHrU4UHH1Z1GCGmY9");

/// Anchor's 8-byte account discriminator for a 0state `Proposal`. Used to verify
/// a supplied account really is a proposal before decoding it by hand (frankcoin
/// avoids a code dependency on zerostate to keep the two programs decoupled).
pub const PROPOSAL_DISCRIMINATOR: [u8; 8] = [26, 94, 189, 187, 116, 136, 53, 33];

/// Unused tail on every Proof account. Rent on this is the cost of an extra
/// mining identity — the only defence against one fast machine farming many
/// wallets. Roughly 0.0157 SOL per registration at current rent rates.
pub const SYBIL_BOND: usize = 900;
