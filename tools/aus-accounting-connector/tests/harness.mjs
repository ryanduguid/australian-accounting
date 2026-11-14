// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

// The built Worker in local workerd through Miniflare, with the rate limiter
// wrangler.toml declares, every outbound request recorded and workerd's own
// output captured. Nothing here reaches the network.
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { Miniflare } from 'miniflare';

export * from './protocol.mjs';
const root = new URL('../', import.meta.url);

// The ratelimits table of wrangler.toml, read from its exact lines.
export async function limiter() {
  const text = await readFile(new URL('wrangler.toml', root), 'utf8');
  const block = text.slice(text.indexOf('[[ratelimits]]'));
  const pairs = block.split(/\r?\n/).map(line => line.trim().split(' = ')).filter(pair => pair.length === 2);
  const lines = new Map(pairs);
  const value = key => JSON.parse(lines.get(key));
  return { name: value('name'), namespace_id: value('namespace_id'), simple: { limit: value('limit'), period: value('period') } };
}

export async function start({ bundle = 'build', limit = true } = {}) {
  const outbound = [];
  const output = [];
  const { name, namespace_id, simple } = await limiter();
  const mf = new Miniflare({
    modules: true,
    scriptPath: fileURLToPath(new URL(`${bundle}/worker/shim.mjs`, root)),
    compatibilityDate: '2026-07-30',
    modulesRules: [
      { type: 'ESModule', include: ['**/*.js'], fallthrough: true },
      { type: 'CompiledWasm', include: ['**/*.wasm'], fallthrough: true },
    ],
    ratelimits: limit ? { [name]: { namespace_id, simple } } : {},
    outboundService: request => {
      outbound.push(request.url);
      return new Response('outbound requests are not allowed', { status: 599 });
    },
    handleRuntimeStdio(stdout, stderr) {
      stdout.on('data', chunk => output.push(String(chunk)));
      stderr.on('data', chunk => output.push(String(chunk)));
    },
  });
  const url = await mf.ready;
  return { mf, url, outbound, output };
}

let address = 0;
// A fresh caller address per request, so the suite stays under the limiter.
export function caller() {
  address += 1;
  return `192.0.2.${address % 250 + 1}`;
}

