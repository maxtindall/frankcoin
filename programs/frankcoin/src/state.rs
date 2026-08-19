use anchor_lang::prelude::*;

/// Global state. The config PDA is also the token's mint authority — no wallet
/// can mint by signing; new franks come only from the `mine` proof-of-work path
/// (plus the General Secretary's single, one-time genesis mint, after which `genesis_minted`
/// latches true forever and even the General Secretary can never mint again).
#[account]
#[derive(InitSpace)]
pub struct Config {
    pub authority_bump: u8,   // bump for the config PDA (= mint authority)
    pub mint_bump: u8,        // bump for the mint PDA
    pub mint: Pubkey,
    pub total_minted: u64,    // base units minted so far (uncapped; grows forever)
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
    // ---- the General Secretary ----
    /// The sitting General Secretary. A constrained steward: may curate the token's identity,
    /// tune the mine's pace within hard bounds, pause mining in emergency, and
    /// pass the office on. May NEVER mint after genesis, NEVER touch a holder's
    /// balance, and NEVER reassign the mint authority away from this program.
    pub general_secretary: Pubkey,
    /// Latches true the instant the General Secretary's one-time genesis mint executes. Once
    /// set, no code path — not even the General Secretary — can mint outside proof-of-work.
    pub genesis_minted: bool,
    /// Emergency brake. While true, `mine` refuses new proofs. The General Secretary alone
    /// may set or clear it; it gates issuance only and can never move a balance.
    pub paused: bool,
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

/// A spent-marker for a treasury withdrawal. One per 0state proposal; its
/// existence (created on withdrawal) prevents a passed spending proposal from
/// being executed more than once.
#[account]
#[derive(InitSpace)]
pub struct Spent {
    pub bump: u8,
    pub proposal: Pubkey,
    pub amount: u64,
    pub ts: i64,
}
