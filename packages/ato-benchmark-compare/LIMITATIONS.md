# Known limitations

Behaviour this tool ships with that produces a figure you should not read at
face value. Each entry names the condition that triggers it, what it affects,
**what stays correct**, and the step that settles it.

For scope the tool never claimed, see [DISCLAIMER.md](DISCLAIMER.md).

Applies to ato-benchmark-compare 0.1.8.

## ABC-1 A bucket no account was mapped to is withheld, not read as a nil

**Trigger.** A bucket a ratio needs is absent from the supplied totals. Where
that bucket is `cost_of_sales` and the business type's key ratio is
`cost_of_sales_to_turnover`, the effect also reaches which ratio is marked key.

This is **not** the same as a profit-and-loss account with no mapping entry.
`route()` raises `MappingError` listing the first 20 unmapped rows and a count
of any beyond that, and the command line exits with an error before any
comparison happens, so that input cannot reach either serialiser. The trigger
here is a bucket that no account was mapped *to*: the mapping is complete, the
bucket simply has nothing in it and so is absent from the supplied-field set
the outputs are gated on.

**Effect.** An absent bucket reaches the raw comparison as exactly the same nil
that a genuine zero produces. The ATO's own fallback to
`total_expenses_to_turnover` triggers on that nil, so the tool would otherwise
mark a key ratio chosen by a figure nobody established.

**Unreleased: `compare` now withholds the verdict as well.** `compute` records
which buckets the supplied totals actually held, and `compare` reads that record
where the caller passes no `supplied_fields`, so a ratio built on an omitted
bucket carries status `not_supplied` instead of `within`, `below` or `above`, and
`outside_key_range` is false on a withheld key ratio for a direct library caller
too. What is left of the divergence is narrower, and it sits in the raw
serialiser and the raw key-ratio field:

- `to_dict()` still prints the computed figure for a withheld row, so that
  payload reads `"value": "0.0000"` beside `"status": "not_supplied"`. It is the
  raw serialiser and it holds no presence record of its own.
- `Comparison.key_ratio` still carries the ATO's nil-triggered fallback.
  `to_evidenced_dict` and `render_text` revert it to the published key ratio
  where `cost_of_sales` was not supplied; a caller reading the field off the
  comparison does not get that revert, and the withheld status on the fallback
  row is what stops the choice reaching a verdict.

**Since 0.1.8 every command-line output withholds what was not supplied.** The
`route()` step now records which buckets received at least one reviewed
account, `compare` passes that set into the same presence gate the exported
library API uses, and the text output, the `--json` payload and the exit code
all honour it:

- A ratio whose buckets were not supplied prints as `not supplied`, with
  status `not_supplied` in JSON, not a computed `0.0000`.
- `key_ratio` reverts to the business type's published key ratio where
  `cost_of_sales` was not supplied, with `is_key_ratio` reverting alongside it,
  so the ATO fallback no longer rests on a nil nobody established.
- The exit code no longer reports "outside the key range" on a ratio the gate
  withheld; the notes say which buckets were omitted and why.
- The turnover basis gate applies too: with no other-income account mapped and
  no `--confirm-other-income-nil` assertion, every ratio is `not_supplied`,
  matching the library rule that the ATO turnover basis is not established.

`to_dict()` remains exported as the raw serialiser with no presence record, so
a caller passing incomplete figures directly to it still receives computed
figures, now beside the withheld status. The command line no longer uses it. In 0.1.7 the divergence was in the
CLI itself: the command line emitted a figure while the library withheld it,
which is the finding this entry originally recorded. Files from that release
keep that behaviour.

**What stays correct.** The comparison behind every output. Each serialiser
renders one `Comparison` object built from one mapping, so turnover, the
turnover basis and every computed figure are the same on both sides.

Row-level parity needs both conditions, and the CLI now applies them:

1. The ratio's own buckets are supplied. `total_expenses_to_turnover` needs
   every expense bucket, `labour_to_turnover` needs the labour set (and
   `associated_persons` once `w1` is given), and the rest need their own field.
2. **Both income fields are supplied.** The gate holds every row back without
   `turnover` and `other_income` together.

Where both hold, the row carries the same value in every output against the
same published range.

**Operator step.** Map an account to every bucket the ratios you intend to read
require, including a deliberate nil where the business genuinely has none, and
pass `--confirm-other-income-nil` where the business has no other income at all.
Where a business genuinely has no cost of sales, the mapped nil keeps the ATO's
own fallback and the result stands.

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
