# Porting notes: `build_provision_list`

`examples/build_provision_list.rs` replaces `tools/build_provision_list.py`.

## Item correspondence

| Python (`tools/build_provision_list.py`) | Rust (`examples/build_provision_list.rs`) | Notes |
|---|---|---|
| `PACKAGE` | `package_root()` | `CARGO_MANIFEST_DIR`, or `RATES_ROOT` when built with `--features differential` |
| `citations()` | `citations(package)` | Reads `data/*.json` sorted by filename; returns sorted unique `(title_id, provision)` pairs |
| `heading_number(row)` | `heading_number(row)` | Replaces U+2011 with ASCII `-` and splits on CPython Unicode whitespace (`py::is_space`) |
| `main()` | `build(package, corpus)` + `run()` | Reads `sources.json` and `markdown/<title_id>/sections.jsonl`, hashes raw bytes with SHA-256, checks `found.len() == 1`, writes `legislation-provisions.json` (`indent=2`, `ensure_ascii=False`, trailing `\n`) |

## Test correspondence

- `tests/test_dated_rates.py::LegislationCitationTests` at the repository root continues to check `legislation-provisions.json` against `data/*.json`.
- `examples/build_provision_list.rs` (`[[example]] test = true`, run under `cargo test --locked`) tests `heading_number`, round-tripping `legislation-provisions.json` byte-for-byte, `citations` against the checked-in list, end-to-end writing and 0-row / 2-row refusal messages.

## Verification

- `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`, `cargo clippy --locked --all-targets --all-features -- -D warnings`, `cargo test --locked`, `cargo test --locked --features differential --example build_provision_list`, and `cargo llvm-cov --locked --fail-under-lines 77` on Linux (`ll`).
- Differential parity harness: 31 inputs compared against `origin/main` `tools/build_provision_list.py` (including the shipped `data/*.json` records and 30 seeded generated corpora with CRLF/LF lines, Unicode spaces, U+2011 hyphens, missing rows and duplicate rows), 0 unexplained differences.

## Intended divergences

- Missing or extra CLI arguments print `usage: build_provision_list <corpus folder>` to stderr and exit `1` instead of raising an uncaught Python `IndexError` traceback.

## Quirks kept

- Heading tokens normalise U+2011 non-breaking hyphens to ASCII `-` whilst keeping all other characters verbatim, matching `tests/test_dated_rates.py`.
- `sections.jsonl` lines split on CR, LF and CRLF (`bytes.splitlines()` semantics) and compute SHA-256 over raw file bytes before decoding.
- Under `--features differential`, `RATES_ROOT` overrides the package folder path cleanly without touching `legislation-provisions.json` in the source checkout.
- Stderr translates `\n` to `\r\n` on Windows via `output_bytes`, matching CPython text-mode stderr whilst `legislation-provisions.json` remains LF-only.
