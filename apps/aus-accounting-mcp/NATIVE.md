# Native server candidate

This unpublished Rust 1.0.0 candidate has passed local Windows and Linux runtime
qualification and independent review. The Python server remains the supported
runtime. Consumer cutover, distribution and release require further qualification.

The native facade links the existing Rust accounting engines. Statutory
calculations, data, warnings and source metadata remain owned by those engines.
Frozen discovery and effective input descriptors preserve the original MCP
boundary. They do not supply financial calculations or rate tables.

Run the fabricated proof from this directory with:

```sh
cargo run --locked --bin aus-accounting-mcp-demo
```

The demo prints both original calls and their complete structured results through
the server's validated tool dispatcher. It needs no Python, network, subprocess
or external data files. The BAS fixture is synthetic and is not a lodgment.
The Division 7A result retains every warning and limitation. This proof does not
verify current law or establish a real taxpayer's position. The stdio server
remains the default `cargo run` command and writes only protocol messages.

Native tests compare the full transcript with `docs/quick-proof.txt`, allowing
only the command header and the existing native engine version and table URI
differences below. A separate stdio test replays both calls and compares complete
results. The original Python proof and its derived image remain tied to the
supported Python distribution.

The short-lived CI candidate archive keeps the server, demo and evaluator beside
each other and includes `examples/group_review` (`.exe` on Windows). Extract it
outside the checkout, keep that layout and run these commands from its root:

```sh
./aus-accounting-mcp --version
./aus-accounting-mcp-demo
./aus-accounting-mcp-evaluate context
./aus-accounting-mcp-evaluate questions
./examples/group_review /approved/local/path/manifest.json
```

On Windows, append `.exe` to each executable name. Point an MCP client's stdio
command at the absolute server path without `--version`. Optional retrieval
folders still need their own local setup.

Component and engine licences, notices and disclaimers retain their repository
paths under `apps/` and `packages/` in the archive. `BUILD-INFO.json` records the
source commit, target, compiler, version and SHA-256 of every other bundled file.
This is an unsigned CI content record. It does not authenticate the producer,
provide a complete dependency licence inventory or replace release attestations.
CI extracts the finished archive and checks its contents and all 4 executables
with fabricated inputs, an empty PATH and an empty working directory before
retaining it. Hosted archive creation and macOS checks remain unverified until
CI runs. The supported Python release, Registry entry and cutover gates remain
unchanged.

The native supplementary evaluation tool preserves the full offline workflow:

```sh
cargo run --locked --bin aus-accounting-mcp-evaluate -- context
cargo run --locked --bin aus-accounting-mcp-evaluate -- questions
cargo run --locked --bin aus-accounting-mcp-evaluate -- score run.json --reference-sha256 SHA256
```

Build the server alongside the evaluator before running `context`. That command
initialises the sibling `aus-accounting-mcp` executable through MCP and prints
its complete instructions, tool definitions and actual scope resource. It
inherits configured retrieval folders and closes its owned child on success,
failure or timeout. The server must be beside the evaluator after installation.
The embedded reference lets `questions` and `score` run without the checkout,
Python, a server or network access. Use `--reference PATH` before the command to
evaluate a separate UTF-8 XML reference; the digest always identifies the exact
reference bytes.

Scoring never executes recorded calls. It preserves integer/decimal/boolean
types, exact decimal values, call order and repetitions, complete arguments,
surrounding answer whitespace rules and the legacy tools-only warning. Duplicate
keys, non-finite numbers, malformed references and digest mismatches are refused
before any scores print. An imperfect valid recording exits 0: this remains a
supplementary measurement requiring human interpretation, rather than a model
performance gate. Native parser diagnostics may differ from Python. Custom
references must be UTF-8; lone surrogate JSON strings, integers beyond 4,300
digits and JSON nesting beyond 1,000 containers are refused. JSON parsing also
has a 1 GiB cumulative scanning budget, counting each nested raw value, to bound
work on large values wrapped in many containers. Intermediate raw values borrow
the input rather than copying every subtree. Custom XML DTDs are
refused before expansion to bound entity amplification. Standard XML escapes and
numeric character references remain supported; external entities are never fetched.

