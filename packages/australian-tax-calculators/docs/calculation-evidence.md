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
| Rental property | Not a Library document: the ATO's Rental properties guide 2026 (NAT 1729-06.2026), Examples 1, 2, 12, 20 to 23, 29, 33, 34 and 40 and Tables 1 and 2, with ITAA 1997 ss 8-1, 26-31, 40-27 and Division 43, TR 2000/2 and TR 93/32 | 30 May 2026 (last updated); 22 November 2026 (read) | One residential property's lines, interest, borrowing expenses, capital works and decline in value for 2020-21 to 2026-27 |
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
| Medicare levy | Not a Library document: Medicare Levy Act 1986 compilations 52 and 53, ITAA 1936 ss 251T and 251U, the ATO's single and family thresholds, its Ashton example and its Priya exemption example | 30 June 2026 (threshold pages); 22 June 2026 (exemption pages); 24 November 2026 (non-resident days) | The 2% rate, the s 7 low-income reduction at the single or SAPTO thresholds, the s 8 family reduction and the s 9 exemption days for 2024-25 to 2026-27, on established SAPTO, spouse, dependant and exemption facts, including a part-year resident's non-resident days (ITAA 1936 s 251U(1)(d) and (2)) |
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
| Government super amounts | Not a Library document: the Co-contribution Act ss 6 to 12E (compilation 28), its Regulations 2004 reg 24(2) and 2022 reg 15(2), ITAA 1997 ss 290-230 and 290-235 (compilation 268), the ATO's Table 25 and its Angelo, Julie and spouse examples | 22 November 2026 (read); 27 April 2026, 8 June 2026 and 2 August 2023 (ATO pages) | Co-contribution, LISTO and spouse contribution tax offset of a full-year resident for 2020-21 to 2026-27 from `income-tests`' components and stated tests |
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

## GST creditable purpose change

Added on 22 November 2026 with `gst-annual-apportionment` and
`gst-financial-acquisitions-threshold`. The A New Tax System (Goods and
Services Tax) Act 1999 was read that afternoon from the Federal Register of
Legislation's API: compilation 96 (C2026C00081, in force from 1 January 2026,
current, no unincorporated amendments) and compilation 82 (C2020C00143, in
force on 1 July 2020). Divisions 21, 66, 129, 131, 136 and 189, Subdivision
38-J and the s 195-1 definitions they use were compared line by line: they
read the same in both, apart from s 131-60(1) step 3, which now refers to an
instrument rather than a Gazette notice. The GST Regulations 2019 (compilation
5, F2025C01015) specify no other amount for ss 131-5(2) and 189-5 or 189-10
and no kind for s 131-40(1)(b). The formulas of ss 11-30, 129-70, 129-75 and
131-40 are images; their alternative text in the EPUB gives them.

GSTR 2000/24 (28 June 2000), GSTR 2003/9 (14 May 2003) and the ATO's 'Annual
private apportionment of GST: Eligibility' (last updated 14 September 2026)
were read from ato.gov.au the same afternoon. The ruling's examples are from
2001 to 2006; the tests move their dates 20 years later, which the unchanged
provisions allow.

s 129-20(1) needs a tax period that 'starts at least 12 months after the end
of' the attributable tax period: the command takes the day after
`attributed_period_end`, adds 12 months, and accepts a 30 June tax period that
starts on or after it. GSTR 2000/24's Farmco (paras 30 and 31: the quarter to
31 March 2001, first adjustment period 1 April to 30 June 2002) fixes the
reading. Para 34's table says '$5,001 to $499,999'; the Act's 'more than
$5,000 but less than $500,000' is applied to the cent.

| Example | Facts given | Result |
| --- | --- | --- |
| GSTR 2000/24 paras 57 to 71, Bill | $300 GST, 90% intended, 60% actual | Increasing **$90**, adjustment period 1 of 2 |
| GSTR 2000/24 paras 64 to 72, XYZ Credit Union | $10,000, 75% intended, 85% actual | Decreasing **$1,000** |
| GSTR 2000/24 paras 75 to 78, Elizabeth | $2,400 after Divisions 19 and 21, 90% to 50% | Increasing **$960** |
| GSTR 2000/24 paras 30 to 36, Farmco | $100,000, quarter to 31 March | 5 adjustment periods from the next June quarter |
| GSTR 2000/24 para 42, Isabelle | Destroyed on 15 March, before the first period | That June quarter is the last |
| GST Act s 131-55 example | $1,100 at 10%; then $10 and $60 adjustments | Credit $100, adjustment **$90**; then **$45** |
| GSTR 2003/9 Examples 1A and 2A, Cedar | $104,550 of $1,100,000; $131,000 of $1,220,000 | Current within; future **exceeded** |
| GSTR 2003/9 Examples 13 and 14, ABC group | $3,600 of $69,000; $10,200 of $99,000 | Current within; future **exceeded** |

`tests/gst_adjustments.rs` holds each case and each boundary.

## GST annual apportionment

