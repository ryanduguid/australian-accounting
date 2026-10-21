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
supported Python distribution. The native candidate archive includes all 3
commands; hosted archive creation remains unverified until CI runs.

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

## Retained Python tooling

These are explicit remaining migration decisions, not retirement approval.
Runtime parity alone does not justify removing the supported Python package.

| Tool or dependency | Current disposition |
| --- | --- |
| `aus_accounting_mcp/demo.py` | Native command implemented above. Retain the Python command and its proof/media checks whilst Python remains supported. |
| `evaluation/tool_selection.py` | All 3 native commands implemented above. Retain the original as the supported Python workflow and migration oracle until distribution and consumer cutover are approved. |
| `scripts/check_installed_worksheets.py` | Retain as the supported wheel/pin check. Native relocation and complete-result tests are additional evidence; a native distribution check must explicitly cover all worksheet kinds and supported periods before this script retires. |
| `scripts/check_corpus_setup.py` | Retain the fabricated integration check until the corpus builder's own Rust migration and native MCP integration are reconciled. It imports that builder's Python entry point; changing only the facade would leave this dependency. |
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
