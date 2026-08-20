use anchor_lang::prelude::*;

/// Global state. The config PDA is the token's mint authority — no wallet can
/// mint by signing; new franks come only from the `mine` proof-of-work path, and
/// only until `total_minted` reaches SUPPLY_CAP. There is no admin, no steward,
/// no owner: frankcoin is renounced by construction.
#[account]
#[derive(InitSpace)]
pub struct Config {
    pub authority_bump: u8,   // bump for the config PDA (= mint authority)
    pub mint_bump: u8,        // bump for the mint PDA
    pub mint: Pubkey,
    pub total_minted: u64,    // base units mined so far (halts at SUPPLY_CAP)
    pub total_burned: u64,    // base units burned so far (only ever grows)
    pub genesis_ts: i64,
    pub difficulty: u8,       // current required leading zero bits in a valid proof
    pub cooldown: i64,        // minimum seconds between one miner's claims
    pub proofs_accepted: u64, // telemetry: total successful mines
    // ---- difficulty retargeting ----
    pub target_interval: i64,     // desired seconds per proof, network-wide
    pub retarget_interval: u64,   // proofs per retarget window
    pub window_start_ts: i64,     // timestamp the current window opened
    pub window_start_proofs: u64, // proofs_accepted when the window opened
    pub min_difficulty: u8,       // difficulty floor (the genesis difficulty)
    /// Forward space so future upgrades need no migration.
    pub reserved: [u8; 64],
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
