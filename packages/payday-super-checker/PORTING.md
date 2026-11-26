# Porting notes: `generate_calendar.py`

`tools/generate_calendar.py` stays in Python: it depends on a library with no adequate Rust equivalent.

## Why it stays

1. **External upstream dependency (`python-holidays`):** `tools/generate_calendar.py` imports `holidays` (`holidays==0.108` pinned in root `uv.lock` / `pyproject.toml`) to compute national, state and territory Australian statutory holiday calendars (`holidays.Australia(subdiv=..., years=..., categories=("public",))`) before applying the project's SGAA 1992 s 6(1) exclusions and 2028 WA overrides.
2. **Dependency policy:** Neither `Cargo.lock` nor `tests/test_boundaries.py::APPROVED_CRATES` includes an Australian public-holiday crate, and reimplementing multi-decade statutory holiday rules (movable Easter, state proclamations and substitution rules across 8 jurisdictions) in-house would diverge from the pinned `holidays` source without independent authority.
3. **Active repository contracts:**
   - `packages/payday-super-checker/tests/docs.rs` pins the locked command (`uv run --locked python packages/payday-super-checker/tools/generate_calendar.py > packages/payday-super-checker/data/business_days.json`) and asserts `tools/generate_calendar.py` source structure.
   - `tests/test_payday_generate_calendar.py` imports and executes `tools/generate_calendar.py` under `uv` and asserts provenance `holidays.__version__` in generated `_comment` metadata.

## Verification

- `cargo test --locked` in `packages/payday-super-checker/` verifies the runtime engine and `tests/docs.rs` contracts against `data/business_days.json`.
- `uv run --locked --group dev pytest -q tests/test_payday_generate_calendar.py` verifies `tools/generate_calendar.py` against `data/business_days.json`.
