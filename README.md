# australian-accounting

Development home for the Aus Accounting MCP application and six independently released
Australian accounting engines. Each component keeps its own distribution name, version,
lockfile, tests, release notes, commands and licence. There is no root package, root
lockfile, shared runtime library or combined version.

## Use Aus Accounting MCP

For the MCP server, install Python 3.10+ and [uv](https://docs.astral.sh/uv/), then
configure your MCP client to run:

```bash
uvx aus-accounting-mcp
```

This starts a **local stdio server**; it waits for an MCP client rather than opening
a web page. It requires no API key. Package installation downloads dependencies;
the tools then use bundled data without network requests or record writes.

[Client setup and examples](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/README.md#client-integration)
· [Tool reference](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/README.md#tools)
· [PyPI](https://pypi.org/project/aus-accounting-mcp/)
· [Glama listing](https://glama.ai/mcp/servers/ryanduguid/australian-accounting)

The seven tools list ATO benchmark industries, compare supplied expense buckets,
review Payday Super timing, look up reviewed Division 7A rates, review an
operator-supplied Division 7A loan, refuse unsupported Division 7A matters, and
generate synthetic CTR/BAS fixtures. Division 7A review is limited to the delegated
engine's s 109N/s 109E scope; it does not form amalgamated loans or classify
repayments under s 109R. Outputs are review aids, not advice or lodgments.

For an editable source installation, run `pip install -e .` from
`apps/aus-accounting-mcp/`. The repository root is not an installable Python package.

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
