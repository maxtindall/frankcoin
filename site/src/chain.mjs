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
const CAP = 1000000000n * BigInt(ONE_FRANK);

function pdas(programId) {
  const pid = new PublicKey(programId);
  const seed = (s) => new TextEncoder().encode(s);
  return {
    programId: pid,
    config: PublicKey.findProgramAddressSync([seed('config')], pid)[0],
    mint: PublicKey.findProgramAddressSync([seed('mint')], pid)[0],
  };
}

/** What the next accepted proof pays. Mirrors reward_for() in the program. */
export function rewardFor(totalMinted) {
  let minted = BigInt(totalMinted), tranche = CAP / 2n, reward = 500n * BigInt(ONE_FRANK);
  while (minted >= tranche && reward > 0n) { minted -= tranche; tranche /= 2n; reward /= 2n; }
  return Number(reward) / ONE_FRANK;
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
    cap: Number(CAP) / ONE_FRANK,
  };
}
