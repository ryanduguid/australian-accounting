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
| Medicare levy | Not a Library document: Medicare Levy Act 1986 compilations 52 and 53, the ATO's single and family thresholds and its Ashton example | 30 June 2026 (ATO pages) | The 2% rate, the s 7 low-income reduction at the single or SAPTO thresholds and the s 8 family reduction for 2024-25 to 2026-27, on established SAPTO, spouse and dependant facts |
| Low income tax offset | Not a Library document: ITAA 1997 s 61-115 compilation 266 and the ATO's worked example | 8 June 2026 (ATO page) | Offset amount from established taxable income for 2024-25 to 2026-27; not applied against tax payable |
| Seniors and pensioners tax offset | Not a Library document: ITAA 1936 s 160AAAA (compilation 193, 1 October 2026), the Income Tax Assessment (1936 Act) Regulations 2025 and Regulation 2015 (compilation 8) ss 9 to 12, and the ATO's examples and thresholds | 2 October 2026 (ATO page) | One eligible individual's offset amount for 2024-25 to 2026-27 from established rebate income, status and spouse facts; not applied against tax payable |
| Division 293 | Not a Library document: ITAA 1997 ss 293-20 and 293-25, the 2013 Imposition Act and the ATO's worked example | 24 August 2026 (ATO page) | Established Division 293 income and low tax contributions for 2024-25 to 2026-27 |
| Division 296 | `superannuation-contributions-to-superannuation-funds-and-rsas` (superannuation), ¶6-495, with the Explanatory Memorandum's examples 1.1 to 1.3, ITAA 1997 ss 296-15 to 296-55 (compilation 268, 1 October 2026), the 2026 Imposition Act and the ATO's Division 296 page | 30 June 2026; 3 November 2026 (ATO page) | One individual's established balance at 30 June 2027 and total superannuation earnings for 2026-27; working out either, s 296-70 interests and later years excluded |
| Termination payment | Not a Library document: ATO Schedule 11 examples 1 to 4, with ITAA 1997 ss 82-10 and 83-170 (compilation 268, 1 October 2026), SISR reg 6.01(2) and `tax-examples-assessable-income` (tax-examples), ¶1-420 | 17 June 2026 (Schedule 11); 30 June 2026 (¶1-420) | One life benefit ETP in 2026-27 with established, classified parts; Schedule 11 withholding, not the tax assessed |
| Schedule 5 withholding | Not a Library document: ATO Schedule 5 (NAT 3348) for payments from 1 July 2024, 1 July 2025 and 1 July 2026, and the 2024, 2025 and 2026 withholding schedules instruments | 17 June 2024, 10 June 2025 and 17 June 2026 (published) | One regular weekly, fortnightly or monthly pay with additional payments by Method A, B(i) or B(ii); the study loan component from 1 July 2026 only |
| Working holiday makers | Not a Library document: ATO Schedule 15 for payments from 1 July 2026, and for 1 July 2024 to 30 June 2026 | 17 June 2026 (published); 17 June 2024 (last updated) | One payment's withholding at the Table A rates or Schedule 1 scale 3 for 2024-25 to 2026-27; visa, registration and TFN status established by the operator |
| Seniors and pensioners | Not a Library document: ATO Schedule 9 (NAT 4466) for payments from 1 July 2026, and for 1 July 2024 to 30 June 2026, with Schedule 9 of the 2026 and 2025 withholding schedules instruments | 17 June 2026 (published); 17 June 2024 (last updated) | One weekly, fortnightly or monthly payment's withholding on scales 1 to 9 for 2024-25 to 2026-27; eligibility, status and exemption established by the operator; other tax offsets and the Medicare levy adjustment excluded |
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
published when the worksheet was written, so it refused 2026-27. On 14 November
2026 the Treasury Laws Amendment (Delivering an Efficient and Trusted Tax
System) Act 2026 (No. 58, 2026, assented 30 June 2026) was read: Sch 5 item 14
applies the $28,011 and $35,013 it put into s 3(1) to the 2025-26 and later
years of income. Compilation 53 was still the latest that day and still holds
them, so 2026-27 takes them as enacted law, not as borrowed figures. The ATO
page still gave 2025-26 figures only, so a 2026-27 result warns that an
amendment for that year can change it. `tests/tables.rs` checks 2026-27 below,
at and above the threshold amount and phase-in limit.

