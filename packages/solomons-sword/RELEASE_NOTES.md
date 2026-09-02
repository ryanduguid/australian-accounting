# v0.1.3

This is the first PyPI release of `solomons-sword`.
The monorepo's [filtered GitHub Releases page](https://github.com/ryanduguid/australian-accounting/releases?q=solomons-sword)
is the canonical release history.

- Move the maintained source to `packages/solomons-sword` in the
  `australian-accounting` monorepo.
- Publish through `release-solomons-sword.yml`, using the monorepo's
  namespaced, attested release workflow.
- Preserve the v0.1.2 Division 6, Section 100A and Section 99B behavior.

# v0.1.2

Releases through v0.1.2 are recorded on the standalone repository's
[GitHub Releases page](https://github.com/ryanduguid/SolomonsSword/releases).
A separate changelog is intentionally not maintained.

Prepared 0.1.2 notes. No PyPI distribution has been published; this source-only
project currently installs from a clone. The statutory corrections below came
from an audit whose claims were checked against the ITAA 1936 compilation in
force from 1 July 2026 and the High Court's own citation of Bamford:

- Division 6 franking credits were allocated on two overlapping bases when dividends were streamed, distributing 150 per cent of the credit pool in the audited case. Streaming is now refused outright, because the Division 6E carve-out with Subdivisions 115-C and 207-B is not implemented and a proportionate answer would be wrong.
- The franking credit gross-up is no longer added on top of the s 95 net income share: s 207-35 ITAA 1997 already includes it in the trust's net income, so adding it counted the credits twice. It is reported separately for the s 207-45 offset.
- Allocation now carries unrounded ratios into the s 95 pool and assigns the rounding residual, so allocated shares reconcile to the net income exactly. Three equal thirds of $100,000 previously lost $10.
- Cases the model does not compute now fail closed instead of returning an empty list or a mislabelled section: no presently entitled beneficiary and nil income of the trust estate (s 99 or s 99A trustee assessment), and non-resident beneficiaries (s 98(2A) or s 98(3)).
- Entitlements outside 0 to 100 per cent are refused; a 150 per cent and negative 50 per cent pair previously allocated a negative assessable share.
- s 99B now models the s 99B(1) residency precondition, the s 99B(2)(a) proviso for corpus attributable to amounts that would have been assessable to a resident, and the s 99B(2)(b) limb; negative inputs and exemptions exceeding the receipt are refused.
- The PCG 2022/2 green zone no longer claims the s 100A(13) ordinary family dealing exception. The guideline is a compliance-resourcing stance, not a determination, and the ordinary-family-dealing field is undetermined for that zone.
- Bamford is cited correctly as Commissioner of Taxation v Bamford [2010] HCA 10; (2010) 240 CLR 481, and the streaming reference reads Subdivisions 115-C and 207-B.

Also: one version source, project URLs, mypy enforced in CI (five real errors
fixed) with Python 3.11 and 3.13 added, CodeQL, deduplicated Dependabot config,
a not-advice boundary in the README, module docstrings and CLI output. The
package release workflow is intentionally absent until publication has a named
user, a fresh index-name availability check and an explicit compatibility
contract.

Not advice. Outputs are review aids for a qualified professional, not determinations.
