# Changes

## 1.0.0 (unreleased)

Standalone Rust binary, replacing Python package 0.3.2 and its `div7aloan`
import API. The `rate`, `gate`, `myr` and `review` commands keep their flags,
help, validation, text and JSON output, and exit codes. The former API
functions are JSON stdin subcommands, documented in README.md. No benchmark
rate, statutory test, formula, rounding, refusal, reason code or message is
intentionally changed, apart from the 3 refusals added under Documented
divergences.

### Added

- A dated scope note on *Commissioner of Taxation v Bendel* [2026] HCA 18 in the README and
  DISCLAIMER. It adds no calculation.
- `derive` writes a minimum yearly repayment derivation: the input, ten steps
  each naming the ITAA 1936 provision it applies, and the
  `minimum-yearly-repayment` result unchanged. `verify` rederives a saved
  derivation offline and names the first step that differs (exit 1), or
  confirms it (exit 0). Both read only the compiled-in benchmark table.
- `rollforward` reads one year of an amalgamated loan on stdin (the year, the
  year the loan was made, the balance at 1 July and dated repayments) and works
  out the year's interest at the benchmark rate and the amount not repaid at
  30 June under s 109E(7), period by period, in the new `rollforward` module.
  The day count is the ATO's worked example's; the 366-day divisor, rounding
  each period to cents and the nil balance once repayments pass the opening
  balance are the ATO calculator's, and the output says so. In the first year
  after the loan year it follows the worked example's opening balance and says
  that the ATO's preceding paragraph points the other way. It refuses the loan
  year, a year without a reviewed rate, a repayment outside the year, a
  repayment of nil and repayments beyond the balance and the year's interest.
- `wasm/` builds the derivation for `wasm32-unknown-unknown`, exporting
  `alloc`, `dealloc`, `derive` and `verify` over JSON.
- `api::myr_facts` is visible inside the crate for the derivation; the public
  library API is unchanged apart from the new `derivation` and `rollforward`
  modules.

- `tests/verdict_cases.rs` turns the edge cases in a published reviewer verdict,
  under CC BY 4.0, the default in the bounty's terms, into fabricated cases, or
  states why one does not apply. The engine needed no change.

### Documented divergences

- Python imports, dataclasses, enums, exception classes and package version
  lookup are removed. `--version` and the JSON envelope's `version` report the
  Cargo version, 1.0.0; Python 0.3.2 reported `0.3.2`.
- The frozen rate table moves from `div7aloan/data/benchmark_rates.csv` to
  `data/benchmark_rates.csv` and is compiled into the binary. Its manifest URI
  is `data/benchmark_rates.csv`; its SHA-256 is unchanged. A table the JSON API
  loads from a path reports `file:<name>`, as Python did for a table outside
  the package.
- An exception the Python engine did not catch, which printed a traceback and
  exited 1, prints one `error:` line with the traceback's last line and exits
  1: `error: decimal.DivisionByZero` for a reviewed rate too small to move
  `1 + rate` at 60 digits, `error: UnicodeDecodeError: ...` for an override
  file that is not UTF-8, and `error: AttributeError: ...` for an override row
  whose optional field is not a string. A trapped decimal signal prints its
  class alone, without the detail Python's traceback added after it.
- Output is UTF-8 with native text-mode newlines, and lone surrogates from the
  command line are escaped, as before. Python's encoding overrides
  (`PYTHONIOENCODING`, `PYTHONUTF8=0`) are not supported; argparse colour
  (`PYTHON_COLORS`, `NO_COLOR`, `FORCE_COLOR`) and width (`COLUMNS`) controls
  are, and are differential-tested.
