# Known limitations

Behaviour this tool ships with that produces a figure you should not read at
face value. Each entry names the condition that triggers it, what it affects,
**what stays correct**, and the step that settles it.

For scope the tool never claimed, see [DISCLAIMER.md](DISCLAIMER.md).

Applies to ato-benchmark-compare 0.1.7.

## ABC-1 An unmapped cost of sales changes which ratio is marked key

**Trigger.** The business type's key ratio is `cost_of_sales_to_turnover`, and
no account was mapped to the `cost_of_sales` bucket.

**Effect.** An omitted bucket reaches the comparison as exactly the same nil
that a genuine zero cost of sales produces. The ATO's own fallback to
`total_expenses_to_turnover` triggers on that nil, so the tool would otherwise
mark a key ratio chosen by a figure nobody established.

The two serialisers resolve this differently, on purpose, and can therefore
report a different `key_ratio` for the same comparison:

| Output | Behaviour when `cost_of_sales` is unmapped |
| --- | --- |
| `to_dict()`, used by the CLI and `--json` | Carries the ATO fallback. `key_ratio` is `total_expenses_to_turnover` and the note `cost_of_sales_key_fallback` is present. |
| `to_evidenced_dict()`, the exported library API | Reverts to the business type's published key ratio. `is_key_ratio` reverts with it, so the payload stays internally consistent. |

`to_dict()` receives no record of which buckets were supplied, so it cannot
tell an unmapped bucket from a nil one. `to_evidenced_dict()` takes
`supplied_fields` and can.

**What stays correct.** The underlying calculations and published ranges are
the same in both serialisers. The exposed payload is not: `to_evidenced_dict()`
suppresses ratios whose inputs were not supplied, emitting `null` figures and a
`not_supplied` verdict. With this unmapped input, total-expense and labour
outputs are therefore also suppressed there, while `to_dict()` retains their
calculated figures and verdicts. The key-ratio flag differs as described above.

**Where it surfaces at runtime.** The `cost_of_sales_key_fallback` note appears
in the CLI text and JSON output whenever the fallback is applied. The note does
not distinguish an unmapped bucket from a genuine nil, because the CLI cannot.

**Operator step.** Map an account to `cost_of_sales`, including a deliberate
nil where the business genuinely has none, before reading which ratio is key.
Where a business has no cost of sales, the fallback is the ATO's own behaviour
and the result is correct as it stands.

**Status.** Open. Passing the supplied-field set into `to_dict()` would let the
CLI make the same distinction the library API makes. Until then, treat the
library payload as authoritative on which ratio is key.

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
