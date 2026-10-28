# Lodgment calendar

`lodgment-calendar` works out lodgement and payment due dates for BAS, IAS and income tax
returns under the ATO's 2026–27 lodgment program, for self-lodgers and for clients of
registered tax and BAS agents. Each date carries its rule, the ATO page it comes from and
the date that page was checked. A date that depends on a fact you did not supply, or that
the ATO pages leave open, comes back `UNKNOWN` with the missing condition named.

Distribution `lodgment-calendar`, import package `lodgmentcalendar`. Library only: this
component ships no command.

**Package lifecycle:** published. Install `lodgment-calendar` from PyPI with
`pip install lodgment-calendar`; see [release notes](RELEASE_NOTES.md) and
[DISCLAIMER.md](DISCLAIMER.md).

It is a preparation aid. It reads the facts you pass, contacts no service, calls no ATO
API and lodges nothing. Ryan Duguid is not a registered tax agent or BAS agent. Project
support is limited to software issues reproduced with fabricated data. Do not send
taxpayer information.

## Example

```python
from lodgmentcalendar import Facts, due_dates

result = due_dates(
    "bas", "company", "2026-27", "bas_agent", reporting_cycle="quarterly",
    facts=Facts(consolidated_head_company=False, lodges_electronically=True,
                previous_statement_lodged_electronically=True,
                includes_monthly_payg_obligation=False),
)
for row in result["obligations"]:
    print(row["period"], row["status"], row["nominal_due_date"], row["due_date"], row["rule_id"])
```

```
2026-27 Q1 DETERMINED 2026-11-25 2026-11-25 AS-Q-AGENT
2026-27 Q2 DETERMINED 2027-02-28 2027-03-02 AS-Q-28
2026-27 Q3 DETERMINED 2027-05-26 2027-05-26 AS-Q-AGENT
2026-27 Q4 UNKNOWN 2027-08-25 None AS-Q-AGENT
```

Quarter 2 is due on Sunday 28 February 2027. Monday 1 March is Labour Day in Western
Australia, which moves the date for every state, so the statement is due on 2 March.
Quarter 4 is `UNKNOWN` because the ATO marks 25 August 2027 'to be confirmed'.

## Inputs

`due_dates(obligation, entity_type, financial_year, channel, *, reporting_cycle=None, facts=None, extra_holidays=None, holidays_complete_until=None)`

| Input | Values |
| --- | --- |
| `obligation` | `bas`, `ias` or `income_tax_return` |
| `entity_type` | `individual`, `trust`, `partnership`, `company` or `smsf` |
| `financial_year` | The year of the statement periods, or the income year a return covers, as `2026-27` |
| `channel` | `self`, `tax_agent` or `bas_agent` (a BAS agent cannot lodge an income tax return) |
| `reporting_cycle` | `monthly` or `quarterly`, for `bas` and `ias` only. A monthly `bas` reports GST monthly; other monthly activity statements are `ias`. For a consolidated head company, a quarterly `ias` is its PAYG instalment activity statement |
| `facts` | A `Facts` with the conditions below; each is `True`, `False` or `None` (not supplied) |
| `extra_holidays` | Up to 50 `date: name` holidays the ATO table does not list (see below) |
| `holidays_complete_until` | A `date` to which the ATO table and `extra_holidays` hold every holiday (see below) |

This release records statement periods in 2026-27 and income tax returns for the 2025-26
income year, which are the returns the 2026–27 program covers. Any other year returns one
`UNKNOWN` row.

