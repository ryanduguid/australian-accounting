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

The Python 0.2.0 suite's `test_library_evidence.py` froze the expected amounts
below. It did not call another engine to generate its expectations.
`test_calculations.py` covered the smaller examples, rounding, boundaries and
refusals. Their calls are frozen in `tests/data/python-api.json` and replayed by
`tests/parity.rs`; [tests/BEHAVIOURS.md](../tests/BEHAVIOURS.md) maps every
original test, including those named below, to its Rust coverage. Neither test
set establishes eligibility, classification or a complete tax liability.

The Python tests named in this file were removed when the engine moved to Rust
in 1.0.0. Every call they made replays from `tests/data/python-api.json` in
`tests/parity.rs`, the table and ATO sample checks are in `tests/tables.rs`,
and `tests/BEHAVIOURS.md` maps each original test to its Rust coverage.

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
| FBT car benefits | `tax-examples-fringe-benefits-tax-fbt` (tax-examples), ¶3-020 and ¶3-045 (reportable value), with the ATO's FBT return 2026 instructions (item 23), employer's guide (7.8 and 7.9) and reportable fringe benefits page | 30 June 2026; 1 April 2026 (item 23); 12 January 2023 (reportable fringe benefits) | One car's established base value or operating costs for the years ended 31 March 2026 and 2027; the reportable taxable value before the employee's $2,000 threshold and type 2 gross-up |
| Main residence | `tax-examples-capital-gains-tax-cgt` (tax-examples), ¶2-060 and ¶2-075, with the ATO's pages 'Treating former home as main residence' (QC66030) and 'Using your home for rental or business' (QC66033) | 30 June 2026; 22 June 2026 (both ATO pages) | One dwelling's taxable days for CGT events in 2024-25 to 2026-27; cost base, market value and discount excluded |
| Borrowing expenses | `tax-examples-deductions` (tax-examples), ¶4-160, with ITAA 1997 s 25-25 (compilation 268, 1 October 2026) and the ATO's 'Borrowing expenses' page | 30 June 2026; 22 May 2026 (ATO page) | One loan's established expenditure, use and dates for 2024-25 to 2026-27; classification, interest and refinancing excluded |
| Depreciation | `tax-examples-depreciation` (tax-examples), ¶6-000 and ¶6-020 | 30 June 2025 | First-year formulas and taxable-use apportionment in 2024–25 examples |
| Quarterly SG | `superannuation-instant-reference-rates-thresholds-and-checklists` (superannuation), ¶18-600 and ¶18-620 | 30 June 2026 | General 2025–26 rate and quarterly maximum contribution base |
| Contribution caps | `tax-examples-individuals` (tax-examples), ¶7-278 | 30 June 2025 | First-year bring-forward for 2024–25 only; the later-year nil cap is outside the worksheet |
| Pension minimum | `superannuation-instant-reference-rates-thresholds-and-checklists` (superannuation), ¶18-500 | 30 June 2026 | Schedule 7 age factors; the pro-rated case is adapted arithmetic |
| Medicare levy | Not a Library document: Medicare Levy Act 1986 compilation 53 and the ATO's single thresholds | 30 June 2026 (ATO page) | Single person without SAPTO, the 2% rate and the s 7 low-income reduction for 2024-25 and 2025-26 |
| Low income tax offset | Not a Library document: ITAA 1997 s 61-115 compilation 266 and the ATO's worked example | 8 June 2026 (ATO page) | Offset amount from established taxable income for 2024-25 to 2026-27; not applied against tax payable |
| Division 293 | Not a Library document: ITAA 1997 ss 293-20 and 293-25, the 2013 Imposition Act and the ATO's worked example | 24 August 2026 (ATO page) | Established Division 293 income and low tax contributions for 2024-25 to 2026-27 |
| Division 296 | `superannuation-contributions-to-superannuation-funds-and-rsas` (superannuation), ¶6-495, with the Explanatory Memorandum's examples 1.1 to 1.3, ITAA 1997 ss 296-15 to 296-55 (compilation 268, 1 October 2026), the 2026 Imposition Act and the ATO's Division 296 page | 30 June 2026; 3 November 2026 (ATO page) | One individual's established balance at 30 June 2027 and total superannuation earnings for 2026-27; working out either, s 296-70 interests and later years excluded |
| Study loan withholding | Not a Library document: ATO Schedule 8 (NAT 3539) for payments from 1 July 2026, and Schedule 8 of F2025L01016 for payments from 24 September 2025 | 17 June 2026 (published); 24 September 2025 (commenced) | Schedule 8 component on one regular weekly, fortnightly or monthly pay, added to the Schedule 1 amount |
| Study loan repayment | Not a Library document: the ATO minimum repayment income page and Gazette C2026G00249 | 13 October 2026 (ATO page) | Marginal repayment on established repayment income for 2025-26 and 2026-27; the loan-balance limit is outside the worksheet |

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
dollars without saying how cents are removed. The ATO's Calculating your FBT page
works the same $16,500/$6,000 values in its MantCo example and rounds each gross-up
to the nearest dollar, so the engine does too: 14A $34,323, 14B $11,321 (from
$11,320.80), item 15 $45,644 and item 16 $21,452.68, against the $21,452.73 estimate.
The optional rounding of item 16 down to a multiple of 5 cents is left to the
preparer.

