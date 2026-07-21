pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("CosvVR3aNvHcFPtyzZuD385kvBo2aVa3jZapttst1aqY");

/// frankcoin — a proof-of-work currency for the open bot project.
/// Denominated in FRANK. Fully mined from zero, 100,000,000 cap, no pre-mint,
/// no admin inflation. The program itself is the only issuer.
#[program]
pub mod frankcoin {
    use super::*;

    /// Genesis. Creates the mint (authority = this program's config PDA) and
    /// the global config. Callable once.
    pub fn initialize(ctx: Context<Initialize>, difficulty: u8, cooldown: i64) -> Result<()> {
        instructions::initialize::handler(ctx, difficulty, cooldown)
    }

    /// Register a miner (creates their Proof account and starting challenge).
    pub fn register(ctx: Context<Register>) -> Result<()> {
        instructions::register::handler(ctx)
    }

    /// Submit a proof-of-work nonce and mint the reward.
    pub fn mine(ctx: Context<Mine>, nonce: u64) -> Result<()> {
        instructions::mine::handler(ctx, nonce)
    }
}
