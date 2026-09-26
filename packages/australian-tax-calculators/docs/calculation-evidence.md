# Calculation evidence

The maintainer's local Library is the source of truth for this example review.
The references below identify its documents by Library document id and
numbered paragraph. Since the Library's 19 September 2026 reorganisation, each
paragraph is an extract at `_reference/<document id>/p<paragraph>-01.md`, and
`python _tools/library.py paragraph "<paragraph>" --collection <collection>`
resolves it. The former chapter paths (`Tax/<chapter>.md` and
`Superannuation/<chapter>.md`) now hold document maps and aliases, not the text.
The repository includes original arithmetic and fabricated test inputs, with
selected established amounts from Library worked examples. It does not reproduce
the Library documents or require access to them at runtime.

`tests/test_library_evidence.py` freezes the expected amounts below. It does not
call another engine to generate its expectations. `tests/test_calculations.py`
covers the smaller examples, rounding, boundaries and refusals. Neither test set
establishes eligibility, classification or a complete tax liability.

## Dates and coverage

The passages were checked locally on 15 September 2026. Their own review dates
are listed below. The existing engine source-check baseline, 10 September 2026,
is preserved in a separate record for each worksheet. The Library check adds
example evidence; it does not claim a fresh review of every supported period.

| Worksheet | Library document id (collection) and paragraphs | Document review date | Evidence boundary |
| --- | --- | --- | --- |
| GST | `tax-examples-gst-and-other-indirect-taxes` (tax-examples), ¶12-020 | 30 June 2026 | Established taxable value and the GST fraction only |
| Resident tax | `tax-examples-individuals` (tax-examples), ¶7-010, step 2 | 30 June 2025 | Basic tax for 2024–25; later offsets and levies excluded |
| Capital gains | `tax-examples-capital-gains-tax-cgt` (tax-examples), ¶2-040 and ¶2-240 | 30 June 2026 | Established ordinary gains, losses and discount entitlement for 2025–26 |
| FBT | `tax-examples-fringe-benefits-tax-fbt` (tax-examples), ¶3-000 and ¶3-020 | 30 June 2026 | Established taxable values and ordinary gross-up rates; car example ends 31 March 2026 |
| Depreciation | `tax-examples-depreciation` (tax-examples), ¶6-000 and ¶6-020 | 30 June 2025 | First-year formulas and taxable-use apportionment in 2024–25 examples |
| Quarterly SG | `superannuation-instant-reference-rates-thresholds-and-checklists` (superannuation), ¶18-600 and ¶18-620 | 30 June 2026 | General 2025–26 rate and quarterly maximum contribution base |
| Contribution caps | `tax-examples-individuals` (tax-examples), ¶7-278 | 30 June 2025 | First-year bring-forward for 2024–25 only; the later-year nil cap is outside the worksheet |
| Pension minimum | `superannuation-instant-reference-rates-thresholds-and-checklists` (superannuation), ¶18-500 | 30 June 2026 | Schedule 7 age factors; the pro-rated case is adapted arithmetic |

The resident passage does not establish the engine's 2025–26 and 2026–27 rate
coverage. Its existing 2026–27 regression and statutory source remain unchanged.
The depreciation case below applies the stored formula to the engine's supported
2025–26 worksheet; it is an adapted arithmetic case, not a Library worked example
for that year. Review the relevant period before relying on either limitation.

## GST

For an already established ordinary taxable supply, GST is one-eleventh of the
inclusive price. With $22,000 inclusive, `22000 / 11 = 2000` and the exclusive
amount is `22000 - 2000 = 20000`. This agrees with ¶12-020's calculation.
The supply classification and non-monetary consideration analysis in that
paragraph remain outside the worksheet.

The smaller existing case uses $1,100 inclusive: GST is $100, exclusive value
is $1,000. Starting with $1,000 exclusive gives `1000 × 0.10 = 100` GST and
$1,100 inclusive.

## Resident tax

Use ¶7-010's already established $144,014 taxable income for 2024–25. The basic
tax slices are `26800 × 0.16 = 4288`, `90000 × 0.30 = 27000` and
`9014 × 0.37 = 3335.18`. Their sum is **$34,623.18**. The later offset, Medicare,
credits and refund calculations in the passage are excluded.

