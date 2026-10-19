# Releasing

This project is a review aid. A release is a tagged source snapshot, not a
determination, and it does not authorise anyone to skip the current-source check
in `docs/aasb-15-source-notes-2026-08-25.md`.

## Before tagging

1. Confirm the operative AASB 15 compilation at standards.aasb.gov.au. If
   paragraph numbers moved, update the source notes and any README citations in
   the same commit as the code.
2. `cargo fmt --check`
3. `cargo clippy --locked --all-targets -- -D warnings`
4. `cargo test --locked`
5. Run the sample:

```bash
wip-tally schedule examples/sample_contracts.csv --as-at 2026-08-31
```

The sample is supposed to exit 2.

## Version

The version source is `[package].version` in `Cargo.toml`. Update the release-note
heading, `CITATION.cff` and `Cargo.lock` in the same commit. Native signing and
publication need a separate authorised release change; no publisher is
configured for 1.0.0.
