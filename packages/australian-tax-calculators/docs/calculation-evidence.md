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
| Foreign resident tax | Not a Library document: Income Tax Rates Act 1986 Schedule 7 Part II (compilations 63, 65 and 66) and the ATO's foreign resident rates page | 1 June 2026 (ATO page) | Basic tax for a full-year foreign resident for 2024-25 to 2026-27; no Medicare levy or low income tax offset applies |
| Capital gains | `tax-examples-capital-gains-tax-cgt` (tax-examples), ¶2-040 and ¶2-240 | 30 June 2026 | Established ordinary gains, losses and discount entitlement for 2025–26 |
| FBT | `tax-examples-fringe-benefits-tax-fbt` (tax-examples), ¶3-000 and ¶3-020 | 30 June 2026 | Established taxable values and ordinary gross-up rates; car example ends 31 March 2026 |
| FBT car benefits | `tax-examples-fringe-benefits-tax-fbt` (tax-examples), ¶3-020 and ¶3-045 (reportable value), with the ATO's FBT return 2026 instructions (item 23), employer's guide (7.8 and 7.9) and reportable fringe benefits page | 30 June 2026; 1 April 2026 (item 23); 12 January 2023 (reportable fringe benefits) | One car's established base value or operating costs for the years ended 31 March 2026 and 2027; the reportable taxable value before the employee's $2,000 threshold and type 2 gross-up |
| FBT loans | `tax-examples-fringe-benefits-tax-fbt` (tax-examples), ¶3-100, with FBTAA ss 18, 19 and 136(1) (compilation 97, 1 July 2026), the FBT return 2026 instructions (item 23C, example 8), the employer's guide (8.5 and 8.8) and the FBT rates and thresholds page | 30 June 2026; 20 May 2026 (rates page) | One loan to a current employee for the years ended 31 March 2026 and 2027; deciding the loan, exemptions and the deduction itself excluded |
| FBT meal entertainment | `tax-examples-fringe-benefits-tax-fbt` (tax-examples), ¶3-220, with FBTAA ss 37AB to 37CD (compilation 97) and the FBT return 2026 instructions (item 23P, example 18) | 30 June 2026 | The year's established expenditure by the 50/50 split or a 12 week register, checked against its age and s 37CD(3)'s 20% rule, for the years ended 31 March 2026 and 2027; the actual method excluded |
| FBT car parking | `tax-examples-fringe-benefits-tax-fbt` (tax-examples), ¶3-055 (the 2026 threshold), with FBTAA ss 39A to 39GF (compilation 97), the employer's guide (chapter 16, examples 12, 13, 15, 17 and 18; 16.2.5 and 16.2.8 last updated November 2023) and the FBT rates and thresholds page | 30 June 2026; 20 May 2026 (rates page) | Commercial parking station, market value, average cost, spaces and 12 week register values for the years ended 31 March 2026 and 2027; the register's 10% rule left to the operator |
| FBT debt waivers | FBT return 2026 instructions (item 23D, example 9), with FBTAA ss 14 and 15 (compilation 97, 1 July 2026) and the employer's guide (8.1 and 8.2) | 1 April 2026 (item 23); 29 May 2024 (guide) | The amount waived for the years ended 31 March 2026 and 2027; deciding the waiver excluded |
| FBT expense payments | `tax-examples-fringe-benefits-tax-fbt` (tax-examples), ¶3-240 and ¶3-340, with FBTAA ss 20 to 24, 62 and 136(1) (compilation 97), the FBT return 2026 instructions (item 23E, example 10) and the employer's guide (9.2 to 9.4 and 19.5) | 30 June 2026; 1 April 2026 (item 23); 29 May 2024 (guide) | One external, in-house property or in-house residual expense payment for the years ended 31 March 2026 and 2027; the in-house otherwise deductible rule excluded |
| FBT property | `tax-examples-fringe-benefits-tax-fbt` (tax-examples), ¶3-260, with FBTAA ss 40 to 44, 62 and 136(1) (compilation 97), the FBT return 2026 instructions (item 23K, example 14) and the employer's guide (17.2 to 17.5 and 19.5) | 30 June 2026; 1 April 2026 (item 23); 29 May 2024 (guide) | One benefit under an in-house or external rule for the years ended 31 March 2026 and 2027; airline transport and car property rules excluded |
| FBT residual benefits | Not a Library document: FBTAA ss 45 to 52 and 62 (compilation 97), the FBT return 2026 instructions (item 23M, example 16) and the employer's guide (chapter 18) | 1 April 2026 (item 23) | One benefit's in-house or external value for the years ended 31 March 2026 and 2027; car residual, airline transport and jointly provided benefits excluded |
| FBT housing | `tax-examples-fringe-benefits-tax-fbt` (tax-examples), ¶3-120, with FBTAA ss 25 to 28 (compilation 97), the FBT return 2026 instructions (item 23F, example 11), the employer's guide (chapter 10) and the FBT rates and thresholds page (housing indexation factors) | 30 June 2026; 20 May 2026 (rates page) | Market value and statutory annual values for the years ended 31 March 2026 and 2027; remote area housing excluded |
| FBT living-away-from-home allowances | `tax-examples-fringe-benefits-tax-fbt` (tax-examples), ¶3-160 and ¶3-180, with FBTAA ss 30 to 31H and 136(1) (compilation 97), the FBT return 2026 instructions (item 23G, example 12) and the employer's guide (chapter 11) | 30 June 2026 | One allowance's established components and expenses for the years ended 31 March 2026 and 2027; eligibility, declarations and the reasonable food amount established by the operator |
| FBT board | Not a Library document: FBTAA ss 35 to 37 (compilation 97), the FBT return 2026 instructions (item 23J, example 13) and the employer's guide (chapter 13) | 1 April 2026 (item 23) | Established meals for the years ended 31 March 2026 and 2027 |
| FBT tax-exempt body entertainment | Not a Library document: FBTAA ss 38 and 39 (compilation 97), the FBT return 2026 instructions (item 23L, example 15) and the employer's guide (chapter 15) | 1 April 2026 (item 23) | Established expenditure for the years ended 31 March 2026 and 2027, actual or per head |
| FBT entertainment classifier | Not a Library document: ITAA 1997 Division 32, ss 27-5 and 995-1 (compilation 268), FBTAA ss 5E, 20, 37A to 37CF, 40, 41, 45, 58P, 135P and 136(1) (compilation 97), the FBT Assessment Regulations 2018 regs 6 to 11, ITAA 1936 ss 51AEA and 51AEB (compilation 193), GST Act s 69-5 (compilation 96), and TR 97/17 and TR 2007/12 as the Commissioner's view | 16 November 2026 (read) | One ordinary taxable employer's entertainment lines with established facts for the years ended 31 March 2026 and 2027; no Subdivision 32-B exception, tax-exempt body or reportable fringe benefits amount |
| Main residence | `tax-examples-capital-gains-tax-cgt` (tax-examples), ¶2-060 and ¶2-075, with the ATO's pages 'Treating former home as main residence' (QC66030) and 'Using your home for rental or business' (QC66033) | 30 June 2026; 22 June 2026 (both ATO pages) | One dwelling's taxable days for CGT events in 2024-25 to 2026-27; cost base, market value and discount excluded |
| Borrowing expenses | `tax-examples-deductions` (tax-examples), ¶4-160, with ITAA 1997 s 25-25 (compilation 268, 1 October 2026; read again 19 November 2026) and the ATO's 'Borrowing expenses' page | 30 June 2026; 22 May 2026 (ATO page) | One loan's established expenditure, use and dates for 2017-18 to 2027-28; classification, interest and refinancing excluded |
| Borrowing expenses schedule | `tax-examples-deductions` (tax-examples), ¶4-160, with ITAA 1997 ss 25-25, 26-50 and 26-155 and the Acts Interpretation Act 1901 ss 2G and 36 (compilation 39) | 30 June 2026; 22 May 2026 (ATO page) | Several loans first drawn from 1 July 2017, every year of each; use by year and refinancing established by the operator; s 26-155 flagged, not applied |
| Hire purchase schedule | Not a Library document: ITAA 1997 ss 240-17, 240-55 and 240-60 on the ATO's legal database and in compilation 268 (1 October 2026) | 15 November 2026 (read) | One agreement's monthly notional interest for the hirer, starting in 2024-25 to 2026-27, from established payments; apportionment to income years under s 240-60(2) excluded |
| Balancing adjustment | Not a Library document: the ATO's Guide to depreciating assets 2026 (Bridget, Andrew and Louise examples) and ITAA 1997 Subdivisions 27-B and 40-D, ss 104-240, 118-5 and 118-10(1) and its example (compilation 268, 1 October 2026) | 1 June 2026 (Guide); 18 November 2026 (read) | One depreciating asset's balancing adjustment, s 40-290 reduction and K7 amount for events in 2024-25 to 2026-27, from established cost, termination value and each year's decline and deduction; pools, s 40-291A and roll-overs excluded |
| CGT parcels | `tax-examples-capital-gains-tax-cgt` (tax-examples), ¶2-240, with the ATO's individual tax return instructions supplement 2026, question 18 (Kathleen's examples 1 to 4), 'Keeping records of shares and units' (Boris's example), 'CGT discount', TD 33 and ITAA 1997 compilation 264 | 30 June 2026; 30 May, 22 June and 29 June 2026 (ATO pages); 19 November 2026 (read) | Parcels of shares or units acquired after 21 September 1999 and sold in 2025-26 or 2026-27; cost base adjustments, indexation and the small business concessions excluded |
| Trade-in | Not a Library document: the ATO's 'Purchasing a motor vehicle' (Darren's example) and 'GST and the disposal of capital assets' (Ian's example), with ITAA 1997 Subdivision 27-B and ss 40-180 to 40-230, 40-305 and 40-365 (compilation 268) and GST Act ss 9-75, 11-30, 69-10 and 132-5 (compilation 96) | 3 June 2026 and 24 May 2017 (ATO pages); 18 November 2026 (read) | One asset traded in at arm's length for a replacement first held and used in 2025-26 or 2026-27; GST status, credits, allowance and price established; involuntary disposals and luxury car tax on the trade-in excluded |
| Payroll tax | Not a Library document: each revenue office's rates page and examples, with each jurisdiction's payroll tax Act and the ACT's DI2026-151 | 17 July 2024 to 29 September 2026 (revenue office pages); 19 November 2026 (read) | One employer outside a group paying wages all year in one jurisdiction in 2026-27; taxable and Australian wages established; exemptions, rebates, discounts, caps and periodic returns excluded |
| Depreciation | `tax-examples-depreciation` (tax-examples), ¶6-000 and ¶6-020 | 30 June 2025 | First-year formulas and taxable-use apportionment in 2024–25 examples |
| Quarterly SG | `superannuation-instant-reference-rates-thresholds-and-checklists` (superannuation), ¶18-600 and ¶18-620 | 30 June 2026 | General 2025–26 rate and quarterly maximum contribution base |
| Contribution caps | `tax-examples-individuals` (tax-examples), ¶7-278, with ITAA 1997 s 291-20 (compilation 268, 1 October 2026) and the ATO's contributions caps and concessional contributions cap pages for the carry-forward by year | 30 June 2025; 11 September and 2 July 2026 (ATO pages) | First-year bring-forward for 2024–25 only; the later-year nil cap is outside the worksheet; unused concessional cap from 2018-19 worked out from each year's established contributions |
| Pension minimum | `superannuation-instant-reference-rates-thresholds-and-checklists` (superannuation), ¶18-500 | 30 June 2026 | Schedule 7 age factors; the pro-rated case is adapted arithmetic |
| Medicare levy | Not a Library document: Medicare Levy Act 1986 compilations 52 and 53, ITAA 1936 ss 251T and 251U, the ATO's single and family thresholds, its Ashton example and its Priya exemption example | 30 June 2026 (threshold pages); 22 June 2026 (exemption pages) | The 2% rate, the s 7 low-income reduction at the single or SAPTO thresholds, the s 8 family reduction and the s 9 exemption days for 2024-25 to 2026-27, on established SAPTO, spouse, dependant and exemption facts |
| Medicare levy surcharge | Not a Library document: Medicare Levy Act 1986 ss 8B and 8D (compilations 52 and 53), the Medicare Levy Surcharge - Fringe Benefits Act 1999 (compilations 41 and 42), the ATO's thresholds page with its Tom example and its M2 2026 instructions with the Josh, Jacinta and Jill and Kevin examples | 22 June 2026 (thresholds page); 30 May 2026 (M2) | The tier, rate and share of the year for 2024-25 to 2026-27, for a person single without dependants or married all year, on established income for surcharge purposes, days without cover and spouse's income |
| Low income tax offset | Not a Library document: ITAA 1997 s 61-115 compilation 266 and the ATO's worked example | 8 June 2026 (ATO page) | Offset amount from established taxable income for 2024-25 to 2026-27; not applied against tax payable |
| Seniors and pensioners tax offset | Not a Library document: ITAA 1936 s 160AAAA (compilation 193, 1 October 2026), the Income Tax Assessment (1936 Act) Regulations 2025 and Regulation 2015 (compilation 8) ss 9 to 12, and the ATO's examples and thresholds | 2 October 2026 (ATO page) | One eligible individual's offset amount for 2024-25 to 2026-27 from established rebate income, status and spouse facts; not applied against tax payable |
| Lump sum arrears tax offset | Not a Library document: ITAA 1936 ss 159ZR to 159ZRD (compilation 193, 1 October 2026), the ATO's "Lump sum payment in arrears" examples, its resident rates for 2012-13 to 2023-24 and Income Tax (Transitional Provisions) Act 1997 s 4-11 (compilation 104) | 8 June 2026 (ATO page) | One full-year resident individual's offset for a year of receipt from 2013-14, from established eligible arrears by accrual year; recent accrual years from 2012-13; no Medicare levy or surcharge relief; not applied against tax payable |
| Division 293 | Not a Library document: ITAA 1997 ss 293-20 and 293-25, the 2013 Imposition Act and the ATO's worked example | 24 August 2026 (ATO page) | Established Division 293 income and low tax contributions for 2024-25 to 2026-27 |
| Division 296 | `superannuation-contributions-to-superannuation-funds-and-rsas` (superannuation), ¶6-495, with the Explanatory Memorandum's examples 1.1 to 1.3, ITAA 1997 ss 296-15 to 296-55 (compilation 268, 1 October 2026), the 2026 Imposition Act and the ATO's Division 296 page | 30 June 2026; 3 November 2026 (ATO page) | One individual's established balance at 30 June 2027 and total superannuation earnings for 2026-27; working out either, s 296-70 interests and later years excluded |
| Termination payment | Not a Library document: ATO Schedule 11 examples 1 to 4, with ITAA 1997 ss 82-10 and 83-170 (compilation 268, 1 October 2026), SISR reg 6.01(2) and `tax-examples-assessable-income` (tax-examples), ¶1-420 | 17 June 2026 (Schedule 11); 30 June 2026 (¶1-420) | One life benefit ETP in 2026-27 with established, classified parts; Schedule 11 withholding, not the tax assessed |
| Unused leave payment | Not a Library document: ATO Schedule 7 for payments from 1 July 2026 and its example, the ATO's 'Withholding from unused leave payments on termination of employment' examples, ITAA 1997 ss 83-10 to 83-115 (compilation 268, 1 October 2026), the STP Phase 2 employer reporting guidelines, the Superannuation Guarantee (Administration) Act 1992 (compilation 78, 1 July 2026) and the ATO's qualifying earnings table | 17 June 2026 (Schedule 7 published); 1 May 2020 (examples); 2 September 2026 (qualifying earnings) | Withholding, assessable shares and labels for unused leave paid on one termination in 2026-27, from established accrual parts |
| Schedule 5 withholding | Not a Library document: ATO Schedule 5 (NAT 3348) for payments from 1 July 2024, 1 July 2025 and 1 July 2026, and the 2024, 2025 and 2026 withholding schedules instruments | 17 June 2024, 10 June 2025 and 17 June 2026 (published) | One regular weekly, fortnightly or monthly pay with additional payments by Method A, B(i) or B(ii); the study loan component for pays from 24 September 2025, and from 1 July 2026 in Method B(i) |
| Working holiday makers | Not a Library document: ATO Schedule 15 for payments from 1 July 2026, and for 1 July 2024 to 30 June 2026 | 17 June 2026 (published); 17 June 2024 (last updated) | One payment's withholding at the Table A rates or Schedule 1 scale 3 for 2024-25 to 2026-27; visa, registration and TFN status established by the operator |
| Seniors and pensioners | Not a Library document: ATO Schedule 9 (NAT 4466) for payments from 1 July 2026, and for 1 July 2024 to 30 June 2026, with Schedule 9 of the 2026 and 2025 withholding schedules instruments | 17 June 2026 (published); 17 June 2024 (last updated) | One weekly, fortnightly or monthly payment's withholding on scales 1 to 9 for 2024-25 to 2026-27, with any study loan component under Schedule 8 for 2026-27 and from 24 September 2025; eligibility, status and exemption established by the operator; other tax offsets and the Medicare levy adjustment excluded |
| PAYG gross-up | Not a Library document: ATO Schedule 1 (NAT 1004) sample data for payments from 1 July 2026, and for 1 July 2024 to 30 June 2026, through the PAYG withholding worksheet | 17 June 2026 (published); 17 June 2024 (last updated) | The gross of one regular pay whose Schedule 1 withholding leaves an established net; refused where more than one gross does |
| Study loan withholding | Not a Library document: ATO Schedule 8 (NAT 3539) for payments from 1 July 2026, and Schedule 8 of F2025L01016 for payments from 24 September 2025 | 17 June 2026 (published); 24 September 2025 (commenced) | Schedule 8 component on one regular weekly, fortnightly, monthly or quarterly pay, added to the Schedule 1 amount |
| Study loan repayment | Not a Library document: [the ATO's Study and training loan repayment thresholds and rates](https://www.ato.gov.au/tax-rates-and-codes/study-and-training-loan-minimum-repayment-income-and-rates), tables 1 and 2 and examples 1 and 2, and Gazette C2026G00249 | 30 June 2026 (ATO page) | Marginal repayment on established repayment income for 2025-26 and 2026-27; the loan-balance limit is outside the worksheet |
| Small business CGT concessions | `tax-examples-capital-gains-tax-cgt` (tax-examples), ¶2-280 and ¶2-303, with ITAA 1997 compilation 264 | 30 June 2026; 19 November 2026 (read) | Conditions for CGT event A1 in 2025-26 or 2026-27 of an asset other than a share, trust interest or partnership right; amounts excluded |
| Income tests | Not a Library document: ITAA 1997 s 995-1 and s 293-20 (compilation 268), ITAA 1936 s 6(1) (compilation 193), HESA s 154-5 (compilation 105), Family Assistance Act Sch 3 (compilation 124), FBTAA ss 135N to 135Q, the co-contribution Act s 8 (compilation 28) and the ATO's Christina and Tom examples | 16 November 2026 (read); 13 October 2026 and 22 June 2026 (ATO pages) | 6 income measures of a full-year resident for 2024-25 to 2026-27 from established, classified components; the thresholds and tests they feed are outside it |

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

## GST margin scheme

Added on 19 November 2026. Its sources were read that afternoon, 2.20 pm to
2.40 pm AEDT, from the ATO's legal database: the A New Tax System (Goods and
Services Tax) Act 1999 ss 9-90, 75-5, 75-10, 75-11, 75-12, 75-14, 75-15,
75-20, 75-35 and the s 195-1 definitions of 'approved valuation', 'long-term
lease' and 'margin scheme'; Schedule 1 to the Taxation Administration Act 1953
s 14-250 (GST at settlement, which it does not work out); GSTR 2006/8 (paras 15
to 39F: the written agreement, a conditional agreement, ineligible supplies,
the time of supply, the Brady King contracts and examples 1 and 2); and the
ATO's 'GST and the margin scheme' (last updated 19 August 2021). The Library's
GST collection, Real Estate, Accommodation and Sale of Business, 11-100
(reviewed 29 July 2025) and 11-110 (reviewed 27 May 2026), and its Tax
Examples, GST, 12-200 and 12-220 (reviewed 30 June 2026), give the worked
examples.

s 75-10(3) uses an approved valuation when one 'has been made' for an interest
held before 1 July 2000; without one, s 75-10(2) takes the consideration for
the acquisition, so `method` is the operator's statement of which exists. The
valuation day is checked against the item the facts select: items 1 and 3 both
fix 1 July 2000, so the command does not ask which applies. Item 2's earlier
day of applying for registration may fall on or before 1 July 2000, so that
day has no lower bound. s 9-90(1) rounds the GST on one supply on one invoice
to the nearest cent, 0.5 up; the margin and the apportioned base are not
rounded. A supply to an associate is refused: s 75-13 (read, with s 11-5, on
19 November 2026 at 9.13 pm) works its margin from the GST inclusive market value. An
interest contracted for before 1 July 2000 but acquired through a fully
taxable supply is `taxable_supply_without_margin_scheme`, ineligible under
s 75-5(3)(a) whatever GSTR 2006/8 para 39A says of its timing.

`tests/gst_margin_scheme.rs` holds each case.

| Example | Facts given | Result |
| --- | --- | --- |
| Tax Examples 12-200, Joanne Black | Held before 1 July 2000, valued at **$448,000** then, sold for **$640,000** | Margin $192,000; GST **$17,454.55** (shown as $17,454) |
| Tax Examples 12-220, Hans | Valued at **$250,000** on 1 July 2000, sold for **$1.5 million** | GST **$113,636.36** (shown as $113,636) |
| Tax Examples 12-220, Luke | Bought for $1.5 million under the scheme; one of 3 townhouses for **$900,000** | Base $500,000; GST **$36,363.64** (shown as $36,363) |
| GST 11-110, improvements | Land **$250,000**, improvements $50,000, sold for **$360,000** | Margin $110,000; GST **$10,000** |
| GST 11-110, equal blocks | $500,000 over 5 blocks of 600 m2; sold for $155,000 and $166,000 | GST **$5,000** and **$6,000** |
| GST 11-110, unequal block | 1,200 of 3,600 m2 of $500,000, sold for **$300,000** | Margin $133,333.33; GST **$12,121.21** (shown as $12,121.18) |
| GST 11-110, strata unit 3 | $600,000 of $2,400,000 expected, of $400,000; sold for **$610,000** | Base $100,000; GST **$46,363.64** (shown as $46,363.63) |
| GST 11-100, held at 1 July 2000 | Valued at **$100,000**, sold for **$144,000** | GST **$4,000** |
| GSTR 2006/8 example 1, Tureia | Bought through a taxable supply without the scheme | Not eligible: `acquisition_eligible`; GST on the full price |

The Library rounds the margin to whole dollars before dividing in the unequal
block, and cuts strata unit 3's GST off rather than rounding it; s 75-15 takes
the corresponding proportion without rounding, and s 9-90(1) rounds only the
GST, so the command differs from it by 3 cents and 1 cent. The tests also cover
each failing condition alone and together, a nil margin, unpaid consideration,
items 2 and 4 and their days, each refused acquisition, and each refusal.

## Resident tax

Use ¶7-010's already established $144,014 taxable income for 2024–25. The basic
tax slices are `26800 × 0.16 = 4288`, `90000 × 0.30 = 27000` and
`9014 × 0.37 = 3335.18`. Their sum is **$34,623.18**. The later offset, Medicare,
credits and refund calculations in the passage are excluded.

The existing $45,000 boundary gives $4,288 at 16%. At the engine's existing
2026–27 first rate of 15%, the same arithmetic gives $4,020. The latter is a
regression of the existing rate, not a rate corroborated by this Library passage.

## Foreign resident tax

This worksheet's evidence is not a Library document. On 14 November 2026 the
Income Tax Rates Act 1986 was read as compiled at 1 July 2024 (compilation
63), 28 March 2025 (compilation 65) and 1 July 2026 (compilation 66). In each,
Schedule 7 Part II clause 1 gives one table for the 2024-25 and later years
of income: the part of ordinary taxable income to $135,000 at the second
resident personal tax rate, the part to $190,000 at the third and the rest at
45%. Section 3(1) defines those rates as items 2 and 3 of the resident table
for the year, which are 30% and 37% in 2024-25, 2025-26 and 2026-27; the
2026-27 and 2027-28 resident changes are to item 1 only. The table starts at
$0, so there is no tax-free threshold. `tests/tables.rs` derives the scale
from the resident scale for each year.

Part II applies to a prescribed non-resident: a person who was a non-resident
at all times in the year, other than one to whom, at any time in the year,
compensation or a pension, allowance or benefit is payable under the Veterans'
Entitlements Act 1986, subsection 4(6) of the Veterans' Entitlements
(Transitional Provisions and Consequential Amendments) Act 1986, the Military
Rehabilitation and Compensation Act 2004 or the Social Security Act 1991 in
respect of which the person is liable to be assessed and to pay income tax in
Australia (s 3(1)). A part-year resident is a resident taxpayer there. The
worksheet takes both as scope conditions.

The ATO page "Tax rates - foreign resident" (last updated 1 June 2026, read on
14 November 2026) gives the same table for 2024-25 and 2025-26: 30c for each
$1 to $135,000, $40,500 plus 37c for each $1 over $135,000, and $60,850 plus
45c for each $1 over $190,000. It has no 2026-27 table, so a 2026-27 result
warns that the rates are the Act's and a later Act can change them. It also
says foreign residents do not pay the Medicare levy. ITAA 1936 s 251S(1),
read on the ATO's legal database the same day, levies it on the taxable income
of a person resident at some time in the year, and ITAA 1997 s 61-110(1) gives
the low income tax offset only to an Australian resident. The worksheet
therefore gives basic tax only, and its scope says that `medicare-levy` and
`low-income-tax-offset` do not apply. `tests/foreign_resident_tax.rs` holds it
to the ATO's table at every bracket boundary, a dollar either side and above
the top threshold for 2024-25 and 2025-26, and to the Act's rates for 2026-27.
2027-28 is refused, as `resident-tax` refuses it; `tests/tables.rs` keeps the
2 worksheets' years the same.

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

Benefit valuation and the choice of gross-up type must already be established.
[FBTAA s 5B(1B) and (1C)](https://www.legislation.gov.au/C2004A03280/2026-07-01/2026-07-01/text/original/epub/OEBPS/document_1/document_1.html)
give the gross-up formulas. With 47% FBT and 10% GST they reduce to
`57000 / 27401` for type 1 and `100 / 53` for type 2. The ATO's published
4-place factors remain in the worksheet's rates. The estimate uses the
unrounded grossed-up amounts before presenting the result to cents.

The `return_item` figures follow the FBT return 2026 instructions instead. Their
example enters 14A as $11,000 × 2.0802 = $22,882.20, written as $22,882; item 15 is
the sum of 14A and 14B and item 16 is item 15 × 47%. The instructions show whole
dollars without saying how cents are removed. The ATO's Calculating your FBT page
works the same $16,500/$6,000 values in its MantCo example and rounds each gross-up
to the nearest dollar, so the engine does too. Applied to that example's
taxable values, the statutory formulas give $34,323.56 and $11,320.75,
then item 14A $34,324, item 14B $11,321, item 15 $45,645 and item 16
$21,453.15. The estimate before rounding the return items is $21,452.83.
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
`tests/tables.rs` checks these figures for 2026 and 2027 and the effect of
using the full formulas on $1,000,000 of each type of benefit.

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

## FBT EV exemption check

Added on 19 November 2026. Its sources were read that morning, 9.34 am to
9.55 am AEDT: the FBTAA (C2004A03280, compilation 97, from 1 July 2026) ss
7(1), 8A, 45, 53(1), 135P and 136(1), from the Federal Register of
Legislation's EPUB text; the Treasury Laws Amendment (Electric Car Discount)
Act 2022 (C2022A00086, No. 86 of 2022) as made, for s 8A(2)(c) and (5) as
inserted, Schedule 1 item 7 (application from 1 July 2022) and Schedule 2
item 5 (the plug-in hybrid cut-off from 1 April 2025 and its exception); the
A New Tax System (Luxury Car Tax) Act 1999 (C2004A00463, compilation of 1
April 2025) ss 5-20 and 25-1; and the ATO's 'Electric cars exemption' (last
updated 1 April 2026), 'FBT on plug-in hybrid electric vehicles' (last
updated 14 March 2025), 'Luxury car tax rate and thresholds' (last updated 1
June 2026), whose fuel-efficient column is the threshold table, and 'Electric
car discount – more sustainable fringe benefits tax treatment of electric
cars' (last updated 3 November 2026), which says the change from 1 April 2027
is not yet law. The Library's Tax Examples, FBT, ¶3-045 (reviewed 30 June
2026) gives the Cleancut example.

