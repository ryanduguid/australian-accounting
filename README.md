# australian-accounting

Development home for the Aus Accounting MCP application and six independently released
Australian accounting engines. Each component keeps its own distribution name, version,
lockfile, tests, release notes, commands and licence. There is no root package, root
lockfile, shared runtime library or combined version.

## Components

| Path | Distribution | Import package | Commands |
|---|---|---|---|
| `apps/aus-accounting-mcp/` | `aus-accounting-mcp` | `aus_accounting_mcp` | `aus-accounting-mcp`, `aus-accounting-mcp-demo` |
| `packages/ato-benchmark-compare/` | `ato-benchmark-compare` | `atobenchmark` | `ato-benchmark-compare` |
| `packages/payday-super-checker/` | `payday-super-checker` | `paydaysuper` | `payday-super-check` |
| `packages/div7a-loan-review/` | `div7a-loan-review` | `div7aloan` | `div7a-loan-review` |
| `packages/the-exchequer-tally/` | `the-exchequer-tally` | `edwinnixon` | `the-exchequer-tally` |
| `packages/solomons-sword/` | `solomons-sword` | `louisgoldberg` | `solomons-sword` |
| `packages/the-wip-tally/` | `the-wip-tally` | `wiptally` | `wip-tally` |

`IMPORTS.md` records the source repository, commit and tree of every imported engine. The
MCP application is the `io.github.ryanduguid/aus-accounting` MCP Registry server; it
depends on the published `ato-benchmark-compare`, `payday-super-checker` and
`div7a-loan-review` distributions, not on the sibling directories.

## Working in a component

Change into the component directory and use the commands its own `README.md`,
`AGENTS.md` or `CONTRIBUTING.md` documents. The root `CONTRIBUTING.md` routes the
common commands and names the root workflow that runs them.

## Boundaries

- The MCP application depends on engines only through their published distributions.
- Engines never import `aus_accounting_mcp` or another engine, and no production module
  uses a relative import that leaves its package; `tests/test_boundaries.py` proves both.
- Only the workflows under the root `.github/workflows/` are active. Workflow files inside
  component directories are historical records of the source repositories.
- No client data, credentials or generated client reports enter this repository.

## Releases

Each component is released on its own namespaced tag `<component>/vX.Y.Z` by its own root
workflow `.github/workflows/release-<component>.yml`, which calls the pinned Release Policy
reusable workflow for that component directory only. `CONTRIBUTING.md` has the table.

Each component's `LICENSE` applies to that component, and
`packages/ato-benchmark-compare/NOTICE` covers its bundled ATO data. Outputs are review
aids, not advice.
