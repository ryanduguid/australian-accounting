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
reconciles all 375 original test functions across 31 files. It records 279 with
native observable checks, 50 with explicit native adaptations and 46 retained
with the supported Python distribution and its release, Registry, documentation
or media workflow. Retained checks are active and are not retirement approvals.
The record identifies proof symbols and limitations for each function; a Rust
test refuses changed original source hashes until the reconciliation is reviewed.
These are function dispositions, not 375 independent Rust tests or proof of
untested inputs, current law or real taxpayer results.

`tests/original_behaviours.rs` tests relationships that frozen payloads cannot
establish: the exact 46 money paths, published grammar and Draft 2020-12 inputs,
model bounds, pension day limits, receipt rounding against the actual CSV,
related-payday alignment, current GIC provenance, conformance cases, labour
warning prerequisites, Division 7A response sizes, current lodgment facts and
linked versions. Retrieval tests use valid external link targets and an invalid
manifest to test refusal and parse ordering. The stdio tests execute the current
37 evaluation questions with fabricated files, derive every reference answer and
check result-dependent arguments. Another session tests all 22 unknown-argument
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

## Runtime qualification

Effective input validation follows the original strict Pydantic models and
shared money/date gates. Published schema annotations are also preserved, but
are not imposed as extra input rules where they differ from callable behaviour,
such as accepted surrounding whitespace in money strings. Every successful
output is checked against its complete published output schema.

The current native comparison covers 786 captured effective SDK calls for all
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
implemented, with 222 captured core/boundary/helper calls from all 165 original
retrieval test nodes and 19 further original-only edge cases. Comparisons cover
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
  rate caveats. Evidence-pack version strings and report SHA-256 references
  reflect the actual native version and exact returned CSV bytes. All four
  returned files retain their remaining content, BOM and newlines.
- Study loan repayment results retain native engine commit `4915b94`: the moved
  ATO minimum repayment income page and `source_checked: 2026-10-20`. That
  existing engine change records its official-source recheck and changes no
  calculation. The frozen Python oracle retains the historical address.

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
can receive `-32600` where the original Python transport stayed silent.

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
can differ from Python when invalid bytes occur after a requested row. Rare JSON
metadata deeper than serde's default 128 containers uses a bounded larger stack,
with a refusal above 1,000 containers. These are explicit parser limits, not
claims of support for every malformed or Python-specific value.
