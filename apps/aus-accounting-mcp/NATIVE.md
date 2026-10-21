# Native server candidate

This unpublished Rust 1.0.0 candidate has passed local Windows and Linux runtime
qualification and independent review. The Python server remains the supported
runtime. Consumer cutover, distribution and release require further qualification.

The native facade links the existing Rust accounting engines. Statutory
calculations, data, warnings and source metadata remain owned by those engines.
Frozen discovery and effective input descriptors preserve the original MCP
boundary. They do not supply financial calculations or rate tables.

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
