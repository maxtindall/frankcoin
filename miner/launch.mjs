#!/usr/bin/env node
// One-shot devnet launch: initialize genesis, register, mine one proof.
// Run AFTER `anchor deploy`. Uses the local IDL (no on-chain IDL needed).
import fs from 'fs';
import { keccak256 } from 'js-sha3';
import anchor from '@coral-xyz/anchor';
import { Connection, Keypair, PublicKey, SystemProgram } from '@solana/web3.js';

const rpc = process.env.RPC || 'https://api.devnet.solana.com';
const kp = Keypair.fromSecretKey(Uint8Array.from(JSON.parse(
  fs.readFileSync(process.env.HOME + '/.config/solana/id.json'))));
const idl = JSON.parse(fs.readFileSync('./target/idl/frankcoin.json'));
const PROGRAM_ID = new PublicKey(idl.address);
const TOKEN = new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA');
const ATA_PROG = new PublicKey('ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL');
const DIFFICULTY = parseInt(process.env.DIFFICULTY || '18', 10);
const COOLDOWN = parseInt(process.env.COOLDOWN || '0', 10);

const conn = new Connection(rpc, 'confirmed');
const provider = new anchor.AnchorProvider(conn, new anchor.Wallet(kp), { commitment: 'confirmed' });
const program = new anchor.Program(idl, provider);
const me = kp.publicKey;
const pda = (s) => PublicKey.findProgramAddressSync(s, PROGRAM_ID)[0];
const config = pda([Buffer.from('config')]);
const mint = pda([Buffer.from('mint')]);
const proof = pda([Buffer.from('proof'), me.toBuffer()]);
const ata = PublicKey.findProgramAddressSync([me.toBuffer(), TOKEN.toBuffer(), mint.toBuffer()], ATA_PROG)[0];

function lzb(bytes){let z=0;for(const b of bytes){if(b===0)z+=8;else{z+=Math.clz32(b)-24;break;}}return z;}
function grind(challenge){const head=Buffer.concat([challenge, me.toBuffer()]);const n=Buffer.alloc(8);
  for(let i=0n;;i++){n.writeBigUInt64LE(i);const h=Buffer.from(keccak256.arrayBuffer(Buffer.concat([head,n])));if(lzb(h)>=DIFFICULTY)return i;}}

async function main(){
  console.log('program', PROGRAM_ID.toBase58());
  console.log('wallet ', me.toBase58());

  try {
    const sig = await program.methods.initialize(DIFFICULTY, new anchor.BN(COOLDOWN)).accounts({
      payer: me, config, mint, tokenProgram: TOKEN,
      systemProgram: SystemProgram.programId,
      rent: new PublicKey('SysvarRent111111111111111111111111111111111'),
    }).rpc();
    console.log('GENESIS ok  difficulty', DIFFICULTY, 'cooldown', COOLDOWN, ' sig', sig);
  } catch(e){ console.log('initialize:', String(e.message||e).slice(0,120)); }

  try {
    const sig = await program.methods.register().accounts({
      miner: me, config, proof, systemProgram: SystemProgram.programId }).rpc();
    console.log('REGISTER ok  sig', sig);
  } catch(e){ console.log('register:', String(e.message||e).slice(0,100)); }

  const pr = await program.account.proof.fetch(proof);
  console.log('grinding difficulty', DIFFICULTY, '...');
  const t0 = Date.now();
  const nonce = grind(Buffer.from(pr.challenge));
  console.log('found nonce', nonce.toString(), 'in', ((Date.now()-t0)/1000).toFixed(1)+'s');
  const sig = await program.methods.mine(new anchor.BN(nonce.toString())).accounts({
    miner: me, config, mint, proof, minerAta: ata,
    tokenProgram: TOKEN, associatedTokenProgram: ATA_PROG, systemProgram: SystemProgram.programId }).rpc();
  console.log('FIRST MINE ok  sig', sig);
  const bal = await conn.getTokenAccountBalance(ata);
  console.log('BALANCE', bal.value.uiAmount, 'FRANKS');
  const cfg = await program.account.config.fetch(config);
  console.log('total_minted', (cfg.totalMinted.toNumber()/1e9), 'FRANKS');
}
main().catch(e=>{console.error(String(e).slice(0,300));process.exit(1);});
