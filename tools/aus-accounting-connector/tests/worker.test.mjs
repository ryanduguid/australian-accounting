// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

// The built Worker in local workerd, compared byte for byte with the same
// handler's replies computed natively (tests/fixtures/reference.json, which
// tests/reference.rs keeps current), and the transport checks of design Appendix D.
import assert from 'node:assert/strict';
import { readFile, readdir, stat } from 'node:fs/promises';
import { after, before, test } from 'node:test';
import { LEGACY, MODERN, caller, elements, headers, limiter, message, metaOf, resultText, servedBy, start } from './harness.mjs';

const root = new URL('../', import.meta.url);
const reference = JSON.parse(await readFile(new URL('tests/fixtures/reference.json', root), 'utf8'));
const cases = JSON.parse(await readFile(new URL('../../apps/aus-accounting-mcp/conformance/cases.json', root), 'utf8')).cases;
const SERVED = ['calc_payday_super_deadline', 'get_div7a_benchmark_rate', 'review_div7a_loan', 'refuse_div7a', 'calculate_tax_worksheet', 'get_data_freshness'];
const EXPIRED = 'stopped answering tool calls on';
let worker;

before(async () => { worker = await start(); });
after(async () => { await worker?.mf.dispose(); });

async function post(body, sent, path = '/mcp', target = worker) {
  const text = typeof body === 'string' ? body : JSON.stringify(body);
  const response = await target.mf.dispatchFetch(new URL(path, target.url), {
    method: 'POST', body: text, headers: { 'cf-connecting-ip': caller(), ...sent },
  });
  return { response, body: await response.text() };
}

async function send(era, id, method, params, extra, target) {
  const body = message(era, id, method, params);
  return post(body, headers(body, era, extra), '/mcp', target);
}

function referenceTools(era) {
  const list = reference[era].tools;
  return elements(list.slice(list.indexOf('"tools":[') + 8, -1));
}

test('the reference holds every conformance case and the served tools', () => {
  for (const era of [MODERN, LEGACY]) {
    const names = reference[era].calls.map(call => call.name);
    for (const item of cases) assert.ok(names.includes(item.id), item.id);
    assert.equal(cases.length, 32);
    assert.deepEqual([...new Set(reference[era].calls.map(call => call.tool))].sort(), [...SERVED].sort());
  }
});

test('2026-07-28 tool results equal the reference byte for byte', async () => {
  const discover = await send(MODERN, 'd', 'server/discover');
  const list = resultText((await send(MODERN, 'l', 'tools/list')).body, 'l');
  const mine = metaOf(list);
  const native = metaOf(reference[MODERN].tools);
  assert.ok(discover.body.includes(`"_meta":${mine}`));
  for (const [index, call] of reference[MODERN].calls.entries()) {
    const { response, body } = await send(MODERN, index, 'tools/call', { name: call.tool, arguments: call.arguments });
    assert.equal(response.status, 200, call.name);
    const text = resultText(body, index);
    // The one difference: `_meta` names the build producing the result.
    assert.equal(text, servedBy(call.result, native, mine), call.name);
  }
});

test('2025-11-25 tool results equal the reference byte for byte', async () => {
  const init = await send(LEGACY, 'i', 'initialize', { protocolVersion: LEGACY, capabilities: {}, clientInfo: { name: 'harness', version: '1' } });
  assert.equal(init.response.status, 200);
  for (const [index, call] of reference[LEGACY].calls.entries()) {
    const { response, body } = await send(LEGACY, index, 'tools/call', { name: call.tool, arguments: call.arguments });
    assert.equal(response.status, 200, call.name);
    const text = resultText(body, index);
    assert.equal(text, call.result, call.name);
  }
});