The year ended 31 March 2027 was opened on 4 November 2026. The ATO's FBT rates
and thresholds page, updated 20 May 2026, gives 47% and the 2.0802 and 1.8868
gross-up rates for years ending 31 March 2023 to 2027, as the Fringe Benefits
Tax Rates Act 1986 s 6 and FBTAA s 5B give them. The Federal Register's 2026
Acts, to the latest made on 26 October, amend no FBT rule for that year; the
Tax Reform No. 1 Act's FBT amendments apply from 1 April 2027, and none amends
the s 8A electric car exemption. The ATO has not published the FBT return 2027
instructions (the address the 2026 instructions' pattern gives returned 'Page
not found' that day), so the items follow the 2026 instructions.
`tests/tables.rs` gives MantCo's figures for 2027.

## FBT car benefits

These worksheets were added on 29 October 2026, after the 15 September check;
their example evidence was read that day. `tests/fbt_car.rs` holds each case.

Statutory formula. The FBT return 2026 instructions, item 23A example 6, value a
$32,000 car available for 274 of 365 days: `32000 × 0.20 × 274 / 365 =
4804.383...`, presented as **$4,804.38**; the ATO writes $4,804. ¶3-020 values
Bourke Pty Ltd's $50,000 car for the full year less Tom's $1,500 fuel
contribution: `50000 × 0.20 − 1500 = 8500`. The employer's guide, example 26,
reduces $30,000 by one-third and leaves a $1,000 accessory unreduced, for a
$21,000 base value; example 23 reduces $39,000 by $13,000 and adds $4,000 of later
accessories.

Operating cost. ¶3-020 adds $5,200 of running costs, deemed depreciation of
`50000 × 0.25 = 12500` and deemed interest of `50000 × 0.0862 = 4310` for
$22,010, then applies 63.64% private use: `22010 × 0.6364 = 14007.164`, less
$1,500, presented as **$12,507.16**. Its statutory formula value of $8,500 is
lower, so under s 10(5) the worksheet's taxable value is **$8,500.00**. The
Library rounds the business use
percentage of `8000 / 22000` to 36.36%; the worksheet takes the percentage as
the operator supplies it. Item 23B example 7 applies 30% private use to $10,000
of total operating costs for **$3,000**. The guide's example 31 used 5.20% for
2020-21; the tests rerun its facts at 8.62% (`7500 + 10000 + 3448 = 20948`,
`× 0.25 − 2000 = 3237`) and 8.27% (`7500 + 10000 + 3308`, giving $3,202).

Deemed depreciation. FBTAA s 11(1AA) sets the rate at the DV percentage over the
car's effective life, in a determination in effect when the provider last became
its owner. For a car first held on or after 10 May 2006 the percentage is 200%
(ITAA 1997 s 40-72(1)). The worksheet covers cars acquired up to 31 March 2027, so
a recent car falls under the Income Tax Assessment (Effective Life of Depreciating
Assets) Determination 2025 (LI 2025/20, F2025L01097), read 6 November 2026 in the
copy retrieved from the register on 4 August 2026. A car acquired before it took
effect falls under the determination then in effect, such as TR 2022/1. Table B of
both gives 8 years for cars generally, so 25%, and 5 for rental cars and
4 for taxis, so 40% and 50%. The worksheet refuses an owned car's life under 2
years: s 11 has no cost cap, so a shorter life would deem more than the car's value.
With no depreciated value, as for a leased car or an owned car at a life of 2 after
its first full year, the life may be 0 or 2 to 1000, and deemed depreciation is nil.

Rounding is a worksheet convention: only the gross taxable value is rounded half
up to cents, before the recipient's payment. The payment is in cents, so this
moves no taxable value and keeps the applied and excess parts adding up. The
base value and deemed amounts stay unrounded until then: `50000.02` reduced by
one-third for 164 days gives $2,995.43, where a base rounded first would give
$2,995.44, and deemed amounts on $40,000 for 100 days with $500 of costs at 50%
give $2,092.19, not $2,092.20. Their displayed figures can therefore differ by a
cent from the total behind them. The ATO examples show whole dollars without
stating a method.

Reportable taxable value. The ATO's
[Reportable fringe benefits](https://www.ato.gov.au/businesses-and-organisations/hiring-and-paying-your-workers/fringe-benefits-tax/fbt-registration-lodgment-payment-and-reporting/reportable-fringe-benefits)
page (last updated 12 January 2023, read 30 October 2026) says an employer
reports an employee's fringe benefits if the total taxable value of their
reportable fringe benefits is more than $2,000 in the FBT year, and that the
reportable fringe benefits amount is that total multiplied by the lower (type 2)
gross-up rate, never the type 1 rate. An eligible electric car's private use is
exempt from FBT but still reportable, at its notional taxable value; car
benefits from the private use of pooled or shared cars are not reportable.
¶3-045 (`tax-examples-fringe-benefits-tax-fbt`, review date 30 June 2026,
currency not verified, read 30 October 2026) includes an exempt electric car's
taxable value in the reportable fringe benefits amount, leaves out its
associated car expense benefits such as registration, and applies the $2,000
threshold to the employee's total including the car. So
`reportable_taxable_value` equals `taxable_value_before_exemption`, after the
recipient's payment and, under the operating cost method, after s 10(5). The
threshold and gross-up apply to the employee's total across all their
reportable benefits, which one car's worksheet cannot know, so it applies
neither, does not decide whether the car is pooled or shared, and says so in a
third warning. `tests/fbt_car.rs` checks a taxable car, an exempt electric car
and a recipient's payment under both methods; `tests/properties.rs` checks the
equality on every seeded result.

## Main residence

This worksheet was added on 1 November 2026; its example evidence was read
that day, the ATO pages in a browser. `tests/main_residence.rs` holds each
case. Every example counts days with both ends included. Where an example gives
only a month or year, the test states the day it uses; none changes the result.

| Case | Source | Worksheet | Published |
| --- | --- | --- | --- |
| Peter: rented 1,004 of 4,564 days before moving in | QC66033 | $50,595.97 | $50,595 |
| Farnaz: 757 of 2,355 days on contract dates | QC66033 | $173,097.45 | $173,097 |
| Thomas: 35% rented throughout | QC66033 | $140,000.00 | $140,000 |
| Fatima: 40% business use; ownership restarts 1 November 2018 at market value; 1,370 of 2,739 days at 40% | QC66033 | $20,007.30 | $20,007 |
| Erin: rented after moving out, no choice | QC66033 | $46,000.00 | $46,000 |
| Roya: rented 25 years under the choice; covered 29 September 1999 to 29 September 2005; 6,940 of 9,133 days | QC66030 | $243,162.16 | $243,162 |
| Helen: 25% surgery before the move, then rented under the choice | QC66030 | $100,000.00 | $100,000 |
| Lisa and Jez: 6 years of broken rental in one absence; 2 absences | QC66030 | nil | nil |
| Stella: 1,460 of 3,652 days | ¶2-060 | $239,868.57 | $240,000, on 6 of 10 rounded years |
| Medfords: both homes from 1 November 2025; 31 of 4,749 days | ¶2-075 | $3,263.85 | $3,264 |

The 6-year limit in s 118-145(2) is 'a maximum period of 6 years'; read
strictly, it ends the day before the sixth anniversary, one day sooner than
Roya's count. Acts Interpretation Act s 36 sets no start day for it. The
worksheet follows the ATO's example.

No example settles 2 s 118-140 boundaries, so the worksheet takes the narrower
reading: the 12 months before ownership ends start the day after the same date
a year earlier, and where the same day of the month 6 months before the end does
not exist (31 August), the 6 months start on the first of the next month.

## Borrowing expenses

This worksheet was added on 4 November 2026. ITAA 1997 s 25-25 was read in
compilation 268 (1 October 2026) on the Federal Register and in the ATO's legal
database, and the ATO's Borrowing expenses page (updated 22 May 2026) that day.
`tests/borrowing_expenses.rs` holds each case, with every day counted, both
ends included. The ATO's examples give a day and a month only; the tests put
each one's Year 4, which it says is a leap year, on 2023-24.

| Case | Source | Worksheet | Published |
| --- | --- | --- | --- |
| Peter: $1,600 over 5 years from 3 July; Year 5 $321.58 × 365 ÷ 367; Year 6 $1.75 × 2 ÷ 2 | ATO page | $319.83; $1.75 | $1,598.25 claimed by the end of Year 5; $1.75 |
| Fiona and Max: $1,670 from 17 July, 170,000 ÷ 209,000 for rent; Year 5 $348.45 × 365 ÷ 381; Year 6 $14.63 × 16 ÷ 16 | ATO page | $271.53; $11.90 | $271.53; $11.90 |
| Ben: $3,600 over a 4-year loan from 1 September 2025; $3,600 × 303 ÷ 1,461, then $2,853.39 × 365 ÷ 1,158 | ¶4-160 | $746.61; $899.38 | $747; $899, in whole dollars |

Each year's maximum is rounded to cents and carried forward at that amount:
Peter's first 4 years, $318.07, $319.83, $319.82 and $320.70, leave the
$321.58 of Year 5, and the ATO's Year 6 remainder, $1.75, follows. The
Library carries whole dollars, so its later years differ by cents.

The s 25-25(4) example counts 302 days from 1 September 1997 to 30 June 1998
and 183 from 1 July to 31 December 1998, one fewer than every day both ends
included, though its 4-year period is 1,461 days, counted that way. The ATO's
examples and ¶4-160 count every day (303 days from 1 September 2025 to 30 June
2026), and the worksheet does too. Acts Interpretation Act 1901 s 15AD, as
substituted from 27 December 2011, says only that an example is not exhaustive
and may extend a provision, so it does not settle which count prevails. Where
the period ends in the year, as in the example's second year, the count does
not change the result.

The $100 test in s 25-25(6) is '$100 or less'; ¶4-160 says 'less than $100',
and the worksheet follows the Act. Money not used to produce income at all is
outside s 25-25(6)(a) and (b), so its expenditure is spread and nothing is
deducted (s 25-25(3), note). No source settles when 5 years from 29 February
end, so a loan first drawn that day is refused. Treasury Laws Amendment (Tax
Reform No. 1) Act 2026, Schedule 2 item 5, applies s 26-155 from the 2027-28
income year, so later years are refused.

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
costs after the start year and effective-life decisions in the wider chapter are
excluded.

A later year uses the asset's opening adjustable value (ITAA 1997 ss 40-70(1),
40-72 and 40-75, ATO legal database, read 13 November 2026). The ATO's 'Prime
cost (straight line) and diminishing value methods' page (last updated 28
August 2026, read the same day) gives the cases `tests/depreciation.rs` checks:
an $80,000 asset with a 5-year life declines by diminishing value $32,000, then
$19,200 on a $48,000 base, $11,520 on $28,800 and $6,912 on $17,280; by prime
cost it declines $16,000 a year. Prime cost's final year is limited to the
opening adjustable value (s 40-75(7)(b)). The closing value is the opening
value less the decline as shown in cents, so carrying it forward never writes
off more than cost.

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
`tests/tables.rs` (`every_scale_matches_the_ato_sample`) requires the worksheet
to reproduce each one.

