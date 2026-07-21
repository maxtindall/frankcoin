use anchor_lang::prelude::*;

/// Global state. The config PDA is also the token's mint authority — no wallet
/// can mint by signing; only this program, via the `mine` proof-of-work path.
#[account]
#[derive(InitSpace)]
pub struct Config {
    pub authority_bump: u8,   // bump for the config PDA (= mint authority)
    pub mint_bump: u8,        // bump for the mint PDA
    pub mint: Pubkey,
    pub total_minted: u64,    // base units minted so far; invariant: <= MAX_SUPPLY
    pub genesis_ts: i64,
    pub difficulty: u8,       // required leading zero bits in a valid proof
    pub cooldown: i64,        // minimum seconds between one miner's claims
    pub proofs_accepted: u64, // telemetry: total successful mines
}

/// Per-miner mining state. The rolling challenge makes each proof single-use.
#[account]
#[derive(InitSpace)]
pub struct Proof {
    pub miner: Pubkey,
    pub challenge: [u8; 32],
    pub last_claim_ts: i64,
    pub total_mined: u64,
    pub count: u64,
    pub bump: u8,
}