See [GST creditable purpose change](#gst-creditable-purpose-change).

## GST financial acquisitions threshold

See [GST creditable purpose change](#gst-creditable-purpose-change).

## GST going concern

Added on 22 November 2026 with `gst-second-hand-goods` and `gst-bad-debt`,
from the same reading of the GST Act as
[GST creditable purpose change](#gst-creditable-purpose-change): s 38-325,
Subdivision 66-A, Division 21, Subdivision 136-A and the s 195-1 definitions
of second-hand goods, valuable metal and overdue read the same in
compilations 82 and 96. GSTR 2002/5 (16 October 2002, last amended 27 March
2013), GSTD 2012/6 (1 August 2012) and GSTR 2000/2 (29 March 2000, last
amended 31 October 2012) were read from ato.gov.au that afternoon. The bad
debt examples are from 2001 and 2002; the tests move Marcus's dates 20 years
later, which the unchanged provisions allow.

`gst-going-concern` works no amount: it reports each condition of
s 38-325 against the stated facts. GSTR 2002/5 para 182 places the written
agreement on or before the day of the supply; paras 168 and 169 put a supply
partly outside the arrangement under s 9-80, which is refused.

s 21-5(1)(c) asks whether the debt 'has been overdue for 12 months or more':
the command takes `overdue_from`, the first day overdue, adds 12 months, and
finds the condition met on that day or later. GSTR 2000/2's Marcus (paras
49AC and 49AD: overdue on 16 April 2001, overdue 12 months on 16 April 2002)
fixes the reading.

| Example | Facts given | Result |
| --- | --- | --- |
| GSTD 2012/6 para 7, Finance Co | $5,500 vehicle from an unregistered vendor | Credit **$500** under an $800 resale GST |
| GST Act s 66-10(1)(b) | The same vehicle resold for $3,300 | Credit **$300** |
| GSTR 2000/2 paras 49AA to 53, Marcus | $550 overdue from 16 April | Decreasing **$50**; none a day early |
| GSTR 2000/2 para 55, ABC Pty Ltd | $660 at 40% taxable | Decreasing **$24** |
| GSTR 2000/2 paras 69 and 70, Isabella | $1,100 written off, 60% creditable purpose | Increasing **$60** |
| GSTR 2000/2 para 72, Y | $4,400 written off, liable for $1,100 of it | Increasing **$100** |

`tests/gst_special_rules.rs` holds each case and each boundary.

## GST second-hand goods

See [GST going concern](#gst-going-concern).

## GST bad debt

See [GST going concern](#gst-going-concern).

## GST error correction

Added on 24 November 2026. Its sources were read that day through the Federal
Register of Legislation: the A New Tax System (Goods and Services Tax)
(Correcting GST Errors) Determination 2023 (F2023L01284, dated 6 September
2023) ss 4 and 6 to 8, the GST Act compilation 96 (C2026C00081) ss 29-10,
29-80, 93-5, 93-10 and 93-15, the GST Regulations 2019 compilation 5 reg
29-80.01 and the Taxation Administration Act 1953 Sch 1 ss 105-65 (as referenced
in F2023L01284 s 4(b)) and 155-35. No ATO example was read; the tests take each
turnover band at its boundary, the last day of each time limit and $75.

The debit error time limit is 'within' a number of months 'after the due
date', so its last day counts: 18 months after 28 February 2025 is 28 August
2026. The value limit is 'less than', so a net sum equal to it fails (and
`net_debit_errors`, being debit error amounts less credit error amounts under s 4,
may be negative). Section
93-5(1)'s 4 years run from the day after the return's due day and end on the
anniversary of it. Under s 29-10(1) to (5), `earlier_attributable_return_given`
and `section_29_10_5_election` ensure an intervening return given whilst
holding the invoice blocks claiming in a later return unless a valid s 29-10(5)
election moved attribution to it. Section 93-15 takes `none`, `partial` or
`full`: `full` preserves entitlement, `none` bars it when the supplier's review
ended without a tax invoice, and `partial` in that case is refused because
s 93-15 ceases entitlement only to the shortfall; s 93-10(1), (4)
and (5) exceptions are refused explicitly. F2023L01284 s 8 provides only that
an error cannot be corrected by amending a later period's assessment.

`tests/gst_corrections.rs` holds each case, each boundary and each refusal.

## GST credit time limit

See [GST error correction](#gst-error-correction).

## Tourist refund

Added on 24 November 2026. Its sources were read that day through the Federal
Register of Legislation: the GST Act compilation 96 (C2026C00081) s 168-5, the
GST Regulations 2019 compilation 5 (F2025C01015, from 1 November 2025) regs
168-5.01 to 168-5.17, the WET Act compilation 32 (C2026C00149) ss 25-5 and
31-10, and the WET Regulations 2019 as made (F2019L01154) regs 25-5.01 to
25-5.07. The ABF's 'Tourist Refund Scheme (TRS)' and 'Making the claim' pages
were read for context: they state the $300, the 60 days and the 30 minutes as
the regulations do, and add that no cash refund is paid.

Reg 168-5.04 sets the $300 for the acquisition Subdivision 168-A specifies;
the 60 days are in Subdivision 168-C. Items failing a Subdivision 168-A
condition never count. Where the qualifying items fall short of $300 but
items failing only the 60 days would bring the total to $300, the command
refuses (`tourist_refund.threshold_unsettled`): neither the regulations nor the
ABF's pages settle whether such items count. The GST Regulations compilation 4
(1 July 2023) and GST Act compilation 95 (1 November 2025) read the same for
these provisions (checked by the review on 24 November 2026), so 1 November
2025 marks the coverage of the law read, not a change in it. Reg
168-5.09's 'within 60 days after the day on which they were acquired' counts
from the next day, so goods bought on 19 January leave in time on 20 March.

No ATO or ABF example with figures was found. The tests work the
regulations' arithmetic:

| Case | Facts given | Result |
| --- | --- | --- |
| One item | $330 with $30 GST | **$30.00** |
| With wine | $330 and $100 of wine with $9.09 GST | GST **$39.09**, wine tax 29% x $50 = **$14.50** |
| Wine at $49.99 | with $300 of goods | wine tax 7.24855, shown **$7.25** |
| 2 items | $150 and $149.99 | below $300: **nil** |
| 60 and 61 days | $200 each | the second fails; the $300 depends on it: **refused** |
| 5-cent item | $299.95 with $27.27 GST and a taxable $0.05 with nil GST | **$27.27** |

`tests/tourist_refund.rs` holds each case, each boundary and each refusal.

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

## Part-year tax-free threshold

Income Tax Rates Act 1986 s 20(1) (compilation 66, 1 July 2026, read on
23 November 2026): with one or more part-year residency periods, the Part I
table applies as if its tax-free threshold were (the $18,200 of s 3(1) less
$4,736) plus $4,736 x the months in the year with a part-year residency period
/ 12. Section 18(1) counts each period from the start of the month residency
began to the end of the month it ended; s 18(2)(a) gives an eligible pensioner
(s 16) no part-year residency period, and s 18(2)(b) excludes a period of the
whole year. Under s 18(1) as written, residency that begins and ends within
one month, or a 1 July resident's residency that ends during July, is no
period, so the threshold stays $18,200 and the inputs are omitted. Division 4
reads the same in the compilations of 1 July 2024 and
28 March 2025, apart from s 16's list of excluded Social Security Act parts.
The Act states no rounding, so the threshold stays exact.

| Case | Source | Months | Threshold | Taxable income | Basic tax |
|---|---|---|---|---|---|
| John, 2025-26 | ATO 'Tax-free threshold for newcomers to Australia' (updated 3 June 2026) | 3 | $14,648 | $20,000 | (20,000 - 14,648) x 16% = **$856.32** |
| Britney, 2025-26 | ATO 'Living overseas and becoming a foreign tax-resident' (updated 3 June 2026) | 6 | $15,832 | $100,000 | (45,000 - 15,832) x 16% + 55,000 x 30% = **$21,166.88** |
| 2026-27 | s 20(1) | 1 | $13,858.666... | $20,000 | (20,000 - 13,858.666...) x 15% = **$921.20** |
| 2027-28 | s 20(1) | 8 | $16,621.333... | $16,000 | **nil** |
| Eligible pensioner | s 18(2)(a) | 3 | $18,200 | as given | the full-year result |

The ATO's John page says the thresholds for the other rates do not change, so
only the first band's lower limit moves. `personal-tax-estimate` composes a part-year
resident with `medicare-levy`'s non-resident days
([Exemptions](#exemptions)). Tests: `tests/part_year_resident.rs`.

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

## R&D tax offset

Added on 22 November 2026. The sources were read that day: ITAA 1997 ss
355-100, 355-105, 355-115, 355-435, 355-440, 355-445, 355-450, 63-10(1) item
35, 67-30 and the s 995-1 'corporate tax rate' definition, and Income Tax
Rates Act 1986 s 23(2), from the ATO's legal database; Act No. 92 of 2020 as
made, whose Sch 4 (the 18.5 percentage point premium, the 8.5% and 16.5% tiers,
the $150 million cap and s 355-115) applies to income years commencing on or
after 1 July 2021 (item 14) and whose Sch 5 Part 2 (the clawback formula) does
too (item 56(3)); the ATO's 'Rates of R&D tax incentive offset' and 'Calculate
the clawback adjustment' (both last updated 13 May 2026), 'Clawback of
feedstock adjustments' and the R&D tax incentive schedule 2022 instructions,
Part E. The current compilation of s 355-100 has the same figures, so the
command runs from 2021-22 to 2026-27.

| Example | Facts given | Result |
| --- | --- | --- |
| Company V (grant), 2021-22 | **$3.6 million**, total expenses **$15 million**, 30%, **$2.3 million** recouped | Offset **$1,650,000**, adjusted **$580,500**, deduction amount **$690,000**, included **$1,265,000** |
| Company F, 2022-23 | **$1 million** at 43.5%, **$500,000** recouped | **$435,000**, **$217,500**, **$125,000**, **$370,000** |
| Company Y, 2023-24 and 2024-25 | **$350,000** and **$150,000**, recouped **$150,000** and **$50,000** | Included **$111,000** and **$37,000** |
| Company Y, alternative | **$1 million** and **$1.5 million**, recouped **$300,000** and **$100,000** | Included **$222,000** and **$74,000** |
| Company Q (grant cap) | **$100,000**, capped recoupment **$9,000** | **$43,500**, **$39,585**, **$2,250**, **$6,660** |
| Company L | **$700,000**, **$250,000** recouped | Included **$185,000** |
| Company J, before and after repayment | **$400,000**, recouped **$200,000** then **$50,000** | Included **$148,000** then **$37,000** |
| Schedule 2022 Part E method | **$200 million**, total expenses **$2,000 million**, 30% | **$15.4 million** + **$51.15 million** + **$15 million** = **$81.55 million** |

The ATO's feedstock example (Landscape Supplies: $22,000 of notional
deductions, $9,000 clawed back, $6,660 included) works the adjusted offset on
$13,000 at 43.5%. On its face s 355-100(2) would limit an offset on a total
under $20,000 to research service provider and CRC expenditure, so the command
refuses a clawback that takes the total, or starts it, under $20,000
(`rd_tax_offset.clawback_below_threshold`), and the test holds the same
feedstock figures on $122,000 instead, which give the same $6,660. The schedule
instructions compare R&D intensity with 2% after rounding it to 4 decimal
places; the command splits the tiers at exactly 2% of total expenses, as s
355-100(1A) does. `tests/rd_tax_offset.rs` also covers the $20,000 threshold,
the $20 million turnover boundary, exempt entity control, the 2% boundary, the
cap at and above $150 million for both offsets, the present year's rate, each
year boundary and each refusal.

## Early stage investor tax offset

Added on 24 November 2026. The sources were read that day: ITAA 1997 ss
360-10 to 360-50 with their history notes, from the ATO's legal database. Act
No. 54 of 2016 inserted Subdivision 360-A for equity interests issued on or
after 1 July 2016; Act No. 8 of 2020 Sch 2 (effective 1 April 2020, for income
years commencing on or after 1 July 2018) added the ESVCLP exclusion, the 30%
income, capital and voting test, the non-cash benefit, 'before the current
year' in s 360-40(1)(a)(ii) and the foreign company test; Act No. 92 of 2020
Sch 5 item 37 added the s 355-450(1) disregard for assessments for income
years commencing on or after 1 July 2021. Nothing is prescribed under ss
360-40(3), 360-40(4) or 360-45(2): Part 3-45 of the Income Tax Assessment
(1997 Act) Regulations 2021 (compilation C16, 27 June 2026) holds Divisions
393, 418 and 419 only, and the Income Tax Assessment Regulations 1997 (1 April
2020) hold no Division 360 provision. The ATO's 'Calculating the early stage
investor tax offset', 'Limits for investors who don't meet the sophisticated
investor test' (both last updated 16 August 2024) and 'Qualifying as an early
stage innovation company' (9 September 2024) give the examples.

| Example | Facts given | Result |
| --- | --- | --- |
| Thomas, 2016-17 | **$100,000**, sophisticated | **$20,000** |
| Savannah, 2016-17 | **$4 million**, sophisticated | **$800,000** capped at **$200,000** |
| Georgia Co, Savannah's affiliate | **$2 million** | Combined with Savannah's, no more than **$200,000**: **$0** after her **$200,000**, the CGT treatment kept |
| Jason, 2017-18 | **$950,000**, **$50,000** carried forward | **$190,000**, of which **$150,000** |
| Tim, 2016-17, not sophisticated | **$50,000**; then **$10,000** more | **$10,000**; then nothing |

Tim's further $10,000 goes to 'an ESIC or shares in any other company'. Section
360-20(1)(b) counts the amount paid for 'the shares' of 'the company', so a
further $10,000 to another company leaves $50,000 for this one; the command
refuses that case (`esic.retail_limit_unsettled`) and the test holds Tim's
loss with the $10,000 paid to the same company. It refuses, too, a case where a
non-cash benefit or money still receivable alone takes the amount over
$50,000, as s 360-20 counts what was 'paid', and a case where an exempt offer
by another ESIC would lift the limit on the ATO's reading ('unless he is a
sophisticated investor in relation to a later ESIC share offer') but not under
s 360-20(1)(a), which looks at this company's offers. Before 2018-19 the offset
is 20% of 'the total amount paid for the shares', so $40,000 paid of a
$60,000 issue gives $8,000; from 2018-19, money 'received, or entitled to be
received', $12,000. A company neither incorporated in Australia nor
registered in the Australian Business Register meets no route of
s 360-40(1)(a) and is not an ESIC. Section 360-40(2)(b) disregards an amount
in 'the company's assessable income', so a 100% subsidiary's s 355-450(1)
amount stays counted. `tests/esic.rs` also covers each incorporation route and year
boundary, the $1 million, $200,000 and $50,000 limits at and a cent over, the
grant and clawback disregard by year, each points combination crossing 100,
the cap with affiliates and carried forward offsets, each investor kind and
each refusal.

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

## FBT instalment

Added on 22 November 2026. Its sources were read that afternoon, 5.30 pm to
6.00 pm AEDT:

- FBTAA, compilation 97 (C2026C00332, 1 July 2026), volume 2, Part VII
  Division 2: s 102 (4 instalments a year), s 103 (due dates), s 104 (notice
  in the approved form), s 105 (the credit on assessment, instalments less
  s 112A credits), s 109 (the relevant fraction: 0.25, 0.50, 0.75, 1.00),
  s 110(1) and (5) (the notional tax amount, and the employer's estimate as
  that amount from the quarter's end), s 111(1) (the formula, read from its
  image's text: '(Notional tax amount x Relevant fraction) - (Previous
  instalments - Previous credits)'; nil unless positive), s 111(2) and (3)
  (an instalment is not payable below $1,000 or an amount the Commissioner
  determines), s 112 (the estimate and the Commissioner's own), s 112A (a
  negative amount is a credit) and s 112B (GIC where the estimate is under
  90% of the tax assessed). The map that scoped this build named TAA 1953
  Sch 1 Div 45; FBT instalments are in the FBTAA, and TAA Sch 1 Part 4-15
  only collects them (the note to s 103).
- ATO, 'Fringe benefits tax (FBT) instalment' (last updated 1 May 2025):
  labels F1 to F4, 6A and 6B; the F3 formula with 25%, 50%, 75% and 100%; nil
  at F3 and a credit at 6B for a negative amount; the 4 reason codes; and the
  example ($28,000 x 75% - $20,000 = $1,000, code 31). It says instalments
  apply after FBT of $3,000 or more; the instrument setting that amount under
  s 111(3) was not found, so the worksheet takes F1 as given and does not
  decide whether instalments are payable.
- ATO, Fringe benefits tax - a guide for employers, chapter 3: the varied
  instalments of $4,000, $3,000 and $4,000 on a $16,000 estimate, and $600 on
  $2,400.

The Act states no rounding for the estimate times the fraction; the worksheet
rounds it half up to the cent. `tests/fbt_instalment.rs` holds each case.

| Case | Source | Result |
|---|---|---|
| December quarter, $28,000 estimate, $20,000 paid | ATO instalment page | F3 and 6A $1,000 |
| $16,000 estimate at quarters 1, 2 (after $5,000) and 3 (after $8,000) | Employer's guide, chapter 3 | $4,000, $3,000, $4,000 |
| $2,400 estimate at quarter 1 | Employer's guide, chapter 3 | $600 |
| $8,000 estimate at quarter 2 after $5,000 | s 112A | F3 nil, 6B $1,000 |
| $1,000.02 estimate at quarter 1 | Rounding | $250.005, $250.01 half up |
| $500.01 of credits after $500 of instalments | s 112A | Refused: a credit is at most the instalments before it |

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

## ATO penalties and interest

Added on 22 November 2026. Its sources were read that day: TAA 1953 ss 8AAC
and 8AAD and Sch 1 ss 280-100, 280-105, 284-75, 284-80, 284-85, 284-90,
284-220, 284-224, 284-225, 286-75, 286-80 and 298-20, ITAA 1997 s 5-15 and
Crimes Act 1914 s 4AA at the ATO's legal database; the Taxation (Interest on
Overpayments and Early Payments) Act 1983, compilation 58 (14 October 2024),
at the Federal Register of Legislation, with ss 8A to 8C, 9, 10, 12D and 12E
also at the ATO's legal database; the ATO's 'General interest charge (GIC)
rates' and 'Shortfall interest charge (SIC) rates' (both last updated
4 September 2026), 'Credit interest rates and calculation' (QC58284, 19 June
2026), 'Penalty units' (QC71196, 26 June 2026) and 'Significant global
entities - penalties' (1 July 2026); PS LA 2006/8, 2011/19, 2011/23 and
2012/5; MT 2008/2 and MT 2012/3.

The GIC table holds 110 quarters, 1 July 1999 to 31 December 2026, and equals
the site's `rates/general-interest-charge/general-interest-charge-rates.csv`
value for value. The SIC table holds 87 quarters from 1 April 2005, each the
GIC rate less 4 points, as ss 8AAD(1) and 280-105(2) imply. The base interest
rate is the GIC rate less 7 points; the ATO's credit interest page publishes
1 July 2023 to 30 September 2026, and all 13 of its quarters agree. October
to December 2026 (4.51%) is not yet on that page; it is both 11.51 - 7 and
SIC's 7.51 - 3.

GIC follows the site's GIC payment plan oracle (`scripts/tests/calculator_oracle.rs`
in `ryanduguid/ryanduguid.github.io`): exact daily compounding, each payment
applied before its day's charge, rounded once. A plan maps to the command as
`amount` the debt, `due_date` the first instalment day, `payments` every
instalment but the last and `to` the day before the last; the balance is then
the plan's final instalment. `tests/ato_interest_penalties.rs` holds the
site's 8 published cases and the 36 seeded oracle cases whose last day falls
in 2020-21 to 2027-28, generated by a port of the oracle's case generator
that reproduces its hand-worked plan and published cases.

The FTL count runs from the due date to the day before the document is
received (PS LA 2011/19 para 16), so a document given 28 days after its due
date bears 1 unit and 29 days 2, as the ATO's significant global entity table
shows ($182,000 for 28 days or less, $364,000 for 29 to 56, to $910,000 above
112 for forms due from 1 July 2026). PS LA 2006/8 Example 14's SIC and
overpayment interest figures do not reproduce from its stated period and rates
under either daily compounding or simple interest, so they are not used.

| Case | Facts given | Result |
| --- | --- | --- |
| Site plan, hand-worked | $1,000 from 1 October 2026, 2 payments of $336.56 | Balance $336.56, GIC $9.68 |
| GIC, hand-worked | $10,000 unpaid 21 November 2025 to 30 June 2026 | GIC $676.86 |
| SIC, hand-worked | $10,000, due 21 November 2024, notice 1 September 2025 | 284 days, $575.45 |
| s 286-80 example | 31 days late, small | 2 units, $660 |
| Across 1 July 2026 | Due 10 June 2026, given 1 August, medium | $330 + $364, x2: $1,388 |
| MT 2008/2 AJ Co and Hill Co | Not reasonably arguable $7,000 on $50,000 tax; $500,000 on $20 million | $0; $125,000 |
| PS LA 2012/5 Scrooge Company | $15,000 intentional disregard, $2,000 failure to take reasonable care | $11,750 |
| MT 2012/3 Benton | $10,000, and $2,000 disclosed during the audit (reasonable care failure assumed) | $2,900 |
| Items 3A to 3C | No shortfall, failure to take reasonable care, 1 August 2026; SGE | $7,280; $14,560 |
| ATO's Evan, in 2025 | $1,926 paid 3 November, due 21 November, at 3.61% | 19 days, $3.62 |
| Overpayment, 2 instalments | $4,000 of $6,000 refunded 15 August 2025 | $3,000 and $1,000 attributed, $86.51 |

## Transfer balance

Added on 22 November 2026. Its sources were read that day: ITAA 1997
ss 292-20 to 292-30, 294-5 to 294-55, 294-80, 294-85, 294-90, 294-200 and
294-230 to 294-245 at the ATO's legal database; IT(TP)A 1997 ss 294-30 and
294-55 (compilation 104, C2026C00450) and the Superannuation (Excess Transfer
Balance Tax) Imposition Act 2016 (C2016A00080) at the Federal Register of
Legislation; the ATO's 'Calculating your personal transfer balance cap'
(QC60627, 31 March 2026), 'Transfer balance account' (18 June 2025),
'Excess transfer balance' (QC54355, 7 February 2025), 'Upcoming personal
transfer balance cap changes' and 'General interest charge (GIC) rates'
(4 September 2026); LCR 2016/9.

The general transfer balance cap is $1.6 million for 2017-18 to 2020-21,
$1.7 million for 2021-22 and 2022-23, $1.9 million for 2023-24 and 2024-25,
$2 million for 2025-26 and $2.1 million for 2026-27 (the CCH note to
s 294-35, the ATO's 'Contributions caps' page and the 2026 pages above); the
last 3 come from `CONTRIBUTION_CAPS`. Earnings use the GIC table of
[ATO penalties and interest](#ato-penalties-and-interest). A title search of
the Federal Register on 22 November 2026 found no rate determined under
s 294-235(3).

`tests/transfer_balance.rs` holds the 6 indexation examples of 'Calculating
your personal transfer balance cap' (Nina, Leanne, Nada twice, Maryanne and
Simon) and the 'Transfer balance account' examples of Elisabeth (a $30,000
negative balance, then $10,000 of excess) and Paolo. The Paolo example states
$2,745.15 of earnings for 31 days on $350,000; no published GIC rate gives
that (it implies about 9.2% a year, where July to September 2017 was 8.73%),
so the test keeps its shape and takes the figure, $2,604.41, from the oracle.
Earnings, tax and balances come from an independent exact-fraction oracle
over the ATO's GIC page, written apart from this code.

## Excess contributions

Added on 22 November 2026. Its sources were read that day: TAA 1953 s 8AAD
and Sch 1 ss 97-1 to 97-35 and 136-10 in compilation 227 (C2026C00468), with
the s 97-25(1)(c) and s 97-30(1) formulas read from the compilation's images;
Sch 1 former Div 95 in compilation 180 (C2021C00276); the Superannuation
(Excess Concessional Contributions Charge) Act 2013 ss 3 to 5 (C2013A00116);
Treasury Laws Amendment (More Flexible Superannuation) Act 2021 Sch 2 items 6
and 14 (C2021A00045); ITAA 1997 ss 292-20, 292-25 and 292-30; and the ATO's
'Contributions caps' page (11 September 2026), Tables 3 and 5.

The charge's rate is the base interest rate plus 3 points over the days in the
calendar year, which is the SIC table's rate: every quarter of the ATO's
Table 3, July 2013 to September 2026, equals the GIC rate less 4 points.

The proxy rate is s 97-30's: the average of the base interest rates for each
day of the contributions year plus 7 points, over the days in the calendar
year of its first day. The ATO's Table 5 rounds a simple average of the 4
quarterly rates to 2 places and divides by the days in the financial year:
the Act's day-weighted rate rounds to the ATO's in every year from 2013-14 to
2025-26 but 2022-23 (Act 9.451452%, ATO 9.46%), and the divisors differ in
2015-16, 2019-20 and 2023-24 (ATO 366, Act 365). The command follows the Act
and warns with the ATO's figure. No rate determined under s 97-30(2) was
found. Test vectors from the oracle: $100,000 for 2024-25 to 15 September
2025, $14,663; $50,000 for 2020-21 to 30 November 2021, $5,251; the charge on
$1,000.00 for 2020-21 due 1 December 2021, $44.17, and on $2,500.50 for
2013-14 due 21 November 2014, $205.03.

## Vic property taxes

This worksheet was added on 24 November 2026. Each source was read on 23 or 24
November 2026:

- Land Tax Act 2005 (Vic), authorised version 086 (1 May 2026): ss 19(1A),
  34A to 34C, 35(3), 37A, 88EA and 88EB, and Sch 3 cl 21 and 23 (no transitional
  rule on the 2 preceding years); the State Revenue Office's 'Vacant
  residential land tax (current rates)' (updated 13 November 2026: 1%, 2% and
  3% by consecutive liable years; 1% for undeveloped metropolitan land and new
  residential land unused, unoccupied and unsold for more than 3 years; 'Before
  2025, VRLT was 1% ... for all properties'), 'Understanding vacant residential
  land tax' (18 November 2026: undeveloped land from 1 January 2026) and
  'Understanding your vacant residential land tax assessment' (22 October 2026).
- State Taxation Acts and Other Acts Amendment Act 2023 (Vic), as made: s 2(3)
  and (4) (Part 10 Division 1 from 1 January 2025, Division 2 from 1 January
  2026), s 27A (s 35(3)(a) to (c) as in force for 2025) and s 28A (ss 88EA to
  88EC). As s 88EB needs an s 88EA exemption in a preceding tax year, the
  concession is refused for 2025.
- Interpretation of Legislation Act 1984 (Vic), authorised version 134: s 44(1)
  (a period expressed to begin on a day leaves that day out) and s 44(6)(a)
  (midnight is the point at which the day ends). The transition period therefore
  ends on the tenth anniversary of the entry date; an entry on 31 December, whose
  period ends at the midnight that fixes liability, is refused.
- Commercial and Industrial Property Tax Reform Act 2024 (Vic), authorised
  version 004: ss 2 (commencement 1 July 2024), 3 ('CIPT taxable land',
  'transition period' of 10 years from the entry date), 7, 17, 19 (1%; 0.5% for
  BTR land) and 22; the State Revenue Office's 'Understanding commercial and
  industrial property tax' (29 September 2026): a 1 July 2024 entry first pays
  in 2035 and a 1 August 2034 entry in 2045.
- Windfall Gains Tax Act 2021 (Vic), authorised version 010: ss 2, 3, 6 to 19
  and 36 to 41; no Windfall Gains Tax regulations are in force
  (legislation.vic.gov.au, statutory rules in force, read 24 November 2026), so
  no deductions are prescribed under s 10; the State Revenue Office's
  'Understanding windfall gains tax': $187,500 on a $400,000 uplift and $300,000
  on $600,000.
- Planning and Environment Act 1987 (Vic), authorised version 168: Part 9B is
  the growth areas infrastructure contribution, not windfall gains tax.

| Case | Expected | Source |
| --- | --- | --- |
| Windfall gains tax on a $400,000 uplift | $187,500.00 | SRO example |
| Windfall gains tax on a $600,000 uplift | $300,000.00 | SRO example |
| Uplifts of $100,000, $100,001, $499,999 and $500,000 | nil, $0.63, $249,999.38, $250,000.00 | s 9 |
| Titles up $300,000 and $150,000 and down $50,000 | $450,000 taxable, $218,750.00 | ss 9, 16(3) |
| Entry 1 July 2024 and 1 August 2034 | earliest possible tax years 2035 and 2045, nil in 2025 and 2026 | SRO; s 3 |
| Entry 1 January 2025 | period ends 1 January 2035, earliest possible tax year 2036 | s 3; ILA s 44(1) |
| Entry 31 December 2024 | refused: the period ends at the midnight that fixes liability | s 3; ILA s 44(6)(a) |
| Uplifts of $50,000 and $50,001 on 2 titles | $100,001 taxable, $0.63 | ss 9, 16(1) |
| Vacant home at $800,000: no, 1 and 2 liable years | $8,000, $16,000, $24,000 | s 35(3)(b) |
| Undeveloped land at $1,200,000 in 2026 | $12,000 | s 35(3)(a) |
| s 88EB concession at $650,000 in 2026, whatever the land's class now | $6,500; refused for 2025 | ss 88EA, 88EB |

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

## Alcohol excise

This worksheet was added on 22 November 2026. The Excise Tariff Act 1921 was
read that day in compilation 58 (1 July 2026): ss 6A and 6L and the
Schedule's definitions and items 1 to 3, whose text is the same in
compilations 54 (31 March 2022), 55 (1 July 2023) and 56 (1 July 2024). The
Excise Act 1901 ss 77FA and 78 were read in compilation 69 (27 August 2026),
and the Excise Regulation 2015 s 8, ss 62 and 64 and Schedule 1 in
compilation 6 (29 November 2025), with compilations 4 and 5 for the $350,000
cap. The rates are the Commissioner's Notices of Substituted Rates of Excise
Duty No. 1 and 3 of each year from 2022 to 2026, the February and August
indexation; every value equals the ATO's Historical Excise Rates workbook
(data.gov.au, 1 September 2026; 336 values across 28 dates from 1 February
2022) and, for 2026, the ATO's 'Excise duty rates for alcohol' (QC63605, 29
July 2026). The ATO's 'Calculating alcohol excise duty' (QC63603), 'Remission
scheme for alcohol manufacturers' (QC65935, 18 November 2026) and chapter 6
of the Excise guidelines for the alcohol industry were read the same day.

Truncating the litres of alcohol to one decimal place is the option the
guidelines give for the excise return ('may be truncated'); the command
takes it, as the ATO's examples do. Section 77FA does not say which
subitem's rate applies where the labelled strength would move the goods to
another subitem, so that case is refused. The part-year cap is truncated to
the cent; the ATO's example shows whole dollars.

`tests/alcohol_excise.rs` holds these cases.

| Case | Source | Worksheet |
| --- | --- | --- |
| 10 cases of 24 x 355 ml at 5% on 8 March 2023 | QC63603 (Brewery Co) | 1.10 at $57.79; 3.2802 LAL, 3.2; $184.92 |
| 840 litres of gin at 37.2% on 10 February 2023 | QC63603 (Bottler's Gin) | 3.2 at $97.90; 312.4 LAL; $30,583.96 |
| The same gin on 10 February 2025 | Excise guidelines Example 6M | $104.31; $32,586.444 truncated to $32,586.44 |
| 840 litres of brandy at 37.2%; 100 litres of beer at 5% | Examples 6B and 6D | 312.48 LAL; 3.85 LAL |
| RTDs of 189.5 LAL on 2 February 2025 and 1,041.3 LAL after | Example 6O | 2 lines: $103.89 and $104.31; $19,687.15 and $108,618.00 |
| 3 days of brandy, 2,356.79 LAL | Examples 6K and 6L | One line, 2,356.7 LAL |
| Every subitem on each period's first and last day | Notices No. 1 and 3, 2022 to 2026 | 10 LAL x each rate |
| Draught beer from 3 February 2025 to 2026 | Excise Tariff Act s 6L | 1.11 stays $43.39 |
| A licence from 30 October 2026 | QC65935 | Cap $267,397.26 |
| Remission lines over the cap in 2025-26 and 2026-27 | Excise Regulation 2015 item 10(c)(i) | $350,000; $400,000 |

## ASIC fees

This worksheet was added on 24 November 2026, when its sources were read.
The amounts are ASIC's fee data (`fees-data-current.json`, YEARFEE 2026,
SHA-256 03281f57…866da), which fills INFO 30 'Fees for commonly lodged
documents' and 'Fee indexation' ('From 1 July 2026'). Each was recomputed
from the regulations: the Corporations (Fees) Regulations 2001, Compilation
No. 41 (1 January 2025), Schedule 2 items 1(a)(i) and (ii) (registration), 8
(name reservation) and 14 (late lodgement), indexed under reg 3(3) to (7),
50 cents rounding up; the Corporations (Review Fees) Regulations 2003,
Compilation No. 11 (12 March 2025), Schedule 1 Part 1 items 1, 2, 4 and 5 and
Part 3 items 1 and 2, indexed under reg 4(6) to (10) from 2024-25, 50 cents
rounding down; and the Business Names Registration (Fees) Regulations 2022
s 6 items 1 to 4, indexed under s 6(2) to (7) with the factor to 3 places,
50 cents rounding down. The factors use the ABS CPI March quarters 2018 to
2026 (97.70 and 101.70 for 2025 and 2026 on the new reference base).

The first month after the due date follows the Acts Interpretation Act 1901
(compilation of 28 March 2026) s 2G(1) and s 36(1) item 6, and the
Corporations Act 2001 (compilation 149, 19 September 2026) s 105(1). INFO 30
and ASIC's 'Late fees' page tier by the day ASIC receives the payment or
document. Section 36(2) of the Acts Interpretation Act and s 105(3) of the
Corporations Act can move a deadline that falls on a weekend; the worksheet
refuses a received date such a move could change. The site page cites
company registration to Schedule 1 item 1(a); Compilation No. 41 puts it in
Schedule 2 item 1(a), and Schedule 1 item 1 is an AFS licence application.

## Beneficiary tax offset

Added on 22 November 2026. ITAA 1936 s 160AAA was read in compilation 193
(1 October 2026), and s 13 of the Income Tax Assessment (1936 Act)
Regulations 2025 (F2025L01060) and of the Regulation 2015 (F2015L01415,
compilation 8) at the Federal Register of Legislation that day. Both formulas
are images; their alternative text reads '0.15 times (Rebatable benefit amount
minus $6,000)' and, above the threshold at the upper conclusion of the lowest
marginal tax rate, that plus '0.15 times (Rebatable benefit amount minus
Threshold ...)'. The 2025 Regulations apply to years starting after they
commenced (s 23), so from 2026-27. The threshold is $45,000 in every year
from 2020-21 to 2026-27 (Income Tax Rates Act 1986 Sch 7 Part I, compilations
61 and 66). The ATO's 'Beneficiary tax offset' (QC72199, 8 June 2026) gives
no worked figures; the cases below are worked by hand from s 13.

| Case | Worksheet |
| --- | --- |
| $6,000, and $6,000.99 rounded down to it | nil |
| $6,001 | $1 (15 cents, rounded up) |
| $30,123 | $3,619 ($3,618.45 up) |
| $45,000; $45,001 | $5,850; $5,851 |
| $50,000.75 | $7,350 |

## Invalid carer tax offset

Added on 22 November 2026. ITAA 1997 Subdivision 61-A was read in compilation
268 (1 October 2026), and the ATO's 'T5 Invalid and invalid carer'
supplementary return instructions for 2021 to 2026 that day (last updated
27 May 2021, 26 May 2022, 25 May 2023, 11 April 2025, 27 May 2025 and
30 May 2026). Each gives the year's amount, the daily rate for part of the
year (the amount over the days in the year, to the cent), the FTB Part B
limit and the income at which the offset is nil ($282 plus 4 times the
amount):

| Year | Amount | Daily rate | FTB Part B limit | Nil from |
| --- | --- | --- | --- | --- |
| 2020-21 | $2,816 | $7.72 | $100,000 | $11,546 |
| 2021-22 | $2,833 | $7.76 | $100,900 | $11,614 |
| 2022-23 | $2,943 | $8.06 | $104,432 | $12,054 |
| 2023-24 | $3,152 | $8.61 | $112,578 | $12,890 |
| 2024-25 | $3,300 | $9.04 | $117,194 | $13,482 |
| 2025-26 | $3,396 | $9.30 | $120,007 | $13,866 |

The ATO's worksheets 1F, 2B and 3C multiply by the share of maintenance after
the income reduction; s 61-45 reduces the amount 'under sections 61-30 to
61-40', after s 61-40(1)(b)'s apportionment for other contributors. The
orders differ, so a dependant also maintained by another person is refused.
`tests/individual_offsets.rs` holds each year's amount, cut-out, daily rate
and limit.

## Zone tax offset

Added on 22 November 2026. ITAA 1936 ss 79A and 79B and Schedule 2 were read
in compilation 193 and ITAA 1997 Subdivisions 961-A and 961-B in compilation
268 that day, with the ATO's 'T4 Zone or overseas forces' supplementary return
instructions for 2021 to 2026, 'Zone tax offset' (QC105018, 8 June 2026) and
the 'Australian zone list' (1 July 2026). Every year's T4 gives the same fixed
amounts ($1,173, $338, $57 and $338), percentages, notional offsets ($376,
$282, $1,607) and sole parent daily rate ($4.40; $4.39 in 2023-24); the ATO
had not published the 2027 instructions, so 2026-27 uses the law's amounts.

Two points are refused rather than decided. Section 79A(3B)(a) needs residence
for more than one-half of the year, 184 days of 2023-24, but the ATO says 183
days or more, so 183 days of 2023-24 are refused in that year and when carried
to a later one. Section 79A(4) makes a relevant preceding year 'any of the next
4 preceding years of income other than the immediately preceding year', but
the ATO's Garry example carries days from 2020-21, the fifth year before
2025-26.

| Case | Source | Worksheet |
| --- | --- | --- |
| 365 days in each area, no base amount, each year | T4 table 1, 2021 to 2026 | $1,173, $338, $57, $338 |
| Narendra: Lord Howe Island, 100 days in 2024-25 and 124 in 2025-26 | QC105018 | resident under s 79A(3B)(d); $1,173 |
| Garry: 122 days in 2020-21 and 92 in 2025-26 | T4 2026 | refused (fifth year back) |
| Sharon: 100 days of overseas service | T4 2026 worksheet 6 | $338 x 100 / 183 = $184 |
| Zone A with a student; 2 children; Zone B with a child | T4 table 2 and worksheets 4 and 5, by hand | $526; $667; $132 |
| A child with $1,000 of income; a child for 100 days | T4 worksheets 1 and 2, by hand | $197 base, $436; $103.01 base, $389 |
| Sole parent for the year with a child; for 100 days | T4 worksheet 3, by hand | $1,329; $440 base |

## Net medical expenses tax offset

Added on 22 November 2026. ITAA 1936 ss 159P and 159Q were read in
compilation 161 (6 April 2019), the last before the offset ended
(s 159P(1A)), and the ATO's 'T5 Total net medical expenses for disability
aids, attendant care or aged care' instructions for 2017, 2018 and 2019 that
day (last updated 25 May 2017, 31 May 2018 and 30 May 2019). The 2016 page at
the same address returns 404, so 2015-16 is refused. The limits are s 159Q's
$2,000 and $5,000 indexed; the thresholds are $90,000 and $180,000 plus $1,500
for each child after the first in each year.

| Year | Lower limit (20%) | Higher limit (10%) | $10,000 at $90,000; at $90,001 |
| --- | --- | --- | --- |
| 2016-17 | $2,299 | $5,423 | $1,540.20; $457.70 |
| 2017-18 | $2,333 | $5,504 | $1,533.40; $449.60 |
| 2018-19 | $2,377 | $5,609 | $1,524.60; $439.10 |

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

## CGT cost base adjustments

This worksheet was added on 22 November 2026, and its sources were read that
day from 13:50 to 14:05 AEDT. ITAA 1997 ss 104-70, 104-71, 104-107A to
104-107E, 104-135, 109-5, 110-25, 110-55, 114-1, 114-10, 115-25, 115-30 and
125-55 to 125-85 were read on the ATO's legal database
(https://www.ato.gov.au/law/view/document?docid=PAC/19970038/104-135 and the
same address for each section) with their history notes. s 104-135 was last
amended by No 91 of 2008, s 104-70 by No 53 of 2016, ss 104-107B to 104-107E
by No 53 of 2016 and Subdivision 125-B by 2014 at the latest; s 104-107A was
substituted in part by No 15 of 2019 for the 2018-19 income year and later
years, so the worksheet starts at 2018-19. Act No. 49 of 2026 changes Division
115 and s 102-5 from the income year that includes 1 July 2027, so 2027-28 is
refused, as for CGT parcels.

The ATO material: CR 2018/59 (Wesfarmers demerger of Coles, implemented
28 November 2018: 71.09% and 28.91%, $5.68 a share) and the ATO's 'Wesfarmers
Limited – demerger of Coles Group Limited (2018)' (last updated 28 June 2019,
examples 1 to 4); CR 2021/50 (Woolworths demerger of Endeavour, implemented
1 July 2021: 85.81% and 14.19%, 71 cents a share), which has no worked
example; the Guide to capital gains tax 2025 (examples 20, 30 and 45); and the
Personal investors guide to capital gains tax 2025, Part C (examples 26 to
28). No CPI figure or indexation factor is used: a company's parcel from
before 22 September 1999 is refused, and another entity's is worked without
indexation, as the Guide's example 45 does for a taxpayer who has not chosen
indexation.

Each side of a demerger is the sum of the post-CGT head shares' cost bases
times its percentage, rounded to the cent on its own, as the ATO's examples
round it (the Guide's example 30 gives $2,373.43 and $126.58 of $2,500), so
the sides can differ from the sum by a cent. Each side, and each E4 or AMIT
amount, is spread over the parcels by units, the last parcel taking what is
left. A G1 payment per share is multiplied by the units and rounded to the
cent.

An E4 or AMIT amount happens just before the year's end, or just before
another CGT event to the units in that year after the payment, which the
operator gives as `before_cgt_event` (ss 104-70(3) and 104-107B(4)). As the
payment date is not an input, an amount without named parcels is refused when
a parcel of the security was acquired in that year, since units bought after
the payment take no part of it (s 104-70(1)). A parcel given an E4 and an AMIT
amount for one year, or 2 AMIT amounts, is refused: an AMIT unit has no CGT
event E4 (s 104-70(1A)) and one AMIT cost base net amount a year
(s 104-107C).

`tests/cgt_cost_base_adjustments.rs` and the unit tests in
`src/engine/cgt_parcels/demergers.rs` hold these cases.

| Case | Source | Worksheet |
| --- | --- | --- |
| Tom: 2,000 Wesfarmers shares at $4.00 and 1,000 at $15.00 (post-CGT), no roll-over | COLES examples 1 and 2 | $3,360 G1 gain; $16,350.70 and $6,649.30 ($5.4502 and $2.2164 a share) |
| Tom with roll-over | COLES example 1 | Gain disregarded; the same cost bases |
| Sarah: 150 shares at $43.00 | COLES example 3 | $4,585.31 and $1,864.70 ($30.5687 and $12.4313 a share) |
| Denise's 200 at $5.00 and 150 at $30.00, as a resident | COLES example 4 | $3,909.95 and $1,590.05, spread $2,234.26 and $1,675.69, $908.60 and $681.45 |
| 1,000 Woolworths shares at $30.00, or at $0.50 (fabricated) | CR 2021/50 paras 10 and 16 | $25,743.00 and $4,257.00; at $0.50 a $210.00 G1 gain |
| Rob: 1,500 shares at $5.00 from 1996, 50 cents a share (date moved into 2025-26) | Guide example 45 | $6,750.00, unindexed, with a warning; refused for a company |
| Bob: cost base $1,200, reduced cost base $1,050, $105 tax-deferred | Guide example 20; investors guide example 26 | $1,095.00 and $945.00 |
| Ilena: $5,000 and $4,700, $30 tax-deferred and $35 tax-free | Investors guide example 27 | $4,970.00 and $4,635.00 |
| Miriam: $55 a unit, $10 a unit AMIT shortfall, or excess | Investors guide example 28 | $65 and $45 a unit |
| Peter: 400 shares at $4.60, 5% to the new shares | s 125-80(3) example | $1,748.00 and $92.00 |
| Anita: $2,500 at 94.937% and 5.063% | Guide example 30 | $2,373.43 and $126.58 |
| G1, E4 and AMIT amounts above the cost base, and an AMIT excess on a nil cost base | Worked by hand under ss 104-135(3), 104-70(4) and (5) and 104-107A(3) | Gains of the excess, then of the whole excess; both bases nil |

COLES is the ATO's page above. The tests also cover a parcel bought after the
record date, an amount spread by units or over named parcels, an amount just
before a sale in the year, a G1 on a new share either side of 12 months from
the head share's date, the parcels passed to `cgt-parcels`, each year boundary
and each refusal.

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

## Land tax

This worksheet was added on 23 November 2026. Each source was read that day,
from 23:15 to 23:35 AEDT:

- Revenue NSW, 'Land tax thresholds and rates' (last updated 16 March 2026):
  the general threshold of $1,075,000 and premium threshold of $6,571,000 for
  2024 onwards, the 2 rates and the $500 example on a 3-year average of
  $1,100,000; 'How trusts are assessed' (a special trust's flat 1.6% to the
  premium threshold, then 2%; $9,600 on $600,000); 'What is surcharge land tax'
  (last updated 8 April 2026: 5% from 2025, no threshold; examples 1 to 3).
  The Land Tax Act 1956 and Land Tax Management Act 1956 (NSW) were not read:
  legislation.nsw.gov.au answered with a Cloudflare challenge, also after one
  change of network and a slower retry.
- Land Tax Act 2005 (Vic), authorised version 086 (1 May 2026), Sch 1 Tables
  1.6, 3.6, 4.5 and 5.5 for 2024 to 2033; the State Revenue Office's 'Land tax
  (current rates)' (updated 29 September 2026), the same 4 tables.
- Land Tax Act 2010 (Qld), current reprint, s 32 and Schs 1, 2 and 3; the
  Queensland Revenue Office's rates for individuals ($1,300 on $680,000,
  $87,000 on $6,400,000), companies and trusts ($106,500; $8,420), absentees
  and foreign companies and trusts ($2,300 with a $1,500 surcharge on
  $400,000).
- Land Tax Act 2002 (WA), compilation 02-d0-04, s 5 Table 11 (2015/16 and
  subsequent financial years); Metropolitan Region Improvement Tax Act 1959
  (WA), compilation 04-a0-08, s 10 Table 3; the Department of Treasury and
  Finance's 'Land tax assessment' (last updated 20 May 2026).
- Land Tax Act 1936 (SA), version 4.12.2025, ss 6 and 8A and Sch 1 Pts 2 and
  3; RevenueSA's 'Rates and thresholds', the 2026-27 general and trust tables
  (thresholds $936,000, $1,504,000, $2,188,000 and $3,504,000).
- Land Tax Rating Act 2000 (Tas), current (authorised 12 July 2024), ss 6, 6A
  and 7 and Sch 1; the State Revenue Office's 'Rates of land tax' (from 1 July
  2025) and foreign investor land tax surcharge guideline (December 2025),
  example 1.
