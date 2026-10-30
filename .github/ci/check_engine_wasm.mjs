// Exercise the exported ABI and compare its derivations with the native CLI.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { spawnSync } from 'node:child_process';

const [engine, binary] = process.argv.slice(2);
const inputs = {
  'div7a-loan-review': {
    facts: {
      year_of_income: '2026-27', year_loan_made: '2023-24',
      amalgamated_loan_unpaid_at_end_of_previous_year: '100000.00',
      remaining_term_years: '5', payments_applied_during_the_year: '25556.00',
      gate_result: { verdict: 'COMPLYING', benchmark_year_used: '2023-24', reasons: [], caveats: [] },
    },
  },
  'payday-super-checker': { qe_day: '2026-07-09', first_to_fund: true },
};
assert.ok(Object.hasOwn(inputs, engine), 'expected a browser engine');
const file = engine.replaceAll('-', '_') + '_wasm.wasm';
const { instance } = await WebAssembly.instantiate(await readFile(`wasm/target/wasm32-unknown-unknown/release/${file}`));
const { memory, alloc, dealloc } = instance.exports;
const encoder = new TextEncoder();
const decoder = new TextDecoder('utf-8', { fatal: true });

function callJson(command, input) {
  const bytes = encoder.encode(input);
  const ptr = alloc(bytes.length);
  try {
    new Uint8Array(memory.buffer, ptr, bytes.length).set(bytes);
    const packed = instance.exports[command](ptr, bytes.length);
    const outputPtr = Number(packed >> 32n);
    const outputLen = Number(packed & 0xffffffffn);
    try {
      return JSON.parse(decoder.decode(new Uint8Array(memory.buffer, outputPtr, outputLen)));
    } finally {
      dealloc(outputPtr, outputLen);
    }
  } finally {
    dealloc(ptr, bytes.length);
  }
}

const call = (command, input) => callJson(command, JSON.stringify(input));

function nativeJson(command, input, expectedCode) {
  const result = spawnSync(resolve(binary), [command], { input, encoding: 'utf8', timeout: 30000 });
  assert.ifError(result.error);
  assert.equal(result.status, expectedCode, result.stderr);
  return JSON.parse(result.stdout);
}

const native = (command, input, code) => nativeJson(command, JSON.stringify(input), code);

const derived = call('derive', inputs[engine]);
assert.ok(derived.ok, JSON.stringify(derived));
assert.deepEqual(derived.ok, native('derive', inputs[engine], 0));
assert.deepEqual(call('verify', derived.ok).ok, native('verify', derived.ok, 0));
const edited = structuredClone(derived.ok);
edited.steps[0].value = 'edited';
const rejected = call('verify', edited);
assert.deepEqual(rejected.ok, native('verify', edited, 1));
assert.equal(rejected.ok.verified, false);
assert.equal(typeof call('verify', {}).error, 'string');
// Raw JSON exercises the same bounds as the CLI, without a JS number round trip.
const raw = JSON.stringify(inputs[engine]);
assert.deepEqual(callJson('derive', raw.padEnd(8 << 20)), derived);
assert.equal(callJson('derive', raw.padEnd((8 << 20) + 1)).error, 'JSON input exceeds 8 MiB');
for (const [input, prefix] of [['{', 'invalid JSON:'], [raw.padEnd((8 << 20) + 1), 'JSON input exceeds 8 MiB']]) {
  assert.ok(callJson('derive', input).error.startsWith(prefix));
  const result = spawnSync(resolve(binary), ['derive'], { input, encoding: 'utf8', timeout: 30000 });
  assert.ifError(result.error);
  assert.equal(result.status, 2);
  assert.ok(result.stderr.startsWith(`error: ${prefix}`));
}
if (engine === 'div7a-loan-review') {
  const facts = inputs[engine].facts;
  const { year_of_income, gate_result, ...mapping } = facts;
  const input = JSON.stringify({year_of_income, gate_result, mapping: {...mapping, loan_id: '9007199254740993'}})
    .replace('"loan_id":"9007199254740993"', '"loan_id":9007199254740993');
  const saved = spawnSync(resolve(binary), ['derive'], {input, encoding: 'utf8', timeout: 30000});
  assert.equal(saved.status, 0, saved.stderr);
  assert.ok(saved.stdout.includes('9007199254740993'));
  assert.equal(callJson('verify', saved.stdout).ok.verified, true);
  assert.equal(call('verify', JSON.parse(saved.stdout)).ok.verified, false);
}
// Repeated differently sized calls also exercise the allocation/deallocation ABI.
for (const n of [0, 4096, 1, 65536, 0]) {
  assert.deepEqual(callJson('derive', raw + ' '.repeat(n)), derived);
}
console.log(`${engine}: native and WebAssembly derive, verify, tamper, raw JSON and bounds checks pass`);
