# Conformance cases

`cases.json` holds tool calls with the results this server must return. The
cases are plain JSON, so another implementation can replay them: a different
MCP server, an agent tool or a calculator adapter that claims the same
behaviour for Payday Super deadlines, Division 7A loans or PAYG withholding.

## Basis

Each case states its `basis` and its `source`:

| Basis | Cases | Where the expected values come from |
| --- | --- | --- |
| `authority` | 26 | 15 from the ATO's Schedule 1 (NAT 1004) withholding amounts sample data, weekly, fortnightly and monthly tables published on 17 June 2026; 8 from the ATO's Payday Super deadlines page or the Superannuation Guarantee (Administration) Act 1992; 3 from the ATO's Division 7A pages or s 109E of the Income Tax Assessment Act 1936 |
| `self-pinned` | 6 | This server's own results, recorded when each case was written |

The sample cases call `calculate_tax_worksheet` with `kind: payg_withholding`,
`year: 2026-27`, the sample row's pay period and earnings, and one scale. Each
expects the amount the sample gives for that scale, in dollars and cents. There is
one case for each of the 3 pay periods and 5 scales the sample publishes (1, 2, 3, 5
and 6). For scales 1, 2, 3, 5 and 6 respectively, the case uses the first, 13th,
25th, 37th and 48th of the period's 48 earnings points.
The sample is held in
`packages/australian-tax-calculators/tests/data/payg_withholding_sample_2026_27.csv`,
which australian-tax-calculators uses as its test oracle and whose provenance it
records in `docs/calculation-evidence.md`. The repository records the publication
and its date but no URL or checksum for the downloaded sample, so the CSV cannot be
re-checked against the publication from this repository alone. Each sample case's
`source` gives the CSV's SHA-256, and `tests/test_conformance_cases.py` fails if the
held CSV no longer has that digest or an expected amount differs from it.

The other authority cases cover the ATO's Payday Super Examples 1 to 4, a statutory
deadline, the 2026-27 Division 7A benchmark rate and 2 minimum yearly repayments.
Each `source` cites the ATO page or section, and each `worked` shows the figure
worked by hand. `tests/test_conformance_cases.py` works the `expect` figures again
without the server: it counts business days, derives the remaining term and applies
the s 109E(6) formula, and holds the ATO's published dates and rate, so changing a
figure in `expect` fails the test. The test needs `worked` to exist; it does not
read what the text says.

The 6 self-pinned cases pin what a reviewer would rely on, and most of all what the
server declines to conclude. Each `source` says why the case stays self-pinned and
names the authority for any part that has one:

- a fund-receipt date without an amount is `UNKNOWN`, not `ON_TIME`;
- a fund receipt with a matched amount is `ON_TIME` under this server's own
  qualification that returned contributions were not assessed;
- a payday before 1 July 2026 is refused, because quarterly SG law applies;
- a Division 7A year with no reviewed benchmark rate is `UNKNOWN`;
- a `COMPLYING` gate rests on a reading of s 109N(1)(b) that conflicts with the
  ATO's guidance and stays unresolved here;
- a loan whose written agreement is not established gets no minimum yearly
  repayment.

## Format

Each case names a `tool`, its `basis`, its `source` and its `arguments`. `basis`
is `authority` or `self-pinned`, and `source` is a sentence naming where the
expected values come from; the test rejects a case without either. `worked`, on an
authority case outside the sample, shows the figure worked by hand. `expect` maps a
[JSON Pointer](https://www.rfc-editor.org/rfc/rfc6901) into the tool's
structured result to the exact JSON value expected there. Money, rates and
dates are strings. Fields a case does not name are not part of it. A case with
`expect_error_contains` expects the call to fail with an error message that
contains that text.

## Run them here

```bash
uv run --locked --extra dev pytest tests/test_conformance_cases.py -q
```

Run from `apps/aus-accounting-mcp/`. The test sends every case through the
MCP tool-call path, the same one a client uses.

## Limits

The self-pinned cases have fabricated inputs and expect this server's own
results; no external practitioner has reviewed them. A matching result on one
shows 2 implementations agree; it does not show either is right in law. A sample
case shows agreement with the ATO's published sample for that pay, not that a
payee's scale, allowances or declarations are right. A worked case shows agreement
with the cited publication on its stated facts; words such as `UNPAID` and `MYR_MET`
are this server's names for the outcome, and no external practitioner has reviewed
the workings.
The rates and law dates come from the delegated engines; each result states
the `engine_version` it was produced under, and the Payday Super and Division 7A
results also state their `law_content_date`.