- Land Tax Act 2004 (ACT), republication 34 (26 December 2025), ss 9 and 17E;
  the Taxation Administration (Amounts Payable—Land Tax) Determination 2026
  (DI2026-152), ss 4 to 7.
- NT Government, 'Property taxes': no land tax in the Northern Territory.

Read on 24 November 2026 after review: Land Tax Act 2005 (Vic) s 46IA
(version 086: a fixed or unit trust's absentee surcharge is on its absentee
beneficiaries' share; a discretionary absentee trust pays Table 5.5 on the
whole); Land Tax Act 2000 (Tas) ss 16A and 16B(1) (foreign investor land tax
is a component of land tax, assessed for each folio); Metropolitan Region
Improvement Tax Act 1959 (WA) s 10 (Table 3, for 2009/10 onwards). RevenueSA's
2026-27 tables were read from the page's own HTML, whose expandable tables
hold every threshold above.

The site's land tax calculator (ryanduguid.github.io, main at 1aa0a6cb) was
run on its 23 calculated cases, the 2 further worked figures in its Rust
oracle and every band edge of every scale (each edge less 1, the edge and the
edge plus 1, under every choice the page offers), 875 rows in
`tests/data/land_tax_site_cases.json`. The worksheet gives the same 4 figures
on 711 rows. The other 164 differ only where an Act differs from the page:
103 rows move and 61 are refused. Tasmania's figures are worked again in the
test from Schedule 1 and s 7, apart from the worksheet. Of the page's own 23
cases, 19 agree, 3 move (South Australia's $936,050 and the Queensland Revenue
Office's 2 surcharge examples, $1,500.03 for $1,500) and 1 is refused (its
Victorian absentee trust at $1,800,000); its 2 refused cases are refused
alike.

| Rule | Act | Rows |
| --- | --- | --- |
| Queensland's surcharge is 3.0c for each $1 more than $349,999, from $350,000; the page and the Queensland Revenue Office work 3% of the value over $350,000, 3 cents less | Land Tax Act 2010 (Qld) Sch 2 Pt 2, Sch 3 Pt 2 | 70 moved |
| South Australian land tax under $20 is nil; the page shows the amount | Land Tax Act 1936 (SA) s 6 | 3 moved |
| Tasmanian amounts are rounded to 10 cents; the page applies no rounding | Land Tax Rating Act 2000 (Tas) s 7 | 30 moved (36 amounts) |
| A Victorian absentee trust's surcharge is on its absentee beneficiaries' share; the page charges Table 5.5 on the whole | Land Tax Act 2005 (Vic) s 46IA | 53 refused |
| Tasmanian foreign investor land tax applies to residential general land, part of the owner's taxable land; the page takes a larger surcharge value | Land Tax Act 2000 (Tas) Pt 3 Div 1A; Land Tax Rating Act 2000 (Tas) s 6A | 8 refused |

Queensland's figure is the Act's, before any rounding the Commissioner
applies to the amount collected (Taxation Administration Act 2001 (Qld)
s 152 permits rounding down to 5 cents); the Queensland Revenue Office's
examples show 3% of the value over $350,000. Where rounding Tasmania's land
tax and foreign investor land tax to 10 cents each and rounding their sum
differ, which s 7 requires is unsettled, and the worksheet refuses; no row of
the fixture does so.

The tests also work by hand the s 7 rounding ($125,001, $125,011, $125,012,
$125,013, and the 5-cent ties $125,100 and $500,010), the s 6 minimum
($939,900 nil; $939,901 and $940,000, $20; $940,001, $20.50; trust $25,000
nil and $25,001, $125.50) and Queensland's absentee and foreign company
scales at $6,400,000 ($103,000 and $106,500).

## Workers compensation premium

This worksheet was added on 24 November 2026. Each source was read that day,
from 03:15 to 03:30 AEST:

- WorkCover Premiums Order (No. 34) 2026/2027, Victoria Government Gazette P1
  (29 April 2026), from WorkSafe Victoria's PDF: cl 5 (new and small
  employer), 6 (rounding), 7 (exemption limit $7,500), 8 (premium payable plus
  GST), 9 (WorkCover premium, preliminary premium, premium rate), 10 (buy-out),
  11 (deductible benefit) and 12(4) and (5) (employer performance rating 1
  for a small or new employer; small employer, total annualised remuneration
  for 2025-26 of $200,000 or less). Gazette S 284 (2 June 2026) items 1 to 3:
  minimum premium $400, buy-out value 0.1, deductible amount 15,500.
- WorkCover Queensland Notice (No. 1) of 2026, Queensland Government Gazette
  No. 43 (26 June 2026), from WorkCover Queensland's PDF (scanned; read by
  OCR): ss 1, 3.1 and 3.2, 4.1 (minimums of $200, $150, $100 and $50 by the
  quarter a policy commences in, inclusive of GST and duty), 5.2, 8.3, 10.1, 11
  and 12. Duties Act 2001 (Qld), current as at 22 October 2026, ss 356, 357,
  361 and 362(3) (5% of net premiums for accident insurance); the Queensland
  Revenue Office's public ruling DA356.1 (net premiums include the GST).
- icare, 'Premium calculation for small employers' (the formula, the $30,000
  small employer limit, Rob's Restaurant, the definitions of D, SER and A) and
  'Premium updates for 2026-27' (updated 9 November 2026: the $240 minimum;
  2025-26 WIC rates kept). SIRA's Workers Compensation Market Practice and
  Premiums Guidelines 2026 were not read: sira.nsw.gov.au answered with a
  Cloudflare challenge, also after one change of network and a slower retry.
- ReturnToWorkSA, 'Your premium explained' (the formula, the discount bands
  and examples 1 to 3), 'Apprentice incentive', the 'Your work injury
  insurance' booklet (June 2026: the $200 minimum 'subject to any rounding';
  new employers pay the base premium for a full year) and its 2026-27 premium
  calculator workbook (the premium is `INT` of the sum after the minimum; the
  band is looked up with `MATCH(BP, {..., 9999.99}, -1)`). The ReturnToWorkSA
  Premium Order 2026-27 in the South Australian Government Gazette was not
  read: governmentgazette.sa.gov.au answered with a Cloudflare challenge.
- NT WorkSafe, 'Understanding workers compensation premiums' (the builder's
  example); WorkSafe Tasmania, 'Workers compensation insurance'; the ACT's
  suggested reasonable premium rates; WorkCover WA's rates page.

The site's workers compensation calculator (ryanduguid.github.io, main at
dcc7ecbe) was run on its 25 page cases, the 31 edge inputs of its unit tests
and a sweep of each scheme's edges, 1,326 rows in
`tests/data/workers_compensation_site_cases.json`. The worksheet gives the
same 6 figures on 849 rows and the same refusal on 308; it refuses the other
169, where a source does not settle the case or the page goes further than
its source (table below). Of the page's own 25 cases, 18 agree and 7 are
refused alike.

| Rule | Source | Rows |
| --- | --- | --- |
| A South Australian base premium above $9,999.99 and below $10,000 (and at each other bound) is refused | ReturnToWorkSA's table and calculator differ | 24 refused |
| A New South Wales apprentice incentive is refused | icare says it is 'discounted from premium after premium payable is calculated', not whether before or after GST | 61 refused |
| A New South Wales average performance premium is compared with $30,000 unrounded | icare does not say APP is rounded before the test; the page rounds it first | 12 refused |
| Queensland actual wages are refused | the 2026-27 notice sets no year-end adjustment for a policy commencing in 2026-27 | 48 refused |
| A Victorian remuneration that rounds to $0 is refused | Premiums Order (No. 34) cl 11 divides by the remuneration | 24 refused |
| N / 365 is rounded to 6 places before the deductible | Premiums Order (No. 34) cl 6(1) and (4); the page uses the exact fraction | none in the fixture; $150,000 at 7.123% for 1 day gives $3.03, not $3.02, in the tests |
| A Queensland premium that rounding the estimated premium and the apprentice discount first would move is refused | the notice does not say which is rounded | none in the fixture; in the tests |

## Motor vehicle duty

This worksheet was added on 24 November 2026. The Acts and determinations were
read that day:

- New South Wales: the Duties Act 1997 (NSW) could not be read (NSW
  Legislation returned a Cloudflare challenge and AustLII refused the
  request). The scale rests on Revenue NSW, 'Motor vehicle duty' (last updated
  16 October 2025), and Service NSW, 'Check motor vehicle stamp duty': $3 for every $100 or part to
  $45,000, then $1,350 plus $5 for every $100 or part.
- Victoria: the Duties Act 2000 (Vic) ss 3(1), 218 and 219 (authorised
  version 141), with the 2026-27 luxury car tax threshold of $80,809 for other
  vehicles (s 218(5)), the State Revenue Office's 'Motor vehicle duty (current
  rates)' and the ATO's 'Luxury car tax rate and thresholds'.
- Queensland: the Duties Act 2001 (Qld) ss 377, 378, 382 and 383 and Sch 4C (reprint
  current as at 22 October 2026), and Transport and Main Roads, 'Vehicle
  registration duty help' (reviewed 14 August 2026), whose 8 light vehicle
  figures the tests hold.
- Western Australia: the Duties Act 2008 (WA) ss 228 to 230 and 236 (03-z0-00,
  as at 22 October 2026), the formula read from the PDF, and RevenueWA,
  'Vehicle licence duty'.
- South Australia: the Stamp Duties Act 1923 (SA) ss 42A and 42B and Sch 2
  cl 2(1) (version 15.9.2026) and RevenueSA, 'Stamp duty on vehicles'.
- Tasmania: the Duties Act 2001 (Tas) ss 3 and 197 (in force 24 November
  2026) and the State Revenue Office, 'Rates of duty'. Section 197(3A) applies
  'Despite subsection (2)' only, so a fleet-discounted vehicle of $600 or less
  pays s 197(3)'s $20.
- ACT: the Duties Act 1999 (ACT) ss 203, 204, 207 and 208 (republication 79),
  the Duties (Motor Vehicle Duty) Determination 2025 (DI2025-152) and its 2026
  successor (DI2026-154), which commences on 1 February 2027 and revokes it,
  and the Legislation Act 2001 (ACT) s 74(2) (republication 127): an
  instrument that commences on a day commences at the beginning of it.
- Northern Territory: the Stamp Duty Act 1978 (NT) ss 4, 57, 59 and 92A and
  Sch 1 cl 6(2) (as in force at 1 July 2025, the current version) and the
  Territory Revenue Office's stamp duty page. The s 92A exemption applies to
  the issue of a certificate of registration under s 57, and s 4 defines a
  motor vehicle certificate of registration as one issued on the registration
  or the transfer of the registration of a motor vehicle, so it applies on a
  transfer too.

Only Transport and Main Roads publishes a current worked passenger example.
The ACT Revenue Office's circular DAA005.9 works its examples at the 2024
rates, which DI2025-152 replaced, so it is used only for the method: duty on
each $100 or part. Every other figure was worked by hand from the Act at each
band edge, and an oracle written in Python from the spec, not the code, gives
946 cases: each edge less 1, at, plus 1, plus 99 and plus 100, and common
values, under every choice each jurisdiction asks.

| Case | Source | Duty |
|---|---|---|
| Queensland, 1 to 4 cylinders, $35,600 and $135,600 | Transport and Main Roads | $1,068.00 and $6,780.00 |
| New South Wales, $45,000 and $45,001 | Worked by hand | $1,350.00 and $1,355.00 |
| Victoria, $80,809 and $80,810 | Worked by hand: 405 lots of $200 at $8.40 and $10.40 | $3,402.00 and $4,212.00 |
| Western Australia, $37,500 | Worked by hand: R = 4.625001875, 4.63% | $1,736.25 |
| South Australia, $300 | Worked by hand: the $5 minimum | $5.00 |
| Tasmania, $40,000 and $40,001 | Worked by hand | $1,600.00 and $1,604.00 |
| ACT, new Category B, $80,000, 31 January and 1 February 2027 | Worked by hand from each determination | $3,177.00 and $3,419.00 |
| Northern Territory, electric, $80,000 | Worked by hand: 300 lots of $100 above $50,000 | $900.00 |

## Insurance duty

This worksheet was added on 24 November 2026. The Acts were read that day:

- New South Wales: the Duties Act 1997 (NSW) could not be read (NSW
  Legislation returned a Cloudflare challenge and AustLII refused the
  request). The rates rest on Revenue NSW, 'Types of insurance': Type A 9 per
  cent, Type B 5 per cent and Type C 2.5 per cent for policies taken out on or
  before 31 December 2017 and exempt for those taken out on or after 1 January
  2018. Whether a renewal takes out a new policy is not stated, so the
  worksheet asks for the day the policy was taken out.
- Victoria: the Duties Act 2000 (Vic) ss 3(1) ('business insurance') and 179
  (authorised version 141), whose s 179(4) Table steps the business rate down
  by 1% each year to nil from 1 July 2033, and the State Revenue Office's
  'Insurance policies (current rates)' (updated 13 November 2026).
- Queensland: the Duties Act 2001 (Qld) s 362 (current as at 22 October 2026)
  and the Queensland Revenue Office's 'Insurance duty rates'.
- Western Australia: the Duties Act 2008 (WA) s 215 (03-z0-00) and
  RevenueWA, 'Self-assess and pay insurance duty'.
- South Australia: the Stamp Duties Act 1923 (SA) s 34(1)(b) (version
  15.9.2026), which charges premiums for insurance of any kind other than life
  insurance; s 34(5)(b)(ii) and (iii) show personal accident insurance is
  among them.
- Tasmania: the Duties Act 2001 (Tas) s 166 (in force 24 November 2026) and
  the State Revenue Office, 'Rates of duty' (10% from 1 October 2012).
- ACT: the Duties Act 1999 (ACT) republication 79, whose endnotes record
  chapter 8 (insurance) omitted by the Revenue Legislation Amendment Act 2017
  (No 2) A2017-11 s 4, part 2 commencing on 1 July 2017. That is the repeal;
  the endnotes do not show whether the rate reached nil earlier, and the
  worksheet claims only the repeal date.
- Northern Territory: the Stamp Duty Act 1978 (NT) Sch 1 cl 6(3) (as in force
  at 1 July 2025, the current version).

No revenue office publishes a worked general insurance example with figures,
so every figure was worked by hand from the rate, and an oracle written in
Python from the spec, not the code, gives 644 cases: each class at 23
premiums (New South Wales Type C for policies taken out on 31 December 2017
and 1 January 2018), including half-cent ties and the largest amount the crate takes, on
the first and last day of the year.

| Case | Source | Duty |
|---|---|---|
| New South Wales Type A, $1,000.00 | Revenue NSW: 9 per cent | $90.00 |
| Victoria business, $1,234.56 | s 179(4): 7%, 86.4192 | $86.42 |
| Queensland class 1, $555.55 | s 362(1)(a): 9%, 49.9995 | $50.00 |
| Queensland CTP, any premium | s 362(2) | $0.10 |
| South Australia, $0.05 | s 34(1)(b): 11%, 0.0055 | $0.01 |
| New South Wales Type C, $1,000.00, policy taken out 31 December 2017 | Revenue NSW: 2.5 per cent | $25.00 |
| ACT, $5,000.00 | ch 8 omitted from 1 July 2017 | nil |

No Act sets a rounding rule for one premium's duty: insurers pay on a return
of total premiums. The worksheet shows `duty` to the cent, as every amount in
this crate is, and gives the exact figure in `rates.exact_duty`.

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

## Payroll tax group

This worksheet was added on 24 November 2026. Each Act was read that day,
from 12:10 to 12:50 AEDT, at its official legislation site: the Payroll Tax
Act 2007 (Vic) Pt 5 and Sch 1 Pts 3 and 4 (authorised version 041, in force
from 1 May 2026); the Payroll Tax Act 1971 (Qld) ss 32, 33 and 44 to 48 and
Pt 4 (current reprint); the Pay-roll Tax Assessment Act 2002 (WA) s 17 and
Pt 4 (version 04-d0-03, current from 1 July 2019); the Payroll Tax Act 2009
(SA) Pt 5 and Sch 1 Pt 3 (version 31.3.2026); the Payroll Tax Act 2011 (ACT)
Pt 5 and Sch 1 Pts 1.3 and 1.4; and the Payroll Tax Act 2008 (Tas) and
Payroll Tax Act 2009 (NT) Pt 5. legislation.nsw.gov.au answered with a
Cloudflare challenge, so New South Wales is refused; the Tasmanian (Sch 1
cl 8 to 9B) and Northern Territory (Sch 1 cl 7) group formulas are images
the text does not carry, so both are refused. SRO Victoria's 'Grouping' and
QRO's 'Payroll tax deductions' were read the same day.

| Case | Source | Result |
|---|---|---|
| 80% x 70% = 56% groups; x 40% = 22.4% does not | Payroll Tax Act 2007 (Vic) s 77(5) | B grouped, C not |
| 40% + 25% x 60% = 55% | s 78(3) (here 40% + 30% x 50%) | grouped; 50% exactly is not |
| Ian controls A, A controls Partnership B | SRO Victoria, 'Grouping' | A, B (and C) grouped; Ian not |
| Lucy, object of a discretionary trust whose trustee holds 80% of Trading Co | SRO Victoria, 'Grouping' | Trading Co and Lucy's café grouped |
| $2,000,000, $4,000,000 and $2,000,000 of $4,000,000 in Victoria | Sch 1 cl 7A | $1,000,000; $500,000; $250,000 |
| $1,000,000 (DGE) and $400,000 in Queensland | s 33, ss 45 and 48(2); QRO | $1,285,714.29; DGE $1,000,000; B $285,714.29 |
| $800,000 and $700,000 in Western Australia | s 17(3A) | $923,076.92, with or without a DGE |
| $1,800,000 in South Australia; $2,500,000 in the ACT | SA Sch 1 cl 9(2); ACT Sch 1 s 1.9(2) | $600,000; $1,750,000 |

`tests/payroll_tax_group.rs` holds these cases.

## Transfer duty

This worksheet was added on 24 November 2026. Each source was read that day:

- Duties Act 1997 (NSW) ss 32, 32A, 32B, 32C, 33AC, 33AD, 33AF, 74, 78A and
  273 on AustLII (as at 29 June 2026; s 78A's formula is an image there), as
  legislation.nsw.gov.au served a challenge; Revenue NSW, 'How to calculate
  transfer duty' (last updated 6 October 2026: the 2026-27 rates, the premium
  threshold of $3,870,000 and the $55,537, $203,237 and $14,437 examples) and
  'First Home Buyers Assistance Scheme'; the NSW Government's table of reduced
  rates for 1 July 2026 to 30 June 2027. The s 33AF notice for 2026-27 was not
  read: the figures are Revenue NSW's, and their base amounts check against
  s 33AD.
- Duties Act 2000 (Vic), authorised version 141 (24 June 2026), ss 28, 57JA
  and 57JB; the State Revenue Office's non-principal place of residence rates
  (updated 29 September 2026) and first home buyer page (updated 20 November
  2026).
- Duties Act 2001 (Qld), current as at 22 October 2026, ss 24, 91, 92 and 92A
  and Schs 3 and 4A; the Queensland Revenue Office's transfer duty rates
  (updated 25 June 2026) and home concession rates (updated 31 July 2026).
- Duties Act 2008 (WA) Sch 2 on AustLII, with No. 18 of 2026; the Department
  of Treasury and Finance's 'Transfer duty assessment' and first home owner
  rate fact sheet (agreements from 7 May 2026).
- Stamp Duties Act 1923 (SA), version 15.9.2026, s 71DD and Sch 2; RevenueSA's
  rate of stamp duty and first home buyer relief pages.
- Duties Act 2001 (Tas) ss 29, 57C and 57G; the State Revenue Office's page on
  relief for first home buyers of established homes (not available after 30
  June 2026).
- DI2026-155 and DI2026-157 (ACT, from 1 July 2026); the ACT Revenue Office's
  Home Buyer Concession Scheme page.
- Stamp Duty Act 1978 (NT), reprint REPS013 (in force at 1 July 2025), s 88A
  and Sch 1 cl 1(2); NT.GOV.AU's home owner assistance pages.

The site's transfer duty calculator (ryanduguid.github.io, main at dcc7ecbe)
was run through its own `transferDuty` and `firstHomeDuty` on its 21 cases,
its Node tests' figures, the 44 worked figures in its Rust oracle, every band
and first home threshold edge (each edge less 1, the edge and the edge plus 1,
for the 3 buyers) and 240 seeded values: 925 rows in
`tests/data/transfer_duty_site_cases.json`. The worksheet gives the same duty,
general duty and concession on 850 rows and refuses the 1 row the page
refuses (cents). The other 74, all Victorian, differ only where the Act
differs from the page:

| Rule | Act | Rows |
| --- | --- | --- |
| Victoria's duty is chargeable to the nearest whole dollar, 50 cents down; the page shows the percentage in cents, and s 57JA's share is then of the rounded duty | Duties Act 2000 (Vic) s 28(1) | 74 moved, each by under $1 |

Of the page's own figures, its 21 cases and the oracle's 44 agree; 3 of its
Node tests' figures move ($350.02 to $350 on $25,001, and the general duty
beside the first home figures on $600,001 and $750,001).

Other amounts are worked exactly and shown to the cent, half up; the Northern
Territory's formula is the only scale with fractions of a cent. New South
Wales' first home bracket is rounded to the cent before it comes off, as
Revenue NSW's calculator does; s 78A prescribes no rounding.

| Case | Source | Worksheet |
| --- | --- | --- |
| $1,350,000 in New South Wales | Revenue NSW: $55,537 | $55,537.00 |
| $4,000,000 of residential land in New South Wales | Revenue NSW: $203,237 at the premium rate | $203,237.00; $201,287.00 for other land |
| A first home at $900,000 in New South Wales | NSW Government: $19,593.50 | $19,593.50, with $30,187.00 on $800,000 |
| $550,000 in Victoria | SRO Victoria: $28,070 | $28,070.00 |
| A first home at $700,000 in Victoria | Worked by hand from s 57JA: $100,000 / $150,000 x $37,070 | $24,713.33 |
| $850,000 in Queensland, and a first home at $730,000 and $850,000 | QRO: $31,275, $6,555 and $24,100 | The same |
| $600,000 in South Australia | RevenueSA: $26,830 | $26,830.00 |
| $800,000 in Western Australia, and a first home at $700,000 | Worked by hand from Sch 2 | $32,315.50 and $16,150.00 |
| $1,500,000 in the ACT | Worked by hand: 4.54% of the whole value | $68,100.00 |
| $500,000 in the Northern Territory | Worked by hand: 0.06571441 x 500^2 + 15 x 500 | $23,928.60 |

`tests/transfer_duty.rs` holds these cases, Victoria's rounding at $3.50,
$10.50 and $52,805.50, and the refusals.

## Quoted source passages

`transfer-duty`, `foreign-purchaser-surcharge`, `land-tax` and
`first-home-owner-grant` add `source_quotes` to each result: for the
jurisdiction given, the passage of the Act, determination or revenue office
page that states the scale, rate or grant applied, with its provision, the
document's title, its URL and `verified`, the day it was read. Their
catalogue entries hold every jurisdiction's passages, which
`data/source-quotes.json` keeps. Both are added fields:
a consumer that refuses unknown fields must allow `source_quotes`. Each
passage was read at the official site on 24 November 2026 and extracted
from the page's text, not retyped: line and table row breaks are newlines,
table cells are joined by a tab, non-breaking spaces are spaces and
zero-width characters are left out.

An Act or a determination is the law; a revenue office page is the office's
statement of it, quoted where the Act could not be read that day or the page
states the current figure. Where a page and the Act differ the command
follows the Act, as its steps say: Queensland's absentee surcharge page
gives (taxable value - $350,000) x 3%, and the Act's 3.0 cents for each $1
more than $349,999 is applied. Revenue NSW's pages stand for the NSW land
tax and grant Acts, which could not be read that day; New South Wales's and
Western Australia's duty surcharge passages are the revenue offices' because
AustLII refused the request. The Northern Territory's surcharge passages
are the Stamp Duty Act's conveyance scale, which sets no rate for a foreign
purchaser, and the NT Government's statement that the Territory levies no
land tax; no passage states in terms that it charges no foreign purchaser
duty. The ACT's first home owner grant passage states that no grant is paid
for a commencement date from 1 July 2019, which the command refuses.

`tests/source_quotes.rs` checks that every jurisdiction of the 4 commands
has a passage with a provision, an official URL and its date, that each
passage states the figure the command applies, that each result carries
its jurisdiction's passages, and that no other worksheet or command gains
them.

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

