use anchor_lang::prelude::*;

/// FRANK has 9 decimals (SPL standard).
#[constant]
pub const DECIMALS: u8 = 9;

/// Base units in one FRANK: 10^9.
pub const ONE_FRANK: u64 = 1_000_000_000;

/// Hard cap: 100,000,000 FRANK. The program can never mint beyond this.
pub const MAX_SUPPLY: u64 = 100_000_000 * ONE_FRANK; // 10^17, fits in u64

/// Genesis reward per accepted proof, in base units (50 FRANK). Halves each
/// supply tranche, so the whole halving series sums to exactly MAX_SUPPLY.
pub const INITIAL_REWARD: u64 = 50 * ONE_FRANK;

// PDA seeds
pub const CONFIG_SEED: &[u8] = b"config";
pub const MINT_SEED: &[u8] = b"mint";
pub const PROOF_SEED: &[u8] = b"proof";
