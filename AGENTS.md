# Agent instructions

This repository holds several independently released components. Read a component's
own `AGENTS.md`, `CONTRIBUTING.md` and `SECURITY.md` before changing it, and run its
checks from its own directory.

- Components: `apps/aus-accounting-mcp/` (the MCP application) and
  `packages/<distribution>/` (one directory per engine).
- Dependency direction: the MCP application depends on engines only through their
  published distributions. Engines never import `aus_accounting_mcp` or another engine,
  and production code never uses a relative import that leaves its component directory.
- Every component keeps its own `pyproject.toml`, lockfile, version, release notes,
  tests, commands and licence. Do not add a root package, root lockfile, shared runtime
  library, unified version, code generator or monorepo framework.
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
