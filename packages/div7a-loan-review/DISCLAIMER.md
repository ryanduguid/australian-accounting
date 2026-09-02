# Disclaimer

`div7a-loan-review` is an experimental review aid for Division 7A of Part III
of the Income Tax Assessment Act 1936. It is not tax, legal, accounting,
financial, investment, BAS-agent, registered-tax-agent, or assurance advice,
and using it creates no professional relationship.

**Its outputs are not a Division 7A determination and not an ATO assessment.**

This repository is not affiliated with, sponsored by, endorsed by, or approved
by:

- the Australian Taxation Office
- the Commonwealth of Australia
- any state or territory revenue office
- Chartered Accountants Australia and New Zealand
- Xero, SAP, LodgeiT, or any other software vendor

The author is a provisional member of Chartered Accountants Australia and New
Zealand. That membership is not an endorsement of this software by CA ANZ.
This work was written independently, in the author's own time, on the author's
own equipment.

## Division 7A outputs are experimental

Any shortfall or deemed-dividend exposure figure this engine reports is an
experimental review aid. It is not the dividend. Section 109E(2) makes the
dividend the shortfall **subject to section 109Y**, which caps the Division's
dividends at the company's distributable surplus, and section 109E(1)(d)
removes the dividend entirely where section 109Q applies. Neither is modelled.
Neither is section 109RB.

This engine does not form amalgamated loans from constituent loans, does not
decide whether a payment is a genuine repayment under section 109R, does not
compute the lodgment day under section 109D(6), and does not touch section
109C payments, section 109F forgiven debts, the interposed-entity rules in
sections 109T to 109X, or unpaid present entitlements under section 109XA.

It marks `UNKNOWN` or refuses where the supplied facts do not establish the
statutory test. An `UNKNOWN` is a finding that work remains, not a soft pass.

## Verify before relying

Outputs can be wrong, incomplete, stale, or unsuitable for a given set of
facts. Tax law, ATO guidance, rates, thresholds and administrative practice
change. The benchmark interest rate table in this repository is frozen and
hand-reviewed; it carries the date it was last checked, and it will be out of
date at some point after that date. Verify every mutable fact against the
compiled Act and current official sources before relying on an output.

## It does not act

This tool does not lodge returns, post journals, execute payments, vary loan
agreements, or send correspondence. It reads a file and prints a review.

## Data hygiene

Do not publish private tax records, TFNs, ABNs, ACNs, Medicare numbers, bank
details, identity documents, client files, or other sensitive personal
information in issues, pull requests, examples, tests, or repository content.
Every example and fixture in this repository is fabricated, and every
identifier in them is synthetic.

See [SECURITY.md](SECURITY.md) for vulnerability reporting.
