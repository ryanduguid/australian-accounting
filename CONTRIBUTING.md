# Contributing

Each component is developed, tested and released from its own directory. The root
holds a development entrypoint so a fresh clone can be set up and verified in one
command; it does not change where a component is developed or released from.

## Quick start

From a fresh clone:

```
uv sync        # install every component and the shared toolchain
just test      # every component's suite, plus the repository boundary checks
```

`uv sync` creates one `.venv` at the root and installs all seven components into it
as editable workspace members. `apps/aus-accounting-mcp` therefore imports
`atobenchmark`, `paydaysuper` and `div7aloan` from the checked-out tree rather than
from their last PyPI release, with no per-component install step.

`just` is optional tooling; install it with `uv tool install rust-just`. The recipes
are `setup`, `lint`, `typecheck`, `test` and `check` (the last three together). Each
loops over the per-component commands in the table below, which remain the authority;
`just` runs them from one place, it does not replace them.

`just check` is the fast local pass, not a CI equivalent. It runs ruff, mypy and
pytest. CI additionally runs, per component, the dependency audit, the distribution
build, the installed-wheel and sdist smoke tests and changed-line coverage listed in
that table and in `ci-package.yml`. A green `just check` is not a green CI.

Two consequences of the workspace are worth knowing before you run a component's
own commands:

- `uv run --locked` from a component directory now validates the root `uv.lock`,
  not the component's. Regenerate it with `uv lock` at the root after changing any
  component's dependencies, and commit it.
- The six engines pin the shared toolchain (`ruff`, `mypy`, `pytest`, `coverage`, `build`)
  to one exact version each, so the workspace resolves them without an override. One
  workspace cannot hold two exact pins of the same tool, so keep the six identical when
  changing one.

## Command routing

Run the checks from the component directory. Inside the workspace, `uv run --locked`
validates the root `uv.lock`; the component's own lockfile is what builds and releases it
on its own.

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
so a gate behaves the same wherever it runs, and the workspace resolution takes the same
versions for `just`. Changing a pin means changing all six, then relocking twice: `uv lock`
at the root, which is what CI validates, and each engine's own lockfile, which is what
builds and releases it alone. Inside the workspace `uv lock` always writes the root lock,
so an engine's own lockfile is regenerated from a copy of the engine outside it:

```bash
tmp=$(mktemp -d) && cp -r packages/<engine>/. "$tmp" && (cd "$tmp" && uv lock) \
  && cp "$tmp/uv.lock" packages/<engine>/uv.lock
```

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
  `SECURITY.md`, `IMPORTS.md`, `.editorconfig`, `.gitignore`, `.mailmap`, `.gitattributes`),
  to a root workspace file (`pyproject.toml`, `uv.lock`, `justfile`) or to anything under
  `.github/` runs every engine. The root workspace files are in that list because the root
  `uv.lock` is what `uv run --locked` validates from inside every component directory. A
  run with no usable comparison point, such as a dispatch or a new branch, runs every engine.
- The MCP application's tests run on every change, because `ci.yml` has no path filter.
  Inside the workspace they import the checked-out engines, so an engine change is proved
  against the application before the engine is published.
- `.github/ci/<engine>/checks.sh` holds an engine's own source guards, and
  `.github/ci/<engine>/smoke.sh` its checks against the built wheel. Both are optional.
- `boundaries.yml` and `codeql.yml` run on every change.
- Workflow files inside component directories are inert historical records of the source
  repositories; only root workflows run.

## Rules

- Keep a change inside one component unless it is a root policy or workflow change.
- Do not move, rename or refactor a component in the same change that alters its
  behaviour.
- The root `pyproject.toml`, `uv.lock` and `justfile` are a development entrypoint
  only. The root is a virtual uv workspace: no root distribution, no root version, no
  root runtime dependency, nothing published from the root. Never add a root package,
  a shared runtime library, a unified version or a code generator.
- The workspace redirects the MCP application's three engine dependencies to the
  checked-out sources for development. It does not change the dependency direction.
  The exact pins in `apps/aus-accounting-mcp/pyproject.toml` stay authoritative, uv
  sources are development metadata and are never written into a built distribution,
  and engines still depend on nothing in this repository.
- Engines must not import the MCP application or each other, and production code must
  not use relative imports that leave the component directory. One shared `.venv` makes
  every component importable from every other; `tests/test_boundaries.py` is what keeps
  that from becoming a dependency, and it runs on every change.
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
