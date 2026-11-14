// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

// Check a running connector with fabricated inputs only: the 32 conformance
// cases, compared byte for byte with the native server's recorded results.
//   node live-check.mjs <endpoint URL> [expected release, such as aus-accounting-connector/v0.1.0]
// It sends 34 requests and exits non-zero on the first wrong answer.
import { readFile } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';
import { MODERN, headers, message, metaOf, resultText, servedBy } from './tests/protocol.mjs';

const TOOLS = ['calc_payday_super_deadline', 'get_div7a_benchmark_rate', 'review_div7a_loan', 'refuse_div7a', 'calculate_tax_worksheet', 'get_data_freshness'];

export async function check(endpoint, release, request = fetch) {
  const url = new URL(endpoint);
  const local = ['localhost', '127.0.0.1', '[::1]'].includes(url.hostname);
  if (url.protocol !== 'https:' && !(local && url.protocol === 'http:')) throw new Error(`refusing ${url.protocol} for a remote endpoint`);
  const reference = JSON.parse(await readFile(new URL('./tests/fixtures/reference.json', import.meta.url), 'utf8'));
  const cases = JSON.parse(await readFile(new URL('../../apps/aus-accounting-mcp/conformance/cases.json', import.meta.url), 'utf8')).cases;
  const post = async (id, method, params) => {
    const body = message(MODERN, id, method, params);
    const response = await request(url, { method: 'POST', body: JSON.stringify(body), headers: headers(body, MODERN) });
    const text = await response.text();
    if (response.status !== 200) throw new Error(`${method} ${params?.name ?? ''}: HTTP ${response.status}`);
    return { response, result: resultText(text, id) };
  };
  const discover = await post('discover', 'server/discover');
  const named = discover.response.headers.get('x-connector-release') ?? '';
  if (release && !named.startsWith(`${release};`)) throw new Error(`release is '${named}', expected ${release}`);
  const list = await post('tools', 'tools/list');
  const names = JSON.parse(list.result).tools.map(tool => tool.name);
  if (JSON.stringify([...names].sort()) !== JSON.stringify([...TOOLS].sort())) throw new Error(`tools are ${names.join(', ')}`);
  const native = metaOf(reference[MODERN].tools);
  const mine = metaOf(list.result);
  let checked = 0;
  for (const item of cases) {
    const recorded = reference[MODERN].calls.find(call => call.name === item.id);
    const { result } = await post(checked, 'tools/call', { name: item.tool, arguments: item.arguments });
    if (result !== servedBy(recorded.result, native, mine)) throw new Error(`${item.id}: the result differs from the native server's`);
    checked += 1;
  }
  return { release: named, cases: checked };
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [endpoint, release] = process.argv.slice(2);
  if (!endpoint) {
    console.error('usage: node live-check.mjs <endpoint URL> [expected release]');
    process.exit(2);
  }
  try {
    const { release: named, cases } = await check(endpoint, release);
    console.log(`ok: ${cases} conformance cases match; ${named}`);
  } catch (error) {
    console.error(`live check failed: ${error.message}`);
    process.exit(1);
  }
}
