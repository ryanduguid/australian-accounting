# Aus Accounting MCP

```
+--------------------------------------------------------+
|                   Aus Accounting MCP                   |
+--------------------------------------------------------+
|          MCP server for AU tax review engines          |
+---------------------------+----------------------------+
| DR  what it gives you     | CR  what it needs          |
+---------------------------+----------------------------+
| AU tax review MCP tools   | MCP-capable host client    |
| uses reviewed engines     | uv or uvx to install it    |
| synthetic SBR fixtures    | -                          |
+---------------------------+----------------------------+
```

[![tests](https://github.com/ryanduguid/australian-accounting/actions/workflows/ci.yml/badge.svg)](https://github.com/ryanduguid/australian-accounting/actions/workflows/ci.yml)
[![PyPI](https://img.shields.io/pypi/v/aus-accounting-mcp.svg?color=5C2D91&labelColor=04001F)](https://pypi.org/project/aus-accounting-mcp/)
[![Python](https://img.shields.io/badge/Python-3.10+-5C2D91?logo=python&logoColor=white&labelColor=04001F)](https://www.python.org/)
[![Model Context Protocol](https://img.shields.io/badge/MCP-Standard%20Protocol-5C2D91?labelColor=04001F)](https://modelcontextprotocol.io/)
[![MCP Registry](https://img.shields.io/badge/MCP_Registry-io.github.ryanduguid%2Faus--accounting-5C2D91?labelColor=04001F)](https://registry.modelcontextprotocol.io/v0.1/servers/io.github.ryanduguid%2Faus-accounting/versions/latest)
[![Glama](https://glama.ai/mcp/servers/ryanduguid/au-tax-mcp-server/badge)](https://glama.ai/mcp/servers/ryanduguid/au-tax-mcp-server)
[![License: MIT](https://img.shields.io/badge/License-MIT-4F485E.svg?labelColor=04001F)](https://opensource.org/licenses/MIT)

[30-second proof](#30-second-proof) · [Install](#install) · [Client setup](#client-integration) · [Tool reference](#tools) · [Release notes](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/RELEASE_NOTES.md)

**Aus Accounting MCP** provides Australian accounting review tools for ATO benchmarks, Payday Super timing and limited Division 7A loan reviews, plus synthetic CTR/BAS fixtures for integration tests. Compatible with Claude Desktop, Claude Code, Cursor, Codex and Antigravity.

Payday Super and Division 7A are experimental reviews, not compliance or tax determinations. SBR payloads are synthetic fixtures, not lodgments.

> [!WARNING]
> **Not tax advice.** This server returns structured results, refusals, and citations. It does not lodge, and it does not replace a registered agent. See [DISCLAIMER.md](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/DISCLAIMER.md).

This MCP applies defined tests to figures the operator supplies: Payday Super timing, ATO small-business benchmark ratios, and a limited Division 7A loan review. It holds no ATO documents. For looking up rulings, use a document-retrieval MCP. Comparison: [Australian tax tools for AI agents](https://duguid.com.au/tools/australian-tax-ai-agents/).

This server does not reimplement tax law. Payday Super, ATO benchmark and Division 7A computations are delegated to:

- [payday-super-checker](https://github.com/ryanduguid/australian-accounting/tree/main/packages/payday-super-checker) (`payday-super-checker`)
- [ato-benchmark-compare](https://github.com/ryanduguid/australian-accounting/tree/main/packages/ato-benchmark-compare) (`ato-benchmark-compare`)
- [div7a-loan-review](https://github.com/ryanduguid/australian-accounting/tree/main/packages/div7a-loan-review) (`div7a-loan-review`)

The Division 7A adapter covers reviewed s 109N loan terms and benchmark rates plus s 109E minimum yearly repayments for one operator-supplied amalgamated loan. It fails closed on unknown facts and refuses unsupported matters such as forming amalgamated loans, s 109R repayment classification, unpaid present entitlements, distributable surplus, interposed entities, debt forgiveness, and Commissioner discretion.

## 30-second proof

![Static terminal proof of synthetic BAS output and Division 7A loan review](https://raw.githubusercontent.com/ryanduguid/australian-accounting/main/apps/aus-accounting-mcp/docs/quick-proof.webp)

**Release proof:** from `apps/aus-accounting-mcp/` in a repository checkout, run the fabricated
demonstration without starting the stdio server:

```bash
uv run --locked aus-accounting-mcp-demo
```

The [checked text transcript](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/docs/quick-proof.txt) is the accessible source of truth for the image. Its registered MCP calls return a synthetic BAS fixture with `synthetic: true` and `not_a_lodgment: true`, then review fabricated Division 7A loan facts through the delegated engine.

Expected structured success:

```text
synthetic: true
not_a_lodgment: true
form_type: BAS_AU_ACTIVITY_STATEMENT
summary.total_payable_to_ato: "42500.00"
```

Expected structured Division 7A review:

```text
engine: div7a-loan-review
gate.verdict: COMPLYING
minimum_yearly_repayment.verdict: MYR_MET
minimum_yearly_repayment.myr_required: "108770.00"
minimum_yearly_repayment.shortfall: "0.00"
```

The example is fabricated, is not a lodgment or Division 7A determination, is not tax advice, and requires human review before any consequential accounting action. It does not model s 109R, distributable surplus, unpaid present entitlements or the other refused areas. It neither uses client data nor contacts external services.

### Asset provenance

| Asset | Purpose | Source | Licence | Creation | SHA-256 | Refresh trigger |
|---|---|---|---|---|---|---|
| `docs/quick-proof.webp` | Static terminal summary of the two checked demonstration outcomes | `docs/quick-proof.txt`, emitted by `aus-accounting-mcp-demo` | MIT | `uv run --locked --extra dev python scripts/render_demo_image.py docs/quick-proof.txt docs/quick-proof.webp` with Pillow 12.3.0 | `90f983a9b8f76455e9f473330c8b999f9929870731f0c9fbc70e87cd790a43df` | Regenerate when the transcript, demo output, render constants or pinned Pillow version changes |

Name mapping: public name Aus Accounting MCP; repository australian-accounting; Python distribution aus-accounting-mcp; stdio MCP executable aus-accounting-mcp; demonstration executable aus-accounting-mcp-demo; MCP Registry identity io.github.ryanduguid/aus-accounting.

Canonical published release and compatibility references: [CI](https://github.com/ryanduguid/australian-accounting/actions/workflows/ci.yml), [v0.1.7 release](https://github.com/ryanduguid/australian-accounting/releases/tag/aus-accounting-mcp/v0.1.7), [PyPI 0.1.7](https://pypi.org/project/aus-accounting-mcp/0.1.7/), [MCP Registry 0.1.7](https://registry.modelcontextprotocol.io/v0.1/servers/io.github.ryanduguid%2Faus-accounting/versions/0.1.7), and [compatibility.json](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/compatibility.json). Treat a version as published only after its target resolves and matches the compatibility record. The record links each engine's maintained source and release. The runtime `law_content_date` and `source` fields stay engine-owned.

## Install

Python 3.10+ and [uv](https://docs.astral.sh/uv/). This server and its engines
are published to PyPI, and the server pins its reviewed engines to exact versions.

```bash
uvx aus-accounting-mcp
```

Use [CITATION.cff](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/CITATION.cff) for this source version. The latest
published provenance is the [v0.1.7 release record](https://github.com/ryanduguid/australian-accounting/releases/tag/aus-accounting-mcp/v0.1.7).

For a local editable tree, clone the repository, change into
`apps/aus-accounting-mcp/`, then run `pip install -e .`. The repository root is not
an installable package.

The executable waits for a client over stdio; it does not open a web page or start
an HTTP service. No API key is required. Installation downloads packages; tool
calls use bundled data locally without writing records or contacting services.

## Client integration

**Standard config** works with hosts that run a local stdio MCP server:

```json
{
  "mcpServers": {
    "aus-accounting": {
      "command": "uvx",
      "args": [
        "aus-accounting-mcp"
      ]
    }
  }
}
```

Ready-made copies live in [`clients/`](https://github.com/ryanduguid/australian-accounting/tree/main/apps/aus-accounting-mcp/clients).

### Cursor

[![Add to Cursor](https://img.shields.io/badge/Cursor-Add%20MCP-black)](https://cursor.com/en/install-mcp?name=aus-accounting&config=eyJjb21tYW5kIjoidXZ4IiwiYXJncyI6WyJhdXMtYWNjb3VudGluZy1tY3AiXX0=)

Or drop the standard config into `~/.cursor/mcp.json`.

### Claude Desktop

Paste the standard config into `claude_desktop_config.json` (`%APPDATA%\Claude\` on Windows, `~/Library/Application Support/Claude/` on macOS).

### Claude Code

```bash
claude mcp add aus-accounting -- uvx aus-accounting-mcp
```

### Codex

```bash
codex mcp add aus-accounting -- uvx aus-accounting-mcp
```

## Tools

| Tool | Job | Engine |
| :--- | :--- | :--- |
| `list_ato_benchmark_industries` | List or search the shipped ATO business types | ato-benchmark-compare |
| `get_ato_benchmarks` | Compare operator-supplied bucket totals to ATO ranges | ato-benchmark-compare |
| `calc_payday_super_deadline` | Review one contribution against Payday Super timing | payday-super-checker |
| `get_div7a_benchmark_rate` | Return the reviewed s 109N(2) rate for a year, or `UNKNOWN` | div7a-loan-review |
| `review_div7a_loan` | Review s 109N terms and s 109E minimum yearly repayment for one operator-supplied amalgamated loan | div7a-loan-review |
| `refuse_div7a` | Refuse Division 7A matters outside the reviewed engine scope | MCP policy |
| `generate_synthetic_sbr_fixture` | Synthetic CTR/BAS for agent tests (`synthetic: true`) | local fixture |

MCP initialization supplies server-wide instructions for choosing tools and handling
missing facts. Every tool publishes an output schema describing its returned fields,
including verdicts, decimal strings, warnings and source information. `ok: true`
means the tool ran, not that the review passed; retain `UNKNOWN`, `REFUSED`,
`not_supplied` and `null` results when presenting findings. Engine audit fields are
preserved, and clients receive the same payload in structured content and JSON text.
An unavailable Division 7A `benchmark_year_used` may be the engine's empty string
or null; neither is evidence of a reviewed year.

The Division 7A tools default to `response_detail="summary"`, retaining outcomes,
amounts, caveats, versions and a verification link while reducing tool-result size.
Pass `response_detail="full"` when the complete provenance, statutory trace and
per-limb audit material are required.

For concise industry discovery, call `list_ato_benchmark_industries` with
`{"search":"shop","year":"2023-24","limit":20}`. `count` is the number returned;
`total_count` is the number matching the search, while `total_business_types` covers
the whole dataset. If `has_more` is true, pass `next_offset` as `offset` with the
same search, limit and returned `benchmark_year` as `year`. `next_offset` is null
at the end. Limits must be integers from 1 to 100 and offsets non-negative integers.
Omitting `limit` or setting it to null preserves full-list calls; an offset still
skips that many matching entries. Source metadata accompanies every page.

Ten fabricated, read-only agent evaluation questions and exact expected answers
are in [evaluation/questions.xml](evaluation/questions.xml). The normal pytest
suite verifies their answers through a real stdio MCP session using the locked
engines. This checks answer reproducibility; it does not measure whether a model
can independently select the right tools. Evaluate that separately by presenting
the questions without the answer elements to an MCP-capable client. The fixed
dataset years and engine versions define the evaluation baseline; review expected
answers when upgrading an engine.

`calc_payday_super_deadline` requires `as_at`. It does not invent clearing-house latency and cannot confirm LCR 2026/1 transition allocation. A remittance date alone cannot produce `ON_TIME`.

Omitted ATO expense buckets are `not_supplied`, not zero. Every ATO ratio divides by turnover, which the ATO rule takes from sales or from total business income, so omitting `other_income` leaves every ratio `not_supplied` until you establish that figure. Pass `0` where you have established there is none. Withholding covers the engine's prose as well as the structured fields. A `notes` or `checks_to_make` entry that states an amount resting on an omitted bucket is withheld with those fields, and `notes` says so.

Amounts, including Division 7A loan balances and payments, are decimal strings, finite, at most two decimal places, and no greater than AUD 1,000,000,000,000.00. Dates are ISO-8601. Payday Super uses payday-super-checker's national SGAA 1992 s 6(1) calendar.

Ask the agent:

```text
Compare these P&L buckets to the ATO small-business benchmarks for this industry. Omit buckets I have not supplied. Do not treat missing as zero.
```

```text
Review this Payday Super contribution. QE day, remitted date, and fund-receipt date are in the CSV. as_at is today. Do not invent an SGC charge.
```

```text
Review this operator-supplied Division 7A amalgamated loan for s 109N terms and the s 109E minimum yearly repayment. Leave unknown facts unknown and refuse questions outside the reviewed scope.
```

## Licence

MIT License. Created by Ryan Duguid. Boundary statement: [DISCLAIMER.md](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/DISCLAIMER.md). Discovery copy: [docs/DISCOVERY.md](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/docs/DISCOVERY.md). Cite: [CITATION.cff](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/CITATION.cff).

<!-- mcp-name: io.github.ryanduguid/aus-accounting -->
