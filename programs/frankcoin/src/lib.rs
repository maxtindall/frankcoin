pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("61yBp4FQSXq6qxS1Scny8LRBNDLDoNQBKupofVSyyHL8");

/// frankcoin — a proof-of-work currency for the open bot project.
/// Denominated in FRANKS. Fully mined from zero, 100,000,000 cap, no pre-mint,
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

    /// One-time: attach Metaplex token metadata (name/symbol/uri) to the mint so
    /// explorers show "frankcoin" instead of a generic SPL token. The program
    /// signs as its own mint authority (config PDA); gated to the upgrade
    /// authority so the identity can't be front-run. Created mutable, to be
    /// frozen at mainnet.
    pub fn create_metadata(ctx: Context<CreateMetadata>, name: String, symbol: String, uri: String) -> Result<()> {
        instructions::create_metadata::handler(ctx, name, symbol, uri)
    }
}
