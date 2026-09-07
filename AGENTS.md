# Agent instructions

This repository holds several independently released components. Read a component's
own `AGENTS.md`, `CONTRIBUTING.md` and `SECURITY.md` before changing it, and run its
checks from its own directory.

- Setup: `uv sync` at the root installs every component and the shared toolchain into
  one `.venv`; `just test` runs every component's suite plus the boundary checks. That
  is the whole setup for a fresh clone. `just` comes from `uv tool install rust-just`.
- Components: `apps/aus-accounting-mcp/` (the MCP application) and
  `packages/<distribution>/` (one directory per engine).
- Dependency direction: the MCP application depends on engines only through their
  published distributions. The root workspace redirects those three dependencies to the
  checked-out sources so the application is developed and tested against the tree, but
  the exact pins in its `pyproject.toml` stay authoritative and uv sources are never
  written into a built distribution. Engines never import `aus_accounting_mcp` or
  another engine, and production code never uses a relative import that leaves its
  component directory. One shared `.venv` makes every component importable from every
  other; `tests/test_boundaries.py` is what keeps that from becoming a dependency.
- Every component keeps its own `pyproject.toml`, lockfile, version, release notes,
  tests, commands and licence, and is released on its own from its own directory. A
  component's lockfile stays the authority for building and releasing that component
  alone.
- The root `pyproject.toml`, `uv.lock` and `justfile` are a development entrypoint
  only. The root is a virtual uv workspace: it declares no package, no version and no
  runtime dependency, and publishes nothing. Do not add a root distribution, shared
  runtime library, unified version, code generator or monorepo framework.
- `uv run --locked` inside a component directory validates the root `uv.lock`, not the
  component's. After changing any component's dependencies, run `uv lock` at the root
  and commit the result.
- Only the workflows under the root `.github/workflows/` are active. Workflow files,
  scripts and instructions inside an imported component directory are historical
  records of the source repository and are never run from here.
- Fixtures and demonstrations are fabricated. No client data, credentials, workpapers
  or generated client reports enter this repository.
- Movement, import and behaviour changes are separate changes. Do not refactor a
  component while moving or importing it.
- A release covers one component, on the namespaced tag `<component>/vX.Y.Z`, through
  that component's root release workflow. Never tag, release or publish without
  explicit approval.
