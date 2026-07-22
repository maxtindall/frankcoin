// Bundles the miner into one self-contained file the static site can serve.
import { build } from 'esbuild';
import { polyfillNode } from 'esbuild-plugin-polyfill-node';

await build({
  entryPoints: ['src/chain.mjs'],
  bundle: true,
  format: 'iife',
  globalName: 'Frankcoin',
  target: ['es2020'],
  minify: true,
  outfile: 'frankcoin.js',
  plugins: [polyfillNode({ polyfills: { crypto: false } })],
  define: { global: 'globalThis' },
});
console.log('built site/frankcoin.js');
