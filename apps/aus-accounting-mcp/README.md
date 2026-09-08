# Aus Accounting MCP

Local Australian accounting tools for AI assistants. Compare business figures with
ATO benchmarks, review Payday Super timing and check limited Division 7A loan terms
and repayments. Includes synthetic CTR/BAS fixtures for integration testing.

> Not tax advice. Payday Super and Division 7A reviews are experimental and need
> human review before consequential accounting action. Fixtures are not a lodgement.
> See [DISCLAIMER.md](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/DISCLAIMER.md).

## Install

Requires Python 3.10+ and [uv](https://docs.astral.sh/uv/):

```bash
uvx aus-accounting-mcp
```

The server waits for an MCP client over stdio. No API key is required. Installation
downloads packages; tool calls use bundled data locally without contacting services
or changing records. It does not retrieve ATO documents or lodge returns.

## Client integration

For Claude Desktop, Cursor and other clients that support local stdio servers:

```json
{
  "mcpServers": {
    "aus-accounting": {
      "command": "uvx",
      "args": ["aus-accounting-mcp"]
    }
  }
}
```

[Add to Cursor](https://cursor.com/en/install-mcp?name=aus-accounting&config=eyJjb21tYW5kIjoidXZ4IiwiYXJncyI6WyJhdXMtYWNjb3VudGluZy1tY3AiXX0=)
or use the [client setup guide](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/docs/REFERENCE.md#client-setup).

For Codex:

```bash
codex mcp add aus-accounting -- uvx aus-accounting-mcp
```

For Claude Code:

```bash
claude mcp add aus-accounting -- uvx aus-accounting-mcp
```

## Tools

| Tool | Use |
|---|---|
| `list_ato_benchmark_industries` | Find a business type in the bundled ATO dataset. |
| `get_ato_benchmarks` | Compare supplied P&L figures with ATO benchmark ranges. |
| `calc_payday_super_deadline` | Review timing for one super contribution. |
| `get_div7a_benchmark_rate` | Get a reviewed Division 7A benchmark rate, or `UNKNOWN`. |
| `review_div7a_loan` | Review s 109N terms and s 109E minimum yearly repayments for one supplied amalgamated loan. |
| `refuse_div7a` | Explain unsupported Division 7A matters. Call without arguments. |
| `generate_synthetic_sbr_fixture` | Generate fabricated CTR/BAS test data. |

Missing figures remain unknown. Preserve `UNKNOWN`, `REFUSED`, `not_supplied` and
`null` results; `ok: true` means the tool ran, not that a review passed.

Payday Super needs an explicit assessment date and fund-receipt evidence before
it can return `ON_TIME`. Check the `aus-accounting://payday-coverage` resource for
bundled rate and calendar coverage, and retain the result's caveats.

Division 7A covers the reviewed s 109N/s 109E scope only. It refuses matters such
as forming amalgamated loans, s 109R repayment classification, unpaid present
entitlements and distributable surplus. The
[reference](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/docs/REFERENCE.md)
covers all exclusions, input rules, prompts, resources and evaluation instructions.

## 30-second proof

From `apps/aus-accounting-mcp/` in a repository checkout, run the fabricated example:

```bash
uv run --locked aus-accounting-mcp-demo
```

![Static terminal proof of synthetic BAS output and Division 7A loan review](https://raw.githubusercontent.com/ryanduguid/australian-accounting/main/apps/aus-accounting-mcp/docs/quick-proof.webp)

The [checked text transcript](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/docs/quick-proof.txt)
and [proof and provenance](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/docs/REFERENCE.md#demonstration-and-provenance)
record the expected output, limitations and asset source.

## Licence and releases

MIT License. Created by Ryan Duguid.
[Release notes](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/RELEASE_NOTES.md),
[v0.1.9 release record](https://github.com/ryanduguid/australian-accounting/releases/tag/aus-accounting-mcp/v0.1.9),
[CITATION.cff](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/CITATION.cff).

<!-- mcp-name: io.github.ryanduguid/aus-accounting -->
