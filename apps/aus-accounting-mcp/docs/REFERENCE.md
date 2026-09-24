# Aus Accounting MCP reference

[Installation and overview](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/README.md). Commands below run from `apps/aus-accounting-mcp/`.

## Scope

Payday Super, ATO benchmark and Division 7A computations are delegated to:

- [payday-super-checker](https://github.com/ryanduguid/australian-accounting/tree/main/packages/payday-super-checker) (`payday-super-checker`)
- [ato-benchmark-compare](https://github.com/ryanduguid/australian-accounting/tree/main/packages/ato-benchmark-compare) (`ato-benchmark-compare`)
- [div7a-loan-review](https://github.com/ryanduguid/australian-accounting/tree/main/packages/div7a-loan-review) (`div7a-loan-review`)

The Division 7A adapter covers reviewed s 109N loan terms and benchmark rates plus s 109E minimum yearly repayments for one operator-supplied amalgamated loan. It fails closed on unknown facts and refuses unsupported matters such as forming amalgamated loans, s 109R repayment classification, unpaid present entitlements, distributable surplus, interposed entities, debt forgiveness, and Commissioner discretion.

## Demonstration and provenance

![Static terminal proof of synthetic BAS output and Division 7A loan review](https://raw.githubusercontent.com/ryanduguid/australian-accounting/main/apps/aus-accounting-mcp/docs/quick-proof.webp)

**Release demonstration:** from `apps/aus-accounting-mcp/` in a repository checkout, run the fabricated
demonstration without starting the stdio server:

```bash
uv run --locked aus-accounting-mcp-demo
```

The [checked text transcript](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/docs/quick-proof.txt) is the accessible source of truth for the image. Its registered MCP calls return a synthetic BAS fixture with `synthetic: true` and `not_a_lodgment: true`, then review fabricated Division 7A loan facts through the delegated engine.

Expected structured success:

```text
synthetic: true
not_a_lodgment: true
form_type: BAS_AU_ACTIVITY_STATEMENT
summary.total_payable_to_ato: "42500.00"
```

Expected structured Division 7A review:

```text
engine: div7a-loan-review
gate.verdict: COMPLYING
minimum_yearly_repayment.verdict: MYR_MET
minimum_yearly_repayment.myr_required: "108770.00"
minimum_yearly_repayment.shortfall: "0.00"
```

The example is fabricated, is not a lodgement or Division 7A determination, is not tax advice, and requires human review before any consequential accounting action. It does not model s 109R, distributable surplus, unpaid present entitlements or the other refused areas. It neither uses client data nor contacts external services.

### Asset provenance

| Asset | Purpose | Source | Licence | Creation | SHA-256 | Refresh trigger |
|---|---|---|---|---|---|---|
| `docs/quick-proof.webp` | Static terminal summary of the two checked demonstration outcomes | `docs/quick-proof.txt`, emitted by `aus-accounting-mcp-demo` | MIT | `uv run --locked --extra dev python scripts/render_demo_image.py docs/quick-proof.txt docs/quick-proof.webp` with Pillow 12.3.0 | `2b655b40c5f9b97451c713aed54d9adaf91a626b271ce7b7d9f411b0d66c26ec` | Regenerate when the transcript, demo output, render constants or pinned Pillow version changes |

Name mapping: public name Aus Accounting MCP; repository australian-accounting; Python distribution aus-accounting-mcp; stdio MCP executable aus-accounting-mcp; demonstration executable aus-accounting-mcp-demo; MCP Registry identity io.github.ryanduguid/aus-accounting.

Release and compatibility references: [CI](https://github.com/ryanduguid/australian-accounting/actions/workflows/ci.yml), [v0.2.7 release](https://github.com/ryanduguid/australian-accounting/releases/tag/aus-accounting-mcp/v0.2.7), [PyPI 0.2.7](https://pypi.org/project/aus-accounting-mcp/0.2.7/), [MCP Registry 0.2.7](https://registry.modelcontextprotocol.io/v0.1/servers/io.github.ryanduguid%2Faus-accounting/versions/0.2.7), and [compatibility.json](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/compatibility.json). Treat a version as published only after its target resolves and matches the compatibility record. The record links each engine's maintained source and release. The runtime `law_content_date` and `source` fields stay engine-owned.

## Client setup

**Standard config** works with hosts that run a local stdio MCP server:

```json
{
  "mcpServers": {
    "aus-accounting": {
      "command": "uvx",
      "args": [
        "aus-accounting-mcp"
      ]
    }
  }
}
```

Ready-made copies live in [`clients/`](https://github.com/ryanduguid/australian-accounting/tree/main/apps/aus-accounting-mcp/clients).

### Cursor

[![Add to Cursor](https://img.shields.io/badge/Cursor-Add%20MCP-black)](https://cursor.com/en/install-mcp?name=aus-accounting&config=eyJjb21tYW5kIjoidXZ4IiwiYXJncyI6WyJhdXMtYWNjb3VudGluZy1tY3AiXX0=)

Or drop the standard config into `~/.cursor/mcp.json`.

### Claude Desktop

Paste the standard config into `claude_desktop_config.json` (`%APPDATA%\Claude\` on Windows, `~/Library/Application Support/Claude/` on macOS).

### Claude Code

```bash
claude mcp add aus-accounting -- uvx aus-accounting-mcp
```

### Codex

```bash
codex mcp add aus-accounting -- uvx aus-accounting-mcp
```

### Gemini CLI

```bash
gemini mcp add -s user aus-accounting uvx aus-accounting-mcp
```

### VS Code

```bash
code --add-mcp "{\"name\":\"aus-accounting\",\"command\":\"uvx\",\"args\":[\"aus-accounting-mcp\"]}"
```

### Windsurf

Paste the standard config into `~/.codeium/windsurf/mcp_config.json`.

### Zed, OpenCode and other stdio hosts

Use the standard config above. Any host that launches a local stdio MCP server can
run this one; there is no hosted endpoint, account or key.

### Hosts that only accept a remote server

ChatGPT connectors and the Claude.ai web app take a remote MCP URL, not a local
command, so they cannot run this server. Use a desktop or CLI host from the list
above. This server is deliberately local: your figures and your configured folders
stay on your machine.

## Tool reference

| Tool | Job | Engine |
| :--- | :--- | :--- |
| `list_ato_benchmark_industries` | List or search the shipped ATO business types | ato-benchmark-compare |
| `get_ato_benchmarks` | Compare operator-supplied bucket totals to ATO ranges | ato-benchmark-compare |
| `calc_payday_super_deadline` | Review one contribution against Payday Super timing | payday-super-checker |
| `review_payday_super_contributions` | Assess up to 200 related contributions for one employer | payday-super-checker |
| `build_payday_super_evidence_pack` | Return 4 in-memory review files for supplied contributions; requires checker evidence-pack support | payday-super-checker |
| `calculate_tax_worksheet` | Run one of 7 bounded worksheets with established scope | australian-tax-calculators |
| `search_accounting_library` | Search explicitly configured local Markdown files | local read-only retrieval |
| `read_accounting_library` | Read bounded lines with file, line, page and hash citations | local read-only retrieval |
| `search_tax_legislation` | Find provisions in a configured local legislation corpus, cited to Act, section, compilation and register page | local read-only retrieval |
| `read_tax_legislation_section` | Read one cited provision from that corpus in full, optionally with the provisions either side | local read-only retrieval |
| `define_tax_term` | Find an expression's statutory definitions in that corpus's dictionary, definitions and interpretation sections | local read-only retrieval |
| `search_tax_rates` | Find legislated rate, threshold, indexation and factor rows with the provision that sets them | local read-only retrieval |
| `get_div7a_benchmark_rate` | Return the reviewed s 109N(2) rate for a year, or `UNKNOWN` | div7a-loan-review |
| `review_div7a_loan` | Review s 109N terms and s 109E minimum yearly repayment for one operator-supplied amalgamated loan | div7a-loan-review |
| `refuse_div7a` | Refuse Division 7A matters outside the reviewed engine scope. Takes no arguments | MCP policy |
| `generate_synthetic_sbr_fixture` | Synthetic CTR/BAS for agent tests (`synthetic: true`) | local fixture |

`refuse_div7a` answers the questions this server does not review, and those arrive
with no loan facts, so it publishes no inputs. Call it with no arguments and do not
invent a borrower, a lender or a principal to reach it; the legacy inputs it once
accepted and ignored were removed so a schema cannot invite them.

Every tool also publishes a human-readable title for host menus.

MCP initialisation supplies server-wide instructions for choosing tools and handling
missing facts. Every tool publishes an output schema describing its returned fields,
including verdicts, decimal strings, warnings and source information. `ok: true`
means the tool ran, not that the review passed; retain `UNKNOWN`, `REFUSED`,
`not_supplied` and `null` results when presenting findings. Engine audit fields are
preserved, and clients receive the same payload in structured content and JSON text,
with one exception: `build_payday_super_evidence_pack` with `response_detail="compact"`
puts only a summary in the text content, so a client using that mode must read the four
evidence files from `structuredContent.files`.
An unavailable Division 7A `benchmark_year_used` may be the engine's empty string
or null; neither is evidence of a reviewed year.

The Division 7A tools default to `response_detail="summary"`, retaining outcomes,
amounts, caveats, versions and a verification link while reducing tool-result size.
Pass `response_detail="full"` when the complete provenance, statutory trace and
per-limb audit material are required.

For concise industry discovery, call `list_ato_benchmark_industries` with
`{"search":"shop","year":"2023-24","limit":20}`. `count` is the number returned;
`total_count` is the number matching the search, while `total_business_types` covers
the whole dataset. If `has_more` is true, pass `next_offset` as `offset` with the
same search, limit and returned `benchmark_year` as `year`. `next_offset` is null
at the end. Limits must be integers from 1 to 100 and offsets non-negative integers.
Omitting `limit` or setting it to null preserves full-list calls; an offset still
skips that many matching entries. Source metadata accompanies every page.

Twenty-four fabricated, read-only agent evaluation questions are in
[evaluation/questions.xml](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/evaluation/questions.xml), each with its exact expected
answer and the tools a correct answer needs. The normal pytest suite replays them
through a real stdio MCP session using the locked engines, checking both the
answer and that reaching it called exactly the published tools, so that list
cannot drift from what the server actually requires. The fixed dataset years and
engine versions define the evaluation baseline; review expected answers when
upgrading an engine.

Answer reproducibility is not tool-selection quality: the replay is told which
tool to call. To measure the other half, `evaluation/tool_selection.py` prints
the server instructions, tool definitions and preloaded scope resource, plus
questions with the answer and tool elements stripped. It then scores a recorded
run against the published selection:

```bash
python evaluation/tool_selection.py context
python evaluation/tool_selection.py questions
python evaluation/tool_selection.py score runs/recorded.json
```

Record each question's calls in order, including the arguments, and its final
answer. For example, a fabricated recording for the unknown-rate question is:

```json
{
  "unknown-rate": {
    "calls": [
      {"name": "get_div7a_benchmark_rate", "arguments": {"year_of_income": "2027-28"}},
      {"name": "get_div7a_benchmark_rate", "arguments": {"year_of_income": "2027-28", "response_detail": "full"}}
    ],
    "answer": "UNKNOWN"
  }
}
```

The scorer compares calls and answers with the checked reference in
`questions.xml`. It detects changed arguments, invented zeroes, extra calls,
changed call order and answers that replace `UNKNOWN`. Matching is exact,
including decimal strings and omitted fields; surrounding answer whitespace is
ignored. An equivalent alternative workflow can differ from the reference, so
review mismatches before judging a model's answer. Unrecorded questions remain
`NOT ANSWERED`. Old lists of tool names still receive a selection score, with
arguments and answers marked `NOT EVALUATED`.

Scoring reads the recording without executing its calls or contacting a model.
Malformed recordings exit with code 2; valid recordings report their score and
exit with code 0. Live model trials remain supplementary to deterministic CI.

`calc_payday_super_deadline` requires `as_at`. It does not invent clearing-house latency and cannot confirm LCR 2026/1 transition allocation. A remittance date alone cannot produce `ON_TIME`.

For partial contributions, pass `matched_amount` for the amount associated with
the payday, including when no remittance date is known. `remitted_amount` records
the amount sent and requires `remitted`. Both use the shared money limits and
the engine's validation. `matched_amount` takes precedence when determining how
much a receipt evidences; otherwise the engine uses `remitted_amount`. If both
are omitted, the existing full-receipt convention applies. A fabricated $120
liability with $50 matched and received leaves a $70 base shortfall. The result
echoes both amounts, with null for an omitted amount.

All tools reject unknown argument names instead of silently discarding facts.
The input schemas also declare `additionalProperties: false`.
Boolean eligibility facts require JSON `true` or `false`; strings such as
`"yes"` and numbers such as `1` are rejected. Omit an unknown Division 7A fact
or supply `null` to preserve `UNKNOWN`.

Payday results carry `assessment_scope: "single_contribution"` and a caveat
about related contributions. This tool does not allocate receipts across QE
days or review s 18C(2) item 4 deadline alignment. Related evidence can change
the deadline or shortfall, so use `review_payday_super_contributions` for a grouped
review where that context matters. Repeated single-contribution calls do not
establish a payroll-wide result. Establish `sg_amount` separately: the tool
does not determine worker eligibility, qualifying earnings or SG entitlement.

Omitted ATO expense buckets are `not_supplied`, not zero. Every ATO ratio divides by turnover, which the ATO rule takes from sales or from total business income, so omitting `other_income` leaves every ratio `not_supplied` until you establish that figure. Pass `0` where you have established there is none. Withholding covers the engine's prose as well as the structured fields: each engine `notes` and `checks_to_make` entry declares the figures needed to state it, and an entry resting on a bucket you omitted is withheld rather than published beside that bucket's `null`. `notes` records how many were withheld. `key_ratio` is withheld the same way, so an omitted `cost_of_sales` does not trigger the ATO's total-expenses fallback.

Amounts, including Division 7A loan balances and payments, are decimal strings, finite, at most 2 decimal places, and no greater than AUD 1,000,000,000,000.00. Dates are ISO-8601. Payday Super uses payday-super-checker's national SGAA 1992 s 6(1) calendar.

Payday Super dates come out of a payroll or clearing-house export, so
`calc_payday_super_deadline` reads the shapes those exports hold, through the
engine's own parser rather than one of its own: `2027-07-13`, day-first
`13/07/2027` or `13-07-2027`, `13 Jul 2027`, and a date-time with no timezone
marker, whose time the law ignores. Two are refused. A stamp carrying `Z` or a
UTC offset is the engine's refusal, because a UTC evening is already the next day
in Australia and keeping the written day could pass a receipt that was really a
day later; convert it to the Australian calendar date first. A numeric date is refused at this
boundary only where the 2 readings give different days, such as `01/07/2027`: it
is read day first, nothing in the text rules out the other reading, and the
difference is a month in a date that decides the verdict. Send those as
`YYYY-MM-DD`. This refusal also applies when a numeric date has a time suffix,
such as `01/07/2027 00:00`. Two things settle the reading and are accepted: a component above 12
can only be the day, so `13/07/2027` is the 13th, and equal components land on the
same date either way, so `12/12/2027` is the 12 December. Results are always
ISO-8601.

Division 7A already takes a loan in the shape a register row holds: every
`review_div7a_loan` argument is one column of the engine's register CSV, passed
through the engine's own `GateFacts` and `MyrFacts` readers. Whole files and
multi-loan registers stay out of scope, because the engines expose those only
behind a file path and reading one here would mean this facade owning input
handling the engines do not.

## Prompts

The 3 documented workflows are registered as MCP prompts, so a host can offer
them from its prompt menu rather than having you paste the text. Each argument is
optional: supply it and the prompt names it, omit it and the prompt asks you for it
instead of assuming one.

| Prompt | Argument | Job |
| :--- | :--- | :--- |
| `compare_ato_benchmarks` | `industry` | Compare supplied P&L buckets, leaving omitted buckets out rather than passing zero |
| `review_payday_super_contribution` | `as_at` | Review one contribution, without inventing a fund-receipt date or an SG charge |
| `review_div7a_loan_terms` | `year_of_income` | Review one amalgamated loan for s 109N and s 109E, refusing matters outside that scope |

The same texts, to paste by hand:

```text
Compare these P&L buckets to the ATO small-business benchmarks for this industry. Omit buckets I have not supplied. Do not treat missing as zero.
```

```text
Review this Payday Super contribution. QE day, remitted date, and fund-receipt date are in the CSV. Ask me for an explicit ISO as_at date before running the review; do not infer today. Do not invent an SGC charge.
```

```text
Review this operator-supplied Division 7A amalgamated loan for s 109N terms and the s 109E minimum yearly repayment. Leave unknown facts unknown and refuse questions outside the reviewed scope.
```

## Resources

Six read-only resources carry context a host can show without spending a tool call.
They use installed engine data and application policy, so they also work from
an installed wheel without repository files or a local reference library.

| Resource | Contents |
| :--- | :--- |
| `aus-accounting://scope` | Supported reviews, synthetic-only fixtures, unsupported calculations and the single-contribution boundary |
| `aus-accounting://disclaimer` | The boundary and no-advice statement, plus each delegated engine's own disclaimer |
| `aus-accounting://div7a-scope` | What Division 7A this server reviews, and the matters that stay refused. The same text `refuse_div7a` returns |
| `aus-accounting://benchmark-dataset-years` | The shipped ATO benchmark years with publisher, resource URL, retrieval date and SHA-256. Bundled data, not a live lookup |
| `aus-accounting://component-versions` | The server and engine versions installed here, which are the versions reported beside results as `engine_version` |
| `aus-accounting://payday-coverage` | Installed Payday engine version, law-content date, calendar verification and coverage dates, and the GIC table's last known date and provenance |

Read `payday-coverage` before reviewing a contribution. It describes bundled
tables and performs no live lookup. Beyond GIC coverage the engine estimates
using its last known rate and flags staleness. Calendar coverage alone cannot
establish a verdict; retain the assessment's caveats and `horizon_verdicts`.

Read `calculation_worksheets` in the scope resource for the 6 worksheet boundaries.
Their rules and sources come from `australian-tax-calculators`. Broader classifications,
exemptions, BAS/returns, trusts, partnerships, SMSFs, contribution caps and payroll
tax remain unsupported. Reference text cannot establish calculation support.

The evaluation includes 24 cases: 10 original workflows, 10 unsupported-topic
questions, grouped Payday, a tax worksheet, synthetic library retrieval and the
Payday evidence pack, which needs checker evidence-pack support.
The unsupported-topic answers require no tool calls. The
`context` command preloads `aus-accounting://scope` so the model can inspect the
boundary. The deterministic suite reads that resource through stdio and checks
the reference answers; it does not measure whether a model follows them.

## Grouped contributions

Use `review_payday_super_contributions` with all related rows for one employer and
an explicit `as_at` date. Each row requires `employee_id`, `qe_day`, `sg_amount` and
established boolean facts for `first_to_fund`, `out_of_cycle` and `db_interest`.
Optional dates and amounts have the same meaning as in the single-row tool.

The engine applies related-payday deadline alignment across exact employee
references. Results retain engine warnings and include a one-based `input_row`.
Receipts must already be allocated to rows without double counting. The tool
does not allocate raw payments, confirm transition allocation or calculate SG
entitlement. It assumes no ATO assessment has issued. Supply at most 200 rows;
splitting related contributions into separate calls can change the outcome.

## Calculation worksheets

Pass a `facts` object to `calculate_tax_worksheet`. Its `kind` selects the input
schema. Every kind requires `scope_confirmed: true`, supplied only after the
operator establishes all conditions in `aus-accounting://scope`. Unknown facts
must be resolved first. Results retain the engine's sources, source-check date,
supported period, exclusions and warnings.

| Kind | Required facts beyond scope confirmation | Supported period |
| --- | --- | --- |
| `gst` | `amount`, `gst_inclusive`, `year` | 2025-26, one ordinary taxable supply |
| `resident_tax` | Whole-dollar `taxable_income`, `year` | 2024-25 to 2026-27, full-year resident basic tax before offsets and levies |
| `capital_gains` | `other_gains`, `discount_gains`, `current_losses`, `prior_losses`, `year` | 2025-26, established ordinary resident-individual gains and losses |
| `fbt` | `type_one_value`, `type_two_value`, `year_ended` | FBT year ended 31 March 2026, ordinary taxable employer |
| `depreciation` | `cost`, `effective_life`, `days`, `taxable_use`, `method`, `year` | 2025-26, first year of an ordinary tangible Division 40 asset |
| `quarterly_sg` | `ordinary_time_earnings`, `qualifying_contributions`, `quarter`, `year` | 2025-26, one complete quarter for one eligible employee and employer |
| `payg_withholding` | `earnings`, `pay_period`, `scale`, `year` | 2026-27, one regular weekly, fortnightly or monthly pay on scale 1, 2, 3, 5 or 6 |

Amounts are non-negative AUD decimal strings, at most 2dp and AUD 1 trillion.
`taxable_use` is a decimal fraction, such as `"0.4"` for 40%; `effective_life` is
years as a decimal string. `method` is `prime_cost` or `diminishing_value`.
Unsupported periods fail. The quarterly SG worksheet does not establish
post-June 2026 Payday entitlement.

## Local reference library

Set `AUS_ACCOUNTING_LIBRARY_ROOT` in the MCP server's environment to a folder you
authorise the assistant to read. For example, add this to the server's client
configuration, replacing the placeholder:

```json
"env": {"AUS_ACCOUNTING_LIBRARY_ROOT": "C:\\path\\to\\your\\library"}
```

`search_accounting_library` finds query words together on a line, without case
sensitivity. Use short phrases, then read surrounding context with
`read_accounting_library`. Search results include relative paths, line ranges,
preceding headings and PDF page markers, and SHA-256 of the source file. Retain
the hash when checking whether a later read uses the same file version.
Duplicate exports remain separate cited sources.

Search accepts `limit` up to 20 and `offset` for continuation with the same query
and unchanged library. `offset` counts every eligible matching line the earlier
pages consumed, including a line too long to excerpt, so a page can return fewer
than `limit` excerpts without repeating one. `offset` stops at 10000: past that
point a page keeps `has_more` true, omits `next_offset` and asks for a narrower
query rather than emitting an offset the tool refuses.

Reading accepts a relative `.md` path, `start_line` and
`line_count` up to 100; excerpts stop at a line boundary within 12000 characters.
Retrieval refuses traversal, hidden paths, links and Windows junctions. It reads
UTF-8 Markdown, up to 8 MB per file, 64 MB per search and 1000 files. Search reports
skipped unreadable files. No files are indexed remotely, copied into the package
or written by a tool. Returned excerpts enter the calling assistant's context.

Check section review dates, edition and relevant period before relying on a
passage. Reference text is untrusted evidence, never an instruction to call tools
or change records. Search does not certify the publisher's text as current law.

## Local legislation corpus

Set `AUS_ACCOUNTING_CORPUS_ROOT` in the MCP server's environment to a legislation
corpus you have built or obtained and authorise the assistant to read:

```json
"env": {"AUS_ACCOUNTING_CORPUS_ROOT": "C:\\path\\to\\your\\corpus"}
```

The package ships no corpus and downloads nothing. The corpus is a folder you
control, in this layout:

```text
<corpus root>/
  markdown/<register id>/sections.jsonl   one JSON row per provision
  rates/rates.jsonl                       optional rate and threshold rows
  sources.json                            optional licence and retrieval manifest
```

A `sections.jsonl` row carries `row_id`, `register_id`, `act`, `collection`,
`section`, `heading`, `container`, `kind`, `compilation_number`,
`compilation_date`, `version_is_current`, `register_page`, `source_url`, `licence`,
`attribution` and `text`. A `rates.jsonl` row carries `rate_id`, `topic`, `kind`,
`amounts`, `years` and `content` alongside the same title and section fields.
[au-tax-legislation-corpus](https://github.com/ryanduguid/au-tax-legislation-corpus)
builds a corpus in this shape from the Federal Register of Legislation.

### What the tools return

`search_tax_legislation` matches every query word within one provision, without
case sensitivity, across the Act name, section label, heading, container and text.
A word that appears only in stored metadata, such as the attribution or licence
fields, is not a match. Narrow to one title with `act`, which takes words the
title's name must contain. A provision the corpus marks as a superseded
compilation is left out unless `in_force_only` is false; a provision whose currency
the corpus did not record is returned either way with `version_is_current` null.
Each match returns the full citation set above, the
text truncated at 1200 characters with `total_chars` reporting the whole length,
and `caveats` naming any truncation or superseded compilation.

`read_tax_legislation_section` takes a `row_id` from a search result and returns
that provision with the same citation fields and up to 12000 characters of text.
`neighbours`, 0 to 5, adds that many provisions on each side in the title's
document order as `before` (nearest last) and `after` (nearest first), each cited
and truncated like a search match, so a subsection can be read with the provisions
around it without guessing their labels. A container heading row counts as a
neighbour.

`define_tax_term` reads every section whose heading is its label followed by
"Definitions", "Interpretation" or "Dictionary", such as ITAA 1997 s 995-1, ITAA
1936 s 6 and GST Act s 195-1 (an operative section headed "Extended definition
of ..." is not one), and splits it into definitions: an entry opens with the defined
expression and the words that introduce its meaning ("means", "has the meaning
given by", "includes", a colon), and keeps the notes and paragraphs that follow it.
A definition whose expression is the `term` is an `exact` match; one whose
expression contains every word of the term is `partial`. Exact matches come first,
then partial ones, in corpus order, up to `limit` (default 5, at most 20), with
`has_more` when more remain and `act` to read one Act's dictionary. Each entry
carries `head`, the expression as the dictionary writes it, the definition text
truncated at 1200 characters, and the citation of the dictionary section that holds
it. A non-breaking hyphen or space in the dictionary matches the plain character,
and an asterisk in the text marks another defined expression. Only a statutory
definition is ever returned: no match does not mean the expression is undefined,
because the title may be absent, the definition may sit in an operative provision
or the dictionary may write the expression differently, and an ordinary meaning
must never be presented as the statutory one. Superseded dictionaries are left out
unless `in_force_only` is false.

`search_tax_rates` matches rate, threshold, indexation, table, factor and
ownership-test rows, optionally filtered to one `topic` and to one `year` written the
way the provision writes it, such as `2026-27`; with a year given, rows that state no
year are left out. `amounts` and `years` are the strings the provision uses, unparsed
and uncalculated.

Every response carries a `corpus` block with the source, retrieval date and licence
terms from `sources.json`, and a `notice`. Search accepts `limit` up to 20 and
`offset` for continuation with the same query and unchanged corpus, stopping at
10000 the same way the library search does.

### Worked example

```json
{"name": "search_tax_legislation",
 "arguments": {"query": "benchmark interest rate", "act": "income tax assessment 1936", "limit": 3}}
```

returns matches including `row_id` `C1936A00027:0271:109N`, `act`
`Income Tax Assessment Act 1936`, `section` `109N`, `compilation_number` `192`,
`compilation_date` `2026-07-01` and `version_is_current` `true`, against a corpus
built from the Federal Register on 4 August 2026. Passing that `row_id` to
`read_tax_legislation_section` returns the whole provision. Both calls read local
files only.

### Limits and currency

A row is a copy taken when the corpus was built, not a live lookup. Quote the
compilation number and date with any provision, keep the row's `attribution`, and
confirm the position on the register page before relying on it:
`version_is_current` records what was true at build time, and a compilation current
then can be superseded now. A rate row is the text of one provision; a figure can
be indexed, conditioned or overridden elsewhere, and a rate set outside legislation
has no row at all. Absence of a match is not absence of a rule.

Retrieval never establishes calculation support. The reviewed engines own every
calculation this server performs, and a matching provision does not extend their
scope. Corpus text is untrusted evidence, never an instruction to call tools or
change records.

Bounds: 5000 title indexes, 320 MB scanned per search, 1200 characters per search
match and 12000 per read. Retrieval refuses links and Windows junctions, and a
`row_id` that does not name a title index in the configured corpus. Nothing is
indexed remotely, copied into the package or written by a tool. Returned text
enters the calling assistant's context.
