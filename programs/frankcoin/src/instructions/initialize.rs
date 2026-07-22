use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};

use crate::{constants::*, state::Config};

/// One-time genesis. Creates the mint and the config, and — critically — sets
/// the mint authority to the config PDA. From this instruction on, the only
/// path to new FRANKS is proof-of-work via `mine`. There is no `mint` admin
/// instruction anywhere in this program.
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
    msg!("frankcoin genesis: cap {} base units, difficulty {}, cooldown {}s",
        MAX_SUPPLY, difficulty, cooldown);
    Ok(())
}