The years 2020-21 to 2024-25 were added on 23 November 2026. Every Guide to
depreciating assets from 2021 to 2026 (read that day) gives the same formulas
and says days held 'can be 366' in a leap year 'even though the denominator
remains 365'. `tests/depreciation_years.rs` checks Laura's $1,500 photocopier
with a 5-year life: in the 2021 guide, **$600.00** by diminishing value and
**$300.00** by prime cost for 2020-21 (closing **$1,200.00**); in the 2024
guide, 366 days in 2023-24, **$601.64** (ATO $602, closing $898) and
**$300.82** (ATO $301, closing $1,199). Paul's $2,000 refrigerator in the
2023 guide, used wholly privately, declines **$200.00** in 2021-22 from a
$1,600 opening value and deducts nothing.

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
the dependants and any family agreement are the operator's. A person not resident at any
time pays no levy (s 251S(1)(a)) and is outside the worksheet, and a spouse must still not be entitled to any exemption, since
s 8(3) then turns on the spouse's own exemption. `tests/properties.rs` checks
that another exemption day never raises the levy and that a full day never
leaves more than a half one.

Non-resident days, read on 24 November 2026: ITAA 1936 ss 251S(1)(a) and
251U(1)(d), (2) and (3) on the ATO's Legal Database; Medicare Levy Act 1986
s 9(1), compilation 53; and "M1 Medicare levy reduction or exemption 2026",
category 2, which lets a part-year foreign resident claim a full exemption for
the time not resident 'if either: you don't have any dependants for that
period; all your dependants are in an exemption category for that period',
counted at label V. A part-year resident gives those days as
`non_resident_days`, at most the days in the year less 1, each counted once
and not also as a full or half day. Each is a whole prescribed day, since
s 251U(3) gives a half period only under paras (a), (b), (ca), (caa) and (cb):

```text
levy after s 9 = levy after s 8 x (2 x days in year
                 - 2 x (full days + non-resident days) - half days)
                 / (2 x days in year)
```

No ATO example gives an amount for one; the cases in
`tests/medicare_exemptions.rs` are worked from the Act.

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

## Private health insurance rebate

Read on 22 November 2026: the Private Health Insurance Act 2007 as compiled at
19 September 2026 (compilation 42), Divisions 22 and 23 and s 282-18; the ITAA
1997 as compiled at 1 October 2026 (compilation 268), Subdivision 61-G and
s 67-23; the ATO pages "Income thresholds and rates for the private health
insurance rebate" (last updated 22 June 2026), its pages for 2021 to 2024
(published 14 May 2025), "Claiming the private health insurance rebate"
(QC72945) and "Your private health insurance statement".

The share of the PHII benefit is 30%, 35% or 40% of the premium for the days
on which the oldest person insured is under 65, 65 to 69 or 70 or over
(s 22-15(1)), less 10 or 20 percentage points for a tier 1 or 2 earner and
nil for tier 3 (s 22-15(2) to (4)). Each percentage is adjusted from 1 April
each year, to 3 decimal places, and applies to premiums paid in that
adjustment year (s 22-15(5A) to (5E)), so premiums paid before 1 April take
the percentages from the previous 1 April. The premium excludes lifetime
health cover loading (s 22-15(6)) and is shared equally among the adults
insured (s 22-20); the statement gives each adult's share.

| From 1 April | Under 65 (base, tier 1, tier 2) | 65 to 69 | 70 or over |
| --- | --- | --- | --- |
| 2020 | 25.059%, 16.706%, 8.352% | 29.236%, 20.883%, 12.529% | 33.413%, 25.059%, 16.706% |
| 2021 to 2024 | 24.608%, 16.405%, 8.202% | 28.710%, 20.507%, 12.303% | 32.812%, 24.608%, 16.405% |
| 2025 | 24.288%, 16.192%, 8.095% | 28.337%, 20.240%, 12.143% | 32.385%, 24.288%, 16.192% |
| 2026 | 24.118%, 16.079%, 8.038% | 28.139%, 20.098%, 12.058% | 32.158%, 24.118%, 16.079% |

| Singles threshold | 2020-21 to 2022-23 | 2023-24 | 2024-25 | 2025-26 | 2026-27 |
| --- | --- | --- | --- | --- | --- |
| Tier 1 | $90,000 | $93,000 | $97,000 | $101,000 | $105,000 |
| Tier 2 | $105,000 | $108,000 | $113,000 | $118,000 | $123,000 |
| Tier 3 | $140,000 | $144,000 | $151,000 | $158,000 | $164,000 |

The family thresholds apply to a person married on 30 June, or who on any day
in the year maintained a dependent person (Schedule 1) who is their child or
dependent sibling (s 22-30(1)); they are double, plus $1,500 for each child
dependant (Fringe Benefits Act meaning) after the first (s 22-40); income must
exceed a threshold to pass it (s 22-30). The
ATO's 2024 page dates its first table to 30 March 2024, a slip for 31 March.
For 2023-24 and earlier, the ATO lets income for surcharge purposes be reduced
by the taxed element of a super lump sum within the low rate cap; that figure
is the operator's.

The offset is the share of the PHII benefit less the person's share of the
premium reduction received, nil if that is as much or more (ITAA 1997
s 61-210); the excess the person repays is the reduction share less the
benefit share (PHI Act s 282-18(3)). Both work premium by premium, so the
worksheet does not net one line against another; a line that sums premiums
with different reductions gives only their net, not each premium's offset or
excess; as its share is rounded once, that net can differ from theirs by a cent.
No provision sets the
rounding: each line's share is rounded to the cent, which reproduces the
ATO's examples. Zoe's $39.90 needs $20.095 rounded to $20.10 before it is taken
from $60, and Narelle's $96.94 needs each line rounded. Two published figures
are not reproduced: Toby's liability is stated to the dollar ($194, from
$194.09), and Donna's offset is stated as $243.12, though $364.32 less $121 is
$243.32.

Not worked out: dependant-only policies (s 22-5(3) and (4)), s 22-25 (the operator gives the band), a trustee's
offset (s 61-205(2)), the choice to claim a spouse's offset (s 61-215) and the
reallocation of the excess (s 282-18(8) to (10)).

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

## Foreign purchaser surcharge

This worksheet was added on 24 November 2026. Each source was read that day:

- Duties Act 1997 (NSW) s 104U on AustLII (as at 29 June 2026); Revenue NSW,
  'What is surcharge purchaser duty' (9%) and 'What is surcharge land tax'
  (last updated 8 April 2026: 5% from 2025, no threshold). The Land Tax Act
  1956 (NSW) could not be read: AustLII served its download page and
  legislation.nsw.gov.au a challenge.
- Duties Act 2000 (Vic) s 28A(2), authorised version 141; SRO Victoria,
  'Calculating foreign purchaser additional duty' (Emily's $16,000) and the
  absentee individuals and corporations pages (4% from the 2024 land tax
  year; $40,000 on $1,000,000). Land Tax Act 2005 (Vic) Sch 1 Tables 1.6 and
  4.5, version 086, read on 23 November 2026: at every band from $50,000 the
  absentee owner rates less the general rates are 4% of the whole value.
- Duties Act 2001 (Qld) s 244, current as at 22 October 2026; the Queensland
  Revenue Office's AFAD and absentee land tax pages. Land Tax Act 2010 (Qld)
  Schs 2 and 3, read on 23 November 2026.
- Duties Act 2008 (WA) ss 205G, 205H and 205O on AustLII; the Department of
  Treasury and Finance's foreign transfer duty fact sheet (22 October 2026:
  Simon's $14,000) and 'About land tax'.
- Stamp Duties Act 1923 (SA) s 72, version 15.9.2026; RevenueSA's foreign
  ownership surcharge page ($7,000 on a $100,000 interest) and land tax rates
  page (no foreign owner surcharge).
- Duties Act 2001 (Tas) ss 30C and 30E; the State Revenue Office's 'Rates of
  surcharge' (8% and 1.5% from 1 April 2020). Land Tax Rating Act 2000 (Tas)
  ss 6A and 7, read on 23 November 2026.
- DI2026-155 (ACT: no foreign purchaser rate); Land Tax Act 2004 (ACT) s 17E
  and DI2026-152 s 7, read on 23 November 2026; the ACT Revenue Office's
  foreign ownership surcharge page (0.75% of the AUV a year).
- NT.GOV.AU, 'Land taxes', and the Stamp Duty Act 1978 (NT): neither
  surcharge.

The site's foreign surcharge calculator (ryanduguid.github.io, main at
dcc7ecbe) was run through its own `foreignSurcharge` on its cases, its Node
tests' and oracle's figures, a grid of every jurisdiction, use and kind of
buyer at each threshold edge and seeded values: 980 rows in
`tests/data/foreign_surcharge_site_cases.json`, each with the page's verdicts
as `foreign_purchaser` and `foreign_owner`. The worksheet gives the same 2
amounts on 871 rows. 2 rows are facts the worksheet does not take (an
exemption and a partner visa, which the page refuses). The other 107 differ
only where an Act differs from the page:

| Rule | Act | Rows |
| --- | --- | --- |
| Queensland's surcharge is 3.0c for each $1 more than $349,999, from $350,000; the page works 3% over $350,000, 3 cents less | Land Tax Act 2010 (Qld) Sch 2 Pt 2, Sch 3 Pt 2 | 56 moved |
| Tasmanian land tax is rounded to 10 cents; the page applies no rounding | Land Tax Rating Act 2000 (Tas) s 7 | 21 moved |
| A Victorian trust's absentee surcharge turns on its absentee beneficiaries' interests; the page works them from its answers | Land Tax Act 2005 (Vic) s 46IA | 30 refused |

Of the page's 23 cases, 4 are its FIRB fee, a federal fee the worksheet does
not work. Of the other 19, 15 agree, 1 moves (Queensland's foreign company,
$1,500.03 for $1,500), 1 is refused (SRO Victoria's Clara, a fixed trust) and
2 are facts the worksheet does not take.

`tests/foreign_purchaser_surcharge.rs` also works by hand Victoria's $50,000
edge, Queensland's $350,000 edge ($0.03), Tasmania's s 7 rounding ($8,000.02
down, $8,000.06 up, 5 cents down) and the ACT's fractions of a cent.

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
year's cap less the amounts counted for earlier lump sums: the low rate cap
by those of earlier income years (s 307-345(2) reduces the next year's cap),
the untaxed plan cap by each earlier lump sum's amount counted, which never
exceeds the cap left then (s 307-350(2)(b) reduces it to nil). This equals the
sections' year by year reduction and indexation (the ATO's Talise and Anh
examples). Other lump sums in the same year at or above preservation age and
under 60 are refused while any of the low rate cap is left: ss 301-20(3) and 301-105(5) and (6) share that year's
low rate cap across them, the element taxed in the fund first, whatever the
order of payment. The Act
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

Income streams (Schedule 13 Part A) were added on 23 November 2026. Sources
read that day: Schedule 13 of F2024L00664, F2025L00647, F2025L01016 (the same
text as F2025L00647) and F2026L00716, and ITAA 1997 ss 301-10, 301-25 to
301-40, 301-100, 301-110, 302-65 to 302-90, 307-125, 307-145 and 307-150 from
the ATO's legal database. Each schedule's Case A, Veronica, is the only worked
Part A example; the PAYG withholding tax table amounts it quotes ($60 and $70)
come out of the crate's Schedule 1 coefficients unchanged, which checks them
against each year's instrument, and its fortnightly adjustment
(($1,100 - $1,076) x 10%) is the weekly band formula converted back to the
fortnight. The schedules do not say whether the weekly adjustment is rounded
before it is converted; the command rounds once, at step 5, and Veronica's
result is the same either way. It converts back exactly: the 10% band is
(withheld_from x d - threshold x m) x 10% / d and the 2% band is
withheld_from x 2%, where the weekly equivalent is withheld_from x d / m, so a
quarter's $7,079 in 2026-27 gives exactly $8.50 and $9 withheld. `tests/super_income_stream.rs` holds each case.

| Example | Facts given | Withheld |
| --- | --- | --- |
| Veronica, 58, disability stream, 2024-25 | fortnightly $1,400: $300 tax free, $1,100 taxed | $70 - $165 offset; adjustment **$10** |
| Veronica, 2025-26 | As 2024-25 | **$10** |
| Veronica, 2026-27 | As 2024-25 | $60 - $165; adjustment $2.40, **$2** |

The tests also cover scale 1, each offset row, a dependant on each side of 60,
each band edge of the adjustment in 2026-27 and the upper edge in 2024-25 and
2025-26, monthly and quarterly
payments, cents ignored and the 50-cent rounding, the no-TFN rates, the
proportion at commencement and each refusal.

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

## Instant asset write-off

Added on 23 November 2026. The sources were read that day: ITAA 1997 ss 40-25,
40-82 and 328-180 and IT(TP)A 1997 ss 40-140 to 40-190, 328-180 and 328-181
from the ATO's legal database; Treasury Laws Amendment (Tax Reform No. 2) Act
2026 (No. 71 of 2026) Sch 2 items 3 and 15 and its commencement table
(Schedule 2 from 1 October 2026); the ATO's 'Instant asset write-off for
eligible businesses' (last updated 28 August 2026), tables 1 and 2 and its
examples; and 'TFE for small business entities using simplified
depreciation', example 1.

The ATO's table 1 adds 'purchased by 31 December 2020' to the $150,000 limit
for a small business. IT(TP)A s 328-180(4A) has no such condition, but an asset
acquired from 1 January to 30 June 2021 is held after the 2020 budget time, so
s 328-181(2) removes the limit and both give the same result. The table gives
no row for first use from 1 July 2021 to 30 June 2023: the law leaves
s 328-180(1)(b)'s $1,000 for an asset acquired before 7.30 pm on
6 October 2020.

| Example | Facts given | Result |
| --- | --- | --- |
| Fiona's computer, 2025-26 | **$6,800**, 80% business use, 18 November 2025 | **$5,440.00** written off |
| Fiona's printer | **$700**, wholly business | **$700.00** |
| Jack's MIG welder | **$4,500**, ready for use 12 March 2026 | **$4,500.00** |
| Daryl's ute | **$40,000**, 40% use, 28 July 2025 | Not written off; **$16,000.00** to the pool |
| Jack's excavator, 2022-23 | **$220,000**, first used 20 January 2023 | **$220,000.00** (temporary full expensing) |
| X Co's van, 2020-21 | **$35,000**, held and used 30 November 2020 | **$35,000.00** |

The ATO does not give the welder's or the excavator's acquisition date; the
tests take the first use date, and any date the ATO's account allows gives the
same result. `tests/instant_asset_write_off.rs` also covers each limit either
side of its amount, the 3 budget times, s 40-82(2A) and (5), the temporary full
expensing refusal and the input refusals.

## Horticulture shearing withholding

Schedule 2 of the Taxation Administration (Withholding Schedules) Instruments
2024 (F2024L00664), 2025 (F2025L00647), (No. 2) 2025 (F2025L01016) and 2026
(F2026L00716), read on 22 November 2026, with the ATO's Schedule 2 page
(NAT 1013, published 17 June 2026). Table A is 0.13 (resident) and 0.30
(foreign resident) with a TFN, and 0.47 and 0.45 without, in each instrument.

| Example | Facts given | Result |
| --- | --- | --- |
| Schedule 2 example, each year | Earnings **$231.50**, ignoring cents $231 | **$30**, **$108**, **$69** and **$103** (resident with and without a TFN, foreign resident with and without) |
| Half a dollar | $1,150 at 13% | **$150** ($149.50, 50 cents up) |

The tests also cover a worker without a TFN, each refusal (a working holiday
maker, a labour mobility scheme worker and a TFN-quoting worker who has not
claimed the tax-free threshold) and the 2023-24 and 2027-28 boundaries.

## Upward variation withholding

Schedule 14 of the same instruments, read on 22 November 2026, with the ATO's
Schedule 14 page (NAT 5441, published 17 June 2026). The normal amount is
`payg-withholding`'s; see [PAYG withholding](#payg-withholding).

| Example | Facts given | Result |
| --- | --- | --- |
| Schedule 14 example, 2026-27 | **$563.00** weekly, scale 2, **$50** more | **$33** + $50 = **$83** |
| Schedule 14 example, 2024 and (No. 2) 2025 instruments | The same | **$39** + $50 = **$89** |

The tests also hold the normal amount equal to `payg-withholding`'s for each
pay period, scale 4 and the tax offset, Medicare levy adjustment and 53 pay
options, in each year.

## Daily casual withholding

Schedule 1, 'Calculating withholding amounts for payments made on a daily or
casual basis', of the same instruments, and the ATO's tax table for daily and
casual workers (NAT 1024) for payments from 1 July 2026 (published 17 June
2026) and from 1 July 2024 to 30 June 2026 (last updated 17 June 2024), read
on 22 November 2026. The daily value of tax offsets is 0.38% of the amount
claimed, rounded to the dollar, in both versions.

| Example | Facts given | Result |
| --- | --- | --- |
| NAT 1024 example, 2026-27 | **$193.62** a day, **$500** of tax offsets | $193 x 5 + 0.99 = 965.99, **$127** a week, **$25** a day, less **$2**: **$23** |
| NAT 1024 example, from 1 July 2024 | The same | **$26** less **$2**: **$24** |

The tests also cover a wage below the tax-free threshold, the no TFN rates,
each refusal and the 2023-24 boundary.

## No ABN withholding

TAA 1953 Sch 1 ss 12-190 and 15-10 from the ATO's legal database; the
Taxation Administration Regulations 2017 ss 4 ('top rate'), 26 and 38 in
compilations 12 (from 1 July 2024) and 18 (from 14 November 2026); the A New
Tax System (Goods and Services Tax) Regulations 2019 s 29-80.01 ($75) in
compilations 4 and 5; the ATO's 'Payments you don't withhold from' (last
updated 11 June 2025) and 'Rental expenses' (Rental properties 2025, last
updated 29 May 2025). All read on 22 November 2026.

| Example | Facts given | Result |
| --- | --- | --- |
| Rental expenses, Derek | **$1,800** cash, no ABN | **$846** |
| Payments you don't withhold from, Ana | **$150** sign paid as 2 payments of $75 | **$35** from each |
| $75 threshold | One $75.00 supply | **nil** |

The tests also cover GST disregarded for the threshold but not for the amount,
cents in the payment and the result, and each refusal.

## Entertainer withholding

Schedule 3 of the Taxation Administration (Withholding Schedules) Instruments
2024 (F2024L00664), 2025 (F2025L00647), (No. 2) 2025 (F2025L01016) and 2026
(F2026L00716), read on 22 November 2026, with the ATO's Schedule 3 page (NAT
1023, published 17 June 2026). Tables A and B are the same in the first 3
instruments and changed on 1 July 2026.

| Example | Facts given | Result |
| --- | --- | --- |
| Sandra, 2026-27 | **$500.35** a performance, 2 in the week, 1 a day, threshold claimed | 1,000.99 x 0.1432 - 60.8377 = 82.5041, **$83**, / 2, **$42** |
| Sandra, 2024 and 2025 instruments | The same | 1,000.99 x 0.1512 - 64.3365 = 87.0132, **$87**, **$44** |
| Lookup tool example, by the formula | **$279.35** a day, 3 performances, **$500** of tax offsets | **$20** less **$3** = **$17** (2026-27); **$21** less $3 = **$18** before |

The tests also cover Table B, 2 performances in a day, the nil band, the no
TFN rates, each refusal and the 2023-24 and 2027-28 boundaries.

## Return to work withholding

Schedule 4 of the same instruments and TAA 1953 Sch 1 s 12-50, read on 22
November 2026, with the ATO's Schedule 4 page (published 17 June 2026).

| Example | Facts given | Result |
| --- | --- | --- |
| George, each instrument | **$18,000** return to work payment | 32%, **$5,760** |
| Cents with a TFN | $101.57 | 32.5024, **$33** |

The tests also cover the no TFN rates with cents ignored and each refusal.

## Annuity withholding

Schedule 6 of the same instruments and TAA 1953 Sch 1 s 12-80, read on 22
November 2026, with the ATO's Schedule 6 page (NAT 1005, published 17 June
2026). The amount on the income is `payg-withholding`'s; see
[PAYG withholding](#payg-withholding).

| Example | Facts given | Result |
| --- | --- | --- |
| Barbara, 2026-27 | **$1,000** a week, deductible amount **$5,200**, 52 instalments, threshold claimed | $1,000 - $5,200 / 52 = **$900**, **$106** |
| Barbara, 2024 and 2025 instruments | The same | **$111** |
| Kenneth, each instrument | **$850** a fortnight, **$1,000** over 10 instalments | **$750**, **$4** |

The tests also hold the amount equal to `payg-withholding`'s on the income for
each pay period, scale and option, and cover the no TFN rates on the payment
and each refusal.

## Voluntary agreement withholding

Schedule 10 of the same instruments and TAA 1953 Sch 1 s 12-55, read on 22
November 2026, with the ATO's Schedule 10 page (published 17 June 2026).

| Example | Facts given | Result |
| --- | --- | --- |
| Francesca, agreement applies her CIR | CIR **16.44%**, $1,000 excluding GST | $164.40, **$164** |
| Francesca, without that term | The same | 20%, **$200** |

The tests also cover a CIR of 20% or more, GST excluded, 50 cents rounded up
and each refusal.

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

## Downsizer contributions

ITAA 1997 s 292-102, read on 24 November 2026 in the ATO's legal database with
its history notes: Act No. 132 of 2017 Sch 2 item 4 applies it to a disposal
under a contract entered into on or after 1 July 2018; the age in (1)(a) fell
from 65 to 60 for contributions from 1 July 2022 and to 55 for contributions
from 1 January 2023 (Act No. 84 of 2022 Sch 5 item 1), so every contribution
in the years `contribution_caps` takes (2024-25 to 2026-27) needs 55. The 90
days of s 292-102(1)(g) run from the day after the change of ownership (Acts
Interpretation Act 1901 s 36(1)), so day 90 is in and day 91 is out unless the
Commissioner allowed longer. Subsection (3) limits the covered amount to the
lesser of $300,000 less the person's other downsizer contributions and the
capital proceeds less the contributions already covered from them for the
person or their spouse. The ATO's page 'Downsizer super contributions' (last
updated 20 January 2026) says a downsizer contribution does not count towards
the contribution caps.

`tests/downsizer_ttr.rs` holds these cases from that page, and
`tests/derivation.rs` checks every step of 2 more:

| Case | Worksheet | Published |
| --- | --- | --- |
| George and Jane sell for $800,000 and each contribute $300,000 | each $300,000 covered | each may contribute up to $300,000 |
| Bruce and Betty sell for $400,000; Betty contributes $300,000, then Bruce $100,000 | $300,000, then $100,000; Bruce at $100,000.01 has $0.01 not covered | a combined total of $400,000 |
| Alisha sells for $1,500,000 and offers $400,000 | $300,000 covered, $100,000 not | her limit is $300,000 |
| Robert and Wendy sell a 20% share for $100,000; Robert contributes $60,000, then Wendy $60,000 | $60,000, then $40,000 covered and $20,000 not | a combined total of $100,000 |
| Ben settles on 1 August 2024 and has an extension to 1 February 2025 | covered with the extension; 184 days without it, not covered | the Commissioner may allow a longer period |
| Rebecca is 54 and James 60 at the contribution | Rebecca not covered, James covered | James is eligible, Rebecca is not |

The other criteria (an Australian dwelling, the main residence exemption, 10
years' ownership, the choice given to the fund and no earlier downsizer
contribution from another dwelling) are the operator's statement in
`downsizer_conditions_met`.

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
rounds once, after pro-rating. The other pension schedules are outside this
worksheet.

For a transition to retirement income stream, `transition_to_retirement` adds
the 10% maximum from the definition in SISR reg 6.01(2), paras (a)(ii) and
(b)(ii): total payments in a financial year, excluding commutations and
including payment splits, of no more than 10% of the account balance on 1 July
or, in the first year, on the commencement day, until the pensioner satisfies
a condition of release with a nil cashing restriction. Read on 24 November
2026 in compilation 159 (1 July 2026) and, word for word the same, compilation
148 (1 July 2024). The definition neither pro-rates nor rounds it, and the
ATO's 'Transition to retirement income streams' page for SMSFs (last updated
2 April 2025) says the same. The command shows it down to the cent, so a
payment of the amount shown stays within 10%: $123,456.78 gives $12,345.67. `tests/downsizer_ttr.rs` checks $500,000 at age
60 (minimum $20,000, maximum $50,000) and a pension commencing on 1 January
2026 with $400,000 (minimum pro-rated to $7,930, maximum $40,000).

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

## Foreign resident withholding

`foreign_resident_withholding` was checked on 22 November 2026 against TAA 1953
Schedule 1 ss 14-200, 14-210, 14-215, 14-220 and 14-235 in compilation 227
(1 October 2026), compilations 148 (1 July 2017) and 215 (11 December 2024)
and the compilation of 1 January 2025; the Treasury Laws Amendment (Foreign
Resident Capital Gains Withholding Payments) Act 2017 (No. 57) Schedule 1 items
1 to 4 (12.5% and $750,000 for acquisitions on or after 1 July 2017); the
Treasury Laws Amendment (2024 Tax and Other Measures No. 1) Act 2024 (No. 135)
s 2 and Schedule 1 items 1 to 7 (15% and no threshold for acquisitions on or
after 1 January 2025); ITAA 1997 ss 108-7 and 109-5 (compilation 268); the
multiple vendor class variation F2016L01123 (from 1 July 2016) and LI 2026/38
(F2026L01231, which repealed it and the other 4 class variations); and the
ATO's foreign resident capital gains withholding pages (last updated 22 June
to 9 October 2026).

Choices the law leaves to the reader:

- Every document is the operator's fact: whether a clearance certificate was
  given before payment and covers the contract date, whether a notice of
  variation applies to the asset, and whether a class variation or a s 14-215
  exclusion applies.
- The ATO's notices state a varied rate and a maximum sale price; the
  worksheet applies the rate to the vendor's share and the full rate when 'the
  sale price of the asset (or its market value, if not at arm's length)'
  exceeds the maximum, as the ATO's purchaser page says (reread 23 November
  2026). That page lets a purchaser entitled to an input tax credit take it off
  only 'in determining how much withholding is required', so the maximum is
  compared with `sale_price` as contracted, GST inclusive. Neither page says
  whether a notice's maximum is GST inclusive, so a maximum at or above the
  base (the price net of the credit) but below `sale_price` is refused; the
  operator settles it from the notice.