Each published phase-in limit is 1.25 times the threshold amount, rounded down
to the dollar: that is the income at which 10% of the excess reaches 2% of the
whole, and the tests derive it. Exemptions, the surcharge, trustees and
part-year residence are outside the worksheet.

### SAPTO thresholds and the family reduction

Read on 14 November 2026: the Medicare Levy Act 1986 as compiled at 28 March
2025 (compilation 52, in force for 2024-25 assessments until 1 July 2026) and
at 1 July 2026 (compilation 53); the Treasury Laws Amendment (More Cost of
Living Relief) Act 2025 (No. 28, 2025), whose Sch 2 item 14 applies its
figures to 2024-25 and later years; Act No. 58, 2026, Sch 5, whose item 14
applies its figures to 2025-26 and later years; the ATO pages "Medicare levy
reduction - family income" and "Medicare levy reduction for low-income
earners" (both last updated 30 June 2026); and myTax 2025 "Medicare levy
reduction or exemption".

| Figure | 2024-25 | 2025-26 and 2026-27 | Provision |
| --- | --- | --- | --- |
| SAPTO threshold amount | $43,020 | $44,268 | s 3(1), threshold amount (a) |
| SAPTO phase-in limit | $53,775 | $55,335 | s 3(1), phase-in limit (a) |
| Family income threshold | $45,907 | $47,238 | s 8(5) |
| With SAPTO | $59,886 | $61,623 | s 8(7) |
| Each dependent child or student | $4,216 | $4,338 | s 8(5) |

The ATO's 2025-26 pages and myTax 2025 table 2 and worksheet 2 give the same
lower figures. Section 8 then applies to a person who had a spouse on the last
day of the year (s 3(3): not one living separately and apart, though the ATO's
Ashton example counts a spouse living apart because of illness; a spouse who
died in the year counts), or who is entitled to the invalid and invalid carer tax
offset for their child (ITAA 1997 Subdivision 61-A) or the sole parent
notional tax offset (Subdivision 961-B). Family income is the person's taxable
income plus the spouse's. At or below the family income threshold no levy is
payable (s 8(1)). Above it the levy after s 7 is reduced by 2% of the
threshold less 0.08 of the excess (s 8(2)), so the reduction reaches nil where
family income passes 1.25 times the threshold. When the spouse would also be
liable, the reduction is shared in proportion to taxable income (s 8(3)), and
the part of either spouse's share above their own levy reduces the other's
(s 8(4)).

The worksheet takes SAPTO entitlement, the spouse, the s 8(1)(b) or (c)
offsets and the number of dependent children as established facts. It does not
work them out. SAPTO entitlement means an offset above nil: the family page
says being eligible for SAPTO without receiving it does not bring in the SAPTO
thresholds. Without a spouse, a child counts only if family tax benefit was
payable to the person for that child (s 8(6)).

The ATO's example on the family page holds the worksheet: Ashton, entitled
to SAPTO, has taxable income of $49,700 in 2025-26 and his spouse, also
entitled, $21,700. His levy after s 7 is 10% of $5,432, $543.20; the family
reduction is 2% of $61,623 less 0.08 of $9,777, $450.30; his levy is
**$92.90**, as the ATO's calculator gives. His spouse is below the spouse's
own threshold amount and pays nothing, so the reduction is not shared. Read
literally, s 8(3) would also give the spouse a share of the reduction, which
s 8(4) would then pass back to him, cutting his levy below $92.90. The ATO's result
shows that only a spouse liable to the levy has a share, and the worksheet
follows it: s 8(4) passes back the excess of a liable spouse only.

The ATO's upper thresholds are the threshold times 1.25: $59,047 and $77,028
for 2025-26, and $57,383 and $74,857 in myTax 2025 worksheet 2, each rounded
down. Its increase in the upper threshold for each child, $5,423 for 2025-26,
is $5,422.50 rounded up, so each child moves it 50 cents further than the
family income at which the s 8(2) reduction reaches nil, 1.25 times the family
income threshold. From 2 children the published 2025-26 upper threshold lies
past that income, and the gap grows with each child: with 2 children $69,893
has a reduction of minus 4 cents, and with 3 children $75,316 has minus 8
cents. Between the 2 points the formula gives no reduction, so the levy is the
levy after s 7. The worksheet applies the formula, and
`tests/medicare_family.rs` holds both cases to the full levy. The
same tests check each SAPTO and family threshold and the top of each shade-in
for all 3 years, the sharing, the excess passed back between spouses and every
refusal. No ATO example covers 2024-25 or 2026-27; those cases are worked from
the Act.

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