The existing $45,000 boundary gives $4,288 at 16%. At the engine's existing
2026–27 first rate of 15%, the same arithmetic gives $4,020. The latter is a
regression of the existing rate, not a rate corroborated by this Library passage.

## Capital gains

The summary in ¶2-240 applies $8,660 losses to $8,000 non-discount gains first.
The remaining $660 reduces $21,435 discount gains to $20,775. Applying the
established 50% discount gives **$10,387.50**, with $8,660 losses used and none
remaining. The test starts after the passage's asset and cost-base decisions.
Paragraph ¶2-040 separately confirms that losses precede the discount.

The existing fabricated case starts with $100 other gains, $1,000 discount
gains and $500 combined losses. After using $100 against other gains and $400
against discount gains, `600 / 2 = 300` remains. With $1,500 losses instead,
all $1,100 gains are absorbed and $400 losses remain. These tests preserve the
engine's established loss allocation; they do not choose a taxpayer's treatment.

## FBT

For ¶3-020's established $8,500 type 1 value, `8500 × 2.0802 = 17681.70` and
`17681.70 × 0.47 = 8310.399`, presented as **$8,310.40**. For ¶3-000's established
$10,000 type 2 value, `10000 × 1.8868 = 18868` and `18868 × 0.47 = 8867.96`.
Benefit valuation and the choice of gross-up type must already be established.

The existing combined $1,000/$1,000 case gives
`(2080.20 + 1886.80) × 0.47 = 1864.49`. The existing $16,500/$6,000 case gives
`(34323.30 + 11320.80) × 0.47 = 21452.727`, presented as $21,452.73.
The estimate retains gross-up precision until final presentation.

The `return_item` figures follow the FBT return 2026 instructions instead. Their
example enters 14A as $11,000 × 2.0802 = $22,882.20, written as $22,882; item 15 is
the sum of 14A and 14B and item 16 is item 15 × 47%. The instructions show whole
dollars but do not say whether cents are dropped or rounded; the engine drops them.
For $16,500/$6,000 that gives 14A $34,323, 14B $11,320, item 15 $45,643 and item 16
$21,452.21, against the $21,452.73 estimate. Rounding to the nearest dollar would
give 14B $11,321 and item 16 $21,452.68. The optional rounding of item 16 down to a
multiple of 5 cents is left to the preparer.

## Depreciation

Paragraph ¶6-000 supplies the first-year prime cost and diminishing value
formulas; ¶6-020 explains taxable-use apportionment. For the adapted first-year
case with $12,400 established cost, 4-year life and 304 days, prime cost is
`12400 × 304 / 365 / 4 = 2581.917808...`. At full taxable use, both decline and
deduction are **$2,581.92**; closing value is **$9,818.08**. The Library presents
this decline as $2,582 in whole dollars. The test follows the engine's cents
presentation without changing the formula.

The existing fabricated $3,000, 4-year, 146-day case uses `146 / 365 = 0.4`:
prime cost decline is $300 and the deduction at 40% use is $120. Diminishing
value doubles these to $600 and $240. Decline is capped at cost. Second-element
costs, later years and effective-life decisions in the wider chapter are excluded.

## Quarterly SG

The general rate in ¶18-600 is 12% for 2025–26. Paragraph ¶18-620 caps the
quarterly earnings base at $62,500. Thus `62500 × 0.12 = 7500`. With $7,500
qualifying contributions already established, no additional contribution is
needed. The existing $80,000 earnings/$2,000 contributions case uses the same
capped base and gives `7500 - 2000 = 5500` still needed.

The Library lists a separate Norfolk Island transitional rate. It is explicitly
excluded from this general-rate worksheet. Post-June 2026 annual-base rules,
contribution timing, eligibility, earnings classification and SGC remain outside
this calculation.

## PAYG withholding

This worksheet's evidence is not a Library document. The ATO publishes sample
data for Schedule 1 (NAT 1004) so that payroll software can be checked against
it, and `tests/payg_withholding_sample_2026_27.csv` holds every value in the
weekly, fortnightly and monthly tables published on 17 June 2026: 48 earnings
points for each period on scales 1, 2, 3, 5 and 6, 720 amounts in all.
`tests/test_payg_withholding.py` requires the worksheet to reproduce each one.