Native installation checks run through a copied stdio server outside the checkout,
with an empty working directory and PATH:

```sh
cargo test --locked --test native_installation -- --nocapture
```

The worksheet check exercises all 14 kinds and all 26 advertised periods, checks
the catalogue against the frozen original and verifies warnings, calculation
identities and all 6 linked engine versions. It runs in the existing all-target
Rust CI job. This is synthetic distribution evidence, not tax validation.

`tests/native_relations.rs` also checks relationships against current sources:
served boundary paragraphs remain in `DISCLAIMER.md`, scope and guidance name the
registered tools and resources, prompt prerequisites match discovery, shared site
and icons match `server.json`, worksheet required inputs match discovered variants,
resource provenance follows linked tables and input bounds admit every shipped
benchmark name/year and engine date format. These complement frozen payload
comparisons; neither a captured call nor this set proves every original assertion.

The source-bound [behaviour record](tests/fixtures/native/behaviour-coverage.json)
reconciles all 433 original test functions across 36 files. It records 324 with
native observable checks, 59 with explicit native adaptations and 50 retained
with the supported Python distribution and its release, Registry, documentation
or media workflow. Retained checks are active and are not retirement approvals.
The record identifies proof symbols and limitations for each function; a Rust
test refuses changed original source hashes until the reconciliation is reviewed.
These are function dispositions, not 433 independent Rust tests or proof of
untested inputs, current law or real taxpayer results.

`tests/original_behaviours.rs` tests relationships that frozen payloads cannot
establish: the exact 46 money paths, published grammar and Draft 2020-12 inputs,
model bounds, pension day limits, receipt rounding against the actual CSV,
related-payday alignment, current GIC provenance, conformance cases, labour
warning prerequisites, Division 7A response sizes, current lodgment facts and
linked versions. Retrieval tests use valid external link targets and an invalid
manifest to test refusal and parse ordering. The stdio tests execute the current
39 evaluation questions with fabricated files, derive every reference answer and
check result-dependent arguments. Another session tests all 24 unknown-argument
refusals, partial receipts, input errors, all 11 rejected evidence-pack argument
variants and recovery, checking for working-directory writes after every reply.

The same test target has an explicit integration check for the native corpus
builder. Build an expressly selected builder checkout with its `fake-http`
feature into a separate target directory, then run:

```sh
FADDEN_SETUP_BINARY=/absolute/path/to/fadden cargo test --locked --test native_installation native_builder_fabricated_page_reaches_registered_mcp_citation_tools -- --ignored --nocapture
```

On Windows, set `$env:FADDEN_SETUP_BINARY` to the absolute executable path before
the Cargo command. This check requires the real builder and is ignored by default
because this repository does not contain that independent component. It copies
both binaries, requires an observed scripted HTTP refusal before supplying the
fabricated HTML, builds a new immutable corpus run and checks registered MCP
search/read against the exact paragraph, source hash, date, URL and withdrawal
observation. Child processes and owned temporary output are cleaned on success
or failure. Set `FADDEN_SETUP_OUT` to an absolute, absent directory with an existing
parent to retain the immutable run and `setup-report.json` for inspection. A
selected output directory is retained even on failure. The check establishes
neither live capture nor current law. See
[the corpus connection guide](docs/corpus-setup.md) for the build command and limits.

## Retained Python tooling

These are explicit remaining migration decisions, not retirement approval.
Runtime parity alone does not justify removing the supported Python package.

