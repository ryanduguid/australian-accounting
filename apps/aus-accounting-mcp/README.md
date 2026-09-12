# Aus Accounting MCP

[![tests](https://github.com/ryanduguid/australian-accounting/actions/workflows/ci.yml/badge.svg)](https://github.com/ryanduguid/australian-accounting/actions/workflows/ci.yml)
[![PyPI](https://img.shields.io/pypi/v/aus-accounting-mcp.svg?color=5C2D91&labelColor=04001F)](https://pypi.org/project/aus-accounting-mcp/)
[![License: MIT](https://img.shields.io/badge/License-MIT-4F485E.svg?labelColor=04001F)](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/LICENSE)
[![Python 3.10+](https://img.shields.io/badge/python-3.10%2B-5C2D91.svg?logo=python&logoColor=white&labelColor=04001F)](https://www.python.org/downloads/)

Local Australian accounting tools for AI assistants. Compare business figures with
ATO benchmarks, review Payday Super timing and check limited Division 7A loan terms
and repayments. Calculate six bounded tax worksheets and search a configured local
Markdown library with file and line citations. Includes synthetic CTR/BAS fixtures
for integration testing.

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
| `review_payday_super_contributions` | Review related contributions together for one employer. |
| `build_payday_super_evidence_pack` | Return four review files in memory; unreleased, requires checker evidence-pack support. |
| `calculate_tax_worksheet` | Calculate one of six worksheets with established scope and period. |
| `search_accounting_library` | Search a configured local Markdown library. |
| `read_accounting_library` | Read cited lines from that library. |
| `get_div7a_benchmark_rate` | Get a reviewed Division 7A benchmark rate, or `UNKNOWN`. |
| `review_div7a_loan` | Review s 109N terms and s 109E minimum yearly repayments for one supplied amalgamated loan. |
| `refuse_div7a` | Explain unsupported Division 7A matters. Call without arguments. |
| `generate_synthetic_sbr_fixture` | Generate fabricated CTR/BAS test data. |

Missing figures remain unknown. Preserve `UNKNOWN`, `REFUSED`, `not_supplied` and
`null` results; `ok: true` means the tool ran, not that a review passed.

The six worksheets cover bounded GST, resident basic tax, CGT, FBT, first-year
depreciation and quarterly SG cases. Read `aus-accounting://scope` before supplying
scope confirmation. Most support 2025-26; see the reference for periods and exclusions.
To enable library retrieval, set `AUS_ACCOUNTING_LIBRARY_ROOT` in the server's
environment to an authorised Markdown folder. Returned excerpts enter the calling
assistant's context. The package contains no reference library.

Payday Super needs an explicit assessment date and fund-receipt evidence before
it can return `ON_TIME`. Check the `aus-accounting://payday-coverage` resource for
bundled rate and calendar coverage, and retain the result's caveats.

Division 7A covers the reviewed s 109N/s 109E scope only. It refuses matters such
as forming amalgamated loans, s 109R repayment classification, unpaid present
entitlements and distributable surplus. The
[reference](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/docs/REFERENCE.md)
covers all exclusions, input rules, prompts, resources and evaluation instructions.

## Payday Super evidence pack (unreleased)

In the reviewed monorepo checkout, `build_payday_super_evidence_pack` accepts the
same `contributions` and explicit `as_at` as grouped review. It delegates the
assessment and all four artefacts to the checker. There are no path arguments,
fixture-path lookups, filesystem writes or network calls.

The response includes `files` keyed by `report.csv`, `practitioner-review.md`,
`exceptions.json` and `decision-log.md`. Save the strings as UTF-8 without changing
newlines or removing the CSV's initial BOM; the Markdown and JSON bind to those
exact report bytes. Source row numbers are one-based positions in the input list.
`review_exit_code` is 2 for any non-`ON_TIME` row and 0 only when all rows are
`ON_TIME`. An error remains an MCP error. No decision or sign-off is generated.

The default `response_detail="full"` includes the pack in both text and structured
content for client compatibility. Hosts that read `structuredContent.files` can
request `response_detail="compact"` to replace the duplicate text with a short
summary, the disclaimer and caveats. Both modes retain every file byte, hash and
review flag. Use full mode if the host only reads text results.

Record decisions and practitioner sign-off in `decision-log.md`; the included
checklist links to that record.

Employee identifiers are omitted from the returned pack. The calling MCP host
still receives the input references; use an approved environment and fabricated
data for demonstrations. Amounts, dates and warnings remain private workpaper
information. Missing receipt dates remain missing.

The 17-column evidence report is for the included checklist. It is not accepted
by legacy `review-pack` or the accounting review pipeline's `PaydaySuper.Report`
Excel importer. Use an ordinary 18-column checker report for that importer.

Published checker v0.1.3 does not include the pack builder. With that installation,
this tool returns a feature-unavailable error and the existing tools continue to
work. Package versions, dependency pins, `compatibility.json` and `server.json`
remain unchanged. After the checker feature is reviewed, its release and the MCP
dependency update require a separate handoff. `uvx aus-accounting-mcp` does not yet
provide this new workflow. No release is implied by these source changes.

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
[v0.2.1 release record](https://github.com/ryanduguid/australian-accounting/releases/tag/aus-accounting-mcp/v0.2.1),
[CITATION.cff](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/CITATION.cff).

<!-- mcp-name: io.github.ryanduguid/aus-accounting -->
