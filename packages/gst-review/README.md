# GST review

Distribution `gst-review`, import package `gstreview`. Library only: this component
ships no command. Not yet published.

Review aids for Australian GST coding over bank lines, for bookkeepers and accountants
checking a coded bank feed before a BAS. The engines read the figures you supply,
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

## Follow-up

Not yet wired into Aus Accounting MCP: the application pins published engine
distributions, so this needs a first release of `gst-review` and a release workflow.
