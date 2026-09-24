# v0.1.5

- Add `payg_withholding`, a Schedule 1 (NAT 1004) worksheet for one regular
  weekly, fortnightly or monthly pay from 1 July 2026 on scales 1, 2, 3, 5 or
  6. It applies the published coefficients to the whole-dollar weekly
  equivalent plus 99 cents and rounds to the dollar as the schedule directs.
  It reproduces all 720 amounts in the ATO's sample data published on
  17 June 2026. Scale 4, tax offsets, Medicare levy adjustments, study and
  training support loans, 53 and 27 pay years and quarterly pays are refused
  or out of scope.
- State in the README and disclaimer that the author is not a registered tax
  or BAS agent, and limit support to software issues reproduced with
  fabricated data.

# v0.1.4

- Compute every worksheet in its own decimal context. A caller's context no
  longer changes a result: at precision 4, resident tax on $200,000 returned
  $56,140.00 rather than $56,138.00, and the $16,500 type 1 FBT example
  returned $21,450.00 rather than $21,452.73. The caller's own context is
  left as it was found.
- Add `austaxcalc.calculations.worksheet_catalogue()`. It returns the supported
  periods with inclusive dates, required inputs and units, available methods,
  scope exclusions and a fabricated example for each worksheet, with money as
  decimal strings. The example's `scope_confirmed` value is fabricated with the
  rest of the example; for real facts it is the operator's own confirmation of
  the scope conditions.
- Hold the resident tax scale as dated data. `metadata.RESIDENT_TAX_SCALES`
  carries one scale per income year and `SUPPORTED_PERIODS["resident_tax"]` is
  derived from its keys, so an unlisted year is refused rather than borrowing
  another year's first-bracket rate. `resident_tax` reports the marginal rates
  it applied in its `rates` field, which was previously empty.
- Cite Library document ids in the calculation evidence, so the citations
  survive the Library chapter reorganisation of 19 September 2026.

# v0.1.3

- Calculate FBT using the ATO's prescribed gross-up factors. A $1,000 type 1
  taxable value produces $977.69 FBT.

# v0.1.2

- Bind the release workflow to the checked current Release Policy main commit.
  Both historical pins are rejected by GitHub Actions despite remaining readable
  through the commit API. Versions 0.1.0 and 0.1.1 produced no release artefacts.
  Calculation behaviour is unchanged.

# v0.1.1

- Use the Release Policy commit already verified by the MCP release workflow.
  GitHub could not resolve the earlier policy reference, so 0.1.0 never built or
  published. Calculation behaviour is unchanged.

# v0.1.0

- Add 6 bounded calculation worksheets with official sources, explicit periods
  and scope confirmation.
- Refuse unsupported periods and malformed amounts. Keep reference retrieval and
  MCP transport outside the calculation engine.