## Seniors and pensioners tax offset

This worksheet's evidence is not a Library document. Read on 14 November 2026:
ITAA 1936 ss 6(1) (rebate income) and 160AAAA as compiled at 1 October 2026
(compilation 193); the Income Tax Assessment (1936 Act) Regulations 2025
(F2025L01060, as made), whose s 23 applies them to years of income starting on
or after 1 October 2025, so from 2026-27, and keeps the Income Tax Assessment
(1936 Act) Regulation 2015 (compilation 8, in force 9 December 2022 to 30
September 2025) for earlier years; the ATO page "Seniors and pensioners tax
offset" (last updated 2 October 2026); and "Do you need to lodge a tax return?
2025". The 2 instruments' ss 4 and 9 to 12 state the same rules and figures;
the 2015 Regulation calls the 3 constants '159N' amounts.

| Rule | Provision |
| --- | --- |
| Base rebate amount: $2,230 not a spouse at any time in the year, $1,602 a spouse, $2,040 a member of an illness-separated couple; the greatest where more than one applies | regs s 9(2) and (3) |
| Rebate threshold: ($445 + base) / lowest marginal rate + tax-free threshold; above $37,000, (lowest rate x tax-free threshold + $445 + base + $37,000 x (0.015 + second lowest rate - lowest rate)) / (0.015 + second lowest rate); rounded up to the dollar | regs s 9(4) to (8) |
| Cut-out: base / 0.125 + rebate threshold, rounded up to the dollar | regs s 10 |
| Entitled when rebate income is below the cut-out; for a spouse, half the couple's rebate income | ITAA 1936 s 160AAAA(3) and (4) |
| Offset: base less 12.5 cents for each dollar of own rebate income above the threshold | regs s 11(2) |
| Unused base amount of an entitled spouse: all of it, or above $6,000 of taxable income base - 0.15 x (taxable income - $6,000), not below nil; added to the base for s 11, not s 10, when the person's offset is below its base | regs s 12 |

The lowest and second lowest marginal rates and the tax-free threshold are the
year's Income Tax Rates Act 1986 Sch 7 Part I figures, which the worksheet
reads from `RESIDENT_TAX_SCALES`: 16% and 30% with $18,200 for 2024-25 and
2025-26, and 15% and 30% for 2026-27. No threshold is stored; each follows from
the formulas.

| Status | Base | Threshold 2024-25 and 2025-26 | Cut-out | Threshold 2026-27 | Cut-out |
| --- | --- | --- | --- | --- | --- |
| Single | $2,230 | $34,919 | $52,759 | $36,034 | $53,874 |
| Illness-separated couple, each | $2,040 | $33,732 | $50,052 | $34,767 | $51,087 |
| Couple, each | $1,602 | $30,994 | $43,810 | $31,847 | $44,663 |

The ATO page gives the 2025-26 thresholds and cut-outs in this table, and "Do
you need to lodge a tax return? 2025" (reason 2) the 2024-25 thresholds. The
2026-27 figures are worked from the formulas: the ATO had not published them,
and a 2026-27 result warns so. As a check of the formula at another rate, 19%
gives the ATO's 2022-23 figures from its withholding declaration page: $32,279
and $50,119, $31,279 and $47,599, and $28,974 and $41,790.

The regulations state no rounding of the offset. The ATO page says it rounds
the offset up to the whole dollar, and its José and Vanh examples ($1,719.875
to $1,720 and $1,402.50 to $1,403) do so; the worksheet follows it. The
cut-out is the income at which the offset reaches nil, so a dollar below it
the offset is 12.5 cents, rounded up to $1. The ATO page requires at least $1
of offset for the Medicare levy SAPTO thresholds, so a result above nil is the
`sapto_entitled` fact `medicare-levy` takes.

