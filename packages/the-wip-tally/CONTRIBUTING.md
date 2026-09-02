# Contributing

This tool builds a construction WIP schedule from a contract CSV. A person reads
the result and decides what it means. Keep that boundary: nothing here should
present a computed asset, liability or fade figure as a conclusion that a
contract is onerous, that a claim is enforceable, or that revenue may be
recognised over time, and nothing should post journals or lodge anything.

## Data boundary

- Use invented data. The `.gitignore` blocks the file names a real job-cost
  export arrives under, including `wip*.csv`, `job-cost*.csv`, `client-data/`
  and spreadsheet files, with exceptions for `examples/` and `tests/fixtures/`.
- No client, taxpayer or employee data, no ABNs tied to a real business, no
  screenshots of a live ledger, no credentials.
- Check what a fixture implies as well as what it says. An invented job with a
  real client's contract number is still that client's contract.

## Standards and figures

- Trace every rule to a primary source and cite it in the pull request. The
  rules currently implemented are recorded in
  `docs/aasb-15-source-notes-2026-08-25.md`.
- Confirm the operative AASB 15 compilation at standards.aasb.gov.au before
  changing a paragraph citation. Compilations are remade and renumbered.
- Amounts stay in `Decimal` end to end. A float anywhere in the progress path
  will be rejected.

## Local verification

Python 3.10 or newer. The runtime imports nothing outside the standard library.
`uv` manages the development environment and the lock file is committed.

```bash
uv sync --locked --extra dev --python 3.12
uv run --locked --extra dev --python 3.12 pytest
uv run --locked --extra dev --python 3.12 python -m build
uv run --locked --extra dev --with "pip-audit==2.10.1" pip-audit --local --strict
```

CI repeats this on Ubuntu with Python 3.10 and 3.13, and on Windows with 3.12.
Keep runtime strings ASCII: on Windows, redirected stdout uses the machine's
ANSI codepage rather than UTF-8.

## Pull requests

- Name the rule you changed and the test that pins it. Run that test against
  the old code first. If it passes there too, it is not testing your change.
- When you change a rule, search for everything else that states it. The README
  table, a docstring and a console string can all keep asserting the old rule
  long after the code has moved.
- A displayed figure and its verdict must never disagree.

For a suspected security vulnerability, follow [SECURITY.md](SECURITY.md)
rather than opening an issue.
