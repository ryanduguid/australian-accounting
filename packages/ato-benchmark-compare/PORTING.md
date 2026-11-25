# Porting notes: dataset and page builders

`examples/build_dataset.rs` replaces `tools/build_dataset.py` and `examples/build_other_benchmarks.rs` replaces `tools/build_other_benchmarks.py`. Both compile only with `[dev-dependencies]`, keeping production `[dependencies]` and `Cargo.lock` unchanged.

## Item correspondence

### `build_dataset` (`examples/build_dataset.rs`)

| Python (`tools/build_dataset.py`) | Rust (`examples/build_dataset.rs`) | Notes |
|---|---|---|
| `openpyxl.load_workbook(..., data_only=True)` | `read_first_sheet_rows(payload)` | Uses existing dev-dependencies `zip` and `roxmltree`, storing columns `1..=16` sparsely in a `BTreeMap` with checked row/column indices, reproducing openpyxl `_cast_number` (`Decimal(str(float))` via `raw::float_repr` for decimals/exponents, integer digits otherwise), `sharedStrings.xml` (`x005F_` stripping), `inlineStr` (verbatim), bools and sparse row handling |
| `money(text)` | `money(text)` | Uses `ato_benchmark_compare::decimal::Decimal` |
| `ratio(value, where)` | `ratio(value, where_)` | Validates finite `[0, 10]` decimal range; preserves string decimals verbatim and IEEE floats via `raw::float_repr` |
| `parse_range_label(label, where)` | `parse_range_label(label, where_)` | Matches `$low - $high` (`-`, `\u2013`, `\u2014`) and `More than $low` (case-insensitive) |
| `pair(min, max, where)` | `pair(minimum, maximum, where_)` | Enforces both ends published and `min <= max` |
| `build_bands(row, name, anomalies)` | `build_bands(row, name, anomalies)` | Makes adjoining bands contiguous (`turnover_from_inclusive: false`), logs `anomaly:` warnings on band gaps |
| `build(xlsx, args)` + `main()` | `build(xlsx, args)` + `main()` | Prints `wrote <out>: <N> business types, sha256 <16-hex>` on stdout, `anomaly:` lines on stderr, JSON with `indent=1` and `ensure_ascii=False` |

### `build_other_benchmarks` (`examples/build_other_benchmarks.rs`)

| Python (`tools/build_other_benchmarks.py`) | Rust (`examples/build_other_benchmarks.rs`) | Notes |
|---|---|---|
| `normalise(text)` | `ato_benchmark_compare::dataset::normalise` | Direct reuse of the crate's NFKD + casefold + punctuation folding |
| `canonical_digest(text)` | `ato_benchmark_compare::dataset::canonical_digest` | Direct reuse of the crate's canonical JSON SHA-256 |
| `slug`, `label_key`, `ratio` | `slug`, `label_key`, `ratio` | Exact Unicode space folding, dash normalisation and `Decimal / 100` |
| `parse_section`, `parse_page` | `parse_section`, `parse_page` | Preserves curly-quote / acute-accent normalisation, CPython `str.splitlines()`, average skipping and unrecognised-heading checks |
| `page_entry` | `page_entry` | Separates `Error::Mismatch` (excluded page) from `Error::Build` (hard build failure) |
| `build(...)` + `main()` | `build(...)` + `main()` | Preserves `MAX_EXCLUDED = 5`, `--expect-excluded` set equality, `wrote ...` stdout and `excluded: ...` stderr |

## Test correspondence

- All builder unit tests previously in `tests/test_ato_page_builder.py` now live in `examples/build_other_benchmarks.rs` (`[[example]] test = true`, run under `cargo test --locked`).
- `tests/test_ato_page_builder.py` remains at the repository root to verify the checked-in `data/other-benchmarks-2023-24.json` counts, workbook matching and canonical SHA-256 using Python stdlib only.
- `examples/build_dataset.rs` (`[[example]] test = true`) adds unit tests covering shipped dataset byte-exact round-trips, `openpyxl`-equivalent OOXML numeric/inline-string parsing, gap anomalies and refusal messages.

## Intended divergences

- In `build_dataset.py`, an uncaught `BuildError`, `InvalidOperation`, missing file or malformed archive produced a Python traceback and exited `1`. `examples/build_dataset.rs` prints one `error: <message>` line to stderr and exits `1`, matching `build_other_benchmarks` and `ato-benchmark-compare` CLI error handling.
- In `src/dataset.rs`, the existing `tools/build_other_benchmarks.py` error message string is left unchanged because `tests/parity.rs` and `tests/data/python-cases.json` hold the runtime engine byte-for-byte to the frozen Python 0.2.2 corpus.

## Quirks kept

- JSON output uses 1-space indentation (`indent=1`, `ensure_ascii=False`, trailing `\n`) via `write_json_indented` with `serde_json` `preserve_order`.
- Stdout and stderr translate `\n` to `\r\n` on Windows via `cli::output_bytes`, whilst written JSON files stay LF-only; CLI paths print via `py::PyPath` and decode arguments via `cli::decode_os`.
- `build_dataset` creates parent directories for `--out`; `build_other_benchmarks` writes directly to `--out` without creating missing parent directories, matching the original scripts.
- CLI flags exit `2` on invalid or missing required arguments and `0` on `-h` / `--help`.

## Verification

- `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`, `cargo test --locked`, and `cargo llvm-cov --locked --fail-under-lines 91` on Linux (`ll`).
- Differential parity harness against `origin/main` Python scripts: 42 inputs for `build_other_benchmarks` and 42 inputs for `build_dataset` (84 total here, plus 31 for `build_provision_list` = 115 across all 3 tools), 0 unexplained differences.
