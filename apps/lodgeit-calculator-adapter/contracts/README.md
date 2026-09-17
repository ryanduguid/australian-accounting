# Contract snapshots

What a provider's surface looked like when a person read it. A snapshot is
reference data: it records a reading, it is not an agreement, and LodgeiT Labs
has not seen or endorsed any of it.

## What a snapshot holds

| Field | Means |
| --- | --- |
| `snapshot_id` | The name this snapshot is cited by, in every evidence record |
| `read_at`, `read_by` | When it was read, and by what. An automated retrieval says so |
| `source_urls` | Where it was read from |
| `calculators` | Per calculator: route, supported periods, input schema ref, required and optional decimal fields, scope notes |
| `response_contract` | Whether a manifest and an advisory are required, and the fields the snapshot knows about |
| `notes` | What was odd, unresolved or worth knowing before calling |

## Changing one

A person reads the live surface, decides what changed, and edits the file. That
is the whole process, and it is deliberate.

`lodgeit-adapter drift` reports differences between the live catalogue and a
snapshot. It prints findings and exits non-zero when there are any. It does not
write anything, and there is no flag that makes it write anything. A contract
change that arrived while nobody was looking is exactly the change that should
not be accepted automatically.

When you do update one, move `read_at` and `read_by` with it. A stale reader
attribution on a fresh reading is worse than no attribution.

## What was read on 18 September 2026

Both snapshots here were read on that day, by automated retrieval reviewed in
session. No practitioner has reviewed either.

Four things worth carrying forward:

- **Money is asymmetric.** The provider returns decimal strings and says so in
  `llms.txt`, but its request schemas declare every money input as a JSON
  `number`. Requests here therefore carry JSON numbers, serialised exactly.
- **Two URN spellings are current.** `publish.html` documents
  `urn:sbrm:calc:{name}`; live discovery returns `urn:sbrm:calculator:{...}`.
  These snapshots record the live spelling, because it is what the routes
  accept.
- **Response shapes are unversioned.** The provider says so plainly. A snapshot
  is the only thing standing between a shape change and a wrong reading, which
  is why an unrecorded required field is a `CONTRACT_FAILURE`.
- **Fano authentication is unresolved.** `llms.txt` says open access with no
  key; the integration kit README says the current engine requires an
  `X-API-Key`. Nothing has been sent from this repository, so neither has been
  confirmed here.

Raw captures of the responses these snapshots were built from are kept outside
the repository, with the session worklog. They are third-party responses that
change without notice, and a stale copy committed here would read as a claim
about the present.
