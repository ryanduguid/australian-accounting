# Company tax and franking checks

The Exchequer Tally 1.0.0 is a standalone Rust binary. It replaces the Python
`the-exchequer-tally` 0.2.0 package and its `edwinnixon` import API.

Ryan Duguid is not a registered tax agent or BAS agent. Project support is limited
to software issues reproduced with fabricated data. Do not send taxpayer
information or request advice, return preparation, tax-treatment confirmation or
lodgement. Outputs are review aids. See [DISCLAIMER.md](DISCLAIMER.md).

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
The current native version is unreleased. PyPI still provides the historical
Python release; `pip install` does not install this binary.

## Original commands

```sh
the-exchequer-tally bre-test --fy 2025 --turnover 4500000 --assessable 800000 --passive 120000
the-exchequer-tally dist-statement --entity "Synthetic Pty Ltd" --acn 123456789 --recipient "Synthetic shareholder" --payment-date 2026-09-01 --amount 10000 --tax-rate 0.25 --franking-pct 100
```

`--help` and each original command's `--help` preserve the Python help text.
Money flags accept non-negative AUD amounts up to $1 trillion, with at most two
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
canonical `YYYY-MM-DD`. Financial years are JSON integers within the signed
64-bit range. An unknown outcome is JSON `null`. An invalid request exits 2,
writes `error: <reason>` to stderr and writes no stdout; type and format errors
name the field. `--api-help` lists these commands.

| Command | Input fields | Result |
| --- | --- | --- |
| `corporate-tax` | `financial_year`, `aggregated_turnover`, `assessable_income`, `passive_income` | CorporateTaxRate fields |
| `bre-properties` | Same four fields | Passive percentage, turnover eligibility, BREPI eligibility and BRE status |
| `turnover-threshold` | `financial_year` | Decimal threshold string |
| `max-franking-rate` | `current_fy`, optional `prior_year_test` containing the four BRE fields | Decimal rate string; missing or wrong-year evidence is refused |
| `statement` | `entity_name`, `abn_or_acn`, `recipient_name`, `payment_date`, `total_distribution`, `franking_percentage`, `corporate_tax_rate` | Statement fields, total and gross assessable income |
| `distribution-event` | `event_date`, `recipient_name`, `distribution_amount`, `franking_credit`, optional `corporate_tax_rate` | Event fields, maximum credit, exact ratio and displayed percentage; measurement requires a rate |
| `benchmark` | `corporate_tax_rate`, optional `distributions` array of event objects | Original-order distributions, benchmark percentage, compliance status and violations |
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

The MCP application consumes four other Python engines and has never imported
this engine. Its current tools and pins remain unchanged. When a Rust MCP server
later exposes this engine, it can launch the relevant JSON subcommand with stdin
and consume stdout, using the process exit status to distinguish refusal.

Root `just test` and `just check` include Cargo. CI builds and tests on Ubuntu and
Windows with read-only repository permissions; CodeQL includes Rust and
Dependabot monitors this Cargo lock. CI retains native candidates. The former
Python/PyPI publisher is retired. Native signing, release-policy integration and
publication require a separate authorised release change; no publisher is
configured for 1.0.0.

The frozen Python corpus in `tests/data/` is replayed by Rust on both platforms.
[tests/BEHAVIOURS.md](tests/BEHAVIOURS.md) maps the original tests to these checks.
The archive-based differential harness and qualification receipts are retained
under the Agent Hub task `rust-port-2026-10-11`, outside this repository.