test('tools/list gives the reference entries for the 6 tools, in order, byte for byte', async () => {
  const six = referenceTools(MODERN);
  assert.deepEqual(six.map(entry => JSON.parse(entry).name).sort(), [...SERVED].sort());
  assert.deepEqual(referenceTools(LEGACY), six);
  const modern = resultText((await send(MODERN, 1, 'tools/list')).body, 1);
  const meta = metaOf(modern);
  assert.equal(modern, `{"resultType":"complete","_meta":${meta},"ttlMs":3600000,"cacheScope":"public","tools":[${six.join(',')}]}`);
  const legacy = resultText((await send(LEGACY, 2, 'tools/list')).body, 2);
  assert.equal(legacy, `{"tools":[${six.join(',')}]}`);
});

test('server/discover lists 2026-07-28 and names the release', async () => {
  const { response, body } = await send(MODERN, 1, 'server/discover');
  assert.equal(response.status, 200);
  const result = JSON.parse(body).result;
  assert.equal(result.resultType, 'complete');
  assert.deepEqual(result.supportedVersions, [MODERN]);
  assert.deepEqual(result.capabilities, { tools: {} });
  assert.equal(result.ttlMs, 3600000);
  assert.equal(result.cacheScope, 'public');
  const info = result._meta['io.modelcontextprotocol/serverInfo'];
  assert.equal(info.name, 'aus-accounting-connector');
  assert.match(info.version, /^0\.1\.0\+\S+$/);
  assert.match(response.headers.get('x-connector-release'), /^aus-accounting-connector\/v0\.1\.0; commit=\S+; serve-until=\d{4}-\d{2}-\d{2}$/);
  assert.equal(response.headers.get('cache-control'), 'no-store');
});

test('a legacy initialize mints no session, and a supplied session is ignored', async () => {
  for (const version of [LEGACY, '2025-06-18', '2024-11-05']) {
    const { response, body } = await send(LEGACY, 1, 'initialize', { protocolVersion: version, capabilities: {}, clientInfo: { name: 'harness', version: '1' } });
    assert.equal(response.status, 200);
    assert.equal(response.headers.get('mcp-session-id'), null);
    const result = JSON.parse(body).result;
    assert.equal(result.protocolVersion, version === '2024-11-05' ? LEGACY : version);
    assert.equal(result.serverInfo.name, 'aus-accounting-connector');
  }
  const plain = await send(LEGACY, 7, 'tools/call', { name: 'refuse_div7a', arguments: {} });
  const sessioned = await send(LEGACY, 7, 'tools/call', { name: 'refuse_div7a', arguments: {} }, { 'mcp-session-id': 'not-a-session', 'last-event-id': '3' });
  assert.equal(sessioned.response.status, 200);
  assert.equal(sessioned.response.headers.get('mcp-session-id'), null);
  assert.equal(sessioned.body, plain.body);
});

test('notifications return 202 with no body', async () => {
  for (const era of [MODERN, LEGACY]) {
    const body = { jsonrpc: '2.0', method: 'notifications/initialized', ...(era === MODERN ? { params: { _meta: { 'io.modelcontextprotocol/protocolVersion': MODERN } } } : {}) };
    const { response, body: text } = await post(body, headers({ method: 'notifications/initialized' }, era));
    assert.equal(response.status, 202);
    assert.equal(text, '');
  }
});

test('only POST to /mcp reaches the handler; a query string is ignored', async () => {
  const body = message(MODERN, 1, 'server/discover');
  assert.equal((await post(body, headers(body, MODERN), '/mcp?x=1')).response.status, 200);
  for (const path of ['/mcp-extra', '/mcp/', '/MCP', '/', '/mcp/x']) {
    const { response, body: text } = await post(body, headers(body, MODERN), path);
    assert.equal(response.status, 404, path);
    assert.equal(text, '', path);
  }
  for (const method of ['GET', 'DELETE', 'PUT']) {
    const response = await worker.mf.dispatchFetch(new URL('/mcp', worker.url), { method, headers: { 'cf-connecting-ip': caller() } });
    assert.equal(response.status, 405, method);
    assert.equal(response.headers.get('allow'), 'POST');
    assert.equal(await response.text(), '');
  }
});

