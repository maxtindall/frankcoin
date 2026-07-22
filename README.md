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
    mac/                      the miner — a macOS app, the only way to mine
      Sources/FrankMinerCore/ keccak · base58 · ed25519 PDAs · tx signing · the engine
      Sources/FrankMiner/     the SwiftUI app
      build.sh                builds "frankcoin miner.app"
    miner/launch.mjs          deployment tooling: genesis + the first proof
    site/                     0state.website — a monitor, not a miner
      src/chain.mjs           read-only: supply, proofs, difficulty, reward
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

## Mining

Mining happens in a **macOS application, on the miner's own machine, tied to
the miner's own wallet.** That is the only miner this project ships.

    cd mac && ./build.sh          # builds build/frankcoin miner.app

It needs nothing but the Swift toolchain from the Xcode Command Line Tools —
no Xcode project, and no third-party packages. Everything the app needs is
written out and tested here: keccak-256, base58, ed25519 curve arithmetic for
program-derived addresses, transaction encoding, and signing. The wallet
keypair is loaded from a file you choose, stays on the machine, and signs every
proof locally.

The app refuses to run in a virtual machine or over SSH. Be clear-eyed about
what that is worth: a VM is detectable and a headless session is detectable,
but a *rented physical Mac* is not — it is real hardware with a real display
session. Those checks raise the effort. They do not make cloud mining
impossible and are not described here as though they do.

**On distribution:** the app is ad-hoc signed, not notarised. macOS will refuse
it on first launch until you open it from the right-click menu, or clear it in
System Settings → Privacy & Security. Proper notarisation needs a paid Apple
Developer account.

## The site

    cd site && npm install && npm run build

`site/` is static and **read-only**: it reports supply, proofs, difficulty and
reward, and offers the download. It has no wallet connection and nothing to
sign, so a visitor risks nothing by looking. It cannot mine, by construction —
the browser client was removed rather than merely hidden.

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

MIT licensed. The Mac app is the only miner that exists — unless somebody
builds another, which the protocol permits and nothing prevents. That is a
property of a permissionless chain, not an oversight: the program accepts a
valid proof from any signer, which is exactly what makes *no person can mint*
true.

*A Max Tindall Inc project.*
