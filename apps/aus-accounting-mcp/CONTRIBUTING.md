# Contributing

## Adapter boundary

Keep MCP registration and policy refusals in `aus_accounting_mcp/server.py`. The
`aus_accounting_mcp/adapters/` modules validate and translate inputs for the delegated
engines and serialise their results; they do not copy statutory calculations, datasets,
rates or dates into this facade. Shared monetary validation belongs in
`aus_accounting_mcp/money.py`. `aus_accounting_mcp/identifiers.py` holds the ABN and ACN
check digit arithmetic, because no published engine holds it and it carries no rate, date or
dataset. Tests and examples must use fabricated, synthetic data.

## Compatibility metadata

Engine dependency pins live in `pyproject.toml` and `uv.lock`. `compatibility.json`
records the last verified published server version and the matching engine releases;
`server.json` records the published MCP Registry package. An unreleased source version
may therefore be ahead of both files. Change compatibility metadata only as part of a
release handoff, and keep repository, PyPI, release, Registry and engine versions
consistent with the artefacts that actually exist.

## Demo evidence

The demo must call real registered MCP tools with fabricated inputs and show both the
synthetic BAS result and Division 7A loan-review outcome. `docs/quick-proof.txt` is the accessible
source of truth; `docs/quick-proof.webp` is derived media. Regenerate and review both when
demo output changes, then run the supplementary demo and media checks in `AGENTS.md`.

## Release handoff

Do not publish from a contribution branch. In the release commit, set every README install
pin, and the release link beside the first one, to the new version: the README is also the
PyPI description. The Registry preflight below refuses a pin mismatch, but only after PyPI
publication. After the CI gates and relevant supplementary checks pass, hand the reviewed
commit to the existing release workflow. A version tag
creates the attested GitHub release, and the same workflow's **Publish to PyPI** job then
publishes that exact distribution through the `pypi-aus-accounting-mcp` environment. Verify
the published package, then dispatch **Publish to MCP Registry** from `main` for the exact
version in `server.json`. That workflow refuses any other ref, and its unprivileged
`preflight` job runs `scripts/registry_preflight.py` first: `server.json` must name this
package at the version in `pyproject.toml`, PyPI must hold that exact version as one wheel
and one source distribution whose README carries the `mcp-name` marker, and PyPI's
provenance for both files must name `release-aus-accounting-mcp.yml` and the
`pypi-aus-accounting-mcp` environment. Only then does the `publish` job, through the
`mcp-registry` environment, publish the byte-identical `server.json` the preflight
verified. The environment's required reviewer and `main`-only deployment branch rule are
repository settings; the workflow cannot prove they are configured, so check them before
relying on the approval boundary. Run the preflight locally with
`uv run --locked python scripts/registry_preflight.py`; it makes public reads only.

After publication, run `uv run --locked python scripts/registry_preflight.py --readback`.
This also requires one exact-name active latest Registry entry matching the package,
version, repository and stdio transport, plus the published stable namespaced GitHub
release. It reports mismatches without writing to the Registry or starting a workflow.
Run it against the published version's source, rather than an unreleased candidate.
