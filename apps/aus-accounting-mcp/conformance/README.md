# Conformance cases

`cases.json` holds tool calls with the results this server must return. The
cases are plain JSON, so another implementation can replay them: a different
MCP server, an agent tool, or a calculator adapter that claims the same
behaviour for Payday Super deadlines or Division 7A loans.

The cases pin what a reviewer would rely on, and most of all what the server
declines to conclude:

- a fund-receipt date without an amount is `UNKNOWN`, not `ON_TIME`;
- a payday before 1 July 2026 is refused, because quarterly SG law applies;
- a Division 7A year with no reviewed benchmark rate is `UNKNOWN`;
- a loan whose written agreement is not established gets no minimum yearly
  repayment.

## Format

Each case names a `tool` and its `arguments`. `expect` maps a
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

The inputs are fabricated and the expected values are this server's own
results; no external practitioner has reviewed them. A matching result
shows two implementations agree; it does not show either is right in law.
The rates and law dates come from the delegated engines; each result states
the `engine_version` and `law_content_date` it was produced under.
