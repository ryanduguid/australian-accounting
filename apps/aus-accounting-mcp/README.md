# Aus Accounting MCP

Ryan Duguid is not a registered tax agent or BAS agent. Project support is limited to software issues reproduced with fabricated data. Do not send taxpayer information or request advice, return preparation, tax-treatment confirmation, or lodgement.

[![tests](https://github.com/ryanduguid/australian-accounting/actions/workflows/ci.yml/badge.svg)](https://github.com/ryanduguid/australian-accounting/actions/workflows/ci.yml)
[![PyPI](https://img.shields.io/pypi/v/aus-accounting-mcp.svg?color=5C2D91&labelColor=04001F)](https://pypi.org/project/aus-accounting-mcp/)
[![License: MIT](https://img.shields.io/badge/License-MIT-4F485E.svg?labelColor=04001F)](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/LICENSE)
[![Python 3.10+](https://img.shields.io/badge/python-3.10%2B-5C2D91.svg?logo=python&logoColor=white&labelColor=04001F)](https://www.python.org/downloads/)

Local Australian accounting tools for AI assistants. Compare business figures with
ATO benchmarks, review Payday Super timing and check limited Division 7A loan terms
and repayments. Calculate 7 bounded tax worksheets, search a configured local
Markdown library with file and line citations, and search a configured legislation
corpus for provisions, rates and thresholds cited to their Act, section and
compilation. Includes synthetic CTR/BAS fixtures for integration testing.

> Not tax advice. Payday Super and Division 7A reviews are experimental and need
> human review before consequential accounting action. Fixtures are not a lodgement.
> See [DISCLAIMER.md](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/DISCLAIMER.md).

## Install

For a local file handoff that retains separate engine outputs, see the
[group review example](examples/GROUP-REVIEW.md).

Requires Python 3.10+ and [uv](https://docs.astral.sh/uv/):

```bash
uvx aus-accounting-mcp
```

The server waits for an MCP client over stdio. No API key, account or sign-in is
required. Installation downloads packages; tool calls then read bundled data and the
folders you explicitly configure, locally, without contacting services or changing
records. It does not fetch documents from the ATO or the Federal Register, and it
does not lodge.

`uvx` keeps using the version it first downloaded, so a new release does not reach
you on its own. To upgrade, name the release in your client configuration, for example
`aus-accounting-mcp==<version>`; uvx downloads it, and later launches reuse the cached
copy without a download until the uv cache is cleaned.

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

For Gemini CLI:

```bash
gemini mcp add -s user aus-accounting uvx aus-accounting-mcp
```

For VS Code:

```bash
code --add-mcp "{\"name\":\"aus-accounting\",\"command\":\"uvx\",\"args\":[\"aus-accounting-mcp\"]}"
```

Windsurf reads the standard config from `~/.codeium/windsurf/mcp_config.json`, and
any other host that launches a local stdio server runs it the same way. ChatGPT
connectors and the Claude.ai web app accept a remote URL rather than a local
command, so they cannot run this server.

## Tools

| Tool | Use |
|---|---|
| `list_ato_benchmark_industries` | Find a business type in the bundled ATO dataset. |
| `get_ato_benchmarks` | Compare supplied P&L figures with ATO benchmark ranges. |
| `calc_payday_super_deadline` | Review timing for one super contribution. |
| `review_payday_super_contributions` | Review related contributions together for one employer. |
| `build_payday_super_evidence_pack` | Return 4 review files in memory using the pinned Payday Super checker. |
| `calculate_tax_worksheet` | Calculate one of 7 worksheets with established scope and period. |
| `search_accounting_library` | Search a configured local Markdown library. |
| `read_accounting_library` | Read cited lines from that library. |
| `search_tax_legislation` | Find in-force provisions in a configured local legislation corpus, cited to Act, section, compilation and register page; `in_force_only=false` includes superseded compilations. |
| `read_tax_legislation_section` | Read one cited provision from that corpus in full, with up to 5 provisions either side when `neighbours` is set. |
| `define_tax_term` | Find an expression's statutory definitions in that corpus's dictionary sections, exact matches first; no match is not proof the expression is undefined. |
| `search_tax_rates` | Find legislated rate and threshold rows with the provision that sets them, optionally for one stated `year`. |
| `get_div7a_benchmark_rate` | Get a reviewed Division 7A benchmark rate, or `UNKNOWN`. |
| `review_div7a_loan` | Review s 109N terms and s 109E minimum yearly repayments for one supplied amalgamated loan. |
| `refuse_div7a` | Explain unsupported Division 7A matters. Call without arguments. |
| `generate_synthetic_sbr_fixture` | Generate fabricated CTR/BAS test data. |

Missing figures remain unknown. Preserve `UNKNOWN`, `REFUSED`, `not_supplied` and
`null` results; `ok: true` means the tool ran, not that a review passed.

The 7 worksheets cover bounded GST, resident basic tax, CGT, FBT, first-year
depreciation, quarterly SG and Schedule 1 PAYG withholding cases. Read `aus-accounting://scope` before supplying
scope confirmation. Most support 2025-26; see the reference for periods and exclusions.

With the development worksheet engine, `aus-accounting://scope` also includes
engine-owned period dates, required inputs and units, methods, Library example
references and fabricated `example.facts` that run the worksheet tool as a
demonstration. The example's `scope_confirmed: true` is fabricated with the
rest of it: for real facts, pass true only after a person has confirmed the
scope conditions, because the engine treats that flag as the operator's
confirmation. With the published pinned engine, the resource retains its existing scope,
source and source-check date fields. The richer catalogue remains unreleased;
published dependency pins have not changed.
To enable library retrieval, set `AUS_ACCOUNTING_LIBRARY_ROOT` in the server's
environment to an authorised Markdown folder. Returned excerpts enter the calling
assistant's context. The package contains no reference library.

## Legislation corpus

Set `AUS_ACCOUNTING_CORPUS_ROOT` to a legislation corpus you have built or obtained
and authorise the assistant to read. The four corpus tools then cite every
provision to its Act, section, compilation number, compilation date and register
page, and carry the corpus licence and attribution with the text.

The package ships no corpus and downloads nothing, so the corpus stays yours: no
account, no hosted index and no record of what you searched for.
[au-tax-legislation-corpus](https://github.com/ryanduguid/au-tax-legislation-corpus)
builds one from the Federal Register of Legislation in the expected layout.

A row is a point-in-time copy from one build, not a live lookup, and
`version_is_current` records what was true when the corpus was built. Check the
compilation date and the register page before relying on a provision, and treat a
rate row as the text of one provision rather than a calculation or a confirmed
current figure. Retrieval does not extend what the reviewed engines calculate. The
[reference](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/docs/REFERENCE.md#local-legislation-corpus)
covers the layout, fields, bounds and a worked example.

Payday Super needs an explicit assessment date and fund-receipt evidence, both
the date and the amount received (`received` with `matched_amount`), before it
should be read as `ON_TIME`. The pinned checker 0.1.7 leaves a timely receipt
date without an amount `UNKNOWN`. A late receipt without an amount stays
`LATE`, without reducing the shortfall. MCP 0.2.4 pins checker 0.1.6, which
assumes full receipt in that case and says so in a caveat. Check the
`aus-accounting://payday-coverage` resource for
bundled rate and calendar coverage, and retain the result's caveats.

Division 7A covers the reviewed s 109N/s 109E scope only. It refuses matters such
as forming amalgamated loans, s 109R repayment classification, unpaid present
entitlements and distributable surplus. The
[reference](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/docs/REFERENCE.md)
covers all exclusions, input rules, prompts, resources and evaluation instructions.

## Payday Super evidence pack

In v0.2.5, `build_payday_super_evidence_pack` accepts the
same `contributions` and explicit `as_at` as grouped review. It delegates the
assessment and all 4 artefacts to the checker. There are no path arguments,
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

This documentation describes v0.2.7, which pins `payday-super-checker==0.1.7`
and `div7a-loan-review==0.1.4` and includes the pack builder. Run
`uvx aus-accounting-mcp==0.2.7` for this version. Check the
release and compatibility references below before treating a source version as published. An installation with
checker v0.1.3 returns a feature-unavailable error for this tool; the existing
tools continue to work. See the [website guide](https://duguid.com.au/tools/australian-tax-ai-agents/)
for examples and scope.

## 30-second demonstration

From `apps/aus-accounting-mcp/` in a repository checkout, run the fabricated example. It shows the tools running on synthetic data; it is not evidence that a review passed:

```bash
uv run --locked aus-accounting-mcp-demo
```

![Static terminal demonstration of synthetic BAS output and Division 7A loan review](https://raw.githubusercontent.com/ryanduguid/australian-accounting/main/apps/aus-accounting-mcp/docs/quick-proof.webp)

The [checked text transcript](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/docs/quick-proof.txt)
and [proof and provenance](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/docs/REFERENCE.md#demonstration-and-provenance)
record the expected output, limitations and asset source.

## Licence and releases

MIT License. Created by Ryan Duguid.
[Release notes](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/RELEASE_NOTES.md),
[v0.2.7 release record](https://github.com/ryanduguid/australian-accounting/releases/tag/aus-accounting-mcp/v0.2.7),
[CITATION.cff](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/CITATION.cff).

<!-- mcp-name: io.github.ryanduguid/aus-accounting -->
