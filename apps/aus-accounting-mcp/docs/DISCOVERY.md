# Discovery metadata

Use this file as the source of truth for public discovery copy across GitHub
About, repository topics, README, `pyproject.toml`, `glama.json`, and
`CITATION.cff`.

## GitHub About

Description:

```text
Aus Accounting MCP, a local MCP server for Australian accounting review: ATO benchmarks, Payday Super 2026, limited Division 7A loan reviews and synthetic SBR fixtures. Not advice.
```

Website:

```text
https://duguid.com.au/tools/australian-tax-ai-agents/
```

Topics:

```text
accounting
accounting-ai
agent-skills
ato
ato-benchmarks
australian-accounting
australian-tax
australian-taxation
claude-code
codex
cursor
division-7a
mcp
mcp-server
model-context-protocol
payday-super
python
tax-prep
```

Division 7A discovery copy must describe the reviewed s 109N/s 109E scope and
must not imply that the server handles every Division 7A question.

Apply with `scripts/publish-github-about.sh` from a session authenticated to
GitHub. The Actions `GITHUB_TOKEN` cannot PATCH homepage (needs repository
admin), so the `github-about` workflow warns and continues rather than failing.

## Keyword map

- Primary: Australian computational accounting MCP, Payday Super review, ATO
  small-business benchmarks, limited Division 7A loan review.
- Agent/platform terms: MCP, Cursor, Claude Code, Claude Desktop, Codex, uvx.

## Copy rules

- Lead with the one-command install (`uvx aus-accounting-mcp`).
- Name tools as jobs, not as the historical engine repositories.
- Keep unsupported Division 7A refusals visible and label SBR fixtures synthetic.
- Do not imply ATO, CA ANZ, or vendor endorsement.
- Do not claim a hosted corpus or a Smithery listing until it exists. The
  `aus-accounting-mcp` project is published on PyPI; its current public release
  is the source of truth for the standard install command.
- Point agents at the comparison page: https://duguid.com.au/tools/australian-tax-ai-agents/
- The current Glama listing is
  https://glama.ai/mcp/servers/ryanduguid/australian-accounting,
  claimed as maintainer `ryanduguid`. The historical `au-tax-mcp-server` URL is
  retained by the existing README badge. Keep the root `glama.json` maintainer
  declaration aligned with the application copy: Glama discovers repository-root
  metadata, while the application lives in `apps/aus-accounting-mcp/`.
  In Glama **Admin → Repository**, set **GitHub Project URL** to
  https://github.com/ryanduguid/australian-accounting/tree/main/apps/aus-accounting-mcp
  so the listing renders the application's README instead of the monorepo index.
  Use **Sync Server** after a source change lands. Profile name and description
  are separate fields under **Admin → Profile**. Review generated FAQs and TDQS
  explanations after syncing: cached text may still describe the old five-tool
  server even when the schema lists seven tools. Do not claim a new grade until
  Glama actually recomputes it.
- Official MCP registry: listed as `io.github.ryanduguid/aus-accounting` since
  25 August 2026. Record:
  https://registry.modelcontextprotocol.io/v0.1/servers/io.github.ryanduguid%2Faus-accounting/versions/latest
  Keep `server.json` in the application directory `apps/aus-accounting-mcp/`, where
  `publish-mcp.yml` runs. The [MCP Registry PyPI package
  guidance](https://github.com/modelcontextprotocol/registry/blob/main/docs/modelcontextprotocol-io/package-types.mdx)
  requires the published package README to contain its matching `mcp-name`
  marker. Version 0.1.7 is the release represented by `server.json`; publish it
  to PyPI and verify it before dispatching the registry workflow. `server.json`
  names that exact package version. GitHub Actions OIDC publishes the registry
  record only after an explicit manual dispatch.
- For future releases, push the namespaced tag `aus-accounting-mcp/vX.Y.Z` to create
  and attest the GitHub release, then dispatch **Publish to PyPI** with the same tag. PyPI's trusted
  publisher is bound to `publish-pypi.yml`; do not add a publisher job to
  `release-aus-accounting-mcp.yml` unless the PyPI publisher configuration is changed
  with it.
- Do not copy AGPL or proprietary-corpus language from other ATO MCP products.

## Release provenance

Release `v0.1.5` predates the repository rename. Its certificate therefore
retains the historical `au-tax-mcp-server` source identity. Verify it with an
owner-scoped lookup that is then bound to the exact source and signer:

```bash
tag=v0.1.5
repo=ryanduguid/aus-accounting-mcp
wheel="aus_accounting_mcp-${tag#v}-py3-none-any.whl"
release_commit="$(git ls-remote "https://github.com/$repo.git" "refs/tags/$tag^{}" | cut -f1)"
test -n "$release_commit"
gh release download "$tag" -R "$repo" --dir "release-$tag"
cd "release-$tag"
sha256sum --check SHA256SUMS
gh attestation verify "$wheel" --owner ryanduguid \
  --source-digest "$release_commit" \
  --source-ref "refs/tags/$tag" \
  --signer-workflow ryanduguid/release-policy/.github/workflows/release-python.yml \
  --signer-digest 3b8a377207cab2c7c808fcc96b66578f4695beea
gh attestation verify "$wheel" --owner ryanduguid \
  --predicate-type https://spdx.dev/Document/v2.3 \
  --source-digest "$release_commit" \
  --source-ref "refs/tags/$tag" \
  --signer-workflow ryanduguid/release-policy/.github/workflows/release-python.yml \
  --signer-digest 3b8a377207cab2c7c808fcc96b66578f4695beea
```

Releases cut after the rename and shared Python-policy migration use the
current repository identity and hardened policy digest. For the next release,
update `tag` if the intended version changes and run these checks after
downloading the assets and checking `SHA256SUMS`:

```bash
tag=aus-accounting-mcp/v0.1.7
repo=ryanduguid/australian-accounting
wheel="aus_accounting_mcp-${tag#aus-accounting-mcp/v}-py3-none-any.whl"
release_commit="$(git ls-remote "https://github.com/$repo.git" "refs/tags/$tag^{}" | cut -f1)"
test -n "$release_commit"
gh attestation verify "$wheel" -R "$repo" \
  --source-digest "$release_commit" \
  --source-ref "refs/tags/$tag" \
  --signer-workflow ryanduguid/release-policy/.github/workflows/release-python.yml \
  --signer-digest 3ff09b654a17b9a3b55548e25e6108ee582b00c4
gh attestation verify "$wheel" -R "$repo" \
  --predicate-type https://spdx.dev/Document \
  --source-digest "$release_commit" \
  --source-ref "refs/tags/$tag" \
  --signer-workflow ryanduguid/release-policy/.github/workflows/release-python.yml \
  --signer-digest 3ff09b654a17b9a3b55548e25e6108ee582b00c4
```
