pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("FJu4SvyPdLYtCmRSgjZi3ShJvoyEPvjdC1MPhz44ngdF");

/// frankcoin — a proof-of-work **memecoin**. Named for Frank.
///
/// The synthesis of what wins on Solana, wrapped around the one thing no other
/// meme has: you **mine** it. Fixed supply — a hard cap of 5,000,000,000 franks,
/// ever — so it can be priced and traded without endless dilution. **Fair
/// launch:** no pre-mine, no team allocation, no presale; every frank is mined
/// from zero by anyone with a laptop, a race to the cap. **Renounced:** no admin,
/// no steward, no owner — the mint authority is a keyless PDA and dies at the
/// cap, and the upgrade authority is renounced at launch, so nothing can ever
/// change. **Deflationary:** anyone can `burn`, and the burn is counted, so
/// circulating supply only shrinks. And it is **instantly tradeable** — a Raydium
/// pool at launch — so the 99% who will never mine can just buy Frank.
#[program]
pub mod frankcoin {
    use super::*;

    /// Genesis. Creates the mint (authority = this program's config PDA) and the
    /// global config. Callable once. No steward is installed — there isn't one.
    pub fn initialize(ctx: Context<Initialize>, difficulty: u8, cooldown: i64) -> Result<()> {
        instructions::initialize::handler(ctx, difficulty, cooldown)
    }

    /// Register a miner (creates their Proof account and starting challenge).
    pub fn register(ctx: Context<Register>) -> Result<()> {
        instructions::register::handler(ctx)
    }

    /// Submit a proof-of-work nonce and mint the full reward to the miner. Halts
    /// forever once the 5,000,000,000 cap is reached.
    pub fn mine(ctx: Context<Mine>, nonce: u64) -> Result<()> {
        instructions::mine::handler(ctx, nonce)
    }

    /// Burn your own franks — deflation, counted on-chain for the scoreboard.
    pub fn burn(ctx: Context<BurnFranks>, amount: u64) -> Result<()> {
        instructions::burn::handler(ctx, amount)
    }

    /// One-time: attach Metaplex metadata (name/symbol/uri = Frank) to the mint.
    /// The program signs as its own mint authority; gated to the upgrade authority
    /// so the identity can't be front-run, then frozen and renounced at launch.
    pub fn create_metadata(ctx: Context<CreateMetadata>, name: String, symbol: String, uri: String) -> Result<()> {
        instructions::create_metadata::handler(ctx, name, symbol, uri)
    }
}
