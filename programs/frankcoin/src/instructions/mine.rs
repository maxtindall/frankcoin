use anchor_lang::prelude::*;
use solana_keccak_hasher::hashv;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};

use crate::{constants::*, error::FrankError, state::{Config, Proof}};

/// The proof-of-work mint. A miner submits a nonce; the program verifies the
/// hash meets difficulty, mints the full reward to the miner, and rolls the
/// challenge forward so the same proof can never be reused. No levy, no treasury:
/// every mined frank goes to the miner. Mining halts forever at SUPPLY_CAP.
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

    let difficulty = ctx.accounts.config.difficulty;
    let cooldown = ctx.accounts.config.cooldown;
    let authority_bump = ctx.accounts.config.authority_bump;
    let total_minted = ctx.accounts.config.total_minted;
    let challenge = ctx.accounts.proof.challenge;
    let last_claim_ts = ctx.accounts.proof.last_claim_ts;

    // 1. The cap. Once 5,000,000,000 franks are mined the reward is zero and
    //    mining is over, permanently — fixed supply, like a memecoin should be.
    let reward = reward_for(total_minted);
    require!(reward > 0, FrankError::FullyMined);

    // 2. Cooldown.
    require!(
        clock.unix_timestamp >= last_claim_ts.saturating_add(cooldown),
        FrankError::Cooldown
    );

    // 3. Verify the proof: keccak(challenge || miner || nonce) meets difficulty.
    let hash = hashv(&[
        &challenge,
        ctx.accounts.miner.key().as_ref(),
        &nonce.to_le_bytes(),
    ]);
    require!(
        leading_zero_bits(&hash.to_bytes()) >= difficulty as u32,
        FrankError::InsufficientDifficulty
    );

    // 4. Mint the whole reward to the miner.
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

    // 5. Update state.
    let cfg = &mut ctx.accounts.config;
    cfg.total_minted = cfg.total_minted.checked_add(reward).ok_or(FrankError::Overflow)?;
    cfg.proofs_accepted = cfg.proofs_accepted.saturating_add(1);

    // 5a. Difficulty retarget: once a window of proofs has been accepted, nudge
    //     difficulty ±1 bit if the pace is off target by more than 2×. Never below
    //     the genesis floor, never above the ceiling.
    if cfg.retarget_interval > 0
        && cfg.proofs_accepted.saturating_sub(cfg.window_start_proofs) >= cfg.retarget_interval
    {
        let elapsed = clock.unix_timestamp.saturating_sub(cfg.window_start_ts).max(0);
        let expected = cfg.target_interval.saturating_mul(cfg.retarget_interval as i64);
        if elapsed.saturating_mul(2) < expected && cfg.difficulty < MAX_DIFFICULTY {
            cfg.difficulty += 1;
        } else if elapsed > expected.saturating_mul(2) && cfg.difficulty > cfg.min_difficulty {
            cfg.difficulty -= 1;
        }
        cfg.window_start_ts = clock.unix_timestamp;
        cfg.window_start_proofs = cfg.proofs_accepted;
    }

    let proof = &mut ctx.accounts.proof;
    proof.last_claim_ts = clock.unix_timestamp;
    proof.total_mined = proof.total_mined.checked_add(reward).ok_or(FrankError::Overflow)?;
    proof.count = proof.count.saturating_add(1);

    // 6. Roll the challenge forward: anti-replay and anti-precompute.
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

/// The emission curve. Starts at INITIAL_REWARD (500 franks) and halves once per
/// supply tranche across the cap — tranche 0 spans the first half of the cap at
/// 500/proof, tranche 1 the next quarter at 250, and so on — decaying to **zero
/// at SUPPLY_CAP**. The last proof mints only what remains, so supply lands
/// exactly on the cap and never a frank over.
pub fn reward_for(total_minted: u64) -> u64 {
    if total_minted >= SUPPLY_CAP {
        return 0;
    }
    let mut reward = INITIAL_REWARD;
    let mut lo: u64 = 0;
    let mut size: u64 = SUPPLY_CAP / 2; // tranche 0 spans the first half of the cap
    loop {
        if reward == 0 {
            return 0;
        }
        let hi = lo.saturating_add(size);
        if total_minted < hi {
            let remaining = SUPPLY_CAP - total_minted;
            return reward.min(remaining); // never mint past the cap
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
        assert_eq!(reward_for(SUPPLY_CAP / 2 - ONE_FRANK), INITIAL_REWARD);
    }

    #[test]
    fn reward_halves_each_tranche() {
        assert_eq!(reward_for(SUPPLY_CAP / 2), INITIAL_REWARD / 2);
        assert_eq!(reward_for(SUPPLY_CAP / 2 + SUPPLY_CAP / 4), INITIAL_REWARD / 4);
    }

    #[test]
    fn emission_is_capped_and_stops() {
        assert_eq!(reward_for(SUPPLY_CAP), 0);
        assert_eq!(reward_for(SUPPLY_CAP + ONE_FRANK), 0);
        assert_eq!(reward_for(u64::MAX / 2), 0);
    }

    #[test]
    fn supply_never_exceeds_the_cap() {
        // The core invariant: from any point, minting the current reward never
        // carries total supply past the cap. (Emission decays to zero a hair
        // under the cap, so it lands just below and never a frank over.)
        for &t in &[0u64, SUPPLY_CAP / 2, SUPPLY_CAP * 3 / 4, SUPPLY_CAP - ONE_FRANK, SUPPLY_CAP - 1] {
            let r = reward_for(t);
            assert!(t.saturating_add(r) <= SUPPLY_CAP, "overshoot at {}", t);
        }
        assert_eq!(reward_for(SUPPLY_CAP), 0);
    }

    #[test]
    fn leading_zero_bits_counts_correctly() {
        let mut h = [0u8; 32];
        assert_eq!(leading_zero_bits(&h), 256);
        h[0] = 0b0000_1000;
        assert_eq!(leading_zero_bits(&h), 4);
        h[0] = 0;
        h[1] = 0b1000_0000;
        assert_eq!(leading_zero_bits(&h), 8);
    }
}
