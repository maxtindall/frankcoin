use anchor_lang::prelude::*;

use crate::{constants::*, error::FrankError, state::Config};

/// The General Secretary tunes the mine's *pace* — cooldown, the target seconds-per-proof,
/// and the retarget window size — each clamped to a hard-coded range so the
/// dials cannot become a covert emission lever or a way to brick mining. The
/// General Secretary touches the tempo of work, never the money it pays.
#[derive(Accounts)]
pub struct GensecSetParams<'info> {
    pub general_secretary: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG_SEED],
        bump = config.authority_bump,
        has_one = general_secretary @ FrankError::NotTheSecretary
    )]
    pub config: Account<'info, Config>,
}

pub fn handler(
    ctx: Context<GensecSetParams>,
    cooldown: i64,
    target_interval: i64,
    retarget_interval: u64,
) -> Result<()> {
    require!(
        (MIN_COOLDOWN..=MAX_COOLDOWN).contains(&cooldown),
        FrankError::ParamOutOfBounds
    );
    require!(
        (MIN_TARGET_INTERVAL..=MAX_TARGET_INTERVAL).contains(&target_interval),
        FrankError::ParamOutOfBounds
    );
    require!(
        (MIN_RETARGET_INTERVAL..=MAX_RETARGET_INTERVAL).contains(&retarget_interval),
        FrankError::ParamOutOfBounds
    );

    let cfg = &mut ctx.accounts.config;
    cfg.cooldown = cooldown;
    cfg.target_interval = target_interval;
    cfg.retarget_interval = retarget_interval;
    msg!("general_secretary set params: cooldown {}s, target {}s/proof, window {} proofs",
        cooldown, target_interval, retarget_interval);
    Ok(())
}