The coefficients in `src/metadata.rs` were transcribed from the ATO page
"Coefficients to use in formulas for withholding from weekly payments", and the
earnings conversion and rounding rules from "Using a formula" and "Working out
the weekly earnings", all read on 24 September 2026. The coefficients page was
read again on 4 November 2026 and still matches all 35 rows.

2024-25 and 2025-26 use the schedule for payments from 1 July 2024 to 30 June
2026, published on 17 June 2024. Its coefficients were transcribed from that
schedule's coefficients page, read on 4 November 2026 with its "Using a
formula" and "Working out the weekly earnings" pages, whose earnings conversion
and rounding rules match the 2026 schedule's. Scale 1 has one
negative b coefficient (-0.6702), which the schedule says is intentional.
`tests/payg_withholding_sample_2024_26.csv` holds that schedule's sample data,
720 amounts at its own 48 earnings points per pay period, and
`every_scale_matches_the_ato_sample_for_payments_from_july_2024_to_june_2026`
requires the worksheet to reproduce each one for both years.

Each schedule also publishes Medicare levy adjustment sample data: scale 2 for
a spouse only and for 1 to 5 children, and scale 6 (the half-levy) for 1 to 5
children, at 48 earnings points for each of the 3 pay periods, 1,584
adjustments a schedule. The 4 files
`tests/data/medicare_levy_adjustment_scale_2_sample_2026_27.csv`,
`medicare_half_levy_adjustment_scale_6_sample_2026_27.csv` and their `_2024_26`
counterparts were transcribed from the ATO's sample data pages (2026: published
17 June 2026; 2024: last updated 17 June 2024), read on 14 November 2026, and
compared value by value with the same tables in the legislative instruments:
LI 2026/18 on the ATO legal database for 2026, and the Taxation Administration
(Withholding Schedules) Instrument 2025 (F2025L00647) for 2024. All 576 rows
matched. `tests/payg_withholding.rs` requires the worksheet to reproduce every
adjustment through `payg-withholding`, and the withholding to be the amount
before it less the adjustment, for 2026-27 and for both 2024-26 years.

