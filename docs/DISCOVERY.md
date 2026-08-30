# Discovery metadata

Use this file as the source of truth for public discovery copy across GitHub
About, repository topics, README, `pyproject.toml`, `glama.json`, and
`CITATION.cff`.

## GitHub About

Description:

```text
Local MCP server for Australian accounting review: ATO small-business benchmarks, Payday Super 2026, refused Division 7A and synthetic SBR fixtures. Not advice.
```

Website:

```text
https://ryanduguid.github.io/tools/australian-tax-ai-agents/
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
mcp
mcp-server
model-context-protocol
payday-super
python
tax-prep
```

Do not add `division-7a`. The only Div 7A tool is `refuse_div7a`.

Apply with `scripts/publish-github-about.sh` from a session authenticated to
GitHub. The Actions `GITHUB_TOKEN` cannot PATCH homepage (needs repository
admin), so the `github-about` workflow warns and continues rather than failing.

## Keyword map

- Primary: Australian computational accounting MCP, Payday Super review, ATO
  small-business benchmarks.
- Agent/platform terms: MCP, Cursor, Claude Code, Claude Desktop, Codex, uvx.

## Copy rules

- Lead with the one-command install (`uvx aus-accounting-mcp`).
- Name tools as jobs, not as the historical engine repositories.
- Keep refusals visible (Division 7A unwired; SBR synthetic).
- Do not imply ATO, CA ANZ, or vendor endorsement.
- Do not claim a hosted corpus or a Smithery listing until it exists. The
  `aus-accounting-mcp` project is published on PyPI; its current public release
  is the source of truth for the standard install command.
- Point agents at the comparison page: https://ryanduguid.github.io/tools/australian-tax-ai-agents/
- The Glama listing is https://glama.ai/mcp/servers/ryanduguid/au-tax-mcp-server,
  claimed as maintainer `ryanduguid`. The generated badge is
  https://glama.ai/mcp/servers/ryanduguid/au-tax-mcp-server/badge.
- Official MCP registry: listed as `io.github.ryanduguid/aus-accounting` since
  25 August 2026. Record:
  https://registry.modelcontextprotocol.io/v0.1/servers/io.github.ryanduguid%2Faus-accounting/versions/latest
  Keep `server.json` in the repo root. The [MCP Registry PyPI package
  guidance](https://github.com/modelcontextprotocol/registry/blob/main/docs/modelcontextprotocol-io/package-types.mdx)
  requires the published package README to contain its matching `mcp-name`
  marker. Version 0.1.5 is published and verified on PyPI, and `server.json`
  names that exact package version. GitHub Actions OIDC publishes the registry
  record only after an explicit manual dispatch.
- For future releases, push the version tag to create and attest the GitHub
  release, then dispatch **Publish to PyPI** with the same tag. PyPI's trusted
  publisher is bound to `publish-pypi.yml`; do not add a publisher job to
  `release.yml` unless the PyPI publisher configuration is changed with it.
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
tag=v0.1.6
repo=ryanduguid/aus-accounting-mcp
wheel="aus_accounting_mcp-${tag#v}-py3-none-any.whl"
release_commit="$(git ls-remote "https://github.com/$repo.git" "refs/tags/$tag^{}" | cut -f1)"
test -n "$release_commit"
gh attestation verify "$wheel" -R "$repo" \
  --source-digest "$release_commit" \
  --source-ref "refs/tags/$tag" \
  --signer-workflow ryanduguid/release-policy/.github/workflows/release-python.yml \
  --signer-digest 8b4de1ed339f1358b5f3e850b63412d8717d01da
gh attestation verify "$wheel" -R "$repo" \
  --predicate-type https://spdx.dev/Document/v2.3 \
  --source-digest "$release_commit" \
  --source-ref "refs/tags/$tag" \
  --signer-workflow ryanduguid/release-policy/.github/workflows/release-python.yml \
  --signer-digest 8b4de1ed339f1358b5f3e850b63412d8717d01da
```
