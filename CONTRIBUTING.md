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

`just test` runs `just setup` first, which repeats `uv sync`.

`uv sync` creates one `.venv` at the root and installs the Python components into it
as editable workspace members. Australian Tax Calculators, Division 7A Loan
Review, ATO Benchmark Compare, Payday Super Checker and GST Review are Rust crates;
until the MCP server links them, the workspace installs their last Python
releases, 0.2.0, 0.3.2, 0.2.2, 0.2.3 and 0.1.0, from PyPI.

`just` is optional tooling; install it with `uv tool install rust-just`. The recipes
are `setup`, `lint`, `typecheck`, `test` and `check` (the last 3 together). Each
loops over the per-component commands in the table below, which remain the authority;
`just` runs them from one place, it does not replace them.

`just check` is the fast local pass, not a CI equivalent. It runs ruff, pyright and
pytest. CI additionally runs, per component, the dependency audit, the distribution
build, the installed-wheel and sdist smoke tests and changed-line coverage listed in
that table and in `ci-package.yml`. A green `just check` is not a green CI.

The same locked environment also supports these commands from the repository root,
after `just setup`:

```bash
uv run --locked --group dev ruff check .
uv run --locked --group dev pyright
uv run --locked --group dev pytest
```

Root pyright checks all 9 runtime packages. Root pytest runs the repository checks
and each component's existing suite in a separate process, from that component's
directory. This keeps repeated test module names and relative fixture paths
independent. Component configuration and CI gates remain authoritative.

Two consequences of the workspace are worth knowing before you run a component's
own commands:

- `uv run --locked` from a component directory now validates the root `uv.lock`,
  not the component's. Regenerate it with `uv lock` at the root after changing any
  component's dependencies, and commit it.
- The 2 Python engines pin the shared toolchain (`ruff`, `pyright`, `pytest`, `pytest-cov`,
  `coverage`, `build`) to one exact version each, so the workspace resolves them without
  an override. One
  workspace cannot hold 2 exact pins of the same tool, so keep the 2 identical when
  changing one.

## Rust engines

The 7 Rust engines under `packages/`, the `au-accounting` command in
`apps/au-accounting/`, which links them, and the LodgeiT calculator adapter in
`apps/lodgeit-calculator-adapter/` are standalone Cargo crates excluded from the
uv workspace. From a crate's directory, run `cargo fmt --check`,
`cargo clippy --locked --all-targets -- -D warnings` and `cargo test --locked`.
`cargo build --locked --release` builds the native command. Root `just test`
and `just lint` include these checks; typecheck remains Python-only. CI calls
`ci-rust.yml` once per crate on Ubuntu, Windows and macOS, from the anchor
workflow (the adapter from its own workflow), and keeps each platform's binary for 7 days as the archive the crate's
`[package.metadata.binstall]` names (the adapter has none: it is not released). The `umbrella-parity` job then runs
`apps/au-accounting/tests/parity.py` over the engines' and `au-accounting`'s
binaries; the adapter is not part of it. CodeQL and Dependabot
include Cargo. All candidate builds have read-only permissions.

The former PyPI publishers of these engines are retired. Native publication is not
configured: no workflow here publishes the binaries. Publishing them means adopting
Release Policy's `release-binary.yml`, which builds Windows and Linux executables, in
a separate authorised release change. Other components retain their Python publishers.

## Command routing

Run the checks from the component directory. Inside the workspace, `uv run --locked`
validates the root `uv.lock`; the component's own lockfile is what builds and releases it
on its own.

Every Python engine under `packages/` runs the same gates with the same commands. `ruff`, `pyright`
and `pytest --cov` read their scope from the engine's own `pyproject.toml` (`[tool.ruff]`,
`[tool.pyright]` and `[tool.coverage.run]`), so nothing in the commands names the engine:

```bash
uv run --locked --extra dev ruff check .
uv run --locked --extra dev pyright
uv run --locked --extra dev pytest --cov --cov-branch --cov-report=term-missing --cov-report=xml
uv run --locked --extra dev --with "pip-audit==2.10.1" pip-audit --local --strict
uv run --locked --extra dev --python 3.14 python -m build
```