`tests/seniors_pensioners_offset.rs` holds the worksheet to the page's 9
examples: Clare and Roy, Deb and Ivan, Ying and Li Jun, Marko, Simon, José
($1,720), Keith ($1,270), Vanh ($1,403) and example 9. Example 9 transfers the
$1,440 unused by a spouse with taxable income of $10,000 to a member of an
illness-separated couple with rebate income of $48,000; the raised base,
$3,480, brings in s 9(6), giving a threshold of $39,912 and an offset of
$2,469. The example gives no rebate income for the spouse, which the test
takes as the $10,000 taxable income. The same tests check every threshold and
cut-out above for all 3 years, the offset a dollar below and at each cut-out,
the transfer at $6,000, $6,001 and $16,680 of the spouse's taxable income, and
each refusal; `tests/properties.rs` checks that the offset never rises as
rebate income rises without a transfer.

Section 12(1)(e) lets the spouse's unused amount pass only when the person's
own offset is below its base amount. A person exactly at the threshold
therefore receives nothing and a person a dollar above receives the whole
unused amount: for a couple in 2025-26 with a spouse on nil income, the offset
is $1,602 at $30,994 and $3,204 at $30,995. The worksheet applies the
regulation as written; at those incomes the base amount and the low income tax
offset already cover the tax. Up to $6,000 of the spouse's taxable income the
worksheet passes the whole base amount under s 12(3)(b)(i), as a resident
spouse with income below the tax-free threshold has no tax payable. Above it
s 12(3)(b)(ii)'s formula reaches nil before the spouse's tax, at the 15% or 16%
lowest rate, reaches the base amount, so the worksheet never needs the
spouse's tax. A foreign resident spouse, whose unused amount uses their
marginal rate, is outside it, as are trustees (s 160AAAB), a person whose
spouse status changed in the year, the transfer of this person's own unused
amount and applying the offset against tax payable.

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

## Termination payment

This worksheet's evidence is the ATO's Schedule 11, 'Tax table for employment
termination payments' (for payments made from 1 July 2026, last updated 17 June
2026), checked with the primary sources on 14 November 2026:

- Schedule 11 sets the 2026-27 ETP cap at $270,000 and the whole-of-income cap
  at $180,000, reduced by the payer's other taxable payments to the employee in
  the year (its steps 1 to 4); applies the ETP cap alone to genuine redundancy,
  early retirement scheme, invalidity and compensation payments and the smaller
  cap to the rest; and withholds, from a life benefit ETP's taxable component,
  17% within the cap from preservation age, 32% below it and 47% above the cap,
  rounding to the nearest dollar with 50 cents up. Its examples 1 to 4 (Lloyd
  $14,400; Jane $3,200; Chris $25,600 and $18,800, $44,400; Alec $127 and
  $5,657, with the $397 excluded part reducing the gratuity's ETP cap to
  $269,603) are in `tests/termination_payment.rs`. Example 1 gives only the ETP,
  so the test holds it as a genuine redundancy payment with no completed year of
  service.
- The ATO's 'Employment termination payments' rates page (last updated 17 April
  2026) gives the same ETP cap (table 17) and the 2026-27 genuine redundancy and
  early retirement scheme limits, $13,598 and $6,801 for each complete year of
  service (table 20). These 4 figures are the rate records
  `etp-cap-life-benefit-2026-27`, `etp-whole-of-income-cap-2026-27`,
  `genuine-redundancy-base-limit-2026-27` and
  `genuine-redundancy-service-amount-2026-27`, which the root
  `tests/test_dated_rates.py` keeps equal to `TERMINATION_PAYMENT`.
- ITAA 1997, compilation 268 (1 October 2026, the current one, read through
  the Federal Register API): s 83-170 makes a genuine redundancy or early
  retirement scheme payment tax-free up to the base amount plus the service
  amount for each whole year of service, and s 82-135(e) takes that part out of
  the ETP; s 82-10(3) and (4) limit the rate on the taxable component within
  the cap to 15% from preservation age on the last day of the income year and
  30% otherwise, and set the caps; s 82-10(6) lists the excluded payments the
  whole-of-income cap does not apply to; and s 82-10(7) treats a partly
  excluded payment as its excluded part first and then the rest. The indexed
  amounts are not in the Act; s 82-160 and s 83-170(3) give the 2007-08 and
  2006-07 bases.
- Preservation age is 55 for a person born before 1 July 1960, rising a year for
  each year of birth from 1 July to 60 for a person born after 30 June 1964
  (Superannuation Industry (Supervision) Regulations 1994 reg 6.01(2),
  compilation 159, applied by ITAA 1997 s 995-1), as Schedule 11's table shows.
  So in 2026-27 the 17% rate applies to anyone born on or before 30 June 1967.

