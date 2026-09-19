# v0.1.8

- Withhold, do not nil: the compare command now gates its text output, `--json` payload and exit code on which buckets the mapping actually supplied, using the same presence rules as the exported library API, so a bucket no account was mapped to reads `not supplied`/`not_supplied` instead of a computed zero.
- Revert the key ratio to the ATO's published one where cost of sales was not supplied, so the total-expenses fallback no longer rests on a figure nobody established, and do not exit "outside the key range" on a withheld ratio.
- Add `--confirm-other-income-nil` for the operator step behind the turnover-basis gate.
- `route()` records which buckets received at least one reviewed account (`supplied_buckets`); `to_dict()` remains the raw serialiser without presence.

# v0.1.7

- Require non-empty band identities and actual boolean flags in datasets.
- Remove negative zero from numeric display helpers.
- Correct optional-W1 behaviour and remove invented provenance dates from examples.
