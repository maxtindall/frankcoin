/**
 * frankcoin browser miner.
 *
 * Everything needed to find a proof and spend it, and nothing else. The
 * program has two instructions worth calling, so the instructions are encoded
 * by hand rather than pulling in Anchor's client — the whole bundle stays
 * small enough to serve from a static host.
 *
 * This file never sees a private key. Every transaction is handed to the
 * visitor's wallet to sign.
 */
import { keccak256 } from 'js-sha3';
import {
  Connection, PublicKey, Transaction, TransactionInstruction, SystemProgram,
} from '@solana/web3.js';

export const PROGRAM_ID = 'CosvVR3aNvHcFPtyzZuD385kvBo2aVa3jZapttst1aqY';

// First eight bytes of sha256("global:<instruction>"). Fixed for the program's life.
const IX_REGISTER = Uint8Array.from([211, 124, 67, 15, 211, 194, 178, 240]);
const IX_MINE = Uint8Array.from([59, 22, 178, 213, 139, 197, 160, 196]);

const TOKEN_PROGRAM = new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA');
const ATA_PROGRAM = new PublicKey('ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL');
const ONE_FRANK = 1e9;
const CAP = 1000000000n * BigInt(ONE_FRANK);

/* ---------------------------------------------------------------- work --- */

/** keccak(challenge ‖ miner ‖ nonce_le) -> count of leading zero bits. */
export function proofBits(challenge, miner, nonce) {
  const buf = new Uint8Array(challenge.length + miner.length + 8);
  buf.set(challenge, 0);
  buf.set(miner, challenge.length);
  new DataView(buf.buffer, challenge.length + miner.length, 8)
    .setBigUint64(0, BigInt(nonce), true);
  const h = new Uint8Array(keccak256.arrayBuffer(buf));
  let z = 0;
  for (const b of h) { if (b === 0) z += 8; else { z += Math.clz32(b) - 24; break; } }
  return z;
}

/** The digest itself, as hex — so a page can show the work being attempted. */
export function digest(challenge, miner, nonce) {
  const buf = new Uint8Array(challenge.length + miner.length + 8);
  buf.set(challenge, 0);
  buf.set(miner, challenge.length);
  new DataView(buf.buffer, challenge.length + miner.length, 8)
    .setBigUint64(0, BigInt(nonce), true);
  return keccak256(buf);
}

/**
 * Search for a nonce meeting `difficulty`. Cooperative: yields to the page
 * every `slice` hashes so the tab stays usable, and stops when asked.
 * Resolves to {nonce, hashes, seconds}, or null if it was stopped.
 */
export async function grind({ challenge, miner, difficulty, shouldStop, onProgress,
                              slice = 20000, start = 0 }) {
  const c = challenge instanceof Uint8Array ? challenge : new Uint8Array(challenge);
  const m = miner instanceof Uint8Array ? miner : new Uint8Array(miner);
  const t0 = Date.now();
  let n = BigInt(start), hashes = 0;
  for (;;) {
    for (let i = 0; i < slice; i++, n++, hashes++) {
      if (proofBits(c, m, n) >= difficulty) {
        return { nonce: n.toString(), hashes, seconds: (Date.now() - t0) / 1000 };
      }
    }
    if (shouldStop && shouldStop()) return null;
    if (onProgress) onProgress({ hashes, seconds: (Date.now() - t0) / 1000 });
    await new Promise((r) => setTimeout(r, 0));
  }
}

/* --------------------------------------------------------------- chain --- */

export function pubkeyBytes(address) {
  return new Uint8Array(new PublicKey(address).toBytes());
}

function pdas(programId, miner) {
  const pid = new PublicKey(programId);
  const seed = (s) => new TextEncoder().encode(s);
  const config = PublicKey.findProgramAddressSync([seed('config')], pid)[0];
  const mint = PublicKey.findProgramAddressSync([seed('mint')], pid)[0];
  const out = { programId: pid, config, mint };
  if (miner) {
    const m = new PublicKey(miner);
    out.proof = PublicKey.findProgramAddressSync([seed('proof'), m.toBuffer()], pid)[0];
    out.ata = PublicKey.findProgramAddressSync(
      [m.toBuffer(), TOKEN_PROGRAM.toBuffer(), mint.toBuffer()], ATA_PROGRAM)[0];
  }
  return out;
}

/** What the next accepted proof pays. Mirrors reward_for() in the program. */
export function rewardFor(totalMinted) {
  let minted = BigInt(totalMinted), tranche = CAP / 2n, reward = 500n * BigInt(ONE_FRANK);
  while (minted >= tranche && reward > 0n) { minted -= tranche; tranche /= 2n; reward /= 2n; }
  return Number(reward) / ONE_FRANK;
}

