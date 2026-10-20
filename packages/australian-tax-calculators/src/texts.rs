//! Scope statements and sources, generated from `austaxcalc.calculations`
//! at the base revision so their wording is exactly the original.

pub const SCOPES: &[(&str, &str)] = &[
    (
        "gst",
        "One ordinary taxable supply at 10%, already classified by the operator. Excludes mixed supplies, exemptions, margin schemes, adjustments, tax invoice rounding across line items, input-credit entitlement and BAS preparation.",
    ),
    (
        "resident_tax",
        "Full-year Australian resident individual, ordinary taxable income only. Excludes minors' special rates, working holiday makers, offsets, Medicare levy, surcharge, HELP, PAYG credits and tax payable/refundable. Supply established whole-dollar taxable income; this is basic tax before offsets and levies.",
    ),
    (
        "capital_gains",
        "Australian resident individual with established ordinary gains and eligible 50% discount gains. Losses apply to non-discount gains first, then discount gains. Excludes asset classification, cost bases, eligibility decisions, collectables, personal-use assets, indexation, exemptions, rollovers, foreign residency and small-business concessions. The result is a net gain, not CGT payable.",
    ),
    (
        "fbt",
        "Ordinary taxable employer, established type 1 and type 2 taxable values for the year ended 31 March 2026. Excludes benefit valuation, exemptions, rebates and not-for-profit caps. Retains gross-up precision for the estimate, then presents the amounts to cents. The return_item figures follow the FBT return instead: items 14A and 14B rounded to the nearest dollar, as the ATO's own worked example does, item 15 as their sum and item 16 at 47%, before any optional rounding down to 5 cents.",
    ),
    (
        "depreciation",
        "First year only, ordinary tangible Division 40 asset first held on or after 10 May 2006. Established cost, effective life and taxable-use proportion. Days run from first use or installation ready for use. No second-element costs, disposals, changed life, pooling, immediate write-offs, special assets or Division 43 capital works. Does not determine an effective life from an asset description.",
    ),
    (
        "quarterly_sg",
        "One eligible employee, one employer and one complete quarter of 2025-26. Established ordinary time earnings and qualifying contributions for that quarter. Excludes eligibility and earnings classification, defined benefits, contribution caps, Norfolk Island transitional rates, special certificates, salary-sacrifice offsets, award entitlements and SGC. Does not test timeliness. From July 2026 Payday rules require a separate calculation.",
    ),
    (
        "payg_withholding",
        "One regular weekly, fortnightly or monthly payment of salary or wages made from 1 July 2026 to a payee who gave a TFN, on scale 1, 2, 3, 5 or 6 as the operator established from the payee's declarations. Earnings include allowances subject to withholding. Excludes scale 4, tax offsets, Medicare levy adjustments, study and training support loans, extra amounts for 53 or 27 pays, quarterly and bi-monthly pays, back payments, bonuses, termination and other schedules.",
    ),
    (
        "contribution_caps",
        "One individual's contributions received by all their funds in the income year, already classified by the operator as concessional or non-concessional, with no bring-forward period started in either of the 2 previous years. Total super balance is at the 30 June before the year; unused concessional cap is the established total of unexpired amounts from the previous 5 years. Excludes classification, fund acceptance and work-test rules, deductibility, the CGT cap, downsizer and other excluded contributions, excess concessional amounts counting as non-concessional, determinations, release elections, Division 293 tax and the transfer balance cap.",
    ),
    (
        "pension_minimum",
        "One account-based pension that started on or after 1 July 2007 and pays under SISR Schedule 7. Established account balance on 1 July, or on the commencement day in the first year, or the withdrawal benefit if higher, and the member's age on that day. Excludes pensions under the pre-2007 schedules, market-linked, lifetime and life expectancy pensions, the transition to retirement 10% maximum, commutations, death and reversion, and whether payments made meet the standard.",
    ),
    (
        "study_loan_repayment",
        "One individual's compulsory study and training support loan repayment for an income year from 2025-26, at the marginal rates the ATO publishes, from established whole-dollar repayment income: taxable income excluding assessable FHSS released amounts, plus reportable fringe benefits, total net investment loss, reportable super contributions and exempt foreign employment income, which the operator establishes. The formula amount only: the lesser of the marginal amount and 10% of repayment income, not limited to the loan balance. Excludes whether a debt exists, voluntary repayments, the overseas levy, allocation between loan types, indexation and the Schedule 8 withholding component. The ATO table prints the 2026-27 base of the 17% band rounded to $9,028 and its example 2 uses that figure; this worksheet applies the exact $9,028.35, so that example differs by 35 cents.",
    ),
    (
        "study_loan_withholding",
        "One regular weekly, fortnightly or monthly payment of salary or wages made from 1 July 2026 to a payee who gave a TFN declaration or withholding declaration saying they have a study and training support loan debt, on scale 1, 2, 3, 5 or 6 as the operator established. Scale 1 uses the Schedule 8 component for a payee who did not claim the tax-free threshold; scales 2, 3, 5 and 6 use the one for a payee who claimed it or is a foreign resident. Earnings include allowances subject to withholding. The result is the Schedule 1 amount plus the Schedule 8 component, each rounded to the dollar; the ATO's combined coefficients can differ by a dollar and it accepts either method. Excludes scale 4, a Medicare levy variation for a spouse or dependants, tax offsets, extra amounts for 53 or 27 pays, quarterly pays, lump sum termination payments, back payments, bonuses and other schedules.",
    ),
    (
        "medicare_levy",
        "Australian resident individual for the full income year, single with no spouse or dependants at any time in the year, not entitled to the seniors and pensioners tax offset, and not entitled to any full or half Medicare levy exemption. The operator must also establish whether s 9A applies: from 2024-25 it takes eligible lump sums in arrears out of taxable income when they are at least 10% of normal taxable income and its accrual-year tests are met. A person to whom s 9A applies is refused. Supply established whole-dollar taxable income. Applies the 2% rate and the low-income reduction: no levy at or below the threshold amount, and up to the phase-in limit no more than 10% of the excess over it. Excludes the family income reduction, SAPTO thresholds, exemptions, the Medicare levy surcharge, trustees and part-year residence.",
    ),
    (
        "low_income_tax_offset",
        "Individual who was an Australian resident at some time in the income year, not a trustee, and not a minor whose offset s 61-115(2) caps because Division 6AA of Part III of the 1936 Act applies to part of their income. Supply established whole-dollar taxable income. Gives the offset amount under ITAA 1997 s 61-115(1): $700 to $37,500, less 5% of the excess to $45,000, then $325 less 1.5% of the excess, nil above $66,667. The 1.5% band can leave half a cent, rounded half up. The offset is non-refundable; this worksheet does not apply it against tax payable or combine it with other offsets.",
    ),
    (
        "division_293",
        "One individual's Division 293 tax for an income year from established Division 293 income (income for surcharge purposes disregarding reportable super contributions) and low tax contributions for the corresponding financial year (low tax contributed amounts less excess concessional contributions). Taxable contributions are the lesser of the low tax contributions and the excess of the sum over $250,000; the tax is 15% of them. Excludes working out either input, defined benefit interests and deferred debt accounts, constitutionally protected funds, State higher level office holders and Commonwealth justices, releasing money from super, interest and the assessment itself.",
    ),
];

