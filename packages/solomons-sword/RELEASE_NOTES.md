# v0.1.9

- Breaking: `TrustResolutionSchedule` gains 2 required facts. `uses_specific_streaming` says whether the resolution streams capital gains or franked distributions; missing streaming powers are now a defect only when it does, so an ordinary proportionate resolution under a deed without streaming powers validates. `deed_resolution_deadline` is the deed's own deadline, or 30 June where the deed sets none; a resolution after an earlier deed date is late, and a later deed date does not extend 30 June. Either fact passed as `None` is reported as not established where the answer turns on it.
- Breaking: `calculate_proportionate_share` refuses a positive trust-level `net_capital_gains` or `franked_dividends`, streamed or not, because Division 6E applies to them either way and is not implemented. Franking credits supplied without franked dividends are still allocated in proportion.
- Section 100A zones follow PCG 2022/2's own scenarios. A corporate unpaid entitlement without a Division 7A loan and a parent retaining an adult child's entitlement no longer return RED on their own; each now blocks GREEN and leaves the arrangement unzoned. Red zone scenario 1 (an adult child's entitlement applied to expenses from before they turned 18) gains its own fact. GREEN needs receipt within 2 years, the rest of scenario 3A or 3B for a retained loan and no paragraph 32 exclusion; the `s100a-check` command gains a flag for each new fact.
- Migration: construct `TrustResolutionSchedule` with `uses_specific_streaming` and `deed_resolution_deadline` (30 June where the deed sets no earlier date). A Section 100A call now needs all 11 facts, not 7, to reach `GREEN` or `OUTSIDE_GREEN`. Unlike 0.1.8, results for fully supplied inputs can change: the 2 former RED triggers now leave an arrangement unzoned, and a trust with net capital gains or franked dividends is refused.
- `calculate_proportionate_share` refuses a non-finite `net_capital_gains` or `franked_dividends` with a `ValueError`.
- These changes arrived in [#300](https://github.com/ryanduguid/australian-accounting/pull/300), which holds the review record. The first two `solomons-sword/v0.1.9` tags stopped at the release gate before building: the first pointed at a commit behind `main`, and the second at a `main` commit whose push-triggered CI had skipped this package. Nothing was published from either.

# v0.1.8

- Breaking input requirements: every Section 100A fact, both Division 6 beneficiary status facts and the 3 trust resolution deed facts are now stated as `True` or `False`, or reported as not established. `BeneficiaryEntitlement.is_resident` and `ForeignTrustReceipt.beneficiary_was_resident_during_year` have no default at all, so a construction that omits either raises `TypeError`; pass them explicitly. `beneficiary_was_resident_during_year` also moves ahead of the optional exemption amounts, so a call that passed 3 or more positional arguments to `ForeignTrustReceipt` needs updating. `is_under_legal_disability`, the Section 100A facts and the resolution facts default to `None`, which the engine refuses or reports rather than reading as `False`.
- Migration: state the facts. `BeneficiaryEntitlement(name, is_resident=True, is_under_legal_disability=False, ...)` reproduces the old defaults where the operator has established them; a Section 100A call needs all 7 facts to reach `GREEN` or `OUTSIDE_GREEN`, and the command line gains a `--no-` form for each one; `validate_trust_resolution` now returns `bool | None`, where `None` means a deed fact was never established. Results for fully supplied inputs are unchanged.
- Section 99B keeps its nil exemption defaults, which give the largest assessable amount, and the result carries one caveat naming every exemption amount that arrived as nil, together with a nil s 99B(2)(a) corpus add-back, which runs the other way and leaves the whole corpus exempt. The `s99b-check` command gains `--resident-during-year` and `--not-resident-during-year`, and a run that states neither is refused instead of assessed on an assumed residency.
- On a Windows console or redirected output, a beneficiary name outside the active code page no longer ends a finished run with a `UnicodeEncodeError`. The command line reconfigures standard output and standard error with `backslashreplace`, so such a character prints as an escape; report lines are otherwise unchanged.
- `README.md` and `DISCLAIMER.md` state that Ryan Duguid is not a registered tax agent or BAS agent, and limit project support to software issues reproduced with fabricated data.
- The build backend pin moves from hatchling 1.32.0 to 1.32.3.

# v0.1.7

- Foot the reported trust-income entitlement column to the income of the trust estate: the
  rounding residual now falls on percentage shares only, and a fixed entitlement is reported
  exactly as supplied.
- Refuse a fixed entitlement that is not stated in whole cents rather than round it.
- Add seeded property tests that every allocated column foots to its pool.

## Previous release

### v0.1.6

- Leave ordinary family dealing undetermined for red-zone results and explain the compliance-review consequence.
- Correct the documented release pointer.

### v0.1.5

- Use the checked release workflow reference. The v0.1.4 tag stopped before
  building or publishing because GitHub could not load its historical workflow.
- Include the calculation corrections listed under v0.1.4.

# v0.1.4

- Reconcile allocated franking credits to the available credit pool.
- Distribute section 95 rounding adjustments across available shares so a
  positive income pool cannot give a beneficiary a negative taxable share.
  Four equal beneficiaries now share a 2-cent pool without a negative amount.

# v0.1.3

This is the first PyPI release of `solomons-sword`.
The monorepo's [filtered GitHub Releases page](https://github.com/ryanduguid/australian-accounting/releases?q=solomons-sword)
is the canonical release history.

- Move the maintained source to `packages/solomons-sword` in the
  `australian-accounting` monorepo.
- Publish through `release-solomons-sword.yml`, using the monorepo's
  namespaced, attested release workflow.
- Preserve the v0.1.2 Division 6, Section 100A and Section 99B behaviour.

# v0.1.2

Releases through v0.1.2 are recorded on the standalone repository's
[GitHub Releases page](https://github.com/ryanduguid/SolomonsSword/releases).
A separate changelog is intentionally not maintained.

Prepared 0.1.2 notes. No PyPI distribution has been published; this source-only
project currently installs from a clone. The statutory corrections below came
from an audit whose claims were checked against the ITAA 1936 compilation in
force from 1 July 2026 and the High Court's own citation of Bamford:

- Division 6 franking credits were allocated on 2 overlapping bases when dividends were streamed, distributing 150% of the credit pool in the audited case. Streaming is now refused outright, because the Division 6E carve-out with Subdivisions 115-C and 207-B is not implemented and a proportionate answer would be wrong.
- The franking credit gross-up is no longer added on top of the s 95 net income share: s 207-35 ITAA 1997 already includes it in the trust's net income, so adding it counted the credits twice. It is reported separately for the s 207-45 offset.
- Allocation now carries unrounded ratios into the s 95 pool and assigns the rounding residual, so allocated shares reconcile to the net income exactly. Three equal thirds of $100,000 previously lost $10.
- Cases the model does not compute now fail closed instead of returning an empty list or a mislabelled section: no presently entitled beneficiary and nil income of the trust estate (s 99 or s 99A trustee assessment), and non-resident beneficiaries (s 98(2A) or s 98(3)).
- Entitlements outside 0 to 100% are refused; a 150% and negative 50% pair previously allocated a negative assessable share.
- s 99B now models the s 99B(1) residency precondition, the s 99B(2)(a) proviso for corpus attributable to amounts that would have been assessable to a resident, and the s 99B(2)(b) limb; negative inputs and exemptions exceeding the receipt are refused.
- The PCG 2022/2 green zone no longer claims the s 100A(13) ordinary family dealing exception. The guideline is a compliance-resourcing stance, not a determination, and the ordinary-family-dealing field is undetermined for that zone.
- Bamford is cited correctly as Commissioner of Taxation v Bamford [2010] HCA 10; (2010) 240 CLR 481, and the streaming reference reads Subdivisions 115-C and 207-B.

Also: one version source, project URLs, mypy enforced in CI (5 real errors
fixed) with Python 3.11 and 3.13 added, CodeQL, deduplicated Dependabot config,
a not-advice boundary in the README, module docstrings and CLI output. The
package release workflow is intentionally absent until publication has a named
user, a fresh index-name availability check and an explicit compatibility
contract.

Not advice. Outputs are review aids for a qualified professional, not determinations.