test('Origin: listed origins are allowed, others get 403', async () => {
  const body = message(MODERN, 1, 'server/discover');
  for (const origin of ['https://claude.ai', 'https://chatgpt.com']) {
    const { response } = await post(body, headers(body, MODERN, { origin }));
    assert.equal(response.status, 200, origin);
    assert.equal(response.headers.get('access-control-allow-origin'), origin);
  }
  const none = await post(body, headers(body, MODERN));
  assert.equal(none.response.headers.get('access-control-allow-origin'), null);
  for (const origin of ['https://attacker.example', 'null', 'https://claude.ai.attacker.example', 'http://claude.ai']) {
    const { response, body: text } = await post(body, headers(body, MODERN, { origin }));
    assert.equal(response.status, 403, origin);
    assert.equal(text, '');
    assert.equal(response.headers.get('access-control-allow-origin'), null);
  }
  const preflight = await worker.mf.dispatchFetch(new URL('/mcp', worker.url), { method: 'OPTIONS', headers: { origin: 'https://claude.ai', 'cf-connecting-ip': caller() } });
  assert.equal(preflight.status, 204);
  assert.equal(preflight.headers.get('access-control-allow-methods'), 'POST');
  const refused = await worker.mf.dispatchFetch(new URL('/mcp', worker.url), { method: 'OPTIONS', headers: { origin: 'https://attacker.example', 'cf-connecting-ip': caller() } });
  assert.equal(refused.status, 403);
});

test('a body over 16 KiB gets 413 before it is parsed; 16 KiB is read', async () => {
  const body = JSON.stringify(message(MODERN, 1, 'server/discover'));
  const exact = body.padEnd(16 * 1024, ' ');
  const sent = headers({ method: 'server/discover' }, MODERN, { origin: 'https://claude.ai' });
  assert.equal((await post(exact, sent)).response.status, 200);
  // Not JSON past the limit, so a 413 shows the body was never parsed.
  const over = await post(exact + '!', sent);
  assert.equal(over.response.status, 413);
  assert.equal(over.body, '');
  assert.equal(over.response.headers.get('access-control-allow-origin'), 'https://claude.ai');
  const stream = new ReadableStream({
    start(controller) {
      for (let i = 0; i < 5; i += 1) controller.enqueue(new Uint8Array(4096).fill(32));
      controller.close();
    },
  });
  const streamed = await worker.mf.dispatchFetch(new URL('/mcp', worker.url), { method: 'POST', body: stream, duplex: 'half', headers: { ...sent, 'cf-connecting-ip': caller() } });
  assert.equal(streamed.status, 413);
  assert.equal(streamed.headers.get('access-control-allow-origin'), 'https://claude.ai');
  assert.equal(streamed.headers.get('content-length') ?? '0', '0');
  await streamed.text();
});

test('a body that is not JSON gets 415', async () => {
  const body = message(MODERN, 1, 'server/discover');
  for (const type of ['text/plain', 'application/x-www-form-urlencoded', '']) {
    const { response } = await post(body, { ...headers(body, MODERN), 'content-type': type });
    assert.equal(response.status, 415, type);
  }
  const charset = await post(body, { ...headers(body, MODERN), 'content-type': 'application/json; charset=utf-8' });
  assert.equal(charset.response.status, 200);
});

