# Contributing

Each component is developed, tested and released from its own directory.

## Command routing

Run the checks from the component directory against its own lockfile.

Every engine under `packages/` runs the same gates, so the commands differ only in the
engine's import package. Substitute the import package from the table below:

```bash
uv run --locked --extra dev ruff check <import-package> tests
uv run --locked --extra dev mypy <import-package>
uv run --locked --extra dev coverage run --branch --source=<import-package> -m pytest
uv run --locked --extra dev coverage report --show-missing
uv run --locked --extra dev --with "pip-audit==2.10.1" pip-audit --local --strict
uv run --locked --extra dev --python 3.12 python -m build
```

| Component | Directory | Import package |
|---|---|---|
| ato-benchmark-compare | `packages/ato-benchmark-compare/` | `atobenchmark` |
| payday-super-checker | `packages/payday-super-checker/` | `paydaysuper` |
| div7a-loan-review | `packages/div7a-loan-review/` | `div7aloan` |
| the-exchequer-tally | `packages/the-exchequer-tally/` | `edwinnixon` |
| solomons-sword | `packages/solomons-sword/` | `louisgoldberg` |
| the-wip-tally | `packages/the-wip-tally/` | `wiptally` |

The other two components keep their own commands:

| Component | Directory | Checks |
|---|---|---|
| Aus Accounting MCP | `apps/aus-accounting-mcp/` | `uv run --locked --extra dev pytest -q`; `uv run --locked --extra dev ruff check aus_accounting_mcp tests`; `uv run --locked --extra dev mypy aus_accounting_mcp` |
| Repository boundaries | `.` | `python -m unittest -v tests/test_boundaries.py` |

The shared toolchain is pinned to one version per tool in every engine's `pyproject.toml`,
so a gate behaves the same wherever it runs. Changing a pin means changing all six and
relocking each component with `uv lock`.

## CI routing

`ci.yml` is the anchor workflow. It carries no path filter, so its required checks always
report, and it runs two things: the MCP application's own gates, and one call of the
reusable `ci-package.yml` for each engine, from a package-name matrix.

- `ci-package.yml` gives every engine the same gates from the engine's own directory:
  `ruff`, `mypy`, `pytest` under branch coverage on Python 3.10, 3.12 and 3.13 (the floor
  and ceiling of the declared `requires-python`), a `pip-audit` dependency audit, and a
  build of the distribution followed by a clean install of the wheel. `ato-benchmark-compare`,
  `payday-super-checker` and `the-wip-tally` add a Windows leg on 3.12.
- The path filter is applied inside each call, by `.github/ci/select_package.py`, rather
  than on the trigger. A change under `packages/<engine>/` runs that engine and skips the
  rest; a change to a root policy file (`AGENTS.md`, `CONTRIBUTING.md`, `README.md`,
  `SECURITY.md`, `IMPORTS.md`, `.editorconfig`, `.gitignore`, `.mailmap`, `.gitattributes`)
  or to anything under `.github/` runs every engine. A run with no usable comparison
  point, such as a dispatch or a new branch, runs every engine.
- The MCP application's tests run on every change, because `ci.yml` has no path filter, so
  an engine change still proves the application that consumes it.
- `.github/ci/<engine>/checks.sh` holds an engine's own source guards, and
  `.github/ci/<engine>/smoke.sh` its checks against the built wheel. Both are optional.
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
