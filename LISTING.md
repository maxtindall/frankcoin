# frankcoin — mainnet launch & listing runbook

How to take frankcoin from a devnet program to a **tradeable memecoin with a live
Raydium market**. This is an engineering runbook, not financial or legal advice,
and not an offer of anything. You execute every money step yourself, with your own
wallet.

---

## 0. Venue reality (read once)

frankcoin is a **Solana SPL token**. That decides where it can list:

| Venue | Chain | frankcoin? |
|---|---|---|
| Uniswap | Ethereum / L2 (EVM) | ❌ wrong chain (needs a bridged ERC-20) |
| PancakeSwap | BNB Chain (EVM) | ❌ wrong chain |
| pump.fun | Solana | ❌ wrong model — it *mints its own* fixed-supply bonding-curve token; you can't import a mined token |
| **Raydium / Orca** | Solana | ✅ **the answer** — permissionless liquidity pool |

"Listing" is not an application. On Raydium you **create a pool** by depositing
franks + a base asset (SOL or USDC); the deposit ratio sets the opening price, and
Jupiter / DexScreener / Birdeye auto-index it within minutes. The only things you
*apply* to — CoinGecko, CoinMarketCap, DexScreener logos — come *after* liquidity
exists, and are cosmetic.

---

## 1. The tokenomics you're launching

The retool (see the program source) makes frankcoin listable:

- **Fixed cap: 5,000,000,000 franks.** Mining halts at the cap; supply lands a
  hair under and never over. No dilution to fight.
- **Fair launch:** no pre-mine, no team allocation, no presale — every frank is
  mined from zero.
- **Renounced:** no admin/steward; mint authority is a keyless PDA that dies at
  the cap; **renounce the program upgrade authority at launch** (step 4) so the
  code is immutable.
- **Deflationary:** anyone can `burn`, and `total_burned` is tracked on-chain for
  a "X% burned forever" scoreboard.
- **Mined, but instantly buyable:** the 99% who won't mine just buy franks from
  the Raydium pool you seed.

---

## 2. The emission question (why a fixed cap makes this work)

At the retarget target of **one proof/minute** (1,440 proofs/day), tranche-0
reward is 500 franks/proof:

| Phase | Reward | New franks/day | Share of the 5B cap/day |
|---|---|---|---|
| Tranche 0 (first 2.5B) | 500 | ~720,000 | ~0.014% |
| Tranche 1 (next 1.25B) | 250 | ~360,000 | ~0.007% |
| … halving each tranche … | | | falling |

Two takeaways:

1. **Distribution is slow and Bitcoin-like** — tranche 0 alone is ~5,000,000
   proofs (~9–10 years at target pace). That's fine: **trading does not wait on
   mining.** You seed a pool from whatever supply exists at launch, and the market
   prices franks from minute one.
2. **Emission is a tiny fraction of a fixed cap** (~0.014%/day early, falling), so
   miner sell-pressure is modest *relative to the pool you seed* — the opposite of
   the old uncapped design, which was unlistable. `INITIAL_REWARD` (500) is the one
   knob: raise it (or lower `TARGET_INTERVAL_SECS`) to distribute faster.

---

## 3. Liquidity sizing

Opening price = (base-asset value in the pool) ÷ (franks in the pool). Pick pool
depth so daily emission is a small slice of the frank side:

> franks in pool  ≥  daily emission ÷ (tolerable %/day)

At 720k franks/day early and a 2%/day tolerance, seed **≥ ~36,000,000 franks**
(0.72% of the cap) plus the matching SOL/USDC value at your target opening price.
Deeper is calmer; shallower is more volatile. Whatever franks you pool must be
**mined first** (there's no other way to get them) — so mine a launch reserve, or
run the pool small and let it grow.

---

## 4. Mainnet launch — step by step

Everything below is on **mainnet-beta**; it costs real SOL and is irreversible.

1. **Deploy the program** (fresh keypair → new program id):
   ```
   solana program deploy target/deploy/frankcoin.so \
     --program-id target/deploy/frankcoin-keypair.json --url mainnet-beta
   ```
2. **Genesis** — `initialize(difficulty, cooldown)` creates the mint + config. Pick
   a launch difficulty that's non-trivial on mainnet hashpower.
3. **Identity** — `create_metadata(name, symbol, uri)` with Frank's name/ticker and
   a logo URI (upload the image + JSON to Arweave/IPFS). Then set the metadata's
   update authority to a wallet you control and, when the identity is final,
   **freeze it** (`is_mutable = false`).
4. **Renounce** — hand the program's upgrade authority to nobody, permanently:
   ```
   solana program set-upgrade-authority <PROGRAM_ID> --final --url mainnet-beta
   ```
   Now nothing about frankcoin can ever change. This is a real trust signal — put
   the tx on the site.
5. **Mine a launch reserve** — mine enough franks to seed the pool (§3).
6. **Create the Raydium pool** — `listing/create-pool.mjs` (§5) or the Raydium UI
   (raydium.io → Liquidity → Create). Deposit franks + SOL/USDC.
7. **Get indexed & submit** — the pool auto-appears on Jupiter / DexScreener /
   Birdeye. Then submit token info + logo to DexScreener, Birdeye, CoinGecko, and
   CoinMarketCap.

---

## 5. Creating the pool from code

`listing/create-pool.mjs` wraps the Raydium SDK v2 CPMM `createPool`. **It is
dry-run by default** (`printSimulate`) — it will not send a transaction unless you
set `CONFIRM=execute`. For most people the **Raydium web UI is safer**; use the
script only if you want it reproducible and you've read it.

```
cd listing && npm install
# dry run — builds and simulates, sends nothing:
FRANK_MINT=<mint> BASE=SOL FRANK_AMOUNT=36000000 BASE_AMOUNT=10 node create-pool.mjs
# for real, on mainnet, with your own funds:
CONFIRM=execute FRANK_MINT=<mint> BASE=SOL FRANK_AMOUNT=36000000 BASE_AMOUNT=10 node create-pool.mjs
```

Verify the exact SDK call against the current Raydium demo before executing with
real funds: https://github.com/raydium-io/raydium-sdk-V2-demo

---

## 6. Cautions

- **Real money, real exposure.** Seeding a pool commits your own SOL/franks at
  market risk. A public tradeable token carries securities/regulatory questions in
  many jurisdictions. Talk to counsel before a real launch.
- **Frank's likeness.** The token is built around a real person's face — have his
  written consent before it's public and tradeable.
- **Immutable means immutable.** After step 4 you cannot patch a bug. Get the
  program audited before mainnet (frankcoin's mainnet is audit-gated).
- Nothing here is financial or investment advice.

---

*Devnet test deployment of the retooled program: program
`FJu4SvyPdLYtCmRSgjZi3ShJvoyEPvjdC1MPhz44ngdF`, mint
`BVEaKRLg7ndqUjU6m2eFTb6jAPbBYu4Emkp2FpnumTed`. Mainnet gets a fresh id + mint.*
