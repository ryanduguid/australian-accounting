# Releasing

This project is a review aid. A release is a tagged source snapshot, not a
determination, and it does not authorise anyone to skip the current-source check
in `docs/aasb-15-source-notes-2026-08-25.md`.

## Before tagging

1. Confirm the operative AASB 15 compilation at standards.aasb.gov.au. If
   paragraph numbers moved, update the source notes and any README citations in
   the same commit as the code.
2. `uv run --locked --extra dev pytest`
3. `uv run --locked --extra dev ruff check wiptally tests`
4. `uv run --locked --extra dev mypy wiptally`
5. Run the sample:

```bash
wip-tally schedule examples/sample_contracts.csv --as-at 2026-08-31
```

The sample is supposed to exit 2.

## Version

The version string lives in `wiptally/__init__.py`. Hatch reads it from there.
Do not duplicate it in `pyproject.toml`.