/** The whole picture in one round trip: the chain's position and yours. */
export async function state({ rpcUrl, programId = PROGRAM_ID, miner }) {
  const conn = new Connection(rpcUrl, 'confirmed');
  const p = pdas(programId, miner);
  const infos = await conn.getMultipleAccountsInfo(miner ? [p.config, p.proof] : [p.config]);
  if (!infos[0]) return { deployed: false, registered: false, programId: p.programId.toBase58() };

  const d = new Uint8Array(infos[0].data);
  const v = new DataView(d.buffer, d.byteOffset, d.byteLength);
  const totalMinted = v.getBigUint64(42, true);
  const out = {
    deployed: true,
    programId: p.programId.toBase58(),
    mint: new PublicKey(d.slice(10, 42)).toBase58(),
    difficulty: d[58],
    cooldown: Number(v.getBigInt64(59, true)),
    totalMinted: Number(totalMinted) / ONE_FRANK,
    proofsAccepted: Number(v.getBigUint64(67, true)),
    nextReward: rewardFor(totalMinted),
    cap: Number(CAP) / ONE_FRANK,
    registered: false,
  };
  if (miner && infos[1]) {
    const q = new Uint8Array(infos[1].data);
    const qv = new DataView(q.buffer, q.byteOffset, q.byteLength);
    out.registered = true;
    out.challenge = q.slice(40, 72);
    out.lastClaimTs = Number(qv.getBigInt64(72, true));
    out.mined = Number(qv.getBigUint64(80, true)) / ONE_FRANK;
    out.proofs = Number(qv.getBigUint64(88, true));
  }
  return out;
}

async function send(provider, conn, ix, feePayer) {
  const tx = new Transaction().add(ix);
  tx.feePayer = feePayer;
  tx.recentBlockhash = (await conn.getLatestBlockhash('confirmed')).blockhash;
  const signed = await provider.signTransaction(tx);
  const sig = await conn.sendRawTransaction(signed.serialize());
  const bh = await conn.getLatestBlockhash('confirmed');
  await conn.confirmTransaction({ signature: sig, ...bh }, 'confirmed');
  return sig;
}

/** Once per wallet: creates the Proof account holding your own challenge. */
export async function register({ provider, rpcUrl, programId = PROGRAM_ID }) {
  const conn = new Connection(rpcUrl, 'confirmed');
  const me = new PublicKey(provider.publicKey.toString());
  const p = pdas(programId, me);
  return send(provider, conn, new TransactionInstruction({
    programId: p.programId,
    keys: [
      { pubkey: me, isSigner: true, isWritable: true },
      { pubkey: p.config, isSigner: false, isWritable: false },
      { pubkey: p.proof, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: IX_REGISTER,
  }), me);
}

/** Spend a found nonce. The program checks the work, mints, rolls the challenge. */
export async function mine({ provider, rpcUrl, programId = PROGRAM_ID, nonce }) {
  const conn = new Connection(rpcUrl, 'confirmed');
  const me = new PublicKey(provider.publicKey.toString());
  const p = pdas(programId, me);
  const data = new Uint8Array(16);
  data.set(IX_MINE, 0);
  new DataView(data.buffer).setBigUint64(8, BigInt(nonce), true);
  return send(provider, conn, new TransactionInstruction({
    programId: p.programId,
    keys: [
      { pubkey: me, isSigner: true, isWritable: true },
      { pubkey: p.config, isSigner: false, isWritable: true },
      { pubkey: p.mint, isSigner: false, isWritable: true },
      { pubkey: p.proof, isSigner: false, isWritable: true },
      { pubkey: p.ata, isSigner: false, isWritable: true },
      { pubkey: TOKEN_PROGRAM, isSigner: false, isWritable: false },
      { pubkey: ATA_PROGRAM, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data,
  }), me);
}

/** Wallets that inject themselves into the page, in preference order. */
export function wallets() {
  const cands = [
    [window.solflare, 'Solflare'],
    [window.phantom && window.phantom.solana, 'Phantom'],
    [window.backpack, 'Backpack'],
    [window.glow || window.glowSolana, 'Glow'],
    [window.braveSolana, 'Brave'],
    [window.exodus && window.exodus.solana, 'Exodus'],
    [window.solana, (window.solana && window.solana.isPhantom) ? 'Phantom' : 'Solana wallet'],
  ];
  const seen = [], out = [];
  for (const [p, name] of cands) {
    if (!p || typeof p.connect !== 'function' || seen.includes(p)) continue;
    seen.push(p); out.push({ provider: p, name });
  }
  return out;
}
