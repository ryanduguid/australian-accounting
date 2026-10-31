# Native calendar candidate

The Rust engine provides the recorded 2026-27 lodgment program through JSON. It
preserves missing facts, source wording, separate payment rows, uncertain concessions
and incomplete holiday coverage. It contacts no service and performs no lodgment.
Confirm each client's dates with the ATO and obtain human review.

This is an unreleased 1.0.0 candidate. Python 0.1.0 remains available for the existing
MCP consumer until its native replacement is qualified. The native command and library
do not start Python. Published compatibility records and Python release routing still
describe the Python interface. The native MCP candidate links this crate. Rust owns the
rules from MCP cutover; until then the Python source stays for the supported release and
rollback, and the oracle below binds this crate to its results. `to_ics` is Python only.

## Build and call

From this directory:

```sh
cargo build --locked --release
./target/release/lodgment-calendar due-dates < request.json
```

The request is one JSON object, for example:

```json
{
  "obligation": "bas",
  "entity_type": "company",
  "financial_year": "2026-27",
  "channel": "bas_agent",
  "reporting_cycle": "quarterly",
  "facts": {
    "consolidated_head_company": false,
    "lodges_electronically": true,
    "previous_statement_lodged_electronically": true,
    "includes_monthly_payg_obligation": false
  }
}
```

Use the obligation, entity, year, channel, cycle and 19 fact names documented in
[README.md](README.md). `facts` is an object of booleans or null; omitted facts and a
null or omitted `facts` object remain unknown. Unknown request or fact fields are
refused. Dates must be real dates written exactly as `YYYY-MM-DD`, within years
0001 to 9999. `extra_holidays` is an object mapping up to 50 date strings to printable
holiday names of at most 120 Unicode characters. `holidays_complete_until` is a date
string or null. Adding holidays alone does not extend coverage.

The command accepts at most 8 MiB of UTF-8 JSON on stdin and refuses duplicate object
keys. Success writes one result
to stdout and exits 0. Invalid input writes an error to stderr and exits 2. Help,
version and API help are available through `--help`, `--version` and `--api-help`.

Rust callers use `lodgment_calendar::api::evaluate("due-dates", &request)`, which
returns `Result<serde_json::Value, String>`. `holidays::roll` provides the JSON
equivalent of the original holiday helper for callers that need its skipped days
and unresolved reason. It takes `day`, optional replacement `holidays` and optional
`covers_to`; the replacement mapping does not imply complete coverage.

The output retains every field documented in README.md. `engine_version` identifies
the native crate. Rule data and the source review date of 28 October 2026 match the
Python source; this port does not constitute a new review of ATO law or publications.

## Verification

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked --release
```

The original 150 Python tests passed before capture. Native tests compare 61 targeted
results or refusals, 13 additional refusal cases and SHA-256 digests of 8,636 complete results over the original
seeded fact samples and targeted fact/calendar combinations. Only the engine version
changes from 0.1.0 to 1.0.0. See [oracle provenance](tests/fixtures/README.md) for
representation adaptations and source bindings. Additional native checks cover JSON
types, field names, real dates, Unicode holiday names, input bounds and process output.

The Rust crate CI matrix includes this candidate. The existing Python checks continue to cover the current MCP dependency. CI builds
short-lived candidates on Linux, Windows and macOS; it publishes no release.