CI also checks that the engine's own `uv.lock` still resolves on its own. Inside the
workspace `uv lock --check` validates the root lock, so run it from a copy of the engine
outside the checkout, the same way the lockfile is regenerated below.

| Component | Directory | Import package |
|---|---|---|
| the-exchequer-tally (Rust) | `packages/the-exchequer-tally/` | none |
| solomons-sword (Rust) | `packages/solomons-sword/` | none |
| the-wip-tally (Rust) | `packages/the-wip-tally/` | none |
| australian-tax-calculators (Rust) | `packages/australian-tax-calculators/` | none |
| div7a-loan-review (Rust) | `packages/div7a-loan-review/` | none |
| ato-benchmark-compare (Rust) | `packages/ato-benchmark-compare/` | none |
| payday-super-checker (Rust) | `packages/payday-super-checker/` | none |
| gst-review (Rust) | `packages/gst-review/` | none |
| lodgment-calendar | `packages/lodgment-calendar/` | `lodgmentcalendar` |

The 2 applications, the rates dataset and the repository boundary checks use their own
commands:

| Component | Directory | Checks |
|---|---|---|
| Aus Accounting MCP | `apps/aus-accounting-mcp/` | `uv run --locked --extra dev pytest -q --cov=aus_accounting_mcp --cov-branch --cov-report=term-missing`; `uv run --locked --extra dev --with "pip-audit==2.10.1" pip-audit --local --strict`; `uv run --locked --extra dev ruff check aus_accounting_mcp tests`; `uv run --locked --extra dev pyright aus_accounting_mcp` |
| LodgeiT calculator adapter (Rust) | `apps/lodgeit-calculator-adapter/` | `cargo fmt --check`; `cargo clippy --locked --all-targets -- -D warnings`; `cargo test --locked`. Its gates run from `.github/workflows/ci-lodgeit-adapter.yml`, not `ci.yml`. |
| AU tax rates data (Rust) | `packages/au-tax-rates-data/` | `cargo fmt --check`; `cargo clippy --locked --all-targets -- -D warnings`; `cargo test --locked`; `cargo run --locked --release -- validate`. These run in the `au-tax-rates-data` job of `.github/workflows/ci.yml` on Ubuntu, Windows and macOS. The dataset is never released. |
| Repository boundaries | `.` | `python -m unittest -v tests/test_boundaries.py tests/test_shared_blocks.py` |

The shared toolchain is pinned to one version per tool in every engine's `pyproject.toml`,
so a gate behaves the same wherever it runs, and the workspace resolution takes the same
versions for `just`. Changing a pin means changing both, then relocking twice: `uv lock`
at the root, which is what CI validates, and each engine's own lockfile, which is what
builds and releases it alone. Inside the workspace `uv lock` always writes the root lock,
so an engine's own lockfile is regenerated from a copy of the engine outside it:

```bash
tmp=$(mktemp -d) && cp -r packages/<engine>/. "$tmp" && (cd "$tmp" && uv lock) \
  && cp "$tmp/uv.lock" packages/<engine>/uv.lock
```

## CI routing

Dependabot runs one Python update job at the workspace root. All 10 components
share that resolution, so separate component jobs can propose conflicting exact
toolchain pins or fail to update the root lockfile. Keep Python updates grouped
at `/`.
Dependabot updates the workspace lock; before merging a dependency PR, regenerate
each affected component's standalone release lock using the procedure above.
The existing component lock checks remain required.

`ci.yml` is the anchor workflow. It carries no path filter, so its required checks always
report, and it runs 2 things: the MCP application's own gates, and one call of the
reusable `ci-package.yml` for each engine, from a package-name matrix.
The `root-checks` job in `boundaries.yml` runs the exact root lock, ruff, pyright and
pytest commands on Ubuntu with Python 3.15. The adapter's jobs run in their own
workflow with a `lodgeit-gates` result check, keeping the MCP application's existing
release-check names unchanged. Branch protection requires one aggregate per
workflow, `tests-gates`, `boundaries-gates`, `lodgeit-gates`, `public-fixtures-gates`
and `codeql-gates`, plus the `CodeQL` results check and `Attribution policy`. Each
aggregate also fails when a job in its workflow is missing from its `needs`, or when
a pull request removes a job without adding a `# removed-jobs: <job>` comment to that
workflow (`.github/ci/check_gates.py`). Release callers select these aggregates as
well as the named component jobs.

