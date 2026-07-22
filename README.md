# frankcoin

A proof-of-work currency on Solana, denominated in **FRANKS**. Live on devnet;
mined, never sold.

    program   CosvVR3aNvHcFPtyzZuD385kvBo2aVa3jZapttst1aqY   (devnet)
    site      https://0state.website

## What it is

The program holds its own mint authority. No wallet, person or company can
issue a FRANK by signing for it — the only way one comes into existence is that
somebody submitted work the program checked and accepted.

| | |
|---|---|
| **Supply** | 1,000,000,000 FRANKS, 9 decimals. Enforced on-chain; unexceedable. |
| **Issuance** | Mined from zero. No pre-mine, no allocation, no admin-mint instruction. |
| **Reward** | 500 FRANKS per accepted proof at genesis, halving each supply tranche (500M → 250M → 125M …). The series sums to the cap; integer dust near the top stays unmined, Bitcoin-style. |
| **Proof** | `keccak(challenge ‖ miner ‖ nonce_le)` must carry ≥ `difficulty` leading zero bits. |
| **Replay** | Each miner holds a rolling challenge that advances on every accepted proof. |

Replay protection is a consequence of the design rather than a feature bolted
on: there is no list of spent nonces to maintain, because against a new
challenge an old nonce is simply a bad guess.

## Layout

    programs/frankcoin/       the Anchor program (Rust)
      src/instructions/       initialize · register · mine
      src/{state,constants,error}.rs
      tests/test_mine.rs      litesvm — grind a real proof, mine, assert the rules
      tests/test_js_parity.rs Rust and JavaScript must agree on the hash
    miner/                    reference CPU miner (Node) and the devnet launch script
    site/                     0state.website — the browser miner
      src/miner.mjs           the whole client: grind, read state, register, mine
      index.html              the page
      frankcoin.js            built bundle (committed, so the site is static)

## Tests

    anchor build
    cargo test -p frankcoin

Covers the cap, the halving series, rejection of stale proofs, the reward at
genesis, and — importantly — that the Rust program and the browser miner
compute the same digest for the same input. If those two ever drifted apart,
every browser-mined proof would be rejected on-chain and the failure would be
silent and baffling.

## The site

    cd site && npm install && npm run build

`site/` is static: any host that serves files will do. The page holds no keys
and takes no cut; every transaction is signed by the visitor's own wallet. And
nobody needs the page at all — `miner/mine.mjs` mines from a terminal, and the
protocol is open to any implementation.

## Deploying

Devnet, which is where this currently lives:

    solana config set --url devnet
    anchor build
    solana program deploy target/deploy/frankcoin.so \
      --program-id target/deploy/frankcoin-keypair.json --url devnet
    node miner/launch.mjs            # genesis + register + first proof

Genesis on devnet is `difficulty 18, cooldown 0` — a proof lands in well under
a second. That is right for a test network and completely wrong for a real one.

## Mainnet — gated

Not deployed to mainnet, and it should not be until:

1. **A third-party audit.** A bug in this program is unlimited silent inflation.
2. **Difficulty and cooldown chosen for real hashpower**, against a deliberate
   issuance schedule. Nothing observed on devnet informs that number.
3. **The upgrade authority is revoked** after a monitored burn-in, after which
   the program is immutable and genesis is permanent.

## What this is not

frankcoin is an artwork about issuing money. It is given away, never sold.
There is no offer, no sale, no fundraising and no investment here, and nothing
in this repository or on the site is financial advice. Devnet FRANKS are test
tokens: they cannot be sold and are worth nothing.

MIT licensed — fork it, read it, write a faster miner.

*A Max Tindall Inc project.*
