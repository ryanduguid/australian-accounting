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

**Release proof:** from `apps/aus-accounting-mcp/` in a repository checkout, run the fabricated
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
| `docs/quick-proof.webp` | Static terminal summary of the two checked demonstration outcomes | `docs/quick-proof.txt`, emitted by `aus-accounting-mcp-demo` | MIT | `uv run --locked --extra dev python scripts/render_demo_image.py docs/quick-proof.txt docs/quick-proof.webp` with Pillow 12.3.0 | `90f983a9b8f76455e9f473330c8b999f9929870731f0c9fbc70e87cd790a43df` | Regenerate when the transcript, demo output, render constants or pinned Pillow version changes |

Name mapping: public name Aus Accounting MCP; repository australian-accounting; Python distribution aus-accounting-mcp; stdio MCP executable aus-accounting-mcp; demonstration executable aus-accounting-mcp-demo; MCP Registry identity io.github.ryanduguid/aus-accounting.

Canonical published release and compatibility references: [CI](https://github.com/ryanduguid/australian-accounting/actions/workflows/ci.yml), [v0.2.1 release](https://github.com/ryanduguid/australian-accounting/releases/tag/aus-accounting-mcp/v0.2.1), [PyPI 0.2.1](https://pypi.org/project/aus-accounting-mcp/0.2.1/), [MCP Registry 0.2.1](https://registry.modelcontextprotocol.io/v0.1/servers/io.github.ryanduguid%2Faus-accounting/versions/0.2.1), and [compatibility.json](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/compatibility.json). Treat a version as published only after its target resolves and matches the compatibility record. The record links each engine's maintained source and release. The runtime `law_content_date` and `source` fields stay engine-owned.

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

## Tool reference

| Tool | Job | Engine |
| :--- | :--- | :--- |
| `list_ato_benchmark_industries` | List or search the shipped ATO business types | ato-benchmark-compare |
| `get_ato_benchmarks` | Compare operator-supplied bucket totals to ATO ranges | ato-benchmark-compare |
| `calc_payday_super_deadline` | Review one contribution against Payday Super timing | payday-super-checker |
| `review_payday_super_contributions` | Assess up to 200 related contributions for one employer | payday-super-checker |
| `calculate_tax_worksheet` | Run one of six bounded worksheets with established scope | australian-tax-calculators |
| `search_accounting_library` | Search explicitly configured local Markdown files | local read-only retrieval |
| `read_accounting_library` | Read bounded lines with file, line, page and hash citations | local read-only retrieval |
| `get_div7a_benchmark_rate` | Return the reviewed s 109N(2) rate for a year, or `UNKNOWN` | div7a-loan-review |
| `review_div7a_loan` | Review s 109N terms and s 109E minimum yearly repayment for one operator-supplied amalgamated loan | div7a-loan-review |
| `refuse_div7a` | Refuse Division 7A matters outside the reviewed engine scope. Takes no arguments | MCP policy |
| `generate_synthetic_sbr_fixture` | Synthetic CTR/BAS for agent tests (`synthetic: true`) | local fixture |

`refuse_div7a` answers the questions this server does not review, and those arrive
with no loan facts, so it requires none. Call it with no arguments: the refusal is
the same whatever is passed, and every input it still accepts is a retained legacy
field that is ignored. Do not invent a borrower, a lender or a principal to reach
it. A `loan_principal` that is supplied is still validated as an amount.

Every tool also publishes a human-readable title for host menus.

MCP initialisation supplies server-wide instructions for choosing tools and handling
missing facts. Every tool publishes an output schema describing its returned fields,
including verdicts, decimal strings, warnings and source information. `ok: true`
means the tool ran, not that the review passed; retain `UNKNOWN`, `REFUSED`,
`not_supplied` and `null` results when presenting findings. Engine audit fields are
preserved, and clients receive the same payload in structured content and JSON text.
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

Twenty fabricated, read-only agent evaluation questions are in
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

Amounts, including Division 7A loan balances and payments, are decimal strings, finite, at most two decimal places, and no greater than AUD 1,000,000,000,000.00. Dates are ISO-8601. Payday Super uses payday-super-checker's national SGAA 1992 s 6(1) calendar.

Payday Super dates come out of a payroll or clearing-house export, so
`calc_payday_super_deadline` reads the shapes those exports hold, through the
engine's own parser rather than one of its own: `2027-07-13`, day-first
`13/07/2027` or `13-07-2027`, `13 Jul 2027`, and a date-time with no timezone
marker, whose time the law ignores. Two are refused. A stamp carrying `Z` or a
UTC offset is the engine's refusal, because a UTC evening is already the next day
in Australia and keeping the written day could pass a receipt that was really a
day later; convert it to the Australian calendar date first. A numeric date is refused at this
boundary only where the two readings give different days, such as `01/07/2027`: it
is read day first, nothing in the text rules out the other reading, and the
difference is a month in a date that decides the verdict. Send those as
`YYYY-MM-DD`. This refusal also applies when a numeric date has a time suffix,
such as `01/07/2027 00:00`. Two things settle the reading and are accepted: a component above 12
can only be the day, so `13/07/2027` is the 13th, and equal components land on the
same date either way, so `12/12/2027` is the 12th of December. Results are always
ISO-8601.

Division 7A already takes a loan in the shape a register row holds: every
`review_div7a_loan` argument is one column of the engine's register CSV, passed
through the engine's own `GateFacts` and `MyrFacts` readers. Whole files and
multi-loan registers stay out of scope, because the engines expose those only
behind a file path and reading one here would mean this facade owning input
handling the engines do not.

## Prompts

The three documented workflows are registered as MCP prompts, so a host can offer
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
Review this Payday Super contribution. QE day, remitted date, and fund-receipt date are in the CSV. as_at is today. Do not invent an SGC charge.
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

Read `calculation_worksheets` in the scope resource for the six worksheet boundaries.
Their rules and sources come from `australian-tax-calculators`. Broader classifications,
exemptions, BAS/returns, trusts, partnerships, SMSFs, contribution caps and payroll
tax remain unsupported. Reference text cannot establish calculation support.

The evaluation includes 23 cases: ten original workflows, ten unsupported-topic
questions, grouped Payday, a tax worksheet and synthetic library retrieval.
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
and unchanged library. Reading accepts a relative `.md` path, `start_line` and
`line_count` up to 100; excerpts stop at a line boundary within 12000 characters.
Retrieval refuses traversal, hidden paths, links and Windows junctions. It reads
UTF-8 Markdown, up to 8 MB per file, 64 MB per search and 1000 files. Search reports
skipped unreadable files. No files are indexed remotely, copied into the package
or written by a tool. Returned excerpts enter the calling assistant's context.

Check section review dates, edition and relevant period before relying on a
passage. Reference text is untrusted evidence, never an instruction to call tools
or change records. Search does not certify the publisher's text as current law.
