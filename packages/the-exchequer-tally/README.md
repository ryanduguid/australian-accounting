# Company tax and franking checks

The Exchequer Tally 1.0.0 is a standalone Rust binary. It replaces the Python
`the-exchequer-tally` 0.2.0 package and its `edwinnixon` import API.

## Build and check

Install Rust with rustup, then run from this directory:

```sh
cargo build --locked --release
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

The toolchain is pinned in `rust-toolchain.toml`. The executable is
`target/release/the-exchequer-tally` (`.exe` on Windows). Add that directory to
PATH or copy the executable to a directory on PATH. No Python runtime is needed.
`--version` prints `the-exchequer-tally 1.0.0`. PyPI still provides the
historical Python release; `pip install` does not install this binary.

## Original commands

```sh
the-exchequer-tally bre-test --fy 2025 --turnover 4500000 --assessable 800000 --passive 120000
the-exchequer-tally dist-statement --entity "Synthetic Pty Ltd" --acn 123456789 --recipient "Synthetic shareholder" --payment-date 2026-09-01 --amount 10000 --tax-rate 0.25 --franking-pct 100
```

`--help` and each original command's `--help` preserve the Python help text.
Money flags accept non-negative AUD amounts up to $1 trillion, with at most 2
decimal places. `dist-statement` requires a positive distribution, an explicit
payment date and tax rate. Rates are fractions strictly between zero and one;
percentages are between zero and 100 inclusive.

Successful commands exit 0. Missing or invalid arguments and domain validation
errors exit 2. Invoking the binary without a command prints help and exits 1.
An unhandled Decimal calculation failure exits 1; its diagnostic adaptation is
listed in [CHANGELOG.md](CHANGELOG.md). Output uses UTF-8, with CRLF on Windows.

## Former Python API

Each API command reads one UTF-8 JSON object from stdin and prints one compact
JSON result followed by a newline. Decimal amounts, rates and percentages are
strings throughout, preserving Python Decimal values and scale. Dates use
canonical `YYYY-MM-DD` and must be real calendar dates. Financial years are
JSON integers within the signed 64-bit range. An unknown outcome is JSON
`null`. An invalid request exits 2, writes `error: <reason>` to stderr and
writes no stdout. Type and format errors name the field, except in
`validate-value`, which keeps the original CLI validator's messages.
`statement` refuses any field not listed for it, as the Python keyword call
did; the other commands ignore unlisted fields. `--api-help` lists these
commands.

| Command | Input fields | Result |
| --- | --- | --- |
| `corporate-tax` | `financial_year`, `aggregated_turnover`, `assessable_income`, `passive_income` | CorporateTaxRate fields |
| `company-tax-statement` | The 4 `corporate-tax` fields, `deductions` and optional losses, offsets, credits and `loss_carry_back` (see below) | Taxable income or tax loss, the calculation statement labels A to S, carried-forward and lost offsets, and the loss carry back tax offset |
| `bre-properties` | Same 4 fields | Passive percentage, turnover eligibility, BREPI eligibility and BRE status |
| `turnover-threshold` | `financial_year` | Decimal threshold string |
| `max-franking-rate` | `current_fy`, optional `prior_year_test` containing the 4 BRE fields | Decimal rate string; missing or wrong-year evidence is refused |
| `statement` | `entity_name`, `abn_or_acn`, `recipient_name`, `payment_date`, `total_distribution`, `franking_percentage`, `corporate_tax_rate` | Statement fields, total and gross assessable income |
| `distribution-event` | `event_date`, `recipient_name`, `distribution_amount`, `franking_credit`, optional `corporate_tax_rate` | Event fields, maximum credit, exact ratio and displayed percentage; measurement requires a rate |
| `benchmark` | `corporate_tax_rate`, optional `distributions` array and `private_company` boolean (required for multiple half-years) | Original-order distributions, benchmark percentage, compliance status, violations and period details when company type is supplied |
| `franking-account` | `financial_year`, optional `opening_balance`, `entries`, `records`, `append_entries` | Entries, totals, closing balance and deficit result |
| `validate-value` | `kind` (`money`, `percentage`, `rate` or `date`), `value` string | Original CLI validator's parsed value string |

For example, in a POSIX shell:

```sh
echo '{"financial_year":2027,"aggregated_turnover":"1000000","assessable_income":"500000","passive_income":"100000"}' | the-exchequer-tally corporate-tax
echo '{"financial_year":2027,"opening_balance":"0.00","records":[{"entry_date":"2026-09-01","entry_type":"PAYG_INSTALMENT","amount":"1000"},{"entry_date":"2027-03-01","entry_type":"FRANKED_DIST_PAID","amount":"3000"}]}' | the-exchequer-tally franking-account
```

In PowerShell use a single-quoted JSON string piped directly to the executable,
without `echo`. Non-ASCII input needs a UTF-8 pipe encoding.

Ledger `entries` and `append_entries` are direct entry objects: `entry_date`,
`entry_type`, decimal `amount`, `description`, optional `statutory_reference`
(omitted means an empty string; an explicit `null` is returned as `null`).
Direct entries may be zero. `records` invokes the recording methods: each object
has `entry_date`, `entry_type`, positive decimal `amount` and an optional
`description`; amounts are rounded half up to cents. A `TAX_REFUND` record also
requires `includes_r_and_d_offset: false`. True, null and omission are refused
before posting. Direct entries retain the original constructor's validation;
use records when classifying and posting refunds.

Entry types are `PAYG_INSTALMENT`, `COMPANY_TAX_PAYMENT`, `FRANKED_DIST_REC`,
`FDT_LIABILITY`, `FRANKED_DIST_PAID`, `TAX_REFUND` and `UNDER_FRANKING_DEBIT`.
Every calculation revalidates that all entries belong to the stated financial
year. `append_entries` exposes this original mutation check without a Python
object. Omitted opening balance stays unknown; an established nil balance is
the explicit string `"0.00"`.

`benchmark` applies the benchmark rule (s 203-25 ITAA 1997): each frankable
distribution is compared with the first in its franking period (s 203-30). A
private company's franking period is its income year (s 203-45); any other
company's are the first 6 months of the income year and the rest
(s 203-40(2)). Income years run from 1 July to 30 June. Substituted accounting
periods, income years other than 12 months (s 203-40(3) to (5)) and the
s 203-20 exemption for listed public companies are not modelled.
Set an event's `frankable` to `false` if it is unfrankable under s 202-45.
It must have zero franking credit and appears in the output without affecting
the benchmark. Omission treats the distribution as frankable. A frankable
distribution with zero credit can set a 0% benchmark.
Supply `private_company` if frankable distributions span more than one
half-year. When supplied, `franking_periods` lists periods with a benchmark
in date order, with their `start`, `end` and `benchmark_percentage`. The
result's `benchmark_percentage` is the percentage when there is one period
with a benchmark and `null` otherwise. `compliant` is `null` when there is
no benchmark.

## Company tax statement

`company-tax-statement` is a native addition, not part of the Python port. It
works a company's return from taxable income to the amount due or refundable
for 2020-21 to 2026-27 (`financial_year` 2021 to 2027), at the rate
`corporate-tax` gives for the year.

- Prior-year losses: `prior_year_losses` lists `{loss_year, amount, test}`, where
  `test` is the stated final outcome under ss 165-10 and 165-15 ITAA 1997:
  `continuity_of_ownership` (s 165-12, with s 165-15 not denying it),
  `same_business` (ss 165-13, 165-210), `similar_business` (s 165-211, loss years
  starting on or after 1 July 2015 only), `failed` (no part deductible) or
  `part_year` (refused: s 165-20). Each loss year appears once. The command never
  judges either test. Assessable income equal to deductions deducts no loss. Losses go oldest first (s 36-17(7)); `net_exempt_income`
  absorbs them first; `loss_deduction_chosen` is the s 36-17(2) or (3) choice,
  and without it the most allowed is deducted. A tax loss is worked out under
  s 36-10.
- Calculation statement: offsets in the order of the Company tax return 2026
  instructions: `non_refundable_non_carry_forward_offsets` (C, of which
  `franking_tax_offsets`), `non_refundable_carry_forward_offsets` (D),
  `refundable_tax_offsets` (E), `franking_deficit_tax_offset` (F), then
  `section_102aam_interest` (G), `eligible_credits` (H) and `payg_instalments`
  (K). S = T5 + G - H - I - K; negative is refundable. Gross tax is rounded half
  up to the cent.
- Loss carry back (Div 160): `loss_carry_back` takes
  `franking_account_balance`, `carry_backs` (`loss_year`, `earlier_year`,
  `amount`), `earlier_years` (`year`, `income_tax_liability` and
  `liability_previously_used`, both required even when nil, and optional
  `net_exempt_income`), `loss_years` for loss
  years before the claim year (`year`, `tax_loss` and that year's 3 rate facts)
  and `conditions`, each stated `true`. Claim years 2020-21 to 2022-23 use
  Div 160 as in force to 30 June 2026 (conditions
  `corporate_tax_entity_throughout`, `returns_lodged`,
  `choice_in_approved_form`, `carried_on_business_each_loss_year`,
  `no_transferred_losses`, `no_excess_franking_offset_losses`,
  `no_integrity_scheme`); 2026-27 uses the Div 160
  that Treasury Laws Amendment (Tax Reform No. 2) Act 2026 substituted
  (`not_significant_global_entity` in place of
  `carried_on_business_each_loss_year`). 2023-24 to 2025-26 have no offset.
  The offset is a refundable tax offset at label E.
- Refused, each with its provision: consolidated or MEC groups, controlled
  foreign company or foreign hybrid attributable income, foreign residents,
  companies with other rates (RSA providers, PDFs, life insurance, non-profit,
  credit unions, trustees), a Subdiv 165-B year, deductions limited by s 26-55,
  shipping exempt income (ss 36-10(5), 36-17(4A)), franking tax offsets with a
  chosen prior-year loss deduction (s 36-17(5)) or in excess (s 36-55), carry-forward
  offsets with net exempt income (Div 65), and any carry back outside its
  version's years, conditions or limits.

## Arithmetic and limits

Arithmetic follows Python's default Decimal context: 28 significant digits,
half even for intermediate operations, half up where the engine explicitly
rounds to cents, and round down for statement franking credits. Eligibility and
benchmark comparisons use the original unrounded operations. Missing facts
remain unknown. Rate and turnover tables cover only FY2018 through FY2027;
this port does not update or extend law data.

The JSON boundary accepts at most 8 MiB. Accounting decimal fields accept at
most 8,192 UTF-8 bytes each and absolute exponent/adjusted exponent up to 10,000.
These resource ceilings replace Python's machine-dependent allocation limits.
`validate-value` intentionally applies the original CLI validator grammar to
its raw string, including large exponents; it performs no accounting arithmetic
and is exempt from the accounting decimal-field ceilings. The 8 MiB JSON limit
still applies. JSON nesting follows serde_json's recursion limit.

## Integration and releases

The MCP application consumes 6 other Python engines and has never imported
this engine. Its current tools and pins remain unchanged. When a Rust MCP server
later exposes this engine, it can launch the relevant JSON subcommand with stdin
and consume stdout, using the process exit status to distinguish refusal.

Root `just test` and `just check` include Cargo. CI builds and tests on Ubuntu,
Windows and macOS with read-only repository permissions; CodeQL includes Rust and
Dependabot monitors this Cargo lock. CI retains native candidates. The former
Python/PyPI publisher is retired. `.github/workflows/release-the-exchequer-tally.yml`
builds and publishes attested x86-64 Linux (static musl) and Windows
executables through `release-policy` on GitHub Releases.

The frozen Python corpus in `tests/data/` is replayed by Rust on all 3 platforms.
[tests/BEHAVIOURS.md](tests/BEHAVIOURS.md) maps the original tests to these checks.
The archive-based differential harness and qualification receipts are retained
under the Agent Hub task `rust-port-2026-10-11`, outside this repository.

<sub>Review aid, not tax, legal or financial advice. Ryan Duguid is not a registered tax agent or BAS agent. Support covers software issues reproduced with fabricated data: do not send taxpayer information or ask for advice, return preparation, tax-treatment confirmation or lodgement. See [DISCLAIMER.md](DISCLAIMER.md).</sub>
