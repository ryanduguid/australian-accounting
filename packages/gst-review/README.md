# GST Review

GST coding and BAS reconciliation review aids for bookkeepers and accountants.
The Rust `gst-review` command reads supplied facts, contacts no service, writes
no files and lodges nothing. Version 1.0.0 is unreleased. The former Python import
package `gstreview` is removed from this checkout. Aus Accounting MCP still uses
the published `gst-review==0.1.0` until its separate migration.

Ryan Duguid is not a registered tax agent or BAS agent. These are experimental
review aids, not advice or return preparation. Obtain human review before relying
on an output. Project support covers software issues reproduced with fabricated
data only. Do not send taxpayer information. See [DISCLAIMER.md](DISCLAIMER.md).

With Rust 1.99, build and run locally:

```sh
cargo build --locked --release
target/release/gst-review --api-help
target/release/gst-review review-coding < coding.json
target/release/gst-review reconcile-bas < bas.json
```

The root command also accepts `au-accounting gst-review review-coding` and
`au-accounting gst-review reconcile-bas`. Prebuilt binaries and a native publisher
are pending; the Cargo binstall metadata reserves the repository's archive layout.

## JSON boundary

Each command takes one JSON object on stdin, up to 8 MiB. Success prints one JSON
result on stdout and exits 0. Invalid inputs print `error: ...` on stderr and exit
2. An output stream failure exits 1. Help and version need no input. Output is
UTF-8 with LF line endings on every platform.

Amounts are decimal strings, parsed exactly with Python Decimal syntax. They must
be finite, have at most 2 decimal places and have magnitude at most `1e12`.
Trailing zeros count: `"1.000"` is refused. Strings are limited to 8,192 bytes per
amount. JSON numbers are refused for money. Calculations use a private 40-digit,
half-even context; the caller's precision is restored afterwards. Signed zero,
scale and scientific formatting are retained in reported amounts.

Dates are canonical `YYYY-MM-DD` strings in years 0001 to 9999. A date field cannot
contain a time. Optional dates may be absent or null. Objects represent the former
dataclasses; arrays retain input order. Flags are JSON booleans, with null allowed
only for `supplier_registered`. Unknown object fields are ignored, like the
existing native engine adapters. Code tables map strings to the categories below;
an explicit empty object replaces all defaults. Null or an omitted table uses the
defaults in [src/constants.json](src/constants.json).

This JSON mapping is new. The former library accepted Python `Decimal`, `date`
and dataclass instances. Python import and packaging identity checks do not apply
to the Rust command. Its reported `engine_version` is `1.0.0`; the frozen oracle
reports `0.1.0`. Other review results and validation messages are preserved.

## Coding review

```json
{
  "lines": [
    {"id":"out","entity":"Alder","date":"2026-08-04","amount":"-5000.00"},
    {"id":"in","entity":"Birch","date":"2026-08-05","amount":"5000.00"}
  ],
  "registrations": {
    "Alder":{"registered_from":"2024-07-01"},
    "Birch":{"registered_from":null}
  },
  "match_window_days":3
}
```

`lines` and `registrations` are required. Each line needs `id`, `entity`, `date`
and `amount`. Optional fields are `description` (default empty), `account`,
`gst_code`, `confirmed` (default false), `suggested_account`,
`suggested_gst_code` and `supplier_registered`. Other optional values default to
null. Each registration needs `registered_from`, which may be null for never
registered, and may supply `registered_to`. Both boundaries are inclusive.
Every line's entity needs registration facts.

Each line receives the first applicable state in this order:

| State | Meaning |
| --- | --- |
| `unknown_gst_code` | The supplied code has no known treatment. |
| `gst_differs_from_registration` | A taxable code conflicts with entity registration or the supplied status of an outgoing payment's supplier. |
| `between_entities` | An opposite amount in another supplied entity is a likely transfer. |
| `coded` | An account and GST code are confirmed. |
| `suggested` | Coding is proposed but unconfirmed. |
| `nothing_suggested` | No coding is proposed. |

Negative amounts are money out. The actual GST code takes precedence over a
suggested code, including an actual empty string. `gst_codes` maps codes to
`taxable`, `gst_free`, `input_taxed` or `outside_gst`. Defaults cover common Xero
and MYOB AU codes. A confirmed line needs an account and GST code.