The Medicare levy adjustment parameters (the weekly earnings and shade-in
thresholds, the formula 1 amount, the family threshold of $47,238, or $43,846
in 2024, and $4,338, or $4,027, for each child), the scale 4 rates, the tax
offset percentages and the additional amounts for 53 weekly or 27 fortnightly
pays were transcribed from each schedule's "Medicare levy adjustment", "Using a
formula", "Working out the weekly earnings" and coefficients pages, read on 14
November 2026; the 2026 figures were checked against LI 2026/18 the same day.
The tests hold the worksheet to each schedule's worked examples: Zail, Abe,
Wendy, Kareem, Scott and Stevi on the Medicare page, and Jabe, Prue and Tan on
the "Examples of calculating the withholding amounts" page. The Taxation
Administration (Withholding Schedules) Instrument (No. 2) 2025 (F2025L01016),
which replaced F2025L00647 from 24 September 2025, has the same Schedule 1 word
for word, so 2025-26 uses one schedule throughout.

The schedules publish no quarterly sample data or worked example. The tests
apply the quarterly rules (a thirteenth of the quarter's earnings as the weekly
equivalent; 13 times the weekly amount and adjustment; 25% of the offsets) to
13 times each weekly sample point, which must withhold 13 times the ATO's weekly
amount on every scale. Scale 4 follows note 3 to the coefficients: the rate is
applied to the pay's earnings, cents ignored in the earnings and the result. The
additional amount for 53 or 27 pays is the table's last band whose lower
limit the pay's earnings reach; each band edge has a test on each side. Schedule 1 does not say what happens when offsets and
the adjustment exceed the amount withheld before them, so the worksheet refuses
that case.

