// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

// Wall time of each conformance case in local workerd: the first call in a new
// Worker (instantiation, catalogue parse and schema compile included), then the
// median of 20 warm calls. Each time includes the Node-to-workerd round trip,
// measured separately as a warm 404. Local wall time is not Cloudflare's CPU
// accounting. Writes JSON to stdout.
import { readFile } from 'node:fs/promises';
import { performance } from 'node:perf_hooks';
import { MODERN, headers, message, start } from './harness.mjs';

const cases = JSON.parse(await readFile(new URL('../../../apps/aus-accounting-mcp/conformance/cases.json', import.meta.url), 'utf8')).cases;
const median = values => [...values].sort((a, b) => a - b)[Math.floor(values.length / 2)];
const round = value => Math.round(value * 1000) / 1000;

async function timed(worker, body, path = '/mcp') {
  const begin = performance.now();
  const response = await worker.mf.dispatchFetch(new URL(path, worker.url), { method: 'POST', body: JSON.stringify(body), headers: { ...headers(body, MODERN), 'cf-connecting-ip': '192.0.2.1' } });
  await response.text();
  return performance.now() - begin;
}

const rows = [];
let overhead;
let startup;
for (const item of cases) {
  const begin = performance.now();
  const worker = await start();
  const ready = performance.now() - begin;
  const body = message(MODERN, 1, 'tools/call', { name: item.tool, arguments: item.arguments });
  const first = await timed(worker, body);
  const warm = [];
  for (let i = 0; i < 20; i += 1) warm.push(await timed(worker, body));
  if (!overhead) {
    const trips = [];
    for (let i = 0; i < 20; i += 1) trips.push(await timed(worker, body, '/not-found'));
    overhead = median(trips);
    const fresh = await start();
    const discoverBegin = performance.now();
    await timed(fresh, message(MODERN, 1, 'server/discover'));
    startup = { miniflare_ready_ms: round(ready), first_discover_ms: round(performance.now() - discoverBegin) };
    await fresh.mf.dispose();
  }
  rows.push({ case: item.id, tool: item.tool, first_ms: round(first), warm_median_ms: round(median(warm)) });
  await worker.mf.dispose();
}
console.log(JSON.stringify({ round_trip_404_warm_median_ms: round(overhead), startup, rows }, null, 2));