- Where every vendor gave a document and one is a notice of variation, the
  class variations' 'not all' condition is not met; the worksheet follows the
  ATO's Zac and Jen example and treats each vendor alike.
- The law sets no rounding; each vendor's amount is rounded half up to the
  cent, except that with no document at all no class variation applies and
  the single s 14-200(3)(a) amount is rounded once; its vendor rows are
  rounded down and the cents left over go to the largest remainders, marked,
  so the rows add up to it.
- With an exercised option the first element includes the option fee (ITAA
  1997 s 134-1(1) item 1); the sale price does not.
- s 14-215(2) (compilation 215, reread 23 November 2026) adds similar
  interests only where 'the CGT asset is an interest in real property, or an
  interest in a mining, quarrying or prospecting right'. A company title
  interest is an indirect Australian real property interest (shares) under
  s 14-215(1)(a)(ii), so its market value is its own.

| Case | Source | Worksheet |
| --- | --- | --- |
| Jane and Toni, contract 16 December 2024, $1.2 million | $150,000 (12.5%); $180,000 after 1 January 2025 | $150,000.00; $180,000.00 |
| Willow and Stanley, 8 January 2025, $600,000, one certificate | $45,000 | $45,000.00 |
| Maisie and Max, $650,000, both certificates; neither | nil; $97,500 ($48,750 each) | $0.00; $97,500.00 |
| Zac and Jen, $1 million, Jen's certificate; Zac's 5% notice | $75,000; $25,000 | $75,000.00; $25,000.00 |
| Kim, 3 vendors, $1.3 million, no certificates | $195,000 ($65,000 each) | $195,000.00 |

## Rental property

`rental_property` was checked on 22 November 2026 against ITAA 1997 ss 6-5,
8-1, 26-31, 26-102, 40-27, 43-10, 43-15, 43-20, 43-25, 43-80, 43-140, 43-145,
43-210, 43-215, 43-220, 43-230, 43-235 and 43-240 on the ATO's legal database,
Act No. 126 of 2017 Schedule 2 item 13, TR 2000/2, TR 93/32 and the ATO's
Rental properties guide 2026 (current at 1 June 2026). The amendment history
in ITAA 1997 compilation 264 (Volume 12, Endnote 4) shows none of those
sections amended after Act No. 126 of 2017 except s 43-145 (No. 138 of 2024,
the build to rent manner, out of scope), so the rules hold from 2020-21.

Choices the law leaves to the reader:

- The guide's Table 1 limits a residential building to works begun from 18
  July 1985; s 43-20(1) reaches back to 22 August 1979 for buildings intended
  for non-residential use or short-term traveller accommodation, so a building
  begun in between is refused rather than given nil, as a converted one may
  qualify.
- s 43-220 is taken as one fact, covering the contract of (1)(a) and the
  qualifying investor's financing of (1)(b). It has no end date, so it is
  required for every building begun after 15 September 1987.
- The use period starts at first use by any entity after construction
  (s 43-230(2)(a)); the worksheet takes it as `first_used` and counts both
  ends. The divisor is 365 in every year, so a leap year's 366 days give
  366/365 of the rate (Example 29's note).
- Works and depreciating assets first used during the year count from that
  day when the property was available every day held; otherwise the days are
  not known and are refused.
- The decline in value given is for the days held before any private use
  reduction, and the time-based share applies to it as to the other lines.
- An asset's `first_used` is its start time, the taxpayer's own first use
  (s 40-60(2)), which can come before it was acquired, though not with
  `held_when_first_used` true. So item 13(2)(a) is decided:
  one acquired before 9 May 2017 with nothing deductible for 2016-17 is under
  s 40-27 if first used by 30 June 2017 and outside it otherwise.
- Each line is rounded to the cent. The ATO's examples round to the dollar,
  and Example 12 halves the rounded $11,475 to $5,737.50 where the worksheet
  gives $5,737.70.

| Case | Source | Worksheet |
| --- | --- | --- |
| $50,000 rent, $37,000 of costs, $3,000 of commission and advertising, 330 of 365 days, 2 equal owners | Guide Example 34 ($36,452, $13,548, $6,774 each) | $36,452.05 and $13,547.95; $6,773.97 each |
| $14,108 of interest, $170,000 of $209,000 for the rental | Example 12 ($11,475) | $11,475.41; $5,737.70 for a borrower of half |
| $1,670 of borrowing expenses from 17 July, 20-year loan, $170,000 of $209,000 | Example 20 ($259.62, $271.53, $271.53) | The same, through `borrowing-expenses-schedule` |
| $1,826 from 1 July 2024, repaid 31 March 2026, used 75% then 50% | The rental skill's card (skills PR 464) | $273.75, then $730.50 |
| $115,800 of construction expenditure, built 1991, acquired 20 July 2025 | Example 29 ($2,744) | $2,744.30 for 346 days |
| Works begun on each edge of the rate table | ss 43-20, 43-25, 43-220; Tables 1 and 2 | Nil, 4% or 2.5% as each sets |
| 4% works first used in 1986 and in December 2000 | ss 43-15(1), 43-240 | Nil; $1,610.96 of $4,000, the undeducted expenditure |
| New curtains, the previous owner's carpet, a friend's television, a dryer from home | Example 21 | The curtains only |
| Assets supplied with new apartments within 6 months; a low-value pool | Example 23; s 40-27(6) | Deductible |
| The worksheet's lines with travel of $436 | Example 33 ($20,629 with travel) | $20,193.33 and -$10,893.33; travel denied (s 26-31) |
| $90,000 of other income and that net rent in 2025-26 | The 2025-26 resident scale | $17,788.00 and $14,520.10; $3,267.90 less tax |

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

## Small business income offset

This worksheet was added on 22 November 2026. ITAA 1997 Subdivision 328-F and
s 328-110 were read that day in compilation 268 (1 October 2026), volume 7
(https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_7/document_7.html),
s 4-10 in volume 1 and s 63-10 in volume 2, and compared with compilations 210
(from 1 July 2020) and 223 (from 1 July 2021), volume 7. Subdivision 328-F
reads the same in all 3 but for the rate in s 328-360(1): 13% in compilation
210 and 16% from compilation 223. The ATO's 'Small business income tax
offset' page (last updated 7 May 2025, read on 22 November 2026) gives the
same table: 13% for 2020-21 and 16% from 2021-22, each with the $5 million
turnover and the $1,000 cap.

Choices the Act leaves to the reader:

- s 63-10(1) puts this offset and the low income and low and middle income
  tax offsets in the same item (item 20) without an order among them.
  `offset_applied` takes `offsets_applied_first` before this one, as the
  site's sole trader page does with those 2 offsets; the total tax is the
  same in any order.
- The amount is worked exactly and shown to the cent, half up, as the site
  does.

`tests/small_business_income_offset.rs` holds these cases.

| Case | Source | Worksheet |
|---|---|---|
| 2026-27, $79,000 taxable, $20,000 of business income, $14,220 liability | site sole trader vector | $576.00 |
| 2026-27, $75,000 all business, $13,020 liability | site vector | $2,083.20 capped at $1,000.00 |
| 2024-25, $160,584 taxable, $29,000 business, $40,754.08 liability | site vector | $1,177.57 capped at $1,000.00 |
| 2025-26, $120,000 all business, $26,788 liability | site vector | $1,000.00 |
| 2023-24, $32,000 taxable, $35,000 business, $2,622 liability, $700 first | site vector | $419.52 |
| 2021-22, $25,000 all business, $1,292 liability, $1,292 first | site vector | $206.72, none applied |
| 2020-21, $50,000 all business, $6,717 liability | the 13% rate | $873.21 |
| Turnovers at $4,999,999.99 and $5,000,000.00 by each rule of s 328-110 | s 328-357 | met; not met |
| Shares of $10,000 less $12,000 and $5,000 less $1,000 | s 328-360(1)(b) | $4,000.00 |

`resident-tax` gives the site's liabilities for the 2024-25 to 2026-27 vectors.

## Trading stock

This worksheet was added on 22 November 2026. ITAA 1997 ss 70-35 to 70-50
(volume 3) and ss 328-280 to 328-295 (volume 7) were read that day in
compilation 268 (1 October 2026) and compared with compilations 210
(1 July 2020) and 223 (1 July 2021): Division 70 reads the same in all 3, and
s 328-285 has no subsection (2) in compilation 210. Act No. 92 of 2020
Schedule 3 item 40(8) applies subsection (2) to income years starting on or
after 1 July 2021.

Choices the Act leaves to the reader:

- `lowest` takes the first of equal values in s 70-45(1)'s order.
- Without `estimated_closing_value`, the items' total is the estimate for the
  s 328-285(1)(b) test.

`tests/trading_stock_prepayments.rs` holds these cases.

| Case | Source | Worksheet |
|---|---|---|
| Angela, $30,000 opening, $34,000 estimate, choice made | s 328-295 example | closing $30,000, nil |
| Colin, $6,800 and $7,820 | ATO Simplified trading stock rules (27 May 2026) | choice open |
| Joel, $5,600 against $8,000 and $12,000 | same page | open; refused, and $6,400 assessable on a $12,000 stocktake |
| A medium business in 2020-21 and 2021-22 | Act No. 92 of 2020 Sch 3 item 40(8) | not open; open |
| Lowest, market selling value and obsolescence items against $10,000 | worked by hand | closing $6,600, $3,400 deductible |

## Prepayments

This worksheet was added on 22 November 2026. ITAA 1936 ss 82KZL to 82KZMF
were read that day in compilation 193 (1 October 2026) and compared with
compilations 169 (1 July 2020) and 175 (4 April 2021): 169 lacks
ss 82KZM(1A) and 82KZMA(2A), and 175 reads as 193. Act No. 92 of 2020
Schedule 3 item 40(5) applies those subsections to expenditure incurred on or
after 1 July 2020. The formulas of ss 82KZM(1), 82KZMD(2) and 82KZMF(1) were
read from the images' text.

Choices the Act leaves to the reader:

- 12 months (s 82KZM(1)(ba)) and 10 years (s 82KZL(1)(d)) are read as 12 and
  120 months under Acts Interpretation Act 1901 s 2G(2) (compilation 39): a
  period from 15 February 2026 is no longer than 12 months if it ends by
  14 February 2027.
- s 82KZM does not apply to an amount s 82KZMF apportions, as s 8-1 then no
  longer allows it in full in the year incurred (s 82KZM(1)(c)); the ATO's
  2026 guide says the same.
- Each apportioned year is rounded to the cent half up on the running total,
  so the years add up to the expenditure; the ATO's examples show whole
  dollars.

`tests/trading_stock_prepayments.rs` holds these cases.

| Case | Source | Worksheet |
|---|---|---|
| Tom Pty Ltd, $15,000 over 395 days | ATO prepaid expenses 2026 | $1,139.24 and $13,860.76 ($1,139, $13,861) |
| Noel Pty Ltd, $10,200 for 15 July 2026 to 14 July 2027 | same | nil, $9,808.77, $391.23 ($9,809, $391) |
| Jacobs Trust, $24,000 lease for 2026-27 paid 1 June 2026 | same | $24,000 in 2025-26 |
| Martin, $1,250 for 1 January 2021 to 31 January 2022 | ATO prepaid expenses 2021 | $571.34 and $678.66 ($571, $679) |
| Jasmin, $1,750 for 1 June 2026 to 31 May 2027 | ATO prepaid expenses 2026 | $1,750 in 2025-26 |
| Marion, $10,000 tax shelter fee | same | $1,671.23 and $8,328.77 ($1,671, $8,329) |
| John's $1,200 registration; Maree's $950 | same | excluded, deductible when incurred |
| A large business's $12,000 for 1 April 2026 to 31 March 2027 | worked by hand, s 82KZMD(2) | $2,991.78 and $9,008.22 |
| 15 years from 1 July 2026 | s 82KZL(1)(d) | period ends 30 June 2036, 3,653 days |

The ATO's 2026 guide gives Martin's example for 2025-26 as 182 and 215 of 396
days, $572 and $678. 1 January to 30 June 2026 has 181 days, and the 2021
guide's same example gives 181 days, $571 and $679; the worksheet follows the
Act.

The same guide gives Mike's lease from 1 December 2025 to 31 December 2026 as
397 days; the period has 396, as the 2021 guide counts the same dates a year
earlier.

## HELP indexation

This command was added on 22 November 2026. HESA Division 140 and s 137-19A
were read that day in compilation 105 (17 October 2026, C2026C00478, the latest
on the Register), with the Universities Accord (Student Support and Other
Measures) Act 2024 (No. 108, 2024, rectified version 2) and the Universities
Accord (Cutting Student Debt by 20 Per Cent) Act 2025 (No. 30, 2025), each as
made. The factors are the ATO's 'Study and training loan indexation rates'
(last updated 17 April 2026) plus one, as s 140-20 has the Commissioner
publish them.

- Act No. 108, 2024 Sch 1 Part 1 made the factor the lower of the CPI and WPI
  factors, and item 56 applies that from the financial year ending 30 June
  2023, recalculating 2022-23 and 2023-24: the ATO shows 3.2% (previously
  7.1%) and 4% (previously 4.7%). Before that the factor was the CPI factor:
  compilation 83 (C2022C00005, from 1 January 2022, read on 23 November
  2026) has s 140-10(1)(b) divide the 4 quarters' index numbers to the March
  quarter by the 4 to the March quarter a year earlier, and s 198-20 makes the
  index number the All Groups CPI. Items 18 and 19 move both
  factors to the quarters ending 31 December from 1 June 2025.
- s 140-26 (Act No. 30, 2025 Sch 2 item 37), for 2024-25 only, takes 20% off
  the step 6 amount before indexation and off the debts incurred from 1 July
  to 31 December 2024 less their repayments; s 137-19A takes 20% off each
  debt incurred from 1 January to 1 June 2025, which joins the 1 June 2026
  calculation.
- The map named 'ss 140-5 to 140-25'; there is no s 140-15 in compilation
  105, and ss 140-26, 140-30 and 137-19A also apply.

Choices the Act leaves to the reader:

- A voluntary repayment counts from the date the operator gives. The ATO
  says a payment can take up to 4 business days to be received and that
  indexation applies if it is not received and processed before 1 June; the
  result warns of this.
- The operator places each voluntary repayment: against the debts indexed
  (`voluntary_repayments`, step 3) or against the debts of the first half of
  the year (`current_repayments`, s 140-25).
- Step 2A and s 140-25(1A) cut each group's total to whole dollars, and the
  Act groups debts by course and provider, which the command cannot see. So
  each debt in a list of more than one names its `group`; a list's one debt
  may omit it.
- `compulsory_repayments` is the step 4 total as s 140-5(1) defines it
  (rechecked in compilation 105 on 23 November 2026): amounts assessed in the
  period, except any from a return given before it, plus any assessed after
  it from a return given before its end, each dated by its notice's date of
  issue (s 140-5(2)). The operator works it out from the notices.

`tests/help_indexation.rs` holds these cases.

| Case | Source | Command |
|---|---|---|
| 2024-25, $90,000 and an $11,000 debt in November 2024 | ATO Example 1 (Maylin) | $18,000 off, indexation $2,304, $83,104 (with the May 2025 debt, $90,304) |
| 2024-25, $10,000 | ATO Example 2 (Asha, a VSL debt with the same arithmetic) | $2,000 off, indexation $256, $8,256 |
| 2019-20, $15,000, $1,500 debt, $525 voluntary, $3,000 compulsory | HESA s 140-5 example, 7 years later | $12,975 before indexation, $13,208 |
| $10,000 for each year | ATO rates | $10,180; $10,060; $10,390; $10,320; $10,400; $8,256; $10,280 |
| 2025-26, $20,000, $5,000 proposed | steps 3 and s 140-25 | $15,420 before 1 June, $15,560 after, $140 saved |
| 2024-25, $10,000, $1,000 proposed | s 140-26(1) | $7,430 before 1 June, $7,256 after |
| Debts on 1 January, 1 June and 2 June 2025 | s 137-19A | $800, $800, $1,000 |
| 2025-26, Maylin's $83,104 carried with the $9,000 May 2025 debt | ss 140-5(1) step 1 and 137-19A | $7,200 added, $90,304 indexed, $92,832 |
| 2024-25, a $1,000 debt in September 2024 less $200 repaid | s 140-26(2) | $640 |
| 2 debts of $0.60 in one group, then in 2 | s 140-25(1A) | $1, then nil; without groups, refused |

## Defined benefit income cap

This command was added on 24 November 2026. ITAA 1997 Subdivision 303-A and
ss 301-100 and 302-85 were read that day in the ATO's legal database, and
against compilation 170 (1 July 2017) for 2017-18. LCR 2017/1 (28 April 2017)
works the same text in its Examples 2 to 6, and the tests reproduce each. The
general transfer balance cap for each year is the ATO's 'Transfer balance cap'
page: $1.6 million to 2020-21, $1.7 million for 2021-22 and 2022-23, $1.9
million for 2023-24 and 2024-25, $2 million for 2025-26 and $2.1 million for
2026-27. 2027-28 is refused: its cap is not published.

- s 303-4(2) counts the days remaining in the year after the first day, then
  adds 1; LCR 2017/1 Example 2 gives $49,590 for 1 January 2018.
- s 303-4(3) reduces the cap by defined benefit income that gets no
  concessional treatment, but not below nil (LCR 2017/1 paragraph 61).
- The amounts are to the cent; the cap is in whole dollars, rounded up.

## Foreign pension UPP

This command was added on 24 November 2026. ITAA 1936 s 27H was read that day
in the ATO's legal database and in the 11 May 2007 compilation, which holds the
same text for 2007-08. The Income Tax Assessment (1936 Act) Regulations 2025
s 7 prescribes the Life Tables for the life expectation factor. The shortcuts
are the ATO's individual supplementary tax return instructions 2026, D11.

Choices the Act leaves to the reader:

- The ATO publishes no calculator and no numeric example for a foreign
  pension: it determines the amount by private ruling (NAT 16543). The command
  works s 27H(2) from the operator's figures and warns of this.
- The command does not hold the Life Tables or translate currency: the factor
  and the Australian dollar amounts are inputs.
- s 27H(2) gives an amount for a year of income with no rule for part of a
  year; part years are refused. A commuted annuity (s 27H(3A)) and any
  relevant number other than a term of years or one life are the
  Commissioner's and are refused.
- The D11 shortcuts were found for 2025-26 only; earlier years use `formula`.
- The amount is kept to the cent; the Act gives no rounding rule.

## Scholarship exemption

This command was added on 24 November 2026. ITAA 1997 s 51-10 item 2.1A and
s 51-35 were read that day in compilation 268 and in compilation 170 (1 July
2017): the text is the same. The ATO's 'Scholarship payments' page gives 6
examples, and the tests reproduce each: Jenny (exempt), Henry (an employment
condition), Kirra (part time), AAA University's vet students (work experience
in the course, exempt), Megan (exempt) and Diego (not principally educational).

- The Commonwealth payments s 51-35(a), (b) and (f) name fall outside item
  2.1A and may be exempt under item 2.1B or Subdivisions 52-E and 52-F; they
  are refused, as the ATO's tool leaves them out. Commonwealth funding alone
  is not one of them: a scholarship the Commonwealth funds or pays takes
  `commonwealth_payment: none` and is tested like any other.
- Friendly society scholarship plans are refused: the ATO says some of their
  benefits are taxable, and its tool does not test them.

## Super crystallisation

This command was added on 24 November 2026. ITTPA 1997 s 307-225 was read that
day in the ATO's legal database, and former ITAA 1936 ss 27A and 27AA in the 19
February 2007 compilation, the law just before 1 July 2007. The pre-July 83
component is s 27AA(1)(d): the lesser of (ETP - C - IC - NQ - EC - CGT) x
pre-July 83 days over all days, and that amount less undeducted
contributions.

- The ATO's 'Calculating components of a super benefit' example 9 counts both
  the first and last days (2,191 days from 1 July 1977 to 30 June 1983); the
  command counts the same way to 30 June 2007.
- NQ (an immediate annuity's non-qualifying component) and EC (the excessive
  component under the reasonable benefit limits) are taken as nil, as the
  ATO's crystallisation calculator takes neither.
- The ATO publishes no numeric crystallisation example; the tests use
  hand-worked cases of the formula. The page rounds $37,037.34 to $37,038
  and says the law gives no rounding rule; the command keeps cents.
- A defined benefit interest's value comes from the regulations' method; it is
  an input.

## Endorsement review

This command was added on 24 November 2026. The worksheets are the ATO's NAT
74110-02.2021, NAT 11685-02.2021 and NAT 11686-02.2021. Each question is tied
to the provision it tests as the ATO's legal database gave it that day: ITAA
1997 ss 30-15 item 1, 30-125, 30-130, 30-228, 50-5, 50-47, 50-50 and 50-110,
and TAA 1953 Sch 1 ss 426-45 and 426-55.

- The worksheet tells an entity without a winding up clause to amend its
  documents. s 30-125(1)(c) makes the clause a condition of entitlement, so a
  'No' gives `not_entitled`, with the notice s 426-45(1) requires and the
  amendment. An entity established by an Act that does not provide for its
  winding up meets the condition without a clause and answers yes.
- A cancelled ABN ends the endorsement without notice from the entity (s
  426-45(3)); every other 'No' that ends entitlement needs written notice.
- Act No. 58 of 2026 repealed s 30-15 item 1 condition (b), the $2 minimum
  gift, a donor-side rule the worksheets do not ask about.
- Only 2026-27 is supported: the provisions were read as at 24 November 2026.

## Non-commercial losses

This worksheet was added on 22 November 2026. ITAA 1997 Division 35 was read
that day in compilation 268 (1 October 2026), volume 1
(https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_1/document_1.html),
and compared with compilations 210 (from 1 July 2020) and 223 (from 1 July
2021): ss 35-10 to 35-55 read the same in all 3 but for defined-term
asterisks. The ATO's 'Four tests', 'The income requirement', 'Excepted
business activities' and 'How to defer your losses' pages (each last updated
7 April 2025) were read the same day.

Choices the Act leaves to the reader:

- s 35-10(2E)(a) disregards the s 35-10(2) excess in taxable income. The
  worksheet adds the business losses back to the taxable income or loss given
  and takes out the FHSS amount, using nil for a negative result, as the
  ATO's income requirement page does.
- The profits test counts a year when its deductions, apart from
  s 35-10(2) and (2C), are less than its assessable income; equal is not a
  profit.
- For a part year, the assessable income test is met by the year's
  assessable income or a reasonable full-year estimate of it reaching
  $20,000 (s 35-30(a) or (b)), and `assessable_income_measured` is the
  larger.
- The amount deferred is the excess less net exempt income (s 35-15); when
  the rule does not apply, the whole excess is deductible this year:
  s 35-15(2) reduces only an amount s 35-10(2)(b) applies to for the year,
  so that year's net exempt income is left to Division 36.

`tests/non_commercial_losses.rs` holds these cases.

| Case | Source | Worksheet |
|---|---|---|
| Slaide: $16,000 for part of the year, estimated at $22,000 | ATO Four tests | assessable income test met |
| Nguyen: a $7,500 loss, then $17,500, $15,000, -$4,000 and $16,000 | ATO Four tests | 3 profit years: met |
| Joe: $155,000 taxable income and a $10,000 net investment loss | ATO income requirement | $165,000: met |
| Fred: $49,000 taxable income and $1,000 of reportable super | ATO income requirement | $50,000: met |
| Alex: $200,000 with $20,000 FHSS and a $6,000 business loss | ATO income requirement | $186,000: met |
| Jennifer: $9,000 income, $10,000 deductions, $2,000 brought forward | s 35-10 example | $3,000 deferred |
| Michael: $15,000 income, $20,000 deductions, $3,000 net exempt income | ATO How to defer | $5,000 excess, $2,000 deferred |
| $19,999.99 and $20,000; $499,999.99 and $500,000; $99,999.99 and $100,000 | ss 35-30, 35-40, 35-45 | not met; met |
| $249,999.99 and $250,000; other income $39,999.99 and $40,000 | ss 35-10(2E), (4) | met; not met |

## Minors income

