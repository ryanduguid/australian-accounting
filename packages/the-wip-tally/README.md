# Construction WIP schedule

Ryan Duguid is not a registered tax agent or BAS agent. Project support is limited to software issues reproduced with fabricated data. Do not send taxpayer information or request advice, return preparation, tax-treatment confirmation, or lodgement.

```
+----------------------------------------------------------------------+
|                      Construction WIP schedule                       |
+----------------------------------------------------------------------+
|        AASB 15 construction WIP schedule, from a contract CSV        |
+----------------------------------+-----------------------------------+
| DR  what it gives you            | CR  what it needs                 |
+----------------------------------+-----------------------------------+
| cost-to-cost earned revenue      | one row per unit of account       |
| contract asset vs liability      | cost to date, ETC, certified      |
| profit fade and stale ETC flags  | billings, variations, retentions  |
+----------------------------------+-----------------------------------+
```

<div align="center">

[![tests](https://img.shields.io/github/actions/workflow/status/ryanduguid/australian-accounting/ci.yml?branch=main&label=tests&labelColor=000000&style=flat)](https://github.com/ryanduguid/australian-accounting/actions/workflows/ci.yml)<br>
[![Rust](https://img.shields.io/badge/Rust-1.99-66023C?logo=rust&logoColor=E5E4E2&labelColor=000000&style=flat&color=66023C)](https://www.rust-lang.org/)
[![License: FSL-1.1-ALv2](https://img.shields.io/badge/License-FSL--1.1--ALv2-66023C?labelColor=000000&style=flat&color=66023C)](LICENSE)

</div>

Project name The WIP Tally. Cargo package `the-wip-tally`, command `wip-tally`.

**Deterministic work-in-progress schedule for Australian construction, civil, mining-services and power-station packages.** Review aid. Not a determination.

**Package lifecycle:** The WIP Tally 1.0.0 is an unreleased standalone Rust
binary. It replaces the Python `the-wip-tally` 0.2.0 package and its `wiptally`
import API. PyPI still provides the historical Python release; `pip install`
does not install this binary.

The [`wip-over-under-billing` skill](https://github.com/ryanduguid/australian-accounting-skills/blob/main/.claude/skills/wip-over-under-billing/SKILL.md) in Australian Accounting Skills (formerly `hardhat-ledger`) already encodes the WIP *workflow*. This engine does the arithmetic that workflow consumes: cost-to-cost progress after AASB 15 para B19 exclusions, constrained variable consideration, per-contract contract assets and contract liabilities, and month-on-month profit fade.

The `australian-accounting` repository contains the maintained source.

In-repo version: `1.0.0`; see [release notes](RELEASE_NOTES.md) and
[CHANGELOG.md](CHANGELOG.md).

Built by Ryan Duguid, a provisional member of Chartered Accountants ANZ. Written independently, in his own time and on his own equipment.

## Not advice

Nothing this engine produces is accounting, tax or legal advice, a journal, a
lodgement, or a conclusion that a contract transfers over time, that a claim is
enforceable, or that a contract is onerous. Those stay with a person. Confirm
the operative AASB 15 compilation at standards.aasb.gov.au before relying on a
paragraph number.

Full boundary statement: [DISCLAIMER.md](https://github.com/ryanduguid/australian-accounting/blob/main/packages/the-wip-tally/DISCLAIMER.md).

## Build

Install Rust with rustup, then run from this directory:

```sh
cargo build --locked --release
```

The toolchain is pinned in `rust-toolchain.toml`. The executable is
`target/release/wip-tally` (`.exe` on Windows). Add that directory to PATH or
copy the executable to a directory on PATH. No Python runtime is needed.

Download the [fabricated sample contract CSV](https://raw.githubusercontent.com/ryanduguid/australian-accounting/main/packages/the-wip-tally/examples/sample_contracts.csv)
as `sample_contracts.csv` to follow the example below.

## Use

```bash
wip-tally schedule sample_contracts.csv --as-at 2026-08-31
```

```
TheWIPTally: 5 contract(s), as at 2026-08-31

  Contract assets:      221,000.00
  Contract liabilities: 183,333.33
  Revenue to date:      3,087,666.67

  Assets and liabilities are per contract. They are not netted.

  Review flags on 4 contract(s):
    ...
```

The sample is designed to raise review flags. Exit code 2 means a person needs
to look; it is not a crash. Exit code 0 means no review flags, and still
requires practitioner sign-off. Exit code 1 is a data or file error.

Full detail goes to `wip-schedule.csv`. Then:

```bash
wip-tally review-pack wip-schedule.csv --source sample_contracts.csv --as-at 2026-08-31 -o practitioner-review.md
```

The pack binds itself to the source and schedule bytes with SHA-256, lists
every flagged contract, and puts a sign-off checklist on the page. Keep the
CSV beside the pack. The CSV is the row-level evidence.

The reporting date is written on every schedule row, so it is inside the bytes
the pack rebuilds and hashes. Without `--as-at`, `review-pack` uses the schedule's
recorded date, or today if no usable date is available. An explicit `--as-at`
takes precedence and must match the schedule's date; the pack refuses to bind
one period's numbers to another period's header.

### Options

| Option | What it does |
| --- | --- |
| `-o, --output` | Where to write the schedule CSV (default `wip-schedule.csv`) or the review pack |
| `--as-at DATE` | Reporting date as `YYYY-MM-DD`, written on every schedule row. `schedule` defaults to today; `review-pack` uses the schedule date, or today if unavailable. An explicit date overrides either default. |
| `--mapping-file FILE` | JSON map of canonical field names to your column headings |

## Input columns

Required: `contract_id`, `original_contract_sum`, `costs_incurred`,
`estimated_cost_to_complete`, `certified_billings`, and
`outcome_reasonably_measurable` on every row. Everything else is optional but
sharpens the answer. Amounts are exclusive of GST.

Every data row must carry one field per header column. Leave an optional field
empty rather than short. A row with the wrong field count is refused with its
row number, because its trailing columns would otherwise shift by one position
and read as absent.

A blank `outcome_reasonably_measurable` is refused by row number rather than
read as `yes`. Both readings state a revenue figure, and the answer to AASB 15
para 44 is a judgement about the contract, not a default this engine can pick.

| Field | Meaning |
| --- | --- |
| `contract_id` | One unit of account. Combine contracts under AASB 15 para 17 *before* they reach this file |
| `original_contract_sum` | Signed contract sum |
| `approved_variations` | Priced approved variations |
| `unapproved_variations_estimate` | Expected-value or most-likely claim *before* the constraint |
| `constraint_include_ratio` | Portion highly probable not to reverse, as `0.25` or `25%`. A bare `25` is refused |
| `costs_incurred` | Cost to date, including waste and uninstalled materials |
| `inefficiency_rework_wastage` | Para B19(a) amounts stripped from the progress measure, kept in margin |
| `uninstalled_materials` | Para B19(b) amounts recognised at cost (zero margin) |
| `estimated_cost_to_complete` | Current ETC. An unchanged ETC after material spend is flagged stale |
| `certified_billings` | Amounts certified, exclusive of GST |
| `uncertified_claims` | Claimed but not certified. Never treated as billing or as entitlement |
| `retention_withheld` | Retention on this contract |
| `retention_classification` | `receivable`, `conditional`, or `review` (the default) |
| `committed_outstanding` | Open POs and subcontracts still to invoice |
| `outcome_reasonably_measurable` | `yes` or `no`, per row. There is no default: a blank refuses the contract, because paras 44-45 decide between percentage-of-completion revenue and revenue limited to recoverable cost. `no` applies para 45 recoverable-cost-only revenue |
| `recoverable_costs` | Required in substance when the outcome is not measurable; defaults to progress cost, and is refused above `costs_incurred` or the transaction price |
| `progress_method` | `cost_to_cost` (default), `output`, or `right_to_invoice` |
| `output_percent` | Required for `output` when the outcome is measurable, as `0.40` or `40%` |
| `prior_*` | Prior-period transaction price, EAC, cost, ETC and revenue, for fade and period revenue |
| `gst_rate` | Default `0.10`. Used only to *show* GST on certified billings and retention |
| `assets_used_carrying` | If the job is underwater, flags AASB 137 para 69 (impair first) |

Dates are not required. This is a position schedule, not a cash-flow forecast.

A `--mapping-file` JSON object maps canonical names to your headings. Download
the [mapping example](https://raw.githubusercontent.com/ryanduguid/australian-accounting/main/packages/the-wip-tally/examples/mapping.example.json)
from the maintained source.

## The rules it applies

Paragraph numbers below are from the compilation current when this engine was
written. Confirm the operative compilation at standards.aasb.gov.au before
citing them. The source notes are in
[docs/aasb-15-source-notes-2026-08-25.md](https://github.com/ryanduguid/australian-accounting/blob/main/packages/the-wip-tally/docs/aasb-15-source-notes-2026-08-25.md).

**Transaction price.** Original sum plus approved variations plus the
constrained slice of unapproved estimates. The excluded slice is flagged, not
booked. Uncertified claims are never added to billings.

**Cost-to-cost progress.** `(costs incurred - B19 exclusions) / (EAC - B19
exclusions)`. Wasted cost stays in estimated cost at completion, so it hits
margin, but it does not accelerate per cent complete. Uninstalled materials are
recognised at cost (zero margin) and the remainder of the transaction price
follows the stripped ratio.

**Contract asset versus liability.** Revenue to date less certified billings, per
contract. Positive is a contract asset. Negative is a contract liability. The
engine never offsets one contract against another. Retention classification
under paras 105-108 is flagged for the engagement lead; it is not auto-posted
to receivables.

For a measurable `right_to_invoice` contract, certified billings are rounded to
cents, half up, before they become revenue. That same amount determines the
contract balance, so rounding cannot create an asset or liability. The GST
review note keeps using the supplied billing evidence, independently of progress
method. The supplied contract remains intact in the Rust result. Other
progress methods and recoverable-cost-only revenue keep their rounding order.

**Profit fade.** Current margin at completion versus prior-period margin, in
percentage points. A movement of 1.00 point or more is a review flag. An ETC
that has not moved after $1,000 or more of additional cost is flagged stale.

**Onerous contracts.** If EAC exceeds transaction price the row is flagged for
AASB 137. This engine does not measure the provision and does not post it.
Where `assets_used_carrying` is supplied it also flags para 69 (impair those
assets before raising a provision).

**GST.** Certified billings and retention are shown grossed-up at `gst_rate` as
a BAS review note. Australian progress claims commonly invoice GST on the full
certified amount, including retention that has not been paid in cash. This is
not a BAS engine.

**Tax.** Do not carry this progress measure into a tax computation. Confirm the
ATO position on long-term construction contracts at ato.gov.au for the year.

## What it does not do

- Decide over-time versus point in time (AASB 15 paras 35-38)
- Combine contracts (para 17)
- Decide whether a variation or claim is legally enforceable
- Measure an onerous-contract provision under AASB 137
- Prepare a Security of Payment payment claim
- Lodge a BAS, compute fuel tax credits, or run TPAR
- Replace the `wip-over-under-billing` skill, which still owns the workflow around this file

Use the [`wip-over-under-billing` skill](https://github.com/ryanduguid/australian-accounting-skills/blob/main/.claude/skills/wip-over-under-billing/SKILL.md)
in Australian Accounting Skills for the review steps. Use this engine for the numbers those steps consume.

## Local file boundary

This is a single-user command-line tool. Its input, mapping and output
arguments designate files the invoking operating-system account has chosen to
read or write; they are not a sandbox. Do not expose the command as a web
endpoint. Outputs must have an explicit `.csv` or `.md` filename and are
written atomically.

## Former Python API

`wip-tally measure` and `wip-tally parse-money` replace the removed
`wiptally.measure` and `wiptally.parse_money`. Each reads one UTF-8 JSON object
from stdin and prints one compact JSON result followed by a newline; a refusal
exits 2, writes `error: <reason>` to stderr and writes no stdout. Amounts are
decimal strings, preserving Python Decimal values and scale. `--api-help`
lists the commands.

| Command | Input fields | Result |
| --- | --- | --- |
| `measure` | Every `ContractInput` field: `contract_id`, `line_number` (integer), `customer`, `description`, the amount and ratio fields as decimal strings, `retention_classification`, `progress_method`, `outcome_reasonably_measurable` (`true`, `false` or `null`), and the optional amounts as strings or `null` | Every computed `ContractPosition` field, `flags` and `needs_review` |
| `parse-money` | `raw` string, optional `where` (default `amount`) | The cent-quantised amount as a string |

Each JSON input is limited to 8 MiB and each decimal field to 8,192 UTF-8 bytes.

`parse-money` refuses amounts that cannot be rounded to cents within the
28-digit decimal context, including a rounding carry beyond that precision.
The refusal retains `where`; CSV refusals name the row, contract and field and
are collected with other unreadable cells. These input errors produce no result.

## Job-to-cash example

`examples/job_to_cash.rs` replaces `examples/job_to_cash.py` and writes the same
files. See [examples/JOB-TO-CASH.md](examples/JOB-TO-CASH.md).

## Tests

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

The frozen Python corpus in `tests/data/` is replayed by Rust on both platforms,
including the original command runs with their input files and every file they
wrote. [tests/BEHAVIOURS.md](tests/BEHAVIOURS.md) maps the original tests to
these checks.

The suite pins the worked examples in `examples/sample_contracts.csv`: a clean
underbilling, B19 wastage with profit fade, a constrained mining-services
claim, uninstalled materials on an underwater power-station civil package, and
para 45 recoverable-cost-only revenue. Test data is synthetic. Never commit
client job-cost data to this repository.

## Disclaimer

This is an educational tool, not tax, legal or financial advice, and using it
creates no professional relationship. Outputs can be wrong, incomplete, stale
or unsuitable for a given set of facts. Check anything material against the
current standards and the entity's facts, and leave sign-off with a registered
practitioner.

Licensed under the Functional Source License, Version 1.1, ALv2 Future License (FSL-1.1-ALv2). You may use, modify and redistribute it for any purpose except a Competing Use as defined in LICENSE (making it available to others in a competing commercial product or service), and each version becomes available under Apache-2.0 two years after it is made available. See [LICENSE](LICENSE) and [NOTICE](NOTICE). If you redistribute this work or build on it, keep the copyright notices in [NOTICE](NOTICE).