test('version and header checks', async () => {
  const error = async (body, sent, status, code) => {
    const reply = await post(body, sent);
    assert.equal(reply.response.status, status, reply.body);
    const parsed = JSON.parse(reply.body);
    assert.equal(parsed.error.code, code, reply.body);
    return parsed.error;
  };
  const discover = message(MODERN, 1, 'server/discover');
  await error(discover, { ...headers(discover, MODERN), 'mcp-protocol-version': LEGACY }, 400, -32020);
  const missing = headers(discover, MODERN);
  delete missing['mcp-protocol-version'];
  await error(discover, missing, 400, -32020);
  await error(discover, { ...headers(discover, MODERN), 'mcp-method': 'tools/list' }, 400, -32020);
  const noMethod = headers(discover, MODERN);
  delete noMethod['mcp-method'];
  await error(discover, noMethod, 400, -32020);
  const future = { ...discover, params: { _meta: { ...discover.params._meta, 'io.modelcontextprotocol/protocolVersion': '2099-01-01' } } };
  const unsupported = await error(future, { ...headers(future, MODERN), 'mcp-protocol-version': '2099-01-01' }, 400, -32022);
  assert.deepEqual(unsupported.data, { supported: [MODERN, LEGACY, '2025-06-18'], requested: '2099-01-01' });
  const noCapabilities = { ...discover, params: { _meta: { 'io.modelcontextprotocol/protocolVersion': MODERN } } };
  await error(noCapabilities, headers(noCapabilities, MODERN), 400, -32602);
  const call = message(MODERN, 2, 'tools/call', { name: 'refuse_div7a', arguments: {} });
  await error(call, { ...headers(call, MODERN), 'mcp-name': 'get_data_freshness' }, 400, -32020);
  const noName = headers(call, MODERN);
  delete noName['mcp-name'];
  await error(call, noName, 400, -32020);
  // Legacy requests other than initialize carry a supported version header.
  const legacyList = message(LEGACY, 3, 'tools/list');
  await error(legacyList, { 'content-type': 'application/json' }, 400, -32022);
  await error(legacyList, { 'content-type': 'application/json', 'mcp-protocol-version': '2025-03-26' }, 400, -32022);
  await error(legacyList, { 'content-type': 'application/json', 'mcp-protocol-version': MODERN }, 400, -32020);
});

test('unknown methods get -32601: 404 in 2026-07-28, 200 before it', async () => {
  const modern = await send(MODERN, 1, 'resources/list');
  assert.equal(modern.response.status, 404);
  assert.equal(JSON.parse(modern.body).error.code, -32601);
  // Before 2026-07-28, HTTP 404 tells a client its session has ended.
  const legacy = await send(LEGACY, 2, 'resources/list');
  assert.equal(legacy.response.status, 200);
  assert.equal(JSON.parse(legacy.body).error.code, -32601);
});

test('an unknown tool is a JSON-RPC error pointing to the local server', async () => {
  for (const era of [MODERN, LEGACY]) {
    const { response, body } = await send(era, 1, 'tools/call', { name: 'review_gst_coding', arguments: {} });
    assert.equal(response.status, 200);
    const parsed = JSON.parse(body).error;
    assert.equal(parsed.code, -32602);
    assert.match(parsed.message, /local Aus Accounting MCP server/);
  }
});

test('responses, batches and malformed JSON are refused with 400', async () => {
  const sent = headers({ method: 'tools/list' }, LEGACY);
  for (const body of ['{"jsonrpc":"2.0","id":1,"result":{}}', '{"jsonrpc":"2.0","id":1,"error":{"code":1,"message":"x"}}',
    '[{"jsonrpc":"2.0","id":1,"method":"tools/list"}]', '[]', '{"jsonrpc":"2.0","id":1,"method":', '"text"',
    '{"jsonrpc":"1.0","id":1,"method":"tools/list"}', '{"jsonrpc":"2.0","id":{},"method":"tools/list"}',
    '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":[]}']) {
    const { response, body: text } = await post(body, sent);
    assert.equal(response.status, 400, body);
    assert.equal(JSON.parse(text).jsonrpc, '2.0', body);
  }
});