The coefficients in `austaxcalc/metadata.py` were transcribed from the ATO page
"Coefficients to use in formulas for withholding from weekly payments", and the
earnings conversion and rounding rules from "Using a formula" and "Working out
the weekly earnings", all read on 24 September 2026. The sample data checks the
transcription; it does not establish that a payee's scale, allowances or
declarations are right. Scale 4, tax offsets, Medicare levy adjustments, study
and training support loans, 53 and 27 pay years, quarterly pays and the other
schedules are outside this worksheet.

## Contribution caps

In ¶7-278, Jane is 55, her total super balance at 30 June 2024 is $1.5 million
and she makes a $240,000 non-concessional contribution in 2024–25 with no earlier
bring-forward. The worksheet gives her `3 × 120000 = 360000` available, triggers
a 3-year period and leaves `360000 - 240000 = 120000`. The example's 2025–26
consequence, a nil cap at a $1.9 million balance, depends on a later year's
balance and period tracking, so it is not tested here.

The example states its 2024–25 band ceilings as $1.68 million and $1.79 million.
Those are the 2023–24 figures. The ATO's table for 2024–25 gives $1.66 million
and $1.78 million, which is `1900000 - 2 × 120000` and `1900000 - 120000` under
ITAA 1997 s 292-85(5). The engine follows the ATO table; Jane's outcome is the
same under both.

The ATO's contributions caps and non-concessional contributions cap pages,
read on 24 September 2026, supply the concessional caps ($30,000, $30,000 and
$32,500), the general transfer balance caps ($1.9 million, $2 million and
$2.1 million) and the $500,000 carry-forward limit. `tests/test_super_worksheets.py`
types in the ATO's published bring-forward tables for each year and checks every
band boundary a cent either side. The age condition follows the ATO's "Age
eligibility" section and s 292-85(3): under 75 at any time in the year. The
non-concessional cap page's bullet that refers to age on 1 July is not used.

## Pension minimum

Paragraph ¶18-500 gives the Schedule 7 factors from 4% under 65 to 14% at 95 or
more. The ATO's worked example (Thavi, pension from 1 January 2023, $250,000,
age 66) pro-rates by 181 of 365 days and rounds $3,099 up to $3,100, using the
halved 2022–23 factor. The adapted case applies the same dates a year later at
the full 5%: `250000 × 0.05 × 181 / 365 = 6198.63`, rounded to $6,200.

SISR Schedule 7 in compilation 159 (in force from 1 July 2026), read on
24 September 2026, sets the balance and age day (cl 1), pro-rates the factor
from the commencement day (cl 3), requires nothing for a pension starting on or
after 1 June (cl 4) and rounds any amount under the Schedule to the nearest $10,
an exact $5 rounding up (cl 5). Because cl 3 pro-rates the factor, the engine
rounds once, after pro-rating. The transition to retirement 10% maximum and the
other pension schedules are outside this worksheet.

## Rechecking the evidence

Resolve the numbered paragraph with the Library helper, read the extract's
period and review date, and compare the original document's digest below before
changing a case. The digests are of the pre-reorganisation chapter files, which
the Library keeps byte-for-byte in `_maintenance/original-markdown.zip` at the
paths listed; the files now at those paths are maps or aliases and will not
match. Derive expected amounts separately from the implementation. Update only
the affected worksheet's evidence and date after its review. A changed digest
means the document bytes changed; it does not establish that the rule changed.

| Original document in `_maintenance/original-markdown.zip` | SHA-256 |
| --- | --- |
| `Tax/Goods and services tax (GST) and other indirect taxes.md` | `ce2b07cd69f7d27ca8cbe2a48ed7dca562db1cd2f35dc3a7894327143bcf3a08` |
| `Tax/Individuals.md` | `311921374a9b011b35d999de38e02458a0b286472c72ea9b3420c41e89ba9a6c` |
| `Tax/Capital Gains Tax (CGT).md` | `beb27226cbb502ecec1432603fcab5d2859efa3d9a437898bdc7fe3c2870a970` |
| `Tax/Fringe Benefits Tax (FBT).md` | `770dc63fda18062c4785ea625cd772391753501e05873a4bbcc099d65e332fc8` |
| `Tax/Depreciation.md` | `395195f3422f9695fee4ea409e34b241384002091b9ea67a8bbd75197d1912e1` |
| `Superannuation/Instant Reference – Rates, Thresholds and Checklists.md` | `de94931c47597286f33e0a952ff1686cbe7120bca48def74d1a44dd388922bf6` |
