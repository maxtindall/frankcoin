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
pub const TREASURY_SEED: &[u8] = b"treasury";
pub const SPENT_SEED: &[u8] = b"spent";

/// The DAO levy: 1 frank in every 10 mined is routed to the treasury. The
/// reward is divided by this to get the treasury's cut (10 = 10%). Spent only
/// by 0state proposal and vote.
pub const TREASURY_BPS_DIVISOR: u64 = 10;

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