s 8A(1)(c) asks whether luxury car tax has become payable on any supply or
importation of the car before the benefit is provided. The ATO tests that by
the value at the first retail sale against the fuel-efficient threshold for
the sale's financial year; LCT Act s 25-1(1) makes a car a luxury car only
when its value exceeds the threshold, so a value equal to it passes. A first
retail sale after the FBT year comes after every benefit in it, so it meets
the condition at any value; a taxed sale within the year after the car was
first held and used meets it only for the benefits before the sale, and is
refused. Any other supply or importation on which luxury car
tax became payable is a fact the operator gives.

`tests/fbt_ev_exemption.rs` holds each case. The ATO's examples are run for a
benefit in the FBT year ended 31 March 2026 or 2027; where an example gives
no price, the value is fabricated below the threshold.

| Example | Facts given | Result |
| --- | --- | --- |
| Library ¶3-045, Cleancut | Battery electric, bought 1 July 2025 for $72,000 plus GST: **$79,200** against **$91,387** (2025-26) | Exempt; reportable |
| ATO, Shelly | First held and used 1 April 2022 | Not exempt: `first_held_and_used_from_1_july_2022` |
| ATO, John | Held 15 June 2022, first used 5 July 2022, no luxury car tax | Exempt |
| ATO, GHI Co and Kerry | **$85,000** in 2022-23 against **$84,916** | Not exempt: `no_luxury_car_tax_at_first_retail_sale` |
| ATO, Sonia | Plug-in hybrid delivered 1 May 2025 | Not exempt: `plug_in_hybrid_exempt_before_1_april_2025` |
| ATO, Simon | Plug-in hybrid leased 1 April 2024 to 31 March 2027, extension not exercised | Exempt in the year ended 31 March 2027 |
| ATO, Tony | Novation broken by unpaid leave from 1 May 2025 | Not exempt: `plug_in_hybrid_no_new_commitment` |
| ATO, Courtney | New employer's novation from 1 October 2025 | Not exempt: no commitment before 1 April 2025 and a new one |
| ATO, pool car | No commitment to a designated employee | Not exempt: `plug_in_hybrid_commitment_before_1_april_2025` |

The tests also cover the threshold itself and a cent above it, each listed
year's threshold, each other failing condition alone and several together, a
hydrogen fuel cell car, a motor cycle, the check's input feeding
`fbt-car-statutory`, and each refusal.

## FBT loan

These worksheets were added on 15 November 2026. Their sources were read that
morning, 7.58 am to 8.17 am AEDT: the FBTAA as compiled on 1 July 2026
(C2004A03280, compilation 97, EPUB documents 1 and 2), the FBT return 2026
instructions' item 23 page, the ATO's FBT rates and thresholds page (last
updated 20 May 2026), chapters 8, 14 and 16 of 'Fringe benefits tax - a guide
for employers', and the ATO's 'Loan and debt waiver fringe benefits' and 'Car
parking and FBT' pages (both last updated 6 March 2025). The rates page gives
the statutory or benchmark interest rate as 8.62% for the year ended 31 March
2026 and 8.27% for 2027, and the car parking threshold as $11.03 and $11.48; it
says these figures are published only there, not in a taxation determination.
`tests/fbt_loan.rs`, `tests/fbt_meal_entertainment.rs` and
`tests/fbt_car_parking.rs` hold each case.

Item 23C example 8: $20,000 lent for the whole year at no interest, `20000 ×
0.0862 = 1724`, **$1,724.00**; the worksheet takes the daily balances' total,
`7300000 × 0.0862 / 365 = 1724`. ¶3-100 (review date 30 June 2026) lends
$100,000 for 2025-26 at 3.45%, 40% spent on shares: notional interest `8620`
less `3450` accrued is `5170`; GD `8620 × 0.4 = 3448` less RD `3450 × 0.4 =
1380` is a notional deduction of `2068`, for **$3,102.00**. Given to `fbt` as a
type 2 value, `3102 × 1.8868 = 5852.85`, item 14B **$5,853**, and FBT of
**$2,750.91**, which the Library rounds to $2,751. The guide's 8.5 example used
5.65% for 2016-17; its $50,000 at 5% reruns at 8.62% to `4310 − 2500 = 1810` and
at 8.27% to `4135 − 2500 = 1635`. Its 8.8 examples, $50,000 at 4% with 60% used
to produce income, rerun at 8.62% to TV `2310`, GD `2586` and ND `2586 − 1200 =
1386`, for **$924.00**; with the rate set by regard to use, RD is `431` and the
value **$155.00**.

Notional interest and the gross deduction are each rounded half up to cents: the
Act gives no rounding, and amounts of interest and deductions are in cents.
Every other amount is a difference or a minimum of amounts in cents. Item 23C
says to show zero rather than a negative value, so the reduction stops at the
gross taxable value; RD above GD leaves no notional deduction.

## FBT meal entertainment

Item 23P example 18: $4,000 of meal entertainment under the 50/50 split,
**$2,000.00**. ¶3-220 (review date 30 June 2026): $177,500 for 2025-26 gives
**$88,750.00** under the 50/50 split, and a register showing 40% gives
**$71,000.00**. The register percentage (s 37CB(2)) is kept exact: a register of
$1,000 of fringe benefits out of $3,000 shows 33.33% but applies a third, so
$900 of expenditure gives $300.00, not $299.97. s 37CD makes a register valid for
its first year and the 4 after it; the worksheet refuses a first year outside
that window, and the operator establishes any later register.

The 20% rule was added on 16 November 2026 from FBTAA s 37CD(3), compilation 97
(C2026C00332, registered 24 July 2026, in force from 1 July 2026, the current
compilation that day), read through the Federal Register of Legislation's API
at 8.25 am AEDT. A register valid for a year 'ceases to be valid at the end of
that FBT year' if that year's meal entertainment expenses are 'more than 20%
higher than the corresponding total for the first FBT year for which the
register was valid'. The worksheet takes each year's expenses from the first
valid year to the year before the one valued and refuses the register if any
later year's is more than 1.2 times the first's; the year valued may rise by
any amount, since the register ceases only at its end. $1,000 then $1,200 is
exactly 20% higher and stays valid; $1,200.01 ends it
(`tests/fbt_meal_entertainment.rs`). The employer's guide, 14.8 (chapter 14's
latest update, January 2020), says 'increase by more than 20% in a year', which reads as a
year-on-year test; the Act compares every year with the first, and the
worksheet follows the Act.

## FBT car parking

The employer's guide, chapter 16. Example 12: one station's $10 fee above the
2023 threshold of $9.72 makes benefits arise, and they are valued at another's
$5.50; the tests apply the same rule with a $12 fee against 2026's $11.03.
Example 13: a $272 monthly fee over 22 business days is the s 39E daily rate
`272 / 22 = 12.36`; 22 benefits at that fee give **$271.92**. Example 15, car park
A: `(17 + 20) / 2 = 18.50` a benefit under the average cost method; car park B,
`(10 + 15) / 2 = 12.50`, where B's $10 first business day fee was above 2023's
threshold but is not above 2026's, so the same facts give nil in 2026. Example 17
(the 12 week register): `2500 × 52 / 12 = 10833.33`, then `× 183 / 366 =
5416.67`, **$5,416.67**; s 39GB divides by 366 in every year. ¶3-055 (review date
30 June 2026) cites the $11.03 threshold for 2026. Item 23N example 17 starts
from a value already worked out, so it tests no method.

s 39A(1)(a)(iii) asks whether any station within 1 km charged a lowest all-day
fee of more than the threshold on the first business day, so the worksheet
compares the highest of those fees with it and reports a nil taxable value when
none is above. The commercial parking station and average cost methods round
the gross value to cents before the recipients' payments, as the car worksheets
do.

The market value and spaces methods were added on 16 November 2026 from FBTAA
ss 39D and 39FA to 39FE, compilation 97 (C2026C00332, the current compilation
that day, read through the Federal Register of Legislation's API at 8.25 am
AEDT), and the employer's guide, chapter 16 (last updated November 2023, read at
8.27 am). s 39D(2) values a benefit at 'the amount that the recipient could
reasonably be expected to have been required to pay' at arm's length, less the
recipients contribution; s 39D(3) makes the election depend on a qualified
valuer's report, which the operator establishes. Example 17 values each of its
register's 250 benefits at an arm's length value of $20 less a $10 recipient
contribution: under the market value method, `250 × 20 = 5,000.00` less
`250 × 10 = 2,500.00` of payments gives **$2,500.00**.

Example 18 (the spaces method): 4 spaces, each at a $10 daily rate amount for
182 days, `10 × 182 / 366 × 228 = 1,133.77` a space and **$4,535.08** for 4;
3 employees and 4 spaces, so s 39FB scales it by `3 / 4` to **$3,401.31**;
`$30 × 26 × 3 = $2,340` of contributions leave **$1,061.31**, the guide's
figures. The example's year ended 31 March 2023; the tests run its facts in
2026 and 2027 with a first business day fee above those years' thresholds.
s 39FA(4) states no rounding, and the guide shows its step 4 amount as $4.97
but carries it unrounded to $1,133.77. The worksheet adds each group's
`daily rate × days × 228 × spaces`, divides by 366 once and rounds the total to
cents, then rounds the s 39FB amount to cents; per-space rounding could differ
by a cent a space (a $12 space for 365 days beside example 18's 4 gives
$7,263.61, where rounding each space first gives $7,263.60).

s 39FB(6) says s 39FB 'does not apply' where the head counts are not
representative; the guide's 16.2.8.2 says 'You cannot use the Spaces method'
then. The worksheet follows the Act's narrower effect and leaves
representativeness to the operator.

s 39GF(3), read the same day, ends a car parking register 'if the number of car
parking fringe benefits ... increases by more than 10% on any day in that FBT
year' without saying what the increase is measured from (the register's 12
weeks, the first valid year or the day before). The guide's 16.2.7.4 says only
'increased by more than 10% in the previous FBT year', and 16.2.2 'since the
register was kept'. With no stated base the rule is not built; the worksheet
still checks the register's age.

## FBT debt waiver

These worksheets were added on 15 November 2026. Their sources were read that
day, 12.25 pm to 12.31 pm AEDT: the FBTAA as compiled on 1 July 2026
(C2004A03280, compilation 97, EPUB documents 1 and 2, with the formulas' text
alternatives for ss 24 and 44, 'GD minus RD' and 'TV minus ND'), the FBT return
2026 instructions' item 23 page (published 1 April 2026), chapters 2, 8, 9, 17
and 19 of 'Fringe benefits tax - a guide for employers' (chapters 8, 9 and 17
last updated 29 May 2024) and the ATO's 'Loan and debt waiver fringe benefits'
page (last updated 6 March 2025). No ATO-published rate or factor enters them.
`tests/fbt_debt_waiver.rs`, `tests/fbt_expense_payment.rs` and
`tests/fbt_property.rs` hold each case.

Compilation 97 is in force from 1 July 2026. Compilation 96 (C2025C00239), in
force from 1 April 2025 to 30 June 2026 and read from the same register on 15
November 2026, gives the same text for every section these worksheets and the
residual, housing, LAFHA, board and entertainment worksheets apply. The one
change in them, s 24(1A) (Treasury Laws Amendment (Tax Reform No. 1) Act 2026,
Sch 4 item 18), applies to FBT years starting on or after 1 April 2027 (item
20), so to neither year the worksheets cover.

The Act states no rounding. The worksheets round half up to cents each amount
they show before a later step uses it: a 75% value, a gross deduction under
ss 24, 37, 44 and 52, and a statutory annual value. The amounts shown then add
up, as `tests/properties.rs` checks. On a half cent the taxable value can differ by a cent from unrounded
arithmetic: a $100.03 external property benefit, half deductible, has a gross
deduction of $50.015, shown and taken off as $50.02, so $50.01, where
unrounded arithmetic gives $50.02 (`tests/fbt_property.rs`).

Item 23D example 9: a $500 debt, principal and interest, waived: **$500.00**,
which as a type 2 value grosses up to `500 × 1.8868 = 943.40`, item 14B
**$943**. The guide's 8.2: a $1,000 loan released is **$1,000.00**, and with
$100 of accrued interest also waived **$1,100.00**. The Act gives the amount
waived and nothing against it (s 15), and item 23D leaves the contribution and
reduction columns blank, so the worksheet takes no other input.

## FBT expense payment

Item 23E example 10: a $1,200 telephone bill paid, 60% for business: GD
`1200 × 0.6 = 720`, so **$480.00**. The guide's 9.4: $500 spent, 80%
deductible, $250 reimbursed without regard to use: GD `400` less RD
`(500 − 250) × 0.8 = 200` is a notional deduction of `200`, for **$50.00**; $350
reimbursed for the business part: RD `400 − 350 = 50`, ND `350`, **nil**. The
operator gives RD, since the guide's step 3 works it out differently by how the
reimbursement was set; the worksheet refuses one above the recipient's portion
(s 136(1)), the expenditure the employee bore. The guide's 9.3 in-house
petrol: the retailer bought it from the employer for $800 (s 42(1)(b), the
lesser of that and the $1,000 notional value), less the recipient's portion
`1000 − 250 = 750`, **$50.00**, and nil once the employee's unused $1,000 s 62
reduction applies. ¶3-240 (review date 30 June 2026): an associate's $10,000
HELP debt reimbursed is **$10,000.00**, type 2, `18,868.00` grossed up; ¶3-340:
$35,000 of mortgage and $24,000 of childcare reimbursements, `59000 × 1.8868`,
item 14B **$111,321**. Applying the statutory s 5B(1C) formula to the same
taxable values gives $18,867.92 and leaves item 14B at $111,321.

s 22A(1)(b) makes the recipient's expenditure less the provider's portion
(the payment less the contribution, or the reimbursement) the notional
contribution, so one formula serves a payment and a reimbursement, with a
contribution of 0.00 for a reimbursement. The otherwise deductible rule for an
in-house expense payment (s 24(1)(b)(i)) measures the gross expenditure 'but
for ... the recipients contribution', which the guide does not work through,
so it is left out rather than guessed.

In-house residual expense payments were added on 16 November 2026 from FBTAA s
22A(2) and (3) and ss 48 and 49, compilation 97 (C2026C00332, read through the
Federal Register of Legislation's API at 8.25 am AEDT), and the guide's 9.3
and 18.4 (chapters 9 and 18, latest updates May 2021 and January 2020), read
at 8.27 am. s 22A(2)
takes the value s 48 or 49 would give 'but for section 52 and Division 14',
with the recipient's portion as the notional contribution, exactly as s
22A(1) does for property. s 22A(3) reads s 49 for the recipient's overall
benefit, so for a period benefit the price or notional value given is the
overall benefit's. The guide's 18.4 rental, $100 to the public with $50 charged,
`0.75 × 100 − 50`, **$25.00**; paid as an expense instead ($100 spent, $50
reimbursed), the recipient's portion is $50 and the value is the same
**$25.00**, as 9.3 says a staff discount should be however it is given. No ATO
example works an in-house residual expense payment, so this case is
hand-worked from the 18.4 example.

## FBT property

The guide's 17.3: identical goods sold to wholesalers for $1,000 less 5% for
early payment, **$950.00**; a desk at a lowest public price of $900 sold to an
employee for $500, `0.75 × 900 − 500`, **$175.00**; a damaged racquet worth $30
sold for $5, `0.75 × 30 − 5`, **$17.50**; a television bought for $500, retailed
at $750 and sold for $400, **$100.00**. 17.5: $500 of goods, $250 paid, 80%
work use: GD `400` less RD `200`, **$50.00**; with the contribution set for 50%
use, RD `150`, **nil**. Item 23K example 14: a $2,000 television and a $1,600
air conditioner at 75%, `2700 − 300 − 1000`, **$1,400.00**, type 1 `2912.28`,
item 14A **$2,912**. ¶3-260 (review date 30 June 2026): a $2,000 television
bought for $1,800 is nil with a $300 excess; a $1,200 sound bar bought for
$600 is **$300.00**, nil after the $1,000 reduction; a $4,000 desktop with the
$700 of reduction left is **$2,300.00**, Jenny's total.

Item 23K example 14, an electrical retailer's television and air
conditioner, and ¶3-260 take 75% of the lowest price to the public without
saying that the provider made the goods. s 42(1)(a)(ii), as compiled, gives 75%
of that price only for property the provider manufactured, produced, processed
or treated; for property it acquired, s 42(1)(b) gives the lesser of the arm's
length price and the notional value (`in_house_acquired`). The worksheet
applies the rule its `method` names, and the tests apply
`in_house_public_price` to reproduce those examples' figures. The
otherwise deductible rule comes before s 62: s 44 gives the taxable value 'but
for Division 14', and s 62 is in Division 14. Benefits provided jointly to an
employee and an associate (s 44(5), the guide's 17.5A) are not covered.

## FBT residual

These 5 worksheets were added on 15 November 2026. Their sources were read
that afternoon, 12.43 pm to 12.52 pm AEDT: the FBTAA as compiled on 1 July 2026
(C2004A03280, compilation 97, EPUB documents 1 and 2, with the alternative text
of their formula images), chapters 10, 11, 13, 15 and 18 of 'Fringe benefits
tax - a guide for employers' (current version 22 November 2024), the ATO's FBT
rates and thresholds page (last updated 20 May 2026) and the FBT return 2026
instructions' item 23 page. `tests/fbt_residual.rs`, `tests/fbt_housing.rs`,
`tests/fbt_lafha.rs`, `tests/fbt_board.rs` and
`tests/fbt_tax_exempt_body_entertainment.rs` hold each case.

Item 23M example 16: a utility with $8,000 of operating costs used 25%
privately, `8000 - 6000 = 2000`, **$2,000.00**, the business use taken off as
the otherwise deductible reduction. The guide's 18.4: an employee of a rental
store hires a video recorder the public rents for $100 and pays $50, `0.75 ×
100 - 50 = 25`, **$25.00** before the s 62 reduction. Its 18.7: $500 of hired
goods, a $250 contribution and 80% income use; GD `400` less RD `200` gives
`250 - 200 = 50`, **$50.00**, and with RD limited to `150` the value is nil. Its
18.6 vehicle example prints `($20,080 × 10%) - $0 = $2,080`; 10% of $20,080
is `2008`, and the test asserts **$2,008.00**. The same example gives 5,200
kilometres as both the private and the total travel, so its 10% private use
does not follow from its facts.

## FBT housing

Item 23F example 11: a market rent of $41,600 for the year less $5,200 rent,
**$36,400.00**. The guide's 10.5: $11,820 for 274 days of the base year is a
statutory annual value of `11820 × 365 / 274 = 15745.62`; indexed by NSW's
1.040 for 2027, `16375.44`, for 91 days `4082.64`; unrounded arithmetic,
`16375.4448` for 91 days, would give $4,082.65. Its single-employee example,
$5,200 for 121 days, prints `$5,200 ÷ 121 × 365 = $15,687.70`, the figure the daily rate gives when rounded to cents first;
s 26(2)(a)'s formula, the market value times the days in the year over the
whole days of the tenancy, gives `5200 × 365 / 121 = 15685.95`, which the test
asserts. The Act does not round the statutory annual value; the worksheet
rounds it to cents because `prior_statutory_annual_value` is taken in cents.
The guide's later years, at the earlier factors it uses, give its $4,040.17
and $11,088.60 whether or not the value is rounded. ¶3-120 (review
date 30 June 2026): $350 a week for 39 weeks less $50 a week, **$11,700.00**.
The indexation factors are the rates page's for each State and Territory for
2026 and 2027; `tests/fbt_housing.rs` checks all 16.

## FBT LAFHA

Item 23G example 12: $591 a week, $350 for accommodation and $241 for food, the
first 40 weeks reducible: `23640 - 14000 - (9640 - 1680) = 1680`; the other 12
weeks' $7,092 is taxed whole (s 31B), for the year's **$8,772.00**. The guide's
11.6 examples give **$0**, **$140**, **$80** and **$147** a week, and its 11.5
Steve and Helen example an exempt accommodation component of $225 each.
¶3-160 and ¶3-180 (review date 30 June 2026): $850 and $700 a week, each with
$200 for total food costs, both **$42.00**.

## FBT board

Item 23J example 13: board valued at **$21,900.00**, 10,950 meals at $2. The
guide's 13.2: $6 a day towards 3 meals takes each day's value to nil.

## FBT tax-exempt body entertainment

Item 23L example 15: a council's Christmas function, **$5,000.00**. The guide's
15.9: Mary entertains 3 clients and is reimbursed; her share per head is 1 of
4, 25%.

## FBT entertainment classifier

Added on 16 November 2026. Its sources were read that evening, 5.58 pm to 6.11
pm AEDT, from the Federal Register of Legislation's current compilations, each
checked against its EPUB text: ITAA 1997 (C2004A05138, compilation 268, from 1
October 2026) Division 32 and ss 27-5 and 995-1; the FBTAA (C2004A03280,
compilation 97, from 1 July 2026) ss 20, 37A to 37CF, 40, 41, 45, 58P and
136(1); ITAA 1936 (C1936A00027, compilation 193, from 1 October 2026) ss 51AEA
and 51AEB; and the GST Act (C2004A00446, compilation 96, from 1 January 2026)
s 69-5. TR 97/17 (as consolidated) and TR 2007/12 were read on the ATO's legal
database at 6.00 pm. Both rulings are the Commissioner's view, not the law;
the classifier cites their paragraphs as such. TR 97/17 para 27 gives the
minor benefits threshold as $100, which is superseded: s 58P(1)(e) says less
than $300, and the classifier applies the Act. FBTAA ss 5E and 135P(1) and the
Fringe Benefits Tax Assessment Regulations 2018 (F2018L01284, as made, the
current version) regulations 6 to 11 were read at 7.50 pm to 7.55 pm for
whether each fringe benefit is excluded from reporting. No rival's
spreadsheet, text or formula was used.

`tests/fbt_entertainment_classifier.rs` holds each case:

| Ruling example | Facts given | Result |
| --- | --- | --- |
| TR 97/17 example 1.1, paras 49 to 52 | $1,100 with GST, a social function on the premises on a working day; 6 employees, 2 associates, 2 clients | Employees $660 exempt (s 41), not deductible; associates $220 a property fringe benefit, deductible **$200.00** with a **$20.00** credit; clients not deductible |
| TR 97/17 example 1.3, paras 55 and 56 | $55 with GST, sandwiches and juice for employees on the premises | Not entertainment, exempt (s 41), deductible **$50.00**, credit **$5.00** |
| TR 97/17 example 2.1, paras 61 and 62 | $1,100 with GST, a social function at a restaurant for employees | Property fringe benefit, deductible **$1,000.00** |
| TR 97/17 example 3.1, paras 66 and 67 | $66, a travelling employee's reimbursed meal | Not entertainment; an expense payment benefit, deductible **$66.00** |
| TR 97/17 example 6.2, para 84 | $120, employee and client both travelling, employer pays | Neither is entertainment; both deductible, **$120.00** |
| TR 97/17 questions 9 and 10, paras 89 to 96 | A travelling employee's meal, non-travelling employees' and a non-travelling client's, as 3 lines | **$50.00**, **$100.00** and **nil** deductible |
| TR 97/17 example 11.1, paras 100 and 101 | $200 business lunch for 2 employees, reimbursed | Expense payment benefit to the reimbursed employee and a property benefit to the other, deductible **$200.00** |
| TR 97/17 example 16.1, para 130 | $440 with GST, employees dine on the corporate card | Property fringe benefit, deductible **$400.00** |
| TR 97/17 example 17.1, para 134 | $50 discounted bill, an employee and a client | Employee's share **$25.00** |
| TR 97/17 example 17.2, para 135 | $120 discounted bill, an employee and 3 clients | Employee's share **$30.00** |
| TR 97/17 paras 136 to 138.1 | A travelling employee's meal and the spouse's, reimbursed | Spouse's meal is meal entertainment and an expense payment benefit |
| TR 97/17 para 44; TR 2007/12 para 14 | Example 1.1 under the 50/50 split method | No s 41 exemption or minor benefit; half deductible and creditable, **$500.00** and **$50.00**; **$1,100.00** for `fbt-meal-entertainment` |
| TR 97/17 Part A table, column R, (a)(ii)1, (a)(ii)4 and (b)1 | Associates at a social function on the premises; associates' light lunches there; employees at a function off the premises, also under the 50/50 split | Meal entertainment **excluded** (s 5E(3)(a)); the light lunches **not excluded** |
| TR 97/17 paras 31 and 32; FBTAA ss 37BA and 37CB(3) | $220 with GST, a client meal under the 50/50 split and the register at 40% | No fringe benefit; in the year's expenditure; deductible **$100.00** and **$80.00** |
| TR 2007/12 example 2, paras 31 to 41 | A meal under $300 a head, not stated infrequent and irregular | Minor benefit still possible, the employer's judgement |

The tests also cover the 12 week register at 40%, recreation under an
election, accommodation or travel taking the food or drink's occasion
(ITAA 1997 s 32-10(1)(b)), undetermined lines, the $300 boundary and each refusal. The ruling
give dollar figures only in examples 11.1, 17.1 and 17.2: the other amounts are
fabricated, and the ruling decides the treatment the test checks.

## FBT meal entertainment register

This worksheet's evidence was read on 19 November 2026 from 9.25 am to
9.35 am AEDT:

- FBTAA ss 37CA to 37CF on the ATO's legal database
  (https://www.ato.gov.au/law/view/document?docid=PAC/19860039/37CC and the
  sections beside it). s 37CA: the election, for a year the employer has a
  valid register for; s 37CB(2): the register percentage is the value of the
  meal entertainment fringe benefits over the value of the meal entertainment
  in the register's 12 week period; s 37CC: the register is kept for a
  continuous period of at least 12 weeks throughout which meal entertainment is
  provided, representative of the first FBT year it is valid for, or it is not
  valid; s 37CD(1) and (2): valid for the year a period within one FBT year
  falls in, or only the second year of a period spanning 2, and the 4 FBT years
  after; s 37CD(3): it ceases to be valid at the end of a year whose meal
  entertainment expenses are more than 20% higher than the first valid year's,
  or when a later valid register covers the year; s 37CE(1): the 6 details,
  and s 37CE(2), entries made as soon as practicable; s 37CF: a false or
  misleading entry invalidates the register.
- The ATO's Fringe benefits tax - a guide for employers, chapter 14 (latest
  update January 2020), 14.8: 'Valid register' and 'Details to be included in
  register' restate those sections; the guide says the minor benefits exemption
  can apply under this method but the property benefits exemption cannot, and
  points to TD 94/25 on apportioning entertainment between employees and
  others, which the check leaves to the operator's `fringe_benefits_value`.
- The Library's ¶3-220 (tax examples, 'Fringe Benefits Tax (FBT)', last
  reviewed 30 June 2026): ABC's register at 40%, kept in the 2025-26 FBT year,
  can be used for that year and the following 4 (s 37CD), and
  $177,500 x 40% = $71,000. The test builds a register at 40% kept in that
  year, checks it valid for the year ended 31 March 2027 and passes its values
  to `fbt-meal-entertainment`, which gives $71,000.00.

