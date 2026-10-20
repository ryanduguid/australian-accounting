# Changes

## 1.0.0 (unreleased)

Standalone Rust binary, replacing Python package 0.2.2 and its `paydaysuper`
import API. The check, `import`, `review-pack` and `evidence-pack` commands keep
their flags, help, validation, console output, written files and exit codes.
The former API functions are JSON stdin subcommands, documented in README.md.
No deadline, business-day, GIC, notional-earnings, uplift, rounding, refusal or
message rule is intentionally changed, except the Employment Hero Reconciled
status below.

### Added

- `derive` writes a due-date derivation for one contribution: the input, eight
  steps each naming the SGAA 1992 provision or instrument it applies, and the
  deadline. `verify` rederives a saved derivation offline and names the first
  step that differs (exit 1), or confirms it (exit 0). Both read only the
  bundled calendar.
- `wasm/` builds the derivation for `wasm32-unknown-unknown`, exporting
  `alloc`, `dealloc`, `derive` and `verify` over JSON. Writing an evidence pack
  is refused on targets other than Unix and Windows, which cannot create its
  private folder.
- The public library API is unchanged apart from the new `derivation` module.

### Migration

- `paydaysuper` imports become JSON stdin subcommands named after the function
  with hyphens: `assess`, `render-csv`, `console-summary`, `import-files` and so
  on. `load_report_snapshot` and `parse_report_snapshot` share
  `parse-report-snapshot`, which takes a `path` or the report `text`;
  `profiles.detect` is `detect-profile`. The functions that write files
  (`write_csv`, `write_canonical`, `write_practitioner_pack`,
  `write_evidence_pack`) are reached through the commands that called them.
- Dataclass results become JSON objects with the same field names; decimals are
  strings and dates are `YYYY-MM-DD` strings. `import-files` returns the whole
  import report, its profiles and each orphan's super row included. `assess`
  adds each line's rounded `figures`, the figures the report prints. The whole
  `figures` object is null if any component cannot be rounded to cents (for
  example under an astronomical custom GIC rate), and the raw fields are kept,
  since Python's `assess()` did no rounding.
- The bundled data moves from `paydaysuper/data/` to `data/` and is compiled
  into the binary; its contents and digests are unchanged. Rebuild after editing
  it.
- The calendar generator and the quick-proof renderer stay in Python under
  `tools/`. The calendar generator's tests moved to the repository root, as
  `tests/test_payday_generate_calendar.py`.
- The workbook builder is the Rust `build_workbook` example
  (`cargo run --example build_workbook`), replacing `tools/build_workbook.py`;
  its formula tests run under `cargo test`. The shipped workbook's tests stay
  at the repository root in `tests/test_payday_workbook.py`, which reads the
  engine figures frozen in `tests/data/workbook-sample.json`. The rebuilt
  workbook records engine 1.0.0, shows its PASS, REVIEW and BLOCKED status
  colours, which the openpyxl-built workbook's conditional formats never
  displayed in Excel, and names portrait A4 in each sheet's page setup.
  Every calculated result is unchanged.
- The fuzz target is a cargo-fuzz target over the CSV readers, in `fuzz/`.

### Verification

`tests/parity.rs` replays 4,883 frozen cases, each with the files it read: up
to 60 distinct top-level engine calls and command runs from each Python 0.2.2
test (those the harness could encode as JSON or command-line input), the final
reviews' counterexamples and generated calls and runs. Results are compared byte
for byte with key order kept, and every file a run wrote is checked against
Python's. `tests/properties.rs` keeps the
fast-check properties' input domains as seeded Rust tests, `tests/evaluation.rs`
reruns the evaluation packs and `tests/workbook.rs` holds the frozen workbook
sample. Python tests that frozen inputs cannot replay run natively: unit tests
for those that called a decision phase or built a result by hand, `tests/cli.rs`
for runs that read the machine's clock or name a directory or symlink, and a
`src/rates.rs`, `src/calendar.rs` and `src/profiles.rs` tests that hold every
malformed bundled-data message to Python's. The
rest tested Python packaging, imports, the fast-check bridge and module layout,
which no longer exist, or the Python tools, whose tests moved to the repository
root.
`tests/BEHAVIOURS.md` maps each Python test to its coverage.