| Tool or dependency | Current disposition |
| --- | --- |
| `aus_accounting_mcp/demo.py` | Native command implemented above. Retain the Python command and its proof/media checks whilst Python remains supported. |
| `evaluation/tool_selection.py` | All 3 native commands implemented above. Retain the original as the supported Python workflow and migration oracle until distribution and consumer cutover are approved. |
| `examples/group_review.py` | A native Cargo example preserves independent reports, metadata, byte digests and scope without an aggregate verdict. See [the group review guide](examples/GROUP-REVIEW.md) for its command, encodings and limits. Retain the supported Python example until its workflow is retired explicitly. |
| `scripts/check_installed_worksheets.py` | The native installation test above covers every worksheet kind and period and linked engine identities. Retain this script's Python wheel/pin checks whilst Python remains supported; they describe a different distribution. |
| `scripts/check_corpus_setup.py` | The native installation test above feeds fabricated HTML through the real native builder and registered MCP retrieval, with optional retained run files and setup report. Retain the Python helper for compatible Python builder checkouts whilst that workflow remains supported. |
| `scripts/registry_preflight.py` | Retain for the supported PyPI/Registry release. Native distribution and provenance checks need an approved release design before replacement. |
| `scripts/render_demo_image.py` | Retain as optional development media tooling. Pillow renders the existing checked transcript; it is not a native server runtime dependency. A maintained Rust renderer is not required to run either native command. |
| `scripts/publish-github-about.sh` | Retain the small native shell helper. Running it changes external metadata and requires separate authority. |
| Python adapters, fixtures, tests and exact engine pins | Retain for the supported Python runtime and as the migration oracle until assertion coverage, packaging and consumer cutover are resolved. |
| Root uv overrides, development tools and Dependabot ignores | Retain whilst the supported Python checks need them. Remove only with the corresponding runtime/tooling retirement. |

The package's `compatibility.json`, `server.json`, Python version and external
consumer pins still describe the supported distribution. Updating or publishing
them is a separate release step.

## Scope of this candidate

The contract is the Python server at the oracle base recorded in
`data/catalogue.v1.json`: 25 tools, including `get_data_freshness`, `validate_abn`,
`validate_acn` and `get_reviewed_rate`, 6 resources
and 3 prompts. `aus-accounting-mcp doctor` prints the Python command's report. `get_data_freshness` reports the provenance of the linked Rust
tables and takes its 31 frozen replies from that server.

The engines also ship the worksheet kinds `fbt_car_statutory` and
`fbt_car_operating_cost`. The Python server does not advertise them, so there is
no oracle for them and adding them later is additive. The frozen input schema
refuses them, and the scope resource lists only the kinds in its discriminator
mapping.

The lodgment rules have 2 implementations in `packages/lodgment-calendar` until
MCP cutover: the Python source for the supported release and rollback, and the
Rust crate bound to its results by oracle. Rust owns the rules from cutover, when
the Python source and `NATIVE_CANDIDATES` in `tests/test_boundaries.py` retire.
`to_ics` is Python only and is not exposed through MCP.

## Runtime qualification

Effective input validation follows the original strict Pydantic models and
shared money/date gates. Published schema annotations are also preserved, but
are not imposed as extra input rules where they differ from callable behaviour,
such as accepted surrounding whitespace in money strings. Every successful
output is checked against its complete published output schema.

The fixtures were first captured at an earlier Python commit and refrozen with
reviewed transforms at the recorded base; `tests/fixtures/native/origin.json`
keeps both commits, and its `oracle_sha256` values remain the first capture's raw
hashes. The second refreeze, at 035847a, added main's ABN and ACN tools, the
returned-contribution qualification on Payday ON_TIME results and packs, and 143
calls from main's new and changed tests and independent ABN and ACN probes. The
third, at 03baab7, followed main's ranked library search, `in_force_only` and
`version_is_current` on rate searches, later ranking of rulings that carry a
withdrawal notice, the unmatched-industry refusal naming
`list_ato_benchmark_industries` and the stated SG-charge estimate exclusions. It
recaptured the retrieval cases from the Python suite and added 6 benchmark calls
from main's new tests and independent probes. The fourth, at d87dac3, followed
`get_reviewed_rate`, the `doctor` command and its folder census, file notes on
worksheet results, the `returned_by_fund` refusal, dated legislation retrieval
(`as_at`), whole-word rulings matching, `dropped_words` and `partial_hits` in
corpus search, and the changed synthetic corpus and rulings builders. It
recaptured the retrieval cases, with the doctor census, and added 648 calls from
main's new and changed tests and probes. Other counts below date from the first
capture.