test('the rate limiter answers 429 with Retry-After: 60, and its absence fails closed; an allowed origin can read both', async () => {
  const limited = await start();
  const unbound = await start({ limit: false });
  try {
    const { simple } = await limiter();
    const body = message(MODERN, 1, 'server/discover');
    const sent = { ...headers(body, MODERN, { origin: 'https://chatgpt.com' }), 'cf-connecting-ip': '198.51.100.7' };
    const statuses = [];
    for (let i = 0; i <= simple.limit; i += 1) {
      const response = await limited.mf.dispatchFetch(new URL('/mcp', limited.url), { method: 'POST', body: JSON.stringify(body), headers: sent });
      statuses.push(response.status);
      if (response.status === 429) assert.equal(response.headers.get('retry-after'), '60');
      assert.equal(response.headers.get('access-control-allow-origin'), 'https://chatgpt.com');
      await response.text();
    }
    assert.deepEqual(statuses, [...Array(simple.limit).fill(200), 429]);
    const other = await limited.mf.dispatchFetch(new URL('/mcp', limited.url), { method: 'POST', body: JSON.stringify(body), headers: { ...sent, 'cf-connecting-ip': '198.51.100.8' } });
    assert.equal(other.status, 200);
    await other.text();
    const response = await unbound.mf.dispatchFetch(new URL('/mcp', unbound.url), { method: 'POST', body: JSON.stringify(body), headers: sent });
    assert.equal(response.status, 503);
    assert.equal(response.headers.get('access-control-allow-origin'), 'https://chatgpt.com');
    await response.text();
  } finally {
    await limited.mf.dispose();
    await unbound.mf.dispose();
  }
});

test('from serve_until every tool call is refused, in both eras', async () => {
  const expired = await start({ bundle: 'build-expired' });
  try {
    const discover = await send(MODERN, 1, 'server/discover', {}, {}, expired);
    assert.equal(discover.response.status, 200);
    assert.match(discover.response.headers.get('x-connector-release'), /serve-until=2000-01-01$/);
    for (const era of [MODERN, LEGACY]) {
      for (const name of SERVED) {
        const { response, body } = await send(era, 2, 'tools/call', { name, arguments: {} }, {}, expired);
        assert.equal(response.status, 200);
        const result = JSON.parse(body).result;
        assert.equal(result.isError, true, name);
        assert.ok(result.content[0].text.includes(`${EXPIRED} 2000-01-01`), name);
      }
    }
    for (const call of reference[MODERN].calls.slice(0, 3)) {
      const { body } = await send(MODERN, 3, 'tools/call', { name: call.tool, arguments: call.arguments }, {}, expired);
      assert.ok(body.includes(EXPIRED), call.name);
    }
  } finally {
    await expired.mf.dispose();
  }
  // The served bundle answers today.
  const { body } = await send(MODERN, 4, 'tools/call', { name: 'refuse_div7a', arguments: {} });
  assert.ok(!body.includes(EXPIRED));
});

test('wrangler.toml holds only the reviewed settings: no storage, no logs, no preview URLs', async () => {
  const lines = (await readFile(new URL('wrangler.toml', root), 'utf8')).split(/\r?\n/).map(line => line.trim()).filter(line => line && !line.startsWith('#'));
  assert.deepEqual(lines, [
    'name = "aus-accounting-connector"',
    'main = "build/worker/shim.mjs"',
    'compatibility_date = "2026-07-30"',
    'workers_dev = false',
    'preview_urls = false',
    'routes = [{ pattern = "duguid.com.au/mcp*", zone_name = "duguid.com.au" }]',
    '[limits]',
    'cpu_ms = 10',
    '[observability]',
    'enabled = false',
    '[[ratelimits]]',
    'name = "LIMITER"',
    'namespace_id = "2272"',
    '[ratelimits.simple]',
    'limit = 60',
    'period = 60',
  ]);
  assert.deepEqual(await limiter(), { name: 'LIMITER', namespace_id: '2272', simple: { limit: 60, period: 60 } });
});

test('the bundle is under the 64 MiB Worker size limit', async () => {
  let total = 0;
  for (const entry of await readdir(new URL('build/', root), { recursive: true, withFileTypes: true })) {
    if (entry.isFile()) total += (await stat(`${entry.parentPath}/${entry.name}`)).size;
  }
  assert.ok(total > 0 && total < 64 * 1024 * 1024, `${total} bytes`);
});

// Last, so it covers every request above.
test('the Worker made no outbound request and logged no request body', () => {
  assert.deepEqual(worker.outbound, []);
  const output = worker.output.join('');
  for (const marker of ['synthetic-conformance-loan', '108770.00', 'payments_applied_during_the_year', 'arguments']) {
    assert.ok(!output.includes(marker), marker);
  }
});
