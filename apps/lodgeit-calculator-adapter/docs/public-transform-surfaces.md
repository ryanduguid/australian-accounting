# LodgeiT public transform interfaces

LodgeiT exposes document and accounting transform interfaces separately from this calculator adapter. The public pages inspected on 9 October 2026 show input and review patterns worth considering when an ingestion feature is requested. Processing and calculation results were not tested.

## Observed interfaces

| Public route | Visible interface | Useful design question |
| --- | --- | --- |
| [Accounting Transforms](https://transforms.lodgeit.net.au/) | Links to depreciation, journal, classification and report tools | Can the reader choose a workflow by the evidence they have? |
| [Depreciation extraction](https://transforms.lodgeit.net.au/depreciation) | A PDF chooser and a PDF-to-CSV description | How will extracted values remain traceable to the source and be reviewed before calculation? |
| [Journal transforms](https://transforms.lodgeit.net.au/journal) | Separate financial data and account classification inputs, with an epoch label | Are classification policy and reporting context explicit inputs? |
| [Asset migration](https://taxagent.lodgeit.net.au/depreciation-transforms/frontend/) | An asset register upload, required transition date, optional formula and report column picker | Can a reviewer inspect assumptions and select the evidence shown before analysing a file? |
| [Hire-purchase calculator](https://taxgenii.lodgeit.net.au/old/calculator) | Payment timing, frequency, regular payments and dated balloon payments | Does a proposed input contract distinguish each cash flow and its timing? |
| [SBRM ledger interface](https://transforms.lodgeit.net.au/sbrm/frontend/) | A LodgeiT sign-in action and claims about mapping and validation | What additional account and data authority would an integration require? |

The home page labels journal transforms as coming soon, while its linked route displays an upload form. That establishes an interface, not a working journal-generation service. The classification and report routes returned HTTP 500 and 502 respectively through local, Firecrawl and Browserbase reads. The tax-agent root displayed a loading shell. No login, upload, query or calculation was submitted.

## Applying the patterns locally

Keep extraction, classification suggestions, deterministic calculation and human review as separate steps. Retain source identity and the supplied period, currency and relevant dates. A preview should distinguish a missing value, a rejected value and a proposed mapping; a generated table should retain the evidence behind its rows. These are design requirements to assess, not guarantees supplied by the observed vendor interfaces.

The [security policy](../SECURITY.md) separates synthetic calculator trials from the [opt-in Core catalogue](core-security.md). These public transform interfaces do not extend either network boundary, define calculation rules or establish an authorised document-upload destination. A future ingestion change belongs with the existing importing component, after its contracts and supported inputs have been inspected. A change to financial calculation or account access needs the required consultation before implementation.