This worksheet was added on 22 November 2026. ITAA 1936 Division 6AA
(ss 102AA to 102AGA) was read that day in compilation 193 (1 October 2026),
volume 2
(https://www.legislation.gov.au/C1936A00027/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_2/document_2.html),
and compared with compilation 169 (from 1 July 2020), which reads word for
word the same; its amendment history lists nothing after Act No 64 of 2020.
The Income Tax Rates Act 1986 ss 3, 13 and 15 and Schedules 7 and 11 were
read in compilation 66 (1 July 2026)
(https://www.legislation.gov.au/C2004A03348/2026-07-01/2026-07-01/text/original/epub/OEBPS/document_1/document_1.html)
against compilation 55 (1 July 2020): the same but for editorial changes, and
no amendment to ss 13 or 15 or Schedule 11 after 2013. The phase-out limits
are the formulas of s 13(10) and s 15(8) (images in the compilation, read
from their text alternatives): $416 x 66% / (66% - 45%) = $1,307 and
$416 x (66% - 30%) / (66% - 45%) = $713, each rounded down. ITAA 1997
s 61-115 and s 995-1 'apportionable deductions' were read in compilation
268 (1 October 2026). The ATO's 'Tax rates if you're under 18 years old'
(last updated 1 June 2026) gives the same bands for 2020-21 to 2025-26; it
has no 2026-27 table yet, so 2026-27 rests on the Act alone, whose rates for
it are the same at every point the worksheet uses but the 15% first band of
the other income.

Choices the Act leaves to the reader:

- Eligible taxable income is worked exactly, not rounded to the dollar, and
  each amount is shown to the cent, half up.
- Schedule 11's 45% applies to eligible taxable income over $416 (any amount
  for a foreign resident), and s 13(2) or s 15(2) only caps it: the tax on it
  is the lesser of 45% and the greater amount those subsections give.
- Where no special rate applies, `tax_on_eligible_income` is the top slice of
  the ordinary tax, so the 2 parts add up to the ordinary tax.
- The second resident personal tax rate is item 2 of the year's Schedule 7
  Part I table (30%), not the first band above the tax-free threshold.

`tests/minors_income.rs` holds these cases.

| Case | Source | Worksheet |
|---|---|---|
| Resident, $416, $417, $1,307, $1,308 and $2,000 of unearned income | ATO table 1 | $0.00, $0.66, $588.06, $588.60, $900.00 |
| Resident, $1,000 of unearned income | TR 92/12 Example 5A (s 13(2)) | $385.44 |
| Foreign resident, $416, $417, $500, $713 and $714, each year | ATO tables 3 and 4; s 15(2) | $124.80, $125.46, $180.24, $320.82, $321.30 |
| $28,000 of wages and $2,000 of dividends, 2025-26 | ATO table 1 row 3 | $1,568.00 + $900.00 |
| $30,000 of wages and $2,000 of interest by year | Schedule 7 tables | other part $1,888.00, $1,888.00, $1,770.00 |
| $45,000 of wages and $500 of dividends | ATO table 1 note (top slice) | $150.00 on the eligible part |
| $2,000 of dividends with a $100 gift, with and without $2,000 of wages | s 102AD(c) | $100.00 and $50.00 apportioned |
| A $5,000 trust share with a $3,000 Division 6AA part | s 102AE(4), s 102AG | $1,260.00 |
| Full-time 3 months, intending full-time study next year | s 102AC(8)(b) | prescribed |

## PCG 2021/4

This worksheet was added on 24 November 2026. Each source was read that day,
from 00:05 to 00:15 AEDT:

- PCG 2021/4, Allocation of professional firm profits (updated guideline of
  19 June 2024) on the ATO legal database
  (https://www.ato.gov.au/law/view/document?docid=COG/PCG20214/NAT/ATO/00001):
  paragraphs 10 to 12 (the gateways), 17 (from 1 July 2022), 76 (Table 1),
  78 to 80 (Table 2), 81 (100% returned), 82 to 86 (factors 1 and 2 and the
  larger of paragraph 85(a) and (b)), 100 and 101 (factor 3), 122 (2020-21
  rates), footnotes 6 (no levy) and 7 (26% for a corporate taxpayer),
  examples 4 to 6 and case studies 1 to 12;
- the ATO's 'Tax rates – Australian resident' (last updated 13 August 2026),
  the resident scales for 2020-21 and 2022-23 to 2026-27, which
  `PRIOR_RESIDENT_TAX_SCALES` and `RESIDENT_TAX_SCALES` already hold;
- the ATO's company tax rates for 2020-21 (26% for a base rate entity, last
  updated 30 May 2024), 2022-23 to 2025-26 (25%) and the Income Tax Rates Act
  1986 s 23(2), Compilation No. 66 (1 July 2026), for 2026-27 (25%, otherwise
  30%), which `deferred_tax` already holds.

The site's PCG 2021/4 pages (ryanduguid.github.io, main at dcc7ecbe,
`assets/pcg-2021-4.mjs`) were run on their 16 cases, the 96 vectors of
their tests (case studies 1 to 12, examples 4 to 6, every Table 1 boundary
and Table 2 edge, rounding and refusals) and a seeded sweep of 160
arrangements and 160 allocations over every year and kind, 432 rows in
`tests/data/pcg_2021_4_site_cases.json`. The worksheet gives the same
percentages, scores, zone, firm income, tax and each recipient's tax and limb
on all 407 rows the site works out, and refuses the 25 it refuses.

The guideline prints Bernie's tax (case study 4) as $116,916 and Frank's
(case study 5) as $105,666; the 2020-21 scale gives $116,917 and $105,667,
as the site does, with the same rate and scores.

| Case | Source | Result |
| --- | --- | --- |
| Case study 1: Nicolas $330,000, his company $270,000 | Table 4 | $119,167 and $70,200; 55.00%, 31.56%; scores 4 and 3; green at 7 |
| Example 4: 6 recipients of $1,100,000 | Paragraphs 87 to 93 | Associate 2 $33,850 under 85(b); 33.31%; green at 7 |
| Example 5: $600,000 less $40,000, with $50,000 other income | Paragraph 94 | $238,450 under 85(b) |
| Case study 7: $360,000 against a $350,000 benchmark | Table 10 | 102.86%, factor 3 score 3; green at 10 on all 3 |
| Exactly 50% returned and 30% tax; a cent less of each | Table 1 | Scores 4 and 3, green; 5 and 4, red at 9 |

## First home owner grant

This worksheet was added on 23 November 2026. That day these were read:

- Victoria: First Home Owner Grant and Home Buyer Schemes Act 2000,
  authorised version 053 (amendments as at 26 November 2025), ss 13(1)(ab),
  13(1A), 13(7) and 18(1A); the State Revenue Office's current rates and
  'Understanding the First Home Owner Grant' pages.
- Queensland: First Home Owner Grant and Other Home Owner Grants Act 2000,
  current as at 22 October 2026, ss 5(1), 5(6) to (8), 15 and 20(1); the
  Queensland Revenue Office's eligibility page ($30,000 for contracts from
  20 November 2023).
- Western Australia: First Home Owner Grant Act 2000, as at 22 October 2026,
  ss 14AC, 14AD and 19(2CA); the Department of Treasury and Finance fact sheet
  as at 28 July 2026 (the $800,000 cap south of the 26th parallel from 7 May
  2026, which s 14AD lets regulations prescribe; the regulations were not
  read).
- South Australia: First Home and Housing Construction Grants Act 2000,
  version 4.12.2025, ss 7(1a) and 18(1); RevenueSA's grant details, eligible
  properties and residence requirement pages.
- Tasmania: First Home Owner Grant Act 2000, authorised version as at
  9 October 2026, ss 18(1), 18(2)(al) and (am), 18L and 18M; the State
  Revenue Office's eligibility page.
- Northern Territory: First Home Owner Grant Act 2000, as in force at 1 July
  2026, ss 12(2), 13(1)(a), 13AA, 18 and 56; the NT Government's HomeGrown
  Territory Grant page.
- ACT: First Home Owner Grant Act 2000, republication 37 (16 November 2025),
  s 18; the ACT Revenue Office's page (no grant from 1 July 2019; the
  Minister's determination was not read).
- New South Wales: Revenue NSW's 'First Home Owner (New Homes) Grant' page.
  The First Home Owner Grant and Shared Equity Act 2000 returned HTTP 403 to
  every route that day, so its lesser-of rule, if any, is not applied and a
  value below $10,000 is refused.

The worksheet follows ryanduguid.github.io's first home owner grant page
(`assets/first-home-owner-grant.mjs`, rules read 22 November 2026) and agrees
with it on all 26 of the page's cases
(`tests/data/first_home_owner_grant_page_cases.json`). Where an Act read
differs, the Act is followed:

- a Northern Territory established home under a contract from 1 July to
  30 September 2025 gets $10,000 (s 13(1)(a)(iii), s 18(3D)(b)); the page
  pays nil;
- a Victorian, Western Australian or Northern Territory home bought for less
  than the grant gets the price (Vic s 18(1A), WA s 19(2CA), NT s 18(1));
  the page refuses;
- a Victorian home built over $750,000 in total is refused without
  `contract_price`, as the cap is on the consideration for the building work
  (s 13(1A) and (7)(b)); the page pays nil.

A home built's optional `contract_price` (read again on 24 November 2026)
is that consideration: 'the total consideration payable for the building
work' (Qld schedule 'consideration' (b), WA s 14AB(b), Vic, SA and Tas
s 13(7)(b), ACT s 13(6)(b), NT s 13(8)(b)). Each Act read pays the lesser of
the consideration and the amount (Qld s 20(1) and s 25EB, Vic s 18(1A),
WA s 19(2CA), SA, Tas, NT and ACT s 18(1); in the NT 'the amount of
consideration actually paid'), so below the amount the grant is the contract
price; the ACT still pays nil. In Victoria the contract price is the value
the $750,000 cap tests. A New South Wales contract price below $10,000 is
refused, as that Act could not be read. Owner builders, whose consideration
is the actual costs less their own labour (s 13(7)(c) and its equivalents),
stay outside the command.

The value tested against the cap is the one each source names, kept apart
from the consideration that limits the grant. For a home bought that is
`total_value`, the price. For a home built every Act read on 24 November
2026 makes it the building work alone (Qld schedule 'consideration' (b),
WA s 14AB(b), Vic, SA and Tas s 13(7)(b), ACT s 13(6)(b), NT s 13(8)(b);
the New South Wales Act could not be read), not the land and contract total: the command pays the lesser of
`contract_price` and the amount, or the amount when `building_contract`
states the contract is at least the grant, refuses with neither, and never
limits a built home's grant by `total_value`. In Queensland
and Western Australia the optional `cap_value` carries the value the Act
tests where it differs: for a home bought, the unencumbered value of the home
and land at the contract date, tested with the price as the greater of the 2
(Qld s 5(6) and (7), WA s 14AC(2)); for a home built, the building contract
plus the land's value, tested in place of `total_value` (Qld s 5(8), WA
s 14AC(3)). The grant stays the lesser of the consideration and the amount
(Qld s 20(1), WA s 19(2CA)): a Queensland home bought for $20,000 and valued
at $100,000 gets $20,000, and a Western Australian one bought for $5,000 and
valued at $100,000 gets $5,000. Each residence requirement is stated
with its principal place of residence condition, period and starting event.
The page's transfer duty rows are not part of the worksheet.
`tests/first_home_owner_grant.rs` runs the page's cases and every
jurisdiction, home and edge date at each amount and cap against an oracle
typed from the rules above.

| Case | Source | Worksheet |
|---|---|---|
| NSW bought at $600,000 and $600,001 | Revenue NSW | $10,000; nil |
| Qld bought at $749,999, $750,000 and $20,000 | Qld Act ss 5(6), 20(1) | $30,000; nil; $20,000 |
| WA south at $780,000 on 6 and 7 May 2026 | DTF fact sheet | nil; $10,000 |
| SA bought for $10,000 | RevenueSA's example | $10,000 |
| Tas built on 30 June and 1 July 2026 | Tas Act ss 18L, 18M | $30,000; $20,000 |
| NT established on 30 September and 1 October 2025 | NT Act ss 13(1)(a), 18(3D) | $10,000; nil |
| Qld built, contract $29,999, $30,000 and $1 | Qld Act s 20(1), schedule 'consideration' (b) | $29,999; $30,000; $1 |
| Vic built, $900,000 total, contract $750,000 and $750,001 | Vic Act ss 13(1A), 13(7)(b), 18(1A) | $10,000; nil |
| NSW built, contract $9,999 | Revenue NSW (Act unread) | refused |

## PSI test

This worksheet was added on 22 November 2026. ITAA 1997 Part 2-42 was read
that day in compilation 268 (1 October 2026), volume 3
(https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_3/document_3.html),
and compared with compilations 210 (from 1 July 2020) and 223 (from 1 July
2021): ss 84-5, 85-30, 85-35 and 87-15 to 87-40 read the same in all 3.
TR 2022/3 was read the same day for examples 20, 29, 30 and 40.

The worksheet follows ryanduguid.github.io's PSI page
(`assets/psi-determination.mjs`, checked 17 November 2026) step for step, so
the two agree on every case the page's tests hold. Its choices:

- each payer is entered with its associates as one row, or as one row for
  each contract when its contracts differ for the results test, the rows
  sharing a `source` label (s 87-15(3));
- a religious practitioner's payment is outside Division 85 under
  s 85-35(2), which the site's page folds into s 85-35(1);
- the employment test's 20% is measured against the PSI entered, as
  TR 2022/3 paragraph 118 does with the market value of the principal work;
- the other 3 tests are worked out even at 80% from one source, as the
  Commissioner weighs them for a determination (s 87-15(1), note 2).

`tests/psi_test.rs` runs the page's 19 cases.

| Case | Source | Worksheet |
|---|---|---|
| Joe: $18,000 for a result, tools and defects | TR 2022/3 example 20 | s 87-18(1), 100.00% |
| Ian's $26,000 of $140,000 from 2 payers | TR 2022/3 example 30 | 18.57%, not a PSB |
| Tom's $36,000 of $156,000 from one payer | TR 2022/3 example 29 | 23.07%, but s 87-15(3) |
| Chris: 70% commission | TR 2022/3 example 40 | the principal is one payer |
| 75% and 74.99% for a result; 80% and 79.99% from one payer; 20% by others | ss 87-18, 87-15(3), 87-25 | met; not met |

## PAYG instalment

This worksheet was added on 22 November 2026. Schedule 1 to the TAA 1953,
Division 45, was read that day in compilation 227 (C2026C00468, from
1 October 2026), volume 2
(https://www.legislation.gov.au/C1953A00001/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_2/document_2.html),
and compared with compilation 219 (C2025C00404, from 1 July 2025): from
s 45-110 to the start of Subdivision 45-N the only change is to the offsets
left out of the Commissioner's notional tax, which this worksheet does not
work out. The ATO's 'How to vary your PAYG instalments' (last updated 23 June
2026), 'PAYG instalments: how to complete your activity statement' (last
updated 9 January 2023) and 'Complete your BAS' pages were read the same day.
On 23 November 2026 ss 45-110, 45-205 and 45-215 in compilation 227 and both
PAYG instalment pages (last updated as before) were read again: no step of the
Act or the pages rounds an amount, and s 45-215(1)(b) compares the varied rate
with 'the instalment rate you used to work out the amount of your instalment
for the previous instalment quarter (if any)'. Under ss 45-110(2)(a) and
45-15 that rate can differ from T2 when the Commissioner gives a new rate
after that quarter, so the worksheet takes it as `previous_rate`.

The worksheet follows ryanduguid.github.io's PAYG instalments page
(`paygInstalment` in `assets/business-calculators.mjs`, checked 15 November
2026). Its 11 cases and 400 seeded random inputs were run through the
command on 22 November 2026: each case agrees, and of the random inputs all
agree or are refused by both except one, a credit claimed before the second
instalment. The page adds such a credit back; s 45-410(3) does not, and no
credit can exist then (s 45-410(2) never gives a negative first amount, and
s 45-215(1)(b) needs an earlier quarter), so the worksheet refuses it. Its
other choices:

- no step rounds; only the instalment and credit drop their cents ('round
  down to whole dollars' on Complete your BAS), so a sub-cent remainder never
  rounds up into a whole dollar (25% of $399.99 is $99.9975, an instalment of
  $99), and other amounts show to the cent toward nil. The page rounds each
  step half up to the cent first;
- a varied rate's credit compares it with `previous_rate`, the rate used for
  the previous quarter (s 45-215(1)(b)), where the page compares it with T2;
- a varied rate keeps 2 decimal places and drops the rest, as the ATO's
  example 2 gives 12.27% for $10,125 / $82,480; the Act prescribes no
  rounding;
- each method takes only its own facts, where the page ignores the others;
- a varied amount adds back each s 45-420 credit the payer was entitled to,
  claimed or not (s 45-410(4) and (5), step 2), so it takes
  `credits_entitled`; the page and the ATO's 'How to vary' page speak of
  credits claimed, which agree when every credit was claimed;
- nil estimated instalment income varies the rate to nil whatever the
  estimated tax ('How to vary', step 3), where the page refuses an estimated
  tax above it.

| Case | Source | Worksheet |
|---|---|---|
| Sophia: $20,100 at 1.7% | Activity statement page | $341.70, 341 at T11 and 5A |
| Cari: $14,000, third quarter after $10,000 | How to vary, example 1 | $500, then $3,500 |
| Harmander: $10,125 / $82,480 | How to vary, example 2 | 12.27% |
| Bill: 10% then 5% on $10,000 | Activity statement page | $500 at 5B |

On 23 November 2026 the page's current `paygInstalment` (main at 1aa0a6cb),
its 12 cases and 400 seeded random inputs were run again, with T2 given as
`previous_rate`: 40 are refused by both, and each of the other 372 equals an
independent reading of the steps, unrounded for the worksheet and rounded
half up to the cent at each step for the page. They differ in the instalment
or credit on 14, each by $1 where the page's rounding carries a sub-cent
remainder into a whole dollar.

## SMSF tax

This worksheet was added on 22 November 2026. That day ITAA 1997 ss 295-10,
295-160, 295-385, 295-387, 295-390, 295-545, 295-550, 295-605 and 295-610
were read on the ATO legal database with their history notes; Division 295 is
in volume 6 of compilation 268 (1 October 2026)
(https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_6/document_6.html).
Since 2015 s 295-390 was last amended by No 81 of 2016, s 295-385 by No 10 of
2022 (the trustee's choice in subsections (8) to (10)), s 295-387 by No 111
of 2021, and ss 295-545 and 295-550 by Act No. 52 of 2024, whose Schedule 7
item 5 applies s 295-545(2A) to income derived in 2018-19 and later years.
The Income Tax Rates Act 1986, compilation 66 (1 July 2026), gives 15% and
45% in s 26(1) (last amended 2007) and the no-TFN rate in s 29(2) (last
amended 2018): Schedule 7 Part I's 45% top rate, plus 2%, less 15%, so 32%;
the ATO's 'Tax rates - Australian residents' (last updated 13 August 2026)
gives 45% as the top rate for each year from 2020-21. The Superannuation (Self
Managed Superannuation Funds) Supervisory Levy Imposition Regulations 2018
s 6 (in force, unamended) set $259 for 2018-19 and each later year, under the
$300 cap of s 6 of the 1991 Imposition Act. The ATO's Instructions to complete
your SMSF annual return 2026 (Sections B and D and Appendix 1, last updated
30 May 2026) and the 2021 return's levy questions (last updated 6 August 2024)
give the same labels and table 9.

Choices the Act leaves to the reader:

- Exempt current pension income from the proportion is rounded down to the
  whole dollar, as Section B takes no cents; this never understates tax.
- A loss gives nil taxable income; the loss and its carry forward (Division
  36) are not worked out beyond the amount.
- Deductions attributable to contributions above the contributions are
  refused rather than allowed to lift the non-arm's length component above
  taxable income.

`tests/smsf_tax.rs` holds these cases.

| Case | Source | Worksheet |
|---|---|---|
| $234,200 unsegregated, 50% proportion, $62,000 franking credits | SAR 2026 Appendix 1 example 1 | ECPI $117,100; T1 $17,565.00; I $44,435.00; S -$44,176.00 |
| $200,000 segregated, $34,200 other, $62,000 franking credits | SAR 2026 Appendix 1 example 2 | T1 $5,130.00; I $56,870.00; S -$56,611.00 |
| $14,500 taxable income | SAR 2026 Section D, SMSF B | T1 $2,175.00 |
| $10,000 contributions, $15,000 net NALI, $6,000 general expense NALI, $5,000 deductions ($1,000 on contributions) | SAR 2026 Section D, SMSF BB | NALC $11,000; T1 $6,300.00 |
| Gross tax $2,175 with C $675, D $675, E $500 or $2,000 | SAR 2026 T2, T3 and T5 examples | T2 $1,500; T3 $1,500; T5 $1,000 or I $500 |
| Gross tax $2,000 ($13,312 taxable, $10 no-TFN) with C $2,500 | SAR 2026 T2 example 1 | T2 nil; $500 lost |
| $10,000 of no-TFN contributions, 2024-25 | SAR 2026 E2 example | J $3,200.00 |
| $3,200 no-TFN tax offset, 2025-26 | SAR 2026 E2 example | I $3,200.00; S -$2,941.00 |
| $200,000 segregated, $34,200 other at 40% | both methods together | ECPI $213,680; T1 $3,078.00 |
| Existing, new, wound up, new and wound up, each year | SAR 2026 table 9 | levy $259, $518, nil, $259 |
| $1,001 at 33.33% | rounding choice | ECPI $333 |

## ESS discount

This worksheet was added on 22 November 2026. ITAA 1997 Division 83A was read
that day in compilation 268 (1 October 2026), volume 3
(https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_3/document_3.html),
and compared with compilations 234 (1 July 2022), which reads the same, and
210 (1 July 2020), which adds only the cessation of employment taxing point
(former ss 83A-115(5) and 83A-120(5)). Act No. 8, 2022 Sch 10 item 9 applies
its repeal to taxing points from 1 July 2022. The Income Tax Assessment (1997
Act) Regulations 2021 ss 83A-315.01 to 83A-315.09 were read in compilation 16
(27 June 2026); their s 83A-315.08 table (266 entries) and s 83A-315.09(2)
percentages (19 rows) agree with the ATO legal database's copy read the same
day. They apply to income years from 2021-22 (s 1000-1.02; commenced 1 April
2021), so the worksheet values a right by them only on a day from 1 July 2021.

Choices the Act leaves to the reader:

- The exercise period is compared by calendar months: a period is no more
  than N months when the last exercise day is on or before the same day N
  months after the valuation day (the month's last day where shorter). The
  ATO's example counts whole months.
- s 83A-110(1) with a cost base above the market value includes nil; the
  loss is a CGT matter.
- Salary sacrificed shares or rights at a real risk of forfeiture, their
  discount equal to their market value, beside shares relying on
  s 83A-105(4) in one year are refused: whether the first count toward the
  $5,000 is not settled here. So are forfeited interests acquired in that
  year, which state no conditions. Only a salary sacrificed share relies on
  s 83A-105(4); a share beside it that is not is treated on its own facts.
- A salary sacrificed right without a real risk of forfeiture is refused:
  its scheme fails s 83A-105(4)(b)(ii), which `salary_sacrifice` states.
- Act No. 8, 2022 Sch 10 item 9 is applied to the taxing point after the
  30-day rule: ending employment before 1 July 2022 counts only where that
  point is before then.
- The 15-year taxing point is the day before the 15th anniversary of
  acquisition, the end of the period starting then.

`tests/ess_discount.rs` holds these cases.

| Case | Source | Worksheet |
|---|---|---|
| $1,500 discount, $91,000 adjusted taxable income | ATO $1,000 reduction example 1 (Lester) | $1,000 reduction, $500 |
| $1,300 discount, $200,000 | example 2 (Jane) | no reduction, $1,300 |
| 2,500 shares at $1.00 bought for $0.85 | ATO start-up concession example (Sarah) | start-up, nil; at $2,124.99, taxed upfront |
| 200,000 rights at 10 cents, shares at 12 cents, 48 months | ATO unlisted rights example (Barry) | 19.4%, $3,880; value $4,000 intrinsic |
| Restrictions lifted 23 June, sold 20 July for $1,518 | s 83A-115(3), the 30-day rule | taxing point 20 July, $1,518 in the next year |
| Employment ended 1 March 2022, then 1 July 2022 | former s 83A-120(5), Act No. 8, 2022 | taxing point 1 March 2022; then the restrictions |
| Forfeited by a hurdle, a lapse, a market value condition | ATO forfeiture examples 1, 7, 9 and 10 | excluded, excluded, not excluded |
| Every s 83A-315.08 entry and s 83A-315.09 row | the regulations | each entry |

## Government super amounts

This worksheet was added on 22 November 2026. Read that day:

- the Superannuation (Government Co-contribution for Low Income Earners) Act
  2003, compilation 28, in force from 1 July 2018 to 30 June 2027
  (https://www.legislation.gov.au/C2004A01195/2018-07-01/2018-07-01/text/original/epub/OEBPS/document_1/document_1.html):
  ss 6 to 11, 12C and 12E. The Register lists one later version, from
  1 July 2027, made by Schedule 4 to Act No. 8 of 2026
  (https://www.legislation.gov.au/C2026A00008/asmade/text), which replaces
  LISTO's $37,000 and $500 from 2027-28 (item 4);
- the Superannuation (Government Co-contribution for Low Income Earners)
  Regulations 2004, compilation 8 (in force to 31 March 2022), reg 24(2), and
  Regulations 2022 (F2022L00155), reg 15(2): an amount paid that is not a
  multiple of 5 cents is increased to the next multiple;
- ITAA 1997 ss 290-230 and 290-235, compilation 268, volume 6
  (https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_6/document_6.html);
- the ATO's 'Government contributions' (QC18123, last updated 27 April 2026,
  Table 25, the thresholds), 'Super co-contribution' and 'Low income super tax
  offset' (last updated 2 August 2023), 'Superannuation-related tax offsets'
  (QC72198, last updated 8 June 2026), 'myTax 2026 Government super
  contributions' (published 1 June 2026) and the FBT rates pages (20 May 2026
  and 1 April 2026), which give 47% for the FBT years ending 31 March 2021 to
  2027, behind the 0.53 factor in adjusted taxable income.

The years from 2020-21 rest on a comparison of the compilations in force on
1 July 2020 with today's: the Family Assistance Act Sch 3 cls 2, 4, 5 and 8
(compilation 101 and 1 July 2026), ITAA 1997 s 995-1 'reportable
superannuation contributions', 'total net investment loss', 'assessable FHSS
released amount' and 'reportable fringe benefits total', and ss 290-230 and
290-235 (compilations 210 and 268) read the same. Sch 3 cl 7's list of tax
free pensions changed, and FBTAA ss 135P and 135Q now count exempt zero or
low emission car benefits; the operator supplies both amounts.

Choices the Act leaves to the reader:

- s 10(1C) reduces $500 by 3.333 cents 'for each dollar' above the lower
  threshold; the worksheet applies 0.03333 to the exact excess, which is the
  same for whole-dollar income.
- s 12C(1)(a) presupposes concessional contributions, and the ATO's page
  requires them, so nil contributions give no LISTO rather than the $10
  minimum of s 12E(2)(c).
- s 6(1)(b) asks what share of total income is eligible; with nil total income
  there is no share, so nil total income is refused where the test would decide
  an amount, and not where another condition already fails.
- s 12C(1)(a) counts concessional contributions that s 12E(3) leaves out of
  the amount (constitutionally protected funds and other ITAA 1997
  s 291-165(1)(c) amounts), so a person with them could be owed the $10
  minimum on a nil 15%; such a person is out of scope.
- s 290-230's conditions that turn on the time of each contribution (spouses,
  both resident, not living apart, a complying fund, not deductible, not a
  Family Law Act payment) decide which contributions count, so the operator
  gives only those.

`tests/government_super_amounts.rs` holds these cases.

| Case | Source | Worksheet |
| --- | --- | --- |
| Angelo, $35,000 and $1,040 of personal contributions | ATO 'Super co-contribution' | $500 in every year from 2020-21 |
| Julie, 2022-23, $32,000, $1,000 of deductions, $3,360 of super guarantee | ATO 'Low income super tax offset' | Adjusted taxable income $31,000; LISTO $500 |
| Robert and Judy; Carmel and Adam; Aleah and Lewis | ATO 'Superannuation-related tax offsets', Examples 1 to 3 | $540; $360; $360 |
| Total income at each year's lower threshold, $1 above, $1 below the higher, and at the higher | Worked by hand from Table 25 and s 10(1C) | $500; $499.97, paid $500.00; $20 (s 11); nil (s 6(1)(c)) |
| 2025-26, $50,000 and $1,000 of contributions; $10 of contributions | Worked by hand | $416.28, paid $416.30; $20 |
| Eligible income $4,999 and $5,000 of $50,000; $60,000 of business income less $20,000 of deductions | Worked by hand under s 6(1)(b) and s 8(2), (3) | Not met, met; total income $40,000, the test on $60,000 |
| Adjusted taxable income $37,000 and $37,001; contributions of $50 and $1,234.57 | Worked by hand under ss 12C(1)(b) and 12E(2) | LISTO, none; $10; $185.19, paid $185.20 |
| Spouse income $39,999 and $40,000; the cap and balance tests not met | Worked by hand under ss 290-230 and 290-235 | $0.18, nil; nil |

## Work expenses

This worksheet was added on 22 November 2026, when every source below was
read.

- ITAA 1997 ss 28-12, 28-25, 28-90, 28-95, 28-100, 28-110, 28-115, 28-120 and
  28-125 on the ATO legal database
  (https://www.ato.gov.au/law/view/document?docid=PAC/19970038/28-25 and the
  same address for each section).
- The cents per kilometre rates for every year on the ATO's 'Cents per
  kilometre method' (last updated 12 August 2026); the 2020 instrument
  (F2020L00676, https://www.ato.gov.au/law/view/pdf/ldt/mve2020-001.pdf), the
  2023 one (F2023L00767, LI 2023/23, 'income years commencing on or after
  1 July 2023 is 85 cents') and the 2026 one (F2026L00785, LI 2026/19, s 6,
  91c for the year commencing 1 July 2026, a base of 89c with a one-off 2c
  uplift) in full; the 2022 (F2022L00813, 78c) and 2024 (F2024L00697, 88c)
  ones through the ATO's software developer notices. The ATO notice for 2022
  records that 2021-22 kept 2020-21's 72c.
- PCG 2023/1 as updated 16 April 2025, paras 14 to 28, 45 to 89 and Table 1
  (https://www.ato.gov.au/law/view/document?docid=COG/PCG20231/NAT/ATO/00001).
- The ATO's 'Fixed rate method' (8 June 2026), 'Fixed rate method - 52 cents'
  and 'Shortcut method' (both 25 June 2024), 'Actual cost method' (8 June
  2026), 'Logbook method' (12 June 2026), 'Self-education expenses' and
  'Self-education reduction in expenses' (both 8 June 2026) and the 2022 D4
  instructions (26 May 2022).
- TR 98/9 (point in time 1 July 2004) paras 25 to 30 and 119 to 155 on ITAA
  1936 s 82A, and Treasury Laws Amendment (2022 Measures No. 2) Act 2022
  (No. 84, 2022) Schedule 3 items 26 and 33(1), which repeal s 82A for
  assessments for 2022-23 and later years. The text of s 82A itself was not
  read: the ATO legal database returned 'currently unavailable' and then not
  found for the repealed section. Its rule is taken from the ATO page and
  TR 98/9, which state and apply it.

The hours diary and log book inputs were added on 24 November 2026, when
PCG 2023/1 (16 April 2025, still the latest version in its history table),
paras 16, 18, 23 to 34, 56 to 62 and 68 to 72 and Table 1; ITAA 1997
ss 28-90, 28-95, 28-100, 28-110 to 28-130, 28-140, 28-150 and 28-155 in compilation
C2026C00471 (No. 268, 1 October 2026) on the Federal Register of
Legislation; and the ATO's 'Logbook method' (12 June 2026) were read again.
A diary is read only from 2022-23 (para 16), since the record rules for the
52c rate and the shortcut were not read again. The log book period may run
into the next or previous income year so long as it includes part of the
year it is kept for (s 28-120(2)). The year's business kilometres from a log
book are the year's total kilometres times the log book percentage, the
estimate s 28-90(5)(a) asks for and the ATO's steps 1 to 5 apply; Lana's
69% is reproduced from a 12 week log book of 2,070 business kilometres in
3,000.

Choices the sources leave to the reader:

- PCG 2023/1 Table 1 gives 70c 'From 1 July 2024' with no end, so 2026-27
  takes 70c; the ATO's 'Fixed rate method' page lists only 2024-25 and
  2025-26.
- The 52c rate's coverage (energy, cleaning and home office furniture) and
  its dedicated home office condition are the ATO's 52 cent page's. That page
  covers energy for heating, cooling and lighting and lists no energy claimed
  beside the rate, so the whole `energy` input, devices included, is taken as
  covered; at worst this understates a 52c claim.
- Every hourly rate needs an additional running expense incurred as a result
  of working from home (PCG 2023/1 paras 18(b) and 49 to 55, and the ATO's
  52 cent and shortcut pages): from 2022-23 one of the kinds the rate covers
  (energy, phone, internet, stationery or computer consumables, para 14), and
  before then any running expense. Amounts of nil do not show that, as the
  fixed rate needs no amounts, so `running_expenses_incurred` asks it; false
  beside an amount for such an expense is refused as a conflict.
- A log book must stay representative: the ATO's 'Logbook method' needs a new
  one after a change such as a new job or home, so
  `log_book_representative` false is refused.
- Category D (cents per kilometre) cannot sit beside log book car expenses in
  category A ('Self-education reduction in expenses'), so a category D above
  0 needs `log_book_car_in_category_a` false.
- TR 98/9 para 129 excludes short-term refresher and staff development
  courses, seminars, conferences and workshops from s 82A, not every course
  of short duration; `prescribed_course` is the operator's answer.
- The business use percentage is not rounded: the deduction uses the exact
  share, and the percentage shown is rounded to 2 places.
- A tie between methods names the fixed rate, which needs fewer records.
- Return amounts disregard cents, as PCG 2023/1's footnotes (TAA 1953 Sch 1
  s 388-85) and the ATO's examples do; the ATO's Johan example shows
  $2,212.32 as $2,212.

`tests/work_expenses.rs` holds these cases.

| Case | Source | Worksheet |
| --- | --- | --- |
| Keisha, 843 hours in 2025-26 | ATO 'Fixed rate method' | $590.10, $590 claimed |
| Yang, 567 hours, a $299 chair and a $250 desk | ATO 'Fixed rate method' | $396.90 + $549.00 = $945.90, $945 claimed |
| Wanda, 136 recorded hours | ATO 'Fixed rate method' | $95.20, $95 claimed |
| Gerry, 768 hours in 2024-25 | PCG 2023/1 example 1 | $537.60, $537 claimed |
| Piruntha, 560 hours, desk and chair $439.20, laptop $696.52 | PCG 2023/1 example 8 | $392 + $1,135.72 = $1,527.72, $1,527 claimed |
| 100 hours in each year | Rates above | 52c and 80c to 2021-22, 67c to 2023-24, 70c from 2024-25 |
| 2021-22 with and without a dedicated home office | ATO 52 cent and shortcut pages | 52c plus phone, internet, stationery and other assets; without one, the shortcut and actual cost only |
| Sergei, no household bills, a $50 laptop decline in value, each year | PCG 2023/1 example 6 | No hourly rate; actual cost $50 |
| Johan, 2,514 km in 2025-26 | ATO 'Cents per kilometre method' | $2,212.32, $2,212 claimed |
| 1,000 km in each year; 5,000 and 5,085 km | Rates above; s 28-25(2) example | Each year's rate; 5,000 km counted |
| Lana, $17,275.33 decline in value at 69% | ATO 'Logbook method' | $11,919.98 |
| Log books kept 4 and 5 years before, after the year, for 11 and 12 weeks | ss 28-115(2) and 28-120(1) | Accepted, refused, refused, refused, accepted |
| A 2021-22 log book no longer representative in 2025-26 | ATO 'Logbook method' | Refused |
| Maureen, $290 of category A in 2021-22 | ATO 'Self-education reduction in expenses' | $40 |
| Con, categories A $350, B $200, C $100, D $100, E $500; then A $500, B $400, E $2,200 | TR 98/9 paras 145 to 155 | $750; $900 |
| $290 of category A in 2022-23; a seminar in 2021-22 | Act No. 84, 2022 Sch 3; TR 98/9 para 129 | $290 each, no reduction |
| Category A $500 and D $100, with and without log book car expenses in A | ATO 'Self-education reduction in expenses' | $600; refused |
| Wendy (new employment), Bert (general only), Callum's second semester (not employed) | ATO 'Self-education expenses' | Nil in every year |
| Nicky: $200 working from home and $250 of other covered expenses; none claimed; $1,050 claimed; $800 allowed | Explanatory memorandum examples 4.1 to 4.4 | $550 standard deduction; $1,000; nil, $1,050 or $1,000 unclaimed; $200 |
| Simon, $400 of wages; Yui, $95,000 of wages and $550 of covered expenses | ATO 'Standard deduction for work-related expenses' | $400 limit; $450 standard deduction |

The standard deduction (added on 24 November 2026, when these were read):
the Treasury Laws Amendment (Tax Reform No. 1) Act 2026 (No. 49 of 2026,
assented to 26 June 2026; the Parliament's Bills page records it passed both
Houses on 25 June 2026) as made on the Federal Register of Legislation:
Schedule 4 item 3 inserts ITAA 1997 s 25-130, commencing 1 July 2026 (s 2
table item 6), and item 17(1) applies it to assessments for 2026-27 and later
years. s 25-130 on the ATO legal database reads as made. Its explanatory
memorandum (paras 4.11 to 4.19 and examples 4.1 to 4.4) and the ATO's
'Standard deduction for work-related expenses' (last updated 2 November 2026)
state the choice whether to itemise. The worksheet takes every amount a
method works out as a covered deduction: the methods are for an employee's
expenses in gaining salary or wages, under s 8-1 (home office,
self-education), Division 28 (car), s 25-10 (repairs) or Subdivision 40-B
(decline in value). Example 4.4 is set in 2027-28, which the worksheet does
not cover; the test works the same figures in 2026-27.

## Notice period

This worksheet was added on 24 November 2026, when every source below was
read.

- Fair Work Act 2009, compilation 73 (C2026C00355, compilation date 7 July
  2026, the latest on the Federal Register of Legislation,
  https://www.legislation.gov.au/C2009A00028/latest/text): s 117 (notice or
  payment in lieu), s 118, s 123 (limits on scope), s 18 (full rate of pay)
  and s 22 (service; s 22(4) for notice of termination). Endnote 4 records
  s 117 as amended only by Act No. 25, 2021, and no amendment of s 123.
- Fair Work Amendment (Supporting Australia's Jobs and Economic Recovery) Act
  2021 (No. 25, 2021) as made
  (https://www.legislation.gov.au/C2021A00025/asmade/text): Schedule 1 item
  19 adds s 117(4); s 2 table item 2 commences Schedule 1 Parts 1 and 2 on
  27 March 2021; the transitional clause 46(9) it inserts in the Fair Work
  Act's Schedule 1 Part 10 applies the casual reference in s 117 to periods
  of employment starting before, on or after that day.
- The Fair Work Ombudsman's 'Dismissal'
  (https://www.fairwork.gov.au/ending-employment/notice-and-final-pay/dismissal-how-much-notice),
  'Notice of termination and redundancy pay' fact sheet and 'Who doesn't get
  notice'.

Choices the sources leave to the reader:

- 'More than 1 year' is service running past the last day of a year from the
  first day: Acts Interpretation Act 1901 s 2G(2) ends a period of years on
  the day before the anniversary, so service from 1 August 2025 to the end of
  31 July 2026 is exactly 1 year (1 week) and to 1 August is more (2 weeks).
- Excluded days come off the end of the period, as they do not count towards
  its length (s 22(4)(b)). The days between employments on a transfer of
  employment are excluded days too (s 22(5)(b)); the first employer's service
  counts (s 22(5)(a)), so `service_start` is its first day. The note to
  s 22(5) and ss 91(1) and 122(1) let a second employer refuse that service
  only for annual leave and redundancy pay, not for notice.
- 'Over 45 years old' (s 117(3)(b)) is refused for an employee aged 45 in
  completed years at the end of the notice day: read as older than 45 years
  it includes them, read as aged above 45 it does not, and neither the Act nor
  the Fair Work Ombudsman's pages say which. 46 or more is over 45; 44 or less
  is not. A 29 February birthday is taken at 28 February for the 45th and
  1 March for the 46th, the earliest and latest it could fall, so the refusal
  covers both readings.
- The notice ends no earlier than the notice day plus 7 days a week: the time
  between giving notice and the termination day must be at least the period
  (s 117(2)(a)), and the Fair Work Ombudsman starts the period on the day
  after notice is given.
- The payment in lieu takes the hours a week as the same in each week; an
  employee whose hours or full rate differ by week is outside it. It is
  rounded up to the cent, as s 117(2)(b) requires at least the full rate for
  the hours.
- 30 June 2027 is the last notice day accepted, not a statement that the law
  stays the same to then; a later amendment is not covered.

`tests/notice_period.rs` holds these cases.

| Case | Source | Worksheet |
| --- | --- | --- |
| Gunter, 5 months' service, notice on Tuesday 4 August 2026 | FWO 'Dismissal' | 1 week, ending Tuesday 11 August |
| Joan, 35, 5 years and 3 months with 12 months' unpaid parental leave | FWO 'Dismissal' | 4 weeks |
| 50, 3 1/2 years | s 117(3) | 3 + 1 = 4 weeks |
| Exactly 1, 3 and 5 years, and a day more | s 117(3)(a), s 2G | 1, 2, 3 weeks; 2, 3, 4 weeks |
| 50, exactly 2 years; a day short | s 117(3)(b) | 3 weeks; 2 weeks |
| Notice the day before the 45th birthday, on it, the day before the 46th, on it | s 117(3)(b) | Not over 45; refused; refused; over 45 |
| 1 year and 10 days with 10 days excluded | s 22(4) | 1 week |
| Service from 29 February 2024: exactly 1 and 3 years, and a day more; 50, exactly 2 years | s 117(3), s 2G | 1, 2 weeks; 2, 3 weeks; 2 + 1 weeks |
| Transfer: 3 years and 5 days with a 5-day gap between employers | s 22(5) | 2 weeks |
| 3 weeks at 38 hours and $40.00; 1 week at 7.25 hours and $33.33 | s 117(2)(b) | $4,560.00; $241.65, rounded up |
| Each s 123 exclusion; a specified period to avoid the Division | s 123(1) to (3) | No notice; notice as worked |

## Residency tests

This worksheet was added on 22 November 2026. Paragraph (a) of 'resident or
resident of Australia' in ITAA 1936 s 6(1) was read that day in compilation
193 (1 October 2026, the latest on the register), volume 1
(https://www.legislation.gov.au/C1936A00027/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_1/document_1.html),
and compared with compilation 169 (1 July 2020): the definition reads the
same, so one rule set serves 2020-21 to 2026-27. TR 2023/1 (issued 7 June
2023, applying before and after its issue, paragraph 170) was read the same
day at https://www.ato.gov.au/law/view/document?docid=TXR/TR20231/NAT/ATO/00001.

Choices the Act leaves to the reader, settled by TR 2023/1:

- 'More than one-half of the year of income' is 183 days or more, 184 in a
  leap year (paragraph 83 and footnote 51). Of 2020-21 to 2026-27 only
  2023-24 holds 29 February.
- Days are added together whether continuous or intermittent, and part of a
  day counts as a whole day (paragraph 86), over 1 July to 30 June
  (paragraph 87). The worksheet counts each calendar day the periods given
  cover once.
- Residing, domicile, permanent and usual places of abode and intention turn
  on all the facts (paragraph 4); the worksheet takes them as answers and
  infers none.
- A resident under any test may be one for part of the year only
  (paragraph 105); the worksheet answers for the year and does not set the
  part-year dates.
- A treaty may allocate residence to another country for its purposes
  (paragraph 108); a resident who is also resident elsewhere is refused.

`tests/residency_tests.rs` holds these cases. The examples give no dates, so
each date below was chosen to fit its narrative.

| Case | Source | Worksheet |
|---|---|---|
| Conchita: 1 August 2025 to 30 April 2026 on holiday, abode in Mexico | TR 2023/1 example 5 | 273 days; not a resident |
| Stuart: domiciled here, no permanent abode abroad, 21 days here | example 11 | resident (domicile) |
| Brian: domiciled here, permanent abode in Vietnam, 34 days here | example 10 | not a resident |
| Megan: here all 2025-26, intends to stay | example 16 | resident (resides, 183 days) |
| Aiko: resides from arrival, 72 days here | example 13 | resident (resides) |
| Alani: 1 September 2025 to 31 May 2026 of seasonal work, abode in Papua New Guinea | example 14 | 273 days; not a resident |
| Grace: 15 August 2025 to 14 August 2026 on a working holiday, abode in the UK | example 15 | 320 days in 2025-26; not a resident |
| Ryan: 1 July 2024 to 30 June 2026 on 2 working holidays, abode in Ireland | example 18 | 365 days in each of 2024-25 and 2025-26; not a resident in either |
| Mark: abroad on assignment, family home here | example 7 | resident (resides, domicile) |
| 182 and 183 days from 1 July in each year but 2023-24; 183 and 184 in 2023-24 | footnote 51 | not met; met |
| A period of one day; periods touching or overlapping; periods across 1 July or 30 June | paragraphs 86 and 87 | each day counted once, inside the year |
| Each Commonwealth fund link | s 6(1)(a)(iii) | resident |

## Lodgment required

This worksheet was added on 22 November 2026. Each year's lodgment
instrument was read that day on the Federal Register of Legislation, as made
(epub text and PDF):

| Year | Instrument |
|---|---|
| 2020-21 | Notice of Requirement to Lodge a Return for the Income Year Ended 30 June 2021, F2021L00577 |
| 2021-22 | Notice ... Ended 30 June 2022, F2022L00508 |
| 2022-23 | Notice ... Ended 30 June 2023, F2023L00533 |
| 2023-24 | Taxation Laws (Requirement to Lodge a Return for the 2024 Year) Instrument 2024, F2024L00550 |
| 2024-25 | ... for the 2025 Year) Instrument 2025, F2025L00570 |
| 2025-26 | ... for the 2026 Year) Instrument 2026, F2026L00618 |

A search of the register that day for 'Requirement to Lodge a Return' found
no instrument for the 2027 year, so 2026-27 is refused. Tables A and B read
the same in the 3 Notices but for the name of the labour mobility program;
s 6(2) and 6(3)(a) read the same in the 3 instruments, and s 6(3)(b) adds
s 6(2)(a)(iv) payments to its exclusions from 2024-25. The amounts that
change by year are in the README.

Choices the instruments leave to the reader:

- Table K is 'to be read with Tables B and E', and s 7(2) names s 6(2)(i)
  and 6(3) and s 7(3) names s 6(3); each says the person 'is not required to
  lodge ... under this instrument', and the notes to s 7 say the same of a
  person covered by s 7(2) or 7(3). So a person covered by a named provision
  whom the exception relieves has no requirement under the instrument, even
  one that also meets a provision it does not name, such as PAYG
  withholding; a person covered by none of the named provisions is not
  relieved. The 2024 explanatory statement (paragraph 17) says the redrafting
  was not meant to change who must lodge, but s 7(2) names s 6(2)(i), which
  Table K did not reach; each year follows its own text.
- Tables E to G and s 6(6) to 6(8) cover trustees, agents and ship masters;
  the worksheet is for an individual's own return, so they are out of scope.
  Each year's instrument sets 31 October after the year as the due date for
  the rest.
- Table P and s 7(1)(f) relieve every requirement of the instrument, so a
  working holiday maker under $45,001 is relieved whatever else holds.
- s 6(10) reaches 'a foreign resident ... during the year': a part-year
  resident is one.

`tests/lodgment_required.rs` holds these cases. The instruments publish no
worked examples, so the facts are fabricated, each at a threshold.

| Case | Provision | Worksheet |
|---|---|---|
| Full-year resident at $18,200.00 and $18,200.01, each year | s 6(3)(a)(i), Table B(1)(a) | not required; required |
| Part-year resident for 1, 5, 11 and 12 months, at the limit and $1 over | s 6(3)(a)(ii) | $13,859, $15,439, $17,809 and $18,200 |
| Non-resident with and without income taxable here | s 6(3)(b) | required; not |
| Under 18 at $416.00 and $416.01, own income or dividends | s 6(2)(i), Table A(9) | not required; required |
| Dividends and franking credits at $18,200.00 and $18,200.01 | s 6(2)(o) | not required; required |
| Each stated condition, 2025-26 and 2021-22 | s 6(2), Table A | required, cited by year |
| Foreign resident with a study debt at each year's amount and $1 over | s 6(10), Table J | not required; required |
| Working holiday maker with PAYG withheld at $45,000 and $45,001 | s 7(1)(f), Table P | relieved; required |
| Listed payment at each year's amount, 'less than' and 'at most' | s 7(2), Table K(1) | relieved below; required at or above |
| Seniors offset, each category, $1 under and at each amount | s 7(3), Table K(2) | relieved; required |
| Seniors offset with PAYG withheld | s 7(3) and 6(2)(a) | refused |
| Under 18 with a listed payment, 2024-25 and 2022-23 | s 7(2); Table K | relieved; refused |

## Deferred tax

Added on 22 November 2026, when the sources were read: the AASB 112
compilations of September 2023 (F2023C00930, the current version on the AASB's
portal) and March 2020 (compilation no. 4) with their tables of amendments;
the AASB's list of AASB 112 versions; AASB 2021-5; AASB 18 Appendix D (June
2024); the AASB's December 2012 agenda decision on substantive enactment; the
Income Tax Rates Act 1986 ss 23(2) and 23AA (compilation 66, 1 July 2026); and
the ATO's company tax rate tables for 2020-21 to 2025-26.

The paragraphs applied (5, 7, 8, 15, 24, 34 to 36, 46 to 48, 53, 56, 58, 60,
80, 81(c), 81(e), 85 and 86) read the same for every reporting period of
2020-21 to 2026-27. AASB 2021-5 changed paras 15 and 24 from 1 January 2023
only in the initial recognition exception, which the caller applies by leaving
those differences out; AASB 1060 replaced Appendix A alone; and AASB 18
Appendix D amends none of the applied paragraphs. The standard is copyright:
the command and these notes state its rules in their own words, and the
vectors below are made up to apply the paragraphs, not the AASB's illustrative
examples.

| Income year | Base rate entity | Otherwise | Source |
| --- | --- | --- | --- |
| 2020-21 | 26% | 30% | ATO, Tax rates 2011-12 to 2020-21 |
| 2021-22 to 2025-26 | 25% | 30% | ATO tables for each year; s 23(2) |
| 2026-27 | 25% | 30% | s 23(2), compilation 66 |

Deferred tax is measured at 25% or 30% as the caller expects the company to be
a base rate entity or not when the differences reverse, which for every
supported year is 2021-22 or later. Opening balances of 2020-21 at the base
rate entity rate are refused: they reverse partly at 26%. A change in the
expected rate with opening balances is refused when the year's rate is not the
closing rate: the balances that reverse in the year do so at the year's rate,
and only the rest is remeasured (paras 47 and 60(a)).

| Vector | Facts given | Result |
| --- | --- | --- |
| V1, 2025-26, 30% | Plant $70,000 / $50,000, leave provision $12,000 / nil, prepayment $3,000 / nil; opening $80,000 / $65,000, $10,000, $2,000; profit $100,000 with $4,000 not deductible | DTL **$6,900**, DTA **$3,600**, deferred expense **$1,200**, current **$30,000**, expense **$31,200**, effective rate 31.2% |
| V2, 2024-25, 30% to 25% | Equipment $40,000 / $30,000 (opening $50,000 / $45,000), losses $20,000 unrecognised at the start, $15,000 used in the year and $5,000 left and recognised at the end, profit $30,000 with $2,000 not assessable | Rate change **-$250**, recognition **-$1,250**, origination **$1,250**, losses used **-$3,750** (para 80(e)), expense **$1,750** |
| V3, 2020-21, 26% and 25% | A new $10,000 taxable difference, profit $50,000, taxable income $40,000 | Current **$10,400**, deferred **$2,500**, expense **$12,900**, a **-$100** rate difference line |

`tests/deferred_tax.rs` also covers an unexplained taxable income, tax
offsets, each class's sign, a loss before tax, each year's rates, a rate
change with the year's rate at and away from the closing rate, a 2-cent
rounding remainder and every refusal.

## WIP lock-up

Added on 24 November 2026. The site's `tools/wip-schedule` page, read that day
from the site's main branch, is the construction WIP page for The WIP Tally
(AASB 15 cost-to-cost progress): it has no calculator script, no ageing and no
lock-up, and its one worked example is The WIP Tally's own test. The
definitions mirrored are Ozzit's, published on the site as
`assets/ozzit-function-reference-v3.7.0.json`: `oz.WIPDaysλ(WIP,
RelatedFlow, [Days])` and `oz.ReceivableDaysλ(Receivables, Sales, [Days])`,
each balance / flow x Days with Days defaulting to 365, a nil flow giving
#DIV/0! and Days of 0 or less #NUM!, and lock-up days as WIP days plus
receivable days (the WIPDaysλ note). No Standard or Act defines lock-up or
the ageing buckets.

| Vector | Facts given | Result |
| --- | --- | --- |
| O1 | `=oz.WIPDaysλ(42000, 510000)` | **30.0588** |
| O2 | `=oz.ReceivableDaysλ(125000, 1200000)` | **38.0208** |
| O3 | `=oz.ReceivableDaysλ(125000, 330000, 90)` | **34.0909** |
| O4 | O1 and O2 together | lock-up **68.0797** |
| W1 | as at 30 Jun 2026: $5,000 (age 0), $7,500 (30), $2,500 (31), $4,000 (90), $3,000 (91), $1,250.50 (211); fees $300,000, receivables $60,000, billings $320,000 | buckets **$12,500.00**, **$2,500.00**, **$4,000.00**, **$4,250.50**; WIP days **28.29**, receivable days **68.44**, lock-up **96.73** |

O1 to O3 are the reference's published examples; W1 is made up and worked in
Python's `decimal`. `tests/wip_lockup.rs` also covers stated buckets, each
bound on buckets, days and entries, the year boundaries and every refusal.

## Impairment test

Added on 24 November 2026, when the sources were read: AASB 136 compilation
COMPjan26_07-26 (F2026C01108, periods beginning from 1 July 2026 to before
1 January 2029) with its tables of amendments, and compilation COMPdec22_01-23
for earlier periods. Since the August 2015 Standard the tables list changes to
paras 2 and 140N (AASB 17), Aus2.1, Aus5.1, Aus6.1, Aus6.2, Aus32.1 and
Aus32.2 (AASB 2016-4), Aus141.1, Appendix E (AASB 1060) and Illustrative
Example 10 (AASB 2026-1) only; none of paras 6, 18 to 20, 30 to 33, 39, 50 to
57, 59 to 62, 104 and 105 changed, so the rules applied read the same for
every reporting period ending in 2020-21 to 2026-27.

Illustrative Example 2 (IE23 to IE32) gives the Country A unit's cash flows
for 20X2 to 20Y2 (230, 253, 273, 290, 304, 313, 307, 289, 245, 184 and 61) at
15%, goodwill of 1,000 and identifiable assets of 1,833. Schedule 2 rounds
each discounted flow to whole units (200, 191, 180, 166, 151, 135, 115, 94,
70, 45 and 13) and totals 1,360; Schedule 3 recognises 1,473 (goodwill 1,000,
identifiable assets 473). Unrounded, value in use is 1,361.20 and the loss
1,471.80 (goodwill 1,000.00, identifiable assets 471.80): each year's present
value rounds to Schedule 2's figure, and the totals differ only by Schedule
2's rounding of each line. E2 so reconciles the 2 rounding conventions; it
does not reproduce Schedule 3's 1,473. I2 to I7 are made up and worked in
Python's `decimal`.

| Vector | Facts given | Result |
| --- | --- | --- |
| E2 | Example 2 as above | value in use **$1,361.20**, loss **$1,471.80**: goodwill $1,000.00, other $471.80 |
| I2 | goodwill 200, plant 600 with floor 550, building 400; fair value less costs of disposal 700 | loss **$500.00**: goodwill $200, plant $50 at its floor, building **$250** |
| I3 | goodwill 100, A 500 floor 450, B 300 floor 280; recoverable nil | loss **$900.00**: goodwill $100, A $50, B $20, unallocated **$730** |
| I4 | recoverable above carrying | no loss |
| I5 | 12,000, 12,500, 13,000, 13,500, 14,000 and terminal 40,000 at 9.25% | value in use **$75,599.58** |
| I6 | 3 assets of $100.00, loss $100.00 | **$33.33**, **$33.34**, **$33.33** |
| I7 | A 300 floor 290, B 300 floor 295, nil, C 700, D 200; loss $100.00 | A and B reach their floors in one round (**$10.00**, **$5.00**); C **$66.11**, D **$18.89** |

`tests/impairment_test.rs` also covers the year boundaries, negative cash
flows and terminal values, an unmeasurable fair value, floors above carrying
amount, losses a cent below goodwill and a cent above the carrying amount, a
non-controlling interest without goodwill and every refusal.

## Leave provisions

Added on 22 November 2026, when the sources were read: AASB 119 compilation
no. 5 (COMPdec22_01-23, periods beginning from 1 January 2023 to before
1 July 2026) with its tables of standards and amendments; AASB 2022-9
(December 2022), effective from 1 July 2026 and not compiled there; and
AASB 101 compilation no. 6 (COMPdec22_01-24).

Read again on 23 November 2026: AASB 119 compilation COMPdec22_07-26
(F2026C01064, periods beginning from 1 July 2026 to before 1 January 2027,
with AASB 2022-9 compiled); AASB 2018-2 and AASB 2021-7; AASB 101 compilations
COMPdec22_01-24, ACOMPdec22_01-23 (2023), ACOMPdec21_01-22 (2022),
COMPmar20_07-21 and COMPmay19_01-20 (2020); and AASB 2020-1, AASB 2020-6 and
AASB 2022-6.

Of the paragraphs applied (AASB 119 paras 8, 9, 11, 13, 15, 16, 67, 68, 70,
72, 75 to 87 and 153 to 156, with para 155's paras 56 to 98; AASB 101 paras
69, 70, 72A and 75A), the AASB 119 table of amendments lists every change since
the August 2015 standard: para 3 (AASB 2021-7, from 1 January 2022, removing a
reference to AAS 25 beside AASB 1056; reporting by
employee benefit plans stays outside AASB 119), para 8 (AASB 17, AASB 2022-8
and AASB 2022-9, insurance definitions and footnote Aus1), and paras 57 and
156 (AASB 2018-2, cross-references to para 122A, from periods beginning
1 January 2019, before every period in scope). None changes a rule applied
here, so the AASB 119 rules read the same for every reporting period ending
from 1 July 2020 to 30 June 2027, whatever its reporting date. AASB 18 and
AASB 2026-2 apply to periods beginning from 1 January 2027 and 1 July 2029,
outside the scope.

AASB 101 para 69 was amended only by AASB 2020-1, which AASB 2020-6 and
AASB 2022-6 deferred to periods beginning on or after 1 January 2024 (para
139U; early application with AASB 2022-6 allowed and disclosed). Before then
para 69(d) needs an 'unconditional right' to defer settlement (the 2020 to
2023 compilations); from then a right at the end of the reporting period, which
must have substance (para 72A), whatever the entity expects (para 75A). The
`year` is the financial year in which the reporting period ends, so every
period ending in 2020-21 to 2023-24, and a 2024-25 period ending before
31 December 2024, takes the earlier test unless the entity applied AASB 2020-1
and AASB 2022-6 early (paras 139U and 139W, with disclosure); a 2024-25 period
ending from 31 December 2024, and every later one, takes the new one. The caller states
`can_defer_settlement` under the test for its period; the calculation is the
same.

The probability is measurement (AASB 119 para 72, through para 155) and the
right to defer is classification (AASB 101 para 69(d)), so a long-term line
without a right to defer takes any probability: V7 below. Para 8 classifies
the whole benefit (IAS 19 Basis for Conclusions BC20(b)), so the part of a
long-term benefit due within 12 months stays long-term; a long-term line takes
1 to 50 whole years, and such a part takes 1, the nearest: V8 below. `hourly_rate` is the rate expected to be paid on a
short-term line (para 16) and the rate at the reporting date on a long-term
line, which salary growth projects (para 87(b)).

AASB 119 has no numeric example for annual or long service leave. V1 to V6
are made-up figures worked independently in Python's `decimal`. P68 applies
the para 68 illustrative example (a lump sum of 1% of final salary for each
year of service, salary 10,000 growing 7%, discount 10%): its obligations at
the end of years 1 to 4 (89, 196, 324 and 476) are the command's amounts with
the cents dropped.

| Vector | Facts given | Result |
| --- | --- | --- |
| V1 | Annual, short-term, 152 h x $45.50, probability 1 | **$6,916.00** current |
| V2 | Long service, 400 h x $52.00, 2 years, growth 3%, discount 4.5%, probability 1, no right to defer | projected $22,066.72, **$20,207.16** current |
| V3 | Long service, 120 h x $38.00, 5 years, growth 3.5%, discount 4.25%, probability 0.4, right to defer | projected $5,415.85, **$1,759.33** non-current |
| V4 | Annual, long-term, 300 h x $40.00, 1 year, growth 3%, discount 4% | **$11,884.62** current |
| V6 | Annual, short-term, 80 h x $30.00, probability 0.85 | **$2,040.00** current |
| V7 | Long service, 400 h x $52.00, 2 years, growth 3%, discount 4.5%, probability 0.9, no right to defer | projected $22,066.72, **$18,186.44** current |
| V8 | Long service, part due within 12 months, 100 h x $50.00, 1 year, growth 3%, discount 5%, probability 1, no right to defer (0 years refused) | **$4,904.76** current |
| V1, V2, V3, V4 and V6 together | | annual **$20,840.62**, long service **$21,966.49**, total **$42,807.11**, current **$41,047.78**, non-current **$1,759.33** |
| P68 | 1 to 4 years' benefit at rates $100.00, $107.00, $114.49 and $122.50, 4 to 1 years | **$89.53**, **$196.96**, **$324.99**, **$476.64** |

`tests/leave_provisions.rs` also covers each year, the year boundaries, each
assumption's bounds (1 to 50 years; 0 refused), both versions of the right to defer in the text, and every refusal.

## Long service leave

This worksheet was added on 24 November 2026. Each source was read that day:

- New South Wales: Long Service Leave Act 1955 s 4(2)(a) and (a3).
  legislation.nsw.gov.au and AustLII answered with Cloudflare challenges, so
  the in-force text was read in the Internet Archive's captures of
  legislation.nsw.gov.au: the version from 25 March 2021 (captured 1 February
  2022) and the version from 1 July 2025 (captured 16 December 2025) give
  s 4(2)(a) to (a3) word for word alike. NSW Industrial Relations' long
  service leave page and short guide (February 2026) agree.
- Victoria: Long Service Leave Act 2018, authorised version 006 (12 December
  2025), ss 6, 9 and 10; in operation from 1 November 2018, s 9 substituted
  from 1 July 2021 (No. 14/2021 s 75).
- Queensland: Industrial Relations Act 2016 (in force) s 95, last amended by
  Act No. 19 of 2020 s 134, in part 13, division 4, which commenced on assent
  (s 2(1)(c)), 22 June 2020. Section 95(2)(a) and (7) give 8.6667 weeks.
- Western Australia: Long Service Leave Act 1958 s 8, last amended by Act
  No. 30 of 2021 from 20 June 2022; s 8(4) (the 2006 transition, commencement
  day 4 July 2006) cannot apply after 4 July 2007.
- South Australia: Long Service Leave Act 1987, version of 1 July 2017, s 5,
  unchanged since 1 January 1988 apart from s 5(1a) (1997).
- Tasmania: Long Service Leave Act 1976 ss 8 and 9 and Schedule 3 (Act No. 41
  of 2011 from 1 July 2012, with a 12-month transition).
- The ACT: Long Service Leave Act 1976 R29 (19 November 2025) ss 3, 4, 11A and
  11C, against R28 (1 October 2022): A2025-30 added only the offence in
  s 11A(4) and (5). WorkSafe ACT's calculation guidelines give the weeks as
  years x 1/5 x 52/12.
- The Northern Territory: Long Service Leave Act 1981 (as in force at
  14 October 2015) ss 8 and 10, s 10 last amended from 27 October 2004.

The site's long service leave calculator (ryanduguid.github.io, main at
dcc7ecbe) was run on its 39 page cases, its unit tests' inputs and a sweep of
every place, status, reason and threshold, 3,016 rows in
`tests/data/long_service_leave_site_cases.json`. The worksheet gives the same
service, weeks, balance and value on 2,485 rows and refuses the 69 the page
refuses. It departs on 462:

| Rule | Source | Rows |
| --- | --- | --- |
| Queensland uses 8.6667 weeks for 10 years, not 8 2/3 | Industrial Relations Act 2016 s 95(2)(a) and (7) | 241 differ in the fourth place of the weeks or in cents (3 page cases) |
| A last day before 1 July 2020, after 30 June 2027 or before the earliest text read is refused | each Act's commencement and amendment history above | 185 refused (2 page cases) |
| A Queensland fixed term ending before 10 years, a South Australian departure without notice between 7 and 10 years, and Tasmanian leave taken with service from before 1 July 2012 are refused | Industrial Relations Act 2016 (Qld) s 95(4)(d); Long Service Leave Act 1987 (SA) s 5(4); Long Service Leave Act 1976 (Tas) Schedule 3 cl 2(4) | 42 refused (6, 6 and 30; 2 page cases) |

The Queensland row count falls to 235 (2 page cases) because 6 of its rows
are now refused. WorkSafe ACT's guidance note 067A rounds 7 years to 6.0667
weeks before multiplying, giving $3,397.35 at $560 a week; the Act's 1/5 of a
month a year gives 91/15 weeks exactly, $3,397.33, which the worksheet uses.
The part year as days over 365.25 (over 364 in Queensland, as Business
Queensland's example of 18 years, 3 months and 16 days giving 15.8548 weeks
requires) is the worksheet's convention: the Acts give a proportion of years
without a day count.

## Foreign income conversion

This worksheet was added on 22 November 2026. ITAA 1997 ss 960-49 to 960-55
were read that day in compilation 268 (1 October 2026) and compared with
compilation 210 (from 1 July 2020): s 960-50(6) differs only by items 9D to 9F
(digital games), which the worksheet does not take. The Income Tax Assessment
(1997 Act) Regulations 2021 (compilation 16, 27 June 2026; commenced 1 April
2021) s 960-50.01 substitute item 8 and add items 11A and 12, and Schedule 2
clause 3 sets the average rate; they apply to income years starting on or
after 1 April 2021 (s 1000-1.02), so 2020-21 is under the Income Tax
Assessment Regulations 1997 (compilation 85, in force 1 July 2020), reg
960-50.01 and Schedule 2 clause 1.3, which read the same in substance. The
ATO's Foreign exchange rates pages (read the same day) publish rates as the
foreign currency equal to 1 AUD, from the Reserve Bank since 1 January 2020,
and accept a bank's or another reliable external rate for a currency they do
not list.

Choices the Act leaves to the reader:

- A specific rate must be for the day the item names. A rate for another day,
  such as the last published one before a weekend, is the operator's judgement
  and is refused rather than used.
- Each amount is rounded half up to cents and the totals add the rounded
  amounts; the ATO's examples show whole dollars.

`tests/foreign_income_conversion.rs` holds these cases.

| Case | Source | Worksheet |
|---|---|---|
| NZD1,000 and NZD1,500 paid when deductible at 1.10 | ATO 'Translation of foreign currency rental expenses' (AUD909, AUD1,364) | $909.09, $1,363.64 |
| NZD600 at 1.20 | the same example (AUD500) | $500.00 |
| NZD2,000 air conditioner held and paid for on the day of 1.05 | the same example's reasoning (AUD1,905); its answer line names the installation day, its figure the day of holding | $1,904.76 at item 2(a) |
| Rent derived when the agent receives it | ATO 'Translation of foreign currency rental income' | item 6(b) |
| USD6,000 at 0.6785, the average for the year ended 30 June 2026 | ATO rates for the financial year ending 30 June 2026 | $8,843.04 |
| An average over 1 July 2025 to 1 July 2026 | Schedule 2 clause 3(1) | refused |
| A specific rate for the day before | s 960-50(6) | refused |

## Foreign income tax offset

This worksheet was added on 22 November 2026. ITAA 1997 Division 770 and
s 63-10(1) item 22 were read that day in compilation 268 (1 October 2026) and
compared with compilation 210 (from 1 July 2020): ss 770-70 to 770-80 are the
same; s 770-10 differs only by a removed offshore banking note and the added
s 770-10(6) on foreign IIR and UTPR taxes, which decide what counts. The
Income Tax Rates Act 1986 ss 3(1), 16, 18 and 20 were read in compilation 66
(1 July 2026) and compared with compilation 55 (1 July 2020): the same, with
the tax-free threshold $18,200 in both. The s 20(1) formula, (the tax-free
threshold - $4,736) + $4,736 x months / 12, is an image in the Act; the
ATO's legal database shows it with the history note that No 150 of 2011
substituted it for 2012-13 onwards. The ATO's Guides to foreign income tax
offset rules 2024, 2025 and 2026 add the Medicare levy and surcharge to income
tax payable in steps 1 and 2.

Choices the Act leaves to the reader:

- The months of a part-year residency period run from the start of the month
  residency began to the end of the month it ended (s 18(1)); 2 periods in one
  month count it once. The threshold is worked exactly and shown to the cent.
- A departure in July, which s 18(1)(a) does not literally cover, counts that
  month, as the ATO's departure page counts the month you left.
- The 2024 guide's step 1 for Anna includes $622.60 of Medicare levy, 2% of
  $31,130 without the low income reduction; the stated method takes it as
  given and the worked method is not offered for 2023-24.

`tests/foreign_income_tax_offset.rs` holds these cases.

| Case | Source | Worksheet |
|---|---|---|
| Anna, 2025-26: $31,130 taxable, $12,000 and $700 disregarded, $390.80 levy, $3,400 paid | FITO guide 2026 Example 16 | $2,459.60 - $260.80: $2,198.80 |
| Anna, 2024-25, $513.00 levy | FITO guide 2025 Example 16 | $2,581.80; $2,321.00 |
| Karen, 2025-26: $80,000 and $60,000, $2,000 paid | FITO guide 2026 Example 17 | $16,388 - $9,988: $6,400; $2,000 |
| Anna and Karen, 2023-24, stated | FITO guide 2024 Examples 16 and 17 | $2,769.60; $6,900 and $2,000 |
| $800 and $1,500 paid, default | s 770-75(2)(a) and note 1 | $800.00; $1,000.00 |
| $1,500 and $300 under s 770-10(2), default | s 770-80 | $1,300.00 |
| John, resident from 17 April, 3 months | ATO 'Tax-free threshold for newcomers to Australia' | $14,648.00 |
| Britney, resident to 22 December 2025, 6 months | ATO 'Living overseas and becoming a foreign tax-resident' | $15,832.00 |
| An eligible pensioner for part of the year | Rates Act s 18(2)(a) | $18,200.00 |
| The worked limit for 2023-24 | resident-tax's years | refused |

`medicare-levy` gives the levy for a single person without an exemption:
$390.80 on $31,130 for 2025-26 and $513.00 for 2024-25, the guides' step 1
totals less the basic tax.

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


A part-year resident (read on 24 November 2026) is composed before 2027-28
from `resident-tax` with the pro-rated tax-free threshold (Income Tax Rates Act
1986 s 20(1), [Part-year tax-free threshold](#part-year-tax-free-threshold)),
the low income tax offset in full (ITAA 1997 s 61-110(1)(a): 'an Australian
resident at any time during the income year'; s 61-115 does not apportion it)
and `medicare-levy` with its non-resident days ([Exemptions](#exemptions)).
The estimate refuses more non-resident days than the year less the residency
months, as the person was resident on at least 1 day of each month counted
(s 18(1)). It refuses for a part-year resident the surcharge, whose worksheet
is for a full-year resident; the seniors and pensioners tax offset, the
private health insurance rebate and a study loan repayment, composed for a
full-year resident only; tax withheld worked out from one regular pay, as the
pays can fall under resident and foreign resident scales; and every year from
2027-28, as s 61-155(1)(b) compares net labour income with 'the tax-free
threshold (within the meaning of the Income Tax Rates Act 1986)', which s 3(1)
sets at $18,200, whilst s 20(1) applies the pro-rated threshold to the
person's year through the Schedule 7 table; the text points to the pro-rated
threshold for the s 61-160 tax on net labour income alone, but no ATO guidance
read on 24 November 2026 confirms it, so the estimate refuses rather than
choose. The worked example (arrived 15 October 2025, 9 months, 106
days, $60,000) gives basic tax $8,977.44, the offset $100, the levy $851.51
and $9,728.95 in all. Tests: `tests/part_year_estimate.rs`.
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

Not-for-profit employers were added on 22 November 2026 from sources read that
afternoon, 5.00 pm to 5.40 pm AEDT:

- FBTAA, compilation 97, volume 1: s 5B(1D) to (1M) (the aggregate
  non-exempt amount: the $17,000 cap for public hospitals, not-for-profit
  hospitals and public ambulance services at step 2, $30,000 for others at
  step 3, and the lesser of $5,000 and the salary packaged meal entertainment
  and entertainment facility leasing amount at step 4; (1L) disregards meal
  entertainment, car parking and entertainment facility leasing among the
  excluded benefits); s 57A(1) to (5) (the exempt employers); s 65J(1),
  (2A) (the rebate, read from the formula image's text: 'FBT rate x (Gross tax
  - Aggregate non-rebatable amount) x Rebatable days in year / Total days in
  year'), (2B) to (2J) (the aggregate non-rebatable amount, times the FBT
  rate at step 4). Volume 2: s 135Q (a s 57A employer's reportable amount, and
  at (1)(c) a car exempt under s 8A counted as if a fringe benefit, which
  s 5B(1K) step 1 reads; refused for these kinds). A rebatable employer's
  bases follow s 65J(2G) step 1, which reads s 5E, so an exempt electric car
  adds nothing; s 135P(3), in the same volume 2 text, brings the car into the
  reportable amount only.
- ATO, 'Completing your FBT return 2026 - not-for-profit employers' and
  'Worked examples for NFP employers completing FBT return 2026' (both
  published 1 April 2026): the steps for items 14C, 17 and 18, item 23 for
  capped employers, and examples 1 to 3 (rebatable: items 17 $13,322.05 and
  18 $35,320.85 at 2.0802 and 1.8868, $13,322.21 and $35,321.00 by the
  formulas; public benevolent institution: 14C $22,005, 16 $10,342.35, 22
  $5,657.65; public hospital: 14C $35,005, 16 $16,452.35, 21 $452.35, all
  matched).
- ATO, FBT return 2026 calculation details for taxable employers, items 20 to
  22 (instalments reported, payment due and credit).

`tests/fbt_return_nfp.rs` holds these cases, the caps' boundaries and the
refusals; `tests/fbt_return.rs` is unchanged and passes.

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
| CGT cost base adjustments | Refused: Schedule 1 to Act No. 49 of 2026 changes how capital gains are worked out from the income year that includes 1 July 2027 | Act No. 49 of 2026 as made (https://www.legislation.gov.au/C2026A00049/asmade/text), Schedule 1 items 82 to 84, read 22 November 2026 at 14:16; see [CGT cost base adjustments](#cgt-cost-base-adjustments) |
| Trade-in | Refused: the car limit for 2027-28 was not published, and CGT event K7 changes with Act No. 49 of 2026 | ATO 'Purchasing a motor vehicle' (last updated 3 June 2026, the 2026-27 limit only), read 18 November 2026 at 07:10; see [Trade-in](#trade-in) |
| Rental property | Refused: s 26-155 (Act No. 49 of 2026) can quarantine residential rental deductions from 2027-28 | ITAA 1997 s 26-155 as the borrowing expenses worksheets read it, 19 November 2026; see [Rental property](#rental-property) |
| Small business CGT concessions | Refused: Schedule 1 to Act No. 49 of 2026 changes Division 152 from the income year that includes 1 July 2027 | Act No. 49 of 2026 as made (https://www.legislation.gov.au/C2026A00049/asmade/text), item 82, read 19 November 2026 at 14:40; see [Small business CGT concessions](#small-business-cgt-concessions) |
| Government super amounts | Refused: Schedule 4 to Act No. 8 of 2026 replaces LISTO's $37,000 limit and $500 maximum from 2027-28, and the Commissioner has not published the 2027-28 co-contribution thresholds | Act No. 8 of 2026 as made (https://www.legislation.gov.au/C2026A00008/asmade/text), item 4, and the ATO's Table 25 (last updated 27 April 2026), read 22 November 2026; see [Government super amounts](#government-super-amounts) |
| Work expenses | Refused: no cents per kilometre rate has been determined under ITAA 1997 s 28-25(4) for 2027-28; LI 2026/19 sets 91c for the income year commencing 1 July 2026 only | LI 2026/19 (https://www.ato.gov.au/law/view/view.htm?docid=OPS/LI202619/00001), read 22 November 2026 |
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

## Charity check

`charity-check` adds no rate. Read on 24 November 2026 (AEDT): the data.gov.au
record for `acnc-register` (licence CC BY 3.0 AU; CSV resource last modified
2026-11-22T17:39:55, 15,227,773 bytes, SHA-256
`559c29245442d98abc473ffa60a2832b4d25ead3ebbf6734dd2857237fef328d`, 66,645
rows); the dataset's user notes (December 2024), paras 4, 5, 7, 16 to 17 and
29 to 32; the ACNC Act 2012 s 25-5(5) table (Compilation No. 11, 5 December
2025); the CC BY 3.0 AU legal code, cl 4A and 4B; and ABN Lookup's page for ABN
50 169 561 394, which showed Australian Red Cross Society registered with the
ACNC from 3 December 2012, as the copy does. The CSV's subtype columns map to
the table's items: Advancing_Health 1, Advancing_Education 2,
Advancing_social_or_public_welfare 3, Advancing_Religion 4, Advancing_Culture 5,
Promoting_reconciliation 6, Promoting_or_protecting_human_rights 7,
Advancing_security_or_safety 8, Preventing_or_relieving_suffering_of_animals 9,
Advancing_natual_environment 10, Purposes_beneficial_to_the_general_public 11,
Promote_or_oppose_a_change_to_law 12 (para (l), shown in the table as advancing
public debate), HPC 13 and PBI 14.

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