The Library's ¶1-420 (tax examples, 'Assessable Income', last reviewed 30 June
2026) works a 2025-26 genuine redundancy: $70,000 after 15 years, wholly within
the $111,380 limit. The test holds it as 2026-27, when the limit is $115,613.

The worksheet computes withholding, not the tax assessed. Two ATO texts differ
on whether the whole-of-income cap for the non-excluded part of a single ETP
counts its excluded part: Schedule 11's example 4 and step 6 of the Alec example
on the ATO's 'Working out the whole-of-income cap amount' page (last updated
24 July 2025) do not, while the Robyn example on the same page and ITAA 1997
s 82-10(5) and (7)(b) count the excluded part's taxable component in taxable
income. The worksheet follows Schedule 11 for withholding, and every result
warns that on assessment the cap uses the year's taxable income, so the tax can
differ from the amount withheld. Classifying payments, death benefit ETPs, a
tax-free component (an invalidity segment or a pre-July 83 segment, s 82-140),
earlier ETPs in the year or for the same termination (which reduce the caps,
s 82-10(4)(a) and (b)), delayed termination payments, payees without a TFN,
foreign residents and working holiday makers are outside it.

## Schedule 5

These worksheets' evidence is not a Library document. Schedule 5 (NAT 3348),
'Tax table for back payments, commissions, bonuses and similar payments', was
read on 14 November 2026 on the ATO's pages for payments from 1 July 2026
(published 17 June 2026), from 1 July 2025 to 30 June 2026 (published 10 June
2025) and from 1 July 2024 to 30 June 2025 (last updated 17 June 2024): 'About
this schedule', 'Working out the withholding amount' and 'Withholding amounts
examples'. Apart from wording ('payee' for 'employee'), the 3 schedules state
the same Methods A, B(i) and B(ii), the same 47% limit for Methods A and B(ii),
the same rule that a negative result is nil, the same 52, 26 or 12 pay periods
with no adjustment for 53 or 27 pays, and the same choice, for a commission,
bonus or similar payment for a defined period of less than 12 months, to divide
by the pay periods it relates to. The same rules (each step's cents, the 47%
limit, the nil result and the pay periods in a year) and examples are in
Schedule 5 of the Taxation Administration (Withholding Schedules) Instrument
2026 (F2026L00716), of the 2025 instrument (F2025L00647) and of the
Instrument (No. 2) 2025 (F2025L01016), which replaced it from 24 September 2025
with the same Schedule 5, and of the 2024 instrument (F2024L00664), read on
legislation.gov.au the same day.

Each schedule works 4 examples on scale 2, which `tests/schedule_5.rs` holds:

| Example | 2026 schedule | 2024 and 2025 schedules |
| --- | --- | --- |
| 1, Jarrod, Method A: $900 bonus on $1,400 a week, HELP debt | PAYG $267, $272, $260; HELP $10, $12, $104; limit $423; $364 on the bonus, $641 in all | PAYG $272, $277, $260; HELP $42, $43, $52; $312 on the bonus, $626 in all |
| 2, Robyn, Method B(i): $800 back pay over 8 weeks, HELP debt | PAYG $363 less $331, $256; HELP $55 less $40, $120; $794 in all | $1,100 then $1,200 a week: PAYG $208 less $175, $264; HELP $12 less $11, $8; $492 in all |
| 3, Laura, Method B(ii): $2,480 of arrears, $1,800 a fortnight | $212, $95, $242, $780; $992 in all | $222, $95, $252, $780; $1,002 in all |
| 4, Bradley, Methods B(i) and B(ii): $1,600 this year and $8,800 last year, monthly, FS debt | PAYG $512 and $2,808, FS $242 and $1,296, earnings $1,452 and $182; $6,492 in all | $4,600 then $5,400 a month: PAYG $512 and $2,856, FS $120 and $888, earnings $962 and $108; $5,446 in all |

The examples work the study loan component from the separate study and
training support loan tax tables; the worksheets add the Schedule 8 component
to each Schedule 1 amount, as `study-loan-withholding` does, which Schedule 5
accepts in place of separate calculations and which gives the same amounts in
every 2026 example. This crate has no Schedule 8 coefficients before 1 July
2026, so the tests hold the 2024 and 2025 examples' PAYG components only and
require a study loan in those years to be refused. Example 4 combines the 2
methods in one pay: the tests run each method and count the earnings' amount
once, as the example does.

