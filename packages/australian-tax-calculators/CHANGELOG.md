# Changes

## 1.0.0 (unreleased)

Standalone Rust binary, replacing Python package 0.2.0 and its `austaxcalc`
import API. The fourteen worksheet functions, `worksheet_catalogue()` and
`periods()` are JSON stdin subcommands, documented in README.md. No worksheet
method, rounding, period, scope statement, source, warning or refusal message is
intentionally changed.

### Documented divergences

- Python imports, keyword signatures, the `metadata` dictionaries and package
  version lookup are removed. `engine_version` in every result is the Cargo
  version, 1.0.0; Python 0.2.0 reported `0.2.0`. That field is the only
  difference in the 3,247 frozen results that carry it.
- Amounts and ratios arrive as decimal strings. A JSON number or `null` reaches
  the worksheet as the int, float or `None` it would have been in Python and is
  refused with the original message; text that is not a decimal is refused as
  `decimal.InvalidOperation`, as `Decimal(text)` raised it. Flags, counts, ages,
  days, quarters and scales are JSON values checked by the original type rules,
  so `2.0` is not a scale and `true` is not an integer. An integer of up to 4,300
  digits, the limit Python's `json.loads` applied, is an `int`, as in Python, and a list or object period is quoted in the
  refusal as Python's `repr()` wrote it.
- The JSON boundary refuses input over 8 MiB, a decimal string over 8,192 bytes
  and a missing argument (`<name> is required`; Python raised `TypeError`). JSON
  uses serde_json's nesting limit and refuses lone Unicode surrogate escapes.
- `periods` with a worksheet name it does not know is refused as
  `unknown worksheet '<name>'`, a missing `kind` as `kind is required` and a
  non-string `kind` as `kind must be a string`; Python raised `KeyError` or
  `TypeError`.
- A field a worksheet does not take is ignored; Python raised `TypeError` for an
  unexpected keyword argument.
- Results are compact JSON on stdout with native text-mode newlines. A refusal
  is one `error:` line on stderr with exit 2.
- The Python/PyPI release workflow, installed-distribution smoke script and the
  fast-check property bridge are removed. The properties run as seeded Rust
  tests over the same input domains. Until the MCP server links this crate,
  the uv workspace installs the 0.2.0 Python release from PyPI for the MCP
  server and `lodgeit-calculator-adapter`. A signed native publisher remains a separate
  release task.
- The ATO moved and retitled its study loan page to "Study and training loan
  minimum repayment income and rates", which now says "minimum repayment
  income" where it said "repayment threshold". Read again on 20 October 2026,
  it prints the same figures, so no calculation changes. Every
  `study-loan-repayment` result cites the new address, with `source_checked`
  `2026-10-20`; the `worksheet-catalogue` entry also carries the new title and
  the page's 13 October 2026 update. `tests/parity.rs` applies these changes to
  the 227 `study-loan-repayment` and 10 `worksheet-catalogue` frozen results.
- The ATO moved its "Lump sum payment in arrears tax offsets" page to "Lump
  sum payment in arrears" under income you must declare; the old address
  redirected there on 26 October 2026. The fourth `medicare-levy` source and
  the s 9A refusal message cite the new address, with no calculation change.
  `tests/parity.rs` applies this to the 162 `medicare-levy` results, 56
  `medicare-levy` refusals and 10 `worksheet-catalogue` frozen results.

### Verification

A differential harness exported base 83288b0, ran its pytest suite and captured
the calls each test made, capped at 60 per test, then compared Python 3.15.0
with the Rust binary on those calls, 2,008 generated cases, 1,215 cases at every
statutory threshold in the base tables and its neighbours (uncapped), and 23 of
the final review's counterexamples. All 4,459 results
match byte for byte, including key order, after the `engine_version` adaptation.
They are frozen in `tests/data/python-api.json` and replayed by
`tests/parity.rs`. `tests/tables.rs` and `tests/properties.rs` run the table
checks, every value in the ATO's Schedule 1 sample data, the seeded properties
and the whole-range sweeps in full. `tests/BEHAVIOURS.md` maps each of the 109
original tests to its Rust coverage.

### Dependencies and reuse

Python 0.2.0 had no third-party runtime dependencies. Its Decimal dependency is
replaced by the BigUint-based Python Decimal compatibility code from The WIP
Tally's port (adapted from The Exchequer Tally and accounting-review-pipeline's
elizabeth-anne-alexander port, Apache-2.0), extended here with a per-thread
precision for the worksheets' 40-digit `localcontext`, ROUND_DOWN, remainder and
integral rounding. Rust num-bigint, num-integer and num-traits implement exact
coefficient arithmetic; serde_json, with insertion order preserved, provides the
JSON boundary. Scope statements and sources in `src/texts.rs` are generated from
the base revision. This component owns its copies and depends on no other
engine.
