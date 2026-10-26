# Agent instructions

This repository holds several independently released components. Read a component's
own `AGENTS.md`, `CONTRIBUTING.md` and `SECURITY.md` before changing it, and run its
checks from its own directory.

- Setup: `uv sync` at the root installs every component and the shared toolchain into
  one `.venv`; `just test` runs every component's suite plus the boundary checks. That
  is the whole setup for a fresh clone. `just` comes from `uv tool install rust-just`.
- Components: `apps/aus-accounting-mcp/` (the MCP application),
  `apps/lodgeit-calculator-adapter/` (a Rust command, an optional adapter for a
  third-party calculator service, off by default; nothing else here depends on it,
  it links 2 engine crates by path, and it is excluded from the uv workspace), `apps/au-accounting/`
  (a Rust command with a subcommand per Rust engine; it links the engine crates by
  path, uses only their public `api` and `cli` modules, and is excluded from the uv
  workspace),
  `packages/<distribution>/` (one directory per engine) and `packages/au-tax-rates-data/`
  (sourced ATO rates and thresholds with their Rust validator, `rates`; not an engine,
  linked by nothing, excluded from the uv workspace and never released).
- Dependency direction: the MCP application depends on engines only through their
  published distributions. The root workspace redirects 2 of those dependencies to the
  checked-out sources so the application is developed and tested against the tree, but
  the exact pins in its `pyproject.toml` stay authoritative and uv sources are never
  written into a built distribution. Engines never import `aus_accounting_mcp` or
  another engine, and production code never uses a relative import that leaves its
  component directory. One shared `.venv` makes every component importable from every
  other; `tests/test_boundaries.py` is what keeps that from becoming a dependency.
- The Exchequer Tally, Solomon's Sword, The WIP Tally, Australian Tax Calculators,
  Division 7A Loan Review and ATO Benchmark Compare are standalone Rust: run their
  Cargo checks from `packages/the-exchequer-tally/`, `packages/solomons-sword/`,
  `packages/the-wip-tally/`, `packages/australian-tax-calculators/`,
  `packages/div7a-loan-review/` and `packages/ato-benchmark-compare/`. Each README documents JSON replacements for the
  removed Python API. They are excluded from the uv workspace and Python gates.
- Every Python component keeps its own `pyproject.toml`, lockfile, version, release notes,
  tests, commands and licence, and is released on its own from its own directory. A
  component's lockfile stays the authority for building and releasing that component
  alone. The rates dataset is the exception: it has no release workflow or release notes.
- The root `pyproject.toml`, `uv.lock` and `justfile` are a development entrypoint
  only. The root is a virtual uv workspace: it declares no package, no version and no
  runtime dependency, and publishes nothing. Do not add a root distribution, shared
  runtime library, unified version, code generator or monorepo framework.
- `uv run --locked` inside a component directory validates the root `uv.lock`, not the
  component's. After changing any component's dependencies, run `uv lock` at the root
  and commit the result.
- Only the workflows under the root `.github/workflows/` are active. Read the
  root [CONTRIBUTING.md](CONTRIBUTING.md) for current command and CI routing. Run
  component commands from their component directory; resolve imported workflow
  references against the root workflows. Historical import details live in `IMPORTS.md`.
- Fixtures and demonstrations are fabricated. No client data, credentials, workpapers
  or generated client reports enter this repository.
- Movement, import and behaviour changes are separate changes. Do not refactor a
  component while moving or importing it.
- A release covers one component, on the namespaced tag `<component>/vX.Y.Z`, through
  that component's root release workflow. Never tag, release or publish without
  explicit approval.
