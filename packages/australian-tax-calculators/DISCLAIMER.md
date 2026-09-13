# Disclaimer

Australian tax calculators runs 6 bounded worksheets on facts the caller has
already established: ordinary GST, resident basic income tax, CGT losses and
the discount, ordinary employer FBT, first-year depreciation and quarterly
super guarantee. It is not tax, legal, accounting, financial, investment,
BAS-agent, registered-tax-agent, or assurance advice. It is not an assessment,
a private ruling, or a determination.

This project is not affiliated with, sponsored by, endorsed by, or approved by:

- the Australian Taxation Office
- the Commonwealth of Australia
- any state or territory revenue office
- Chartered Accountants Australia and New Zealand
- the Australian Securities and Investments Commission
- any software vendor

Outputs can be wrong, incomplete, stale, or unsuitable for a given set of
facts. Rates, thresholds, effective lives, ATO guidance and administrative
practice change, and a worksheet sees only the figures and the period it is
handed. Sources were checked on 10 September 2026; a source-check date is not
an assurance that every tax rule or taxpayer circumstance has been reviewed.
Confirm every rate, threshold and consequence against the current law and the
taxpayer's facts before acting, and leave lodgement decisions with a registered
practitioner.

Every worksheet takes a `scope_confirmed` flag, and each result states the
scope it was calculated under and the exclusions that sit outside it. That
flag is the caller's assertion that those scope conditions hold. Nothing in
this package verifies eligibility, classifies a supply, tests residency,
establishes a cost base, or decides whether a benefit is a fringe benefit. A
figure calculated on an unmet scope condition is arithmetic, not an answer.

The worksheets are deliberately narrow. Unsupported periods fail rather than
extrapolate. The super guarantee worksheet does not implement the post-June
2026 Payday entitlement rules; the separate Payday engine reviews timing on a
supplied liability. Contribution caps, SMSF tax, trusts, payroll tax, HELP and
Medicare calculations remain outside these worksheets.

Nothing here lodges a return or any other official form, posts journals, pays
tax, or contacts the ATO. The package performs no network calls and no writes.
Those remain authorised human actions.

Do not publish private tax records, TFNs, client-identifying ABNs, bank
details, identity documents, client files, or other sensitive personal
information in issues, pull requests, examples, tests, or repository content.
Fixtures in this repository are fabricated.

See [LICENSE](LICENSE) for copyright and [SECURITY.md](SECURITY.md) for
vulnerability reporting.
