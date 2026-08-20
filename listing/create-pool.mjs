// Create a Raydium CPMM pool for frankcoin on Solana mainnet.
//
// ⚠️  THIS SPENDS REAL MONEY. It deposits your franks + SOL/USDC as liquidity.
//     It is DRY-RUN by default (simulates, sends nothing). It only sends a real
//     transaction when you set  CONFIRM=execute.  Read it before you run it, and
//     verify the SDK call against the current Raydium demo:
//     https://github.com/raydium-io/raydium-sdk-V2-demo/blob/master/src/cpmm/createCpmmPool.ts
//
// Usage:
//   cd listing && npm install
//   FRANK_MINT=<mint> BASE=SOL FRANK_AMOUNT=36000000 BASE_AMOUNT=10 node create-pool.mjs        # dry run
//   CONFIRM=execute FRANK_MINT=<mint> BASE=SOL FRANK_AMOUNT=36000000 BASE_AMOUNT=10 node create-pool.mjs
//
// Env:
//   FRANK_MINT    the frankcoin mint address (required)
//   BASE          SOL | USDC        (the other side of the pool; default SOL)
//   FRANK_AMOUNT  franks to deposit, human units (e.g. 36000000)
//   BASE_AMOUNT   SOL/USDC to deposit, human units (e.g. 10)
//   RPC           mainnet RPC (default https://api.mainnet-beta.solana.com)
//   KEYPAIR       path to your wallet keypair (default ~/.config/solana/id.json)
//   CONFIRM       set to "execute" to actually send; anything else = dry run

import fs from 'fs';
import BN from 'bn.js';
import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import {
  Raydium, TxVersion, CREATE_CPMM_POOL_PROGRAM, CREATE_CPMM_POOL_FEE_ACC, printSimulate,
} from '@raydium-io/raydium-sdk-v2';

const RPC = process.env.RPC || 'https://api.mainnet-beta.solana.com';
const KEYPAIR = (process.env.KEYPAIR || process.env.HOME + '/.config/solana/id.json').replace(/^~/, process.env.HOME);
const FRANK_MINT = process.env.FRANK_MINT;
const BASE = (process.env.BASE || 'SOL').toUpperCase();
const FRANK_AMOUNT = Number(process.env.FRANK_AMOUNT);
const BASE_AMOUNT = Number(process.env.BASE_AMOUNT);
const EXECUTE = process.env.CONFIRM === 'execute';

const FRANK_DECIMALS = 9;
const BASES = {
  SOL:  { mint: 'So11111111111111111111111111111111111111112', decimals: 9 },
  USDC: { mint: 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v', decimals: 6 },
};

function die(m) { console.error('create-pool: ' + m); process.exit(1); }
if (!FRANK_MINT) die('FRANK_MINT is required');
if (!BASES[BASE]) die('BASE must be SOL or USDC');
if (!(FRANK_AMOUNT > 0) || !(BASE_AMOUNT > 0)) die('FRANK_AMOUNT and BASE_AMOUNT must be positive');

const toBase = (human, decimals) => new BN(Math.round(human * 10 ** decimals).toString());

const owner = Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(KEYPAIR))));
const connection = new Connection(RPC, 'confirmed');

console.log('=== frankcoin → Raydium CPMM pool ===');
console.log('  mode        ', EXECUTE ? '⚠️  EXECUTE (will send a real tx)' : 'dry run (simulate only)');
console.log('  wallet      ', owner.publicKey.toBase58());
console.log('  frank mint  ', FRANK_MINT);
console.log('  base        ', BASE, BASES[BASE].mint);
console.log('  deposit     ', FRANK_AMOUNT.toLocaleString(), 'franks  +', BASE_AMOUNT, BASE);
console.log('  opening px  ~', (BASE_AMOUNT / FRANK_AMOUNT), BASE, 'per frank\n');

const raydium = await Raydium.load({
  owner, connection, cluster: 'mainnet',
  disableFeatureCheck: true, blockhashCommitment: 'finalized',
});

const mintA = await raydium.token.getTokenInfo(FRANK_MINT);
const mintB = await raydium.token.getTokenInfo(BASES[BASE].mint);
const feeConfigs = await raydium.api.getCpmmConfigs();

const { execute, transaction, extInfo } = await raydium.cpmm.createPool({
  programId: CREATE_CPMM_POOL_PROGRAM,        // mainnet CPMM program
  poolFeeAccount: CREATE_CPMM_POOL_FEE_ACC,   // mainnet fee account
  mintA,
  mintB,
  mintAAmount: toBase(FRANK_AMOUNT, FRANK_DECIMALS),
  mintBAmount: toBase(BASE_AMOUNT, BASES[BASE].decimals),
  startTime: new BN(0),
  feeConfig: feeConfigs[0],
  associatedOnly: false,
  ownerInfo: { useSOLBalance: true },
  txVersion: TxVersion.V0,
});

if (!EXECUTE) {
  console.log('DRY RUN — simulating, sending nothing. Set CONFIRM=execute to create the pool.');
  await printSimulate([transaction]);
  process.exit(0);
}

console.log('EXECUTING — creating the pool on mainnet…');
const { txId } = await execute({ sendAndConfirm: true });
console.log('pool created. tx:', txId);
console.log('pool keys:', JSON.stringify(extInfo?.address ?? {}, null, 2));