pub const SOURCES: &[(&str, &str)] = &[
    (
        "gst",
        "https://smallbusiness.taxsuperandyou.gov.au/goods-and-services-tax/fast-facts",
    ),
    (
        "resident_tax",
        "https://www.legislation.gov.au/C2004A03348/2026-07-01/2026-07-01/text/original/epub/OEBPS/document_1/document_1.html",
    ),
    (
        "capital_gains",
        "https://www.ato.gov.au/forms-and-instructions/capital-gains-tax-guide-2014/part-b-completing-the-capital-gains-section-of-your-tax-return/step-6-applying-current-year-capital-losses",
    ),
    (
        "fbt",
        "https://www.ato.gov.au/businesses-and-organisations/hiring-and-paying-your-workers/fringe-benefits-tax/calculating-your-fbt",
    ),
    (
        "depreciation",
        "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/depreciation-and-capital-expenses-and-allowances/general-depreciation-rules-capital-allowances/prime-cost-straight-line-and-diminishing-value-methods",
    ),
    (
        "quarterly_sg",
        "https://www.ato.gov.au/tax-rates-and-codes/key-superannuation-rates-and-thresholds/super-guarantee",
    ),
    (
        "payg_withholding",
        "https://www.ato.gov.au/tax-rates-and-codes/payg-withholding-schedule-1-statement-of-formulas-for-calculating-amounts-to-be-withheld",
    ),
    (
        "contribution_caps",
        "https://www.ato.gov.au/tax-rates-and-codes/key-superannuation-rates-and-thresholds/contributions-caps",
    ),
    (
        "pension_minimum",
        "https://www.ato.gov.au/individuals-and-families/super-for-individuals-and-families/self-managed-super-funds-smsf/paying-smsf-benefits/income-stream-pension-rules-and-payments",
    ),
    (
        "study_loan_repayment",
        "https://www.ato.gov.au/tax-rates-and-codes/study-and-training-support-loans-rates-and-repayment-thresholds",
    ),
    (
        "study_loan_withholding",
        "https://www.ato.gov.au/tax-rates-and-codes/schedule-8-statement-of-formulas-for-calculating-study-and-training-support-loans-components",
    ),
    (
        "medicare_levy",
        "https://www.legislation.gov.au/C2004A03351/2026-07-01/2026-07-01/text/original/epub/OEBPS/document_1/document_1.html",
    ),
    (
        "low_income_tax_offset",
        "https://www.legislation.gov.au/C2004A05138/2026-07-01/2026-07-01/text/original/epub/OEBPS/document_2/document_2.html",
    ),
    (
        "division_293",
        "https://www.legislation.gov.au/C2004A05138/2026-07-01/2026-07-01/text/original/epub/OEBPS/document_6/document_6.html",
    ),
];

