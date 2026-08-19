/**
 * frankcoin — read-only chain client for the site.
 *
 * The site reports; it does not mine. Grinding, registering and submitting
 * proofs live in the Mac app and nowhere else, deliberately: mining is meant to
 * happen on the miner's own machine, tied to their own wallet. There is no
 * wallet connection here and nothing to sign, so a visitor risks nothing by
 * looking.
 */
import { Connection, PublicKey } from '@solana/web3.js';

export const PROGRAM_ID = '61yBp4FQSXq6qxS1Scny8LRBNDLDoNQBKupofVSyyHL8';
const ONE_FRANK = 1e9;
// NOT a cap. frankcoin is uncapped: the reward halves
// across this distribution phase, then floors at a perpetual 1-frank tail —
// emission never stops.
const DISTRIBUTION_PHASE = 1000000000n * BigInt(ONE_FRANK);
const TAIL = 1n * BigInt(ONE_FRANK);

function pdas(programId) {
  const pid = new PublicKey(programId);
  const seed = (s) => new TextEncoder().encode(s);
  return {
    programId: pid,
    config: PublicKey.findProgramAddressSync([seed('config')], pid)[0],
    mint: PublicKey.findProgramAddressSync([seed('mint')], pid)[0],
  };
}

/** What the next accepted proof pays. Mirrors reward_for() in the program:
 * halves each tranche across the distribution phase, then **floors at the tail
 * (1 frank) forever** — it never returns 0, because mining is uncapped. */
export function rewardFor(totalMinted) {
  const minted = BigInt(totalMinted);
  let reward = 500n * BigInt(ONE_FRANK), lo = 0n, size = DISTRIBUTION_PHASE / 2n;
  while (true) {
    if (reward <= TAIL) return Number(TAIL) / ONE_FRANK;
    const hi = lo + size;
    if (minted < hi) return Number(reward) / ONE_FRANK;
    lo = hi; size /= 2n; reward /= 2n;
  }
}

/** The chain's position: supply, proofs, difficulty, reward. */
export async function state({ rpcUrl, programId = PROGRAM_ID }) {
  const conn = new Connection(rpcUrl, 'confirmed');
  const p = pdas(programId);
  const [cfg] = await conn.getMultipleAccountsInfo([p.config]);
  if (!cfg) return { deployed: false, programId: p.programId.toBase58() };

  const d = new Uint8Array(cfg.data);
  const v = new DataView(d.buffer, d.byteOffset, d.byteLength);
  const totalMinted = v.getBigUint64(42, true);
  return {
    deployed: true,
    programId: p.programId.toBase58(),
    mint: new PublicKey(d.slice(10, 42)).toBase58(),
    difficulty: d[58],
    cooldown: Number(v.getBigInt64(59, true)),
    totalMinted: Number(totalMinted) / ONE_FRANK,
    proofsAccepted: Number(v.getBigUint64(67, true)),
    nextReward: rewardFor(totalMinted),
    uncapped: true,
    distributionPhase: Number(DISTRIBUTION_PHASE) / ONE_FRANK,
  };
}
