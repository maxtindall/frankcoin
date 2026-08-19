use anchor_lang::prelude::*;

use crate::{constants::*, error::FrankError, state::Config};

/// The two remaining acts of the office: the emergency brake, and succession.
/// Neither can create a frank or move a holder's balance.

#[derive(Accounts)]
pub struct GensecGovern<'info> {
    pub general_secretary: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG_SEED],
        bump = config.authority_bump,
        has_one = general_secretary @ FrankError::NotTheSecretary
    )]
    pub config: Account<'info, Config>,
}

/// Raise or lower the emergency brake. While paused, `mine` refuses new proofs.
/// This gates *issuance* only — it can never touch a balance — and any General Secretary who
/// pauses can unpause.
pub fn set_paused(ctx: Context<GensecGovern>, paused: bool) -> Result<()> {
    ctx.accounts.config.paused = paused;
    msg!("general_secretary {} mining", if paused { "paused" } else { "resumed" });
    Ok(())
}

/// Succession. The General Secretary serves until the office passes to another;
/// here it is simply reassigned to a successor's key. The powers travel with the
/// office and are fixed by the code, so succession changes who the Secretary is,
/// never how much a Secretary may do — the Party line (the code) is supreme.
pub fn transfer(ctx: Context<GensecGovern>, new_secretary: Pubkey) -> Result<()> {
    let cfg = &mut ctx.accounts.config;
    let former = cfg.general_secretary;
    cfg.general_secretary = new_secretary;
    msg!("the office passes: {} -> {}", former, new_secretary);
    Ok(())
}
