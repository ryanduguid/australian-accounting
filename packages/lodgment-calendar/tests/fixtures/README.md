# Original Python contract oracle

The original 150 pytest cases passed unchanged at base
`c940a25bb6a89d71be8c0d21ab853e3b92566445` before the native implementation.
The capture observed each actual engine/helper call and exception. All data is
fabricated. The Python runtime and tests remain in this component during the MCP
migration, so the original implementation is also directly inspectable.

`python-contracts.json` contains 65 targeted results or exact refusal messages:
38 `due-dates` cases and 27 direct holiday `roll` cases. Another 17,006 complete
successful results are represented by their canonical JSON SHA-256 digests, with
247 request profiles and 261 fact sets. This includes the original 60 seeded
random fact sets plus all-unknown facts across every request shape, both with the
ATO calendar and with a caller's calendar, and targeted selector tests.

For each matrix row, clone its indexed request, add its indexed `facts`, evaluate
`due-dates`, recursively sort all object keys, encode compact UTF-8 JSON and compare
its SHA-256 digest. Arrays retain their order. Nothing is removed from the result.
`refusals.json` adds 13 original Python errors for unusual quoting, controls,
compound inputs and numeric notation after independent review identified a gap.

Representation adaptations:

- Python `Facts` becomes a JSON object, dates become canonical date strings and
  keyword arguments become request fields. The native engine version is 1.0.0;
  the original result's 0.1.0 is the only output normalisation.
- Python datetime/date-key identity failures, the rejection of a dictionary in
  place of `Facts` and an object impersonating `bool` have no direct JSON
  equivalent. Their 6 excluded calls are recorded in `origin.json`. Native tests
  instead reject timestamps, malformed dates, non-object facts and every
  non-boolean fact type. Facts objects are the intended native representation.
- JSON rejects unknown field names and duplicate object keys. These new boundary
  checks prevent misspelt facts or ambiguous calendars from silently taking effect.

The 2026-10-28 refreeze replayed all 8,636 rows on the Python engine at 3f7815b.
Each reproduced its original digest with the old check date restored, so only
`source_checked` moved.

The 2026-10-31 refreeze replayed them on the Python engine at 035847a, which adds
FBT returns. Each reproduced its digest once that commit's 3 changes were undone:
the check date (2026-10-31), the FBT item in `not_decided` and the null
`on_fbt_client_list_by_21_may` in `facts_used`. Every fact set gained that null
fact. 8,370 rows were added from Python for 90 `fbt_return` requests (every entity
and channel; 2026-27 with 4 calendars, 2025-26 and 2027-28) over the 9 combinations
of its 2 facts and, for 2026-27, every earlier fact set with the new fact varied.
4 targeted FBT cases were added. Python's replies replaced 11 targeted results, the
obligation refusal and 13 refusals in `refusals.json`, whose messages now list
`fbt_return`. 28 targeted cases have no Python call (holiday `roll` and non-object
calendars) and were kept. `origin.json` keeps the first capture's records.

`origin.json` records the original tracked file hashes, capture counts, fixture
hashes and hashes of both compiled data files. `program-2026-27.json` copies the
original rule texts, sources, fact meanings and holiday table. `nonprintable.json`
records the ranges rejected by Python's Unicode 17.0.0 `str.isprintable`; it keeps
holiday validation and diagnostic escaping independent of the host Unicode version.
Neither data file is generated at build time.
