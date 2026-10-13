# Discovery metadata

Use this file as the source of truth for public discovery copy across GitHub
About, repository topics, README, `pyproject.toml`, `glama.json`, and
`CITATION.cff`.

## GitHub About

Description:

```text
Local MCP for ATO benchmarks, Payday Super timing, limited Division 7A loan reviews, ten bounded tax and super worksheets and cited local Markdown retrieval. Synthetic SBR fixtures. Not advice.
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
GitHub; the Actions `GITHUB_TOKEN` cannot PATCH homepage (needs repository
admin). That script carries the description and topics above verbatim, so
running it is what reconciles the live About with this file.

### Live About drift, read 7 September 2026

The live About has been edited away from this file and the script, in 2 ways.
Neither is fixed here: applying it is a repository-admin action.

- The description reads 'Division 7A loan review' where this file and the script
  read 'limited Division 7A loan reviews'. Dropping 'limited' is the one wording
  change the rule above forbids, because it implies the server handles every
  Division 7A question when it reviews s 109N terms and s 109E repayments only.
  The live text also leads with the engines rather than the server, which is a
  fair description of a monorepo and is not the problem.
- The `division-7a` topic is absent from the live repository. The other 17 topics
  above are all set.

Re-running the script restores both. It only adds topics, so it will not remove
anything a later edit added.

## Keyword map

- Primary: Australian computational accounting MCP, Payday Super review, ATO
  small-business benchmarks, limited Division 7A loan review, bounded tax worksheets
  and cited local Markdown retrieval.
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
  claimed as maintainer `ryanduguid`. The README no longer displays the badge
  for the historical `au-tax-mcp-server` URL. Verify the current badge endpoint
  before adding a replacement. The root `glama.json` carries the
  maintainer declaration: Glama discovers repository-root metadata, while the
  application lives in `apps/aus-accounting-mcp/`.
  In Glama **Admin → Repository**, set **GitHub Project URL** to
  https://github.com/ryanduguid/australian-accounting/tree/main/apps/aus-accounting-mcp
  so the listing renders the application's README instead of the monorepo index.
  Use **Sync Server** after a source change lands. Profile name and description
  are separate fields under **Admin → Profile**. Review generated FAQs and TDQS
  explanations after syncing: cached text may still describe an earlier tool set.
  Version 0.3.0 exposes 18 tools. Do not claim a new grade until
  Glama actually recomputes it.
- Official MCP registry: listed as `io.github.ryanduguid/aus-accounting` since
  25 August 2026. Record:
  https://registry.modelcontextprotocol.io/v0.1/servers/io.github.ryanduguid%2Faus-accounting/versions/latest
  Keep `server.json` in the application directory `apps/aus-accounting-mcp/`, where
  `publish-mcp.yml` runs. The [MCP Registry PyPI package
  guidance](https://github.com/modelcontextprotocol/registry/blob/main/docs/modelcontextprotocol-io/package-types.mdx)
  requires the published package README to contain its matching `mcp-name`
  marker. `server.json` names the exact package version the registry record
  represents; publish that version to PyPI and verify it there before dispatching
  the registry workflow. GitHub Actions OIDC publishes the registry
  record only after an explicit manual dispatch.
- For future releases, push the namespaced tag `aus-accounting-mcp/vX.Y.Z`.
  `release-aus-accounting-mcp.yml` creates and attests the GitHub release, then its
  `pypi` job publishes the exact attested distribution through the
  `pypi-aus-accounting-mcp` environment, the same shape as the engine workflows.
  PyPI's trusted publisher must name that workflow file and environment; change the
  publisher configuration together with the workflow, never one without the other.
- Do not copy AGPL or proprietary-corpus language from other ATO MCP products.

## Glama profile

Copy prepared for the next authorised profile update:

```text
Local MCP for Australian accountants: ATO benchmarks, Payday Super timing, limited Division 7A reviews, 10 bounded tax and super worksheets and cited legislation search. Synthetic CTR/BAS test data. Not advice.
```

The package contains no reference library. Retrieval requires an authorised local
Markdown folder. The worksheets retain their documented periods and exclusions;
the tool list does not establish support for every tax or accounting question.
These local copy changes do not update GitHub, Glama, PyPI or the MCP Registry.

Description saved and verified on 9 September 2026:

```text
Local MCP tools for ATO benchmarks, Payday Super timing and limited Division 7A loan reviews, plus synthetic CTR/BAS test data. Experimental reviews require professional judgement. No advice or lodgements.
```

The profile description, imported README and generated FAQs are separate.
After a manual sync on that date, the capability FAQ still claimed that no
reviewed Division 7A engine was connected. The profile and repository admin
pages expose no FAQ editor. Verify that claim is removed before reporting
the FAQ as corrected.

## Release provenance

Release `v0.1.5` predates the repository rename. Its certificate therefore
retains the historical `au-tax-mcp-server` source identity. Verify it with an
owner-scoped lookup that is then bound to the exact source and signer. The
monorepo import rewrote history, so the `v0.1.5` tag now resolves to
`5f40beffd15be54a975619d9dabacf0d1a5d128e`. That commit has the same tree
(`13698f0`) as the attested source commit, which the check names directly:

```bash
tag=v0.1.5
repo=ryanduguid/aus-accounting-mcp
wheel="aus_accounting_mcp-${tag#v}-py3-none-any.whl"
release_commit=965b4a57353c24efb978fdaff948d9b60e0158fc
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

Releases cut after the rename use the current repository identity. The next
release is signed by release-policy commit
`a601f1d8b143f92b0517f709216993660d21f480`, the commit the release workflow
pins. Update `tag` if the intended version changes and run these checks after
downloading the assets and checking `SHA256SUMS`:

```bash
tag=aus-accounting-mcp/v0.3.1
signer=a601f1d8b143f92b0517f709216993660d21f480
repo=ryanduguid/australian-accounting
wheel="aus_accounting_mcp-${tag#aus-accounting-mcp/v}-py3-none-any.whl"
release_commit="$(git ls-remote "https://github.com/$repo.git" "refs/tags/$tag^{}" | cut -f1)"
test -n "$release_commit"
gh attestation verify "$wheel" -R "$repo" \
  --source-digest "$release_commit" \
  --source-ref "refs/tags/$tag" \
  --signer-workflow ryanduguid/release-policy/.github/workflows/release-python.yml \
  --signer-digest "$signer"
gh attestation verify "$wheel" -R "$repo" \
  --predicate-type https://spdx.dev/Document/v2.3 \
  --source-digest "$release_commit" \
  --source-ref "refs/tags/$tag" \
  --signer-workflow ryanduguid/release-policy/.github/workflows/release-python.yml \
  --signer-digest "$signer"
```

Earlier monorepo releases carry the digest pinned when they were cut. Set
`signer` to:

- `3ff09b654a17b9a3b55548e25e6108ee582b00c4` for v0.1.7
- `d487b1a854bc7a6f97cc114fc68cf3fe3989ce7e` for v0.1.9
- `ff86e30038ca4fd028ce3663646ee03ae5e94692` for v0.2.0 and v0.2.1
- `c24612618f177f6dccb7ed9ee2a8648959b87e2e` for v0.2.2
- `2adf9e19b7c73970a1dd6703afb3f9c27b7972d7` for v0.2.3 to v0.2.6
- `87767ec809dc7f77bcd45808219adaf67841ae7b` for v0.2.7 to v0.2.10
- `ec6b0ee76446f11aefb7fa0c203f2e01b4c9a711` for v0.2.11 to v0.3.0
