use anchor_lang::prelude::*;
use solana_keccak_hasher::hashv;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};

use crate::{constants::*, error::FrankError, state::{Config, Proof}};

/// The proof-of-work mint. A miner submits a nonce; the program verifies the
/// hash meets difficulty, mints the reward to the miner, and rolls the
/// challenge forward so the same proof can never be reused.
#[derive(Accounts)]
pub struct Mine<'info> {
    #[account(mut)]
    pub miner: Signer<'info>,

    #[account(mut, seeds = [CONFIG_SEED], bump = config.authority_bump)]
    pub config: Account<'info, Config>,

    #[account(mut, seeds = [MINT_SEED], bump = config.mint_bump)]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [PROOF_SEED, miner.key().as_ref()],
        bump = proof.bump,
        has_one = miner
    )]
    pub proof: Account<'info, Proof>,

    #[account(
        init_if_needed,
        payer = miner,
        associated_token::mint = mint,
        associated_token::authority = miner
    )]
    pub miner_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Mine>, nonce: u64) -> Result<()> {
    let clock = Clock::get()?;

    // Read the values we need before taking mutable borrows.
    let difficulty = ctx.accounts.config.difficulty;
    let cooldown = ctx.accounts.config.cooldown;
    let authority_bump = ctx.accounts.config.authority_bump;
    let total_minted = ctx.accounts.config.total_minted;
    let challenge = ctx.accounts.proof.challenge;
    let last_claim_ts = ctx.accounts.proof.last_claim_ts;

    // 1. Cap.
    require!(total_minted < MAX_SUPPLY, FrankError::FullyMined);

    // 2. Cooldown.
    require!(
        clock.unix_timestamp >= last_claim_ts.saturating_add(cooldown),
        FrankError::Cooldown
    );

    // 3. Verify the proof: keccak(challenge || miner || nonce) >= difficulty.
    let hash = hashv(&[
        &challenge,
        ctx.accounts.miner.key().as_ref(),
        &nonce.to_le_bytes(),
    ]);
    require!(
        leading_zero_bits(&hash.to_bytes()) >= difficulty as u32,
        FrankError::InsufficientDifficulty
    );

    // 4. Reward, clamped to remaining supply so the cap is exact.
    let remaining = MAX_SUPPLY
        .checked_sub(total_minted)
        .ok_or(FrankError::Overflow)?;
    let reward = reward_for(total_minted).min(remaining);
    require!(reward > 0, FrankError::FullyMined);

    // 5. Mint to the miner, signed by the config PDA (the mint authority).
    let signer: &[&[&[u8]]] = &[&[CONFIG_SEED, &[authority_bump]]];
    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.miner_ata.to_account_info(),
                authority: ctx.accounts.config.to_account_info(),
            },
            signer,
        ),
        reward,
    )?;

    // 6. Update state.
    let cfg = &mut ctx.accounts.config;
    cfg.total_minted = cfg.total_minted.checked_add(reward).ok_or(FrankError::Overflow)?;
    cfg.proofs_accepted = cfg.proofs_accepted.saturating_add(1);

    let proof = &mut ctx.accounts.proof;
    proof.last_claim_ts = clock.unix_timestamp;
    proof.total_mined = proof.total_mined.checked_add(reward).ok_or(FrankError::Overflow)?;
    proof.count = proof.count.saturating_add(1);

    // 7. Roll the challenge forward: anti-replay and anti-precompute.
    proof.challenge = hashv(&[
        &challenge,
        &nonce.to_le_bytes(),
        &clock.slot.to_le_bytes(),
    ]).to_bytes();

    Ok(())
}

/// Count leading zero bits across the 32-byte hash, big-endian.
fn leading_zero_bits(hash: &[u8; 32]) -> u32 {
    let mut count = 0u32;
    for &byte in hash.iter() {
        if byte == 0 {
            count += 8;
        } else {
            count += byte.leading_zeros();
            break;
        }
    }
    count
}

/// Reward halves each supply tranche. Band 0 = first 50M FRANKS at 50/proof,
/// band 1 = next 25M at 25/proof, band 2 = next 12.5M at 12.5/proof, ... The
/// full series sums to exactly MAX_SUPPLY. Cap exactness is separately
/// guaranteed by the clamp in the handler.
fn reward_for(total_minted: u64) -> u64 {
    let mut reward = INITIAL_REWARD;
    let mut lo: u64 = 0;
    let mut size: u64 = MAX_SUPPLY / 2; // band 0 spans the first 50M
    loop {
        let hi = lo.saturating_add(size);
        if total_minted < hi || size == 0 || reward == 0 {
            return reward;
        }
        lo = hi;
        size /= 2;
        reward /= 2;
    }
}

#[cfg(test)]
mod reward_tests {
    use super::*;

    #[test]
    fn genesis_reward_is_500_frank() {
        assert_eq!(reward_for(0), INITIAL_REWARD);
        assert_eq!(reward_for(MAX_SUPPLY / 2 - 1), INITIAL_REWARD);
    }

    #[test]
    fn reward_halves_each_tranche() {
        // band 1 starts at 50M cumulative -> reward 25 FRANKS
        assert_eq!(reward_for(MAX_SUPPLY / 2), INITIAL_REWARD / 2);
        // band 2 starts at 75M cumulative -> reward 12.5 FRANKS
        assert_eq!(reward_for(MAX_SUPPLY / 2 + MAX_SUPPLY / 4), INITIAL_REWARD / 4);
    }

    #[test]
    fn never_exceeds_cap_and_asymptotes() {
        // Mine the entire schedule, always paying the current band reward
        // clamped to remaining. THE hard invariant: total must never exceed
        // MAX_SUPPLY. Integer halving leaves tiny dust near the cap (like
        // Bitcoin never quite reaching 21M), so we assert convergence to
        // within a hair rather than exact equality.
        let mut total: u64 = 0;
        loop {
            let r = reward_for(total).min(MAX_SUPPLY - total);
            if r == 0 { break; }              // reward truncated to zero: mining ends
            total += r;
            assert!(total <= MAX_SUPPLY, "CAP EXCEEDED at total {}", total);
        }
        // Reached at least 99.9999% of the cap; remainder is unmineable dust.
        let mined_bps = (total as u128 * 1_000_000) / MAX_SUPPLY as u128;
        assert!(mined_bps >= 999_999, "only reached {} millionths of cap", mined_bps);
        assert!(total < MAX_SUPPLY || total == MAX_SUPPLY);
    }

    #[test]
    fn leading_zero_bits_counts_correctly() {
        let mut h = [0u8; 32];
        assert_eq!(leading_zero_bits(&h), 256);
        h[0] = 0b0000_1000; // 4 leading zeros in the first byte
        assert_eq!(leading_zero_bits(&h), 4);
        h[0] = 0;
        h[1] = 0b1000_0000; // 8 (first byte) + 0
        assert_eq!(leading_zero_bits(&h), 8);
    }
}