pub const EXTRA_SOURCES: &[(&str, &[&str])] = &[
    (
        "fbt",
        &[
            "https://www.ato.gov.au/forms-and-instructions/fringe-benefits-tax-return-2026-instructions/fbt-return-2026-calculation-details-for-taxable-employers",
        ],
    ),
    (
        "study_loan_repayment",
        &["https://www.legislation.gov.au/C2026G00249/latest/text"],
    ),
    (
        "study_loan_withholding",
        &[
            "https://www.ato.gov.au/tax-rates-and-codes/payg-withholding-schedule-1-statement-of-formulas-for-calculating-amounts-to-be-withheld",
        ],
    ),
    (
        "medicare_levy",
        &[
            "https://www.ato.gov.au/individuals-and-families/medicare-and-private-health-insurance/medicare-levy/medicare-levy-reduction/medicare-levy-reduction-for-low-income-earners",
            "https://www.ato.gov.au/individuals-and-families/your-tax-return/instructions-to-complete-your-tax-return/mytax-instructions/2025/medicare-and-private-health-insurance/medicare-levy-reduction-or-exemption",
            "https://www.ato.gov.au/individuals-and-families/income-deductions-offsets-and-records/tax-offsets/lump-sum-payment-in-arrears-tax-offsets",
        ],
    ),
    (
        "low_income_tax_offset",
        &[
            "https://www.ato.gov.au/individuals-and-families/income-deductions-offsets-and-records/tax-offsets/low-income-tax-offset",
        ],
    ),
    (
        "division_293",
        &[
            "https://www.legislation.gov.au/C2013A00087/2013-06-28/2013-06-28/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.ato.gov.au/tax-rates-and-codes/key-superannuation-rates-and-thresholds/division-293-tax",
            "https://www.ato.gov.au/individuals-and-families/super-for-individuals-and-families/super/growing-and-keeping-track-of-your-super/caps-limits-and-tax-on-super-contributions/division-293-tax-on-concessional-contributions-by-high-income-earners",
        ],
    ),
];
