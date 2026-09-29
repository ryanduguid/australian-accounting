# For a firm assessing this server

A firm that uses AI on client work may need two things from a tool like this:
supplier information for its AI register and risk assessment, and answers to the
questions it would put to any tax-research product. Both describe this server, the
[au-tax-legislation-corpus](https://github.com/ryanduguid/au-tax-legislation-corpus)
builder and the [DrDebits](https://github.com/ryanduguid/llm-tax-guardrails) guardrails
as their documentation stood on 29 September 2026, except the response times, which
date from 26 September 2026. Neither is a certification, an approval of any use or a
statement that a firm's use complies with anything.

## Supplier information for the firm's AI register

The National AI Centre's Guidance for AI Adoption (October 2025) asks organisations to
keep an AI register as part of its fourth practice, sharing essential information, and
asks developers to share technical details, test results, limitations and risks with
the organisations that deploy their systems. Items 4.1.1 and 4.3.2 of its
[implementation guidance](https://www.ai.gov.au/staying-safe-and-responsible/essential-ai-practices/guidance-ai-adoption-implementation-guidance)
set out both. The table below gives the supplier's side of that record. Its first seven
rows follow the columns of the National AI Centre's
[AI register template](https://www.ai.gov.au/staying-safe-and-responsible/essential-ai-practices/ai-systems-register),
so a firm can start those columns from them; the other rows serve the rest of its
register and its risk assessment. The firm adds the template's remaining columns (owner,
status, purpose, registered date and screening outcome) and its own use case,
deployment environment, risk and impact assessment, controls and review cycle, and may
record the server as a component of its AI assistant rather than as a register row of
its own.
`tests/test_firm_assessment.py` fails when the server's version or its evaluation and
conformance counts move without this page.

| Field | Entry |
| --- | --- |
| Name and version | Aus Accounting MCP (`aus-accounting-mcp`) 0.2.11, a local MCP server of Australian accounting tools |
| Source and updates | Ryan Duguid, the sole maintainer, under the MIT licence, with no warranty or support agreement. Each release has [release notes](../RELEASE_NOTES.md). `uvx` keeps the version it first downloaded, so name the release in the client configuration (`aus-accounting-mcp==<version>`) to control which version runs. |
| Intended use cases | Gives an assistant bounded calculations and reviews (ATO benchmarks, Payday Super timing, Division 7A s 109N and s 109E, 10 worksheets), cited retrieval from folders the firm configures, and synthetic test data |
| Known limitations and prohibited use | Not tax advice, and not for use without human review; Payday Super and Division 7A reviews are experimental; each worksheet holds only within its stated period and scope; retrieval returns point-in-time copies, and no match does not mean no rule; library search does not rank by authority |
| Foreseeable misuse and failure | A figure outside its period or scope, or a superseded provision or ruling, is relied on as current. Each worksheet holds only within its stated period and scope, a result the tools cannot establish stays `UNKNOWN` or `REFUSED`, every provision the legislation tools return carries its compilation date, and every rulings paragraph its fetch date. Client data reaching a hosted model through the host is outside the server's control: it makes no network calls, and the README asks for fabricated data whenever the host uses a hosted model. |
| Data sources and type | Structured tool arguments supplied through the host, text from a Markdown library, a legislation corpus and ATO rulings runs the firm configures, and bundled reference data. The README asks for fabricated data whenever the host uses a hosted model. |
| Key stakeholders affected | May include the firm's clients, whose tax, super and Division 7A positions can rest on a result; employees, where Payday Super timing is reviewed; and the practitioners who remain responsible for the advice |
| Contains an AI model | No. The MCP host's model calls the tools; the server runs no model. |
| Datasets and training | Bundled reference data: the ATO's 2023-24 small business benchmarks (data.gov.au, CC BY 2.5 AU), the ranges on the ATO's industry pages, and the dated rates and thresholds in the calculation engines; a worksheet result names its official sources and the date they were checked. The evaluation questions and conformance cases are fabricated. It trains and fine-tunes no model. |
| Technical requirements | Python 3.10 or later, [uv](https://docs.astral.sh/uv/) and an MCP host that runs local stdio servers |
| Network access | None once installed. The host still sends tool arguments and results to its model provider. |
| Where a person decides | Every result needs human review before consequential accounting action. `ok: true` means the tool ran, not that a review passed, and `UNKNOWN` and `REFUSED` results stand. |
| Acceptance and testing | A change reaches `main` only when the required CI checks pass. The test suite replays 33 fabricated evaluation questions, each with an exact expected answer, through a real stdio session, and 8 conformance cases for Payday Super and Division 7A. |
| Independent assurance | None. No external audit, certification or practitioner review. |
| Report a problem | A vulnerability through the server's [security policy](../SECURITY.md); a wrong result as an issue reproduced with fabricated data, never client data |
| For the firm to complete | Owner, status, purpose and business goals, registered date and screening outcome (the template's firm columns); installed version, approved uses, approved hosts and model providers, whether client data may reach a hosted model, impact and risk assessment outcome and treatment, any audit requirement, next review date |

## ISO/IEC 42001

AS ISO/IEC 42001:2023 is the Australian identical adoption of ISO/IEC 42001:2023, a
management system standard for organisations that provide or use AI systems.
Certification against it is voluntary and is carried out by external certification
bodies, not by ISO. The table above is supplier information a firm may use in its own
AI register, risk assessment or other governance records. It is not a certification,
an audit or a conformity assessment, and it does not establish that any organisation's
AI management system conforms to the standard or that this server is certified,
approved or assured under it.

## Questions for a tax-research tool

Acuity's October–December 2026 issue suggests 10 questions to put to an AI
tax-research tool. The answers below cover this server's library, legislation and
rulings tools, the corpus builder that feeds them, and DrDebits.

**Which sources and jurisdictions does it cover?** The library holds whatever Markdown
the firm puts in it. The corpus builder takes Commonwealth Acts and instruments from the
Federal Register of Legislation whose titles contain Tax, Excise, Superannuation,
Customs Tariff or Medicare Levy, so a tax title without one of those words is absent,
and no state or territory law is included. The legislation tools read legislation
only. The rulings tools read ATO Legal Database documents from the rulings runs the
firm fetches with the corpus builder's rulings stage; the server ships none. DrDebits
draws on the Tax Practitioners Board framework, APES 110, APES 220 and the AML/CTF
obligations, with a link for each source.

**How does it treat private rulings?** A private ruling protects only its applicant,
and an edited version of private advice, published without identifying details,
protects no one else. The legislation tools never return one. The library search does
not rank by authority: matches come in folder and file order, so an edited version in
the library sits beside a public ruling with nothing marking it down. Keep such
material in its own folder and cite it as background, never as the position. The
rulings tools label every paragraph with its document family, attach to every
paragraph of an edited version of private advice the caveat that nobody can rely on
it, and show that family last among results that tie on where the words sit, in a
display order they describe as a presentation choice, not a ranking of authority.
DrDebits tells the model that private rulings protect only the applicant for the ruled
scheme.

**What does it cost?** Nothing to use. The server and the corpus builder's code are
MIT-licensed, and DrDebits is licensed CC BY 4.0. There is no account or API key. The
MCP host and its model provider charge separately.

**How often is it updated?** The corpus is rebuilt when the firm runs the builder;
nothing rebuilds it on a schedule. Each provision records its compilation number and
date, and whether that compilation was current when the corpus was built. A rulings
run is a copy from the day it was fetched. Search returns a document from the run with
the latest fetch date or, between runs fetched the same day, the one whose folder name
sorts last, and a citation's `row_ref` still reads any other copy, marked as no longer
served. The library changes when the firm changes it. DrDebits
states at the top of its README when its sources were last checked. Server releases
are listed in the
[release notes](https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/RELEASE_NOTES.md).

**How is answer quality monitored?** The server's test suite replays 33 fabricated
questions, each with an exact expected answer, through a real stdio session and checks
both the answer and the tools called. That shows the tools reproduce their answers,
not that a model picks the right tool: `evaluation/tool_selection.py` scores a
recorded run for that, and no scored run is published. On 29 September 2026,
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
`sources.json` manifest supplies; without one the block is empty. Every
`search_ato_rulings` paragraph carries a `row_ref` naming its run, its docid,
source address, fetch date and page hash, and whether that run still serves the
document. The server
writes nothing to disk, so any record of what was asked is
the host's.

**What is on the roadmap?** No roadmap is published. The rulings tools read only the
documents a firm has fetched, and they do not assess whether a document is still
current, applicable or binding.

**Can a team share it?** There is no shared service. Each person runs the server on
their own machine over stdio and points it at folders they can read.

**How quickly does it respond?** Everything is read locally. A legislation search
takes about 0.5 seconds on a corpus of 946 titles, and up to about 1.6 seconds for a
word nearly every provision contains. The model's own response time depends on the
host.
