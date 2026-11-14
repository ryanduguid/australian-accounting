// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

// The live check, run only against the Worker in local workerd.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { after, before, test } from 'node:test';
import { check } from '../live-check.mjs';
import { start } from './harness.mjs';

let worker;
before(async () => { worker = await start(); });
after(async () => { await worker?.mf.dispose(); });

const endpoint = () => new URL('/mcp', worker.url).href;

test('the live check passes against the built Worker', async () => {
  assert.match(endpoint(), /^http:\/\/(127\.0\.0\.1|localhost):\d+\/mcp$/);
  const result = await check(endpoint(), 'aus-accounting-connector/v0.1.0');
  assert.equal(result.cases, 32);
  assert.match(result.release, /^aus-accounting-connector\/v0\.1\.0; /);
});

test('the live check fails on another release name', async () => {
  await assert.rejects(check(endpoint(), 'aus-accounting-connector/v0.2.0'), /release is/);
});

test('the live check fails when one result differs', async () => {
  let calls = 0;
  const tampered = async (url, init) => {
    const response = await fetch(url, init);
    calls += 1;
    if (calls !== 10) return response;
    // A byte changed, not a value: the check compares bytes.
    const text = (await response.text()).replace('"isError":', '"isError" :');
    return new Response(text, { status: response.status, headers: response.headers });
  };
  await assert.rejects(check(endpoint(), undefined, tampered), /differs from the native server's/);
});

test('the live check fails against an expired release', async () => {
  const expired = await start({ bundle: 'build-expired' });
  try {
    await assert.rejects(check(new URL('/mcp', expired.url).href), /differs from the native server's/);
  } finally {
    await expired.mf.dispose();
  }
});

test('the live check refuses plain HTTP to a remote host before sending anything', async () => {
  let calls = 0;
  const counted = async () => { calls += 1; throw new Error('sent'); };
  await assert.rejects(check('http://example.invalid/mcp', undefined, counted), /refusing http:/);
  assert.equal(calls, 0);
});

test('the live check needs an endpoint and has no default', () => {
  const run = spawnSync(process.execPath, [fileURLToPath(new URL('../live-check.mjs', import.meta.url))], { encoding: 'utf8' });
  assert.equal(run.status, 2);
  assert.match(run.stderr, /usage/);
});