The steps are the schedule's own. Method A ignores the cents of the earnings
before the table, so a monthly amount ending in 33 cents does not gain the cent
Schedule 1 adds ($4,346.33 a month is worked as $4,346, $598, not $602); it
ignores the cents of the share and of the result after the limit. Method B(i)
takes the amount withheld from each earlier period as the operator's figure,
nets the periods and has no limit. Method B(ii) ignores the cents of the
average and of the share, and subtracts what earlier Method B(ii) payments in
the year had withheld before the limit. The schedule leaves the choice of
method, whether a payment is a back payment, commission or bonus, the periods
it relates to and the normal earnings to the payer, and so do the worksheets.
Working holiday makers, who must use Schedule 15, payees who gave no TFN, who
have 47% or 45% withheld from the whole payment, withholding variations and
superannuation income streams are outside them.

## Working holiday makers

This worksheet's evidence is not a Library document. On 14 November 2026 the
ATO's Schedule 15 for payments from 1 July 2026 (published 17 June 2026) and
the one for 1 July 2024 to 30 June 2026 (last updated 17 June 2024) were read in
full. Their columns, Table A rates, formula, rounding rules and examples are the
same apart from the income year in Table A's heading and the examples' dates:

- A registered employer whose payee has given a TFN uses column 2 while the
  total payments made to them in the income year before this payment are less
  than $45,001, column 3 from $45,001 to $135,000, column 4 from $135,001 to
  $190,000 and column 5 above $190,000; Table A's rates are 0.15, 0.30, 0.37
  and 0.45. A registered employer whose payee has not given a TFN uses
  column 6, 45%.
- y = a × x, where x is the payment for the pay period ignoring any cents. The
  result is rounded to the nearest dollar, 50 cents up, without a preliminary
  rounding to cents; at the top marginal rate and without a TFN, cents in the
  result are ignored.
- An unregistered employer withholds at the foreign resident rates in the
  weekly, fortnightly or monthly tax tables, which the worksheet takes from
  Schedule 1 scale 3 through `payg-withholding`. Without a TFN it withholds
  45%, cents ignored: the schedule's 45% rule for a payee without a TFN does
  not depend on registration, and Schedule 1 scale 4 gives the same rate for
  a foreign resident.
- 45% applies without a TFN only if the payee has also not claimed an
  exemption from quoting it and not advised that they applied for one or made
  an enquiry with the ATO; a payee who so advised has 28 days to provide it.
  The worksheet takes `tfn_quoted` as established on that basis and leaves
  exemption claimants out of scope.
- The schedule does not apply to workers employed under the Seasonal Worker
  Programme, Pacific Labour Scheme or Pacific Australia Labour Mobility scheme.

`tests/working_holiday_maker.rs` holds the schedule's examples for each year:

| Example | Payment | Paid before | Rate | Withholding |
| --- | --- | --- | --- | --- |
| Withholding lookup tool, weekly | $680.70 | Not over $45,000 | 0.15 | $102 |
| 1, monthly | $2,825.75 | Not over $45,000 | 0.15 | 0.15 × 2,825 = 423.75, $424 |
| 2, monthly | $8,173.20 | $47,000 | 0.30 | 0.30 × 8,173 = 2,451.90, $2,452 |

The columns follow the schedule's words, so $45,000.50 paid before is column 2
('less than $45,001') and $190,000.50 column 5 ('more than $190,000'), which
Table A's rate column ('45% on each $1 over $190,000') agrees with. For
$45,000.01 to $45,000.99 the formula reads differently: payments that 'exceed
$45,000' go to Table A, whose rate column gives '30% on each $1 over $45,000';
the worksheet follows column 2, as the schedule accepts either the lookup tool
or the formula. Column 3 ends at $135,000 and column 4 starts at $135,001, and
Table A's income bands leave the same gap, which only its rate column ('37% on
each $1 over $135,000') reaches, so payments before of $135,000.01 to
$135,000.99 are refused rather than placed in either column, and the refusal
names the 37% for the payer to confirm with the ATO.

