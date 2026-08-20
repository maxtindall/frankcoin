use anchor_lang::prelude::*;

/// franks has 9 decimals (SPL standard).
#[constant]
pub const DECIMALS: u8 = 9;

/// Base units in one frank: 10^9.
pub const ONE_FRANK: u64 = 1_000_000_000;

/// The hard cap: **5,000,000,000 franks**, ever. frankcoin is a memecoin now —
/// fixed supply, so it can be priced and traded without endless dilution.
/// Mining is a race to this cap: once `total_minted` reaches it, the reward is
/// zero and `mine` refuses. No tail, no perpetual emission, no exceptions.
pub const SUPPLY_CAP: u64 = 5_000_000_000 * ONE_FRANK;

/// Genesis reward per accepted proof (500 franks). Halves once per supply tranche
/// across the cap — tranche 0 spans the first half of the cap at 500/proof,
/// tranche 1 the next quarter at 250, and so on — decaying to zero exactly as the
/// cap is reached. No pre-mine and no team allocation: every frank is mined.
///
/// This is the one knob that sets how fast the 5B distributes. At the retarget
/// target of one proof/minute the early reward pays 500 franks/proof; raise this
/// (or lower TARGET_INTERVAL_SECS) to distribute faster, lower it to distribute
/// slower. Trading does not wait on full distribution — a Raydium pool is seeded
/// at launch — so this only shapes the mining race, not liquidity.
pub const INITIAL_REWARD: u64 = 500 * ONE_FRANK;

// ---- Difficulty retargeting -------------------------------------------------
// Difficulty floats toward a target issuance *pace*, so franks mint on a
// predictable schedule regardless of hashpower, and the marginal cost to produce
// one rises as miners compete.

/// Target seconds between accepted proofs, network-wide.
pub const TARGET_INTERVAL_SECS: i64 = 60;

/// Retarget every this many accepted proofs (one difficulty-adjustment window).
pub const RETARGET_INTERVAL: u64 = 20;

/// Difficulty moves at most ±1 bit per window, never below the genesis floor,
/// never above this ceiling (a valid nonce stays findable within a u64).
pub const MAX_DIFFICULTY: u8 = 56;

// PDA seeds
pub const CONFIG_SEED: &[u8] = b"config";
pub const MINT_SEED: &[u8] = b"mint";
pub const PROOF_SEED: &[u8] = b"proof";

/// Unused tail on every Proof account. Rent on this is the cost of an extra
/// mining identity — the only defence against one machine farming many wallets.
/// Roughly 0.0157 SOL per registration at current rent rates.
pub const SYBIL_BOND: usize = 900;