Neither the Act nor the guide gives a worked example of a register failing a
check; the tests work each failure from the section's words. 12 weeks is
taken as 84 days with both the first and last day counted.

## FHSS

Added on 20 November 2026. Its sources were read between 11.59 pm on 19
November and 12.15 am on 20 November 2026: from the ATO's legal database, TAA 1953 Sch 1 ss 12-460, 131-5,
131-10, 138-10, 138-25, 138-30, 138-35, 138-40 and 280-105, and ITAA 1997
ss 313-20, 313-25, 313-50 and 313-60; the Taxation Administration
Regulations 2017, compilation 18 (14 November 2026), reg 53A, from the Federal
Register of Legislation; the Commissioner's guidance note GN 2024/1 'First
home super saver scheme'; the ATO's 'About FHSS release amounts', 'About your
contributions' and 'FHSS tax assessment' pages; and the ATO's 'Shortfall
interest charge (SIC) rates' page, last updated 4 September 2026, whose
quarterly annual rates from July to September 2017 to October to December
2026 are the table in `src/engine/fhss.rs`. The 20% FHSS tax rate is the
First Home Super Saver Tax Act 2017 s 4, read on 19 November 2026, and GN
2024/1 example 13 applies it.

What the Act settles, where the guidance is less exact:

- **The withholding rate.** s 12-460 only requires the Commissioner to
  withhold; reg 53A sets the amount as his estimate of the tax payable on the
  assessable FHSS released amount, or 17% of it where he cannot estimate. The
  marginal rate plus Medicare levy less 30% is GN 2024/1's account of that
  estimate, and the command follows it, labelled an estimate. The 17% applies
  only where the Commissioner cannot estimate (reg 53A(1)(b)), so the caller
  states that with `commissioner_cannot_estimate`; a missing marginal rate is
  refused rather than taken to mean it. Neither sets a rounding; examples 11
  to 13 round down to the dollar ($1,589.50 to $1,589).
- **Determinations before 15 September 2024.** Schedule 4 to the Treasury
  Laws Amendment (2023 Measures No. 3) Act 2023 (No. 75 of 2023, as made,
  read on 20 November 2026) commenced then. Its item 28(1) applies the
  amendments to determinations from 1 July 2018; item 28(4) keeps 14 days, not
  90, in s 313-35(1)(c)(i) for an earlier determination, a deadline the
  command does not compute; and item 30(4) takes a release after a
  transitional action (amending a determination made before 15 September
  2024, its request or its release authority by 14 September 2027) out of
  Subdivisions 313-C to 313-E, so no FHSS tax or recontribution applies. The
  amounts are otherwise worked the same way; the command adds
  `transitional_note` for every determination made before 15 September 2024.
- **The earnings base.** s 138-40(2) defines the amount of a contribution as
  'the amount of the contribution that is counted in your FHSS releasable
  contributions amount', and s 138-30(1)(b) counts 85% of an eligible
  concessional contribution in that amount. The command therefore runs the
  earnings on 85% of a concessional contribution's counted part.
- **The order.** s 138-30(2) counts contributions earliest first across
  financial years.

The published example that does not reproduce: GN 2024/1 example 6 and the
'About FHSS release amounts' page (Jill: $1,500 a month salary sacrifice from
July 2020 to October 2024, determination 15 December 2024) give $15,000 in
each of 2020-21 to 2022-23 and $5,000 from 2024-25, a releasable $42,500 and
associated earnings of $5,190. The command reaches the $50,000 and $42,500,
but s 138-30(2) counts July to October 2023 before July to October 2024, and
the associated earnings come to $7,859. An independent day-by-day script
over the same SIC table gives $7,859 on that order, $7,530 on the example's
2024-25 order, and $9,247 or $8,860 on the contribution in full; none gives
$5,190, and nothing in the method was changed to match it. The other published
examples with associated earnings (Sue's $850, Alex's $1,700, Milena's
determinations) give no contribution dates, so they cannot test the method.
So associated earnings from contributions are labelled `estimate`, and a
caller with the Commissioner's determination gives its components instead.
The Jill regression checks both paths. The statutory estimate is $7,859 and a
determination supplied with $5,190 keeps its own figure.

The tests run Jill (limits and the 85%), Mary ($15,000 of $25,000; $12,750),
Hasan (example 7: $3,000 of the $5,000 counted in 2023-24), the s 138-30
example (the first 10 fortnightly contributions non-concessional), Sue
(example 11: $1,589 withheld at 17%, a $2,805 offset), Alex (example 12: 9%,
$765, $27,735 paid) and George (example 13 and the ATO page: $1,809 withheld,
$24,291 paid, $4,020 FHSS tax), and one quarter's compounding against its
closed form.

## Fuel tax credits

This worksheet was added on 19 November 2026. The Fuel Tax Act 2006 ss 41-5,
41-20, 43-5, 43-10, 47-5 and 47-10 were read that day at the ATO's legal
database (https://www.ato.gov.au/law/view/document?docid=PAC/20060072/43-5 and
the like), with the ATO's 'Rates - business' (QC44494, last updated 29 July
2026) and its year pages: QC69964 (2022-23) and QC72768 (2023-24), last
updated 27 June 2024; QC102637 (2024-25), 29 January 2025; QC105117
(2025-26), 28 April 2026; and QC107638 (2026-27), 29 July 2026. Their 16
tables give 6 fuel rows in 2 uses each; the 192 values were read from the
pages' text into `data/fuel-tax-credit-rates.json` and compared with
data.gov.au's 'Historical FTC rates' workbook (August 2026 edition, read the
same day), which gives the same 192 values. Rates are stored as published, not
derived: the 2022 excise relief's road user charge (26.4 c/L on QC44494)
does not match the gap the later 2022-23 rates imply (27.2 c/L), and the
credit rates are what the ATO applies.

Each period carries why it starts, for PCG 2016/2 Method 2, which applies
where the rate changes in a tax period 'because of indexation' (para 11).
QC44494 says rates are indexed in February and August; the February and
August days are marked `indexation`, except 3 August 2026, when the 30.4%
reduction also ended and the road user charge rose to 32.4 c/L (QC107638
note 4). 1 August 2022 fell inside the 2022 excise relief and is marked
indexation as an August indexation day; no page read states that nothing
else changed that day. The
ATO's 'Simplified fuel tax credits' (QC48520, last updated 6 July 2021) gives
the period-end rate for any change 'for example, in February and August';
the guideline's narrower words are followed.

The basic method follows PCG 2021/2 paras 4 to 12; its Schedule 1 table is an
image on the legal database, so the rigid truck rates come from the ATO's
'Basic method for heavy vehicles' (QC64655, last updated 2 June 2021) Table
1, and articulated combinations' rates are not held (`litres_per_km` from the
engine's report covers them). Method 1 follows PCG 2016/2 para 9, which names
the Australian Institute of Petroleum's state and national prices for the
average GST inclusive price; QC48520 links its weekly retail pump prices
(https://aip.com.au/pricing/pump-prices-retail/). The AIP's pages, read on
19 November 2026, give weekly average retail prices for petrol and diesel by
location and state (from MotorMouth data) and terminal gate prices, under
'All Rights Reserved' with no licence to republish, so the price is an input.

The ATO's 'Working out your fuel tax credits' (QC44511, last updated 6 July
2021) multiplies quantity by rate, divides by 100 and claims the whole
dollars at label 7D; QC64655's examples drop the cents ($1,885.95 is claimed
as $1,885; $1,991.52 as $1,991). Each line is rounded half up to the cent
before the lines are added, as the guideline's We Deliver example rounds each
use. The Library's Tax Examples, GST and other indirect taxes, 12-300
(reviewed 30 June 2026), gives *Linfox* [2019] FCAFC 131: a credit taken into
account in an assessment is not lost under s 47-5, so lines past 4 years are
flagged, not refused.

`tests/fuel_tax_credits.rs` and the module's unit tests hold these cases.

| Case | Source | Worksheet |
| --- | --- | --- |
| We Deliver: 11,430 L, 30,550 public road km at 0.35 L/km, 16.5 and 42.3 c/L | PCG 2021/2 paras 19 to 24 | $1,764.26 + $311.96 = $2,076.22 (unit test: December 2020 rates are outside the data) |
| Jim: 11,430 L, 32,000 total km (31,488 derived), 0.35 L/km | QC64655 example 2 | $1,818.43 + $173.09; $1,991 at 7D (unit test) |
| Jim's figures at the rates from 3 August 2026 | Worked by hand | $2,347.43 + $219.74 = $2,567.17; $2,567 at 7D |
| 10,000 L heavy vehicle on public roads and 2,000 L off road from 3 August 2026 | Worked by hand at 21.3 and 53.7 c/L | $2,130 + $1,074 = $3,204 |
| 100 L of diesel on the first day of each of the 16 periods, both uses | QC69964 to QC107638 | The period's rate in dollars |
| 1,000 L acquired 10 January 2026 in a January to March 2026 quarter | PCG 2016/2 Method 2 across the 2 February 2026 indexation | $526 at 31 March's 52.6 c/L, against $516 at its own day's |
| $2,459 of diesel at an average 245.9 c/L | PCG 2016/2 Method 1 | 1,000 L, $537 |
| $23.98 of diesel at an average 214.8 c/L on 30 September 2026 | PCG 2016/2 Method 1, worked by hand | 23.98 x 53.7 / 214.8 = $5.995 exactly, $6.00 half up (cost x rate before the division) |
| 300 L of E85 at 23.015 c/L | Rounding | $69.045, $69.05 half up |

## FBT minor benefit check

Added on 19 November 2026. Its sources were read that afternoon, 3.00 pm to
3.10 pm AEDT, from the ATO's legal database: FBTAA s 58P (s 58P(1)(b), the
airline transport exclusion, shown as repealed by Act No. 88 of 2013) and the
s 136(1) definition of 'fringe benefit', whose para (g) leaves out an exempt
benefit; and TR 2007/12 (paras 8 to 23 and examples 1 to 13). The Library's Tax
Examples, FBT, 3-060 and 3-080 (reviewed 30 June 2026) give the flowers, gym
and golf day examples.

s 58P(1)(f) asks whether, having regard to all its criteria, it would be
concluded that treating the benefit as a fringe benefit would be
unreasonable: a weighing, not a test. The command takes each criterion as the
operator's statement and decides only where TR 2007/12's examples decide: each
exempt example has similar benefits that are not frequent and totals that are
not substantial. Example 9's voucher is a reward for services but ad hoc, and
examples 1 to 3 recur each Christmas but are not rewards; no example has a
regular reward, so that is left to judgement. Example 8, the one not
exempt on the criteria, has frequent similar benefits with a substantial
total, given as a reward for services, readily valued and not for an
unexpected event; the screen gives `not_exempt` only on all of those. The
Library's 3-060 gym fees and 3-080 golf days are not exempt in the Library; the
screen agrees when they are stated as rewards for services, and otherwise
leaves them to judgement. Paras 14 and 16, on the 50/50 split and salary
sacrifice, are the Commissioner's view and are cited as such.

`tests/fbt_minor_benefit.rs` holds each case.

| Example | Facts given | Result |
| --- | --- | --- |
| Library 3-060 (b), flowers | Property, **$80**, on a birthday | Exempt |
| Library 3-060 (a), gym | Expense payment, **$50** a month | Not exempt as a reward for services; otherwise judgement |
| Library 3-080, golf days | Expense payment, about **$83** a month | Not exempt as a reward for services; otherwise judgement |
| TR 2007/12 examples 1 to 3 | Christmas gift or party, each year, under $300 | Exempt |
| TR 2007/12 examples 5, 6, 7, 10, 11 | Flowers, a car after a storm, tolls, a gym membership, babysitting | Exempt |
| TR 2007/12 example 8 | Monthly incentive vouchers, a reward | Not exempt |
| TR 2007/12 example 9 | An ad hoc recognition voucher, a reward | Exempt |
| TR 2007/12 example 12 | A tax-exempt body's movie tickets | Not exempt: `tax_exempt_body_exception` |
| TR 2007/12 example 13 | A benefit under salary sacrifice | Not exempt: `not_salary_sacrifice` |

The examples give no amounts beyond 'less than $300'; the values in the tests
are fabricated under it. The tests also cover $300 itself and $299.99, each
exclusion alone and together, both s 58P(1)(d) exceptions, mixed criteria left
to judgement, and each refusal.

## PPSR timing

Added on 20 November 2026. Its sources were read that night, 12.00 am to
12.15 am AEDT, from the Federal Register of Legislation: the Personal Property
Securities Act 2009, compilation 22 (in force from 14 October 2024, the latest),
ss 10 ('business day'), 62, 151, 153, 167, 168, 179, 293 and 294; the Personal
Property Securities Regulations 2010, compilation 6 (from 1 July 2022, the
latest), whose only use of 'business day' is in 2 notice rules, so they
prescribe no day for the s 10 definition, and whose reg 3.1 sets the class of
collateral for s 153 item 7; the Personal Property Securities (Fees)
Determination 2018 as made (in force from 1 August 2018 with no later
compilation, made by the Attorney-General under s 190); and the Corporations
Act 2001, compilation 149 (19 September 2026 to 30 November 2026), ss 9
('business day'), 588FL, 588FM and 588FN. On 20 November 2026 at about
1.40 am those sections were compared with compilation 142 (from 1 July 2025):
the text is the same. The Register lists versions from
1 December 2026, 4 December 2026, 8 April 2027 and 1 July 2027, made by Acts
Nos. 61, 35, 38 and 57 of 2026 and No. 72 of 2025; none of them amends the
'business day' definition or ss 588FL to 588FN, so the command takes a
critical time on any day of its 2 income years, 1 July 2025 to 30 June 2027,
and refuses one outside them. The Determination has no later version before its
sunset on 1 October 2028. The Registrar's 'Fees for using the PPSR' page, read
at 12.03 am and again at about 1.40 am, gives the same amounts and says they are not
subject to GST. It names the Treasurer in one paragraph and the Attorney
General in another; the Determination itself is the Attorney-General's.

The Determination's tiers are by end time: up to the end of the day 7 years
after the registration time ($6.00), more than that and up to the end of the
day 25 years after ($25.00), and no end time ($115.00). An end time on the
7 year day is the lower tier. s 153(2) takes a non-complying end time, which
s 153(3) says includes one earlier than the registration time, to be the
default time; the command applies it rather than refusing, and says so.

Counting follows the Acts: '15 business days after the day' (PPSA s 62(3)(b))
and '20 business days after the security agreement ... came into force'
(Corporations Act s 588FL(2)(b)(ii)) start on the next day, and each ends at
the end of the last business day. '6 months before the critical time' is the
same day and time 6 months earlier; where that day does not exist (31 August
less 6 months) the command refuses, as neither Act settles it. A registration
at the latest time is not 'after' it, so does not vest. The registrar's timing
page speaks of 20 working days after the agreement is signed; the Act counts
from when it came into force, and the command follows the Act.

No published example with dates was found on the Registrar's site, so
`tests/ppsr_timing.rs` holds constructed cases, each checked by hand against
the calendar:

| Case | Facts given | Result |
| --- | --- | --- |
| Consumer, registered 19 November 2026 | No end date | Ends 19 November 2033, $6.00 |
| Commercial, same day | No end date; 10 years; 7 years; 7 years and a day | $115.00; $25.00; $6.00; $25.00 |
| Serial numbered, end asked for in 2040 | Past the 7 year default | Default time, $6.00 |
| Non-inventory goods, possession 21 December 2026 | 25 December and 1 January listed | Register by the end of 19 January 2027 |
| Inventory, possession 2 November 2026, 3.00 pm | Registered at 3.00 pm; 3.01 pm | Super-priority; none |
| Company, agreement 12 October 2026 | 3 November listed, or not | 10 November, or 9 November |
| Same, critical time 20 November 2026 | Registered 10 November 4.00 pm; 11 November | Does not vest; vests |
| Company, agreement 2 November 2026, critical time 1 March 2027 | 3 November listed; registered 25 November, or 2 December | Latest time the end of 1 December: does not vest; vests |
| Agreement 2 February 2026, critical time 1 November 2026 | Registered 15 April; 1 May 10.00 am; 1 May 10.01 am | Latest time 1 May 10.00 am: not, not, vests |

## Wine equalisation tax

This worksheet was added on 19 November 2026. The WET Act was read that day
at the ATO's legal database (ss 5-5, 5-50, 7-10, 9-5, 9-25, 9-30, 9-35, 9-40,
9-45, 13-5, 19-5, 19-15, 19-20, 19-25, 31-1 and 33-1), and ss 19-15 and 19-25
in compilation 32 (1 April 2026) at the Federal Register of Legislation, with
Schedule 6 to the Treasury Laws Amendment (Supporting Choice in
Superannuation and Other Measures) Act 2026 (No. 12, 2026), which raises the
cap to $400,000 for assessable dealings from 1 July 2026. WETR 2009/1 and
WETR 2009/2 were read as the ATO publishes them. The ATO's pages
'Wholesale sales' and 'Retail sales and own use' (QC22750 and QC22751, both
last updated 19 March 2025), 'Producer rebate' (QC22756, 30 June 2026),
'Calculate and claim your rebate' (QC22758, 2 July 2026), 'Reporting and
paying WET' (QC22762), 'Wine equalisation tax (WET)' (QC33703, 1 May 2025)
and 'Instructions for completing your BAS' (QC17458, 14 September 2026) were
read that day, as was the Library's GST collection at ¶22-280 and ¶22-290.

The Act fixes no value basis for the 10% wholesale share of s 9-25(3), so the
command takes it as a stated fact. The ATO rounds its tasting bottle's
notional wholesale selling price to $5.83 where this command gives $5.84; the
tax, $1.69, is the same. Years before 2023-24 are refused, as the earlier
rebate reductions for wine of 2017 and earlier vintages sold before 1 July
2023 are not worked.

`tests/wine_equalisation_tax.rs` holds these cases.

| Case | Source | Worksheet |
| --- | --- | --- |
| Alison sells a dozen for $120 before WET and GST | QC22750 | WET $34.80, GST $15.48, $170.28 in all |
| Swillo sells 5 cases at $120 | Library ¶22-280 | WET $174.00, GST $77.40, $851.40 in all |
| A dozen at the cellar door for $140 | QC22751 (Happy Valley) | Taxable value $70.00, WET $20.30 |
| A tasting bottle normally sold for $11.67 | QC22751 | WET $1.69, no GST |
| A $22 retail sale; a staff sale taxed at the normal $150 | WETR 2009/1 example 15 and para 159 | WET $3.19 and GST $2.00; WET $21.75 |
| 70% of wholesale sales at $80 a dozen and 30% at $90, then a dozen at the cellar door for $190 | WETR 2009/1 example 16, QC22751 (Good Wines) | Average $83.00, WET $24.07, GST $17.27 |
| 2026-27 rebates on a retail, a wholesale, 2 quoted and a tasting dealing | QC22758's table | WET $11,890, rebate $16,240 |
| A $50,000 wholesale sale with $340,000 (2025-26) or $390,000 (2026-27, 2027-28) claimed earlier | Worked by hand under s 19-15(2) | Rebate earned $14,500, $10,000 within the cap |
| A hamper's wine share, $150 | Library ¶22-280 | WET $43.50 |

## Luxury car tax

This worksheet was added on 19 November 2026. The LCT Act was read that day
at the ATO's legal database (ss 5-15, 5-20, 7-15, 9-5, 9-15, 15-20, 15-25,
15-30, 18-5, 18-10, 25-1 and 27-1, with their history notes), and in
compilation 27 (1 April 2025) at the Federal Register of Legislation, with
Schedule 1 to the Treasury Laws Amendment (Tax Incentives and Integrity) Act
2025 (No. 29, 2025), which lowers the fuel-efficient rating to 3.5 litres
for supplies and importations from 1 July 2025 and keeps 7 litres for a car
supplied or imported and used other than for a quotable purpose before then
(item 4(2)). The rate, 33%, is s 4 of the Luxury Car Tax Imposition (General)
Act 1999 and of the (Customs) Act. The ATO's 'Luxury car tax rate and
thresholds' (QC38161, last updated 1 June 2026) gives both thresholds for
each financial year from 2012-13 to 2026-27; 'Working out the LCT on a sale'
(QC22107) and 'on an import' (QC22108), both 1 June 2026, give the
examples; 'Luxury car tax credits and refunds' (QC39467, 16 July 2025) says
refunds are claimed on the approved form within 4 years and not on the BAS;
and the BAS page 'Luxury car tax (LCT)' (QC33694, 1 May 2025) gives labels
1E and 1F. The Library's ¶12-340 (reviewed 30 June 2026) gives the same
2025-26 thresholds.

The formula's 33% x 10/11 is exactly 0.3, so each line's tax is exact before
rounding to the cent. Tax periods start at 2019-20 because ss 18-5(3) and
18-10(3), as substituted by Act No. 94 of 2019, give the $10,000 refunds only
for cars supplied or imported from 1 July 2019; earlier refunds (8/33 of the
tax, at most $3,000) are not worked. 2027-28 is refused until the ATO
publishes its thresholds.

`tests/luxury_car_tax.rs` holds these cases.

