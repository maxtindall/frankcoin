use anchor_lang::prelude::*;
use solana_keccak_hasher::hashv;

use crate::{constants::*, state::{Config, Proof}};

/// A miner registers once, creating their Proof account with a starting
/// challenge. One Proof per wallet (PDA seeded by the miner's pubkey).
#[derive(Accounts)]
pub struct Register<'info> {
    #[account(mut)]
    pub miner: Signer<'info>,

    #[account(seeds = [CONFIG_SEED], bump = config.authority_bump)]
    pub config: Account<'info, Config>,

    #[account(
        init,
        payer = miner,
        // 8 discriminator + the struct + SYBIL_BOND of unused tail.
        //
        // Once the cooldown binds, extra hashpower buys nothing: the only way
        // to out-mine a laptop is to run many wallets. Paying rent on this tail
        // makes each additional wallet a real capital lockup rather than dust.
        // The lamports are recoverable by closing the account, so it is a bond
        // and not a fee. Anchor ignores trailing bytes, so the account layout
        // every client parses is unchanged.
        space = 8 + Proof::INIT_SPACE + SYBIL_BOND,
        seeds = [PROOF_SEED, miner.key().as_ref()],
        bump
    )]
    pub proof: Account<'info, Proof>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Register>) -> Result<()> {
    let clock = Clock::get()?;
    let proof = &mut ctx.accounts.proof;
    proof.miner = ctx.accounts.miner.key();
    proof.bump = ctx.bumps.proof;
    proof.last_claim_ts = 0;
    proof.total_mined = 0;
    proof.count = 0;
    // Starting challenge, unique to this miner and this moment.
    proof.challenge = hashv(&[
        &ctx.accounts.config.genesis_ts.to_le_bytes(),
        ctx.accounts.miner.key().as_ref(),
        &clock.slot.to_le_bytes(),
    ]).to_bytes();
    Ok(())
}