The native comparison covers 1,583 captured effective SDK calls for all
non-retrieval tools and unknown tools, plus 39 independent validation probes and
22 further original-only money, date, year and large-pack cases and 9 year-zero
refusal cases. Three large-catalogue cases compare complete original rulings
provenance at 500 runs and 15,000/30,000/60,000 documents.
All pass, including Payday's four complete evidence-pack files and compact
responses. Six resource payloads, 34 prompt cases and 1,175 JSON-RPC replies across
34 legacy and modern sessions also pass. The latter include 200-row full and
compact packs, pagination and configured-folder status without disclosing paths.
Negative schema checks reject misleading money, dates, safety markers and missing
audit sections whilst retaining engine extension fields. All eight retrieval entry points are
implemented, with 823 captured core/boundary/helper/census calls from 409
original retrieval test nodes and 19 further original-only edge cases. Two nodes,
`test_a_title_changed_after_validation_cannot_return_uncaptured_text` and
`test_a_read_failure_while_counting_refusals_stops_the_search`, change the corpus
or its reader in the middle of the call, which a frozen replay cannot do; native
unit tests in `src/retrieval/corpus.rs` prove the guards they cover. The 28 cases of
`test_the_one_word_floor_fails_the_gate_as_recorded` replace corpus search's
partial-match floor and stop words, a configuration the server never runs, and
are not replayed. The capture deselects 2 nodes of `tests/test_as_at.py`,
`test_withheld_counts_stream_without_retaining_every_result` and
`test_dated_search_does_not_retain_each_titles_first_text`: they measure the
Python process's memory, which the capture's own snapshots inflate, so they yield
no cases. Comparisons cover
complete returned objects and exact refusals. Windows and isolated WSL Linux fmt, locked Clippy, debug/release tests, release
builds and copied-binary runs with an empty cwd/PATH pass. Root checks and a fresh
RustSec audit pass. Independent Sol 6.1 and Calvino reviews found no remaining
blocking defect for this local candidate. Aikido was unavailable; hosted CI,
hosted scans and macOS remain unrun gates.
The original 1,083 Python tests passed during oracle
capture. Mapping those tests is not proof of native coverage.

Comparisons preserve all figures, warnings and refusal messages. JSON object
order and pretty-print indentation are incidental; both successful output
channels must contain the same complete JSON payload. The following exact
identity and already-reviewed provenance differences are allowed:

- Native engine versions report the linked 1.0.0 candidates.
- Division 7A's bundled rate table URI is `data/benchmark_rates.csv`, rather
  than the Python package path `div7aloan/data/benchmark_rates.csv`. Its bytes,
  hash and source remain unchanged.
- Payday's bundled GIC URI is `data/gic_rates.json`, rather than
  `paydaysuper/data/gic_rates.json`, including the update-path wording in stale
  rate caveats. Its business-day calendar caveat likewise names
  `data/business_days.json`, rather than `paydaysuper/data/business_days.json`. Evidence-pack version strings and report SHA-256 references
  reflect the actual native version and exact returned CSV bytes. All four
  returned files retain their remaining content, BOM and newlines.
- Study loan repayment results retain native engine commit `4915b94`: the moved
  ATO minimum repayment income page and `source_checked: 2026-10-20`. That
  existing engine change records its official-source recheck and changes no
  calculation. The frozen Python oracle retains the historical address.
- Medicare levy worksheet sources retain native engine commit `afa19a4`: the moved
  ATO lump-sum payment in arrears page. No figure depends on the address.
