# Releasing

This project is a preparation aid. A release is a tagged source snapshot, not a
determination, and it does not authorise anyone to skip the current-source check below.

## Before tagging

1. Read every ATO page in `SOURCES` in `lodgmentcalendar/rules.py` and the holiday
   page in `lodgmentcalendar/holidays.py` live. If a date, condition or table moved, update
   the rule, its test, `SOURCE_CHECKED` and the primary-source review in the same commit.
2. `uv run --locked --extra dev pytest --cov --cov-branch`
3. `uv run --locked --extra dev ruff check .`
4. `uv run --locked --extra dev pyright`
5. Run the README example and compare its output with the README.

## Version

The version source is `[project].version` in `pyproject.toml`. Keep `__version__` in
`lodgmentcalendar/_version.py` aligned with it; the package identity test checks both
copies. Update the release-note heading, citation and lockfiles in the same commit.
