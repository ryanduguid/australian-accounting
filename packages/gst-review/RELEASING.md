# Releasing

This project is a review aid. A release is a tagged source snapshot, not a
determination, and it does not authorise anyone to skip the current-source check
below.

## Before tagging

1. Confirm the GST Act compilation cited in `gstreview/coding.py` is still the one in
   force at legislation.gov.au, and that the ATO BAS pages cited in `gstreview/bas.py`
   still say what the README relies on: G labels follow the G1 choice of including GST,
   and ledger totals round down to whole dollars. If either moved, update the source
   links, `SOURCE_CHECKED` and the README in the same commit as the code.
2. `uv run --locked --extra dev pytest --cov --cov-branch`
3. `uv run --locked --extra dev ruff check .`
4. `uv run --locked --extra dev pyright`
5. Run both README examples and compare their output with the README.

## Version

The version source is `[project].version` in `pyproject.toml`. Keep `__version__` in
`gstreview/_version.py` aligned with it; the package identity test checks both copies.
Update the release-note heading, citation and lockfiles in the same commit.