The sample data checks the
transcription; it does not establish that a payee's scale, allowances or
declarations are right. Bi-monthly, daily and casual pays, a Medicare levy
variation that increases withholding, study and training support loans and the
other schedules are outside this worksheet.

## Medicare levy

This worksheet's evidence is not a Library document. The rate and the
low-income reduction were read on 15 October 2026 in the Medicare Levy Act 1986
as compiled at 1 July 2026 (compilation 53): s 6(1) sets the rate at 2%, s 7(1)
removes the levy at or below the threshold amount, and s 7(2) limits it to 10%
of the excess up to the phase-in limit. For a person not entitled to the seniors
and pensioners tax offset, s 3(1) sets those at $28,011 and $35,013, which the
ATO page "Medicare levy reduction for low-income earners" (last updated 30 June
2026) gives for 2025-26. The 2024-25 figures, $27,222 and $34,027, are from
table 2 of the ATO's myTax 2025 instructions, which apply to 1 July 2024 to 30
June 2025. The frozen replay in `tests/parity.rs` holds the worksheet to the ATO page's
example ($29,000 in 2025-26 gives $98.90), to each threshold and phase-in limit
and to the dollar either side of them. The 2026-27 thresholds were not
published when the worksheet was written, so 2026-27 is refused rather than
given the 2025-26 figures.

Each published phase-in limit is 1.25 times the threshold amount, rounded down
to the dollar: that is the income at which 10% of the excess reaches 2% of the
whole, and the tests derive it. The family income reduction (s 8), exemptions,
the surcharge, trustees and part-year residence are outside the worksheet.

