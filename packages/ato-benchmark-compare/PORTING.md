# Porting notes: dataset and page builders

`examples/build_dataset/` replaces `tools/build_dataset.py` and `examples/build_other_benchmarks/` replaces `tools/build_other_benchmarks.py`. Both compile only with `[dev-dependencies]`, keeping production `[dependencies]` and `Cargo.lock` unchanged. `examples/shared/` holds what both share: an argparse-compatible command line, the `indent=1` JSON writer and the parity harness.

## Item correspondence

### `build_dataset` (`examples/build_dataset/`)

| Python (`tools/build_dataset.py`) | Rust | Notes |
|---|---|---|
| `openpyxl.load_workbook(..., data_only=True)` | `xlsx.rs`: `read_first_sheet_rows(payload)` | Reads the first worksheet with the existing dev-dependencies `zip` and `roxmltree`, storing columns 1 to 16 sparsely with checked row and column indices, and reproduces openpyxl 3.1.5's number casting (`Decimal(str(float))` through `raw::float_repr`), shared strings (`x005F_` stripped), inline strings (verbatim), booleans and sparse rows |
| `money`, `ratio`, `parse_range_label`, `pair` | `bands.rs` | Same names; ratios must be finite and within `[0, 10]` |
| `build_bands` | `bands.rs`: `build_bands`, `band_start`, `band_entry` | Adjoining bands are made contiguous (`turnover_from_inclusive: false`) and a gap other than 0 or 1 dollar is an `anomaly:` line |
| `build`, `main` | `bands.rs`: `build`; `main.rs`: `run` | `wrote <out>: <N> business types, sha256 <16 hex>` on stdout, `anomaly:` lines on stderr, JSON with `indent=1` and `ensure_ascii=False` |

### `build_other_benchmarks` (`examples/build_other_benchmarks/`)

| Python (`tools/build_other_benchmarks.py`) | Rust | Notes |
|---|---|---|
| `normalise`, `canonical_digest` | `ato_benchmark_compare::dataset` | The crate's own functions, as before |
| `slug`, `label_key`, `ratio` and the line patterns | `text.rs` | Unicode space folding, dash normalisation, `Decimal / 100` and CPython `str.splitlines()` |
| `parse_section`, `parse_page` | `page.rs` | Curly-quote and acute-accent folding, `Average` blocks skipped, unrecognised headings refused |
| `page_entry`, `build`, `main` | `build.rs`, `main.rs` | `Error::Mismatch` excludes a page, `Error::Build` stops the build; `MAX_EXCLUDED = 5`, `--expect-excluded` set equality, `wrote ...` on stdout and `excluded: ...` on stderr |

## Parity

`tests/data/builders/` holds the answers the Python scripts gave, recorded by running them under CPython 3.15 and openpyxl 3.1.5 on 24 `build_dataset` cases (17 workbooks in `xlsx/`, written as raw OOXML so shared strings, inline strings, sparse rows and number spellings are exercised) and 31 `build_other_benchmarks` cases (every refusal the page builder has, plus agreeing, excluded, aliased and CRLF pages). Each example's `port_answers_as_the_python_script_did` test lays out every case's inputs, runs the port in process and compares the exit code, stdout, stderr and every file written, byte for byte. The `about` field of each file says which cases compare less than everything and why.

## Intended divergences

- An uncaught Python exception printed a traceback and exited 1. The port prints one `error: <message>` line and exits 1; for `BuildError` the message is the one Python raised.
- Usage and help text name the program `build_dataset` or `build_other_benchmarks`, not the script file, and are not wrapped to the terminal width. The `error:` text after the usage line, the exit code 2 and argparse's prefix matching, ambiguity and missing-value rules are kept.
- In `src/dataset.rs`, the message that names `tools/build_other_benchmarks.py` is unchanged because `tests/parity.rs` holds the engine byte for byte to the frozen Python 0.2.2 corpus.

## Quirks kept

- JSON files are LF only with a trailing newline; stdout and stderr use CRLF on Windows, as CPython's text mode does. Arguments decode as CPython decodes `sys.argv`.
- `build_dataset` creates the folder for `--out`; `build_other_benchmarks` does not, as the scripts did.

## Verification

`cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings` and `cargo test --locked` in this folder; CI runs them on Ubuntu and Windows.