The `Facts` fields are the conditions the ATO pages make the dates depend on:
`prior_returns_outstanding`, `overdue_returns_lodged_by_31_october`,
`on_agent_client_list_by_cutoff`, `large_or_medium`, `member_large_or_medium`,
`new_registrant`, `prior_year_return_taxable`, `current_year_return_taxable`,
`latest_tax_liability_20k_or_more`, `total_income_over_2m`,
`prior_year_non_taxable_or_refund`, `current_year_non_taxable_or_refund`,
`consolidated_head_company`, `first_consolidated_return`, `lodges_electronically`,
`previous_statement_lodged_electronically`, `includes_monthly_payg_obligation`,
`annual_turnover_up_to_10m` and
`deferred_gst_registered`. `FACT_MEANINGS` in `lodgmentcalendar.due_dates` gives the
ATO's wording for each. Only the facts on a date's decision path are read; a `None` on
that path makes the date `UNKNOWN`.

Invalid inputs raise `ValueError`: an unknown value, a malformed year, a cycle on an
income tax return, a BAS agent lodging an income tax return or a fact that is not
`True`, `False` or `None`.

## Output

The result holds `obligations`, `counts` by status, `request`, `facts_used`,
`holiday_calendar`, `not_decided`, `warnings` and `source_checked`. Each obligation row has:

| Field | Meaning |
| --- | --- |
| `kind` | `lodgment`, `payment` (only where the payment date differs) or `concession` |
| `status` | `DETERMINED` or `UNKNOWN` |
| `due_date` | The date after the next-business-day roll; `null` when `UNKNOWN` |
| `nominal_due_date` | The date the rule names, before the roll, where the rule names one |
| `rule_id`, `summary` | The rule applied, in plain English |
| `source_url`, `source_provision`, `source_checked` | The ATO page, the table or heading on it, and the review date |
| `holiday_roll` | The weekend days and holidays passed over, with the holiday table's URL |
| `missing` | Each fact not supplied or question the ATO pages leave open |
| `payment` | On lodgement rows: `with_lodgment`, `separate`, `none` or `unknown` |

## Weekends and public holidays

The holiday calendar is the ATO's own: Tables 1 and 2 of
[Lodgment and payment dates on weekends or public holidays](https://www.ato.gov.au/tax-and-super-professionals/for-tax-professionals/prepare-and-lodge/registered-agent-lodgment-program/lodgment-and-payment-dates-on-weekends-or-public-holidays),
checked on 20 October 2026 and again on 28 October 2026. Under that page, a date that
falls on a Saturday, a Sunday or a public holiday for the whole of any state or territory
moves to the next business day for taxpayers in every state. The tables cover 1 July 2026 to 30 June 2027. A later date
that is not a weekend cannot be shown to be a business day, so it is `UNKNOWN` with its
nominal date kept. A holiday missing from the tables can only move a date later.

To settle those dates, pass the missing holidays from an official source as
`extra_holidays` and the last date your list is complete to as
`holidays_complete_until`. Holidays alone never extend coverage, because a list holding
one 2027 holiday is not a list that has 2027 covered. Each one must be a weekday the ATO
table does not already list, and shows in `holiday_roll` and `holiday_calendar` marked
'(caller-supplied)'.

```python
from datetime import date

result = due_dates("bas", "trust", "2026-27", "self", reporting_cycle="quarterly",
                   facts=Facts(lodges_electronically=True),
                   holidays_complete_until=date(2027, 8, 31))
print(result["obligations"][3]["due_date"])
```

```
2027-08-11
```

This assumes no whole-of-state holiday falls between 1 July and 31 August 2027 beyond
those you pass; the calendar takes that statement as given.

## Scope

Modelled rules, open questions and every source are listed in the
[primary-source review](docs/primary-source-review-2026-10-20.md). The calendar does not
model substituted accounting periods, companies that are not full self-assessment
taxpayers, APRA-regulated funds, annual GST returns, instalment notices, FBT or other
obligation types. It does not check whether the facts you pass are true, apply a
client-specific date the ATO has set, or decide whether anything was lodged or paid on
time.

## Licence

Licensed under the Functional Source License, Version 1.1, ALv2 Future License (FSL-1.1-ALv2). You may use, modify and redistribute it for any purpose except a Competing Use as defined in LICENSE (making it available to others in a competing commercial product or service), and each version becomes available under Apache-2.0 two years after it is made available. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