Section 9A, read in the same compilation on 15 October 2026, takes the total
arrears amount out of taxable income for ss 6(1) and 7 when a person's
assessable income includes eligible lump sums in arrears of at least 10% of
normal taxable income and the accrual-year tests in s 9A(1)(c) are met; the
terms come from ITAA 1936 s 159ZR(1). The ATO page "Lump sum payment in arrears
tax offsets" (last updated 8 June 2026, read on 15 October 2026) says the
Medicare levy exemption on the arrears amount applies from 1 July 2024. By 26
October 2026 its address redirected to the ATO page "Lump sum payment in
arrears", which the worksheet now cites; that page has not been re-read. The
worksheet takes `section_9a_adjustment_applies` as an operator-established
fact and refuses when it is true rather than estimate the adjustment. In the
tests, $40,000 for 2025-26 that includes $12,000 of arrears from two earlier
years is refused, because s 9A would leave $28,000, below the threshold, and
nil levy; the same income without s 9A gives $800.

## Low income tax offset

This worksheet's evidence is not a Library document. The table was read on 15
October 2026 in ITAA 1997 s 61-115(1) as compiled at 1 July 2026 (compilation
266, volume 2): $700 where taxable income does not exceed $37,500, $700 less 5%
of the excess up to $45,000, and $325 less 1.5% of the excess up to $66,667.
Section 61-110(1) gives the offset for 2020-21 and later income years to an
individual who is an Australian resident at any time in the year with taxable
income of no more than $66,667. The ATO page "Low income tax offset" (last
updated 8 June 2026) states the same bands, and its page on low and middle
income earner tax offsets works the $45,000 case to $325, which
the frozen replay in `tests/parity.rs` holds. `tests/tables.rs` also derives the $325
from the first band and the $66,667 cut-out as the first whole dollar at which
the third item reaches nil.

The provision states no rounding. Whole-dollar taxable income in the 1.5% band
can leave half a cent, which the worksheet rounds half up like its other
amounts. At $66,667 the third item gives minus half a cent and the worksheet
returns nil. The cap for minors in s 61-115(2), trustees under s 61-110(2) and
applying the offset against tax payable are outside the worksheet.

## Division 293

This worksheet's evidence is not a Library document. ITAA 1997 s 293-20(1), as
compiled at 1 July 2026 (compilation 266, volume 6) and read on 15 October
2026, gives taxable contributions equal to the lesser of the low tax
contributions and the excess when Division 293 income (income for surcharge
purposes disregarding reportable super contributions) and low tax contributions
together exceed $250,000; s 293-20(2) makes them nil when the low tax
contributions are nil, and s 293-25 defines low tax contributions as low tax
contributed amounts less excess concessional contributions. Section 5 of the
Superannuation (Sustaining the Superannuation Contribution Concession)
Imposition Act 2013 sets the tax at 15%. The ATO's key superannuation rates
table gives the $250,000 threshold for 2017-18 onwards (last updated 7 August
2026), and its Division 293 page (last updated 24 August 2026) works Jan's
example: $240,000 and $15,000 give taxable contributions of $5,000 and tax of
$750, which the frozen replay in `tests/parity.rs` holds.

Both inputs are the operator's established figures; the worksheet does not
build Division 293 income from its components or classify contributions.
Defined benefit interests and their deferred debt accounts, constitutionally
protected funds, State higher level office holders and Commonwealth justices
are outside it.

## Division 296

The Library's ¶6-495 (superannuation collection, 'Contributions to
Superannuation Funds and RSAs', last reviewed 30 June 2026) says Division 296
was enacted by the Treasury Laws Amendment (Building a Stronger and Fairer Super
System) Act 2026 (No. 8, 2026) and applies from 1 July 2026, and reproduces the
Explanatory Memorandum's examples. Both were checked against primary sources on
13 November 2026:

- ITAA 1997 ss 296-15 to 296-50, as compiled at 1 October 2026 (compilation
  268, the current one, read through the Federal Register API), are word for
  word as enacted, apart from the formula images the text copy omits: liability needs taxable superannuation earnings (s 296-15);
  they arise when the total superannuation balance just before the start or at
  the end of the year is greater than the large superannuation balance
  threshold and total superannuation earnings are greater than nil
  (s 296-40(1)); the percentage is the reference amount less the threshold,
  over the reference amount, times 100, rounded to 2 decimal places, rounding
  up when the third is 5 or more (s 296-40(2) and (3)); s 296-45 is the same
  with the very large threshold. The thresholds are $3,000,000 and $10,000,000
  for 2026-27 (ss 296-30 and 296-35).
