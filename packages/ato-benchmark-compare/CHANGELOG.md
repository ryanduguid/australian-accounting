# Changes

## 1.0.0 (unreleased)

Standalone Rust binary, replacing Python package 0.2.2 and its `atobenchmark`
import API. The `industries`, `show`, `buckets`, `map` and `compare` commands
keep their flags, help, validation, text and JSON output, written files and exit
codes. The former API functions are JSON stdin subcommands, documented in
README.md. No benchmark figure, ratio rule, rounding, refusal or message is
intentionally changed.

### Migration

- `atobenchmark` imports become JSON stdin subcommands. The Python `compare()`
  is `compare-figures`, because `compare` remains the command-line comparison.
  `to_dict`, `to_evidenced_dict`, `render_text` and `evidenced_ratio` are its
  `output` choices.
- `route()`'s `RoutedTotals` becomes `route`'s `totals` and `supplied_buckets`;
  pass `supplied_buckets` as `totals_supplied` to `compute` or
  `compare-figures` to keep the routed presence record.
- `examples/benchmark_history.py` is `examples/benchmark_history.rs`, run with
  `cargo run --release --example benchmark_history -- ...`; its flags and JSON
  output are unchanged.
- The data and workbook build tools stay in Python. `build_other_benchmarks.py`
  carries its own copies of `normalise` and `canonical_digest`, and
  `build_workbook.py` reads the bands through the binary's `load-dataset`. Their
  tests moved to the repository root, as `tests/test_ato_page_builder.py` and
  `tests/test_ato_workbook.py`.

### Verification

`tests/parity.rs` replays 3,818 frozen cases: every top-level engine call and
command run the Python 0.2.2 tests made, and generated calls and runs, each with
the files it read, compared byte for byte with key order kept, and every file a
run wrote checked against Python's. `tests/properties.rs` keeps the Python
property tests' seeds, examples and input domains at 500 runs each, and
`tests/workbook.rs` holds the frozen bakery comparison the workbook test reads.
The Python tests not replayed by the corpus tested Python packaging, imports, the
fast-check bridge, module layout and release workflows, which no longer exist, or
the Python build tools, which moved to the repository root.

### Documented divergences

- Python imports, dataclasses, exception classes and package version lookup are
  removed. `--version` reports the Cargo version, 1.0.0; Python 0.2.2 reported
  `0.2.2`.
- The data files move from `atobenchmark/data/` to `data/` and are compiled into
  the binary; their contents and digests are unchanged. An error about a shipped
  file names it as `data/<file>` rather than the installed package path. Such an
  error needs malformed embedded data, so no release can show it.
- An exception the Python engine did not catch, which printed a traceback and
  exited 1, prints one `error:` line with the traceback's last line and exits 1:
  for example `error: _csv.Error: field larger than field limit (131072)` for
  an oversized CSV field, and `error: ValueError: invalid literal for int() with
  base 10: '²'` for an `--amount-column` of a digit `int()` cannot read. A
  trapped decimal signal prints its class alone, `error: decimal.Overflow`,
  without the detail Python's traceback added after it.
- The JSON stdin commands call each function as Python would have been called
  with the same keyword arguments: an omitted field takes the function's default,
  and an explicit `null` is passed as `None`. The JSON API's own refusals exit 2.
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
  library caller could lower it in Python.
- The JSON boundary keeps every JSON integer exact, as Python's `int` does, and
  refuses input over 8 MiB and lone Unicode surrogate escapes. A non-finite
  number (`NaN`, `Infinity`, or a float too large for a double) in a custom
  dataset's metadata is returned as `null`, because JSON output has no spelling
  for it; Python's `json.dumps` wrote `NaN` or `Infinity`.
- Name matching uses Unicode 17.0.0 case folding and decomposition tables, the
  version Python 3.15.0 ships; Python on another version used its own tables.
