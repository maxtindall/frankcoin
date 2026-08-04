use anchor_lang::prelude::*;
use crate::{constants::*, error::DaoError, state::{Dao, Citizen}};

/// Remove a member. The symmetric half of a trusted commune: the authority that
/// holds the door can also close it. Closing the citizen account returns its
/// rent to the authority and ends the member's franchise.
#[derive(Accounts)]
pub struct Revoke<'info> {
    #[account(mut)]
    pub admit_authority: Signer<'info>,

    #[account(mut, seeds = [DAO_SEED], bump = dao.bump, has_one = admit_authority @ DaoError::NotTheAuthority)]
    pub dao: Account<'info, Dao>,

    /// CHECK: only the key is used, to locate the citizen account.
    pub member: UncheckedAccount<'info>,

    #[account(
        mut,
        close = admit_authority,
        seeds = [CITIZEN_SEED, member.key().as_ref()],
        bump = citizen.bump,
    )]
    pub citizen: Account<'info, Citizen>,
}

pub fn handler(ctx: Context<Revoke>) -> Result<()> {
    ctx.accounts.dao.citizen_count = ctx.accounts.dao.citizen_count.saturating_sub(1);
    Ok(())
}
