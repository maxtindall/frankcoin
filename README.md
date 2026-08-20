# frankcoin

A proof-of-work **memecoin** on Solana, denominated in **franks**. The synthesis
of what wins on Solana, wrapped around the one thing no other meme has: you
**mine** it. Fixed supply, fair launch, renounced, deflationary.

    program   FJu4SvyPdLYtCmRSgjZi3ShJvoyEPvjdC1MPhz44ngdF   (devnet; mainnet gets a fresh id at launch)
    site      https://frankcoin.website

## What it is

The program holds its own mint authority. No wallet, person, or company can
issue a frank by signing for it — the only way one comes into existence is that
somebody submitted work the program checked and accepted. And it stops: at the
cap, the reward is zero and there will be no more.

| | |
|---|---|
| **Supply** | Hard cap **5,000,000,000 franks**, 9 decimals. Mining halts at the cap; supply lands a hair under and never over. |
| **Issuance** | Mined from zero. No pre-mine, no team allocation, no presale, no admin-mint instruction. |
| **Reward** | 500 franks per accepted proof at genesis, halving each supply tranche and decaying to zero at the cap (a race to the top, Bitcoin-style). |
| **Renounced** | No admin, no steward, no owner. The mint authority is a keyless PDA that dies at the cap; the program's upgrade authority is renounced at launch, so nothing can ever change. |
| **Burn** | Anyone can `burn` their own franks; `total_burned` is tracked on-chain for the deflation scoreboard. |
| **Rate** | One claim per wallet per **cooldown** (300s on devnet). This is what keeps CPU mining viable. |
| **Proof** | `keccak(challenge ‖ miner ‖ nonce_le)` must carry ≥ `difficulty` leading zero bits, with the difficulty retargeting toward a target pace. |

## Instructions

`initialize` · `register` · `mine` · `burn` · `create_metadata`. That's all —
there is no governance, no treasury, no privileged role.

## Layout

    programs/frankcoin/   the Anchor program (Rust) — a buildable, verifiable mirror of what's deployed
    miner/                the node reference miner (mine.mjs)
    mac/                  the Swift CLI + app miner (installed via Homebrew)
    site/                 frankcoin.website
    listing/              Raydium pool-creation script for the mainnet launch
    LISTING.md            the mainnet launch & listing runbook

Reproduce the on-chain build:

    anchor build --ignore-keys

## Mine it

    brew install maxtindall/frankcoin/frankcoin
    frankcoin mine
    brew upgrade maxtindall/frankcoin/frankcoin   # pull updates

Or run the node reference miner in `miner/`. Signing happens locally with your
own Solana keypair — no server, no custody.

## What this is not

frankcoin runs on Solana's **devnet**; these franks are a test network's and are
worth nothing. Nothing here is an offer, a sale, a security, or financial advice.
See [LISTING.md](LISTING.md) for the (audit-gated) path to a real mainnet market.

MIT licensed. *A Max Tindall Inc project.*