Pairing is greedy in line order, one to one across entities, within
`match_window_days` (default 3, integer 0 to 31). It uses numeric Decimal equality,
prefers the nearest date and breaks equal-distance ties by candidate input order.
Zero amounts never pair. Pair discovery precedes state selection, so a registration
conflict can outrank a pair on one side. The example pairs `out` and `in`.

Results include each line's state, reason, counterpart and cited sections, counts
in state order, used registration facts, the code table, `not_decided`, sources,
source-check date and warnings. The review does not establish whether an entity
was required to register, the merits of a GST treatment, the nature of a transfer
or whether suggested coding is correct. Refunds, reversals and creditable
importations are not distinguished from other lines.

## BAS reconciliation

```json
{
  "basis":"cash",
  "lines":[
    {"id":"inv-101","side":"sale","gross":"1100.00","gst":"100.00",
     "gst_code":"GST","invoice_date":"2026-07-14","payment_date":"2026-07-30"},
    {"id":"inv-102","side":"sale","gross":"550.00","gst":"50.00",
     "gst_code":"GST","invoice_date":"2026-09-25","payment_date":"2026-10-06"}
  ],
  "periods":[{"label":"2026-Q1","start":"2026-07-01","end":"2026-09-30"}],
  "lodged":{"2026-Q1":{"G1":"1650","1A":"150"}}
}
```

`basis`, `lines`, `periods` and `lodged` are required. Each line needs `id`, `side`
(`sale` or `purchase`), `gross`, `gst`, `gst_code` and `invoice_date`.
`payment_date` is optional. Gross includes GST; both amounts are negative for a
credit note. GST must share the gross amount's sign and cannot exceed its magnitude.
Periods need unique non-empty labels, start and end dates. Their inclusive date
ranges cannot overlap. Results preserve supplied period order.

On `cash`, attribution follows payment. On `accruals`, it follows the earlier of
invoice and payment. Each period's `timing` lists lines invoiced in it but
attributed elsewhere, or unpaid on cash. Unpaid and outside-period lines are
listed before GST placement, so an unknown code on such a line does not refuse
the reconciliation.

`bas_codes` maps codes to `taxable`, `capital`, `gst_free`, `capital_gst_free`,
`export`, `input_taxed` or `excluded`. Excluded lines still count as attributed
and appear in `kept_out`. Any attributed line that cannot be placed, including
an unknown code, capital sale, export purchase or GST on a GST-free code, refuses
the entire reconciliation with ordered diagnostics.

G labels include GST unless `g1_includes_gst` is false; that choice applies to
every G label. Labels appear in order `G1,G2,G3,G10,G11,1A,1B`. Ledger totals
round towards zero to whole dollars before comparison. Lodged figures must be
whole non-negative dollar amounts. Each label reports:

| Status | Result |
| --- | --- |
| `matches` | Whole-dollar ledger and lodged amounts agree. |
| `differs` | Reports their difference and contributing line IDs. |
| `not_reported` | No lodged amount for this label. |
| `negative_ledger` | Ledger total is negative; reports contributing IDs without a comparison. |

The example reports G1 difference `-550`, 1A difference `-50` and timing line
`inv-102`. Results also list `not_lodged`, `no_ledger_lines`, `outside_periods`
and `unpaid`, facts, limitations, sources and warnings. A present lodged period
`{}` is distinct from an absent period.

The engine does not decide whether a difference needs a revised BAS, whether a
code is right or which basis applies. Cash part payments need one line per
payment. Other labels, adjustments and GST on imports are outside scope.

## Sources and checks

The Rust port retains the Python engine's GST Act and ATO links, limitations and
`source_checked` value `2026-10-19` from [src/constants.json](src/constants.json).
Port qualification checks implementation parity, not a fresh legal-source review.
Recheck current authoritative sources before relying on the software.

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked --release
```

The native suite contains every call from all 77 original behavioural cases,
full expected results and refusal messages, plus order, context, tie, malformed
input and physical command checks. Fixture provenance and adaptations are in
[tests/fixtures/README.md](tests/fixtures/README.md). No Python interpreter is
needed to run the native suite. See [RELEASE_NOTES.md](RELEASE_NOTES.md) and
[RELEASING.md](RELEASING.md).
