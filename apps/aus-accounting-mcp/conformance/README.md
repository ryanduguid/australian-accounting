# Conformance cases

`cases.json` holds tool calls with the results this server must return. The
cases are plain JSON, so another implementation can replay them: a different
MCP server, an agent tool, or a calculator adapter that claims the same
behaviour for Payday Super deadlines, Division 7A loans or PAYG withholding.

## Basis

Each case states its `basis` and its `source`:

| Basis | Cases | Where the expected values come from |
| --- | --- | --- |
| `authority` | 15 | The ATO's Schedule 1 (NAT 1004) withholding amounts sample data, weekly, fortnightly and monthly tables published on 17 June 2026 |
| `self-pinned` | 8 | This server's own results, recorded when each case was written |

The authority cases call `calculate_tax_worksheet` with `kind: payg_withholding`,
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
re-checked against the publication from this repository alone. `tests/test_conformance_cases.py` fails if an
authority case's expected amount differs from that CSV.

The self-pinned cases are the Payday Super and Division 7A cases, including the
refusal. They pin what a reviewer would rely on, and most of all what the server
declines to conclude:

- a fund-receipt date without an amount is `UNKNOWN`, not `ON_TIME`;
- a payday before 1 July 2026 is refused, because quarterly SG law applies;
- a Division 7A year with no reviewed benchmark rate is `UNKNOWN`;
- a loan whose written agreement is not established gets no minimum yearly
  repayment.

## Format

Each case names a `tool`, its `basis`, its `source` and its `arguments`. `basis`
is `authority` or `self-pinned`, and `source` is a sentence naming where the
expected values come from; the test rejects a case without either. `expect` maps a
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
shows two implementations agree; it does not show either is right in law. An
authority case shows agreement with the ATO's published sample for that pay,
not that a payee's scale, allowances or declarations are right.
The rates and law dates come from the delegated engines; each result states
the `engine_version` it was produced under.