- On Unix, a stdout or stderr the binary cannot write (a closed pipe, a full
  disk) is reported as Python's interpreter shutdown reported it: `Exception
  ignored while flushing sys.stdout:` and the OSError on stderr, with exit 120.
  The binary writes its output once, at the end, so this is its policy for every
  write failure; Python could instead fail inside a command when output passed
  its 8 KiB buffer, and report that through the command's own error handler. On
  Windows the binary exits 1 without that message.
- The CSV field size limit is fixed at Python's default, 131,072 characters; a
  library caller could lower it in Python. `load-rows` omits a row's extra
  unnamed cells, which `csv.DictReader` gathered under a `None` key.
- The JSON stdin commands call each function as Python would have been called
  with the same keyword arguments: an omitted field takes the function's default
  (`load-table` without a path reads the frozen table) and an explicit `null` is
  passed as `None`; an omitted required argument is refused as `<name> is
  required`, where Python raised `TypeError`. Every JSON integer stays exact, as
  Python's `int` does, up to the 4,300 digits Python's `json.loads` accepted. The
  boundary refuses input over 8 MiB and lone Unicode surrogate escapes; its own
  refusals exit 2.
- A reason without its code, and comparing a year of income with a bare
  number, can no longer be written: reasons and codes are built as pairs, and
  years are typed.
- The Python/PyPI release workflow and the fast-check property bridge are
  removed. The formula properties run as seeded Rust tests against exact
  rational arithmetic. Until the MCP server links this crate, the uv workspace
  installs the 0.3.2 Python release from PyPI for the MCP server and
  `lodgeit-calculator-adapter`. A signed native publisher remains a separate
  release task.
- The workbook builder is the Rust `build_workbook` example
  (`cargo run --example build_workbook`), replacing `tools/build_workbook.py`;
  its formula tests run under `cargo test`, and it imports the engine constants
  it restates instead of copying them (`facts::TRUE` and `facts::FALSE` are
  public for it). The shipped workbook's tests stay at the repository root in
  `tests/test_div7a_workbook.py`, which holds it to the engine's review frozen
  in `tests/data/workbook-sample.json`. The rebuilt workbook records engine
  1.0.0, shows its PASS, REVIEW and BLOCKED status colours, which the
  openpyxl-built workbook's conditional formats never displayed in Excel, and
  names portrait A4 in each sheet's page setup. Every calculated result is
  unchanged.
- A register review (`review`, `myr` and `review-register`) refuses the minimum
  yearly repayment with `REFUSED_REMAINING_TERM_ABOVE_MAXIMUM_TERM` where the
  gate passed the loan, its security facts establish the s 109N(3) maximum term
  (7 or 25 years, before any s 109N(3A) to (3D) reduction, which can only
  shorten it), and `remaining_term_years`, rounded up, is longer. Python
  0.3.2 worked a repayment from those facts. They cannot both hold: each
  constituent loan's term is within its maximum term (s 109E(3)(b),
  s 109N(1)(c)), the constituent loans share one maximum term (s 109E(3)(c)),
  and the s 109E(6) remaining term is at most the longest of their terms. The
  refusal tells the caller to enter a loan secured and extended under
  s 109E(3A) as the new amalgamated loan s 109E(3B) creates. A row
  whose security facts are unknown, a gate that did not pass and the
  `minimum-yearly-repayment` command are unchanged. The replay applies the 17
  changed generated answers from `tests/data/amendments.json`, leaving the
  frozen Python record unchanged.
- A register review also refuses the minimum yearly repayment, with
  `REFUSED_REMAINING_TERM_ABOVE_LOAN_TERM`, where the gate passed the loan and
  `remaining_term_years`, rounded up, is longer than the row's own
  `maximum_term_years`, rounded up. The s 109E(6) remaining term is the longest
  constituent-loan term less the years since the loan year, so it cannot be
  longer than that term. Python 0.3.2 and earlier 1.0.0 builds worked a lower
  repayment from such a row, such as a 5-year unsecured loan entered with 7
  years remaining. For an amalgamated loan, `maximum_term_years` is the longest
  constituent-loan term. Where the remaining term is also past the maximum term,
  only `REFUSED_REMAINING_TERM_ABOVE_MAXIMUM_TERM` is given. The
  `minimum-yearly-repayment` command, which takes no loan term, is unchanged.
  The replay applies 26 more changed generated answers and updates 1 from the
  earlier change. In 2 of them the refused row was the register's only
  undecided one, so the run now exits 2 instead of 0; those cases set
  `exit_code_changes`, which the replay now requires for any changed exit code.
- A register review also refuses the minimum yearly repayment, with
  `REFUSED_REMAINING_TERM_ABOVE_TERM_LESS_YEARS_ELAPSED`, where the gate passed
  the loan, `year_loan_made` is earlier than the year reviewed, and
  `remaining_term_years`, rounded up, is longer than `maximum_term_years` less
  the years from the end of the loan year to the end of the year before the one
  reviewed, rounded up after the subtraction (s 109E(6) paragraph (b)). Python
  0.3.2 and earlier 1.0.0 builds worked a lower repayment from such a row, such
  as a 7-year loan made in 2020-21 entered with 7 years remaining in 2025-26,
  when at most 3 remain. In the first repayment year nothing has elapsed, so the
  limit is the loan's term and `REFUSED_REMAINING_TERM_ABOVE_LOAN_TERM` covers
  it; where either earlier refusal applies, only it is given. A row without
  `year_loan_made` and the `minimum-yearly-repayment` command are unchanged. The
  years are counted as 12-month years of income, so the count can be out by part
  of a year for a company whose accounting period changed in between; a
  substituted accounting period stays outside v1. The refusal tells the caller
  that a loan secured and extended under s 109E(3A) has the year before the
  later year of income as its `year_loan_made`, because s 109E(3B) treats the
  new amalgamated loan as made just before that year starts. The replay applies
  5 more changed answers and updates 2 from the earlier changes. One is an API
  call, which the replay can now amend by the hash of its frozen answer. Another
  reviews `examples/sample_loans_myr_met.csv`, whose remaining terms are those
  for 2026-27, for 2027-28: all 3 rows are now refused and the run exits 2
  instead of 0.

### Verification

A differential harness exported base 83288b0, ran its pytest suite and captured
the calls and command runs 204 test functions made (up to 60 per test), then
compared Python 3.15.0 with the Rust binary on those and on generated API calls
and command runs: registers with odd cells, overrides valid and malformed,
argparse errors, colour and width settings. A deterministic group adds every
counterexample from the final review (special operands in the formula, gate
verdicts, non-mapping rows, argument presence, rate-table read order, argparse
edge cases). All 3,622 results match byte for
byte after 1,230 documented adaptations (the version, table URI and traceback
lines above). They
are frozen in `tests/data/python-cases.json` and replayed by `tests/parity.rs`.
`tests/BEHAVIOURS.md` maps each original test to its Rust coverage.

### Dependencies and reuse

Python 0.3.2 had no third-party runtime dependencies. The Decimal emulation is
The WIP Tally's BigUint port of Python's `decimal`, with Australian Tax
Calculators' per-thread precision, extended here with ROUND_CEILING and
libmpdec's integer power (`_mpd_qpow_uint`, with its power-of-one and
certain-underflow checks). The CSV reader, JSON reader, path and OSError text,
argparse and terminal helpers come from The WIP Tally's port. Rust
num-bigint, num-integer and num-traits implement exact arithmetic; serde_json,
with insertion order preserved, writes the JSON; sha2 digests the rate table;
libc reads Unix terminal state. This component owns its copies and depends on
no other engine.
