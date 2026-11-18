// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

// Check a running connector with fabricated inputs only: the 32 conformance
// cases, compared byte for byte with tests/fixtures/reference.json.
//   node live-check.mjs ENDPOINT_URL [EXPECTED_RELEASE, such as aus-accounting-connector/v0.1.0]
// It sends 34 requests and exits non-zero on the first wrong answer.
import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';
import { MODERN, headers, message, metaOf, resultText, servedBy } from './tests/protocol.mjs';

const require = createRequire(import.meta.url);
const modern = new Map(Object.entries(require('./tests/fixtures/reference.json'))).get(MODERN);
const { cases } = require('../../apps/aus-accounting-mcp/conformance/cases.json');

const byText = (a, b) => a.localeCompare(b);
const TOOLS = ['calc_payday_super_deadline', 'get_div7a_benchmark_rate', 'review_div7a_loan', 'refuse_div7a', 'calculate_tax_worksheet', 'get_data_freshness'];

export async function check(endpoint, release, request = fetch) {
  const url = new URL(endpoint);
  const local = ['localhost', '127.0.0.1', '[::1]'].includes(url.hostname);
  if (url.protocol !== 'https:' && !(local && url.protocol === 'http:')) throw new Error(`refusing ${url.protocol} for a remote endpoint`);
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
  if (JSON.stringify(names.toSorted(byText)) !== JSON.stringify(TOOLS.toSorted(byText))) throw new Error(`tools are ${names.join(', ')}`);
  const native = metaOf(modern.tools);
  const mine = metaOf(list.result);
  let checked = 0;
  for (const item of cases) {
    const recorded = modern.calls.find(call => call.name === item.id);
    const { result } = await post(checked, 'tools/call', { name: item.tool, arguments: item.arguments });
    if (result !== servedBy(recorded.result, native, mine)) throw new Error(`${item.id}: the result differs from the reference`);
    checked += 1;
  }
  return { release: named, cases: checked };
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [endpoint, release] = process.argv.slice(2);
  if (!endpoint) {
    console.error('usage: node live-check.mjs ENDPOINT_URL [EXPECTED_RELEASE]');
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
