# au-tax-mcp-server

[![tests](https://github.com/ryanduguid/au-tax-mcp-server/actions/workflows/ci.yml/badge.svg)](https://github.com/ryanduguid/au-tax-mcp-server/actions/workflows/ci.yml)
[![Python](https://img.shields.io/badge/Python-3.10+-5C2D91?logo=python&logoColor=white&labelColor=04001F)](https://www.python.org/)
[![Model Context Protocol](https://img.shields.io/badge/MCP-Standard%20Protocol-5C2D91?labelColor=04001F)](https://modelcontextprotocol.io/)
[![Glama](https://img.shields.io/badge/Glama-MCP%20Directory-5C2D91?labelColor=04001F)](https://glama.ai/mcp/servers/ryanduguid/au-tax-mcp-server)
[![License: MIT](https://img.shields.io/badge/License-MIT-4F485E.svg?labelColor=04001F)](https://opensource.org/licenses/MIT)

MCP facade over reviewed Australian computational accounting engines. Compatible with Claude Desktop, Claude Code, Cursor, Codex and Antigravity.

Payday Super is an experimental review (not a compliance determination). Division 7A is **refused** - there is no repayment calculator. SBR payloads are synthetic fixtures.

> [!WARNING]
> **Not tax advice.** This server returns structured results, refusals, and citations. It does not lodge, and it does not replace a registered agent. See [DISCLAIMER.md](DISCLAIMER.md).

This server does **not** reimplement tax law. Payday Super and ATO small-business benchmarks are delegated to:

- [payday-super-checker](https://github.com/ryanduguid/payday-super-checker) (`payday-super-checker`)
- [ato-benchmark-compare](https://github.com/ryanduguid/ato-benchmark-compare) (`ato-benchmark-compare`)

Division 7A is **refused** until a reviewed engine exists. SBR payloads are **synthetic fixtures**, not lodgments.

## Install

Python 3.10+ and [uv](https://docs.astral.sh/uv/). The engines come from PyPI at exact versions; this server itself installs from the repository until its own first PyPI release:

```bash
uvx --from git+https://github.com/ryanduguid/au-tax-mcp-server aus-accounting-mcp
```

Clone and `pip install .` still works when you want a local editable tree.

## Client integration

**Standard config** works with hosts that run a local stdio MCP server:

```json
{
  "mcpServers": {
    "aus-accounting": {
      "command": "uvx",
      "args": [
        "--from",
        "git+https://github.com/ryanduguid/au-tax-mcp-server",
        "aus-accounting-mcp"
      ]
    }
  }
}
```

Ready-made copies live in [`clients/`](clients/).

### Cursor

[![Add to Cursor](https://img.shields.io/badge/Cursor-Add%20MCP-black)](https://cursor.com/en/install-mcp?name=aus-accounting&config=eyJjb21tYW5kIjoidXZ4IiwiYXJncyI6WyItLWZyb20iLCJnaXQraHR0cHM6Ly9naXRodWIuY29tL3J5YW5kdWd1aWQvYXUtdGF4LW1jcC1zZXJ2ZXIiLCJhdXMtYWNjb3VudGluZy1tY3AiXX0=)

Or drop the standard config into `~/.cursor/mcp.json`.

### Claude Desktop

Paste the standard config into `claude_desktop_config.json` (`%APPDATA%\Claude\` on Windows, `~/Library/Application Support/Claude/` on macOS).

### Claude Code

```bash
claude mcp add aus-accounting -- uvx --from git+https://github.com/ryanduguid/au-tax-mcp-server aus-accounting-mcp
```

### Codex

```bash
codex mcp add aus-accounting -- uvx --from git+https://github.com/ryanduguid/au-tax-mcp-server aus-accounting-mcp
```

## Tools

| Tool | Job | Engine |
| :--- | :--- | :--- |
| `list_ato_benchmark_industries` | List or search the shipped ATO business types | ato-benchmark-compare |
| `get_ato_benchmarks` | Compare operator-supplied bucket totals to ATO ranges | ato-benchmark-compare |
| `calc_payday_super_deadline` | Review one contribution against Payday Super timing | payday-super-checker |
| `refuse_div7a` | Returns a refusal. No reviewed Div 7A engine is wired | none |
| `generate_synthetic_sbr_fixture` | Synthetic CTR/BAS for agent tests (`synthetic: true`) | local fixture |

`calc_payday_super_deadline` requires `as_at`. It does not invent clearing-house latency and cannot confirm LCR 2026/1 transition allocation. A remittance date alone cannot produce `ON_TIME`. Omitted ATO expense buckets are `not_supplied`, not zero.

Amounts are decimal strings, finite, at most two decimal places, and no greater than AUD 1,000,000,000,000.00. Dates are ISO-8601. Payday Super uses payday-super-checker's national SGAA 1992 s 6(1) calendar.

Ask the agent:

```text
Compare these P&L buckets to the ATO small-business benchmarks for this industry. Omit buckets I have not supplied. Do not treat missing as zero.
```

```text
Review this Payday Super contribution. QE day, remitted date, and fund-receipt date are in the CSV. as_at is today. Do not invent an SGC charge.
```

## Licence

MIT License. Created by Ryan Duguid. Boundary statement: [DISCLAIMER.md](DISCLAIMER.md). Discovery copy: [docs/DISCOVERY.md](docs/DISCOVERY.md). Cite: [CITATION.cff](CITATION.cff).