- Income Tax (Transitional Provisions) Act 1997 s 296-1, inserted by the same
  Act, applies s 296-15 from 2026-27, uses only the balance at the end of
  2026-27 in both tests and formulas, and excuses anyone who dies on or before
  30 June 2027.
- The Superannuation (Building a Stronger and Fairer Super System) Imposition
  Act 2026 (No. 9, 2026) s 5 sets the tax at 15% of taxable superannuation
  earnings plus 10% of any very large superannuation balance earnings
  component.
- The Explanatory Memorandum to the Bill, read on ParlInfo, works examples 1.1
  (Jordan: $4,000,000 and $100,000 give 25% and tax of $3,750), 1.2 (Kelly:
  $12,000,000 and $500,000 give 75% and 16.67%, $56,250 and $8,335, $64,585 in
  all) and 1.3 (Kenny: balances of $150,000 at 30 June 2027 and $170,000 at 30
  June 2028 give no taxable earnings). `tests/division_296.rs` holds 1.1 and
  1.2, and holds 1.3, a 2027-28 example, as a 2026-27 balance of $150,000.
- The ATO's 'Division 296 tax on large super balances' page (last updated 3
  November 2026) gives the same thresholds and rates, disregards LRBA amounts
  in the balance (s 296-260) and lists the 3 exceptions.

The Division 296 regulations (F2026L00726) note that earnings attributed to an
interest may be negative (reg 296-65.02), so the earnings input may be negative;
earnings of nil or less give no taxable superannuation earnings and so no tax,
even above $10,000,000, where s 296-45 alone has no 'greater than nil'
condition. Its very large component is then negative, and as results carry no
negative amounts it is reported as null. An exception under s 296-20 or 296-25
removes the liability but not the earnings ss 296-40 and 296-45 define, which
the result still shows; after death the balance is taken to be nil (s 296-50),
so the percentages and earnings are nil too. Amounts stay exact until each is
rounded half up to cents for display, which can leave a displayed tax a cent from its rate applied to the
displayed earnings, and the displayed total a cent from the sum of the displayed
taxes. The statute sets no rounding for the dollar amounts; the
Memorandum's examples are whole dollars either way.

Both inputs are the operator's established figures; the worksheet does not
work out balances, fund earnings (s 296-60) or their attribution. Defined
benefit and other interests whose earnings use s 296-70 are outside it, as their
earnings use a different formula and tax on a defined benefit interest may be
deferred to a debt account, and so are departing temporary residents
(ss 296-195 to 296-205). Later income years wait for their indexed thresholds.

## Study loan withholding

This worksheet's evidence is not a Library document. The component rates in
`src/metadata.rs` were transcribed from the ATO page "Schedule 8 -
Statement of formulas for calculating study and training support loans
components" (NAT 3539, published 17 June 2026, for payments made from 1 July
2026), read on 15 October 2026, together with its rules for the weekly
equivalent of fortnightly and monthly earnings and for rounding to the dollar.
The ATO's sample data page, checked the same day, lists no Schedule 8 data. The frozen replay in `tests/parity.rs`
holds the component to the page's examples 1 to 3 ($2,608.36 weekly gives $193,
$4,409.75 fortnightly gives $260 and $10,627.88 monthly without the tax-free
threshold gives $979) and to the published starting earnings for each pay
period ($1,337, $2,674 and $5,793.67 with the tax-free threshold; $987, $1,974
and $4,277 without it).

The worksheet adds the component to the Schedule 1 amount, which comes from the
PAYG withholding worksheet above. The schedule also publishes combined
coefficients for each scale and says the two methods can differ slightly
through rounding, accepting either. The tests type in the combined scale 1 and
2 coefficients, require each to equal the Schedule 1 row plus the component
row to 4 decimal places, and require the worksheet to stay within $1 of them
for every whole-dollar weekly amount to $4,000. The published scale 3 combined
rows from $1,337 to below $2,596 omit the 30 cent constant of the Schedule 1
scale 3 row; the worksheet follows the component method and does not use them. Quarterly
pays, scale 4 and a Medicare levy variation for a spouse or dependants are
outside the worksheet.

