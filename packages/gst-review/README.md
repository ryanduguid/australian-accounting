# GST review

Distribution `gst-review`, import package `gstreview`. Library only: this component
ships no command. Not yet published.

Review aids for Australian GST over bank and ledger lines, for bookkeepers and
accountants checking a coded bank feed before a BAS and reconciling a lodged BAS after it. The engines read the figures you supply,
contact no service, write to no ledger and lodge nothing. They are experimental review
aids, not advice or return preparation.

Ryan Duguid is not a registered tax agent or BAS agent. Project support is limited to
software issues reproduced with fabricated data. Do not send taxpayer information.

## Coding review

`review_coding` gives every bank line one review state and the reason for it:

| State | Meaning |
| --- | --- |
| `unknown_gst_code` | The GST code is not in the code table, so its treatment is unknown. |
| `gst_differs_from_registration` | The code charges or claims GST, but the entity is not registered on that date, or the purchase is from a supplier recorded as not registered (GST Act ss 9-5 and 11-5). |
| `between_entities` | An opposite amount sits in another supplied entity's lines within the match window: likely a transfer, not income or expense. |
| `coded` | Coding confirmed. |
| `suggested` | Coding proposed but not confirmed. |
| `nothing_suggested` | No account or GST code proposed. |

States are checked in that order, so a registration conflict shows even on a
confirmed line. Lines pair one to one across entities, nearest date first, within
`match_window_days` (default 3, at most 31).

A negative amount is money out. The GST code on the line, or else the suggested code,
is read through a code table. The default covers common Xero and MYOB AU codes; pass
`gst_codes` to use your own, mapping each code to `taxable`, `gst_free`,
`input_taxed` or `outside_gst`.

```python
from datetime import date
from decimal import Decimal

from gstreview import BankLine, Registration, review_coding

result = review_coding(
    [
        BankLine("1", "Alder Pty Ltd", date(2026, 8, 3), Decimal("-110.00"),
                 account="Repairs", gst_code="GST", confirmed=True,
                 supplier_registered=False),
        BankLine("2", "Alder Pty Ltd", date(2026, 8, 4), Decimal("-5000.00")),
        BankLine("3", "Birch Trust", date(2026, 8, 5), Decimal("5000.00")),
        BankLine("4", "Birch Trust", date(2026, 8, 6), Decimal("330.00"),
                 suggested_account="Rent", suggested_gst_code="GST"),
    ],
    {"Alder Pty Ltd": Registration(date(2024, 7, 1)), "Birch Trust": Registration(None)},
)
for row in result["lines"]:
    print(row["id"], row["state"])
```

```
1 gst_differs_from_registration
2 between_entities
3 between_entities
4 gst_differs_from_registration
```

The result also carries `counts` by state, `facts_used` (registration dates, match
window and code table), `not_decided`, `sources` and `source_checked`.

The review does not decide whether an entity was required to be registered
(s 23-5), whether a supply is taxable, GST-free or input taxed on its merits, what a
transfer between entities is, or whether a suggestion is right. Refunds, reversals and
creditable importations are not distinguished from other lines.

## BAS reconciliation

`reconcile_bas` works out G1, G2, G3, G10, G11, 1A and 1B from ledger lines for each
tax period and compares them with the lodged figures. Each label gets a status:
`matches`, `differs` (with the difference and the lines behind the ledger figure),
`not_reported` (no lodged figure for that label) or `negative_ledger` (credit notes
exceed sales or purchases; a BAS takes no negative figures).

- **Attribution.** On the cash basis a line falls in the period of its payment date.
  On accruals it falls in the period of the earlier of its invoice and payment dates
  (GST Act ss 29-5 and 29-10). A period's `timing` list names lines invoiced in it but
  attributed elsewhere, or still unpaid on the cash basis.
- **Rounding.** Ledger totals are rounded down to whole dollars before comparison, as
  the ATO's BAS instructions require. Lodged figures must be whole non-negative dollars.
- **GST-inclusive or not.** G labels include GST unless `g1_includes_gst=False`; the
  ATO requires every other label to follow the choice made at G1.
- **Codes.** Each line has a side (`sale` or `purchase`) and a GST code, read through
  a code table into a category: `taxable`, `capital`, `gst_free`,
  `capital_gst_free`, `export`, `input_taxed` or `excluded`. The default covers common
  Xero and MYOB AU codes; pass `bas_codes` to use your own. Excluded lines and lines
  that cannot be placed (an unknown code, a capital code on a sale, GST on a GST-free
  code) are listed in `kept_out`, and the second two in `line_issues` with a reason.

The result also lists periods with ledger lines but no lodged figures (`not_lodged`),
lodged periods with no ledger lines (`no_ledger_lines`), lines outside every period
and unpaid lines.

```python
from datetime import date
from decimal import Decimal as D

from gstreview import LedgerLine, Period, reconcile_bas

lines = [
    LedgerLine("inv-101", "sale", D("1100.00"), D("100.00"), "GST", date(2026, 7, 14),
               date(2026, 7, 30)),
    LedgerLine("inv-102", "sale", D("550.00"), D("50.00"), "GST", date(2026, 9, 25),
               date(2026, 10, 6)),
    LedgerLine("bill-7", "purchase", D("330.00"), D("30.00"), "GST", date(2026, 8, 2),
               date(2026, 8, 2)),
]
result = reconcile_bas(
    lines,
    [Period("2026-Q1", date(2026, 7, 1), date(2026, 9, 30))],
    {"2026-Q1": {"G1": D(1650), "1A": D(150), "1B": D(30)}},
    basis="cash",
)
for label, row in result["periods"][0]["labels"].items():
    if row["status"] != "not_reported":
        print(label, row["status"], row["difference"], row["lines"])
print("timing", result["periods"][0]["timing"])
```

```
G1 differs -550 ['inv-101']
1A differs -50 ['inv-101']
1B matches None []
timing ['inv-102']
```

The lodged figures here include invoice `inv-102`, which was paid in the next quarter.
On the cash basis it belongs to that quarter, so G1 and 1A differ and the period's
`timing` list names it.

It does not decide whether a difference needs a revised BAS, whether any code is right,
or which basis applies (s 29-40). Part payments on the cash basis need one line per
payment. Labels other than those above, adjustments and GST on imports are out of scope.

## Follow-up

Not yet wired into Aus Accounting MCP: the application pins published engine
distributions, so this needs a first release of `gst-review` and a release workflow.
