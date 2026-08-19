use anchor_lang::prelude::*;
use anchor_lang::system_program;

use crate::{constants::*, error::FrankError, state::Config};

/// One-time, non-breaking migration for a `Config` created before difficulty
/// retargeting and the General Secretary existed. It grows the account to the current layout
/// and fills in the new fields, carrying every existing value forward untouched.
///
/// **No token balances are involved.** Franks live in the SPL mint, which this
/// never touches; only the program's own counter account changes shape. So an
/// existing, already-trading frankcoin can be upgraded to the memecoin+General Secretary
/// program and keep mining without a single frank moving, freezing, or being at
/// risk. Gated to the program's upgrade authority, and idempotent (re-running on
/// an up-to-date account is a no-op error).
///
/// The account is loaded raw (`UncheckedAccount`) because its current bytes
/// predate the new `Config` and cannot be deserialized as the new struct; the
/// existing fields sit at fixed offsets, read and rewritten directly.
///
/// On an already-distributed coin the migration sets `genesis_minted = true`:
/// the premine window is closed. It would be neither safe nor fair to inject a
/// founder allocation into a supply that is already mined and traded, so the
/// General Secretary inherits every steward power EXCEPT the one-time mint, which is spent.
#[derive(Accounts)]
pub struct Migrate<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: verified by the config PDA seeds; loaded raw (see above).
    #[account(mut, seeds = [CONFIG_SEED], bump)]
    pub config: UncheckedAccount<'info>,

    // ---- upgrade-authority gate: only the program's upgrade authority may call.
    #[account(constraint = program.programdata_address()? == Some(program_data.key()))]
    pub program: Program<'info, crate::program::Frankcoin>,
    #[account(
        constraint = program_data.upgrade_authority_address == Some(authority.key())
            @ FrankError::NotUpgradeAuthority
    )]
    pub program_data: Account<'info, ProgramData>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Migrate>) -> Result<()> {
    let ai = ctx.accounts.config.to_account_info();
    let new_len = 8 + Config::INIT_SPACE;
    let old_len = ai.data_len();
    require!(old_len < new_len, FrankError::AlreadyMigrated);

    // Read the values we carry forward, at their fixed byte offsets in the
    // legacy (75-byte) account: difficulty @58, proofs_accepted @67..75.
    let (difficulty, proofs_accepted) = {
        let data = ai.try_borrow_data()?;
        let difficulty = data[58];
        let proofs_accepted = u64::from_le_bytes(data[67..75].try_into().unwrap());
        (difficulty, proofs_accepted)
    };

    // Fund the extra rent, then grow the account (new bytes zeroed).
    let rent = Rent::get()?;
    let needed = rent.minimum_balance(new_len);
    let current = ai.lamports();
    if needed > current {
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.key(),
                system_program::Transfer {
                    from: ctx.accounts.authority.to_account_info(),
                    to: ai.clone(),
                },
            ),
            needed - current,
        )?;
    }
    ai.resize(new_len)?; // zero-inits the new bytes

    // Write the appended fields at their new-layout offsets. Retargeting begins
    // its first window now; the existing difficulty becomes the permanent floor.
    // The General Secretary is the migrating upgrade authority; the premine window is closed.
    let now = Clock::get()?.unix_timestamp;
    let general_secretary = ctx.accounts.authority.key();
    let mut data = ai.try_borrow_mut_data()?;
    data[75..83].copy_from_slice(&TARGET_INTERVAL_SECS.to_le_bytes());   // target_interval
    data[83..91].copy_from_slice(&RETARGET_INTERVAL.to_le_bytes());      // retarget_interval
    data[91..99].copy_from_slice(&now.to_le_bytes());                   // window_start_ts
    data[99..107].copy_from_slice(&proofs_accepted.to_le_bytes());       // window_start_proofs
    data[107] = difficulty;                                              // min_difficulty (floor)
    data[108..140].copy_from_slice(general_secretary.as_ref());                       // general_secretary
    data[140] = 1;                                                       // genesis_minted = true
    data[141] = 0;                                                       // paused = false
    // reserved[64] @142..206 stays zeroed by realloc.

    msg!("frankcoin config migrated to memecoin+General Secretary layout ({} -> {} bytes); general_secretary {}; premine window closed",
        old_len, new_len, general_secretary);
    Ok(())
}