| Case | Source | Worksheet |
| --- | --- | --- |
| A car sold for $88,000 including GST in 2026-27, not fuel efficient | QC22107 (Matty Bee Motors) | $2,157.30; charged $90,157.30 |
| A 1977 Mustang imported in 2026-27 with an LCT value of $101,500 | QC22108 (Jimmy) | $6,207.30, paid to Home Affairs |
| Both thresholds for each year from 2019-20 to 2026-27 | QC38161; Library ¶12-340 for 2025-26 | The year's figures |
| A $95,000 car rated 5 litres in 2026-27; the same under item 4(2); one rated 3.5 litres | Worked by hand under s 25-1(4) and Act No. 29 of 2025 Sch 1 item 4 | $4,257.30; $1,001.70; $1,001.70 |
| A $100,000 sale with $2,000 or $6,000 of earlier tax on the car | Worked by hand under s 5-15(2) | $3,757.30; nil |
| A 2025-26 sale's price cut to $85,000 after $2,157.30 was attributed | Worked by hand under s 15-25 | $827.40 decreasing adjustment at 1F |
| A car bought under quote for $100,000 in 2025-26 then used privately | Worked by hand under s 15-30(3) and (4)(a) | $5,829.90 increasing adjustment at 1E |
| Refunds on $12,000 and $4,000 of tax borne | ss 18-5(3) and 18-10(3) | $10,000 and $4,000, off the BAS |

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
deducted (s 25-25(3), note). From 19 November 2026 a loan first drawn on
29 February is worked out, its 5 years ending on 28 February, and income
years from 2017-18 to 2027-28 are taken, 2027-28 flagged for s 26-155; see
[Borrowing expenses schedule](#borrowing-expenses-schedule) for both.

## Borrowing expenses schedule

This worksheet was added on 19 November 2026. ITAA 1997 s 25-25, its
history notes, s 26-50 and s 26-155 were read on the ATO's legal database
that day, with the ATO's Borrowing expenses page (updated 22 May 2026) and
the Acts Interpretation Act 1901 (compilation 39, 28 March 2026) on the
Federal Register. `tests/borrowing_expenses_schedule.rs` holds each case;
every row also equals `borrowing_expenses` for that loan and year.

| Case | Source | Worksheet | Published |
| --- | --- | --- | --- |
| Peter and Fiona and Max in one run, every year | ATO page | $318.07, $319.83, $319.82, $320.70, $319.83, $1.75; $259.62, $271.53, $271.53, $272.27, $271.53, $11.90 | Peter's $318.07 and $1.75 and $1,598.25 by Year 5; Fiona and Max's table |
| Ben, every year: 303 of 1,461 days, 365 of 1,158, 366 of 793, 365 of 427, 62 of 62 | ¶4-160 | $746.61, $899.38, $901.85, $899.39, $152.77 | $747, $899, $902, $899, $153 |
| A $1,500 loan refinanced on 31 December 2025 by an $800 loan | Built from s 25-25(4) and (5)(b) | $311.09, $1,188.91; $79.74 to $80.18 over 6 years | None published |

s 25-25 has had its present text since Act No 15 of 2017 substituted step 1
of s 25-25(4) from 1 April 2017 (correcting 'the the'); the rest dates from
Act No 121 of 1997 and Act No 79 of 2010's note. The worksheet takes loans
first drawn from 1 July 2017. s 26-155 was inserted by Act No 49 of 2026 for
the 2027-28 income year and later years (history note read 19 November
2026). It quarantines the excess of residential deductions over residential
income after the other provisions have worked the deductions out, so it does
not change the s 25-25 amount: the year is worked out and flagged.

s 25-25(5)(c) says '5 years starting on the first day on which the money was
borrowed'. Acts Interpretation Act 1901 s 36(1) item 2 counts that day.
Read as 60 months, s 2G(2) ends the period immediately before the same day
of the month 60 months on or, where that month has no such day, at its end:
a loan first drawn on 29 February 2024 runs to 28 February 2029, 1,827 days.
The anniversary reading (immediately before 29 February 2029, which does not
exist) gives the same day, so both readings agree.

## Hire purchase schedule

This worksheet was added on 15 November 2026. ITAA 1997 ss 240-17, 240-55 and
240-60 were read that day on the ATO's legal database and in compilation 268
(1 October 2026) on the Federal Register of Legislation. Section 240-60(1)'s
method statement adds earlier notional interest to the notional loan
principal (step 1), subtracts arrangement payments made or due (step 2), works
out the implicit interest rate for the period from the payments and any
termination amounts (step 3, as substituted by No 79 of 2010 from 1 July 2010)
and multiplies (step 4). The worksheet takes each month as an arrangement
payment period, so an instalment in advance is due at the start of its month
and comes off before the month's interest. It finds the rate by halving 0 to
100% a month 64 times on the sign of the balance left after the last payment,
which is nil at the implicit rate, negative below it and positive above it,
and refuses where that balance does not then round to nil.

`tests/hire_purchase.rs` holds these cases. The ATO's pages and the Library
give no worked hire purchase schedule. LodgeiT Labs' public repository
(`lodgeit-labs/clawdog-calculator-api`, `tests/fixtures/hp/anchors_v1_4.json`
at e00f6f87, read on 15 November 2026) publishes 4 schedules from LodgeiT's
printed hire purchase reports, each at a stated nominal rate and leaving a
residual; its `hp:schedule` OpenAPI document and `llms.txt` give no example.
The last 2 rows below take 2 of those agreements' payments.

| Case | Source | Worksheet |
| --- | --- | --- |
| $1,000 financed, $100 a month in arrears for 2 months, $1,000 balloon | Worked by hand: 10% of $1,000 is each $100 instalment | 10% a month; $100.00 interest each month |
| $1,000 financed, $100 a month in advance for 2 months, $979 balloon | Worked by hand: ($1,000 - $100) × 10% = $90; ($990 - $100) × 10% = $89 | 10% a month; $90.00 and $89.00 |
| $45,000 financed, $850 a month in arrears for 60 months, $5,000 balloon | Fabricated; an independent calculation from the payments' present value to 60 digits | 0.0068669766960432 a month; first month $309.01, last $39.90; $11,000.00 in all |
| The same, in advance | As above | 0.0070812410136161 a month; first month $312.64, last $35.16 |
| $10,000 financed, $500 a month in arrears for 24 months | As above | 0.0151308439023100 a month; first month $151.31, last $7.45; $2,000.00 in all |
| $1,000 financed, one $1,050 instalment in arrears; and $100 in advance with a $945 balloon | Worked by hand: 5% of $1,000 is $50; ($1,000 - $100) × 5% = $45 | 5% a month; $50.00 and $45.00 |
| LodgeiT hp/290: $180,630.00 financed, $3,346.99 a month in arrears for 60 months, stated 4.25% a year | LodgeiT's report: first month $639.73, last $11.81, $20,189.64 in all and $0.24 left | 4.25% a year to 2 places; first month within 2 cents, last $11.81; $20,189.40 in all, nil left |
| LodgeiT hp/235: $252,450.00 financed, 59 instalments of $3,893.76 and a $53,893.16 balloon, stated 3.99% a year | LodgeiT's report: first month $839.40, last $178.60, $31,175.69 in all and $0.69 left | 3.99% a year to 2 places; first month within 2 cents, last $178.60; $31,175.00 in all, nil left |
| By income year: $1,000 financed, $100 a month in arrears for 3 months from 1 May 2027, $1,000 balloon | Worked by hand: $100 a month at 10%; May and June fall in 2026-27, July in 2027-28 | 2026-27, instalments 1 and 2, $200.00; 2027-28, instalment 3, $100.00; no month spans 30 June |
| Across 30 June: the same for 2 months from 15 May 2027; and $100 in advance with a $979 balloon from 15 June 2027 | Worked by hand: 16 days from 15 June to 1 July and 14 to 15 July | In arrears, 2026-27 $100.00 and month 2 ($100.00) set apart; in advance, month 1 ($90.00) set apart and 2027-28 $89.00 |

Section 240-60(2), read on 16 November 2026 on the ATO's legal database
(https://www.ato.gov.au/law/view/document?docid=PAC/19970038/240-60), relates
the part of an arrangement payment period in an income year to it 'in
accordance with generally accepted accounting principles'. It names no
method, so `interest_by_income_year` totals only the months wholly in each
year and `interest_across_30_june` sets out a spanning month's interest and
its days on each side for the caller to relate. A day-weighted split was not
adopted: it is one reading of those principles, and a split at the implicit
rate compounded within the month differs from it by cents.

### Frequencies, segments, payouts, journals and GST

Added on 18 November 2026. ITAA 1997 Division 240 (ss 240-17 to 240-115),
s 40-230 and the s 995-1 definitions of 'hire purchase agreement' and
'termination amount' were read that evening (22:48 to 22:50 AEDT) in
compilation 268 (1 October 2026, the latest),
https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_5/document_5.html
(volumes 2 and 10 for s 40-230 and s 995-1), the s 240-105(4) formula from
its text alternative ('Notional loan principal plus Assessed notional
interest'). GST Act ss 9-90, 11-5, 11-15, 11-25, 11-30, 29-10, 69-10 and
158-5 were read at 22:51 in compilation 96 (1 January 2026, the latest),
https://www.legislation.gov.au/C2004A00446/2026-01-01/2026-01-01/text/original/epub/OEBPS/document_1/document_1.html
and document 2. The ATO's pages were read from 22:54 to 22:57: 'GST – Hire
purchase and leasing' (last updated 6 April 2017), 'Purchasing a motor
vehicle' (last updated 3 June 2026), the GST issues register's 'Input tax
credits and hire purchase - acquisition of a car' and 'Assets and exclusions'
(last updated 27 May 2026: $69,674 for 2024-25 and 2025-26). An independent
review reread these on 18 November 2026 from 23:50 AEDT, with ss 240-55 to
240-110 and GST Act ss 9-90 and 69-10, and read the GST issues register
'Financial services - questions and answers', 'Hire purchase agreements and
chattel mortgage', issues 1.1, 1.3, 1.9, 1.15 and 1.17,
https://www.ato.gov.au/law/view/print?DocID=GIR%2FFinancial-services-ch1&PiT=99991231235958,
and checked 3 dated-segment agreements (months with a holiday, quarters and a
year split in 2; months in advance; weeks and fortnights in advance, each
with a period across 30 June) against a separate script that solves the
rate and the periods' interest from the dated payments: the rates agree to
16 places, each period to within a cent, and the income years and spanning
periods add up to the total interest.

- **Periods.** Section 240-70(1) makes an arrangement payment period the
  period a payment is allocated to, and s 240-70(2) splits one over 6 months
  into parts of 6 months, so an annual instalment's year is 2 periods. Section
  240-60(1) step 3 works out the implicit rate 'for the arrangement payment
  period'. The worksheet states its convention: one compound rate a month (a
  week for weekly and fortnightly instalments), and (1 + r)^n - 1 for a period
  of n units, so periods of different lengths in one agreement share one
  rate. A month with no instalment (a payment holiday, or the first half of
  an annual period) is its own period with nil payment; compounding monthly,
  this moves no interest between periods other than how it is grouped.
  Segments mixing weeks and months are refused, as no one unit counts both.
- **Deposit and trade-in.** Section 240-25(3) makes the notional loan
  principal the consideration less any amount paid, or credited by the
  notional seller as paid, at or before the start for the cost of the
  property. The amount financed stays the principal, and the deposit and
  trade-in only add to the consideration the asset is booked at.
- **Payout.** A payout is a 'termination amount' (s 995-1: an amount payable
  because the arrangement ends, including one for acquiring the property), so
  not an arrangement payment (s 240-65(b)), and the arrangement ends when it
  ceases to have effect (s 240-75(1)(b)). Subdivision 240-G then compares the
  payments and termination amounts with the notional loan principal plus the
  assessed notional interest: an excess is the notional seller's assessable
  income (s 240-105(2)) and the hirer may deduct a corresponding amount
  (s 240-110(1)); a shortfall is the seller's deduction (s 240-105(3)) and the
  hirer's assessable income (s 240-110(2)), each to the extent s 240-110(3)
  allows. With every earlier payment made, that difference is the payout
  less the balance outstanding. The Division gives no notional interest for
  part of a period that a payout cuts short (s 240-60(2) apportions only
  between income years), so a payout must fall on the end of a period. The
  implicit rate stays the one the scheduled payments give, which
  s 240-60(1) step 3 and (3) support only where the payout was neither known
  nor expected at the start, so the reasonable estimate of termination
  amounts then was nil. The caller states that in `known_at_start`; `true`,
  which also covers a case no reasonable estimate could be made of at the
  start (s 240-60(4)), is refused. The result flags the rest for
  review. A first use year before the agreement is taken as given, as for a
  car first used under an earlier hire (s 69-10(1)(c)).
- **GST.** For an agreement entered into on or after 1 July 2012 every
  component of the supply is taxable, the credit component included, and the
  credit is 1/11 of all of them (the ATO page; its Albert example claims
  1/11 of $40,200 on a $33,000 freezer). The consideration is therefore the
  price (amount financed, deposit and trade-in) plus the notional interest,
  which is the deposit, trade-in and arrangement payments. Luxury car tax is
  left out of the value (s 9-75(2)), and s 9-90(1) rounds GST to the nearest
  cent, 0.5 up (s 9-90(2) gives the same where the price and the credit
  component are 2 supplies on one invoice), so Albert's credit is $3,654.55
  where the ATO's page prints $3,654.54, a cent less than one-eleventh of
  $40,200 rounded. The full input tax credit is that rounded GST (s 11-25),
  and s 11-30(3) multiplies it by the extent of creditable purpose: $1,827.28
  for Albert at 50%. A hirer accounting on a cash basis is treated as not doing so for
  a hire purchase acquisition (s 158-5), so the whole credit is attributable
  to the tax period in which consideration is first provided or, before
  that, an invoice is issued (s 29-10(1)). Section 69-10(1) caps a car's
  credit at 1/11 of the car limit for the financial year it is first used,
  where the hirer may not quote an ABN for luxury car tax and its GST
  inclusive market value exceeds that limit, and s 69-10(3) reduces it for
  creditable purpose. Section 69-10(4)(b) excludes 'the acquisition of a car
  by lease or hire'; the ATO's financial services register (issue 1.17,
  updated 7 September 2012, after the change for agreements from 1 July 2012)
  applies the cap to a car acquired under a hire purchase agreement, since for
  GST a hire purchase agreement is a sale of goods (issue 1.1). The same issue
  1.1 distinguishes the sale of goods from the provision of finance. When
  a held tax invoice establishes both components and their GST agrees with
  separate cent rounding, s 69-10(1) caps the goods credit only. The worksheet
  adds the separately shown finance GST and then applies the creditable-purpose
  fraction. A nil finance charge carries nil GST. For an invoice that uses
  another rounding method or does not establish both components, the preparer
  reviews the figures. The worksheet leaves attribution and journals for the
  split supplies to the preparer.
- **Journals.** Each period's interest is an entry at its end, grouped with
  its period's income year, and each payment an entry on its day, grouped
  with that day's year, so an instalment paid on 1 July counts in the year it
  starts.

Left out and why: substituted accounting periods (`income_year` runs from
1 July, and a substituted period would change the years and the 30 June
split throughout, with ITAA 1936 s 18 not read); fees, charges and stamp duty
(their GST and Division 240 treatment were not worked); the car's cost for
Division 40 (s 40-230 was read, but s 27-80 on a credit that includes the
credit component was not); any reduction of the notional interest deduction
for its input tax credit; agreements entered into before 1 July 2012; GST
adjustments after the start; and the trade-in's own GST, which `trade-in`
works.

`tests/hire_purchase_worksheet.rs` holds these cases, and
`tests/properties.rs` holds random regimes to the totals, the 30 June split,
a present value check and balanced journals.

| Case | Source | Worksheet |
| --- | --- | --- |
| $1,000 financed, 2 quarterly instalments of $331 in arrears, $1,000 balloon | Worked by hand: 1,000 x 1.1^3 = 1,331 | 10% a month; $331.00 each quarter |
| $1,000, 2 weekly instalments of $100 and a $1,000 balloon; $1,210 once fortnightly | Worked by hand: 10% a week; 1,000 x 1.1^2 = 1,210 | 10% a week, 5.2 a year; $100.00 a week; $210.00 a fortnight |
| $1,000, one annual instalment of $2,250 in arrears | Worked by hand: 50% a half-year, 1,500 then 2,250 | 2 periods of 6 months, $500.00 then $750.00 |
| $1,000, 2 nil months then $1,331 | Worked by hand at 10% a month | $100.00, $110.00 and $121.00 |
| $1,000, $100 after a month then $1,331 a quarter later | Worked by hand at 10% a month | $100.00, then $331.00 |
| The 2-month agreement with a $100 deposit and $250 trade-in | s 240-25(3) | $1,350.00 consideration; asset $1,350.00 against a $1,000.00 liability |
| $1,000, 3 months of $100 in arrears and a $1,000 balloon, paid out after 2 months | Worked by hand: $200 interest, $100 paid, $1,100 outstanding | $50.00 excess at $1,150 (s 240-110(1)), $50.00 shortfall at $1,050 (s 240-110(2)) |
| The 2-month agreement with a $100 deposit, registered, fully creditable | Worked by hand: 1,300 / 11 and 200 / 11 | $118.18 credit, $18.18 of it on the interest; asset $1,000.00 |
| Albert's second freezer: $33,000, 60 x $670 | 'GST – Hire purchase and leasing'; s 9-90 | $3,654.55 (the ATO prints $3,654.54); $1,827.28 at 50% |
| Darren's car: $91,242.80 with $2,407.80 luxury car tax, 2026-27, financed by 60 x $1,800 | 'Purchasing a motor vehicle'; s 69-10(1) | No figure, `gst.review` |
| A $60,000 car, 60 x $1,300; the same car worth $80,000, quoted for or not; worth $69,700, first used in 2026-27 or 2024-25 | GST Act s 69-10(1)(b) and (c) | $7,090.91 twice; no figure; $7,090.91; no figure |
| A $91,242.80 car including $2,407.80 luxury car tax, with $16,757.20 in separate finance charges supported by a held invoice | GST Act ss 9-75(2), 11-25 and 69-10(1) and ATO GST issues register 1.1, 1.2 and 1.17 | $6,353.00 goods credit plus $1,523.38 finance credit, yielding $7,876.38 |
| $1,000, 2 x $100 in arrears and a $1,000 balloon from 1 June 2027; the same in advance | Worked by hand | June's $100 interest in 2026-27 and its payment on 1 July in 2027-28; in advance, $100 paid 1 June in 2026-27 |
| $50,000, 260 weekly payments of $230 and a $5,000 balloon | The payments discounted at the rate shown | $50,000 to within a cent |

## Chattel mortgage schedule

This worksheet was added on 19 November 2026, with every source below read
that day (AEDT). ITAA 1997 compilation 268 (1 October 2026): s 8-1 in
volume 1 at 09:27, s 240-10 in volume 5 and s 995-1 'hire purchase
agreement' in volume 10 at 09:30. GST Act compilation 96 (1 January 2026):
ss 9-90, 29-10, 40-5 and 69-10 at 09:27. The ATO's GST issues register
'Financial services - questions and answers' (issues 1.1, 1.14, 1.17 and
1.19 to 1.21) and its issue 1.18 public ruling at 09:27 to 09:29; TR 93/16 at
09:31; 'Purchasing a motor vehicle' (last updated 3 June 2026) at 09:27; and
'Assets and exclusions' (last updated 27 May 2026: $69,674 for 2024-25 and
2025-26) at 09:40. The Library's GST ¶12-110 (reviewed 3 June 2025) gives
the same car limits.

- **Not hire purchase.** Under a chattel mortgage the purchaser takes title
  from the purchase and finances the price with a loan (register issue
  1.20). A hire purchase agreement is a hire of goods, or an instalment
  purchase, under which title does not pass until the option is exercised or
  the final instalment is paid (s 995-1), and only such an agreement is
  recharacterised as a notional sale and loan (s 240-10). So the interest is
  the lender's actual charge, not notional interest.
- **Interest.** TR 93/16 accepts the actuarial method for estimating the
  interest component of instalments under a fixed term loan (para 6),
  describes it as the outstanding balance times the rate for the instalment
  period (para 18) and calls it the most appropriate for long loans
  (para 29); where the interest paid in the year is known, that figure must
  be used (para 2). Given the instalment, the rate is found as
  `hire-purchase-schedule` finds the implicit rate. Given `annual_rate`, the
  rate a period is `annual_rate / 12` (or `/ 52`), taken only for monthly or
  weekly instalments in arrears, where that division is unambiguous; other
  frequencies and instalments in advance are refused rather than guessed. The
  instalment is the closed-form annuity payment, worked out from the
  balance's fall per dollar of instalment, rounded half up to the cent, with
  the balance that rounding leaves added to the last instalment.
- **Deduction.** Section 8-1(1) allows a loss or outgoing to the extent it is
  incurred in gaining or producing assessable income, or necessarily in
  carrying on a business for that purpose, and s 8-1(2) denies it to the
  extent it is capital, private or domestic. The worksheet states that rule
  and leaves the extent to the operator. A period spanning 30 June is set
  apart with its days on each side, as `hire-purchase-schedule` sets it
  apart: the sources read do not say how to split it between the years.
- **GST.** The supply of the loan is an input taxed financial supply
  (s 40-5(1); register issue 1.14), so the input tax credit is on the price
  alone: 1/11 of it, less luxury car tax for a car (s 9-75(2)), rounded under
  s 9-90(1). For a car worth more than the car limit for the year it is first
  used, where no ABN may be quoted, s 69-10(1) makes the credit 'the amount
  of GST payable on the supply ... of the car up to 1/11 of that limit', and
  s 69-10(3) reduces it for a partly creditable acquisition. Under register
  issue 1.21 a purchaser accounting on a non-cash basis is entitled to the
  whole credit in the tax period in which the invoice is received or any
  payment is made, whichever is earlier (s 29-10(1)), and one on a cash basis
  in the tax period in which the borrowed funds pay the full price at the
  time of acquisition (s 29-10(2)(a)).

`tests/chattel_mortgage.rs` holds these cases.

| Case | Source | Worksheet |
| --- | --- | --- |
| $1,000 at 12% a year, 3 monthly instalments in arrears | Worked by hand: 1.01^3 = 1.030301, 10 x 1.030301 / 0.030301 = 340.0221 | $340.02 twice and $340.03; interest $10.00, $6.70, $3.37 |
| $10,000 at 12% with a $10,000 balloon; $1,000 at 52% weekly with a $1,000 balloon | Worked by hand: interest only | $100.00 a month; $10.00 a week |
| $45,000 at 7.99%, 60 months, $5,000 balloon | The closed-form annuity payment, PMT(0.0799/12, 60, -45000, 5000), in exact fractions | $844.16, last $843.87, interest $10,649.31; given $844.16, the implied rate is 7.99% |
| $20,000 at 6.5%, 104 weeks | The closed-form annuity payment in exact fractions | $205.20, interest $1,340.63 |
| $50,000 over 360 months at 30% and at 50% | The rounded instalment's shortfall carried at the rate, in exact fractions | At 30%, $1,250.17 with $678.01 added to the last; at 50%, $2,083.33 would leave $242,950.50, more than a whole instalment, so it is refused, as is a last instalment below nil |
| $1,000, 2 x $100 and a $1,000 balloon; the same in advance with $979 | As `hire-purchase-schedule` | 10% a month; $90.00 and $89.00 |
| Darren's car: $91,242.80 with $2,407.80 luxury car tax, 2026-27, $10,000 deposit | 'Purchasing a motor vehicle'; s 69-10(1) and (3) | GST $8,075.91; credit $6,353.00, $3,176.50 at 50%; $8,075.91 when an ABN is quoted |
| $33,000 of other goods with a $3,000 trade-in, 50% creditable | s 9-75(1), s 11-30(3) | $3,000.00 GST; $1,500.00 credit |

## Loan transactions

This worksheet was added on 19 November 2026; ITAA 1997 s 8-1 (compilation
268, volume 1) was read that day at 09:27 AEDT. A day's transactions and any
rate change take effect on that day, and its interest is the balance at the
end of the day, when above nil, times the annual rate then in force over 365,
in leap years too. The interest is debited on the last day of each month and
on `end_date`, rounded half up to the cent, so 30 June always closes a debit
and each income year's interest is exactly what is debited in it. These are
the worksheet's conventions, stated in its scope, not rules of law: a lender
who counts days, rests or rounding differently gives different figures, and
its statement prevails. A balance below nil at the end of a day is refused,
as is everything the scope excludes.

`tests/loan_transactions.rs` works these cases by hand at 7.3% a year,
exactly $0.0002 a dollar a day.

| Case | Worksheet |
| --- | --- |
| $10,000 drawn 1 July 2026, $1,000 repaid 31 August, to 30 September | $62.00, $62.18 and $54.75 interest; $9,178.93 owing |
| The same with 3.65% from 1 August | August's interest $31.09 |
| $10,000 drawn 1 June 2026, to 31 July | $60.00 in 2025-26 and $62.37 in 2026-27 |
| Paid out on 10 July with $10,018.00 | $18.00 interest, nil owing; $10,018.01 refused |
## Contract register

`contract-register` works no amount of its own: each contract's payments,
interest and principal are those of its `hire-purchase-schedule` or
`chattel-mortgage-schedule` result, unchanged, and the register only dates
and adds them. Its scheduled 12-month split uses the due-date test in
AASB 101 paragraph 69(c), AASB 18 paragraph 101(c) and AASB 1060 paragraph
40(c), read on 21 November 2026. The preparer determines the applicable
standard and assesses other classification grounds, including deferral
rights and covenants. See [Contract register](contract-register.md) for
application dates.
[AASB 101](https://standards.aasb.gov.au/node/3592),
[AASB 18](https://standards.aasb.gov.au/node/3717),
[AASB 1060](https://standards.aasb.gov.au/node/4270).

ITAA 1997 s 240-60(2), already read for `hire-purchase-schedule`, leaves
the interest of a period spanning 30 June to accounting principles, and a
chattel mortgage's interest is deductible as each year incurs it (s 8-1),
so the register sets it apart with its days rather than apportioning it.

`tests/contract_register.rs` holds the fabricated register in
`examples/contract-register.csv`, 8 contracts with monthly, weekly,
fortnightly, quarterly and annual instalments, in arrears and in advance,
starting from 15 September 2024 to 20 August 2026, to each contract's own
schedule run in the test on facts written out by hand: every total by month,
income year, financier, client and register is the sum of the schedules'
amounts, and each amount outstanding follows the schedule's closing balance.

| Case | Source | Register at 30 June 2026 |
| --- | --- | --- |
| $10,000 financed, 12 monthly instalments of $100 in arrears from 1 March 2026, $10,000 balloon | Worked by hand: 1% a month; 4 periods ended charge $400, 3 instalments paid $300 | $10,100.00 outstanding, all current |
| $24,000 financed, 24 monthly instalments of $1,000 in arrears from 1 August 2025 | Worked by hand: nil rate; 10 paid, 12 due by 30 June 2027 | $14,000.00 outstanding, $12,000.00 current, $2,000.00 non-current |
| Monthly from 15 September 2024 | Worked by hand: 15 to 30 June is 16 of the period's 30 days | `accruing_days_elapsed` 16 of 30 |
| $10,000 financed, 24 monthly instalments of $10, $13,000 balloon | The balance builds up over the next 12 months | all non-current |
| Chattel mortgage: $45,000 at a stated 7.99% from 15 July 2025, 60 monthly instalments in arrears, $5,000 balloon | Its own `chattel-mortgage-schedule` | The 11th row's closing balance outstanding; current, its fall to the 23rd row's; 16 of 30 days accruing |

## Business capital expenditure

This worksheet was added on 16 November 2026. ITAA 1997 s 40-880 was read
that day in compilation 268 (1 October 2026, registered as C2026C00471, the
latest compilation by the Federal Register of Legislation's API at 18:08
AEDT), volume 2. Its endnotes list the section's last amendment as Act No 92
of 2020, the Treasury Laws Amendment (A Tax Plan for the COVID-19 Economic
Recovery) Act 2020, whose Schedule 3 items 25 to 27 added s 40-880(2B) and
whose item 40(7) applies them to capital expenditure incurred on or after
1 July 2020; s 40-880(2A) itself came from Schedule 2 of the Tax Laws
Amendment (Small Business Measures No. 3) Act 2015 (No 114 of 2015), for
assessments for 2015-16 and later. Both Acts were read as made that day. An
item incurred before 1 July 2020 is refused rather than worked under an
earlier form. Section 40-880(2) deducts 'in equal proportions over a period
of 5 income years starting in the year in which you incur it'; TR 2011/6
para 10 (the Commissioner's view, read on the ATO's legal database at 18:13)
adds that the expenditure is not apportioned when incurred part way through a
year. The ATO's 'Deductions for depreciating assets and capital expenses'
(last updated 28 August 2026, read at 18:11) states the start-up conditions
and gives the examples of advice on a business structure, the fee for
creating a company and stamp duty on transferring assets to the entity that
will carry on the business.

The worksheet takes 2027-28 as well, on a reading of 17 November 2026.
Neither s 40-880 nor Subdivision 328-C, which the entity tests rely on, sets
a figure by year, and the endnotes of compilation 268 list no amendment of
either after Act No 92 of 2020 and Act No 40 of 2023. The Federal Register
of Legislation's API lists one later version, from 1 December 2026, whose
amendments (Act No 61 of 2026, Schedule 1 items 41 to 45) change ss 25-110,
715-375, 715-379 and 995-1 only. From 2027-28, s 26-155 (Act No 49 of 2026,
applied to the 2027-28 and later income years) limits deductions relating
to using or holding residential dwellings as residential accommodation; the
scope leaves that limit out, as it leaves out Division 35. TR 2011/6 para 10
also states that eligibility is decided once, when the expenditure is
incurred, which is why an item's later fifths continue after its business
ceases, and paras 157 and 160 that the taxable purpose of a current or
proposed business is tested when the expenditure is incurred and that of a
former business over a period that reasonably reflects it.

`tests/business_capital_expenditure.rs` holds these cases. TR 2011/6's
examples date from 2007 to 2010, before the current form, so those with
figures are run with the year moved to 2020-21 or later; the others are
stated facts the ruling decides.

| Case | Source | Worksheet |
| --- | --- | --- |
| $10,000 incurred on 1 September | TR 2011/6 example 41 (paras 319 to 321): $2,000 a year, not apportioned for part of the first year | $2,000.00 in each of 2020-21 to 2024-25; the ruling's 2007-08 is refused |
| A company's registration fee paid by an individual not in business; advice on a business structure to a small business entity | ATO page, 'Other capital expenses' | Deducted in full in the year incurred under s 40-880(2A)(b)(ii) and (c)(i) and (ii), and (2A)(b)(i) and (c) |
| A business proposed to start once approvals that take 2 years are obtained | TR 2011/6 example 14 (paras 128 and 129) | Deductible; stated not to be within a reasonable time, excluded by s 40-880(7) |
| A business carried on 80%, 87.5% and 25% for a taxable purpose | TR 2011/6 examples 16 to 18 (paras 163 to 173); the $10,000 is fabricated | $8,000, $8,750 and $2,500 deductible |
| Expenditure on a holding company in which the taxpayer holds no shares | TR 2011/6 example 21 (paras 189 and 190) | Excluded by s 40-880(4)(b) |
| Inspecting an asset that is then bought, and one that is not | TR 2011/6 examples 22 and 23 (paras 201 to 205) | Refused under s 40-880(5)(a), pointing to Division 40; deductible |
| A feasibility study in a project pool; capital works not claimed | TR 2011/6 examples 24 and 25 (paras 209 to 212) | Excluded by s 40-880(5)(b) |
| A gaming machine entitlement | TR 2011/6 example 40A (paras 318A to 318D) | Refused under s 40-880(5)(f), pointing to the cost base; with s 40-880(6) stated, deductible |
| $1,666.67 over 5 years | Worked by hand: $1,666.67 / 5 = $333.334 | `333.334` in the item, $333.33 in the amounts |
| A member's wind-up costs not in connection with deriving assessable income from the company | s 40-880(4)(b); TR 2011/6 paras 130 to 134 and 191 | Excluded by s 40-880(4)(b) |
| $2,200 of start-up advice to an entity covered by s 40-880(2B) | Worked by hand | $2,200.00 in the year incurred |
| $10,000 incurred in 2023-24 and a $576 company fee in 2027-28 | Worked by hand: $10,000 / 5 in the fifth year; the fee in full under s 40-880(2A) | $2,576.00 in 2027-28, nothing remaining |

A fifth of an amount in cents has at most 3 decimal places, so each item is
worked exactly and only the result's amounts are rounded to the cent, as
every worksheet here shows them. Rounding each fifth to the cent with the
last year taking the rest was not adopted: neither the Act nor the ATO's
pages state a rounding for s 40-880.

## Balancing adjustment

This worksheet was added on 18 November 2026. ITAA 1997 Subdivisions 40-B
and 40-D, s 995-1's definition of a car and Subdivision 27-B were read that
day from 06:56 AEDT in compilation 268 (1 October 2026), volume 2
(https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_2/document_2.html),
and ss 104-235, 104-240, 118-5 and 118-10 in volume 3; the formulas, which the
Register renders as images, were read from their text alternatives. The
endnotes (volume 12) list s 40-285's last amendment as Act No 69 of 2014,
s 40-370's as Act No 162 of 2015, which left the cents per kilometre and log
book methods, and none for s 40-325 since it was added. Act No 49 of 2026
(Treasury Laws Amendment (Tax Reform No. 1) Act 2026, read as made at 07:06)
added a note to s 40-290 and s 40-291A, a fixed 50% reduction a taxpayer may
choose from 2026-27 for an asset used to produce assessable labour income
(Schedule 4 item 17); the worksheet applies s 40-290 and warns of the choice.
Its Schedule 1 changes capital gains from 1 July 2027, so 2027-28 is refused,
as for CGT.

The ATO's Guide to depreciating assets 2026 (current at 1 June 2026, read on
the legal database at 07:00,
https://www.ato.gov.au/law/view/document?docid=SAV/DEPRECIATING/00001) gives
the examples below, states that an asset used wholly for a non-taxable
purpose has its balancing adjustment amount reduced to nil with the
difference between termination value and cost as its capital gain or loss,
and gives the car limit for 2025-26 as $69,674.

Step 3 of s 40-370 counts 'the total number of days for which you deducted
the decline in value of the car', so a log book year with a nil deduction
adds no days. A car year before 2015-16 is refused, as the one-third of
actual expenses and 12% of original value methods could then apply and
s 40-370 as compiled now names neither. Section 40-230 reduces the first
element of cost to the car limit for the year the car was first held, and
s 40-325 uses the limit for the year it was first used, so the cost must be
that limit plus the second element; a car whose 2 years' limits differ is
refused.
The car limit is an input: the Guide states only 2025-26's, and s 40-325 uses
the limit for the financial year the car was first used, which can be years
before the event.

`tests/balancing_adjustment.rs` holds these cases. The examples state an
adjustable value, not each year's decline; the facts they leave out are
fabricated so that they give the stated figures.

| Case | Source | Worksheet |
| --- | --- | --- |
| Bridget's cabinet, wholly taxable use, adjustable value $1,200, sold for $1,300 or $1,000 | Guide, 'Example: working out an assessable balancing adjustment amount' and '... a deductible balancing adjustment amount' | $100.00 assessable; $200.00 deductible; no reduction |
| Andrew's $1,000 computer, 60% taxable use, $300 decline, sold for $600 | Guide, 'Example: depreciating asset used partly for a taxable purpose – balancing adjustment and capital loss' | $60.00 deductible, $40.00 reduction, $160.00 capital loss |
| Louise's car, log book method at 40% in 2024-25 and 2025-26, adjustable value $18,200, sold for $24,500 | Guide, 'Example: using the logbook method' | $2,520.00 assessable; K7 disregarded (s 118-5) |
| Gayle's $450 print, $31 decline, $6 deducted, sold for $700 | ITAA 1997 s 118-10(1), its example (2001-02, moved to 2025-26) | $54.39 assessable and a $201.61 capital gain; the Act rounds to $54 and $202 |
| $2,000 asset used wholly privately, $500 decline, sold for $1,800 | Worked by hand on the Guide's statement | Nothing assessable or deductible; $200.00 capital loss, nil for a motor cycle (s 118-5) |
| $30,000 car: cents per kilometre in 2024-25 ($7,500 notional decline), log book at 60% in 2025-26 ($5,625) and for 200 days of 2026-27 ($2,000), sold for $18,000 | Worked by hand under s 40-370: ($18,000 - $14,875) x (15,125 - 9,050) / 15,125 x 565 / 930 | $762.55 assessable |
| A car claimed only by the cents per kilometre method | s 40-370(1) Note 2 | Nil |
| $80,000 car first used in 2025-26, cost reduced to $69,674, $10,000 decline at 100% business use, sold for $50,000 | Worked by hand under s 40-325: $50,000 x 69,674 / 80,000 = $43,546.25 | $16,127.75 deductible |

The working keeps each fraction exact and divides once, last, so a reduction
of $3,125 x 9,050 / 15,125 is not rounded before the days scale it. The Act's
own example rounds to dollars; the worksheet shows cents, half up, as every
worksheet here does.

## CGT parcels

This worksheet was added on 19 November 2026. ITAA 1997 ss 102-5, 102-10,
102-15, 104-10, 109-5, 110-25, 110-35, 110-55, 112-30, 114-1, 114-10, 115-5 to
115-25, 115-100 and 116-40 were read that day in compilation 264 (21 May 2026),
volume 3, at 14:39 AEDT
(https://www.legislation.gov.au/C2004A05138/2026-05-21/2026-05-21/text/original/epub/OEBPS/document_3/document_3.html).
That compilation is the text for 2025-26 and 2026-27: Act No. 49 of 2026,
read at 14:40, rewrites s 102-5(1) and Subdivision 115-C, and Schedule 1
items 82 to 84 apply those changes to assessments for the income year that
includes 1 July 2027 and later years, so later compilations' step numbers are
not this worksheet's. The ATO's pages were read at 14:30: 'Keeping records of
shares and units' (last updated 22 June 2026: each parcel is a separate CGT
asset, and Boris may choose which shares he sells), 'CGT discount' (last
updated 29 June 2026: the 12 months exclude the day of acquisition and the day
of the CGT event; a complying super fund's discount is 33.33%; a company has
none) and TD 33 (the Commissioner accepts first in, first out and the
taxpayer's selection; average cost only for shares acquired on the same day).
Question 18 of the individual tax return instructions supplement 2026 (last
updated 30 May 2026) gives Kathleen's examples and labels H, A and V.

The Act does not say how a parcel's cost is split between the units sold and
those kept. Each share or unit is a CGT asset, so the worksheet takes the part
of the cost base reasonably attributable to the units (s 112-30(1) and (1A)):
elements 1, 2, 4 and 5 together, rounded once, and costs of owning (element 3)
apart, as the reduced cost base leaves them out (s 110-55(2)), and of a sale's
proceeds reasonably attributable to each part (s 116-40(1)) by number of
units, rounded to the cent, half up, with the last units taking what is left
so the parts add up.

`tests/cgt_parcels.rs` holds these cases.

| Case | Source | Worksheet |
| --- | --- | --- |
| Kathleen: 1,000 shares bought 17 November 2000 for $3,000 and sold for $6,000; 130 bought 10 October 2025 for $520 and sold for $1,040; 600 bought then for $2,400 and sold for $1,800; $400 of earlier losses | Question 18 supplement 2026, examples 1 to 4 | Label H $3,520; $520 and $80 of the loss, then $400, applied; $1,260 net capital gain; collectables left out |
| Boris: 1,000 shares at $5 and 3,000 at $10, 1,500 sold at $8 | 'Keeping records of shares and units' (days fabricated) | Selecting the $10 shares: $3,000 loss, 1,000 at $5 and 1,500 at $10 left; first in, first out: $3,000 discount gain and $1,000 loss, $1,000 net |
| David Jones: 2,000 CSR originals at $2 sold at $1 first in, first out; 1,000 of 5,000 WBC shares ($60,000) sold at $20 within 12 months; IBM loss $6,660 and TEL gain $21,435 given | Library ¶2-240 (the CSR bonus shares' $2,000 cost fabricated) | $8,660 of losses take the $8,000 WBC gain and $660 of the TEL gain; $10,387.50 net |
| Bought 2 February 2025, sold 3 February 2026 and 2 February 2026 | 'CGT discount', 12-month count | Discount capital gain from 3 February; not on 2 February |
| A $289,755 gain held over 12 months | Worked by hand under s 115-100 | $96,585 discount for a complying superannuation entity, $144,877.50 for a trust, none for a company |
| 3 units for $110 sold 1 then 2; 3 units sold for $100 from parcels of 1 and 2 | Worked by hand | Cost bases $36.67 and $73.33; proceeds $33.33 and $66.67 |
| $1,000 cost and $100 costs of owning, sold for $1,050 | Worked by hand under ss 110-25(4) and 110-55(2) | Neither gain nor loss |
| A company in 2026-27: a $100 parcel sold for nothing, $150 of other gains and $20 of earlier losses | Worked by hand under s 102-5(1) steps 1 and 2 | $30 net capital gain, nothing carried forward |

## Trade-in

This worksheet was added on 18 November 2026, beside the balancing adjustment,
whose working it reuses for the traded-in asset. ITAA 1997 ss 27-80, 27-95,
40-180, 40-185, 40-225, 40-230, 40-300, 40-305 and 40-365 were read that day
in compilation 268 (1 October 2026), volumes 1 and 2, and GST Act ss 9-5, 9-70,
9-75, 11-5, 11-25, 11-30, 69-10 and 132-5 at 07:17 AEDT in compilation 96
(1 January 2026,
https://www.legislation.gov.au/C2004A00446/2026-01-01/2026-01-01/text/original/epub/OEBPS/document_1/document_1.html
and document 2), the formulas from their text alternatives. The ATO's pages
were read at 07:10 and 07:17: 'Purchasing a motor vehicle' (last updated
3 June 2026) gives the 2026-27 car limit, $69,883, the 1/11 cap and Darren's
example; 'Disposing of a motor vehicle' (last updated 29 May 2024) says a
registered business that trades in a vehicle used solely or partly for the
business accounts for GST on it as a taxable sale; and 'GST and the disposal
of capital assets' (last updated 24 May 2017) gives the decreasing adjustment
formula, its cap and Ian's example. The 2025-26 car limit, $69,674, is the
Guide to depreciating assets 2026's.

The Act does not name a trade-in. The worksheet reads the old asset's
termination value as the liability to pay the replacement's price that the
trade-in terminates (s 40-305(1)(b) item 2), and the replacement's cost as the
liability incurred to pay its whole price (s 40-185(1)(b) item 2), which at
arm's length is the allowance plus the cash paid. The Guide's Kristine
example, where the allowance was below market value to reduce the price,
shows why a trade-in not at arm's length is refused. The car limit for
2027-28 was not published when read, and Schedule 1 to Act No. 49 of 2026
changes capital gains from 1 July 2027, so 2027-28 is refused; the car limit
for 2024-25 was not read, so 2024-25 is refused too.

`tests/trade_in.rs` holds these cases. Each ATO example is one side of a
trade-in; the other side and the years held are fabricated.

| Case | Source | Worksheet |
| --- | --- | --- |
| Ian's vehicle, bought for $44,000 including $4,000 GST, $3,200 credited at 80% business use, traded in for $22,000 as a taxable sale | 'GST and the disposal of capital assets', 'Example: Capital assets used for private purposes' | $2,000 GST, $400 decreasing adjustment, $20,400 termination value; $8,160 deductible after the $2,040 private share |
| Darren's car, $91,242.80 including $8,075.91 GST and $2,407.80 luxury car tax, 2026-27, at 100% and 50% business use | 'Purchasing a motor vehicle', 'Example: purchasing a new car for the business' | Credit $6,353.00 and $3,176.50; cost reduced to $69,883.00 |
| An old car whose cost s 40-230 reduced from $80,000 to $69,674, traded in for $55,000 including $5,000 GST | Worked by hand under s 40-325: $50,000 x 69,674 / 80,000 | $43,546.25 termination value, $16,127.75 deductible |
| A $11,000 asset (full credit $1,000, $800 claimed) traded in for $22,000 | Worked by hand under GST Act s 132-5(3): $2,000 x 0.2 = $400, capped at $200 | $200.00 decreasing adjustment, $2,000.00 K7 gain |
| An asset bought without GST (nil full and adjusted credits) traded in for $22,000 | Worked by hand under GST Act s 132-5(3): the adjustment is at most the nil difference between the credits | $2,000.00 GST, nil decreasing adjustment, $20,000.00 termination value |
| A trade-in that is not a taxable supply | GST Act s 9-5 | No GST; the termination value is the allowance |

## Payroll tax

This worksheet was added on 19 November 2026. The revenue offices' pages were
read from 09:36 to 09:37 AEDT that day:

- Revenue NSW, 'Payroll tax thresholds and rates' (last updated 1 July 2026):
  $1,200,000 and 5.45% for 2026-27, the interstate formula and its $360,000
  example.
- The Victorian State Revenue Office, 'Payroll tax (current rates)' (updated
  29 September 2026) and 'Payroll tax surcharges' (its example 2, $80 million
  in Victoria and $70 million in New South Wales).
- The Queensland Revenue Office, 'Payroll tax rates and thresholds' (last
  updated 17 July 2024), 'Payroll tax deductions' and 'Calculating the mental
  health levy' (last updated 11 August 2026).
- RevenueWA, 'Calculation: Payroll Tax Employer Guide' (last updated 2 June
  2026).
- RevenueSA, 'Rates and Thresholds'.
- The Tasmanian State Revenue Office, 'Rates and thresholds' for 2026-27.
- The ACT Revenue Office, 'About payroll tax' (Table 1 for 2026-27) and
  'Calculating payroll tax'.
- The Territory Revenue Office, 'Payroll tax rates and thresholds' and
  'Payroll changes from 1 July 2026'.

The Acts were read the same day: the Payroll Tax Act 2007 (NSW) Sch 1 (version
current from 10 July 2026); the Payroll Tax Act 2007 (Vic) s 3A and Sch 1
(authorised version 041, 1 May 2026); the Payroll Tax Act 1971 (Qld) ss 10,
29 and 43I (reprint current as at 22 October 2026); the Pay-roll Tax Assessment
Act 2002 (WA) ss 8 and 13 (04-d0-03) and the Pay-roll Tax Act 2002 (WA) s 5
(00-d0-02); the Payroll Tax Act 2009 (SA) Sch 1 (version 31.3.2026); the
Payroll Tax Act 2008 (Tas) Sch 1; the Payroll Tax Act 2011 (ACT) Sch 1
(republication R20) with the Taxation Administration (Amounts and
Rates—Payroll Tax) Determination 2026 (DI2026-151); and the Payroll Tax Act
2009 (NT) s 7A and Sch 1 (as in force at 1 July 2026). The NT deduction formula
in Sch 1 cl 3 is an image in the Act's PDF, AD = ETA - (TW + IW - ETA) / 2;
the revenue office's pages say only that the deduction settings did not change
on 1 July 2026. Every figure agrees with duguid.com.au's payroll tax calculator,
published 17 November 2026, whose cases these tests also hold.

The worksheet works each amount as its formula reads, to 40 digits, and rounds
each step to the cent. RevenueWA rounds the deduction in its 2 interstate
examples to the dollar, so its published tax is 1 and 3 cents lower than the
worksheet's. The Queensland Revenue Office gives its deduction to the dollar
($932,460); the worksheet's is $932,459.64.

For Tasmania, the worksheet rounds the tax at each rate to the cent before
adding. On $2,400,000.21 of Tasmanian wages out of $3,000,000 in Australia, the
unrounded amounts are $24,000.0021 and $48,800.00427. Rounding each gives
$72,800.00. Rounding only the sum would give $72,800.01. The 2-rate formula
comes from [Payroll Tax Act 2008 (Tas) Sch 1 cl 4(5)](https://www.legislation.tas.gov.au/view/whole/html/inforce/current/act-2008-016).

`tests/payroll_tax.rs` holds these cases, and the threshold edges, rate bands
and refusals beside them.

| Case | Source | Worksheet |
| --- | --- | --- |
| $1,500,000 of NSW wages, all in NSW | Revenue NSW: taxed on $300,000 | $16,350.00 |
| $900,000 of NSW wages in $3,000,000 Australian wages | Revenue NSW: a threshold of $360,000 | $540,000.00 taxed, $29,430.00 |
| $80 million in Victoria of $150 million | SRO surcharges example 2: thresholds of $5,333,333.33 and $53,333,333.33 | $3,880,000.00 tax, $506,666.67 each surcharge, $4,893,333.34 in all |
| $4,200,000 in Victoria, and the same for a regional employer | Worked by hand from Sch 1 cl 1: $1,000,000 - $1,200,000 x 0.5 | $400,000.00 deduction, $184,300.00 and $46,075.00 |
| $1,356,789 in Queensland of $1,789,776 | QRO 'Payroll tax deductions': $932,460 to the dollar | $932,459.64 deduction, $20,155.64 |
| $1,104,000 in Western Australia | RevenueWA: $6,600 | $984,000.00 deduction, $6,600.00 |
| $500,000 in South Australia of $2,000,000 | RevenueSA: a deduction of $150,000 | $17,325.00 |
| $800,000 in South Australia of $1,600,000 | RevenueSA: $300,000; Sch 1 cl 5(2)(a): 2.475% | $12,375.00 |
| $2,400,000 in Tasmania, all in Tasmania | SRO Tasmania: $54,400 | $30,000.00 at 4%, $24,400.00 at 6.1% |
| $2,400,000 in Tasmania of $3,000,000 | SRO Tasmania example 2: $72,800 | $72,800.00 |
| $3,000,000 in the ACT | Worked by hand: ($3,000,000 - $1,750,000) x 6.75% | $84,375.00 |
| $3,000,000 in the Northern Territory | Worked by hand from Sch 1 cl 3: $2,500,000 - $500,000 / 2 | $2,250,000.00 deduction, $41,250.00 |

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

## PAYG gross-up

Schedule 1 states how much to withhold from a gross pay, not which gross leaves
a given net, so this worksheet has no rule of its own to cite: it takes every
amount from the PAYG withholding worksheet above, whose evidence applies. A
gross `g` leaves a net `n` when `g - withholding(g) = n`. Withholding is a whole
number of dollars, so `g` is `n` plus a whole-dollar `w` with
`withholding(n + w) = w`. As pay rises a cent, withholding never falls
(`tests/properties.rs` checks every bracket edge), so the net rises by at most a
cent and every net from $0 has a gross; but it can rise by a dollar, and then
2 grosses leave the same net. The search repeats `w = withholding(n + w)` from
0 up to the least such `w` and from a bound down to the greatest, then tests
each `w` between. The bound is `n + 52`: every Schedule 1 coefficient `a` is at
most 0.47 and every `b` at least -1 (the lowest is -0.6702, scale 1 for 2024-25
and 2025-26), and scale 4 withholds at most 47%, so a week's amount is at most
`0.47 x + 1.50` after rounding and withholding on a gross `g` at most
`g / 2 + 26`, the 26 covering a quarter's 13 weeks of that $1.50 and of 47% of
the 99 cents added to weekly earnings; `tests/payg_gross_up.rs` checks those
coefficients.

With `study_loan`, the search takes the study loan withholding worksheet's
amount: the Schedule 1 amount plus Schedule 8's component, each rounded on its
own, which Schedule 8's 'Software' paragraph accepts in place of its combined
coefficients. Every component `a` is at most 0.17 and every `b` at least 0, so
a week's total is at most `0.64 x + 2` and withholding on a gross `g` at most
`0.64 g + 35`; a fixed point `w` then has `w <= (0.64 n + 35) / 0.36`, below
`16 m / 9 + 99` for `m` the whole dollars of `n`, so as a whole number of
dollars `w` is at most `2 m + 98`, the bound. That worksheet's withholding never falls as weekly
earnings rise to $4,000, past every bracket edge (`tests/properties.rs`). The
ATO's Schedule 8 for payments from 1 July 2026 (published 17 June 2026) and its
Schedule 1 weekly coefficients were read on 16 November 2026. Schedule 8 gives
the component only, so `tests/payg_gross_up.rs` works each total by hand. Its
examples 1 and 2 (weekly $2,608.36 and fortnightly $4,409.75 on scale 2)
withhold $654 + $193 = $847 and $1,048 + $260 = $1,308 and gross up to their
earnings; example 3 (monthly $10,627.88 on scale 1) withholds $3,155 + $979 =
$4,134, and $10,622.88 leaves the same net, so it is refused as a tie naming
both. The combined coefficients give the same 3 totals. F2025L01016's example 1
for 2025-26 withholds $659 + $202 = $861 and ties with $2,609.36 and $2,610.36.

For each row of the ATO's sample data used above (48 earnings points for each
of 3 pay periods on scales 1, 2, 3, 5 and 6, for both schedules), the tests
take the row's earnings less its withholding as the net and require the
worksheet to return the row's earnings or to refuse a tie whose list names
them. `tests/properties.rs` grosses up random nets on every scale, pay period
and year, and requires each gross listed to leave the net and no other within
$60 of withholding either side of them to, and does the same with a study
loan on every scale and pay period of both Schedule 8 years.

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
whole, and the tests derive it. Exemptions follow under
[Exemptions](#exemptions), and the surcharge under
[Medicare levy surcharge](#medicare-levy-surcharge); trustees and part-year
residence are outside the worksheet.

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

### Exemptions

Read on 14 November 2026: ITAA 1936 ss 251T and 251U on the ATO's Legal
Database; Medicare Levy Act 1986 s 9 as compiled at 1 July 2026; and the ATO
pages "Medical exemption from Medicare levy" and "Not entitled to Medicare
benefits" (both last updated 22 June 2026) and "M1 Medicare levy reduction or
exemption 2026" (last updated 30 May 2026), part B.

A prescribed person for a period (s 251U(1)) is, among others, a person
entitled to free medical treatment for every condition through the Defence
Force or under the Veterans' Entitlements and related Acts, a blind pensioner,
a non-resident, a diplomat or consular official who is not an Australian
citizen and not ordinarily resident, or a person the Health Minister certifies
is not entitled to Medicare benefits. Under s 251U(2) nobody is a prescribed
person while a dependant is not. Under s 251U(3) a person who would be a
prescribed person in one of the medical categories but for a dependant is a
prescribed person for half of the period. Medicare levy (other than the
surcharge) is not payable by a person who was a prescribed person for the whole
year (s 251T), and for part of the year s 9(1) reduces the levy 'payable but for
this section' by the share of the year's days for which the person was a
prescribed person. The ATO's return asks for the full exemption days (label V)
and the half exemption days (label W), at most 365 together, a day in both
counted once as a full day; the worksheet takes the same 2 counts.

Sections 7(2) and 8(2) each work out the levy 'but for' s 9, so the exemption
reduces the levy after the low-income and family reductions:

```text
levy after s 9 = levy after s 8 x (2 x days in year - 2 x full days - half days)
                 / (2 x days in year)
```

When s 8(3) shares the family reduction, that levy is worked times family
income, as above, and divided once with the s 9 share. The ATO's Priya example
(a full year without Medicare entitlement and no dependants, 365 full days)
holds a full-year exemption to nil, and its Paul example (a Gold Card holder
whose spouse is below the low-income threshold) is a half exemption. No ATO
example gives an amount for a half or part-year exemption, or for one combined
with the low-income or family reduction; those cases in
`tests/medicare_exemptions.rs` are worked from the Act. The worksheet takes
the days as established facts and does not test entitlement: the categories,
the dependants and any family agreement are the operator's. Foreign residents,
whom s 251U(1)(d) also exempts, are outside a worksheet for full-year
residents, and a spouse must still not be entitled to any exemption, since
s 8(3) then turns on the spouse's own exemption. `tests/properties.rs` checks
that another exemption day never raises the levy and that a full day never
leaves more than a half one.

## Medicare levy surcharge

Read on 14 November 2026: the Medicare Levy Act 1986 as compiled at 28 March
2025 (compilation 52) and 1 July 2026 (compilation 53); the A New Tax System
(Medicare Levy Surcharge - Fringe Benefits) Act 1999 as compiled at the same
dates (compilations 41 and 42); the ATO pages "Medicare levy surcharge income,
thresholds and rates" (last updated 22 June 2026) and "M2 Medicare levy
surcharge 2026" (last updated 30 May 2026). The compiled text omits each Act's
part-year formula, which is an image; the ATO's M2 examples apply it as the
rate times the base times the days the section applies over the days in the
year, and the worksheet does the same.

The surcharge is 2 levies on the one rate. The Medicare Levy Act increases the
levy by 1% of taxable income (s 8B(2) for a person not married and without
dependants, s 8D(3) and (4) for a married person), read with s 3(2A) so that
it includes an amount on which family trust distribution tax was paid; the
Fringe Benefits Act imposes 1% of the reportable fringe benefits total
(ss 12, 15 and 16). Each rate rises by 0.25 of a percentage point for a tier 2
earner and 0.5 for a tier 3 earner (s 8B(3) and (4); Fringe Benefits Act
s 12(4) and (5)). Income for surcharge purposes, which also counts net
investment losses and reportable super contributions, decides only whether
the surcharge applies and the tier. The ATO's Josh example shows the split:
$127,000 makes him a tier 2 earner, and 1.25% applies to his $90,000 taxable
income and $20,000 of fringe benefits, $1,375. The thresholds page lets a
person reduce income for surcharge purposes by the taxed element of a super
lump sum within the low rate cap only for 2023-24 and earlier years, so no such
reduction applies to the years the worksheet supports.

| Threshold | 2024-25 | 2025-26 | 2026-27 |
| --- | --- | --- | --- |
| Singles tier 1 | $97,000 | $101,000 | $105,000 |
| Singles tier 2 | $113,000 | $118,000 | $123,000 |
| Singles tier 3 | $151,000 | $158,000 | $164,000 |
| Family tier 1 | $194,000 | $202,000 | $210,000 |
| Family tier 2 | $226,000 | $236,000 | $246,000 |
| Family tier 3 | $302,000 | $316,000 | $328,000 |
| Married person's own income floor | $27,222 | $28,011 | $28,011 |

The thresholds are the ATO's for each year; each family threshold is double
the singles one, as Private Health Insurance Act 2007 s 22-40 sets it, and
rises by $1,500 for each dependent child after the first. Income must exceed a
threshold to pass it. Through Medicare Levy Act s 3A and the Private Health
Insurance Act s 22-30, a person married on the last day of the year takes the
family thresholds against the couple's combined income. A married person is
liable only when that combined income exceeds the family tier 1 threshold and
their own exceeds the floor in s 8D(3)(c) (Fringe Benefits Act s 15(1)(c)),
the same figure as the s 3(1) threshold amount in each compilation; Act No. 58,
2026, Sch 5, which set $28,011, amends both Acts.

The ATO's examples hold the worksheet: Tom (2026-27, $117,000, tier 1,
$1,170), Josh (above), Jacinta (2025-26, 199 days without cover; the ATO gives
the days, and the test supplies her taxable income) and Jill and Kevin
(2025-26, married with 3 children, combined $220,000 against a family
threshold of $205,000, 193 days; the test supplies their split). A further
test gives Tom $10,000 of reportable super contributions: he becomes a tier 2
earner, and 1.25% applies to the same $117,000, $1,462.50. Applying the rate to
income for surcharge purposes instead would give $1,587.50.
`tests/medicare_surcharge.rs` also checks every tier boundary in each year,
for a single person and families with 0 to 4 children, the married person's
floor and every refusal, and `tests/properties.rs` checks that the surcharge
never falls as its base or the days without cover rise.

Scope limits, each documented in the worksheet's scope: the ATO's Michelle and
Alice examples, whose marital status changed during the year, apply the single
and family tests to different periods and are outside it, as is a person with
dependants but no spouse (s 8C), which no published example works. Days in an
exemption category, which the 2 Acts treat differently through ITAA 1936
s 251U(2) and (3), trustees (ss 8E to 8G), part-year residents, the tax offset
for the surcharge on lump sums in arrears and the private health insurance
rebate are also outside it.

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
and a 2026-27 result warns so.

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
$2,469. The example gives no rebate income for the spouse. It is at least the
$10,000 taxable income and, as the spouse's offset is the full $2,040, at most
the $33,732 threshold; it only tests entitlement, so the test checks both ends
and gets the same result. The same tests check every threshold and
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
spouse whose income is all ordinary taxable income below the tax-free
threshold has no tax payable. A superannuation or employment termination
remainder, which Income Tax Rates Act 1986 Sch 7 Part I clause 1(a) and (aa)
tax at 45%, would leave tax payable, so a spouse with one is outside the
worksheet. Above $6,000
s 12(3)(b)(ii)'s formula reaches nil before the spouse's tax, at the 15% or 16%
lowest rate, reaches the base amount, so the worksheet never needs the
spouse's tax. A foreign resident spouse, whose unused amount uses their
marginal rate, is outside it, as are trustees (s 160AAAB), a person also
entitled to the beneficiary tax offset (s 160AAA(4)), a person whose
spouse status changed in the year, the transfer of this person's own unused
amount and applying the offset against tax payable.

## Lump sum arrears tax offset

This worksheet's evidence is not a Library document. Read on 16 November 2026:
ITAA 1936 Part III Division 17 Subdivision AB, ss 159ZR to 159ZRD, as compiled
at 1 October 2026 (compilation 193), whose endnotes show ss 159ZRA to 159ZRD
unchanged since Act No. 153, 1988 and s 159ZR last amended by Act No. 112,
2015; the ATO page "Lump sum payment in arrears" (last updated 8 June 2026);
the ATO's "Tax rates – Australian resident" (last updated 13 August 2026);
and Income Tax (Transitional Provisions) Act 1997 s 4-11 (compilation 104,
1 October 2026) with the Income Tax Rates Amendment (Temporary Budget Repair
Levy) Act 2014, as made.

| Rule | Provision |
| --- | --- |
| Accrual years: each earlier year in which part of the eligible lump sums accrued; recent accrual years are the 2 latest when there are 3 or more, otherwise each one; the rest are distant | ITAA 1936 s 159ZR(1) |
| Normal taxable income: taxable income without the net capital gain | s 159ZR(1) |
| Threshold: the total arrears amount is at least 10% of normal taxable income less it, not below nil | s 159ZRA(1)(b) |
| Offset: tax on arrears less notional tax on arrears, if more than nil | s 159ZRB |
| Tax on arrears: rebated tax on the year's taxable income less rebated tax on it without the total arrears amount | s 159ZRB |
| Recent year: rebated tax on its taxable income with its arrears added less the rebated tax without them; a rebate year's own arrears deducted, an earlier rebate year's arrears that accrued in it added | s 159ZRC |
| Distant years: their arrears at the average of the recent years' rates, each the increase in gross tax on normal taxable income over the arrears, truncated to 3 decimal places and raised by 0.001 when the fourth would be more than 4 | s 159ZRD |

Rebated tax and gross tax are the tax at the year's resident rates for the
individuals the scope admits: the scope leaves out every amount taxed at
another rate and every offset other than this one. `PRIOR_RESIDENT_TAX_SCALES`
holds the ATO's rates for 2012-13 to 2023-24, read at 17:23 on 16 November
2026; `tests/lump_sum_arrears_offset.rs` checks each year's tax at the start
of every band against the page. The page leaves out the temporary budget
repair levy; s 4-11 makes it extra income tax of 2% of taxable income above
$180,000 for 2014-15 to 2016-17, so those years carry it as an overlapping
row. The 2011-12 flood levy and earlier years are not held, so a recent
accrual year before 2012-13 is refused; distant years need no rates and may
be any earlier year.

The worksheet computes in the order its `derive` steps show. A recent year's
rate is `floor(floor(increase / arrears x 10000) / 10 + 0.5) / 1000`: the
first `floor` keeps 4 decimal places, and adding 0.5 to the fourth before the
second `floor` raises the third by 0.001 exactly when the fourth is 5 or more.

`tests/lump_sum_arrears_offset.rs` holds the worksheet to the page's 4
examples. Sudeep ($3,500 against a $4,950 threshold) gets no offset. Anthony's
$43,979 for 2016-17, received with $111,231 of net capital gain, meets the
$15,895.60 threshold (the ATO rounds it to $15,896) and gets $19,790.55 less
$16,272.23, $3,518.32 (the ATO shows $3,518); his 2016-17 taxable income is
not given, and $100,000 keeps it in the 37% band. Heather meets the test, and
with assumed incomes in the 19% band pays $1,360 now against $1,615, so no
offset, as the page says is common. Yan's 6 accrual years, with made-up
incomes, give 2 recent years at 0.19 and 0.099 (0.0988 cut to 0.098 and
raised) and 4 distant years at 0.1445, an offset of $1,688.25, in any input
order. The other tests check the rounding at 0.0985 and 0.0984, a net capital
gain in a recent year, the rebate-year adjustments, the levy years, that the
offset never exceeds the tax on the arrears, and each refusal.

The Medicare levy exemption for lump sums in arrears (Medicare Levy Act 1986
s 9A), whose 10% test includes the arrears in normal taxable income and which
sets a condition on the levy in each of the 2 latest accrual years
(s 9A(1)(c)), and the Medicare levy
surcharge offset (ITAA 1997 Subdivision 61-L) are separate questions the
result names. The worksheet refuses a field it does not take, and leaves
out foreign and part-year residents, employment termination payments,
superannuation lump sums, arrears not classified as eligible, trustees
(s 159ZRA(2)) and applying the offset against tax payable.

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

## Franking account

This worksheet's evidence was read on 19 November 2026 from 9.45 pm to
10.10 pm AEDT:

- ITAA 1997 ss 202-60, 202-65, 202-75, 203-20 to 203-55, 204-70, 204-75,
  205-15, 205-25, 205-30, 205-40, 205-45, 205-50, 205-70, 214-15 to 214-25,
  214-150 and 214-155 on the ATO's legal database
  (https://www.ato.gov.au/law/view/document?docid=PAC/19970038/205-70 and the
  sections beside it). The disclosure of significant variations is
  Subdivision 204-E (ss 204-70 and 204-75), not Division 203.
- New Business Tax System (Franking Deficit Tax) Act 2002 s 5: the tax is the
  deficit at the year's end (s 5(a)); (Over-franking Tax) Act 2002 s 5: the tax
  is the amount s 203-50(2)(a) works out. Both on the Federal Register of
  Legislation (C2004A00982 and C2004A00981, latest versions).
- The ATO's Franking account tax return 2026 instructions (published
  30 May 2026). 'How to get': the return is needed only for franking deficit
  tax, over-franking tax or a significant variation. 'About', example 2: RI Pty
  Ltd's $45,000 R&D refund, then $30,000 of tax credited nil and $36,000
  credited $21,000. 'How to lodge and pay': the last day of the month after the
  year's end. Section A: codes S, D, F and P. Section B: label A includes the
  credit for the previous year's tax and not the opening balance; the step 3
  method for the attributable part; example 4, EKW Ltd, offset $62,000 on
  $80,000; example 5, Salomon Pty Ltd, $500 x (60% / 2.3333) = $128. The
  example's $214 is 99.87% of the $214.2857 maximum under s 203-35, so the test
  allocates $214.29, which s 202-65 caps at the maximum; the command shows
  $128.57 and does not round to the return's whole dollars, for which no rule
  was read. Section C: examples 7 to 9 (XYZ, Dombey, Isanini and Acolmal) and
  benchmarks to 2 places, 5 up.
- The s 205-70(2) note's example: $20,000 of a $60,000 offset used and $40,000
  carried forward.
- A private company case (opening $55,000, $181,714 of credits, FDT $123,286,
  offset $86,300.20) is worked by hand in the test; it follows the Library's
  Otage Pty Ltd example at ¶10-020, which could not be reread on
  19 November 2026.

The statute reduces 'so much of [the tax] as is attributable to' item 1, 2, 3,
5 and 6 debits (s 205-70(2) step 1); this worksheet takes that part by the
ATO's step 3 method, as section B does. The opening surplus counts in it, and a
deficit opening does not, its credit being in the year's credits.

## DASP tax

Added on 19 November 2026. Its sources were read that afternoon: the
Superannuation (Departing Australia Superannuation Payments Tax) Act 2007
(No. 13, 2007; compilation 5, 2 December 2016), ss 4 to 6, from the Federal
Register of Legislation; ITAA 1997 ss 301-170 and 301-175 from the ATO's legal
database; and the ATO's 'Payments from super' rates (last updated 16 September
2026) and 'Departing Australia superannuation payment (DASP)' (last updated 21
April 2026), which give the same rates, the 417 and 462 visas and that the
65% rate covers the whole payment. The Library's Superannuation Benefits,
8-420 (reviewed 30 June 2026), gives the worked example and the s 3A meaning
of working holiday maker.

s 5(3) applies to a payment made on or after 1 July 2017, so to every year
the command takes. The ATO's rates page gives s 6's temporary budget repair
levy years as 2014-15 to 2016-17, so none of them is taken. The Act states
percentages and no rounding; rounding each element's tax to the cent is the
command's.

`tests/dasp_tax.rs` holds each case.

| Example | Facts given | Tax |
| --- | --- | --- |
| Library 8-420 | Tax free component **$25,000**, element taxed in the fund **$17,000** | **$5,950.00** (35%) |
| The same, working holiday maker | As above | **$11,050.00** (65%) |
| Both elements | Tax free $1,000, taxed $10,000, untaxed $20,000 | **$12,500.00**; working holiday maker **$19,500.00** |

The last 2 rows are worked by hand from s 5. The tests also cover rounding
at the half cent, a payment under $200, a wholly tax free payment and each
refusal.
## Super benefit tax

Added on 22 November 2026. Its sources were read that day: ITAA 1997
ss 301-10 to 301-40, 301-90 to 301-115, 301-225, 302-60, 302-140, 302-145,
302-195, 303-10, 307-120, 307-125, 307-145, 307-150, 307-205, 307-275 to
307-290, 307-345, 307-350 and 960-285 from the ATO's legal database; the
ATO's 'Payments from super' (last updated 16 September 2026): Table 9 (low
rate cap), Table 10 (untaxed plan cap), Table 12 (preservation age) and
Table 13 (the lump sum tax table); 'Tax on super benefits' (last updated
2 August 2023) for the worked examples; 'Tax rates – Australian resident'
(last updated 13 August 2026) for the 45% top rate in each year; and
Schedule 12 of the Taxation Administration (Withholding Schedules)
Instrument 2026 (F2026L00716) and the ATO's Schedule 12 from 13 October 2020
to 30 June 2024.

| Year | Low rate cap | Untaxed plan cap |
| --- | --- | --- |
| 2020-21 | $215,000 | $1,565,000 |
| 2021-22 | $225,000 | $1,615,000 |
| 2022-23 | $230,000 | $1,650,000 |
| 2023-24 | $235,000 | $1,705,000 |
| 2024-25 | not used | $1,780,000 |
| 2025-26 | not used | $1,865,000 |
| 2026-27 | not used | $1,935,000 |

Schedule 12 (2026) says that from 2024-25 the preservation age is 60 for
everyone born after 30 June 1964, and everyone born before is 60 by 30 June
2024, so no member is at or above preservation age and under 60 after
2023-24 and the command holds no low rate cap for those years. Table 9 prints
'2025–26 $260,000' with no 2026-27 row whilst its text says the cap is
$260,000 from 1 July 2026; neither figure is used. Each cap left is the
year's cap less the amounts counted for earlier lump sums: each amount
counted never exceeds the cap then, so this equals the section's year by
year reduction and indexation (the ATO's Talise and Anh examples). The Act
states no rounding of the proportioning rule; rounding the tax free component
to the cent and each part's tax to the cent is the command's.

`tests/super_benefit_tax.rs` holds each case. The figures in brackets are the
ATO's rates with the 2% Medicare levy; the command gives them without it.

| Example | Facts given | Most tax |
| --- | --- | --- |
| s 307-125(2) | $100 from a $1,000 interest with $200 tax free | tax free **$20**, taxable **$80** |
| Tony, 58, 2021-22 | $100,000 tax free, $250,000 taxed | **$3,750.00** on $25,000 above the cap (17%) |
| Heather, 59, 2020-21 | $30,000 tax free, $220,000 taxed | **$750.00** on $5,000 (withheld $850 at 17%) |
| Dean, 61, 2020-21 | As Heather | **$0.00** |
| Mei, 60, 2021-22 | $45,000, $130,000 taxed, $205,000 untaxed | **$30,750.00** (17%) |
| Suzi, 50 | $1,000, $10,000 taxed | **$2,000.00** (22%) |
| Frankie, 45 | $5,000, $45,000 taxed, $20,000 untaxed | **$15,000.00** (22% and 32%) |
| Peta, 59, 2020-21, past both caps | $20,000, $40,000 taxed, $45,000 untaxed | **$26,250.00** (17% and the top rate) |
| Howard, 72, past the plan cap | As Peta | **$20,250.00** (the top rate) |
| Anh, 2020-21 | $90,000 untaxed after $165,000 | plan cap left **$1,400,000** |
| Chloe, 62, 2026-27 | $30,000, $190,000 taxed, $80,000 untaxed | **$12,000.00** (withheld $13,600 at 17%) |

The ATO examples give rates, not dollar amounts, except Heather's, Chloe's
and Anh's; the amounts are worked by hand from those rates. The tests also
cover the low rate cap going to the element taxed in the fund first
(s 301-105(6)), the untaxed plan cap's edge, each year's caps, birthdays
(29 February included) and the preservation age table's edges, death
benefits, a terminal medical condition, a lump sum under $200 and each
refusal.

## Super withholding

Sources, read on 22 November 2026: the ATO's Schedule 12 from 13 October 2020
to 30 June 2024, and Schedule 12 of the Taxation Administration (Withholding
Schedules) Instruments 2024 (F2024L00664), 2025 (F2025L00647), (No. 2) 2025
(F2025L01016, the same Schedule 12 text) and 2026 (F2026L00716). Each Table A
rate is the matching tax offset ceiling rate in `super-benefit-tax` plus the
2% Medicare levy; the 2024 to 2026 tables drop the preservation age to 59 rows,
as nobody is in that band from 1 July 2024. The schedules round 'withholding
amounts' to the nearest dollar with no preliminary rounding to the cent; the
command rounds the total once, and the schedules give no example where
rounding each part would differ. `tests/super_withholding.rs` holds each case.

| Example | Facts given | Withheld |
| --- | --- | --- |
| Tom, roll-over, 2020-21 | $1,585,000 untaxed; cap $1,565,000 | **$9,400** |
| Tom, roll-over, 2024-25 | $1,820,000 untaxed; cap $1,780,000 | **$18,800** |
| Tom, roll-over, 2025-26 | $1,900,000 untaxed; cap $1,865,000 | **$16,450** |
| Tom, roll-over, 2026-27 | $1,967,000 untaxed; cap $1,935,000 | **$15,040** |
| Chloe, 62, 2026-27 | $30,000, $190,000 taxed, $80,000 untaxed | **$13,600** |
| Heather, 59, 2020-21 | $30,000, $220,000 taxed | **$850** |
| Dean, 61, 2020-21 | As Heather | **$0** |

The tests also cover each Table A rate, the total's rounding, a foreign
resident, death benefits, nil payments, the no-TFN rates ignoring cents, each
year's untaxed plan cap and each refusal.

## Depreciation low-value pool

Added on 22 November 2026, with the 2 sections after it. The sources were read
that day: ITAA 1997 ss 40-425, 40-430, 40-435, 40-440 and 40-445 from the ATO's
legal database; the ATO's 'Low-value pool' (last updated 26 June 2025), whose
examples 1 to 3 are John's printer in 2020-21 and 2021-22; and the Guide to
depreciating assets 2026 (NAT 1996-06.2026), which repeats the example for
2025-26. The 18.75% and 37.5% rates do not change in 2020-21 to 2026-27. The
ATO shows whole dollars; the command shows cents, rounding the decline once.

| Example | Facts given | Result |
| --- | --- | --- |
| Guide 2026, John | Closing balance **$5,000**, printer **$594** (60% of $990) | Decline **$1,986.38** (ATO $1,986), closing **$3,607.62** (ATO $3,608) |
| Low-value pool page, 2021-22 | Closing balance **$3,608**, printer sold, **$300** (60% of $500) | Decline **$1,353.00**, closing **$1,955.00** |

`tests/depreciation_pools.rs` also covers low-value assets, second element
amounts, an excess termination value and the temporary full expensing warning.

## Depreciation small business pool

ITAA 1997 ss 328-185, 328-190, 328-195, 328-200, 328-210 and 328-215 and
IT(TP)A 1997 ss 328-180 and 328-181 from the ATO's legal database; Treasury
Laws Amendment (Tax Reform No. 2) Act 2026 (No. 71 of 2026) Sch 2 items 7 and
15; the ATO's 'Small business pool calculations' (last updated 16 June 2025)
and 'Instant asset write-off for eligible businesses' (last updated 28 August
2026); and LCR 2021/3. The low pool value threshold by year:

| Years | Threshold | Provision |
| --- | --- | --- |
| 2020-21 to 2022-23 | None: the whole balance above zero | IT(TP)A s 328-181(5) |
| 2023-24 to 2025-26 | $20,000 | IT(TP)A s 328-180(6)(e) |
| 2026-27 | $20,000 | s 328-210(1), Act No. 71 of 2026 Sch 2 item 7, item 15(3) |

| Example | Facts given | Result |
| --- | --- | --- |
| Pool calculations, example 2 (Loretta, 2018-19, run for 2025-26) | Opening **$100,000**, trailer **$28,000**, sale **$8,000** | **$30,000** + **$4,200**, closing **$85,800** |
| s 328-210(3), Cassidy's Gardens, 2024-25 | Opening **$28,500**, mower **$16,800**, car sold **$29,600** | **$15,700** written off, closing nil |
| LCR 2021/3, example 13, 2020-21 | Opening **$200,000** | **$200,000** written off |

The tests also cover each year's threshold either side of $20,000, an amount
below zero, a closing balance below zero and the 2020-21 refusal.

## Capital works

ITAA 1997 ss 43-140, 43-145, 43-210 and 43-215 from the ATO's legal database;
the ATO's 'Capital works deductions'; and the Rental properties guide 2026
(NAT 1729-06.2026), tables 1 and 2 and example 29, which also says the
denominator is 365 in a leap year while the days can be 366.

| Example | Facts given | Result |
| --- | --- | --- |
| Rental properties guide 2026, example 29 | **$115,800** at 2.5% for **346** days in 2025-26 | **$2,744.30** (ATO $2,744) |
| Leap year | $365,000 at 2.5% for 366 days in 2023-24 | **$9,150.00** |

The tests also cover the 4% rate, partial use, the undeducted limit, the
16 September 1987 and 27 February 1992 boundaries and days limited by a
start of construction in or after the year.

## ETP tax free component

This worksheet's evidence was read on 19 November 2026 from 9.25 am to
9.40 am AEDT:

- ITAA 1997 ss 82-10, 82-140, 82-145, 82-150, 82-155 and 995-1(1) ('last
  retirement day') on the ATO's legal database
  (https://www.ato.gov.au/law/view/document?docid=PAC/19970038/82-150 and the
  sections beside it). s 82-150(2): the invalidity segment is the ETP times the
  days to retirement over the employment days plus the days to retirement;
  s 82-155(2): the pre-July 83 segment is the ETP less the invalidity segment,
  times the days of employment before 1 July 1983 over all of them; s 82-140:
  the tax free component is the 2 segments; s 82-145: the taxable component is
  the rest; s 82-10(6)(c): a payment that includes an invalidity segment, or
  would but for s 82-150(1)(c), is an excluded payment. The same day's s 995-1
  already omits the 'laundry expense' definition that Act No. 49 of 2026
  repealed, so the database was current.
- The ATO's 'Tax-free component of ETPs' (last updated 24 July 2025): Jackson's
  $102,000 ETP, $102,000 x [370 / (3,305 + 370)] = $10,269 tax free and
  $91,731 taxable, in whole dollars. Its own dates, 2 September 2013 to
  25 September 2022 with a 65th birthday on 25 September 2023, count 3,311
  employment days and 365 days to retirement on the worksheet's rule, not the
  3,305 and 370 it shows. The test gives dates that make its counts, so the
  arithmetic is held to the page, and works its own dates separately: $10,127.86.
- The ATO's 'Employment termination payment with an invalidity segment
  worksheet' (NAT 71949, June 2014; page last updated 24 September 2014),
  read from its PDF: step 1 counts the days before 1 July 1983 including
  30 June, step 2 the days from 1 July 1983 to the termination day including
  it, and steps 6 to 9 apply the 2 formulas in that order.
- The Library's ¶8-850 (superannuation, 'Superannuation Benefits · Termination
  Payments', last reviewed 30 June 2026): Jim's $100,000 ETP on termination for
  invalidity on 31 July 2026, his 65th birthday on 15 July 2027, gives 349 days
  to retirement (1 August 2026 to 15 July 2027), the count this worksheet
  makes, and $2,128 on 16,050 employment days. From his start on
  20 August 1981 the days are 16,417, of which 680 fall before 1 July 1983,
  giving $2,081.59 and a pre-July 83 segment of $4,055.83, worked by hand in
  the test. Neither published example's employment days match its dates.
- The Library's ¶8-860 (the same document): Jon's $70,000 at step 1 after a
  $30,000 invalidity segment, employed from 1 January 1980 to
  31 December 2026, has 1,277 days before 1 July 1983, the count this worksheet
  makes, and $5,205 on 17,175 total days; the dates give 17,167, so the test
  holds the pre-July 83 segment at $5,207.08, read on 19 November 2026.

Rounding each segment half up to the cent is this command's own convention:
neither the Act nor NAT 71949 sets a rounding rule, and the ATO's examples show
whole dollars. Death benefit ETPs (which cannot
include an invalidity segment), payments that relate to part of the employment
or to broken periods of it, unused leave and superannuation disability benefits
under s 307-145 (read the same day; its days to retirement run from the day the
person stopped being capable of gainful employment, and its service days are
the benefit's service period) are outside it.

## Unused leave payment

This worksheet's evidence was read on 16 November 2026 from 17:57 to 18:01
AEDT:

- The ATO's Schedule 7, 'Tax table for unused leave payments on termination of
  employment' (published 17 June 2026, for payments from 1 July 2026), made
  under TAA 1953 Schedule 1 ss 15-25 and 15-30
  (https://www.ato.gov.au/tax-rates-and-codes/payg-withholding-schedule-7-tax-table-for-unused-leave-payments-on-termination-of-employment).
  Its table withholds 32% from long service leave from 16 August 1978 to
  17 August 1993, from annual leave and loading from before 18 August 1993 on
  a normal termination, and from all annual leave and loading and all post-17
  August 1993 long service leave on a genuine redundancy, invalidity or early
  retirement scheme, all at label A; 5% of pre-16 August 1978 long service
  leave at marginal rates, label B; and post-17 August 1993 leave on a normal
  termination at marginal rates, included in salary and wages. A post-17 August
  1993 payment from normal termination under $300 has the lesser of the table
  amount and 32% withheld. Amounts round to the nearest dollar, 50 cents up.
  Its marginal rate calculation has 7 steps, ignores cents after dividing by
  the 12, 26 or 52 pay periods, and withholds nothing for a study and training
  support loan. Its example (Norman: $780, $6,420 and $14,820 of long service
  leave on $1,155 a week, $6,786 withheld) adds the 5% to the post-17 August
  1993 component before one marginal rate calculation; it is the catalogue
  example and a test in `tests/unused_leave_payment.rs`, with the $188 and $279
  from the 2026-27 weekly tax table that `payg-withholding` gives. It sends
  working holiday makers to Schedule 15. Its method when a TFN is provided
  does not depend on residency, and its marginal rate calculation uses 'the
  relevant PAYG withholding tax table', so a foreign resident who gave a TFN
  uses Schedule 1 scale 3. When no TFN is provided it withholds 47% from the
  payment, or 45% for a foreign resident, with cents ignored; the ATO's
  'Payments when no tax file number (TFN) is quoted' (last updated 1 May 2020,
  read at 18:58 on 16 November 2026) says the same of 'any payment of unused
  leave'. The schedule ignores cents only in its rounding of withholding
  amounts, so the worksheet ignores the cents of the result, not of the
  payment.
- The ATO's 'Withholding from unused leave payments on termination of
  employment' pages (last updated 1 May 2020): Greg's invalidity example
  ($2,680 at 32%, $857 with cents disregarded, where Schedule 7 now rounds to
  $858), Helen's and Mark's annual leave examples 2 and 4, and Robyn's, Matt's
  and Sali's long service leave examples 7 to 9. Their withholding at marginal
  rates uses the 2014-15 tax tables, so the tests check only the parts that do
  not depend on those tables: Robyn's $2,560, Matt's $192.60, $10,050,
  $44,933.60 and $864, Sali's $800 and $15, Helen's split and Mark's $76.
  'Payments made after death' says nothing is withheld from unused leave paid
  after death and it is not shown on the employee's statement. Greg, Helen and
  Mark are on 'Unused annual leave', read at 17:59
  (https://www.ato.gov.au/forms-and-instructions/employment-termination-payments-withholding-from-unused-leave-payments/unused-annual-leave),
  and Robyn, Matt and Sali on the long service leave 'Examples', read at 18:01
  (https://www.ato.gov.au/forms-and-instructions/employment-termination-payments-withholding-from-unused-leave-payments/unused-long-service-leave/examples).
- ITAA 1997, compilation 268 (1 October 2026, the current one, read through
  the Federal Register API): s 83-10 makes unused annual leave, and a bonus or
  other additional payment for it, assessable; s 83-15 gives a tax offset that
  keeps its rate at most 30% when paid in connection with a genuine redundancy
  payment (s 83-175), an early retirement scheme payment (s 83-180) or an
  invalidity segment (s 82-150), or in respect of employment before 18 August
  1993; s 83-80 makes 5% of unused long service leave from the pre-16/8/78
  period and all of the rest assessable, the 95% being neither assessable nor
  exempt income; s 83-85 gives the 30% offset for the pre-18/8/93 period and
  for the post-17/8/93 period on those 3 reasons; and ss 83-90 to 83-115
  define the periods and attribute a payment to each by days, which the caller
  does.
- The ATO's STP Phase 2 employer reporting guidelines, 'When an employee
  transfers or leaves' (last updated 12 November 2025): lump sum A type R for
  all unused annual leave and loading and long service leave from 16 August
  1978 paid for genuine redundancy, invalidity or early retirement scheme
  reasons, type T for annual leave and loading from before 17 August 1993 and
  long service leave from 16 August 1978 to 17 August 1993 paid for other
  reasons, lump sum B for the whole of pre-16 August 1978 long service leave,
  and paid leave type U for post-17 August 1993 leave on a normal termination.
  The guidelines say 17 August 1993 for annual leave where s 83-15(b) says
  before 18 August 1993; the worksheet follows the Act.
- The ATO's 'What payments are qualifying earnings' (last updated 2 September
  2026), table 13: unused leave on termination, including annual leave, leave
  loading and long service leave, is neither ordinary time earnings nor
  qualifying earnings, whatever the reason for termination or its tax
  treatment. An award or agreement can still require super on it.
- The Superannuation Guarantee (Administration) Act 1992, compilation 78
  (1 July 2026, the current one, with the Payday Superannuation amendments),
  read at 18:57 on 16 November 2026: s 6(1) leaves out of ordinary time
  earnings a lump sum paid on termination that is an unused annual leave
  payment or unused long service leave payment within the meaning of ITAA
  1997, which covers leave loading (s 83-10(3)(b)) and every termination
  reason; s 10A(1)(a) brings ordinary time earnings into qualifying earnings,
  and its other paragraphs name commissions, directors' and parliamentarians'
  pay, labour under contracts and other payments that are not leave.

The worksheet computes withholding and the assessable amounts, not the tax
assessed: the offset depends on the employee's other income for the year.

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
every 2026 example. The 2024 and 2025 examples work the component as the
earlier percentage of weekly earnings, which F2025L01016 replaced from 24
September 2025, so the tests hold their PAYG components only. A 2025-26 study
loan pay by Method A or B(ii) takes F2025L01016's coefficients for its
`pay_date`, and the tests work Jarrod and a Method B(ii) pay by hand on them
(Jarrod: $289, $296, $7 x 52 = $364 on the bonus, $653 in all). Method B(i)
refuses a 2025-26 study loan. It recalculates each earlier period, and step 3
says only 'the relevant tax table'. F2025L01016 commenced on 24 September 2025
and repealed the earlier instrument without a transitional rule, yet its own
Schedule 5 examples use the study loan tables from 1 July 2024, so neither
instrument settles which coefficients recalculate a period before that date,
and the worksheet does not know the periods' dates. Method A, which Schedule 5
accepts for any additional payment, takes the 2025-26 study loan. A 2024-25
study loan, a 2025-26 one without its date or before 24 September 2025, and a
pay date without a study loan are also refused. Example 4 combines the 2
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
  the extra Schedule 13 calculation for a super income stream and lump sum
  leave loading (Schedule 5) are outside the worksheet.
- Study and training support loans. Read on 15 November 2026: both ATO
  Schedule 9 pages say that a payee with a HELP, VSL, FS, SSL or AASL debt may
  need additional amounts withheld and, 'To calculate additional withholding
  amounts', to 'refer to Study and training support loans weekly tax table'
  (Schedule 9 of F2026L00716 has the first sentence but not the referral).
  That table (NAT 2173, published 17 June 2026; for 24 September 2025 to
  30 June 2026, last updated 9 September 2025) says to work out the withholding
  amount, add the component from column 2 for a payee claiming the tax-free
  threshold, and use Schedule 8's formulas in software. A Schedule 9 payee has
  answered yes at question 5 of the Withholding declaration or its short
  version, which both ask whether they claim the tax-free threshold, so the
  worksheet uses Schedule 8's 'Tax-free threshold claimed or foreign resident'
  rates. Schedule 8 (F2026L00716 and the ATO page, both read the same day) takes
  x, the weekly earnings in whole dollars plus 99 cents, with the same
  fortnightly and monthly conversions as Schedule 9, and withholds no component
  for a Medicare levy variation declaration claiming a reduction or exemption
  'for having a spouse or dependants and low family income'; that declaration's
  exemption categories (questions 5 to 7) are separate from its family income
  reduction (questions 8 to 10), so a full or half exemption keeps the
  component, and the worksheet's `study_loan` description tells the caller to
  omit it for a reduction. 2025-26 uses F2025L01016's rates from 24 September
  2025, as `study-loan-withholding` does; the 2024-26 Schedule 9 page links to
  the 2024-25 weekly table, whose percentage method this crate does not hold,
  so 2024-25 and earlier 2025-26 pays are refused. All 2,616 rows of the table
  from 1 July 2026 ($985 to $3,600), column 2, equal Schedule 8's formula, and
  `tests/seniors_pensioners_study_loan.rs` checks them through this worksheet
  with each status, with the tables' $1,816.23 example ($72, and $79 from
  24 September 2025), Schedule 8's examples 1 and 2 and each rate's boundary.

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
scale 3 row; the worksheet follows the component method and does not use them.
Scale 4 and a Medicare levy variation for a spouse or dependants are
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

Both schedules state the quarterly rule in 'Calculating fortnightly, monthly
or quarterly withholding amounts': divide the quarter's earnings by 13, ignore
the cents and add 99 cents, then multiply the rounded weekly amount by 13.
Neither works a quarterly example, so `tests/study_loan_withholding.rs` holds
hand-worked figures, example 1's $2,608.36 a week taken 13 times ($33,908.68
gives $8,502 and $2,509 for 2026-27, $8,567 and $2,626 for 2025-26), and the
quarterly starting earnings each schedule states ($17,381 and $12,831 for
2026-27, $16,744 and $12,194 for 2025-26). `tests/properties.rs` requires a
quarterly pay to withhold 13 times the weekly amount.

## Study loan repayment

This worksheet's evidence is not a Library document. The thresholds in
`src/metadata.rs` were transcribed from the ATO page "Study and training
loan repayment thresholds and rates" (last updated 30 June 2026, read on 27
September 2026), tables 1 (2026-27) and 2 (2025-26), and the 2026-27 minimum
repayment income of $69,528 and replacement indexable amount of $129,717 were
checked against Gazette C2026G00249 of 20 April 2026, made under HESA 2003
ss 154-10 and 154-20. The page has since moved to
<https://www.ato.gov.au/tax-rates-and-codes/study-and-training-loan-minimum-repayment-income-and-rates>.
Read there on 15 November 2026, it keeps the title "Study and training loan
repayment thresholds and rates", last updated 30 June 2026, and its tables and
examples print the same figures. Its example 3, for 2026-27, compares
$254,780 with the 2025-26 threshold of $179,286 rather than 2026-27's
$186,051; the repayment is 10% either way. The frozen replay in `tests/parity.rs` holds the worksheet to the
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

The carry-forward by year was added on 16 November 2026. ITAA 1997 s 291-20 was
read that day in compilation 268 (1 October 2026, the current one), through the
Federal Register API at 17:22 AEDT and on the ATO's legal database at 17:29.
Subsection (6) makes a year's unused cap the shortfall of its concessional
contributions below its cap, and (7) gives none before 2018-19. Under (3) and
(4) a year's cap is increased, by no more than the excess over it, from the
previously unapplied unused cap of the previous 5 years when the total
superannuation balance just before the year is below $500,000, and (5) applies
those amounts from the earliest year to the most recent. An amount not applied
within those 5 years is no longer among any later year's previous 5, so it
lapses at the end of the 5th year after its own. The caps for 2018-19 to
2023-24 come from table 1.1 of the ATO's 'Contributions caps' page (last
updated 11 September 2026, read at 17:23 AEDT): $25,000 to 2020-21 and $27,500
from 2021-22. The legal database's editorial note to s 291-20 lists $32,000 for
2026-27; the ATO's table gives $32,500, which the worksheet uses.

`tests/contribution_caps.rs` holds these cases, and `tests/derivation.rs`
checks every step of the first and third:

| Case | Source | Worksheet | Published |
| --- | --- | --- | --- |
| Sam: $11,000 to $15,000 a year from 2019-20 to 2023-24, $97,500 in 2024-25 below $500,000 | ATO 'Concessional contributions cap' (last updated 2 July 2026, read 16 November 2026 at 17:24 AEDT), table 1 | unused $14,000, $13,000, $14,500, $13,500 and $12,500; cap available $97,500; all $67,500 applied; no excess | the same, cumulative $67,500; 'an individual concessional cap of $97,500'; no excess |
| Table 2: $3,000, $3,000, nil and nil from 2018-19 to 2021-22, then nil to 2024-25 | ATO 'Contributions caps', table 2 and its expiry example | unused $22,000, $22,000, $25,000 and $27,500; the 2018-19 amount lapsed; $22,000 of 2019-20 lapses at the end of 2024-25 | unused $22,000, $22,000, $25,000 and $27,500; 'a 2019-20 unused cap amount that isn't used by the end of 2024-25 ... will expire' |
| 2022-23 over its $27,500 cap by $22,500 below $500,000 | Hand-worked under s 291-20(4) and (5) | $20,000 of 2018-19 applied, then $2,500 of 2020-21; 2025-26 then applies 2020-21's last $7,500 before 2023-24's amount | (none published) |

Sam's table gives no balance, only that it was below $500,000; the test uses
$450,000. Table 2's later rows (the cap available each year) need each year's
balance and are years this worksheet does not run as the current year, so the
test checks its unused amounts and the 5-year expiry.

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

## Income tests

This worksheet's evidence is not a Library document. Read on 16 November 2026:
ITAA 1997 s 995-1 and ss 4-15, 291-15, 293-20, 301-20 and 313-20 (compilation
268, 1 October 2026); ITAA 1936 s 6(1) and s 251S (compilation 193, 1 October
2026); HESA ss 154-5, 154-16 and 154-17 (compilation 105, 17 October 2026);
the A New Tax System (Family Assistance) Act 1999 Sch 3 cls 2, 4, 5, 7 and 8
(compilation 124, 1 July 2026); FBTAA ss 135N to 135Q; SISR reg 6.01(2)
(compilation 159); the Superannuation (Government Co-contribution for Low
Income Earners) Act 2003 s 8 (compilation 28, 1 July 2018); the ATO's
'Income tests', its rebate income, adjusted taxable income, IT1 and A3
instructions for 2025 and 2026, 'Medicare levy surcharge income, thresholds and
rates' (last updated 22 June 2026), 'When you must repay your loan' (13
October 2026), its Division 293 page (24 August 2026) and its FBT rates page
(20 May 2026). The ITAA 1997, ITAA 1936 and Family Assistance Act compilations
of 1 July 2024 (250, 184 and 119) define each measure as the current ones do,
so the definitions cover 2024-25 to 2026-27; the 2026 compilation adds one tax
free benefit to cl 7's list, which the operator classifies.

| Measure | Sum | Provision |
| --- | --- | --- |
| Income for surcharge purposes | Taxable income without the FHSS amount, family trust distribution amounts, reportable fringe benefits total, reportable super contributions, total net investment loss | ITAA 1997 s 995-1 (a) to (d) |
| Division 293 income | Income for surcharge purposes less reportable super contributions | ITAA 1997 s 293-20(1)(a) |
| Rebate income | Taxable income without the FHSS amount, reportable super contributions, total net investment loss, adjusted fringe benefits total | ITAA 1936 s 6(1) |
| Repayment income | Taxable income without the FHSS amount, total net investment loss, reportable fringe benefits total, exempt foreign employment income, reportable super contributions | HESA s 154-5(1) |
| Adjusted taxable income | Taxable income without the FHSS amount, adjusted fringe benefits total, target foreign income, total net investment loss, tax free pensions, reportable super contributions, less child support paid | Family Assistance Act Sch 3 cl 2(1), disregarding cls 3 and 3A |
| Co-contribution total income | Assessable income without the FHSS amount, reportable fringe benefits total, reportable employer contributions less excess concessional contributions (not below nil), less business deductions | Co-contribution Act s 8(1), (1A) and (2) |

Reportable super contributions are reportable employer contributions and
Subdivision 290-C deductions, reduced, but not below nil, by excess
concessional contributions (s 995-1). The adjusted fringe benefits total takes
a section 57A employer's amounts before FBTAA s 135Q(4)'s gross-up at
1 / (1 - 47%), so at 53%; the ATO's 2025 and 2026 worksheets round that down
to the dollar, and the worksheet does too. The s 301-20(3) amount income for
surcharge purposes takes off is nil from 2024-25: it applies only below
preservation age, which is 60 for anyone born after 30 June 1964 (SISR reg
6.01(2)), and the ATO's surcharge page limits it to 2023-24 and earlier.
Adjusted taxable income can be below nil, as cl 2(1) sets no floor.

Section 995-1 does not add exempt foreign employment income to income for
surcharge purposes, and nor does s 293-20 to Division 293 income. The ATO's
surcharge page adds it where taxable income is $1 or more. The Medicare Levy
Act 1986 (compilation 53, 1 July 2026) and the A New Tax System (Medicare Levy
Surcharge—Fringe Benefits) Act 1999 (compilation 42, 1 July 2026), read on 16
November 2026, do not add it: s 3(1) of each gives 'income for surcharge
purposes' the ITAA 1997 meaning, and neither mentions ITAA 1936 s 23AF or
23AG. The only provision read that counts it beside the surcharge amounts is
the one-eleventh test for the surcharge lump sum offset (ITAA 1997
s 61-580(1)(d)), which does not define the surcharge's income. The worksheet
follows s 995-1 and warns when the two differ.
The Commonwealth Seniors Health Card income test and family assistance
adjusted taxable income are Services Australia's
(https://www.servicesaustralia.gov.au/income-test-for-commonwealth-seniors-health-card),
and the worksheet does not work them out.

`tests/income_tests.rs` holds the worksheet to the ATO's Christina example
(repayment income of $73,810 from taxable income of $50,420, a $1,250 total
net investment loss, $4,560 of reportable fringe benefits, $2,580 of exempt
foreign employment income and $15,000 of reportable super contributions) in
all 3 years, and its Tom example (income for MLS purposes of $117,000 from
$90,000 taxable income and $27,000 of fringe benefits, then $1,170 of
surcharge from `medicare-levy-surcharge`). Jan's Division 293 example
($240,000 and $15,000 give $750) runs from fabricated components through
`division-293`. No ATO example breaks down rebate income, adjusted taxable
income or co-contribution total income, so their cases are worked by hand
from the provisions: the 53% rounding, excess concessional contributions,
the FHSS amount, family trust distribution amounts, child support and
business deductions.

## Small business CGT concessions

This worksheet was added on 19 November 2026. ITAA 1997 Division 152 was read
that day in compilation 264 (21 May 2026), volume 4, at 14:40 AEDT
(https://www.legislation.gov.au/C2004A05138/2026-05-21/2026-05-21/text/original/epub/OEBPS/document_4/document_4.html),
s 104-190(1A) in volume 3 at 14:39 and s 328-110 in volume 7 at 15:20
(document_7). That compilation is the text for 2025-26 and 2026-27: Act
No. 49 of 2026, read at 14:40, amends Division 152, and Schedule 1 item 82
applies those changes to assessments for the income year that includes
1 July 2027 and later years.

Choices the Act leaves to the reader:

- s 152-10(1AA)(b) puts $2 million in place of each $10 million in s 328-110.
  An entity under $2 million on any rule of s 328-110 is also under
  $10 million on it, and one not barred by s 328-110(3) at $2 million is not
  barred at $10 million, so s 152-10(1AA)(a) adds nothing to the test.
- s 152-10(1A)(a) and (1B)(b) test the affiliate, connected entity or
  partnership that carries on the business, so for a passively held asset
  the turnover is that entity's, and s 152-10(1A)(b) and (1B)(c) add that
  you carry on no business of your own outside partnership.
- s 152-35 counts a total of days. The test period counts both its first and
  last days; half is met when twice the active days reaches the period's
  days. 7.5 years is 2,737 days (7 years with one leap day and the shortest
  6 months, 181 days) to 2,741 days (2 leap days and 184 days), so 2,741 or
  more is met, under 2,737 is not, and a total between them is refused.
- 15 years from 29 February, a 55th birthday on 29 February and one year
  before or 2 years after a 29 February event fall on 1 March.

`tests/sb_cgt_concessions.rs` holds these cases.

| Case | Source | Worksheet |
| --- | --- | --- |
| Neil Swan, 54, sells the business he bought in 2007 on turnover of $1.1 million and $1.2 million | Library ¶2-280 (the days, his birthday, the turnover 2 years back and the net value fabricated) | Basic conditions met; the 15-year exemption closed by his age; the reduction, the retirement exemption with a super contribution, and the roll-over open; as the Library concludes |
| The same at 55 and retiring, or permanently incapacitated | ¶2-280 ('If Neil was to hold off selling') | The 15-year exemption open and the others closed by it (ss 152-215, 152-330 and 152-430) |
| Wendy Simpson's land, owned 20 years and used throughout by her and her husband's company | Library ¶2-303 (the dates, turnover and net value fabricated) | Basic conditions met through s 152-10(1A); the 15-year exemption closed as she is not retiring |
| The same company on $2 million or more, or Wendy with her own business, net value over $6 million | Worked by hand under s 152-10(1A)(a) and (b) | Basic conditions not met |
| Turnover of $1,999,999.99 and $2,000,000.00 on each rule; 2 years at $2 million; winding up | Worked by hand under s 328-110(1), (3), (4) and (5) | Met under the rule each figure passes; the likely turnover barred after 2 years at $2 million |
| 1,000 days owned, 500 and 499 active; a business that ceased 11 months or 12 months and a day before the event | Worked by hand under s 152-35 | Met at 500, not at 499; the period ends at the cessation within 12 months, or with a longer period allowed |
| Owned more than 15 years, 2,736, 2,737, 2,740 and 2,741 days active | Worked by hand under s 152-35(1)(b) | Not met, refused, refused, met |
| A company with and without a significant individual | Worked by hand under ss 152-110 and 152-305(2) | Without one: the 15-year and retirement exemptions closed, the reduction and roll-over open |
| $500,000 and $125,000.50 of the retirement limit used | Worked by hand under s 152-320 | The exemption closed; $374,999.50 left |
| Neil's roll-over also obtained for a CGT event on 30 June 2026 | Worked by hand under s 104-190(1A)(a) and (b)(i) | Replacement asset period 30 June 2025 to 30 June 2028, from the last event, not 15 May |
| Neil's sale with a look-through earnout whose last payment could fall due on 31 January 2028, 15 November 2027 or 31 August 2028 | Worked by hand under s 104-190(1A)(b)(ii) | The period ends 31 July 2028, 6 months after it; 15 May 2028, 2 years after the event, the later; or 28 February 2029 |

## Personal tax estimate

`personal-tax-estimate` adds no rate or threshold: each amount is a
worksheet's own, and the tests check the composition. Read on 15 November
2026: ITAA 1997 s 63-10 on the ATO legal database, whose table applies the
seniors and pensioners tax offset (item 5) before any offset not covered by
another item (item 20, the low income tax offset) against basic income tax
liability; the ATO's "Low income tax offset" (last updated 8 June 2026), which
says the offset "can only reduce your tax payable to $0"; and its "Division 293
tax on concessional contributions by high-income earners" (last updated
24 August 2026), under which the ATO sends a separate Division 293 notice.
These were read again on 16 November 2026 and are unchanged, apart from s 63-10's
new item 3 for the working Australians tax offset, which applies from 2027-28.

Taxable income from labour income, read on 16 November 2026 on the ATO legal
database: ITAA 1997 s 4-15(1), taxable income is assessable income less
deductions, and where the deductions equal or exceed assessable income there is
no taxable income (step 3); s 25-130, the standard deduction for work-related
expenses, inserted by Schedule 4 to the Treasury Laws Amendment (Tax Reform
No. 1) Act 2026 (No. 49 of 2026, assented to 26 June 2026, Schedule 4 commenced
1 July 2026), whose item 17(1) applies it to assessments for the 2026-27 income
year and later years, read on the Federal Register of Legislation as made. It
gives a resident with assessable labour income the lesser of $1,000 and that
income, reduced, but not below zero, by the deductions in s 25-130(2)(c) to (g),
so the work-related deductions and the standard deduction together are the
greater of the work-related deductions and that lesser amount. TAA 1953 Sch 1
s 18-15(1) gives a credit equal to the amounts withheld in the year once an
assessment is made. The tests work their figures by hand: $85,000 salary,
$300 of work expenses, $500 interest and $200 of union fees in 2026-27 give a
$700 standard deduction, $84,300 taxable income, $15,810 tax, a $1,686 levy and
$17,496, so $1,496 is payable after $16,000 withheld; $60,000 salary and $300
of work expenses in 2025-26 give $59,700, $8,698 tax less a $104.50 offset plus
a $1,194 levy, $9,787.50, and a $212.50 refund after $10,000 withheld.

Tax withheld from regular pays, read on 16 November 2026: TAA 1953 Sch 1
s 12-35 on the ATO legal database (an entity must withhold from salary, wages,
commission, bonuses or allowances it pays an employee) and s 18-15(1) (the
credit); the ATO's 'Tax tables' (last updated 17 June 2026), whose schedules
from 1 July 2026 are the latest published; Schedule 1 (published 17 June
2026), whose 'Using a formula' provides for 53 weekly and 27 fortnightly pays in a
financial year, which bound `pays_in_year`; Schedule 5's 'Working out the withholding
amount' (Method A spreads the additional payment over 52, 26 or 12 pay
periods, or the periods a bonus for less than 12 months relates to, and its
withholding is limited to 47% of the payment); and Schedule 9, which is for
payees who declare that they are Australian residents for tax purposes. The
estimate adds no rate: the tax withheld is the pay's own command's
`withholding` times the pays, plus Method A's `withholding_on_additional_payment`.
The tests work their figures by hand from the 2026-27 Schedule 1 scale 2
coefficients (a = 0.3200 and b = 181.7319 below $2,596 a week): $3,269.23 a
fortnight is $1,634.99 a week, $341 withheld a week and $682 a fortnight, and
26 pays withhold $17,732 against the $17,496 above, a $236 refund. A $5,000
bonus by Method A adds $192 a pay, so $3,461 a fortnight, $1,730.99 a week,
$372 a week and $744 a fortnight; ($744 - $682) x 26 is $1,612, under the
$2,350 limit, and the year withholds $19,344 against $19,096 on $90,000 of
salary, a $248 refund. On scale 3 a foreign resident's $1,634.99 a week
withholds $490, $25,480 over 26 fortnights, against 30% of $85,000, $25,500.

The foreign income tax offset rules guides for 2025 (2024-25, last updated 29
May 2025) and 2026 (2025-26, last updated 30 May 2026) give, at step 1 of
examples 16 and 17, the tax on a taxable income including the Medicare levy
and disregarding offsets: $16,388 on $80,000, $9,988 on $60,000 and $260.80 on
$19,830 in both years. The estimate's basic tax and levy give the same, and its
total then takes off the low income tax offset: $100 at $60,000
(`325 - 0.015 × 15000`) and all $260.80 at $19,830, where the $700 offset
leaves nil. Anna's $31,130 is the exception. The 2026 guide gives $2,459.60:
$2,068.80 tax and a levy of 10% of the excess over $27,222, the 2024-25
threshold. For 2025-26 the threshold is $28,011, so the estimate gives a
$311.90 levy and $2,380.70. The 2025 guide's $2,581.80 for 2024-25 likewise
applies 2023-24's $26,000.

Tom ("Medicare levy surcharge income, thresholds and rates", 2026-27) pays a
$1,170 surcharge on $90,000 taxable income and $27,000 reportable fringe
benefits; the estimate adds it to $17,520 tax and a $1,800 levy, $20,490 in
all. José ("Seniors and pensioners tax offset", last updated 2 October 2026,
2025-26) has a $1,720 offset on $39,000 rebate income. Taking that as his
taxable income too, a fabricated assumption, the offset comes off $3,328 tax
first, the low income tax offset of `700 - 0.05 × 1500 = 625` next, and his
levy is nil at the SAPTO thresholds, where it would be $780 without them.

For a full-year foreign resident the estimate runs `foreign-resident-tax`.
Read on 15 November 2026: ITAA 1997 s 61-110(1)(a) gives the low income tax
offset only to an individual resident at any time in the year; ITAA 1936
s 251S(1)(a) levies the Medicare levy only on the taxable income of a person
resident at any time in the year, and the surcharge is an increase in that
levy (Medicare Levy Act 1986 ss 8B to 8D); and the ATO's "2026 Non-resident
foreign income" instructions say a foreign resident's study loan repayment may
be a compulsory repayment, an overseas levy or both, depending on worldwide
income. `study-loan-repayment` excludes the overseas levy, so the estimate
refuses a study loan for a foreign resident. ITAA 1936 s 160AAAA sets no
residence condition and the ATO's SAPTO page works out a foreign resident
spouse's unused offset, but a foreign resident with a pension assessable here
is not a prescribed non-resident (Income Tax Rates Act 1986 s 3(1)) and pays
resident rates, which
`foreign-resident-tax` excludes; the estimate composes SAPTO for a resident
only. ITAA 1997 ss 293-15 and 293-20 set no residence condition, so Division
293 tax is composed as for a resident. On the ATO's "Tax rates - foreign
resident" (last updated 1 June 2026), $60,000 in 2025-26 gives $18,000, the
$19,830 of the guide's example 16 gives $5,949 in 2024-25 where a resident pays
$260.80 and then nothing after the low income tax offset, and $250,000 gives
$87,850, with Division 293 tax of 15% of $30,000, $4,500, outside the total.

## FBT return

`fbt-return` adds one threshold, the $2,000 of FBTAA s 135P(1); every other
amount is a worksheet's or `fbt`'s, and the tests check the composition. Read on
16 November 2026 at 18:09 to 18:15 AEDT:

- FBTAA, compilation 97 (C2026C00332, in force from 1 July 2026, the latest on
  the Federal Register of Legislation), volumes 1 and 2: s 5B(1A) to (1C) (the
  fringe benefits taxable amount as the 2 grossed-up amounts added, the type 2
  amount times 1 / (1 - FBT rate)); s 5C(3) and (4) (type 1 and
  type 2 aggregate amounts, excluded benefits included at step 3); s 5E(2) and
  (3) (the individual fringe benefits amount and the excluded fringe benefits:
  meal entertainment not salary packaged, car parking fringe benefits,
  entertainment facility leasing not salary packaged, remote area fuel and
  housing, amortised and reducible benefits, prescribed benefits, remote area
  travel and freight, and security); s 5F (an employee's share of a shared or
  class-valued benefit); s 135M (outline); s 135N (the total across employers); s 135P(1)
  (more than $2,000, with examples 1 and 2: a $2,545 debt waiver is reportable,
  $1,900 is not), (2) (the individual amount times 1 / (1 - rate of tax), read
  from the formula's text description) and (3) (s 8A disregarded); s 135Q
  (employers under s 57A or s 58, not composed); s 10(5) (an operating cost value
  above the s 9 value is replaced by it).
- ATO, FBT return 2026 calculation details for taxable employers (published
  1 April 2026), items 14A to 25: examples 2 and 4 ($11,000 x 2.0802 =
  $22,882.20; $9,000 x 1.8868 = $16,981.20; subtotal $18,735.61); items 14C,
  17 and 18 left blank by taxable employers.
- ATO, Item 23 fringe benefit categories for FBT return 2026: labels A to G
  and J to P (no H), what each Number column counts, and examples 6 to 12.
- ATO, Completing your FBT return 2026 - all employers: items 1 to 13,
  including item 10's count of employees and associates.
- ATO, Reportable fringe benefits (last updated 12 January 2023): Derek's
  example, $3,500 x 1.8868 = $6,603, car parking $450 left out.
- ATO, Fringe benefits tax - a guide for employers, chapter 5: 5.2 excluded
  benefits (pooled or shared cars among the prescribed ones), 5.3 benefits to
  an associate allocated to the employee, 5.4 the formula
  and the example $2,890 x 1.8868 = $5,452 'in whole dollars'.
- ATO, FBT rates and thresholds (last updated 20 May 2026): 47% and 1.8868 for
  the years ending 31 March 2023 to 2027.
- ATO, FBT return 2027 instructions: not found (HTTP 404), so 2027 follows the
  2026 labels with a warning.

The reportable amount follows s 135P(2)'s formula, the individual amount over
1 - 0.47, not the ATO's 1.8868 (1 / 0.53 is 1.88679...). No provision states
how to round it; the ATO's examples drop the cents, so the return does too.
Rechecked on 16 November 2026 at 19:22 to 19:30 AEDT, from compilation 97's
EPUB on the Federal Register's API (the formulas' text descriptions in
s 5B(1C) and s 135P(2)) and the guide's chapter 5: 5.4 writes $3,500 /
(1 - 0.47) as '$6,603 (in whole dollars)', where the formula gives $6,603.77,
and 5.8 gives a $15,000 house as $15,000 x 1.8868 = $28,302, where the formula
gives $28,301.89 and so $28,301. The guide's other examples give the same
dollars both ways. Item 23's instructions say which columns to leave blank for
each label; the result leaves those out. The Number at 23A, 23B, 23C and 23F
counts cars, loans and units of accommodation, which the benefits do not
identify, so it is left to the preparer; 23G counts employee labels.

The examples give taxable values only. The tests produce them with worksheet
inputs: examples 2 and 4's $10,000 car as a $50,000 car for the full year, its
$1,000 of meal entertainment as half of $2,000 under the 50/50 split, and the
$6,000 school fees and $3,000 rent as external expense payments; Derek's car
parking as 25 days at an $18 lowest fee, and the guide's holiday accommodation
as a $600 residual benefit. Example 12 is the 40 weeks the s 31 reduction
covers ($23,640, of which $14,000 and $7,960 are exempt) and 12 weeks at $591
($7,092) without it: item 23G is $30,732, $21,960 and $8,772 for 1 employee.
Example 6's $4,804.38 is $4,804 in whole dollars on the ATO's page.

## Income year 2027-28

Read on 16 November 2026 to decide which worksheets take 2027-28. A worksheet
takes it only when every rate and threshold it uses is already set for that
year; the others refuse it and say what is missing.

| Parameter | 2027-28 | Source, time read (AEDT) |
| --- | --- | --- |
| Resident rates | Enacted: 14% to $45,000, 30% to $135,000, 37% to $190,000, 45% above, from $18,200 | Income Tax Rates Act 1986 Sch 7 Part I clause 1(b), compilation 66 (https://www.legislation.gov.au/C2004A03348/2026-07-01/2026-07-01/text/original/epub/OEBPS/document_1/document_1.html), 16:07; the ATO's 'Tax rates - Australian residents' gave years only to 2026-27 |
| Foreign resident rates | Enacted: 30%, 37% and 45%, tied to the second and third resident rates | The same Act, Part II, 16:07; the ATO's 'Tax rates - foreign residents' gave none for 2027-28 |
| Low income tax offset | Enacted, no year: $700, less 5 cents a dollar from $37,500 and 1.5 cents from $45,000 | ITAA 1997 s 61-115 (https://www.ato.gov.au/law/view/document?docid=PAC/19970038/61-115), 16:07 |
| Medicare levy thresholds | Enacted: the 2025-26 figures, which Act No. 58, 2026 Sch 5 item 14 applies to later years | Medicare Levy Act 1986 ss 3(1) and 8, compilation 53 (https://www.legislation.gov.au/C2004A03351/2026-07-01/2026-07-01/text/original/epub/OEBPS/document_1/document_1.html), 16:08; the ATO's low-income page gave 2025-26 only |
| SAPTO thresholds | Set by formula: s 9 and 10 at the 14% lowest rate give $37,137 and $54,977 single, $35,950 and $52,270 illness-separated, $32,822 and $45,638 a couple | Income Tax Assessment (1936 Act) Regulations 2025 (https://www.legislation.gov.au/F2025L01060/asmade/text), 16:09; the ATO's SAPTO page (last updated 2 October 2026) gave 2025-26 only |
| Division 293 | Enacted, no year: $250,000 and 15% | ITAA 1997 s 293-20 (https://www.ato.gov.au/law/view/document?docid=PAC/19970038/293-20), 16:07 |
| GST | Enacted, no year: 10% | GST Act s 9-70 (https://www.ato.gov.au/law/view/document?docid=PAC/19990055/9-70), 16:13 |
| Pension minimum | Standing: SISR Schedule 7 age factors, pro-rated over 366 days in a leap year, to the nearest $10 | ATO 'Income stream (pension) rules and payments', 16:13 |
| Working Australians tax offset | Enacted from 2027-28: the lesser of $250 and the basic income tax on net labour income, for a resident whose net labour income exceeds the tax-free threshold; applied first (s 63-10(1) item 3) | ITAA 1997 ss 61-150 to 61-160 and 63-10 (https://www.ato.gov.au/law/view/document?docid=PAC/19970038/61-160), 16:55; Act No. 49 of 2026 Sch 3, item 4 applying it to assessments for 2027-28 and later years (https://www.legislation.gov.au/C2026A00049/asmade/text), 16:59; the estimate applies it |
| Minimum tax on capital gains | Enacted from 2027-28 for a resident | ITAA 1997 s 119-10 (https://www.ato.gov.au/law/view/document?docid=PAC/19970038/119-10), 16:13; not worked out, and `resident-tax` warns |
| Withholding schedules | Not published: the latest are for payments from 1 July 2026 | ATO 'Tax tables' (https://www.ato.gov.au/tax-rates-and-codes/tax-tables-overview, last updated 17 June 2026), 15:58 |
| Medicare levy surcharge thresholds | Not published; with no day without cover the surcharge is nil, as Medicare Levy Act 1986 ss 8B(1)(c), 8C(1)(c) and 8D(1)(b) and the Fringe Benefits Act ss 12(1)(b)(iii), 13(1)(b)(iii) and 14(1)(b) apply only to a period without private patient hospital cover | ATO thresholds page (last updated 22 June 2026, years to 2026-27), 16:07; both Acts as compiled at 1 July 2026, 17:05 |
| Study loan repayment thresholds | Not published | ATO minimum repayment income page (last updated 30 June 2026, years to 2026-27), 16:07 |
| Contribution caps, ETP caps | Not published | ATO contributions caps page (last updated 11 September 2026) and ETP page (17 April 2026), 16:07 |
| Division 296 thresholds | Not published: indexed from 2027-28 | ATO Division 296 page (last updated 3 November 2026, 2026-27 only), 16:13 |
| Borrowing expenses | Worked out and flagged: s 25-25 sets no figure by year, and s 26-155 applies after it | ITAA 1997 ss 25-25 and 26-155 on the ATO's legal database, read 19 November 2026 |
| CGT and rental deductions | Changed from 1 July 2027 by Act No. 49 of 2026 Schedules 1 and 2 (s 26-155) | ITAA 1997 s 26-155 (https://www.ato.gov.au/law/view/document?docid=PAC/19970038/26-155), 16:13 |
| Balancing adjustment | Refused: CGT event K7 is a capital gain, which Schedule 1 to Act No. 49 of 2026 changes from 1 July 2027 | Act No. 49 of 2026 as made (https://www.legislation.gov.au/C2026A00049/asmade/text), read 18 November 2026 at 07:06; see [Balancing adjustment](#balancing-adjustment) |
| CGT parcels | Refused: Schedule 1 to Act No. 49 of 2026 changes how capital gains are worked out from the income year that includes 1 July 2027 | Act No. 49 of 2026 as made (https://www.legislation.gov.au/C2026A00049/asmade/text), items 82 to 84, read 19 November 2026 at 14:40; see [CGT parcels](#cgt-parcels) |
| Trade-in | Refused: the car limit for 2027-28 was not published, and CGT event K7 changes with Act No. 49 of 2026 | ATO 'Purchasing a motor vehicle' (last updated 3 June 2026, the 2026-27 limit only), read 18 November 2026 at 07:10; see [Trade-in](#trade-in) |
| Small business CGT concessions | Refused: Schedule 1 to Act No. 49 of 2026 changes Division 152 from the income year that includes 1 July 2027 | Act No. 49 of 2026 as made (https://www.legislation.gov.au/C2026A00049/asmade/text), item 82, read 19 November 2026 at 14:40; see [Small business CGT concessions](#small-business-cgt-concessions) |
| Business capital expenditure | Enacted, no year: ITAA 1997 s 40-880 and the small business entity test of Subdivision 328-C, neither amended in compilation 268 or the version from 1 December 2026; s 26-155 left out of scope | Compilation 268 (https://www.legislation.gov.au/C2004A05138/latest/text) and the Register's API, read 17 November 2026 from 01:33; see [Business capital expenditure](#business-capital-expenditure) |

Worked by hand: resident tax on $45,000 is 14% of $26,800, $3,752; on
$100,000, $3,752 + 30% of $55,000 = $20,252; on $250,000, $51,102 + 45% of
$60,000 = $78,102. A foreign resident's $200,000 gives $40,500 + 37% of $55,000
+ 45% of $10,000 = $65,350. The Medicare levy on $30,000 is 10% of $1,989,
$198.90. A single person's SAPTO threshold, from s 9(6) as (0.14 x 18,200 +
445 + 2,230 + 37,000 x (0.015 + 0.30 - 0.14)) / 0.315, is $37,136.51, rounded
up to $37,137. A pension of $250,000 at age 67 for 181 of 2027-28's 366 days
has a minimum of 5% x 250,000 x 181 / 366 = $6,181.42, or $6,180.

The working Australians tax offset, read on 16 November 2026 in Act No. 49 of
2026 as made and on the ATO legal database: s 61-155(1) entitles an individual
resident at any time in the year whose net labour income (labour amounts less
labour deductions, s 61-155(2)) exceeds the tax-free threshold, $18,200 under
the Income Tax Rates Act 1986 s 3(1); s 61-160 sets the offset as the lesser
of $250 and the basic income tax liability on a taxable income of that net
labour income alone; Sch 3 item 2 inserts it as s 63-10(1) item 3, before
SAPTO (item 5) and the low income tax offset (item 20), non-refundable. No
figure is left to an instrument. Worked by hand: $85,000 of salary, $300 of
work expenses and the $700 standard deduction leave $84,000 of net labour
income, whose tax of $15,452 gives the full $250; $19,000 gives 14% of $800,
$112, and $18,201 gives 14 cents. The labour inputs' `work_related_deductions`
are the s 25-130(2)(c) to (g) kinds, all labour deductions under
s 61-155(2)(a) and (e), and the standard deduction is one under (c).

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

## DGR check

`dgr-check` adds no rate; its rules and data were read on 16 November 2026
(times UTC):

- ITAA 1997 as compiled on 1 October 2026 (legislation.gov.au, C2004A05138,
  Compilation No. 268, C2026C00471, read 00:19 and again 04:31): s 30-15, whose
  table items 1 and 2 no longer hold a '$2 or more' condition for money; s
  30-15(2), no deduction for a gift under a will; s 30-17, a fund, authority or
  institution endorsed for operation; s 30-228(1), what a DGR's receipt for a
  gift states; s 30-229, the Register's DGR statements, with note 1, that an
  entry or its lack does not affect deductibility, and note 2, that an entity
  described or named in Subdivision 30-A with an ABN has one too, endorsed or
  not.
- Treasury Laws Amendment (Delivering an Efficient and Trusted Tax System) Act
  2026, No. 58, 2026, as made (C2026A00058, assented to on 30 June 2026, read
  00:21 and again 04:31): Sch 1 items 1 to 6 repeal the '$2 or more'
  conditions, s 2 table item 2 commences Sch 1 on the first 1 January, 1
  April, 1 July or 1 October after assent (1 July 2026), and item 7 applies the
  repeal to gifts and contributions made on or after 1 July 2024. The ATO's
  page below says the same.
- The ATO's "Gifts and donations" (last updated 6 July 2026, read 00:20 and
  again 04:31): a
  gift is a voluntary transfer of money or property with no material benefit
  or advantage in return, a deduction needs a record such as a receipt, and
  DGR status is checked at ABN Lookup. s 30-228 sets what a DGR's receipt
  states.
- ABN Lookup's "ABN format" (read 00:27), whose worked example ABN
  51 824 753 556 the tests use, and its disclaimer (read 00:27).
- The ABN Bulk Extract on data.gov.au (read 00:18): weekly, current details
  only, Creative Commons Attribution 3.0 Australia, with its readme and XML
  schema. The `DGR` element carries a status date and, for an endorsement only
  for the operation of a fund, authority or institution, the fund's name; it
  carries no DGR item and no end date.

The data is the extract of 11 November 2026 (extract time 2026-11-11T12:20:05),
downloaded at 00:25 to 00:29 as `public_split_1_10.zip` (501,188,909 bytes,
SHA-256 `d6ce13ba5682848163e1560635fc2f6dbc5da0531590129f56b986471b3e77c9`)
and `public_split_11_20.zip` (500,936,917 bytes, SHA-256
`7cef0479739f648834464adf67fddd9f094d733a05b2010983750494ed1fa4ae`), 12.7 GB
of XML in 20 files. `examples/dgr_register.rs` read all 20,658,780 ABN records,
each file's count matching its `RecordCount`, and kept 31,876 ABNs with DGR
entries: 26,456 entries for an entity itself and 8,821 for a fund, authority or
institution, 55 of them with an empty fund name. No individual's record had a
DGR entry. A `DGR` element for a fund carries a `NonIndividualName` and no
`status`; one for the entity itself carries `status="ACT"` and no name. The 55
carry an empty name: read at first as the entity's own entries, they were
rebuilt as funds after ABN Lookup was found to list them so (below). The raw
files were deleted after each run.

The data was compared with ABN Lookup's DGR listing downloads
(https://abr.business.gov.au/Tools/DgrListing, read again at 04:33, after the
ABN Lookup maintenance of 13 to 16 November). They were used only to check the
data, as ABN Lookup states no licence for them. Of the 26,456 entity entries,
26,451 match by ABN and start date. The other 5 ABNs are in neither listing:
ABN Lookup showed 2 of them (read 04:34 and 06:12) as not currently entitled,
with the ABN last updated on 11 and 13 November, at or after the extract. The
listing has 35 entity entries for ABNs not in the data, most starting in 2026;
the 3 earliest-starting showed the ABN last updated on 11 or 13 November. In
each case read, the difference is timing: the record changed at or after the
extract. Of the 8,821 fund entries, 8,753 match by ABN, start date and name, and
66 more once the listing's ' (Public giving fund)' suffix is removed; the
other 2 differ in the name only. The listing has 8 fund rows not in the data.
The listing also gives the DGR item, which the extract does not.

The 55 unnamed fund entries were found in that comparison: the listing gives
each as a fund row with no name, and ABN Lookup's pages for the 3 such ABNs read
(04:34) list the entry under the funds an entity operates, with no name. For
one of them, the Ensemble Theatre Foundation, the entity is endorsed itself
only from 1 January 2024; read as its own entry, the unnamed fund from 1 July
2000 would have shown a 2010 gift to the entity as made to a DGR.

`tests/dgr_check.rs` uses 5 ABNs read the same in the listing: an entity
endorsed itself from 1 March 2006 and for a building fund from 1 July 2000; a
school endorsed only for its building fund; a cancelled ABN whose library fund
is listed as cancelled; the Ensemble Theatre Foundation above; and the
Arthritis Foundation of Australia, Item 1, Endorsed DGR. The Library's worked example ¶7-020 (Tax Examples, Individuals,
reviewed 30 June 2025) deducts a $250 donation in 2024-25 to 'the Arthritis
Foundation' under s 30-15; by name the check gives only candidates, and with
the receipt's ABN it reads the entity as endorsed on the date.

No rival read on 16 November 2026 (00:25 to 00:26) offers a DGR check:
Nagaris's `llms.txt` and skills page, LodgeiT's `llms.txt` and calculators
page, `au-compliance-kit` at e94cef24 and `william-laverty/ato-mcp` at
d3d84941. ABN Lookup itself shows each ABN's DGR status and history, and its
keyed web services, which need a registered GUID, would add both; this
command uses only the open bulk data.