- The native server links the engine crates on main; the Python server runs
  their PyPI releases (australian-tax-calculators 0.2.0, div7a-loan-review 0.3.2,
  payday-super-checker 0.2.3, gst-review 0.1.0). `tests/engine/mod.rs` applies
  each engine commit to the frozen Python value, as the engine's own
  `tests/parity.rs` does: `04d493f7` (#542) applies only the unused concessional
  cap that covers the excess, `b4a70733` (#551) adds the PAYG withholding 2024-26
  Schedule 1 source, and `d8f34dc2` (#567) adds the FBT rates source and extends
  the source passage. The served disclaimer lists the worksheet scopes as the
  scope resource serves them. Payday `56afe0c` (#602) words the first-contribution
  hint for employment that starts or recommences, including a worker rehired under
  a new arrangement, or a fund switch; no deadline, verdict or figure changes, and an
  evidence pack's report SHA-256 follows its changed report.
- The input schema fixes or bounds the year of every worksheet whose periods
  those commits opened, so the scope resource, `get_data_freshness` and `doctor`
  list only the periods it admits. The 2026-27 periods `fd0b593e` (#543) opened for
  gst, capital gains and depreciation, the 2024-25 and 2025-26 periods #551 opened
  and the FBT year ending 31 March 2027 that #567 opened are refused as Python
  refuses them. #551 and #567 also widen the PAYG and FBT scope text to name those
  periods; the server serves the 0.2.0 words for the periods its schema admits
  (`NARROWED_SCOPE` in `src/adapters/tax.rs`), so the scope a caller confirms is
  the scope it gets. A unit test fails if a served scope names a year from 2024
  on, the earliest advertised period, outside the worksheet's advertised periods;
  earlier years name legislation or history. `tests/narrowed_scope.rs` fails if a
  worksheet's served scope differs from the engine's without an engine period the
  schema refuses, so a narrowing cannot outlive the schema change that opens its
  periods.

No other provenance changes are normalised. The original Python application,
published compatibility metadata and external consumer pins are unchanged.

The rmcp handler preserves the original legacy initialise flags and separate
modern discovery capabilities. A small service adapter retains the original
modern unsupported-version error list whilst using the SDK's validation and
dispatch. Protocol tests use expected replies frozen only from the original
server. Only JSON formatting and the specific documented identities above are
adapted; schemas are compared in full.

Modern subscription listening acknowledges the accepted notification filter and
remains open until cancellation. Its 1,024 concurrent stream limit refuses excess
requests before acknowledgement and frees slots on cancellation. The first
request fixes the connection's protocol era. Legacy initialise capabilities retain their
historical flags. The typed transport adapter preserves the original unknown
bootstrap method and metadata diagnostics whilst retaining rmcp's negotiation,
dispatch and active-session metadata checks. Discover-only EOF exits cleanly.
Malformed JSON-RPC envelopes remain governed by the SDK: an invalid method type
can receive `-32600` where the original Python transport stayed silent, and a
`tools/call`, `prompts/get` or `resources/read` missing its required parameter
receives `-32601` where Python returns `-32602`. Non-string prompt arguments
receive Python's `-32602` on the wire.

Retrieval reads only operator-configured folders and never writes or downloads
source material. Link and Windows reparse checks, bounded scans, immutable run
assumptions, Unicode 17 word/case folding and code point offsets are preserved.
Rulings validate every run before serving any paragraph, retain historical
citations and apply later exclusions across older copies. Validated runs are
cached by both files' sizes and modification times.

The Rust JSON boundary cannot represent lone surrogate names. Relevant Markdown
paths with non-Unicode filesystem names are explicitly refused instead of being
lossily attributed to another file. Hidden and unrelated non-Markdown files are
ignored. Incremental UTF-8 reads handle CR, CRLF and LF records; decoder read-ahead
can differ from Python when invalid bytes occur after a requested row. `doctor`
reports an undecodable line in Python's words; Python counts the position from
its 8,192-byte read chunk and the native server from the start of the file, so
the two agree for an error in the first chunk or at the end of the file. Rare JSON
metadata deeper than serde's default 128 containers uses a bounded larger stack,
with a refusal above 1,000 containers. These are explicit parser limits, not
claims of support for every malformed or Python-specific value.

## Release notes for 1.0.0

`RELEASE_NOTES.md` keeps 0.7.1 first until the release, as its release test
requires; the release moves these notes there. Against the published 0.7.1:

New tools and command:

- `get_reviewed_rate` returns one bundled, reviewed ATO rate or threshold for a
  date in 2025-26 or 2026-27, with its source page and verbatim quote. Any other
  date, a date outside the record's period or an unknown id is refused with no
  figure. `get_data_freshness` lists the reviewed records.
- `validate_abn` and `validate_acn` test the check digit arithmetic that ABN
  Lookup and ASIC publish. A valid check digit proves neither ABR nor GST
  registration.
- `aus-accounting-mcp doctor` prints the version, data periods and each
  configured folder, and exits 1 if a folder is broken.

Changed tools:

- `calculate_tax_worksheet` takes `file_note`; true also returns a Markdown note
  restating the result for a workpaper.
- `calc_payday_super_deadline`, and each row of
  `review_payday_super_contributions` and `build_payday_super_evidence_pack`,
  take `returned_by_fund`; true refuses the review, because the checker cannot
  assess a returned contribution. Otherwise an `ON_TIME` verdict, or an `UNKNOWN`
  listing `ON_TIME` in `horizon_verdicts`, carries `verdict_qualification`
  saying it assumes the fund kept every receipt. The SG-charge estimate fields
  and the disclaimer say the estimates exclude choice loading, the maximum
  contributions base and post-assessment penalties.
- `search_tax_legislation` and `search_tax_rates` take a plain question. Stop
  words are dropped and listed in `dropped_words`; words of legal force such as
  'must', 'may' and 'not' stay. Room left on a page goes to `partial_hits`,
  ranked below every match and labelled with the words each holds and lacks.
- All 4 searches match whole words only, and read a modal with its 'not', so
  'must lodge' does not match 'must not lodge'. Library search lists lines whose
  heading holds the words first, then lines holding them as a phrase. Rulings
  search puts a paragraph whose captured page held a withdrawal notice after
  those without, within the same tier.
- `search_tax_legislation` and `read_tax_legislation_section` take `as_at` and
  answer for that date from each title's compilation window: text comes back
  only from a title whose status is captured, and other titles are refused by
  name with their status and next step.
- `search_tax_rates` takes `in_force_only`, which defaults to true, so a rate
  search now leaves out rows from a compilation the corpus marks as superseded.
  Set it to false to see them, each with a caveat. `search_tax_legislation` and
  `define_tax_term` keep their default of true; with `as_at`, the date's status
  decides instead.
- An unknown industry in `get_ato_benchmarks` names
  `list_ato_benchmark_industries` and the year to search.
- The server instructions say to state a differing figure beside the tool's,
  and never substitute either silently.

Worksheet engine changes:

- `carry_forward_applied` in the `contribution_caps` worksheet is now only the
  unused concessional cap needed to cover contributions over the year's general
  cap (ITAA 1997 s 291-20, engine commit `04d493f7`). With $15,000 of eligible
  unused cap and $40,000 contributed, it is $10,000 for 2024-25 and 2025-26
  ($30,000 cap) and $7,500 for 2026-27 ($32,500 cap), where 0.7.1 reported
  $15,000. Available capacity, remaining room and excess are unchanged. No other
  figure differed in the 1,716-call parity replay against the Python server,
  which runs 0.7.1's engine releases.
- The PAYG withholding and FBT worksheets cite an added ATO source and extend
  the source passage (`b4a70733`, `d8f34dc2`). Their scope and input schema are unchanged: the
  2026-27 PAYG year and the FBT year ending 31 March 2026.
