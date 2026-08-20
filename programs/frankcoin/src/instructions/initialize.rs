use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};

use crate::{constants::*, state::Config};

/// One-time genesis. Creates the mint and config, and sets the mint authority to
/// the config PDA. From here the only path to new franks is proof-of-work via
/// `mine`, and only until the 5,000,000,000 cap. There is no admin, no steward,
/// no owner — nothing to renounce, because there is nothing to hold.
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
    cfg.total_burned = 0;
    cfg.genesis_ts = Clock::get()?.unix_timestamp;
    cfg.difficulty = difficulty;
    cfg.cooldown = cooldown;
    cfg.proofs_accepted = 0;
    // Difficulty retargeting: the launch difficulty is also the permanent floor.
    cfg.target_interval = TARGET_INTERVAL_SECS;
    cfg.retarget_interval = RETARGET_INTERVAL;
    cfg.window_start_ts = cfg.genesis_ts;
    cfg.window_start_proofs = 0;
    cfg.min_difficulty = difficulty;
    cfg.reserved = [0u8; 64];
    msg!("frankcoin genesis: fixed cap {} base units, difficulty {}, cooldown {}s — mined, renounced",
        SUPPLY_CAP, difficulty, cooldown);
    Ok(())
}