For 2025-26 the component rates were transcribed from Schedule 8 of the
Taxation Administration (Withholding Schedules) Instrument (No. 2) 2025
(F2025L01016), as made, on the Federal Register of Legislation, read on 14
November 2026. It commenced on 24 September 2025 and repealed the
Taxation Administration (Withholding Schedules) Instrument 2025 (F2025L00647),
whose Schedule 8 worked the component as a percentage of weekly earnings.
F2025L01016's Schedule 1 is F2025L00647's word for word, and the ATO
publishes it as the schedule for payments from 1 July 2024 to 30 June 2026. The
worksheet therefore takes the Schedule 1 amount for 2025-26 from that
schedule and requires the pay date, refusing a 2025-26 pay before 24 September
2025 and any date outside the income year; `periods` starts 2025-26 on that
date. `tests/study_loan_withholding.rs` holds the worksheet to the
instrument's examples 1 to 3 ($2,608.36 weekly gives $202, $4,409.75
fortnightly gives $274 and $10,627.88 monthly without the tax-free threshold
gives $1,018), to its starting earnings for each pay period ($1,288, $2,576
and $5,581.33 with the threshold; $938, $1,876 and $4,064.67 without it), to
each row edge, and to pays on 23 and 24 September 2025. `tests/common/mod.rs`
types in the instrument's combined scale 1 and 2 coefficients, held to the
same checks as 2026-27's.

## Study loan repayment

This worksheet's evidence is not a Library document. The thresholds in
`src/metadata.rs` were transcribed from the ATO page "Study and training
loan repayment thresholds and rates" (last updated 30 June 2026, read on 27
September 2026), tables 1 (2026-27) and 2 (2025-26), and the 2026-27 minimum
repayment income of $69,528 and replacement indexable amount of $129,717 were
checked against Gazette C2026G00249 of 20 April 2026, made under HESA 2003
ss 154-10 and 154-20. The ATO has since moved and retitled the page "Study and
training loan minimum repayment income and rates" (last updated 13 October
2026), which now says "minimum repayment income" where it said "repayment
threshold". Read again on 20 October 2026, its tables and examples print the
same figures. The frozen replay in `tests/parity.rs` holds the worksheet to the
page's examples 1 and 3 ($86,380 gives $2,527.80; $254,780 gives $25,478.00)
and to every band boundary in both years.

Two published figures are derived rather than stored as the rule. The ATO
prints the base of the 17% band as $9,028 for 2026-27; 15% of ($129,717 minus
$69,528) is $9,028.35, and the tests require the printed figure to be that
amount rounded to the dollar. The page's example 2 uses the printed $9,028 and
reaches $10,276.99 for $137,064; the worksheet applies the exact base and
returns $10,277.34, and the test pins that 35-cent difference so a reviewer
sees it. The ATO's own calculators also start the 17% band from the printed
$9,028. On 7 October 2026 their HELP repayment routine, run in an ATO
calculator page, returned $9,028.35 at $129,717 of repayment income and
$9,028.17 at $129,718, where this worksheet gives $9,028.52. The calculators'
figure therefore falls by 18 cents as income rises by $1, and stays 35 cents
below the worksheet up to $186,050. For 2025-26 the printed base of $8,700 is
exact, and the two agreed at the 5 incomes checked. The published crossover to 10% of repayment income ($179,286 for
2025-26 and $186,051 for 2026-27) is the first whole-dollar income at which
10% of the whole is the smaller amount; the tests derive it from the exact base
and require it to equal the printed threshold, which the rounded base would not
give. Repayment income is the operator's established fact: the worksheet does
not compute it from its components, limit the repayment to a loan balance, or
apply the overseas levy.

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
$2.1 million) and the $500,000 carry-forward limit. The frozen replay in
`tests/parity.rs` holds the worksheet to the ATO's published bring-forward tables
for each year at every band boundary and a cent either side. The age condition follows the ATO's "Age
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
| `Tax/Deductions.md` | `d8134b72378f43b2f7c9d4312518a3d8974d2421e859e1e8e983eabd010302cb` |
| `Tax/Depreciation.md` | `395195f3422f9695fee4ea409e34b241384002091b9ea67a8bbd75197d1912e1` |
| `Superannuation/Instant Reference – Rates, Thresholds and Checklists.md` | `de94931c47597286f33e0a952ff1686cbe7120bca48def74d1a44dd388922bf6` |
