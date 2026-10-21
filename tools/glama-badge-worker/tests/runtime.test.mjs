import assert from 'node:assert/strict';
import { readFile, readdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { test, after } from 'node:test';
import { Miniflare, Response } from 'miniflare';

const fixtures = new URL('../../glama-badge-server/tests/fixtures/', import.meta.url);
const sample = await readFile(new URL('server-a-b-d.svg', fixtures), 'utf8');
const ids = (await readFile(new URL('../../glama-badge-server/allowed.txt', import.meta.url), 'utf8')).trim().split(/\r?\n/);
let mode = 'ok';
let body = sample;
let styledSample;
let calls = [];
let releaseGate;
const mf = new Miniflare({
  modules: true,
  scriptPath: fileURLToPath(new URL('../build/worker/shim.mjs', import.meta.url)),
  compatibilityDate: '2026-07-30',
  modulesRules: [{ type: 'ESModule', include: ['**/*.js'], fallthrough: true }, { type: 'CompiledWasm', include: ['**/*.wasm'], fallthrough: true }],
  outboundService: async request => {
    calls.push({ url: request.url, method: request.method, headers: Object.fromEntries(request.headers) });
    if (mode === 'gate') await new Promise(resolve => { (releaseGate ??= []).push(resolve); });
    if (mode === 'slow') await new Promise(resolve => setTimeout(resolve, 9_000));
    if (mode === 'redirect') return new Response(null, { status: 302, headers: { location: 'https://example.com/forbidden' } });
    if (/^status-/.test(mode)) return new Response(mode === 'status-204' ? null : body, { status: Number(mode.slice(7)), headers: { 'content-type': 'image/svg+xml' } });
    if (mode === 'error') return new Response('unavailable', { status: 503 });
    if (mode === 'media') return new Response(body, { headers: { 'content-type': 'text/html' } });
    if (mode === 'length') return new Response(body, { headers: { 'content-type': 'image/svg+xml', 'content-length': '32769' } });
    if (mode === 'body-stall') return new Response(new ReadableStream({ start(c) { c.enqueue(new TextEncoder().encode(body.slice(0, 100))); const timer = setTimeout(() => { try { c.enqueue(new TextEncoder().encode(body.slice(100))); c.close(); } catch {} }, 9_000); timer.unref(); } }), { headers: { 'content-type': 'image/svg+xml' } });
    if (mode === 'stream') return new Response(new ReadableStream({ start(c) { c.enqueue(new Uint8Array(20_000)); c.enqueue(new Uint8Array(20_000)); c.close(); } }), { headers: { 'content-type': 'image/svg+xml' } });
    return new Response(body, { headers: { 'content-type': 'image/svg+xml; charset=utf-8' } });
  },
});
const deadline = setTimeout(async () => { console.error('Worker test deadline exceeded'); await mf.dispose(); process.exitCode = 1; }, 55_000);
after(async () => { clearTimeout(deadline); await mf.dispose(); });
const url = id => 'https://badges.example/glama/' + id + '.svg';
const key = id => 'https://flat-glama-badges.invalid/v1/' + id + '.svg';
async function reset(next = 'ok', svg = sample) { await mf.purgeCache(); mode = next; body = svg; calls = []; }
async function request(id = ids[0], init) { return mf.dispatchFetch(url(id), init); }
function glyphPaths(svg) {
  const defs = svg.match(/<defs>([\s\S]*?)<\/defs>/)?.[1];
  assert.ok(defs, 'glyph definitions must exist');
  return [...defs.matchAll(/<path\b[^>]*>/g)].map(([tag]) => ({ id: tag.match(/\bid="([^"]+)"/)?.[1], d: tag.match(/\bd="([^"]+)"/)?.[1] }));
}
async function failure(response, status = 502) { assert.equal(response.status, status); assert.equal(response.headers.get('cache-control'), 'no-store'); await response.text(); }

await test('Worker boundary and live rendering contract', async t => {
  await mf.ready;
  await t.test('routes reject unknown identities, query strings and methods without fetching', async () => {
    await reset();
    for (const path of ['/', '/glama/unknown.svg', '/glama/' + ids[0] + '.svg?token=x', '/glama/' + ids[0].replace('/', '%2F') + '.svg']) await failure(await mf.dispatchFetch('https://badges.example' + path), 404);
    const response = await request(ids[0], { method: 'POST' });
    assert.equal(response.headers.get('allow'), 'GET, HEAD');
    await failure(response, 405);
    assert.equal(calls.length, 0);
  });
  await t.test('fixed HTTPS origin with no caller credentials and security headers', async () => {
    await reset();
    const response = await request(ids[0], { headers: { authorization: 'test-only', cookie: 'test-only', 'x-untrusted': 'test-only' } });
    assert.equal(response.status, 200);
    assert.equal(response.headers.get('content-type'), 'image/svg+xml; charset=utf-8');
    assert.equal(response.headers.get('content-security-policy'), "default-src 'none'; sandbox");
    assert.equal(response.headers.get('x-content-type-options'), 'nosniff');
    assert.equal(response.headers.get('referrer-policy'), 'no-referrer');
    assert.equal(response.headers.get('cache-control'), 'public, max-age=60, must-revalidate');
    const output = await response.text(); styledSample = output;
    assert.match(output, /#000000/); assert.match(output, /#FFFFF0/); assert.doesNotMatch(output, /linearGradient/);
    assert.equal(calls.length, 1);
    assert.equal(calls[0].url, 'https://glama.ai/mcp/' + ids[0] + '/badges/score.svg');
    assert.equal(calls[0].method, 'GET');
    for (const name of ['authorization', 'cookie', 'x-untrusted']) assert.equal(calls[0].headers[name], undefined);
  });
  await t.test('HEAD has no body and successful GET uses the source cache', async () => {
    // Cache writes finish through waitUntil before the next dispatched request.
    const response = await request(ids[0], { method: 'HEAD' });
    assert.equal(response.status, 200); assert.equal(await response.text(), '');
    assert.equal(calls.length, 1);
    const hit = await request(); assert.equal(hit.status, 200); assert.equal(await hit.text(), styledSample); assert.equal(calls.length, 1);
  });
  for (const name of (await readdir(fixtures)).filter(name => name.endsWith('.svg'))) {
    await t.test('preserves glyph paths and metadata: ' + name, async () => {
      const input = await readFile(new URL(name, fixtures), 'utf8'); await reset('ok', input);
      const response = await request(); assert.equal(response.status, 200);
      const output = await response.text();
      for (const tag of ['title', 'desc']) assert.equal(output.match(new RegExp('<' + tag + '>(.*?)</' + tag + '>'))?.[1], input.match(new RegExp('<' + tag + '>(.*?)</' + tag + '>'))?.[1]);
      const glyphs = glyphPaths;
      assert.ok(glyphs(input).length > 0); assert.deepEqual(glyphs(output), glyphs(input));
      const uses = svg => [...svg.matchAll(/<use\b[^>]*>/g)].map(m => [...m[0].matchAll(/(?:href|x|y)="[^"]*"/g)].map(a => a[0]));
      assert.deepEqual(uses(output), uses(input));
    });
  }
  await t.test('glyph comparison detects changed geometry', () => { const original = glyphPaths(sample); assert.ok(original.length > 0); assert.ok(original[0].id && original[0].d); const changed = structuredClone(original); changed[0].d = changed[0].d.replace(/[0-9]/, '9'); assert.notEqual(changed[0].d, original[0].d); assert.throws(() => assert.deepEqual(changed, original)); });
  for (const next of ['redirect', 'error', 'media', 'length', 'stream', 'status-202', 'status-204', 'status-206']) await t.test('fails closed: ' + next, async () => { await reset(next); await failure(await request()); assert.equal(calls.length, 1); });
  await t.test('rejects an unsupported SVG grammar', async () => { await reset('ok', '<svg xmlns="http://www.w3.org/2000/svg"><script>alert(1)</script></svg>'); await failure(await request()); });
  await t.test('expired or future cache entries cannot hide a source failure', async () => {
    const cache = (await mf.getCaches()).default;
    for (const time of [Date.now() - 301_000, Date.now() + 60_000]) {
      await reset('error');
      await cache.put(key(ids[0]), new Response(styledSample, { headers: { 'cache-control': 'max-age=300', 'x-glama-source-fetched-at': String(time) } }));
      await failure(await request()); assert.equal(calls.length, 1);
    }
  });
  await t.test('near-expiry hits reduce client freshness', async () => {
    await reset();
    const cache = (await mf.getCaches()).default;
    await cache.put(key(ids[0]), new Response(styledSample, { headers: { 'cache-control': 'max-age=300', 'x-glama-source-fetched-at': String(Date.now() - 298_000) } }));
    const response = await request(); assert.equal(response.status, 200);
    assert.match(response.headers.get('cache-control'), /max-age=[01],/); assert.equal(await response.text(), styledSample);
    assert.equal(calls.length, 0);
  });
  await t.test('invalid cached bodies are discarded and fetched again', async () => {
    const cache = (await mf.getCaches()).default;
    for (const cached of [new Uint8Array(65_537), new Uint8Array([255])]) {
      await reset();
      await cache.put(key(ids[0]), new Response(cached, { headers: { 'cache-control': 'max-age=300', 'x-glama-source-fetched-at': String(Date.now()) } }));
      const response = await request(); assert.equal(response.status, 200); await response.text(); assert.equal(calls.length, 1);
    }
  });
  await t.test('a previously fresh cached badge expires instead of masking failure', async () => {
    await reset('error'); const cache = (await mf.getCaches()).default;
    await cache.put(key(ids[0]), new Response(styledSample, { headers: { 'cache-control': 'max-age=300', 'x-glama-source-fetched-at': String(Date.now() - 299_000) } }));
    const fresh = await request(); assert.equal(fresh.status, 200); assert.equal(await fresh.text(), styledSample); assert.equal(calls.length, 0);
    await new Promise(resolve => setTimeout(resolve, 1_050));
    await failure(await request()); assert.equal(calls.length, 1);
  });
  await t.test('256 distinct cold badges complete with bounded concurrency', async () => {
    await reset(); const start = performance.now();
    for (let offset = 0; offset < 256; offset += 32) await Promise.all(ids.slice(offset, offset + 32).map(async id => { const response = await request(id); assert.equal(response.status, 200); await response.text(); }));
    assert.equal(calls.length, 256);
    console.log(JSON.stringify({ coldBadges: 256, concurrency: 32, elapsedMs: Math.round(performance.now() - start) }));
  });
  await t.test('cache misses admit at most 32 upstream fetches per isolate', async () => {
    await reset('gate'); releaseGate = [];
    const pending = Array.from({ length: 32 }, () => request(ids[0]));
    try {
      const deadline = Date.now() + 2_000;
      while (calls.length < 32 && Date.now() < deadline) await new Promise(resolve => setTimeout(resolve, 5));
      assert.equal(calls.length, 32);
      await failure(await request(ids[0]), 503);
      assert.equal(calls.length, 32);
    } finally {
      for (const release of releaseGate) release();
      for (const response of await Promise.all(pending)) { assert.equal(response.status, 200); await response.text(); }
    }
    mode = 'ok';
    const recovered = await request(ids[33]); assert.equal(recovered.status, 200); await recovered.text();
  });
  for (const stalled of ['slow', 'body-stall']) await t.test('deadline fails closed: ' + stalled, async () => {
    await reset(stalled); const start = performance.now(); await failure(await request());
    assert.ok(performance.now() - start >= 7_500); assert.ok(performance.now() - start < 10_500);
  });
});
