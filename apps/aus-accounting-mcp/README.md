# Aus Accounting MCP

Release `aus-accounting-mcp==0.5.0` provides fourteen worksheets through
`australian-tax-calculators==0.2.0`, including Schedule 8 study-loan withholding,
the bounded annual Medicare levy, LITO and Division 293. The preceding 0.4.0
release provides ten through calculators 0.1.8. See the
[worksheet release verification](docs/worksheet-release.md) for the installed
package checks. The annual Medicare worksheet does not apply Medicare
adjustments to payroll withholding.

<div align="center">

[![tests](https://img.shields.io/github/actions/workflow/status/ryanduguid/australian-accounting/ci.yml?branch=main&label=tests&labelColor=000000&style=flat)](https://github.com/ryanduguid/australian-accounting/actions/workflows/ci.yml)<br>
[![PyPI](https://img.shields.io/pypi/v/aus-accounting-mcp.svg?color=66023C&labelColor=000000&style=flat)](https://pypi.org/project/aus-accounting-mcp/)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache--2.0-66023C?labelColor=000000&style=flat&color=66023C)](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/LICENSE)
[![Python 3.14+](https://img.shields.io/badge/python-3.14%2B-66023C?logo=python&logoColor=E5E4E2&labelColor=000000&style=flat&color=66023C)](https://www.python.org/downloads/)

</div>

Australian accounting tools for Claude, Cursor, Codex and other MCP clients. The
server runs on your machine over stdio, needs no API key or account, and makes no
network calls once installed.

Find its published identity in the
[MCP Registry](https://registry.modelcontextprotocol.io/v0.1/servers?search=io.github.ryanduguid%2Faus-accounting&version=latest).
The listing identifies the distribution; it does not prove a client's installed transport works.

- **ATO small business benchmarks:** compare a business's P&L figures with the
  benchmark ranges for its industry.
- **Payday Super:** review contribution timing for one payday or a whole employer,
  and build a 4-file evidence pack for a practitioner's workpaper.
- **Division 7A:** get the benchmark interest rate and review s 109N loan terms and
  s 109E minimum yearly repayments for one loan. Matters outside that scope are refused.
- **14 tax and super worksheets:** GST, resident basic tax, CGT, FBT, first-year
  depreciation, quarterly SG, Schedule 1 PAYG withholding, super contribution caps,
  account-based pension minimums, study and training loan repayments, Schedule 8 study
  loan withholding, a single person's Medicare levy, the low income tax offset and
  Division 293 tax, each within a stated period and scope. The Medicare levy worksheet
  refuses a person to whom s 9A (lump sum payments in arrears) applies.
- **Cited research:** search your own Markdown library, or a local copy of federal tax
  legislation, with every result cited to its file and line, or its Act, section and
  compilation.
- **Test data:** generate synthetic CTR and BAS fixtures for integration testing.

> Not tax advice. Payday Super and Division 7A reviews are experimental and need
> human review before consequential accounting action. Fixtures are not a lodgement.
> Ryan Duguid is not a registered tax agent or BAS agent. Project support is limited
> to software issues reproduced with fabricated data. Do not send taxpayer information
> or request advice, return preparation, tax-treatment confirmation, or lodgement.
> See [DISCLAIMER.md](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/DISCLAIMER.md).

## Install

Requires Python 3.14+ and [uv](https://docs.astral.sh/uv/):

```bash
uvx --from 'aus-accounting-mcp==0.6.0' aus-accounting-mcp
```

This selects the published [0.6.0 release](https://github.com/ryanduguid/australian-accounting/releases/tag/aus-accounting-mcp/v0.6.0).
The source checkout may be ahead of the published release. The pin selects the
server version; it does not lock every transitive dependency.

The server waits for an MCP client over stdio. Tool calls read bundled data and any
folders you configure. They do not contact the ATO or the Federal Register, change
records or lodge. Your MCP host still sends tool arguments and results to its model
provider, so use fabricated data with a hosted model.

To upgrade, replace the version in your client configuration with a later release
published on PyPI.

## Client integration

For Claude Desktop, Cursor, Windsurf and other hosts that run local stdio servers:

```json
{
  "mcpServers": {
    "aus-accounting": {
      "command": "uvx",
      "args": ["--from", "aus-accounting-mcp==0.6.0", "aus-accounting-mcp"]
    }
  }
}
```

[Add to Cursor](https://cursor.com/en/install-mcp?name=aus-accounting&config=eyJjb21tYW5kIjoidXZ4IiwiYXJncyI6WyItLWZyb20iLCJhdXMtYWNjb3VudGluZy1tY3A9PTAuNi4wIiwiYXVzLWFjY291bnRpbmctbWNwIl19),
or from a terminal:

```bash
claude mcp add --scope user aus-accounting -- uvx --from 'aus-accounting-mcp==0.6.0' aus-accounting-mcp
codex mcp add aus-accounting -- uvx --from 'aus-accounting-mcp==0.6.0' aus-accounting-mcp
```

The [client setup guide](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/docs/REFERENCE.md#client-setup)
covers Gemini CLI, VS Code, Zed and others. ChatGPT connectors and the Claude.ai web
app accept only remote servers, so they cannot run this one.

## Example requests

Ask in plain English. Each of these fabricated requests resolves to one tool call:

| Ask | Tool |
|---|---|
| "Compare a hairdresser with $180,000 sales, no other income, $60,000 wages, $30,000 rent and $40,000 other expenses to the ATO benchmarks." | `get_ato_benchmarks` |
| "Super for a 6 August 2026 payday was $120 and was sent on 14 August. As at 20 August, is it on time?" | `calc_payday_super_deadline` |
| "What is the Division 7A benchmark interest rate for 2026-27?" | `get_div7a_benchmark_rate` |
| "An employee who gave a TFN and claims the tax-free threshold (scale 2) is paid a regular $1,000 weekly wage in 2026-27, with no offsets, loans, bonuses or other adjustments. How much PAYG should be withheld?" | `calculate_tax_worksheet` |

## Tools

| Tool | Use |
|---|---|
| `list_ato_benchmark_industries` | Find a business type in the bundled ATO dataset. |
| `get_ato_benchmarks` | Compare supplied P&L figures with ATO benchmark ranges. |
| `calc_payday_super_deadline` | Review timing for one super contribution. |
| `review_payday_super_contributions` | Review related contributions together for one employer. |
| `build_payday_super_evidence_pack` | Return 4 review files in memory for a practitioner's workpaper. |
| `calculate_tax_worksheet` | Calculate one of 14 worksheets with established scope and period. |
| `get_div7a_benchmark_rate` | Get a reviewed Division 7A benchmark rate, or `UNKNOWN`. |
| `review_div7a_loan` | Review s 109N terms and s 109E minimum yearly repayments for one supplied amalgamated loan. |
| `refuse_div7a` | Explain unsupported Division 7A matters. Call without arguments. |
| `review_gst_coding` | Give each coded bank line a review state against supplied GST registration dates. |
| `reconcile_bas` | Compare lodged BAS label figures with ledger lines per period. |
| `get_lodgment_due_dates` | List ATO 2026-27 lodgment program due dates for BAS, IAS and income tax returns, self-lodged or through an agent, with `UNKNOWN` where a fact is missing. |
| `search_accounting_library` | Search a configured local Markdown library. |
| `read_accounting_library` | Read cited lines from that library. |
| `search_tax_legislation` | Find provisions in a configured local legislation corpus, best match first. |
| `read_tax_legislation_section` | Read one cited provision in full, in parts when it is long. |
| `define_tax_term` | Find an expression's statutory definitions in that corpus. |
| `search_tax_rates` | Find legislated rates and thresholds with the provision that sets them. |
| `generate_synthetic_sbr_fixture` | Generate fabricated CTR/BAS test data. |

Missing figures remain unknown. Preserve `UNKNOWN`, `REFUSED`, `not_supplied` and
`null` results; `ok: true` means the tool ran, not that a review passed. Supply an
explicit assessment date and fund-receipt evidence, both the date and the amount
received (`received` with `matched_amount`), before reading a Payday Super result
as `ON_TIME`.

The research tools need a folder you supply: set `AUS_ACCOUNTING_LIBRARY_ROOT` to an
authorised Markdown folder, `AUS_ACCOUNTING_CORPUS_ROOT` to a legislation corpus, or
`AUS_ACCOUNTING_RULINGS_ROOT` to ATO rulings runs, such as those built by
[au-tax-legislation-corpus](https://github.com/ryanduguid/au-tax-legislation-corpus).
The [corpus setup walkthrough](docs/corpus-setup.md) verifies the existing builder
and MCP tools together using a fabricated page before any live capture.
The package ships none of them. A corpus or rulings row is a point-in-time copy, not
a live lookup, and an edited version of private advice cannot be relied on by anyone.
Legislation results preserve a row's capture date and recorded successor date when
available. Rulings results expose a captured withdrawal notice; `UNKNOWN` means
withdrawal status was not established, including in older runs. Notice presence
does not decide its effect for an earlier period or establish today's status.
Point the library at reference material, or at one engagement at a time, never at a
folder of mixed client files: every match enters the assistant's context, whichever
client it came from.

The [reference](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/docs/REFERENCE.md)
covers every exclusion, input rule, prompt, resource, the evidence pack format and
the evaluation suite. The [website guide](https://duguid.com.au/tools/australian-tax-ai-agents/)
has further examples. For the supplier information a firm needs for its AI register,
and answers to the questions buyers put to tax-research tools, see
[For a firm assessing this server](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/docs/firm-assessment.md).

## 30-second demonstration

From `apps/aus-accounting-mcp/` in a repository checkout, run the fabricated example. It shows the tools running on synthetic data; it is not evidence that a review passed:

```bash
uv run --locked aus-accounting-mcp-demo
```

![Static terminal demonstration of synthetic BAS output and Division 7A loan review](https://raw.githubusercontent.com/ryanduguid/australian-accounting/main/apps/aus-accounting-mcp/docs/quick-proof.webp)

The [checked text transcript](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/docs/quick-proof.txt)
and [proof and provenance](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/docs/REFERENCE.md#demonstration-and-provenance)
record the expected output, limitations and asset source.

## Conformance cases

[`conformance/cases.json`](conformance/cases.json) holds eight Payday Super and
Division 7A calls with the results this server returns, including the cases it
leaves `UNKNOWN` or refuses. Another implementation can replay them; see
[conformance/README.md](conformance/README.md).

## Licence and releases

Apache-2.0 licensed. Created by Ryan Duguid. If you redistribute this work or build on it, keep the attribution in [NOTICE](NOTICE).
[Release notes](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/RELEASE_NOTES.md),
[v0.7.0 release record](https://github.com/ryanduguid/australian-accounting/releases/tag/aus-accounting-mcp/v0.7.0),
[CITATION.cff](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/CITATION.cff).

<!-- mcp-name: io.github.ryanduguid/aus-accounting -->