The schedule says results from its formulas may differ slightly from the
withholding lookup tool and that either is accepted. The tool's workbook for
each schedule version, opened on 14 November 2026, works out columns 2 to 4 as
the rate times the payment with cents ignored, rounded to the dollar, and
columns 5 and 6 as 45% of it with cents ignored again, at the same rates; the
worksheet does the same. The workbook's column headings ('$0 - $45,000' to
'$190,001 and over') do not choose a column for payments before of
$45,000.01 to $45,000.99, $135,000.01 to $135,000.99 or $190,000.01 to
$190,000.99. The payee's visa and residency and the employer's registration
are the operator's to establish.

## Seniors and pensioners

This worksheet's evidence is not a Library document. On 14 November 2026 the
ATO's Schedule 9 (NAT 4466), 'Tax table for seniors and pensioners', for
payments from 1 July 2026 (published 17 June 2026) and the one for payments
from 1 July 2024 to 30 June 2026 (last updated 17 June 2024) were read in full,
with each schedule's PDF and Withholding lookup tool workbook. Schedule 9 of the
Taxation Administration (Withholding Schedules) Instrument 2026 (F2026L00716)
and of the 2025 instrument (F2025L00647), whose Schedule 9 says it was last
updated on 1 July 2024, were read on legislation.gov.au the same day. Each
instrument's coefficients for scales 1 to 9 equal its ATO page's, and each
workbook's equal its page's. The 2024 instrument (F2024L00664) was not read; the
ATO page for 1 July 2024 to 30 June 2026 and the 2025 instrument give the same
coefficients for both years.

- Columns 2, 3 and 4 of the tax table are for a payee who is single, a member
  of an illness-separated couple or a member of a couple. The 'Using a formula'
  section gives scales 1 to 3 for those payees, 4 to 6 for the same payees
  claiming a full Medicare levy exemption and 7 to 9 for a half exemption;
  `status` and `medicare_levy_exemption` select one.
- y = a × x − b, where x is the weekly earnings in whole dollars plus 99 cents,
  rounded to the nearest dollar with 50 cents up. A step with no coefficients
  ('–') withholds nothing. The schedule says to read the section with Schedule
  1, whose weekly earnings rule this is.
- A fortnightly pay's weekly equivalent is half its earnings, cents ignored,
  plus 99 cents, and its amount twice the rounded weekly amount; a monthly
  pay's is 3/13 of its earnings, after adding a cent to an amount ending in 33
  cents, and its amount 13/3 of the weekly amount, rounded to the dollar. The
  schedule gives no quarterly rule, so quarterly pays are refused.
- The schedule incorporates the seniors and pensioners tax offset. Other tax
  offsets reduce the weekly amount by a value from its ready reckoner, which
  lists weekly values for some amounts claimed and says to add the values 'for
  an appropriate combination' otherwise. Different combinations can give
  different values for the same amount ($230 is $5 as $200 and $30, or $4 as
  $200, $20 and $10), so the worksheet does not take tax offsets. Each of the
  46 listed values is Schedule 1's weekly rule, 1.9% of the amount claimed
  rounded to the dollar, but Schedule 9 does not state that rule. The schedule gives Medicare levy parameters for an adjustment but
  says to work it out from the Medicare levy adjustment weekly tax table
  (NAT 1010), whose figures are Schedule 1's, so the worksheet does not take
  that either.
- A payee without a TFN has 47% or 45% withheld, as Schedule 1 scale 4 does;
  study and training support loans, the extra Schedule 13 calculation for a
  super income stream and lump sum leave loading (Schedule 5) are outside the
  worksheet.

`tests/seniors_pensioners.rs` holds the worksheet to the schedules' worked
example and to every row of the weekly tax table in each schedule's PDF:

| Schedule | $662.95 a week: single, illness-separated, couple | Weekly tax table |
| --- | --- | --- |
| From 1 July 2026 | $2, $6, $21 | $566 to $2,685, 2,120 rows, 6,360 amounts |
| 1 July 2024 to 30 June 2026 | $5, $11, $26 | $553 to $2,672, 2,120 rows, 6,360 amounts |

The tax tables cover scales 1 to 3 only. Each schedule's Withholding lookup
tool works out the exemption scales' amounts from the same coefficients but
publishes no table of them, so the tests check those against amounts worked by
hand from the coefficients, as they do the fortnightly and monthly conversions.
`tests/properties.rs` sweeps every whole-dollar weekly amount to $3,699 for each
year and status: withholding never falls as earnings rise, and a full exemption
never withholds more than a half one, nor a half one more than none.

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
