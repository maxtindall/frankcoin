use anchor_lang::prelude::*;

/// franks has 9 decimals (SPL standard).
#[constant]
pub const DECIMALS: u8 = 9;

/// Base units in one franks: 10^9.
pub const ONE_FRANK: u64 = 1_000_000_000;

/// Hard cap: 1,000,000,000 franks. The program can never mint beyond this.
pub const MAX_SUPPLY: u64 = 1_000_000_000 * ONE_FRANK; // 10^18, fits in u64 (max ~1.8e19)

/// Genesis reward per accepted proof, in base units (500 franks). Halves each
/// supply tranche, so the whole halving series sums to exactly MAX_SUPPLY.
pub const INITIAL_REWARD: u64 = 500 * ONE_FRANK;

// PDA seeds
pub const CONFIG_SEED: &[u8] = b"config";
pub const MINT_SEED: &[u8] = b"mint";
pub const PROOF_SEED: &[u8] = b"proof";

/// Unused tail on every Proof account. Rent on this is the cost of an extra
/// mining identity — the only defence against one fast machine farming many
/// wallets. Roughly 0.0157 SOL per registration at current rent rates.
pub const SYBIL_BOND: usize = 900;
