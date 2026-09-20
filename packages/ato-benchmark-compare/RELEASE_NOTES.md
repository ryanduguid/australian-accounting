# v0.1.9

- Refuse a neutral CSV whose header names `account`, `amount` or `section` more than once, instead of reading the first such column silently.
- Breaking for a direct library caller: `compare()` now withholds the verdict on a ratio built from a bucket the supplied totals did not hold. `compute()` records those buckets on `Figures.supplied_fields`, `compare()` reads that record when no `supplied_fields` argument is passed, and the affected verdicts carry status `not_supplied` instead of `within`, `below` or `above`, so `outside_key_range` is false on a withheld key ratio.
- `route()` returns its totals as `mapping.RoutedTotals`, a dict that carries the buckets a reviewed account reached, so routing a file and comparing it no longer treats the routing zero-fill as evidence. The compare command now passes its own evidenced set into `compare()` as well, which leaves its text, JSON and exit code unchanged.
- Migration: pass `Decimal("0")` for a bucket the operator established as nil, or pass `supplied_fields` to `compare()` to state the set yourself. The command line and the MCP adapter build totals across every bucket and are unchanged, as is any call that passes the argument explicitly.
- LIMITATIONS ABC-1 narrows accordingly: `to_dict()` still prints the computed figure beside the withheld status, and `Comparison.key_ratio` still carries the ATO's nil-triggered fallback that the evidenced serialisers revert.

# v0.1.8

- Withhold, do not nil: the compare command now gates its text output, `--json` payload and exit code on which buckets the mapping actually supplied, using the same presence rules as the exported library API, so a bucket no account was mapped to reads `not supplied`/`not_supplied` instead of a computed zero.
- Revert the key ratio to the ATO's published one where cost of sales was not supplied, so the total-expenses fallback no longer rests on a figure nobody established, and do not exit "outside the key range" on a withheld ratio.
- Add `--confirm-other-income-nil` for the operator step behind the turnover-basis gate.
- `route()` records which buckets received at least one reviewed account (`supplied_buckets`); a suggested-only mapping routes its amount but does not evidence its bucket, and `to_dict()` remains the raw serialiser without presence. The text output filters notes, checks and published ranges by the same presence rules as the JSON payload. The mapping reader now refuses a source value other than the canonical `reviewed` and `suggested` (case-insensitive), so a typo cannot evidence a bucket.

# v0.1.7

- Require non-empty band identities and actual boolean flags in datasets.
- Remove negative zero from numeric display helpers.
- Correct optional-W1 behaviour and remove invented provenance dates from examples.
