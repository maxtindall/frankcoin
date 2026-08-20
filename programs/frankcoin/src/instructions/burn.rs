use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount};

use crate::{constants::*, error::FrankError, state::Config};

/// Burn franks — send them to the void, forever. Anyone may burn their **own**
/// franks (the caller signs as the token authority), and the config keeps a
/// running `total_burned` so the site can show how much supply is gone for good.
/// This is the memecoin's deflation lever: with a fixed cap and a growing burn,
/// circulating supply only shrinks. Community burn campaigns get a scoreboard.
#[derive(Accounts)]
pub struct BurnFranks<'info> {
    pub burner: Signer<'info>,

    #[account(mut, seeds = [CONFIG_SEED], bump = config.authority_bump)]
    pub config: Account<'info, Config>,

    #[account(mut, seeds = [MINT_SEED], bump = config.mint_bump)]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = burner
    )]
    pub burner_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<BurnFranks>, amount: u64) -> Result<()> {
    require!(ctx.accounts.burner_ata.amount >= amount, FrankError::InsufficientBalance);

    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.key(),
            Burn {
                mint: ctx.accounts.mint.to_account_info(),
                from: ctx.accounts.burner_ata.to_account_info(),
                authority: ctx.accounts.burner.to_account_info(),
            },
        ),
        amount,
    )?;

    let cfg = &mut ctx.accounts.config;
    cfg.total_burned = cfg.total_burned.checked_add(amount).ok_or(FrankError::Overflow)?;
    msg!("burned {} base units; {} burned forever", amount, cfg.total_burned);
    Ok(())
}
