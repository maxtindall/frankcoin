use anchor_lang::prelude::*;
use crate::{constants::*, error::DaoError, state::Dao};

/// Step two: the nominated key itself signs to take the door. Because the
/// recipient must sign, a nomination to an unsignable or mistyped key can never
/// take effect — the power stays with the current authority until a real key
/// claims it. This is what makes the transfer un-brickable.
#[derive(Accounts)]
pub struct AcceptAuthority<'info> {
    pub new_authority: Signer<'info>,

    #[account(
        mut,
        seeds = [DAO_SEED],
        bump = dao.bump,
        constraint = dao.pending_authority == new_authority.key() @ DaoError::NotThePendingAuthority
    )]
    pub dao: Account<'info, Dao>,
}

pub fn handler(ctx: Context<AcceptAuthority>) -> Result<()> {
    let dao = &mut ctx.accounts.dao;
    dao.admit_authority = dao.pending_authority;
    dao.pending_authority = Pubkey::default();
    Ok(())
}
