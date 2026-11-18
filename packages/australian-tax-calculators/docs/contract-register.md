# Contract register

`contract-register` sets out many clients' finance contracts together. It
reads a contracts file on stdin, CSV with a header line and then one contract
a line, rather than a JSON object, and writes the register's reports as JSON,
text or CSV:

```sh
australian-tax-calculators contract-register --balance-date 2026-06-30 \
  --scope-confirmed --format text < examples/contract-register.csv
```

`--balance-date` (required) is the date the amounts outstanding are struck
at. `--expiring-by` closes the window for contracts expiring, 12 months after
the balance date unless given. `--format` is `json` (the default), `text` or
`csv`. `--scope-confirmed` confirms the scope of each contract's schedule
command for every contract, and `--refusal-json` works as it does for the
JSON commands. [examples/contract-register.csv](../examples/contract-register.csv)
is a fabricated register of 8 contracts.

| Column | Holds |
| --- | --- |
| `contract` | the contract's reference, unique in the file |
| `client`, `financier` | the names the reports total by |
| `kind` | `hire_purchase` |
| `start_date` | `YYYY-MM-DD` |
| `amount_financed`, `instalment`, `instalments`, `timing`, `balloon` | as `hire-purchase-schedule` takes them; `balloon` is `0.00` for none |
| `frequency` | optional, as `hire-purchase-schedule` takes it; monthly when empty |

The columns may come in any order. `hire-purchase-schedule` sets out each
contract on its line's facts, with `year` the income year its start falls
in, so a contract it would refuse, such as one starting before 1 July 2024,
is refused with that command's code and message, after the line number and
contract. The register changes no schedule: it dates and adds the payments
and interest each one gives. A line is refused, by its number, for a quote
out of place, more or fewer fields than the header, a contract named twice, a
kind the register does not set out, an empty cell its kind needs, a filled
cell its kind does not take, or a contract, client or financier name that
is empty, over 200 characters, holds a control character or begins with
`=`, `+`, `-` or `@`, which a spreadsheet reads as a formula. The header is
refused for an unknown, repeated or missing column. The file holds 1 to
5,000 contracts and at most 8 MiB.

The JSON gives `totals`, `expiring`, `by_financier`, `by_client`,
`cashflow_by_month`, `by_income_year` and `contracts`, each contract with
its figures at the balance date, its payments `by_month` and its
`by_income_year` rows, then `basis`, which states how each figure is worked:

- A payment counts on the day it is made: an instalment on its row's
  `payment_date`, a balloon on its row's `period_end`. A period's interest
  counts on its last day, the day before `period_end`.
- `outstanding` at the balance date is the amount financed plus the interest
  of the periods ended by then, less the payments made by then, which equals
  `remaining_payments` less `unexpired_interest`. The interest of the period
  running at the balance date is left out: `accruing_interest` is that
  period's interest, over `accruing_days`, of which `accruing_days_elapsed`
  fall on or before the balance date, for the preparer to accrue.
- `current` is `outstanding` less the amount outstanding at `current_to`, 12
  months after the balance date, or nil where that is below nil, and
  `non_current` is the rest. AASB 101 paragraph 69(c) makes a liability
  current when it is due to be settled within twelve months after the
  reporting period. The register does not assess paragraph 69(a), (b) or
  (d): a breached covenant that makes a loan payable on demand (paragraphs
  72B to 75) is for the preparer.
- By income year, interest counts in the year its period ends.
  `interest_across_30_june` is the part of it from a period spanning 30
  June, with that period's `days_before_1_july` and `days_from_1_july`:
  s 240-60(2) relates such a period's interest to each year in accordance
  with accounting principles, which the caller applies. `principal` is the
  payments less the interest, the fall in the amount outstanding over the
  year, below nil where interest builds up before the payments start.
- A contract starting after the balance date has no balance-date figures and
  adds nil to them; its payments still count by month and by income year.

The CSV is one table: a `report` column (`contract`, `contract_month`,
`contract_income_year`, `expiring`, `financier`, `client`, `month`,
`income_year` or `total`), then every key those rows give, empty where a row
has none. The text output sets the same reports out as tables. The same file
always gives the same output.

The register takes no deposit, trade-in, payout, GST, fees or irregular
payments, and no lease, chattel mortgage or loan. A further kind joins
`KINDS` in [`src/contract_register.rs`](../src/contract_register.rs) once a schedule command gives the
schedule rows `hire-purchase-schedule` gives. `tests/contract_register.rs`
holds the example register to each contract's own schedule; see
[calculation evidence](calculation-evidence.md#contract-register).