### Documented divergences

- The Employment Hero profile treats Beam's Reconciled status as not yet sent
  to the fund. Python 0.2.2 counted it as sent, but Beam's status order puts
  Reconciled (employer money cleared by Beam) before Sent to fund. A Reconciled
  row now gets no remitted date and its payday reads as unfunded, with a
  warning naming the status. The replay applies the changed answers from
  `tests/data/amendments.json`, leaving the frozen Python record unchanged.
- Python imports, dataclasses, exception classes and package version lookup are
  removed. `--version` reports the Cargo version, 1.0.0; Python 0.2.2 reported
  `0.2.2`. The report CSV's NOTE row records the version too, in a written
  report and in the evidence pack's `report.csv`, so those files and the
  digests over them differ from 0.2.2's for the same input.
- The data files are named `data/<file>` in messages and in the evidence pack's
  rate-table manifest (`data/gic_rates.json`), rather than the installed package
  path `paydaysuper/data/<file>`. The manifest digest is unchanged.
- An exception the Python engine did not catch, which printed a traceback and
  exited 1, prints one `error:` line with the traceback's last line and exits 1.
  A trapped decimal signal prints its class alone, without the detail Python's
  traceback added after it.
- The JSON stdin commands take typed fields. An omitted field takes the
  Python function's or dataclass's default, and a field Python required must be
  given. `null` is accepted only where the Python parameter or field was
  optional, and means `None` there. A `null` anywhere else, a field of another
  JSON type, and an integer outside the signed 64-bit range are refused with the
  field's name and exit 2, where Python's dataclasses carried such a value
  through or failed later with their own error. A nullable argument Python
  required, such as `money`'s `value`, must still be given. The `calendar`,
  `quarters` and `mapping` selectors take the bundled table or the default
  mapping when absent or null, and are refused when of another type. Python's
  literal-boolean checks are kept, and `join` still reads its column flags by
  truthiness.
- A command name given as the only argument runs that JSON command, so a
  contribution file named exactly like one (for example `assess`) must be
  checked as `./assess`.
- Paths must be valid Unicode. Python accepted any bytes on Unix through
  surrogate escapes.
- Output is UTF-8 with native text-mode newlines, and lone surrogates from the
  command line are escaped, as before. Python's encoding overrides
  (`PYTHONIOENCODING`, `PYTHONUTF8=0`) are not supported; argparse colour
  (`PYTHON_COLORS`, `NO_COLOR`, `FORCE_COLOR`) and width (`COLUMNS`) controls
  are, and are differential-tested.
- On Unix, a stdout or stderr the binary cannot write (a closed pipe, a full
  disk) is reported as Python's interpreter shutdown reported it: `Exception
  ignored while flushing sys.stdout:` and the OSError on stderr, with exit 120.
  The binary writes its output once, at the end, so this is its policy for every
  write failure. On Windows the binary exits 1 without that message.
- The CSV field size limit is fixed at Python's default, 131,072 characters; a
  library caller could lower it in Python.
- The JSON boundary keeps every JSON integer exact, as Python's `int` does, and
  refuses input over 8 MiB and lone Unicode surrogate escapes. An integer of
  more than 4,300 digits is refused as Python's `json.loads` refused it. Deeply
  nested JSON is not refused for depth, where Python raised `RecursionError`.
- Date parsing follows the `strptime` patterns and `fromisoformat` shapes of
  Python 3.15.0, and name and header normalisation use Unicode 17.0.0 tables,
  the version Python 3.15.0 ships; Python on another version used its own.