For engines with held files, the Python 3.14 test job filters its existing coverage
data to those files and applies the 100% changed-line branch-coverage gate. The
release callers require that test job; no second pytest run is needed.

- `ci-package.yml` gives every engine the same gates from the engine's own directory,
  the same definition `ryanduguid/accounting-review-pipeline` uses for its components: a
  check that the engine's own `uv.lock` resolves on its own, `ruff`, `pyright`, `pytest` with
  branch coverage on Python 3.14 and 3.15 (the floor and ceiling of the declared
  `requires-python`), a `pip-audit` dependency audit, and a build of the distribution
  followed by an install of the one built wheel into a clean environment, an import of
  the engine from it, and the engine's own wheel smoke script.
- The path filter is applied inside each call, by `.github/ci/select_package.py`, rather
  than on the trigger. A change under `packages/<engine>/` runs that engine and skips the
  rest; a change to a root policy file (`AGENTS.md`, `CONTRIBUTING.md`, `README.md`,
  `SECURITY.md`, `IMPORTS.md`, `.editorconfig`, `.gitignore`, `.mailmap`, `.gitattributes`),
  to a root workspace file (`pyproject.toml`, `uv.lock`, `justfile`) or to anything under
  `.github/` runs every engine. The root workspace files are in that list because the root
  `uv.lock` is what `uv run --locked` validates from inside every component directory. A
  run with no usable comparison point, such as a dispatch or a new branch, runs every engine.
  Git paths use NUL delimiters so Unicode, tabs and newlines remain intact; malformed
  input also runs every engine.
  On a push to `main`, an engine whose pyproject version has no `<engine>/v<version>` tag
  yet also runs, whatever changed, so its release can be tagged on `main`'s head.
- Each engine's `<engine> / gates` job reports on every run: it passes when each gate
  passed or was skipped by the path filter and fails when any gate failed or was
  cancelled. `tests-gates` needs every engine's call, so branch protection requires it
  rather than the per-engine contexts. The individual jobs are not required, because a
  skipped matrix job never reports under its matrix names and would block every pull
  request the path filter skips.
- Application jobs use `.github/ci/app_ci.py` to select PR changes. An application's own
  files, its engine dependencies, root tests, shared policy and workspace files, and any
  unknown path or comparison run its complete checks. Only known unrelated component
  paths can skip them. The MCP application includes its 4 engine dependencies; the
  adapter includes the 2 engines its trials link. Root pytest checks these
  dependency lists against each application's manifest on every change. The aggregate requires a
  successful selection and exactly the corresponding successful or skipped app jobs.
  Every main push and manual run executes all app checks, preserving release evidence.
- `.github/ci/<engine>/checks.sh` holds an engine's own source guards, and
  `.github/ci/<engine>/smoke.sh` its checks against the built wheel. Both are optional.
- `boundaries.yml` and `codeql.yml` run on every change.
- Only root workflows run; component directories carry no workflow files.

## Rules

`joined-fixtures.yml` checks the close-to-forecast, quarter and WIP-to-cash examples
on pull requests, pushes to `main` and manual runs, on Linux and Windows.
It uses this repository's proposed
checkout with full commit pins for Accounting Review Pipeline and au-fpa-pack.
Update companion pins deliberately when adopting compatible changes.

