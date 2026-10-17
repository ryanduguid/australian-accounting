# Existing worksheet release handoff

This is preparation for two releases, not a publication record. No new tax
calculation is proposed here. The four existing development additions are
Schedule 8 study-loan withholding, a bounded annual Medicare levy, LITO and
Division 293. Annual study-loan repayment and payroll withholding are distinct;
the Medicare worksheet does not apply withholding variations.

## Published and candidate identities

| Component | Published | Prepared candidate |
| --- | --- | --- |
| australian-tax-calculators | 0.1.9, ten worksheets, Python 3.11+ | 0.2.0, fourteen worksheets, Python 3.14+ |
| aus-accounting-mcp | 0.4.0, pins calculators 0.1.8 | 0.5.0, pins calculators 0.2.0 |

The minor calculator bump covers both the additions already merged in source
and its existing Python-floor change. Other engine pins stay unchanged.
Preserve the reviewed calculation logic, source records, warnings and refusals,
including the missing 2026-27 Medicare thresholds and the section 9A boundary.

## Verify installed distributions

Build the candidate engine and facade wheels. Install both files in one fresh
Python 3.14 environment outside the workspace. Passing both wheel paths lets
the resolver satisfy the facade's exact future engine pin locally without
claiming that PyPI already contains it. Do not use editable installs or uv
workspace overrides for this check.

Run the existing engine smoke check, adjusted to the candidate version, and
`scripts/check_installed_worksheets.py` with that environment's Python and `-I`.
The latter requires installed packages, verifies all four first-party dependency
pins, exercises every advertised worksheet and period through registered MCP
tools, and checks returned engine versions and warnings. It must also pass
before a future facade publication; the historical BAS demonstration alone
does not exercise the new kinds.

## Publication order, after explicit authorisation

1. Update engine version, release notes, smoke expectation and standalone/root
   locks for 0.2.0. Require its existing checks, source review and release gates.
2. Publish the engine through its namespaced workflow and verify the exact
   wheel, source archive and PyPI metadata. Never reuse or move an earlier tag.
3. Update the facade to 0.5.0, its exact engine pin, citation, fourteen-worksheet
   description and release notes. Regenerate the standalone facade lock against
   the engine now on PyPI, then regenerate the workspace lock. The standalone
   lock cannot honestly be finalised against an unpublished PyPI version.
4. Prepare the matching `server.json` and `compatibility.json` in the release
   handoff. Until then, retain their verified published 0.4.0 records. Repeat
   installed-wheel verification using the actual published engine.
5. Publish the facade, verify PyPI, then pass the existing Registry preflight
   before publishing its exact Registry record. Update website and plugin pins
   only after those artefacts exist. Historical examples keep their old pins.

If any check fails, retain the currently published versions and the failure
evidence. A passing workspace suite or locally supplied future wheel is not a
successful PyPI resolution, Registry publication or practitioner verdict.
