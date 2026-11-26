# Porting notes: `build_provision_list`

`examples/build_provision_list.rs` replaces `tools/build_provision_list.py`.

## Item correspondence

| Python (`tools/build_provision_list.py`) | Rust (`examples/build_provision_list.rs`) | Notes |
|---|---|---|
| `PACKAGE` | `package_root()` | `CARGO_MANIFEST_DIR`, or `RATES_ROOT` when built with `--features differential` |
| `citations()` | `citations(package)` | Reads `data/*.json` sorted by filename; returns sorted unique `(title_id, provision)` pairs |
| `heading_number(row)` | `heading_number(row)` | Replaces U+2011 with ASCII `-` and splits on CPython Unicode whitespace (`py::is_space`) |
| `main()` | `build(package, corpus)` + `run()` | Reads `sources.json` and `markdown/<title_id>/sections.jsonl`, hashes raw bytes with SHA-256, checks `found.len() == 1`, writes `legislation-provisions.json` (`indent=2`, `ensure_ascii=False`, trailing `\n`) |

## Tests and parity

- `tests/test_dated_rates.py::LegislationCitationTests` at the repository root still checks `legislation-provisions.json` against `data/*.json`.
- `tests/data/build_provision_list-python.json` holds the answers `tools/build_provision_list.py` gave, recorded under CPython 3.15 on 7 cases: the shipped records against a corpus built from the shipped list, CRLF lines, Unicode spaces in headings, a provision with no row and one with 2 rows, a corpus without `sources.json` and no arguments. The example's `port_answers_as_the_python_script_did` test (`[[example]] test = true`, run by `cargo test --locked`) lays out each case, runs the port and compares the exit code, stderr and the written `legislation-provisions.json` byte for byte. Other tests cover `heading_number`, the shipped file's byte-exact round trip and its citations.

## Verification

`cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings` and `cargo test --locked` in this folder; CI runs them on Ubuntu and Windows.

## Intended divergences

- Missing or extra arguments print `usage: build_provision_list <corpus folder>` and exit 1, where Python raised an `IndexError` traceback (also exit 1). A missing or unreadable corpus file is one error line instead of a traceback.

## Quirks kept

- Heading tokens normalise U+2011 non-breaking hyphens to ASCII `-` whilst keeping all other characters verbatim, matching `tests/test_dated_rates.py`.
- `sections.jsonl` lines split on CR, LF and CRLF (`bytes.splitlines()` semantics) and compute SHA-256 over raw file bytes before decoding.
- Under `--features differential`, `RATES_ROOT` overrides the package folder path cleanly without touching `legislation-provisions.json` in the source checkout.
- Stderr translates `\n` to `\r\n` on Windows via `output_bytes`, matching CPython text-mode stderr whilst `legislation-provisions.json` remains LF-only.
