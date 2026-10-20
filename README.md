# Australian accounting engines and MCP server

[![tests](https://img.shields.io/github/actions/workflow/status/ryanduguid/australian-accounting/ci.yml?branch=main&label=tests&labelColor=000000&style=flat-square)](https://github.com/ryanduguid/australian-accounting/actions/workflows/ci.yml)
[![Codecov](https://img.shields.io/codecov/c/github/ryanduguid/australian-accounting?label=coverage&labelColor=000000&style=flat-square)](https://codecov.io/gh/ryanduguid/australian-accounting)
[![CodeQL](https://img.shields.io/github/actions/workflow/status/ryanduguid/australian-accounting/codeql.yml?branch=main&label=CodeQL&labelColor=000000&style=flat-square)](https://github.com/ryanduguid/australian-accounting/actions/workflows/codeql.yml)
[![Snyk](https://img.shields.io/github/actions/workflow/status/ryanduguid/australian-accounting/snyk.yml?branch=main&label=Snyk&labelColor=000000&style=flat-square)](https://github.com/ryanduguid/australian-accounting/actions/workflows/snyk.yml)
[![SonarCloud](https://img.shields.io/sonar/quality_gate/ryanduguid_australian-accounting?server=https%3A%2F%2Fsonarcloud.io&label=SonarCloud&labelColor=000000&style=flat-square)](https://sonarcloud.io/summary/new_code?id=ryanduguid_australian-accounting)
[![Codacy code quality](https://img.shields.io/codacy/grade/657574ca064f48499b4839693bafd4bf?label=Codacy&labelColor=000000&style=flat-square)](https://app.codacy.com/gh/ryanduguid/australian-accounting/dashboard)<br>
[![OpenSSF Scorecard](https://img.shields.io/ossf-scorecard/github.com/ryanduguid/australian-accounting?label=Scorecard&labelColor=000000&style=flat-square)](https://scorecard.dev/viewer/?uri=github.com/ryanduguid/australian-accounting)
[![OpenSSF Best Practices](https://img.shields.io/cii/level/15429?label=Best%20Practices&labelColor=000000&style=flat-square)](https://www.bestpractices.dev/projects/15429)
[![SLSA 3](https://img.shields.io/badge/SLSA-level%203-990024?labelColor=000000&style=flat-square)](https://github.com/ryanduguid/australian-accounting/attestations)
[![VirusTotal](https://img.shields.io/badge/VirusTotal-scanned-990024?labelColor=000000&style=flat-square&logo=virustotal&logoColor=FFFFF0)](https://github.com/ryanduguid/australian-accounting/releases)<br>
[![licence: Apache-2.0](https://img.shields.io/badge/licence-Apache--2.0-66023C?labelColor=000000&style=flat-square)](LICENSE)
[![Open in GitHub Codespaces](https://github.com/codespaces/badge.svg)](https://codespaces.new/ryanduguid/australian-accounting)
[![Glama MCP server score](https://glama.ai/mcp/servers/ryanduguid/australian-accounting/badges/score.svg)](https://glama.ai/mcp/servers/ryanduguid/australian-accounting)

Seven independently released Python engines that apply Australian tax and payroll
rules to figures you supply, plus **Aus Accounting MCP**, a local Model Context
Protocol server that hands those engines to an AI assistant. It serves Australian
accountants and bookkeepers, and the developers who build tooling for them.

Each tool returns a calculation, the facts it used and the matters it refuses to
decide. The engines and the MCP server run locally: they read files you point them at, contact no
service, write to no ledger and lodge nothing. Two boundaries sit outside that: an MCP host sends every tool argument and result to its model provider, and the optional LodgeiT adapter below makes bounded HTTP calls only when you enable it. Missing facts stay unknown: a Payday Super row with a fund-receipt date but no matched_amount or remitted_amount evidences when the fund received something, not how much, so the checker leaves it UNKNOWN rather than ON_TIME and names the amount it needs (from payday-super-checker 0.1.7; 0.1.6 and earlier read that receipt as covering the whole SG amount and say so in a caveat). Supply the amount the fund received with every receipt date. A person reviews every output before it is relied on.

**Start here:** [what each component is](#components) ·
[MCP client setup](apps/aus-accounting-mcp/README.md#client-integration) ·
[tool reference](apps/aus-accounting-mcp/README.md#tools) ·
[scope, limits and examples](https://duguid.com.au/tools/australian-tax-ai-agents/)

Ryan Duguid is not a registered tax agent or BAS agent. Project support is limited to software issues reproduced with fabricated data. Do not send taxpayer information or request advice, return preparation, tax-treatment confirmation, or lodgement.

Synthetic examples. Accounting decisions stay with the reviewer.

**Input:** $120 super contribution for payday 6 August 2026, remitted 14 August, with no fund receipt evidence as at 20 August.

## Try one contribution

With [uv](https://docs.astral.sh/uv/) installed, [download the synthetic CSV](https://duguid.com.au/assets/examples/payday/timely_remittance_no_receipt.csv) as `timely_remittance_no_receipt.csv`. Open a terminal in that folder and run:

```bash
uvx --from payday-super-checker==0.1.7 payday-super-check timely_remittance_no_receipt.csv --as-at 2026-08-20 --confirm-remittance-only -o payday-report.csv
```

This downloads the pinned PyPI package without cloning the repository. It writes `payday-report.csv` in the current folder, replacing that file if it exists. Open the report to see the due date and verdict.

The confirmation acknowledges the missing receipt; it does not establish payment. Omit `--confirm-remittance-only` to see the same `AT_RISK` result with exit 2, which asks you to resolve or acknowledge that evidence gap.

**Output:** `AT_RISK`, due 17 August 2026; exit 0 acknowledges the remittance-only evidence, without proving timely receipt.

| Evidence | Result | Human decision |
| --- | --- | --- |
| Remitted before the deadline; receipt missing | `AT_RISK` | Obtain fund receipt evidence before concluding it was on time. |
| Fund receipt recorded on 17 August for the full $120 contribution | `ON_TIME` on supplied facts | Confirm the evidence and any applicable timing exception. |

[Read the 5-minute Payday Super case](packages/payday-super-checker/docs/manager-case-study.md) · [Division 7A repayment case](packages/div7a-loan-review/docs/manager-case-study.md) · [MCP setup](apps/aus-accounting-mcp/README.md#client-integration)

The [recorded public evaluation](https://duguid.com.au/evaluate/payday-super-evidence/) uses release 0.1.3. Follow its fixed revision and commands to reproduce that historical run; the quick trial above uses release 0.1.7, which requires an evidenced amount alongside a fund-receipt date.

## Choose the next example

| Your task | Start here |
| --- | --- |
| BAS pack review | [Two synthetic packs with expected findings](https://duguid.com.au/evaluate/manager-review-gate/) |
| Xero Activity Statement transaction GST review | [BAS workflow and transaction procedure](https://github.com/ryanduguid/australian-accounting-skills/blob/main/docs/bas-walkthrough.md#transaction-gst-review-in-codex-chatgpt-or-copilot) |
| Month-end close | [Balanced trial balance with unresolved exceptions](https://duguid.com.au/tools/monthly-close-controls/#worked-example) |
| GST planning | [Browser planning calculators](https://duguid.com.au/tools/business-calculators/) |
| Division 7A | [Repayment assertions and the review boundary](packages/div7a-loan-review/docs/manager-case-study.md) |
| Other engines and agent workflows | [Examples and reproduction routes](https://duguid.com.au/evaluate/#example-routes) |

For transaction GST coding, load the linked `bas-preparation` skill and its
bundled transaction reference. They require row coverage and separate evidence
for supply treatment, buyer credits and BAS attribution. Aus Accounting MCP does
not provide a transaction GST classifier; its calculations or a control-account
tie-out cannot establish correct tax coding. Missing facts remain unresolved for
the authorised reviewer. The walkthrough explains instruction-file delivery to
Codex, ChatGPT and Copilot without claiming equivalent tested behaviour.

<details>
<summary>Installation, component identities, integration and reference</summary>

Development home for the Aus Accounting MCP application and 7 independently released
Australian accounting engines. Each component keeps its own distribution name, version,
lockfile, tests, release notes, commands and licence. There is no root distribution,
shared runtime library or combined version.

## Use Aus Accounting MCP

For the MCP server, install Python 3.14+ and [uv](https://docs.astral.sh/uv/), then
configure your MCP client to run:

```bash
uvx aus-accounting-mcp
```

This starts a **local stdio server**; it waits for an MCP client rather than opening
a web page. It requires no API key. Package installation downloads dependencies;
the tools then use bundled data without network requests or record writes. The
MCP host you connect it to is a separate boundary: it sends every tool argument
and result to its model provider, so use fabricated data with a hosted model.

`uvx` keeps using the version it first downloaded, so a new release does not reach
you on its own. To move to a release, name it in your client configuration, for
example `aus-accounting-mcp==<version>` from [PyPI](https://pypi.org/project/aus-accounting-mcp/);
uvx downloads it, and later launches reuse the cached copy without a download until
the uv cache is cleaned.

![Static terminal demonstration of synthetic BAS output and Division 7A loan review](https://raw.githubusercontent.com/ryanduguid/australian-accounting/main/apps/aus-accounting-mcp/docs/quick-proof.webp)

Once it is connected, ask your assistant in plain English. Each of these fabricated
requests resolves to one tool call:

| Ask | Tool |
|---|---|
| "Compare a hairdresser with $180,000 sales, no other income, $60,000 wages, $30,000 rent and $40,000 other expenses to the ATO benchmarks." | `get_ato_benchmarks` |
| "Super for a 6 August 2026 payday was $120 and was sent on 14 August. As at 20 August, is it on time?" | `calc_payday_super_deadline` |
| "What is the Division 7A benchmark interest rate for 2026-27?" | `get_div7a_benchmark_rate` |
| "An employee who gave a TFN and claims the tax-free threshold (scale 2) is paid a regular $1,000 weekly wage in 2026-27, with no offsets, loans, bonuses or other adjustments. How much PAYG should be withheld?" | `calculate_tax_worksheet` |

[Client setup and examples](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/README.md#client-integration)
· [Tool reference](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/README.md#tools)
· [PyPI](https://pypi.org/project/aus-accounting-mcp/)
· [Glama listing](https://glama.ai/mcp/servers/ryanduguid/australian-accounting)

The tools list ATO benchmark industries, compare supplied expense buckets,
review Payday Super timing, look up reviewed Division 7A rates, review an
operator-supplied Division 7A loan, refuse unsupported Division 7A matters, and
generate synthetic CTR/BAS fixtures. They also assess related Payday contributions,
calculate 14 bounded tax and super worksheets and retrieve cited local Markdown excerpts.
Division 7A review is limited to the delegated
engine's s 109N/s 109E scope; it does not form amalgamated loans or classify
repayments under s 109R. Outputs are review aids, not advice or lodgements.

For an editable source installation, run `pip install -e .` from
`apps/aus-accounting-mcp/`. The repository root is not an installable Python package.

## Components

| Path | Distribution | Import package | Commands |
|---|---|---|---|
| `packages/australian-tax-calculators/` | `australian-tax-calculators` (Rust) | none | `australian-tax-calculators` (JSON stdin worksheets) |
| `apps/aus-accounting-mcp/` | `aus-accounting-mcp` | `aus_accounting_mcp` | `aus-accounting-mcp`, `aus-accounting-mcp-demo` |
| `packages/ato-benchmark-compare/` | `ato-benchmark-compare` (Rust) | none | `ato-benchmark-compare` |
| `packages/payday-super-checker/` | `payday-super-checker` (Rust) | none | `payday-super-check` |
| `packages/div7a-loan-review/` | `div7a-loan-review` (Rust) | none | `div7a-loan-review` |
| `packages/the-exchequer-tally/` | `the-exchequer-tally` (Rust) | none | `the-exchequer-tally` |
| `packages/solomons-sword/` | `solomons-sword` (Rust) | none | `solomons-sword` |
| `packages/the-wip-tally/` | `the-wip-tally` (Rust) | none | `wip-tally` |
| `packages/gst-review/` | `gst-review` | `gstreview` | Python review functions: GST coding states against registration, BAS-to-ledger reconciliation |
| `packages/lodgment-calendar/` | `lodgment-calendar` (not yet released) | `lodgmentcalendar` | Python function: ATO lodgment program due dates for BAS, IAS and income tax returns, `UNKNOWN` where a fact is missing |
| `apps/lodgeit-calculator-adapter/` | `lodgeit-calculator-adapter` | `lodgeitadapter` | optional adapter for a third-party calculator service; off by default, makes HTTP calls only when `LODGEIT_ADAPTER_ENABLED=1` |
| `packages/au-tax-rates-data/` | `au-tax-rates-data` (never released) | `rates` (module) | `python rates.py validate`, `snapshot` and `check`: ATO rates and thresholds with verbatim source quotes, validated offline |

`IMPORTS.md` records the source repository, commit and tree of every imported component. The
MCP application is the [io.github.ryanduguid/aus-accounting MCP Registry server](https://registry.modelcontextprotocol.io/v0.1/servers?search=io.github.ryanduguid%2Faus-accounting&version=latest); it
depends on the published `ato-benchmark-compare`, `payday-super-checker`,
`div7a-loan-review`, `australian-tax-calculators` and `gst-review` distributions. In
the development workspace, uv resolves `gst-review` to its checked-out source and
takes `australian-tax-calculators` 0.2.0, `div7a-loan-review` 0.3.2,
`ato-benchmark-compare` 0.2.2 and `payday-super-checker` 0.2.2, the last Python
releases, from PyPI.

The Exchequer Tally 1.0.0, Solomon's Sword 1.0.0, The WIP Tally 1.0.0,
Australian Tax Calculators 1.0.0 and Division 7A Loan Review 1.0.0 are
unreleased standalone Rust binaries. Build each with Cargo using its README
([The Exchequer Tally](packages/the-exchequer-tally/README.md),
[Solomon's Sword](packages/solomons-sword/README.md),
[The WIP Tally](packages/the-wip-tally/README.md),
[Australian Tax Calculators](packages/australian-tax-calculators/README.md),
[Division 7A Loan Review](packages/div7a-loan-review/README.md)). The
uv workspace continues to develop the other engines and applications; the MCP
server uses none of these crates yet.

## Working in a component

Change into the component directory and use the commands its own `README.md`,
`AGENTS.md` or `CONTRIBUTING.md` documents. The root `CONTRIBUTING.md` routes the
common commands and names the root workflow that runs them.

To skip local setup, [open the repository in GitHub Codespaces](https://codespaces.new/ryanduguid/australian-accounting).
The devcontainer installs uv and runs `uv sync --locked`, so the fabricated examples
above run as written. Codespaces usage counts against your own GitHub quota.

## Boundaries

- The MCP application depends on engines only through their published distributions.
- Engines never import `aus_accounting_mcp` or another engine, and no production module
  uses a relative import that leaves its package; `tests/test_boundaries.py` proves both.
- Only the workflows under the root `.github/workflows/` are active; component directories
  carry none.
- No client data, credentials or generated client reports enter this repository.

## Releases

Each component is released on its own namespaced tag `<component>/vX.Y.Z` by its own root
workflow `.github/workflows/release-<component>.yml`, which calls the pinned Release Policy
reusable workflow for that component directory only. `CONTRIBUTING.md` has the table.
The rates dataset in `packages/au-tax-rates-data/` is never released.

Each component's `LICENSE` applies to that component, and
`packages/ato-benchmark-compare/NOTICE` and `packages/au-tax-rates-data/NOTICE` cover their
ATO material. Outputs are review aids, not advice.

</details>
