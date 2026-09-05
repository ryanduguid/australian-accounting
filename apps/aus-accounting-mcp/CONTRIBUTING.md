# Contributing

## Adapter boundary

Keep MCP registration and policy refusals in `aus_accounting_mcp/server.py`. The
`aus_accounting_mcp/adapters/` modules validate and translate inputs for the delegated
engines and serialise their results; they do not copy statutory calculations, datasets,
rates or dates into this facade. Shared monetary validation belongs in
`aus_accounting_mcp/money.py`. Tests and examples must use fabricated, synthetic data.

## Compatibility metadata

Engine dependency pins live in `pyproject.toml` and `uv.lock`. `compatibility.json`
records the last verified published server version and the matching engine releases;
`server.json` records the published MCP Registry package. An unreleased source version
may therefore be ahead of both files. Change compatibility metadata only as part of a
release handoff, and keep repository, PyPI, release, Registry and engine versions
consistent with the artifacts that actually exist.

## Demo evidence

The demo must call real registered MCP tools with fabricated inputs and show both the
synthetic BAS result and Division 7A loan-review outcome. `docs/quick-proof.txt` is the accessible
source of truth; `docs/quick-proof.webp` is derived media. Regenerate and review both when
demo output changes, then run the supplementary demo and media checks in `AGENTS.md`.

## Release handoff

Do not publish from a contribution branch. After the CI gates and relevant supplementary
checks pass, hand the reviewed commit to the existing release workflow. A version tag
creates the attested GitHub release, and the same workflow's **Publish to PyPI** job then
publishes that exact distribution through the `pypi-aus-accounting-mcp` environment. Verify
the published package, then dispatch **Publish to MCP Registry** for the exact version in
`server.json`. The registry publication remains an explicit, approval-gated action.
