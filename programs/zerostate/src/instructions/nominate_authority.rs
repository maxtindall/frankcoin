use anchor_lang::prelude::*;
use crate::{constants::*, error::DaoError, state::Dao};

/// Step one of handing on the one unequal power — who may admit and remove.
/// The current authority NOMINATES a successor. This only records a pending key;
/// it transfers nothing. The power does not move until that key itself accepts,
/// so a nomination to a mistyped or unsignable address is harmless and can be
/// overwritten by simply nominating again (nominate the default/zero key, or
/// the current authority, to cancel).
#[derive(Accounts)]
pub struct NominateAuthority<'info> {
    pub admit_authority: Signer<'info>,

    #[account(
        mut,
        seeds = [DAO_SEED],
        bump = dao.bump,
        has_one = admit_authority @ DaoError::NotTheAuthority
    )]
    pub dao: Account<'info, Dao>,

    /// CHECK: the nominee; only its key is stored, as pending. Not a signer —
    /// it proves it can sign later, in `accept_authority`.
    pub new_authority: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<NominateAuthority>) -> Result<()> {
    ctx.accounts.dao.pending_authority = ctx.accounts.new_authority.key();
    Ok(())
}
