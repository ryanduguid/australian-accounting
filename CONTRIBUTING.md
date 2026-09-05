# Contributing

Each component is developed, tested and released from its own directory.

## Command routing

Run the checks from the component directory. The root workflow named in the last column
runs the same commands in CI.

| Component | Directory | Checks | Root workflow |
|---|---|---|---|
| Aus Accounting MCP | `apps/aus-accounting-mcp/` | `uv run --locked --extra dev pytest -q`; `uv run --locked --extra dev ruff check aus_accounting_mcp tests`; `uv run --locked --extra dev mypy aus_accounting_mcp` | `ci.yml` |
| ato-benchmark-compare | `packages/ato-benchmark-compare/` | `uv run --locked --extra dev pytest -q`; `uv run --locked --extra dev ruff check atobenchmark tests`; `uv run --locked --extra dev mypy atobenchmark`; `uv run --locked --extra dev --with "pip-audit==2.10.1" pip-audit --local --strict`; `uv run --locked --extra dev --python 3.12 python -m build` | `ci-ato-benchmark-compare.yml` |
| payday-super-checker | `packages/payday-super-checker/` | `uv run --locked --extra dev pytest -q`; `uv run --locked --extra dev ruff check paydaysuper tests`; `uv run --locked --extra dev mypy paydaysuper`; `uv run --locked --extra dev --with "pip-audit==2.10.1" pip-audit --local --strict`; `uv run --locked --extra dev --python 3.12 python -m build` | `ci-payday-super-checker.yml` |
| div7a-loan-review | `packages/div7a-loan-review/` | `python -m pip install ".[dev]"`; `python -m ruff check div7aloan tests`; `python -m mypy`; `python -m pytest -q`; `python -m build` | `ci-div7a-loan-review.yml` |
| the-exchequer-tally | `packages/the-exchequer-tally/` | `uv run --locked --extra dev ruff check edwinnixon tests`; `uv run --locked --extra dev mypy edwinnixon`; `uv run --locked --extra dev pytest -q` | `ci-the-exchequer-tally.yml` |
| solomons-sword | `packages/solomons-sword/` | `uv run --locked --extra dev ruff check louisgoldberg tests`; `uv run --locked --extra dev mypy louisgoldberg`; `uv run --locked --extra dev pytest -q` | `ci-solomons-sword.yml` |
| the-wip-tally | `packages/the-wip-tally/` | `uv run --locked --extra dev pytest -q`; `uv run --locked --extra dev ruff check wiptally tests`; `uv run --locked --extra dev mypy wiptally`; `uv run --locked --extra dev --with "pip-audit==2.10.1" pip-audit --local --strict`; `uv run --locked --extra dev --python 3.12 python -m build` | `ci-the-wip-tally.yml` |
| Repository boundaries | `.` | `python -m unittest -v tests/test_boundaries.py` | `boundaries.yml` |

## CI routing

- A change under `apps/aus-accounting-mcp/` runs `ci.yml`, which produces the required
  checks `lint`, `test (ubuntu-latest, 3.10)`, `test (ubuntu-latest, 3.12)` and
  `test (windows-latest, 3.12)`.
- A change under `packages/<engine>/` runs that engine's workflow, including its
  `mcp-integration` job (the MCP application's tests), because the MCP application consumes
  published engines.
- A change to a root policy file (`AGENTS.md`, `CONTRIBUTING.md`, `README.md`,
  `SECURITY.md`, `IMPORTS.md`, `.editorconfig`, `.gitignore`, `.mailmap`) or to anything
  under `.github/` runs every component.
- `boundaries.yml` and `codeql.yml` run on every change.
- Workflow files inside component directories are inert historical records of the source
  repositories; only root workflows run.

## Rules

- Keep a change inside one component unless it is a root policy or workflow change.
- Do not move, rename or refactor a component in the same change that alters its
  behaviour.
- Never add a root package manager, root lockfile, shared runtime library, unified
  version or code generator.
- Engines must not import the MCP application or each other, and production code must
  not use relative imports that leave the component directory.
- Use fabricated data only, and follow the component's own `CONTRIBUTING.md` and
  `SECURITY.md`.

## Releases

A release covers one component, from `main`, on the annotated namespaced tag
`<component>/vX.Y.Z`, where `<component>` is both the final segment of the component
directory and the normalised distribution name. The tag triggers only
`.github/workflows/release-<component>.yml`, which calls the pinned Release Policy
reusable workflow with that component's `source-directory` and `tag-prefix`. The policy
checks the tag, the `main` commit, the clean tree, the component's `RELEASE_NOTES.md`
header (`# vX.Y.Z` on the first line), the lockfile and the distribution identity, then
builds, attests and publishes the GitHub release. Every component workflow then publishes the
exact attested distribution to PyPI under the component's own `pypi-<component>`
environment and trusted publisher. The MCP application additionally publishes to the MCP
Registry through `publish-mcp.yml`.

| Component | Tag | Workflow | Version source | PyPI environment |
|---|---|---|---|---|
| aus-accounting-mcp | `aus-accounting-mcp/vX.Y.Z` | `release-aus-accounting-mcp.yml` | `pyproject.toml` | `pypi-aus-accounting-mcp` |
| ato-benchmark-compare | `ato-benchmark-compare/vX.Y.Z` | `release-ato-benchmark-compare.yml` | `atobenchmark/__init__.py` | `pypi-ato-benchmark-compare` |
| payday-super-checker | `payday-super-checker/vX.Y.Z` | `release-payday-super-checker.yml` | `pyproject.toml` | `pypi-payday-super-checker` |
| div7a-loan-review | `div7a-loan-review/vX.Y.Z` | `release-div7a-loan-review.yml` | `pyproject.toml` | `pypi-div7a-loan-review` |
| the-exchequer-tally | `the-exchequer-tally/vX.Y.Z` | `release-the-exchequer-tally.yml` | `pyproject.toml` | `pypi-the-exchequer-tally` |
| solomons-sword | `solomons-sword/vX.Y.Z` | `release-solomons-sword.yml` | `pyproject.toml` | `pypi-solomons-sword` |
| the-wip-tally | `the-wip-tally/vX.Y.Z` | `release-the-wip-tally.yml` | `wiptally/__init__.py` | `pypi-the-wip-tally` |

`IMPORTS.md` records which components still lack a Release Policy prerequisite; their
workflows fail closed until a reviewed component change adds it. Nothing publishes from a
contribution branch, and no tag or release is created without explicit approval.
