# Agent instructions

This package is an MCP facade. Keep statutory calculations in the delegated
`payday-super-checker`, `ato-benchmark-compare` and `div7a-loan-review` engines;
adapters may validate, translate and serialise, but must not reimplement their law
or datasets.

- Keep Division 7A limited to the delegated engine's reviewed s 109N and s 109E scope;
  refuse unsupported matters.
- Keep repository fixtures and demonstrations synthetic-only; never add client data or
  present a fixture as a lodgment.
- Route all MCP-boundary money parsing through `aus_accounting_mcp.money`; preserve
  finite decimal strings and the domain limits.
- Never invent current rates, thresholds, law dates, source dates or missing facts.
  Mutable facts and citations remain owned by the delegated engines and official sources.
- Preserve visible warnings, refusals, engine versions, no-advice language and the need
  for human review before consequential accounting action.

## CI gates

These are the current commands in `.github/workflows/ci.yml`:

```bash
uv run --locked --extra dev pytest -q
uv run --locked --extra dev ruff check aus_accounting_mcp tests
uv run --locked --extra dev mypy aus_accounting_mcp
```

## Supplementary local and release-readiness checks

These checks are not CI gates. Use them when their affected artifact changes:

```bash
uv sync --locked --extra dev
uv run --locked --extra dev python -m build
uv run --locked aus-accounting-mcp-demo
uv run --locked --extra dev python scripts/render_demo_image.py docs/quick-proof.txt docs/quick-proof.webp
uv run --locked --extra dev pytest -q tests/test_demo.py tests/test_demo_media.py tests/test_compatibility.py tests/test_engine_versions.py
```

Keep `docs/quick-proof.txt` as the accessible source of truth for
`docs/quick-proof.webp`. Route publication through the existing release workflows; do not
publish, tag, or change public metadata without explicit approval.
