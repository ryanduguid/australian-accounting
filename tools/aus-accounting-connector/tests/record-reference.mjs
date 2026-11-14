// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

// Record the native stdio server's complete results, as raw JSON text, for the
// requests the Worker harness compares byte for byte:
//   node tests/record-reference.mjs <aus-accounting-mcp binary> <its commit>
// It speaks both protocol eras: 2026-07-28 per request, and the 2025-11-25
// initialize handshake. Nothing here runs in CI; the fixture it writes does.
import { spawn } from 'node:child_process';
import { readFile, writeFile } from 'node:fs/promises';
import { createInterface } from 'node:readline';
import { requests } from './requests.mjs';

const [binary, commit] = process.argv.slice(2);
if (!binary || !commit) throw new Error('usage: record-reference.mjs <binary> <commit>');

async function session(lines) {
  const child = spawn(binary, [], { stdio: ['pipe', 'pipe', 'inherit'] });
  const replies = new Map();
  const reader = createInterface({ input: child.stdout });
  reader.on('line', line => {
    const id = JSON.parse(line).id;
    if (id !== undefined) replies.set(id, line);
  });
  for (const line of lines) child.stdin.write(JSON.stringify(line) + '\n');
  const wanted = lines.filter(line => line.id !== undefined).length;
  while (replies.size < wanted) await new Promise(resolve => setTimeout(resolve, 20));
  child.stdin.end();
  await new Promise(resolve => child.on('exit', resolve));
  return replies;
}

// The text of `result` in a line `{"jsonrpc":"2.0","id":<id>,"result":<result>}`.
function result(line, id) {
  const head = `{"jsonrpc":"2.0","id":${JSON.stringify(id)},"result":`;
  if (!line.startsWith(head) || !line.endsWith('}')) throw new Error(`unexpected reply: ${line.slice(0, 200)}`);
  return line.slice(head.length, -1);
}

const cases = JSON.parse(await readFile(new URL('../../../apps/aus-accounting-mcp/conformance/cases.json', import.meta.url), 'utf8')).cases;
const meta = { 'io.modelcontextprotocol/protocolVersion': '2026-07-28', 'io.modelcontextprotocol/clientCapabilities': {} };
const fixture = { source: { server: 'aus-accounting-mcp native stdio server (draft PR 599)', commit, engines: 'packages/ at the core branch base', patch: "the core's PAYG withholding entry in NARROWED_SCOPE" } };
for (const era of ['2026-07-28', '2025-11-25']) {
  const list = requests(cases);
  const lines = era === '2025-11-25'
    ? [{ jsonrpc: '2.0', id: 'init', method: 'initialize', params: { protocolVersion: era, capabilities: {}, clientInfo: { name: 'reference-recorder', version: '1' } } },
       { jsonrpc: '2.0', method: 'notifications/initialized' }]
    : [{ jsonrpc: '2.0', id: 'discover', method: 'server/discover', params: { _meta: meta } }];
  lines.push({ jsonrpc: '2.0', id: 'tools', method: 'tools/list', params: era === '2025-11-25' ? {} : { _meta: meta } });
  for (const [index, request] of list.entries()) {
    const params = { name: request.tool, arguments: request.arguments };
    if (era !== '2025-11-25') params._meta = meta;
    lines.push({ jsonrpc: '2.0', id: index, method: 'tools/call', params });
  }
  const replies = await session(lines);
  fixture[era] = {
    tools: result(replies.get('tools'), 'tools'),
    calls: list.map((request, index) => ({ ...request, result: result(replies.get(index), index) })),
  };
}
await writeFile(new URL('./fixtures/reference.json', import.meta.url), JSON.stringify(fixture, null, 1) + '\n');
console.log(`recorded ${fixture['2026-07-28'].calls.length} calls in each era`);
