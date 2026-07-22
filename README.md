# frankcoin

A proof-of-work currency for the open bot project. Denominated in **FRANKS**.

- **Cap:** 1,000,000,000 FRANKS (9 decimals). Enforced on-chain; unexceedable.
- **Issuance:** fully mined from zero. No pre-mint, no founder allocation, no
  admin mint. The program's config PDA holds the mint authority — no wallet can
  mint by signing.
- **Reward:** 500 FRANKS per accepted proof at genesis, halving each supply
  tranche (500M → 250M → 125M …). The series sums to the cap; integer dust near
  the top stays unmined, Bitcoin-style.
- **Proof:** `keccak(challenge || miner || nonce)` must have ≥ `difficulty`
  leading zero bits. Per-miner rolling challenge → every proof is single-use.

## Layout

    programs/frankcoin/       Anchor program (Rust)
      src/instructions/       initialize · register · mine
      src/{state,constants,error}.rs
      tests/test_mine.rs      litesvm: grind a real proof, mine, assert rules
    miner/mine.mjs            reference CPU miner (permissionless; anyone can fork)

## Tests

    anchor build
    cargo test -p frankcoin          # unit (cap/halving math) + integration (mine flow)

All green: cap never exceeded, halving series converges to the cap, stale
proofs rejected, reward = 500 FRANKS/proof at genesis.

## Devnet deploy (safe rehearsal — do this first)

    solana config set --url devnet
    solana-keygen new -o target/deploy/frankcoin-keypair.json   # if not present
    # put the printed program id into declare_id!() and Anchor.toml, then:
    anchor build
    solana airdrop 2
    anchor deploy --provider.cluster devnet

    # genesis: low difficulty + zero cooldown for testing
    #   initialize(difficulty=18, cooldown=0)
    # then mine from any wallet:
    cd miner && npm install
    node mine.mjs --rpc https://api.devnet.solana.com --keypair ~/.config/solana/id.json

## Mainnet — GATED on external audit

Do NOT deploy to mainnet until:
1. The program has a professional third-party audit (a bug = unlimited silent inflation).
2. Difficulty/cooldown chosen for real hashpower.
3. After a monitored burn-in, the **upgrade authority is revoked** — after which
   the program is immutable and genesis is permanent.

Nothing here mints real value until those are done.