The existing [joined example driver](https://github.com/ryanduguid/accounting-review-pipeline/blob/3dcb5304f4c67c01de98b34638999b5f274d1810/packages/monthly-close-control-plane/docs/utility-workflows.md)
checks both fabricated thirteen-week cash scenarios and retains REVIEW findings.
Processes and fresh environments stay separate; workspace locks remain authoritative.
Separate operating-system artefacts retain fabricated results, failure diagnostics,
the escaped summary, success manifest and replay record for seven days. No private
checkout or secret is needed. Linux keeps the `public-fixtures` check name; configure
the added `public-fixtures (Windows)` check separately if it should block merges.

- Keep a change inside one component unless it is a root policy or workflow change.
- Do not move, rename or refactor a component in the same change that alters its
  behaviour.
- The root `pyproject.toml`, `uv.lock` and `justfile` are a development entrypoint
  only. The root is a virtual uv workspace: no root distribution, no root version, no
  root runtime dependency, nothing published from the root. Never add a root package,
  a shared runtime library, a unified version or a code generator.
- The workspace redirects the MCP application's 4 engine dependencies to the
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
builds, attests and publishes the GitHub release. Every Python component workflow then publishes the
exact attested distribution to PyPI under the component's own `pypi-<component>`
environment and trusted publisher. The MCP application additionally publishes to the MCP
Registry through `publish-mcp.yml`.

| Component | Tag | Workflow | Version source | PyPI environment |
|---|---|---|---|---|
| aus-accounting-mcp | `aus-accounting-mcp/vX.Y.Z` | `release-aus-accounting-mcp.yml` | `pyproject.toml` | `pypi-aus-accounting-mcp` |
| lodgment-calendar | `lodgment-calendar/vX.Y.Z` | `release-lodgment-calendar.yml` | `pyproject.toml` | `pypi-lodgment-calendar` |
| the-exchequer-tally (Rust) | reserved `the-exchequer-tally/vX.Y.Z` | native publisher pending | `Cargo.toml` | none |
| solomons-sword (Rust) | reserved `solomons-sword/vX.Y.Z` | native publisher pending | `Cargo.toml` | none |
| the-wip-tally (Rust) | reserved `the-wip-tally/vX.Y.Z` | native publisher pending | `Cargo.toml` | none |
| australian-tax-calculators (Rust) | reserved `australian-tax-calculators/vX.Y.Z` | native publisher pending | `Cargo.toml` | none |
| div7a-loan-review (Rust) | reserved `div7a-loan-review/vX.Y.Z` | native publisher pending | `Cargo.toml` | none |
| ato-benchmark-compare (Rust) | reserved `ato-benchmark-compare/vX.Y.Z` | native publisher pending | `Cargo.toml` | none |
| payday-super-checker (Rust) | reserved `payday-super-checker/vX.Y.Z` | native publisher pending | `Cargo.toml` | none |
| gst-review (Rust) | reserved `gst-review/vX.Y.Z` | native publisher pending | `Cargo.toml` | none |

Tag and push with `just release <component>...` from the repository root, with `gh`
authenticated with Administration read access. Before tagging anything it refuses unless
release immutability is enabled and every named component has a release workflow, a
`RELEASE_NOTES.md` header for its pyproject version, no tag locally or on GitHub, no
GitHub or PyPI release of that version, and a successful job for each of its
`required-checks` in the push runs of `main`'s head. It then tags `main`'s head and releases the
components in the order given, pushing each tag on its own: GitHub creates no push event,
so starts no workflow, for any tag when more than 3 tags arrive in one push ([events that
trigger
workflows](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#push)).
The next tag waits until the previous release run succeeds, and the first failure stops
the run and lists the components left untagged. Release an engine before an MCP release
that pins it. `--dry-run` runs the checks and tags nothing.

Do not merge to `main` until `just release` finishes. Every tag is made on the same `main`
commit, and Release Policy's first job requires that commit to be `main`'s head, so a merge
before the last tag's run is under way stops the remaining releases. Once a run is past that
job, a merge stops it only if it changes that component's source directory or its release
workflow. A stopped version cannot be reused, because a pushed tag cannot be deleted.

`IMPORTS.md` records which components still lack a Release Policy prerequisite; their
workflows fail closed until a reviewed component change adds it. Nothing publishes from a
contribution branch, and no tag or release is created without explicit approval.

## Release checks

The release caller names the component checks that must have succeeded for the
exact release commit on `main`. It advances the policy SHA, `required-checks`
and `actions: read` together. Skipped, missing, cancelled or failed checks block
publication, including component tests skipped by a path filter. An aggregate
gates job cannot replace those checks. Before tagging, choose a main-branch
commit with successful component CI; a successful run for an older commit is
not evidence for the release. Once the version bump is on `main`, every push to
`main` runs that component until its tag exists, so `main`'s head carries the
checks as soon as its push CI passes. Tags and publication still require explicit approval.
