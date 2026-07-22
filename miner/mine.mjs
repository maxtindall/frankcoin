#!/usr/bin/env node
// frankcoin CPU miner (reference implementation).
//
// Grinds keccak proofs against the frankcoin program and submits them to mint
// FRANKS to your wallet. Permissionless: this is one miner; anyone can write
// another. Signs with YOUR keypair — no server, no custody.
//
// Usage:
//   node mine.mjs --keypair ~/.config/solana/id.json --rpc https://api.devnet.solana.com
//
// Requires: @solana/web3.js, @coral-xyz/anchor, js-sha3  (npm i in this dir)

import fs from 'fs';
import { keccak256 } from 'js-sha3';
import anchor from '@coral-xyz/anchor';
import { Connection, Keypair, PublicKey, SystemProgram } from '@solana/web3.js';

const PROGRAM_ID = new PublicKey(process.env.FRANK_PROGRAM || 'CosvVR3aNvHcFPtyzZuD385kvBo2aVa3jZapttst1aqY');
const TOKEN = new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA');
const ATA_PROG = new PublicKey('ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL');

function arg(name, def) {
  const i = process.argv.indexOf('--' + name);
  return i >= 0 ? process.argv[i + 1] : def;
}

const rpc = arg('rpc', 'https://api.devnet.solana.com');
const kpPath = arg('keypair', process.env.HOME + '/.config/solana/id.json');
const conn = new Connection(rpc, 'confirmed');
const wallet = Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(kpPath))));
const me = wallet.publicKey;

const pda = (seeds) => PublicKey.findProgramAddressSync(seeds, PROGRAM_ID)[0];
const config = pda([Buffer.from('config')]);
const mint = pda([Buffer.from('mint')]);
const proof = pda([Buffer.from('proof'), me.toBuffer()]);
const ata = PublicKey.findProgramAddressSync(
  [me.toBuffer(), TOKEN.toBuffer(), mint.toBuffer()], ATA_PROG)[0];

// keccak(challenge || miner || nonce_le) -> leading zero bits
function leadingZeroBits(bytes) {
  let z = 0;
  for (const b of bytes) { if (b === 0) z += 8; else { z += Math.clz32(b) - 24; break; } }
  return z;
}
function grind(challenge, difficulty) {
  const head = Buffer.concat([challenge, me.toBuffer()]);
  const nonce = Buffer.alloc(8);
  for (let n = 0n; ; n++) {
    nonce.writeBigUInt64LE(n);
    const h = Buffer.from(keccak256.arrayBuffer(Buffer.concat([head, nonce])));
    if (leadingZeroBits(h) >= difficulty) return n;
  }
}

async function main() {
  console.log('frankcoin miner');
  console.log('  rpc     ', rpc);
  console.log('  wallet  ', me.toBase58());
  console.log('  program ', PROGRAM_ID.toBase58());

  const provider = new anchor.AnchorProvider(conn, new anchor.Wallet(wallet), {});
  const idl = await anchor.Program.fetchIdl(PROGRAM_ID, provider);
  if (!idl) throw new Error('could not fetch IDL — is the program deployed on this cluster?');
  const program = new anchor.Program(idl, provider);

  // register once (ignore error if the proof account already exists)
  try {
    await program.methods.register().accounts({
      miner: me, config, proof, systemProgram: SystemProgram.programId,
    }).rpc();
    console.log('  registered.');
  } catch (e) { console.log('  already registered (or:', String(e.message).slice(0, 60), ')'); }

  const cfg = await program.account.config.fetch(config);
  const difficulty = cfg.difficulty;
  console.log('  difficulty', difficulty, 'bits\n');

  let mined = 0;
  for (;;) {
    const pr = await program.account.proof.fetch(proof);
    const challenge = Buffer.from(pr.challenge);
    process.stdout.write('grinding... ');
    const t0 = Date.now();
    const nonce = grind(challenge, difficulty);
    process.stdout.write(`found nonce ${nonce} in ${((Date.now() - t0) / 1000).toFixed(1)}s — submitting... `);
    try {
      const sig = await program.methods.mine(new anchor.BN(nonce.toString())).accounts({
        miner: me, config, mint, proof, minerAta: ata,
        tokenProgram: TOKEN, associatedTokenProgram: ATA_PROG, systemProgram: SystemProgram.programId,
      }).rpc();
      mined++;
      console.log(`ok (${sig.slice(0, 8)}). total mines this session: ${mined}`);
    } catch (e) {
      const m = String(e.message || e);
      if (/FullyMined/.test(m)) { console.log('\nfrankcoin is fully mined. done.'); break; }
      if (/Cooldown/.test(m)) { console.log('cooldown — waiting.'); await new Promise(r => setTimeout(r, (cfg.cooldown.toNumber?.() || 5) * 1000)); }
      else console.log('submit failed:', m.slice(0, 80));
    }
  }
}
main().catch(e => { console.error(e); process.exit(1); });
