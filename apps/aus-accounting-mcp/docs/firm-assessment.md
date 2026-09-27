# For a firm assessing this server

A firm that uses AI on client work may need two things from a tool like this: an
entry for its AI register, and answers to the questions it would put to any
tax-research product. Both describe this server, the
[au-tax-legislation-corpus](https://github.com/ryanduguid/au-tax-legislation-corpus)
builder and the [DrDebits](https://github.com/ryanduguid/llm-tax-guardrails) guardrails
as their documentation stood on 26 September 2026. Neither is a certification, an
approval of any use or a statement that a firm's use complies with anything.

## AI register entry

The National AI Centre's Guidance for AI Adoption (October 2025) asks organisations to
keep an AI register as part of its fourth practice, sharing essential information.
Copy this entry into the firm's register and complete the last row.

| Field | Entry |
| --- | --- |
| System | Aus Accounting MCP (`aus-accounting-mcp`), a local MCP server of Australian accounting tools |
| Contains an AI model | No. The MCP host's model calls the tools; the server runs no model. |
| Role in an AI-assisted workflow | Gives an assistant bounded calculations and reviews (ATO benchmarks, Payday Super timing, Division 7A s 109N and s 109E, 9 worksheets), cited retrieval from folders the firm configures, and synthetic test data |
| Data handled | Tool arguments supplied through the host, a Markdown library and a legislation corpus the firm configures, and bundled reference data. The README asks for fabricated data whenever the host uses a hosted model. |
| Network access | None once installed. The host still sends tool arguments and results to its model provider. |
| Where a person decides | Every result needs human review before consequential accounting action. `ok: true` means the tool ran, not that a review passed, and `UNKNOWN` and `REFUSED` results stand. |
| Known limits | Not tax advice; Payday Super and Division 7A reviews are experimental; each worksheet holds only within its stated period and scope; retrieval returns point-in-time copies, and no match does not mean no rule; library search does not rank by authority |
| Evidence of testing | 32 fabricated evaluation questions replayed through a real stdio session in the test suite, 8 conformance cases for Payday Super and Division 7A, and the CI gates |
| For the firm to complete | Owner, approved uses, approved hosts and model providers, whether client data may reach a hosted model, next review date |

## Questions for a tax-research tool

Acuity's October–December 2026 issue suggests 10 questions to put to an AI
tax-research tool. The answers below cover this server's library and legislation
tools, the corpus builder that feeds the legislation tools, and DrDebits.

**Which sources and jurisdictions does it cover?** The library holds whatever Markdown
the firm puts in it. The corpus builder takes Commonwealth Acts and instruments from the
Federal Register of Legislation whose titles contain Tax, Excise, Superannuation,
Customs Tariff or Medicare Levy, so a tax title without one of those words is absent,
and no state or territory law is included. The legislation tools read legislation
only, not ATO rulings. DrDebits draws on the Tax Practitioners Board framework,
APES 110, APES 220 and the AML/CTF obligations, with a link for each source.

**How does it treat private rulings?** A private ruling protects only its applicant,
and an edited version of private advice, published without identifying details,
protects no one else. The legislation tools never return one. The library search does
not rank by authority: matches come in folder and file order, so an edited version in
the library sits beside a public ruling with nothing marking it down. Keep such
material in its own folder and cite it as background, never as the position. The
corpus builder's rulings stage, which this server does not read, labels every
paragraph with its document family (`EV` for edited private advice), so a later
consumer could rank it below public rulings. DrDebits tells the model that private
rulings protect only the applicant for the ruled scheme.

**What does it cost?** Nothing to use. The server and the corpus builder's code are
MIT-licensed, and DrDebits is licensed CC BY 4.0. There is no account or API key. The
MCP host and its model provider charge separately.

**How often is it updated?** The corpus is rebuilt when the firm runs the builder;
nothing rebuilds it on a schedule. Each provision records its compilation number and
date, and whether that compilation was current when the corpus was built. The library
changes when the firm changes it. DrDebits states at the top of its README when its
sources were last checked. Server releases are listed in the
[release notes](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/RELEASE_NOTES.md).

**How is answer quality monitored?** The server's test suite replays 32 fabricated
questions, each with an exact expected answer, through a real stdio session and checks
both the answer and the tools called. That shows the tools reproduce their answers,
not that a model picks the right tool: `evaluation/tool_selection.py` scores a
recorded run for that, and no scored run is published. On 26 September 2026,
DrDebits' recorded run had passed 19 of the 25 behaviour tests it covered; its
evaluation notes describe the failures and a pending rerun.

**Is user data used for training?** Nothing here trains a model or sends data
anywhere. The MCP host sends tool arguments and results to its model provider, whose
terms decide whether they are used for training.

**Is there an audit trail?** Every library match carries its relative path, line
range, preceding headings and the SHA-256 of the file. Every provision
`search_tax_legislation` returns carries the Act, section, compilation number and
date, register page, source URL and attribution. A `search_tax_rates` match carries
the Act, section, compilation number and date and register page, but not the source
URL or attribution. Every legislation response also has a `corpus` block holding
whichever of the corpus source, retrieval date and licence terms a readable
`sources.json` manifest supplies; without one the block is empty. The server
writes nothing to disk, so any record of what was asked is
the host's.

**What is on the roadmap?** No roadmap is published. The largest gap is that the
legislation tools cannot search ATO rulings, although the corpus builder can already
fetch named ones.

**Can a team share it?** There is no shared service. Each person runs the server on
their own machine over stdio and points it at folders they can read.

**How quickly does it respond?** Everything is read locally. A legislation search
takes about 0.5 seconds on a corpus of 946 titles, and up to about 1.6 seconds for a
word nearly every provision contains. The model's own response time depends on the
host.
