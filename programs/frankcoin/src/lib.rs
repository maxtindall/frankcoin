pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("61yBp4FQSXq6qxS1Scny8LRBNDLDoNQBKupofVSyyHL8");

/// frankcoin — a proof-of-work **memecoin**, denominated in franks, governed by
/// the **General Secretary**.
///
/// Mined from zero and freely traded: franks come from proof-of-work via `mine`,
/// plus a single one-time genesis mint the founding General Secretary performs in the open —
/// after which `genesis_minted` latches and the mint is proof-of-work only,
/// forever. Uncapped, with a decaying reward that floors at a perpetual tail, so
/// percentage inflation trends to zero while mining never stops.
///
/// The General Secretary is the paramount servant of the coin, bound absolutely
/// by the Party line — and here the Party is the code. Democratic centralism,
/// enforced in bytecode: the Secretary may curate the coin's identity, tune the
/// mine's pace within hard bounds, pause mining in an emergency, and pass the
/// office to a successor. The Secretary may **never** mint after genesis,
/// **never** touch a holder's balance, and **never** reassign the mint authority
/// away from this program. Those are not promises; they are the absence of any
/// instruction that could do otherwise. No cult of personality outlives the code.
#[program]
pub mod frankcoin {
    use super::*;

    /// Genesis. Creates the mint (authority = this program's config PDA) and the
    /// global config, and installs the deployer as the founding General Secretary. Callable once.
    pub fn initialize(ctx: Context<Initialize>, difficulty: u8, cooldown: i64) -> Result<()> {
        instructions::initialize::handler(ctx, difficulty, cooldown)
    }

    /// Register a miner (creates their Proof account and starting challenge).
    pub fn register(ctx: Context<Register>) -> Result<()> {
        instructions::register::handler(ctx)
    }

    /// One-time, non-breaking upgrade of a legacy `Config` to the memecoin+General Secretary
    /// layout. Grows the counter account and fills the new fields; no frank
    /// moves. Upgrade-authority gated. Run once, immediately after deploying the
    /// new program to an existing frankcoin, so mining never stops.
    pub fn migrate(ctx: Context<Migrate>) -> Result<()> {
        instructions::migrate::handler(ctx)
    }

    /// Submit a proof-of-work nonce and mint the full reward to the miner.
    pub fn mine(ctx: Context<Mine>, nonce: u64) -> Result<()> {
        instructions::mine::handler(ctx, nonce)
    }

    /// The General Secretary's one and only mint: a single founder allocation, performed once,
    /// after which the mint is proof-of-work only forever. General Secretary-gated.
    pub fn gensec_genesis_mint(ctx: Context<GensecGenesisMint>, amount: u64) -> Result<()> {
        instructions::gensec_genesis_mint::handler(ctx, amount)
    }

    /// Curate the token's Metaplex identity (name/symbol/uri). A curatorial act
    /// of the General Secretary — it can never mint or move a balance. The program signs as
    /// its own mint authority (config PDA).
    pub fn create_metadata(ctx: Context<CreateMetadata>, name: String, symbol: String, uri: String) -> Result<()> {
        instructions::create_metadata::handler(ctx, name, symbol, uri)
    }

    /// Tune the mine's pace — cooldown, target seconds/proof, retarget window —
    /// each clamped to a hard-coded range. General Secretary-gated.
    pub fn gensec_set_params(ctx: Context<GensecSetParams>, cooldown: i64, target_interval: i64, retarget_interval: u64) -> Result<()> {
        instructions::gensec_set_params::handler(ctx, cooldown, target_interval, retarget_interval)
    }

    /// The emergency brake: pause or resume mining. Gates issuance only. General Secretary-gated.
    pub fn gensec_set_paused(ctx: Context<GensecGovern>, paused: bool) -> Result<()> {
        instructions::gensec_govern::set_paused(ctx, paused)
    }

    /// Pass the office to a successor. The powers are fixed by the code, so this
    /// changes only who the General Secretary is. General Secretary-gated.
    pub fn gensec_transfer(ctx: Context<GensecGovern>, new_secretary: Pubkey) -> Result<()> {
        instructions::gensec_govern::transfer(ctx, new_secretary)
    }
}
