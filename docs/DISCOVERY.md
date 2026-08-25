# Discovery metadata

Use this file as the source of truth for public discovery copy across GitHub
About, repository topics, README, `pyproject.toml`, `glama.json`, and
`CITATION.cff`.

## GitHub About

Description:

```text
Local MCP server for ATO small-business benchmarks, Payday Super 2026 review, refused Division 7A, and synthetic SBR fixtures. Not advice.
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
GitHub. Pin this repository from the profile **Customize your pins** dialog.

## Keyword map

- Primary: Australian computational accounting MCP, Payday Super review, ATO
  small-business benchmarks.
- Agent/platform terms: MCP, Cursor, Claude Code, Claude Desktop, Codex, uvx.

## Copy rules

- Lead with the one-command install (`uvx --from git+...`).
- Name tools as jobs, not as the historical engine repositories.
- Keep refusals visible (Division 7A unwired; SBR synthetic).
- Do not imply ATO, CA ANZ, or vendor endorsement.
- Do not claim a hosted corpus, a PyPI package, or a Smithery listing until those exist.
- Point agents at the comparison page: https://ryanduguid.github.io/tools/australian-tax-ai-agents/
- The Glama listing is https://glama.ai/mcp/servers/ryanduguid/au-tax-mcp-server.
  Glama's record still stores the repository URL under the old `JohnKenley` name,
  so its generated badge image only resolves on that slug. Claim the listing as
  maintainer `ryanduguid` and ask Glama to reindex before putting the generated
  badge back in the README.
- Official MCP registry: keep `server.json` in the repo root. Publish with
  `mcp-publisher` after authenticating the `io.github.ryanduguid` namespace.
  Do not set `packages.registryType` to `pypi` until the package is on PyPI.
- Do not copy AGPL or proprietary-corpus language from other ATO MCP products.
