# Known limitations

Behaviour this tool ships with that produces a figure you should not read at
face value. Each entry names the condition that triggers it, what it affects,
**what stays correct**, and the step that settles it.

For scope the tool never claimed, see [DISCLAIMER.md](DISCLAIMER.md).

Applies to ato-benchmark-compare 0.1.7.

## ABC-1 An unmapped bucket reads as a nil in one output and as withheld in the other

**Trigger.** A bucket a ratio needs is absent from the supplied totals. Where
that bucket is `cost_of_sales` and the business type's key ratio is
`cost_of_sales_to_turnover`, the effect also reaches which ratio is marked key.

This is **not** the same as a profit-and-loss account with no mapping entry.
`route()` raises `MappingError` listing the first 20 unmapped rows and a count
of any beyond that, and the command line exits with an error before any
comparison happens, so that input cannot reach either serialiser. The trigger
here is a bucket that no account was mapped *to*: the mapping is complete, the
bucket simply has nothing in it and so is absent from the supplied-field set
the library serialiser is given.

**Effect.** An absent bucket reaches the comparison as exactly the same nil
that a genuine zero produces. The ATO's own fallback to
`total_expenses_to_turnover` triggers on that nil, so the tool would otherwise
mark a key ratio chosen by a figure nobody established.

The two serialisers resolve this differently, on purpose, for the same
comparison:

| | `to_dict()`, used by the CLI and `--json` | `to_evidenced_dict()`, the exported library API |
| --- | --- | --- |
| A ratio whose buckets were not supplied | Emitted as a computed figure, typically `0.0000`, with no marker separating it from an established nil | Value `null`, status `not_supplied` |
| `key_ratio` when `cost_of_sales` is unmapped | Carries the ATO fallback to `total_expenses_to_turnover`; the note `cost_of_sales_key_fallback` is present | Reverts to the business type's published key ratio, with `is_key_ratio` reverting alongside it |

`to_dict()` receives no record of which buckets were supplied, so it cannot
tell an unmapped bucket from a nil one. `to_evidenced_dict()` takes
`supplied_fields` and can.

**What stays correct.** The comparison behind both. Each serialiser renders one
`Comparison` object built from one mapping, so turnover, the turnover basis and
every computed figure are the same on both sides.

Parity at the row level is narrower than that, and needs both conditions:

1. The ratio's own buckets are supplied. `total_expenses_to_turnover` needs
   every expense bucket, `labour_to_turnover` needs the labour set (and
   `associated_persons` once `w1` is given), and the rest need their own field.
2. **Both income fields are supplied.** `to_evidenced_dict()` gates every row
   on `income_evidenced`, which is `turnover` and `other_income` together.
   Without both, even a ratio holding all of its own buckets is emitted by the
   command line and withheld by the library.

Where both hold, the row carries the same value in both payloads against the
same published range.

Three things diverge independently, and running them together overstates the
effect:

| What | When it diverges |
| --- | --- |
| A ratio's **value** | Its own buckets are absent, or an income field is. The library withholds it as `not_supplied`; the command line emits a figure. |
| A ratio's **benchmark range** | Only when an income field is absent. With both income fields supplied the range is retained, even on a row whose value is withheld. |
| The **key ratio and its flag** | Only in the `cost_of_sales` fallback above. A missing non-key bucket leaves the key ratio and every key flag alone. |

Do not read the two payloads as interchangeable.

**Where it surfaces at runtime.** The `cost_of_sales_key_fallback` note appears
in the CLI text and JSON output whenever the fallback is applied. The note does
not distinguish an unmapped bucket from a genuine nil, because the CLI cannot.

**Operator step.** Map an account to every bucket the ratios you intend to read
require, including a deliberate nil where the business genuinely has none.
Where a figure must not be shown unless it was established, read the library
payload: a `not_supplied` status is the honest answer that a CLI `0.0000`
cannot give. Where a business genuinely has no cost of sales, the fallback is
the ATO's own behaviour and the result stands.

**Status.** Open. Passing the supplied-field set into `to_dict()` would let the
CLI make the same distinction the library API makes. Until then, treat the
library payload as authoritative on both which ratio is key and which figures
were established.

## Not limitations

Recorded here so they are not re-audited:

- **A repeated account name is refused, not guessed.** Where two ledger rows
  share a name and one mapping row answers for both, the second is reported as
  repeated and no amount is routed. Guessing the bucket is the silent error
  this tool exists to avoid.
- **An account in the income section is treated as income.** `suggest()`
  deliberately returns the weaker but correct `turnover` for an
  expense-sounding name sitting in the income section, rather than filing it as
  an expense.
- **Wages inside cost of sales get their own bucket.** The ATO takes salary and
  wages out of the cost of sales ratio, so `cost_of_sales_labour` is a correct
  narrow definition, not an approximation. It is published on the
  [tool page](https://duguid.com.au/tools/ato-benchmarks/).
