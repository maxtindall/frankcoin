use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};

use crate::{constants::*, state::Config};

/// One-time genesis. Creates the mint and the config, sets the mint authority to
/// the config PDA, and installs the deployer as the founding **General Secretary**.
/// From this instruction on, the only paths to new franks are proof-of-work via
/// `mine` and the General Secretary's single, one-time `gensec_genesis_mint`. There is no
/// standing `mint` admin instruction anywhere in this program.
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + Config::INIT_SPACE,
        seeds = [CONFIG_SEED],
        bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        init,
        payer = payer,
        seeds = [MINT_SEED],
        bump,
        mint::decimals = DECIMALS,
        mint::authority = config,
    )]
    pub mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<Initialize>, difficulty: u8, cooldown: i64) -> Result<()> {
    let cfg = &mut ctx.accounts.config;
    cfg.authority_bump = ctx.bumps.config;
    cfg.mint_bump = ctx.bumps.mint;
    cfg.mint = ctx.accounts.mint.key();
    cfg.total_minted = 0;
    cfg.genesis_ts = Clock::get()?.unix_timestamp;
    cfg.difficulty = difficulty;
    cfg.cooldown = cooldown;
    cfg.proofs_accepted = 0;
    // Difficulty retargeting: begin the first window at genesis. The launch
    // difficulty is also the permanent floor, so retargeting can make mining
    // harder as hashpower grows but never easier than day one.
    cfg.target_interval = TARGET_INTERVAL_SECS;
    cfg.retarget_interval = RETARGET_INTERVAL;
    cfg.window_start_ts = cfg.genesis_ts;
    cfg.window_start_proofs = 0;
    cfg.min_difficulty = difficulty;
    // The General Secretary: the deployer serves first. The office is transferable
    // (`gensec_transfer`) but its powers are fixed by the code, not the holder.
    cfg.general_secretary = ctx.accounts.payer.key();
    cfg.genesis_minted = false;
    cfg.paused = false;
    cfg.reserved = [0u8; 64];
    msg!("frankcoin genesis: uncapped memecoin (tail {} base units/proof), difficulty {}, cooldown {}s, general_secretary {}",
        TAIL_REWARD, difficulty, cooldown, cfg.general_secretary);
    Ok(())
}
