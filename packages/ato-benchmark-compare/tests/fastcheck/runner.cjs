'use strict';
// The fast-check worker for tests/_fastcheck.py. It reads one JSON configuration
// line, sends each generated input to Python as a JSON line and reads back the
// verdict, so the assertions and the arithmetic stay in Python. fast-check
// generates, shrinks and replays; nothing here touches the network or the
// component's own code. properties.cjs holds this component's input domains.
// Each component keeps an identical copy of this file.
const fc = require('fast-check');
const readline = require('node:readline');

// An exact decimal string for a non-negative integer count of 10^-places units.
function decimal(units, places) {
  const digits = String(units).padStart(places + 1, '0');
  return { $decimal: places ? `${digits.slice(0, -places)}.${digits.slice(-places)}` : digits };
}

const helpers = {
  fc,
  // Every value from min to max units of 10^-places, inclusive.
  scaled: (min, max, places) => fc.integer({ min, max }).map((units) => decimal(units, places)),
  // An ISO date from the first to the last, inclusive, both YYYY-MM-DD.
  day: (first, last) => {
    const start = Date.parse(`${first}T00:00:00Z`);
    const days = (Date.parse(`${last}T00:00:00Z`) - start) / 86_400_000;
    return fc.integer({ min: 0, max: days })
      .map((offset) => ({ $date: new Date(start + offset * 86_400_000).toISOString().slice(0, 10) }));
  },
};

// Arbitraries for the bridge's own controls in tests/test_fastcheck_bridge.py.
const controls = {
  bridge_control_integer: () => fc.record({ value: fc.integer({ min: 0, max: 10_000 }) }),
};

const properties = { ...controls, ...require('./properties.cjs')(helpers) };

const input = readline.createInterface({ input: process.stdin });
const lines = input[Symbol.asyncIterator]();
const send = (message) => process.stdout.write(JSON.stringify(message) + '\n');

async function receive() {
  const next = await lines.next();
  if (next.done) throw new Error('Python closed the bridge');
  return JSON.parse(next.value);
}

(async () => {
  const config = await receive();
  if (!Object.hasOwn(properties, config.id)) throw new Error(`Unknown property: ${config.id}`);
  // A broken bridge must stop the run, not be shrunk as if it were a counterexample.
  let broken;
  const property = fc.asyncProperty(properties[config.id](config.context), async (value) => {
    if (broken) throw broken;
    let status;
    try {
      send({ type: 'case', value });
      ({ status } = await receive());
      if (!['pass', 'fail', 'discard'].includes(status)) throw new Error(`Unknown verdict: ${status}`);
    } catch (error) {
      broken = error;
      throw error;
    }
    fc.pre(status !== 'discard');
    return status === 'pass';
  });
  const options = { seed: config.seed, numRuns: config.runs, maxSkipsPerRun: 100 };
  if (config.path !== undefined) options.path = config.path;
  const details = await fc.check(property, options);
  if (broken) throw broken;
  const { failed, interrupted, numRuns, numSkips, numShrinks, seed, counterexamplePath, counterexample } = details;
  const error = failed && details.errorInstance ? String(details.errorInstance) : null;
  send({
    type: 'result',
    result: { failed, interrupted, numRuns, numSkips, numShrinks, seed, counterexamplePath, counterexample, error },
  });
  input.close();
})().catch((error) => {
  process.stderr.write(`${error}\n`);
  process.exitCode = 1;
  input.close();
});
