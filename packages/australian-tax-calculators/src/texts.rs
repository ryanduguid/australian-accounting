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
        "main_residence",
        "An individual who was an Australian resident when the CGT event happened, disposing of (CGT event A1) one dwelling, with any adjacent land used mainly for private or domestic purposes with it and of up to 2 hectares in total, acquired on or after 20 September 1985 and held as the same ownership interest throughout, with the disposal in the 2024-25, 2025-26 or 2026-27 income year, before the changes from 1 July 2027. The operator establishes the gain (cost base) or loss (reduced cost base) on the whole interest held, before any exemption or discount; the ownership period's first and last days; the days the dwelling was their main residence, including any treated as such from acquisition under s 118-135; the days it or part of it produced assessable income where interest would have been deductible, with the part of the floor area used; the days covered by a choice under s 118-145; and when a new home that became their main residence was acquired. Every day is counted, both ends included. Each day is taxable in full, in part (the floor area used while living there, s 118-190) or not at all. A choice period keeps the dwelling exempt, except a part used to produce income before the move, which stays taxable; income use is covered for 6 years per absence, to the sixth anniversary of its first day inclusive, as the ATO counts it, and counted across breaks within the same absence; later income-producing days are taxable. Under s 118-140 the last 6 months of ownership, from the same day of the month 6 months earlier (or the first of the next month where that day does not exist), or from the new home's acquisition if later, are exempt when the dwelling was the main residence for a continuous 3 months in the 12 months before ownership ended, which start the day after the same date a year earlier, and produced no income in that time while not the main residence. Under s 118-192, when income use first began after 20 August 1996, a full exemption would have applied just before, and the dwelling is only partly exempt because of that use, the ownership period starts on that day and the gain or loss must be worked out from the market value then. The assessable part is the gain or loss times the taxable days over the days in the ownership period (s 118-185). Excludes deciding residence, main residence status, interest deductibility, floor area, the choice itself and market value; inherited dwellings (s 118-195); a spouse or dependant child with a different main residence (s 118-170); building, repairing or renovating (s 118-150) and destroyed dwellings (s 118-155); adjacent land over 2 hectares or sold separately; compulsory acquisitions; marriage or relationship breakdown rollovers; granny flat arrangements; another dwelling treated as the main residence during a choice period except under s 118-140; the CGT discount and its 12-month test; and small business concessions. Refuses income use of part of the dwelling that changed, paused or stopped between its start and a move that starts a choice period, and a partial exemption with causes besides income use where s 118-192 would otherwise apply.",
    ),
    (
        "borrowing_expenses",
        "One loan's borrowing expenses under ITAA 1997 s 25-25 for one income year from 2024-25 to 2026-27, before s 26-155 quarantines residential rental deductions from 2027-28. The operator establishes the expenditure the taxpayer incurred for borrowing the money, all in the income year the loan began; the extent the borrowed money is used to produce assessable income, the same throughout the loan; the first day the money was borrowed; the last day of the period the original loan contract specifies; any repayment day; and the income-producing share of the taxpayer's other borrowing expenses incurred in that year. Under s 25-25(6), when the money is used to produce income and this loan's income-producing share plus those other amounts is $100 or less, that share is deducted in the year incurred and later years are refused. Otherwise the period of the loan is the shortest of the contract's period, the period to the repayment day and 5 years from the first day borrowed (s 25-25(5)). Each income year's maximum deduction is the remaining expenditure over the days left in that period, times those days in the year, as determined at the end of the year, so the year of repayment takes all that remains (s 25-25(4)); each maximum is rounded to cents and carried forward at that amount, as the ATO's examples are, and the deduction is the maximum times the income-producing share (s 25-25(2) and (3)). Every day is counted, both ends included, as the ATO and the Library count: 5 years from 3 July end on 2 July. Excludes deciding which costs are borrowing expenses and the money's use; interest; refinancing, redraws and changes in use; expenditure incurred before the income year the loan began; a loan first drawn on 29 February; leisure facilities such as holiday homes unless s 26-50(3) applies at all times in the year; commercial debt forgiveness (Subdivision 245-E); private company loans (Division 7A); and s 26-155.",
    ),
    (
        "fbt",
        "Ordinary taxable employer, established type 1 and type 2 taxable values for the year ended 31 March 2026 or 2027, both at the 47% FBT rate and the 2.0802 and 1.8868 gross-up rates. Excludes benefit valuation, exemptions, rebates and not-for-profit caps. Retains gross-up precision for the estimate, then presents the amounts to cents. The return_item figures follow the FBT return instead: items 14A and 14B rounded to the nearest dollar, as the ATO's own worked example does, item 15 as their sum and item 16 at 47%, before any optional rounding down to 5 cents. For 2027 the items follow the FBT return 2026 instructions until the ATO publishes the 2027 return.",
    ),
    (
        "fbt_car_statutory",
        "One car held by an ordinary taxable employer, as the only provider and for a single holding period, and used or available for an employee's private use in the FBT year ended 31 March 2026 or 2027, with no commitment to provide it made before 7.30 pm AEST on 10 May 2011, so the 20% statutory percentage applies. The operator establishes the base value under s 9(2) as the employer's guide (7.8.1) describes: cost_price holds the part the one-third reduction applies to (where the provider or an associate owned the car at the earliest holding time, its cost price to them with dealer delivery and non-business accessories fitted at acquisition; otherwise its leased car value then, which is the lessor's cost price only if the lease began then and the lessor bought the car at or about that time, and otherwise its arm's-length purchase price), and later_accessories holds non-business accessories fitted after the earliest holding time and still fitted while the car was held in the year, which it never reduces. one_third_reduction is true only from the FBT year starting after the car has been held for 4 years. Days used or available for private use, recipient's payments and whether the electric cars exemption applies are established facts. Excludes deciding whether a car benefit or exemption arises, the base value itself, arm's-length deeming, dealer demonstrator cars, the type 1 or type 2 choice, gross-up and FBT payable. Only the gross taxable value is rounded to cents, before the recipient's payment, which cannot take the taxable value below nil; an unused excess is shown and reduces no other benefit. An exempt electric car's taxable value is nil, and taxable_value_before_exemption is the notional value its reportable fringe benefits amount is worked out from, before gross-up.",
    ),
    (
        "fbt_car_operating_cost",
        "One car held by an ordinary taxable employer, as the only provider and for a single holding period, to provide car fringe benefits in the FBT year ended 31 March 2026 or 2027, valued under a written election for the operating cost method. The operator establishes the GST-inclusive car expenses for the holding period under s 10(3) (repairs, maintenance and fuel, and registration and insurance attributable to the period, including those the employee paid, but not insured repair expenses, an owned car's registration and insurance incurred before the provider owned it, or expenses the lessor bears under the lease), the lease charges for a leased car, or for an owned car acquired on or after 10 May 2006 (a hire purchase car is owned) its s 12 depreciated value at the start of the FBT year (cost price less earlier FBT deemed depreciation, or its cost price if acquired in the year) and the effective life in effect when the provider last became its owner, the days in the holding period the car was owned or leased, the business use percentage from log book and odometer records (0 without them), recipient's payments, the same car's statutory formula value and whether the electric cars exemption applies. Deemed depreciation at 200% over the effective life (25% for a car with an 8-year life) and deemed interest at the year's benchmark rate are apportioned to the days held. Non-business accessories fitted before the year belong in depreciated_value; accessories fitted during the year, a car both owned and leased in the year, a car neither owned nor leased by the provider, cars acquired before 10 May 2006, the business use estimate itself, arm's-length deeming, the type 1 or type 2 choice, gross-up and FBT payable are excluded. Only the gross taxable value is rounded to cents, before the recipient's payment, which cannot take the taxable value below nil; an unused excess is shown and reduces no other benefit. Under s 10(5) the result is the lower of this value and the statutory formula value supplied for the same car and holding period; gross_taxable_value, employee_contribution_applied and excess_employee_contribution are the operating cost method's figures, and where the statutory formula value is lower, fbt-car-statutory's applied and excess figures apply. An exempt electric car's taxable value is nil, and taxable_value_before_exemption is the notional value its reportable fringe benefits amount is worked out from, before gross-up.",
    ),
    (
        "depreciation",
        "Ordinary tangible Division 40 asset first held on or after 10 May 2006, in the income year its start time occurs or, given its established opening adjustable value for the year, a later year. Established cost, effective life and taxable-use proportion. Days run from first use or installation ready for use. No second-element costs after the start year, disposals, changed life or other s 40-75(2) adjustment, pooling, immediate write-offs, special assets or Division 43 capital works. Does not determine an effective life from an asset description.",
    ),
    (
        "quarterly_sg",
        "One eligible employee, one employer and one complete quarter of 2025-26. Established ordinary time earnings and qualifying contributions for that quarter. Excludes eligibility and earnings classification, defined benefits, contribution caps, Norfolk Island transitional rates, special certificates, salary-sacrifice offsets, award entitlements and SGC. Does not test timeliness. From July 2026 Payday rules require a separate calculation.",
    ),
    (
        "payg_withholding",
        "One regular weekly, fortnightly, monthly or quarterly payment of salary or wages made from 1 July 2024, in the income year given, on the scale the operator established from the payee's declarations: 1, 2, 3, 5 or 6 for a payee who gave a TFN, or 4 for one who did not, as a resident or foreign resident. The year selects the Schedule 1 coefficients and rules for payments made in it. Earnings include allowances subject to withholding. When given, the yearly tax offsets claimed on a Withholding declaration reduce scales 2, 5 and 6, the Medicare levy adjustment claimed on a Medicare levy variation declaration for a spouse or dependent children reduces scales 2 and 6, and the additional amount a payee asks for in a year of 53 weekly or 27 fortnightly pays is added. Reductions that exceed the amount withheld before them are refused. Excludes a Medicare levy variation that increases withholding, study and training support loans, bi-monthly, daily and casual pays, back payments, bonuses, termination and other schedules.",
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
    (
        "division_296",
        "An estimate of one individual's Division 296 tax (ITAA 1997 Subdivision 296-B; Superannuation (Building a Stronger and Fairer Super System) Imposition Act 2026 s 5) for the 2026-27 income year, its first. The operator establishes the individual's total superannuation balance at the end of 30 June 2027 for Division 296, including reversionary retirement phase interests and Division 296 excluded interests and disregarding any LRBA amount (s 296-260) and interests in foreign superannuation funds (s 307-230(1)(a)); their total superannuation earnings for the year (s 296-55), the total of the relevant superannuation earnings for each of their interests and for each interest supporting a superannuation income stream they receive as a retirement phase recipient because of another person's death (s 296-55(1)), which may be negative, with foreign fund earnings, Division 296 excluded interests and prescribed amounts taken as nil; and whether they were a child recipient of a superannuation income stream at any time in the year (s 296-20), had a structured settlement contribution made in respect of them in that or an earlier year (s 296-25), or died on or before 30 June 2027 (Income Tax (Transitional Provisions) Act 1997 s 296-1(3)), each of which means no tax. For 2026-27 only the balance at the end of the year counts, so the worksheet takes no opening balance (Income Tax (Transitional Provisions) Act 1997 s 296-1(2)). Tax arises only when that balance is greater than the $3,000,000 large superannuation balance threshold and the earnings are greater than nil (s 296-40(1)); otherwise the tax is nil and liability gives the reason. The earnings amounts follow ss 296-40 and 296-45 even when an exception removes the liability, so the very large superannuation balance earnings component can be negative, when it is reported as null, as results carry no negative amounts, and after death by 30 June 2027 the balance is taken to be nil (s 296-50). The shares of the balance above $3,000,000 and above the $10,000,000 very large superannuation balance threshold are each a percentage rounded to 2 decimal places, half up (ss 296-40(3) and 296-45(3)), so a balance just above a threshold can give 0.00. Taxable superannuation earnings are the earnings times the first percentage and the very large superannuation balance earnings component the earnings times the second; the tax is 15% of the first plus 10% of the second (Imposition Act s 5). Excludes working out the balance, fund earnings and their attribution (ss 296-60 to 296-75 and the regulations); defined benefit interests and other interests whose earnings use s 296-70, including income streams with no account balance, and the deferral of tax on a defined benefit interest to a debt account; temporary residents who depart Australia (ss 296-195 to 296-205); assessment, payment, release authorities and interest charges; and income years from 2027-28, whose indexed thresholds are not yet known.",
    ),
    (
        "termination_payment",
        "The tax-free part of a genuine redundancy or early retirement scheme payment and the PAYG withholding from one life benefit employment termination payment (ETP) under Schedule 11, made in 2026-27 to a resident employee who gave a TFN and is not a working holiday maker, within 12 months of the termination. The operator establishes the redundancy payment as a genuine redundancy payment (ITAA 1997 s 83-175, including dismissal before pension age) or an early retirement scheme payment (s 83-180), above what voluntary termination would have paid; the completed years of service (s 83-170(3)); the rest of the ETP's taxable component, split into excluded payments (s 82-10(6)) and non-excluded payments; and the payer's other taxable payments to the employee in the year. The redundancy payment is tax-free up to $13,598 plus $6,801 for each completed year of service (s 83-170) and the rest of it is an excluded ETP part. The excluded part is taken first and capped at the $270,000 ETP cap; the non-excluded part is capped at the smaller of the ETP cap less the excluded part within it and the whole-of-income cap, $180,000 less the other taxable payments, not below nil (s 82-10(4) and (7); Schedule 11). Withholding is 17% up to the cap if the employee is their preservation age or older on 30 June 2027 (SISR reg 6.01(2)), 32% if younger, and 47% above the cap, each part rounded to the nearest dollar, 50 cents up. Excludes classifying the payments, death benefit ETPs, ETPs with a tax-free component (an invalidity segment or employment before 1 July 1983), earlier ETPs to the employee in the income year or for the same termination, unused annual and long service leave payments, delayed termination payments, payees without a TFN, foreign residents, working holiday makers and the income tax assessed on the ETP.",
    ),
    (
        "schedule_5_method_a",
        "Withholding under Schedule 5 (NAT 3348) Method A from one regular weekly, fortnightly or monthly pay of salary or wages that includes back payments (including lump sums in arrears), commissions, bonuses or similar payments, made from 1 July 2024 in the income year given, to a payee who gave a TFN, on scale 1, 2, 3, 5 or 6 as the operator established from the payee's declarations. The year selects the Schedule 1 coefficients, as payg-withholding uses them. The operator establishes the pay's gross earnings excluding the additional payments, the additional payments in it and whether the payee declared a study and training support loan debt, when each table amount includes the Schedule 8 component, as Schedule 5 allows, for the years study-loan-withholding supports. Cents are ignored in the earnings (so a monthly amount ending in 33 cents gains no cent) and in the additional payments divided by 52 weekly, 26 fortnightly or 12 monthly pay periods, with no adjustment for 53 or 27 pays, or, for a commission, bonus or similar payment for a defined period of less than 12 months, by the pay periods it relates to, which the operator chooses. The extra withheld from the earnings plus that share, times the same pay periods, is limited to 47% of the additional payments, with cents ignored and a negative result taken as nil; the pay's withholding adds it to the earnings' amount. A payment for a single pay period belongs in that pay under Schedule 1, so 1 pay period is refused. Excludes working holiday makers (Schedule 15), payees who gave no TFN (scale 4), withholding variations, tax offsets, Medicare levy adjustments, superannuation income streams, employment termination and unused leave payments, and other schedules.",
    ),
    (
        "schedule_5_method_b_i",
        "Withholding under Schedule 5 (NAT 3348) Method B(i) from one regular weekly, fortnightly or monthly pay of salary or wages that includes a back payment for earlier pay periods in the current financial year, made from 1 July 2024 in the income year given, to a payee who gave a TFN, on scale 1, 2, 3, 5 or 6 as the operator established from the payee's declarations. The year selects the Schedule 1 coefficients, as payg-withholding uses them. For each earlier pay period the back payment applies to, the operator establishes the normal earnings paid for it (including back payments already applied to it by Method B(i)), the part of the back payment for it and the amount withheld from it; and the current pay's gross earnings excluding additional payments and whether the payee declared a study and training support loan debt, when each table amount includes the Schedule 8 component, as Schedule 5 allows, for the years study-loan-withholding supports, and the amounts withheld include it too. Each period's withholding is recalculated on its earnings plus its part of the back payment, less the amount withheld from it; the total is the withholding on the back payment, a negative total taken as nil, with no 47% limit, and the pay's withholding adds it to the current earnings' amount. A financial year holds at most 53 weekly, 27 fortnightly or 12 monthly pays, so at most 52, 26 or 11 earlier periods. When Method B(ii) applies to another additional payment in the same pay, count the earnings' amount once. Excludes back payments for a prior financial year and commissions, bonuses and similar payments (Method A or B(ii)), working holiday makers (Schedule 15), payees who gave no TFN (scale 4), withholding variations, tax offsets, Medicare levy adjustments, superannuation income streams and other schedules.",
    ),
    (
        "schedule_5_method_b_ii",
        "Withholding under Schedule 5 (NAT 3348) Method B(ii) from one regular weekly, fortnightly or monthly pay of salary or wages that includes a back payment for a prior financial year, or a commission, bonus or similar payment that does not relate to a single pay period, made from 1 July 2024 in the income year given, to a payee who gave a TFN, on scale 1, 2, 3, 5 or 6 as the operator established from the payee's declarations. The year selects the Schedule 1 coefficients, as payg-withholding uses them. The operator establishes the normal earnings paid in the financial year to date, including the current pay, plus current-year back payments for which Method B(i) was used; the pay periods to date, including the current one; the additional payment in this pay; additional payments made earlier in the year for which Method B(ii) was used and the amounts withheld from them; the current pay's gross earnings excluding additional payments; and whether the payee declared a study and training support loan debt, when each table amount includes the Schedule 8 component, as Schedule 5 allows, for the years study-loan-withholding supports, and the earlier amounts withheld include it too. Average total earnings are the earnings to date divided by the pay periods to date, cents ignored. All the year's Method B(ii) additional payments are divided by 52 weekly, 26 fortnightly or 12 monthly pay periods, cents ignored, with no adjustment for 53 or 27 pays; the extra withheld from the average plus that share, times those pay periods, less the earlier amounts withheld, is limited to 47% of this additional payment, with cents ignored and a negative result taken as nil; the pay's withholding adds it to the current earnings' amount. When Method B(i) applies to a back payment in the same pay, count the earnings' amount once. Excludes working holiday makers (Schedule 15), payees who gave no TFN (scale 4), withholding variations, tax offsets, Medicare levy adjustments, superannuation income streams, employment termination and unused leave payments, and other schedules.",
    ),
    (
        "working_holiday_maker_withholding",
        "Withholding under Schedule 15 from one payment to a working holiday maker, made from 1 July 2024 in the income year given: an individual who holds a working holiday visa (subclass 417), a work and holiday visa (subclass 462) or a bridging visa that Schedule 15 covers, as the operator established. The payment is the total for the pay period, including allowances, back payments, commissions, bonuses and other payments Schedule 15 covers; for an employment termination payment, only its taxable component. The operator establishes the payments made to the payee in the income year before this one, whether the employer is registered with the ATO to withhold at working holiday maker rates, and whether the payee quoted a TFN or, within the last 28 days, advised that they applied for one or made an enquiry with the ATO. A registered employer withholds the payment, cents ignored, times the Table A rate for the column the payments before it select: 15% for less than $45,001, 30% from $45,001 to $135,000, 37% from $135,001 to $190,000 and 45% for more than $190,000, rounded to the dollar with 50 cents up, or with cents ignored at 45%; payments before of $135,000.01 to $135,000.99, which no column covers, are refused. Without a TFN any employer withholds 45% of the payment, cents ignored. An unregistered employer withholds at foreign resident rates, the Schedule 1 scale 3 amount for a weekly, fortnightly or monthly pay, as payg-withholding gives it. Every payment needs a weekly, fortnightly or monthly pay period. Excludes a payee who claimed an exemption from quoting a TFN, Medicare levy adjustments and tax offsets, which do not reduce this withholding, the Seasonal Worker Programme, Pacific Labour Scheme and Pacific Australia Labour Mobility scheme, withholding variations, the employer's registration and the payee's visa, residency and annual return.",
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
        "main_residence",
        "https://www.ato.gov.au/individuals-and-families/investments-and-assets/capital-gains-tax/property-and-capital-gains-tax/your-main-residence-home/treating-former-home-as-main-residence",
    ),
    (
        "borrowing_expenses",
        "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_1/document_1.html",
    ),
    (
        "fbt",
        "https://www.ato.gov.au/businesses-and-organisations/hiring-and-paying-your-workers/fringe-benefits-tax/calculating-your-fbt",
    ),
    (
        "fbt_car_statutory",
        "https://www.ato.gov.au/law/view/document?DocID=SAV/FBTGEMP/00008",
    ),
    (
        "fbt_car_operating_cost",
        "https://www.ato.gov.au/law/view/document?DocID=SAV/FBTGEMP/00008",
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
        "https://www.ato.gov.au/tax-rates-and-codes/study-and-training-loan-minimum-repayment-income-and-rates",
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
    (
        "division_296",
        "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_6/document_6.html",
    ),
    (
        "termination_payment",
        "https://www.ato.gov.au/tax-rates-and-codes/payg-withholding-schedule-11-tax-table-for-employment-termination-payments",
    ),
    ("schedule_5_method_a", SCHEDULE_5_METHODS),
    ("schedule_5_method_b_i", SCHEDULE_5_METHODS),
    ("schedule_5_method_b_ii", SCHEDULE_5_METHODS),
    (
        "working_holiday_maker_withholding",
        "https://www.ato.gov.au/tax-rates-and-codes/schedule-15-tax-table-for-working-holiday-makers",
    ),
];

/// Schedule 5 (NAT 3348) for payments from 1 July 2026: its methods, and the
/// earlier schedules and worked examples the 3 method worksheets also cite.
const SCHEDULE_5_METHODS: &str = "https://www.ato.gov.au/tax-rates-and-codes/schedule-5-tax-table-for-back-payments-commissions-bonuses-and-similar-payments/working-out-the-withholding-amount";
const SCHEDULE_5_EXTRA: &[&str] = &[
    "https://www.ato.gov.au/tax-rates-and-codes/schedule-5-tax-table-for-back-payments-commissions-bonuses-and-similar-payments/withholding-amounts-examples",
    "https://www.ato.gov.au/tax-rates-and-codes/schedule-5-tax-table-01-july-2025-to-30-june-2026/working-out-the-withholding-amount",
    "https://www.ato.gov.au/tax-rates-and-codes/payg-withholding-schedule-5-tax-table-01-july-2024-to-30-june-2025/working-out-the-withholding-amount",
];

pub const EXTRA_SOURCES: &[(&str, &[&str])] = &[
    (
        "depreciation",
        &[
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/40-70",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/40-72",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/40-75",
        ],
    ),
    (
        "main_residence",
        &[
            "https://www.ato.gov.au/individuals-and-families/investments-and-assets/capital-gains-tax/property-and-capital-gains-tax/your-main-residence-home/using-your-home-for-rental-or-business",
        ],
    ),
    (
        "borrowing_expenses",
        &[
            "https://www.ato.gov.au/individuals-and-families/investments-and-assets/property-and-land/residential-rental-properties/rental-expenses/borrowing-expenses",
        ],
    ),
    (
        "fbt",
        &[
            "https://www.ato.gov.au/forms-and-instructions/fringe-benefits-tax-return-2026-instructions/fbt-return-2026-calculation-details-for-taxable-employers",
            "https://www.ato.gov.au/tax-rates-and-codes/fringe-benefits-tax-rates-and-thresholds",
        ],
    ),
    (
        "fbt_car_statutory",
        &[
            "https://www.ato.gov.au/forms-and-instructions/fringe-benefits-tax-return-2026-instructions/item-23-fringe-benefit-categories-for-fbt-return-2026",
            "https://www.ato.gov.au/businesses-and-organisations/hiring-and-paying-your-workers/fringe-benefits-tax/types-of-fringe-benefits/fbt-on-cars-other-vehicles-parking-and-tolls/electric-cars-exemption",
        ],
    ),
    (
        "fbt_car_operating_cost",
        &[
            "https://www.ato.gov.au/forms-and-instructions/fringe-benefits-tax-return-2026-instructions/item-23-fringe-benefit-categories-for-fbt-return-2026",
            "https://www.ato.gov.au/tax-rates-and-codes/fringe-benefits-tax-rates-and-thresholds",
            "https://www.ato.gov.au/businesses-and-organisations/hiring-and-paying-your-workers/fringe-benefits-tax/types-of-fringe-benefits/fbt-on-cars-other-vehicles-parking-and-tolls/electric-cars-exemption",
        ],
    ),
    (
        "payg_withholding",
        &[
            "https://www.ato.gov.au/tax-rates-and-codes/schedule-1-tax-table-01-july-2024-to-30-june-2026",
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
            "https://www.ato.gov.au/individuals-and-families/income-deductions-offsets-and-records/income-you-must-declare/lump-sum-payment-in-arrears",
            "https://www.legislation.gov.au/C2026A00058/asmade/text",
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
    (
        "division_296",
        &[
            "https://www.legislation.gov.au/C2026A00009/asmade",
            "https://www.legislation.gov.au/C2026A00008/asmade",
            "https://www.ato.gov.au/individuals-and-families/super-for-individuals-and-families/super/growing-and-keeping-track-of-your-super/caps-limits-and-tax-on-super-contributions/division-296-tax/division-296-tax-on-large-super-balances",
        ],
    ),
    (
        "termination_payment",
        &[
            "https://www.ato.gov.au/tax-rates-and-codes/key-superannuation-rates-and-thresholds/employment-termination-payments",
            "https://www.ato.gov.au/businesses-and-organisations/hiring-and-paying-your-workers/engaging-a-worker/when-a-worker-leaves-your-business/taxation-of-termination-payments/applying-the-etp-caps",
            "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_3/document_3.html",
            "https://www.legislation.gov.au/F1996B00580/2026-07-01/2026-07-01/text/original/epub/OEBPS/document_2/document_2.html",
        ],
    ),
    ("schedule_5_method_a", SCHEDULE_5_EXTRA),
    ("schedule_5_method_b_i", SCHEDULE_5_EXTRA),
    ("schedule_5_method_b_ii", SCHEDULE_5_EXTRA),
    (
        "working_holiday_maker_withholding",
        &[
            "https://www.ato.gov.au/tax-rates-and-codes/schedule-15-tax-table-for-working-holiday-makers-01-july-2024-to-30-june-2026",
            "https://www.ato.gov.au/tax-rates-and-codes/payg-withholding-schedule-1-statement-of-formulas-for-calculating-amounts-to-be-withheld",
            "https://www.ato.gov.au/tax-rates-and-codes/schedule-1-tax-table-01-july-2024-to-30-june-2026",
        ],
    ),
];

/// The car worksheets' reportable fringe benefits note, read from the ATO's
/// reportable fringe benefits page on 30 October 2026. It follows the 2
/// warnings every worksheet carries.
const CAR_REPORTABLE_NOTE: &str = "reportable_taxable_value is this car benefit's taxable value before any exemption, not a reportable fringe benefits amount. The employee's reportable fringe benefits amount is the total reportable taxable value of all their reportable fringe benefits for the FBT year multiplied by the type 2 (lower) gross-up rate, when that total exceeds $2,000. A car benefit from the private use of a pooled or shared car is not reportable. This worksheet determines neither.";

/// Warnings added after the original's 2, by worksheet.
pub const EXTRA_WARNINGS: &[(&str, &[&str])] = &[
    ("fbt_car_statutory", &[CAR_REPORTABLE_NOTE]),
    ("fbt_car_operating_cost", &[CAR_REPORTABLE_NOTE]),
    (
        "division_296",
        &[
            "An estimate, not the tax payable: the Commissioner assesses Division 296 tax from the balances and earnings superannuation funds report, and the notice of assessment sets the amount.",
            "Each amount is worked out exactly and rounded half up to cents only for display, so a displayed tax can differ by a cent from its rate applied to the displayed earnings, and the displayed total by a cent from the sum of the displayed taxes.",
        ],
    ),
    (
        "termination_payment",
        &[
            "Withholding, not the tax payable: on assessment the taxable component is assessable income, a tax offset limits the rate on the part within the cap to 15% or 30% (ITAA 1997 s 82-10(3)), and the Medicare levy applies; the whole-of-income cap then uses taxable income for the whole year, including the excluded part's taxable component (s 82-10(5) and (7)), so the tax can differ from the amount withheld.",
        ],
    ),
];

/// A `medicare_levy` warning for a year whose thresholds the ATO had not
/// published when they were read from the Act.
pub const MEDICARE_YEAR_WARNINGS: &[(&str, &str)] = &[(
    "2026-27",
    "2026-27 uses the threshold amount and phase-in limit Medicare Levy Act 1986 s 3(1) sets for 2025-26 and later years (Act No. 58, 2026, Sch 5 item 14). The ATO had not published 2026-27 thresholds when this was checked on 14 November 2026. That Act, assented on 30 June 2026, raised the figures for 2025-26, so a later Act can raise them for 2026-27 and change this result.",
)];

/// Every refusal code and the failure it names. A code stays when a message is
/// reworded and never moves to a different failure; a new failure gets a new code.
/// A code carries no message text, parameter value or source location.
pub const REFUSAL_CODES: &[(&str, &str)] = &[
    (
        "borrowing.loan_date_invalid",
        "A loan date is neither null nor a date on or after the start of the loan.",
    ),
    (
        "borrowing.loan_period_ended",
        "The period of the loan ended before the income year began.",
    ),
    (
        "borrowing.loan_start_29_february",
        "A loan first drawn on 29 February, for which no source settles when 5 years end.",
    ),
    (
        "borrowing.loan_starts_after_year",
        "The loan starts after the income year ended.",
    ),
    (
        "borrowing.s25_25_6_not_spread",
        "s 25-25(6) applied in the income year the expenditure was incurred, so nothing spreads to this year.",
    ),
    (
        "borrowing.s26_50_denied",
        "s 26-50 denies the deduction for a leisure facility, or the worksheet does not cover a part-year exception.",
    ),
    ("command.unknown", "The command does not exist."),
    (
        "decimal.division_by_zero",
        "A worksheet step divided by zero.",
    ),
    (
        "decimal.invalid_operation",
        "A worksheet step is undefined, such as an order comparison with NaN.",
    ),
    (
        "decimal.overflow",
        "A worksheet step exceeded the decimal range.",
    ),
    (
        "depreciation.precision_exceeded",
        "An effective life's significant digits and the taxable use's decimal places total more than 22.",
    ),
    (
        "derivation.unsupported",
        "derive was asked to explain a command whose amounts it does not derive.",
    ),
    (
        "fbt_car.cost_basis_conflict",
        "A car has neither or both of a depreciated value and lease charges.",
    ),
    (
        "input.age_invalid",
        "An age is not a whole number of years from 0 to 150.",
    ),
    (
        "input.amount_invalid",
        "An amount is not a finite non-negative decimal of at most 2dp and 1e12.",
    ),
    (
        "input.amount_not_whole_dollars",
        "An amount the worksheet reads in whole dollars has cents.",
    ),
    (
        "input.back_payment_periods_invalid",
        "Back payment periods are not a list of 1 to the most earlier pays a year holds, each with exactly earnings, back_payment and withheld as decimal strings.",
    ),
    (
        "input.children_invalid",
        "A number of dependent children is not a JSON integer from 0.",
    ),
    (
        "input.date_invalid",
        "A date is not YYYY-MM-DD from 1900 to 2100.",
    ),
    (
        "input.day_count_invalid",
        "A count of days is outside the range for the year.",
    ),
    (
        "input.decimal_too_long",
        "A decimal string exceeds 8192 bytes.",
    ),
    (
        "input.decimal_unreadable",
        "Decimal text cannot be read as a number.",
    ),
    (
        "input.depreciation_method_invalid",
        "The method is neither prime_cost nor diminishing_value.",
    ),
    (
        "input.effective_life_invalid",
        "An effective life is outside 0.01 to 1000 years.",
    ),
    (
        "input.field_unknown",
        "A field the worksheet does not take was given (division-296, depreciation, payg-withholding, termination-payment, the Schedule 5 methods and working-holiday-maker-withholding only).",
    ),
    ("input.flag_invalid", "A flag is not a JSON boolean."),
    ("input.fraction_invalid", "A fraction is outside 0 to 1."),
    (
        "input.integer_too_long",
        "A JSON integer has more than 4,300 digits.",
    ),
    ("input.invalid_json", "The input is not JSON."),
    ("input.kind_invalid", "A worksheet kind is not a string."),
    ("input.missing_field", "A required argument is absent."),
    ("input.not_object", "The JSON input is not an object."),
    (
        "input.opening_value_above_cost",
        "An opening adjustable value is more than the asset's cost.",
    ),
    (
        "input.pay_period_invalid",
        "The pay period is not weekly, fortnightly or monthly (or quarterly, for payg-withholding).",
    ),
    (
        "input.pay_periods_invalid",
        "A count of pay periods is outside the range the method allows for the pay period.",
    ),
    ("input.quarter_invalid", "A quarter is not 1, 2, 3 or 4."),
    (
        "input.signed_amount_invalid",
        "An amount that may be negative is not a finite decimal of at most 2dp within 1e12 of nil.",
    ),
    (
        "input.tax_scale_unsupported",
        "The withholding scale is not 1, 2, 3, 5 or 6 (or 4, for payg-withholding), or has no table for the period.",
    ),
    ("input.too_large", "The JSON input exceeds 8 MiB."),
    ("input.unreadable", "Standard input could not be read."),
    (
        "input.years_of_service_invalid",
        "A count of completed years of service is not a whole number from 0 to 150.",
    ),
    (
        "main_residence.absence_choice_period_invalid",
        "An absence choice period does not start the day after a lived period ends, or holds a lived day.",
    ),
    (
        "main_residence.first_income_use_rule_applies",
        "The home first used to produce income rule (s 118-192) applies, and the market value basis was not supplied.",
    ),
    (
        "main_residence.first_income_use_rule_does_not_apply",
        "The market value basis was supplied, and the home first used to produce income rule does not apply.",
    ),
    (
        "main_residence.floor_area_invalid",
        "A floor area is not a decimal string above 0 and at most 1, to 4 places.",
    ),
    (
        "main_residence.gain_and_loss_both_supplied",
        "Both a capital gain and a capital loss were supplied.",
    ),
    (
        "main_residence.income_use_changed_before_absence",
        "Income use changed or stopped before the move that started an absence choice period.",
    ),
    (
        "main_residence.income_use_started_20_august_1996",
        "Income use first started on 20 August 1996, when the market value rule's 7.30 pm start applies.",
    ),
    (
        "main_residence.new_home_date_invalid",
        "A new home's date is neither null nor a date no later than the end of ownership.",
    ),
    (
        "main_residence.ownership_dates_reversed",
        "Ownership ends before it starts.",
    ),
    (
        "main_residence.ownership_ends_before_year",
        "Ownership ends before the income year began.",
    ),
    (
        "main_residence.ownership_starts_after_year",
        "Ownership starts after the income year ended.",
    ),
    (
        "main_residence.partial_exemption_other_cause",
        "The dwelling is only partly exempt for a reason besides income use as well as that use.",
    ),
    (
        "main_residence.period_date_invalid",
        "A period's start or end is not a YYYY-MM-DD date from 1900 to 2100.",
    ),
    (
        "main_residence.period_outside_ownership",
        "A period runs backwards or leaves the ownership period.",
    ),
    (
        "main_residence.period_shape_invalid",
        "A period is not an object with exactly the required keys.",
    ),
    (
        "main_residence.periods_not_a_list",
        "A list of periods is not a list.",
    ),
    (
        "main_residence.periods_overlap",
        "Periods in a list overlap.",
    ),
    (
        "main_residence.periods_too_many",
        "A list holds more than 1,000 periods.",
    ),
    (
        "main_residence.pre_cgt_dwelling",
        "The dwelling was acquired before 20 September 1985.",
    ),
    (
        "medicare.s9a_adjustment_out_of_scope",
        "Medicare Levy Act 1986 s 9A applies, and the worksheet does not make that adjustment.",
    ),
    (
        "payg.extra_pays_not_weekly_or_fortnightly",
        "Additional withholding for 53 or 27 pays was asked for on a pay that is not weekly or fortnightly.",
    ),
    (
        "payg.foreign_resident_not_scale_4",
        "Residency was given for a scale other than 4, which settles it itself.",
    ),
    (
        "payg.medicare_adjustment_not_allowed",
        "A Medicare levy adjustment was claimed on a scale other than 2 or 6.",
    ),
    (
        "payg.reductions_exceed_withholding",
        "Tax offsets and the Medicare levy adjustment exceed the amount withheld before them.",
    ),
    (
        "payg.tax_offsets_not_allowed",
        "Tax offsets were claimed on a scale other than 2, 5 or 6.",
    ),
    (
        "period.unsupported",
        "The worksheet has no rates for the period.",
    ),
    (
        "scope.not_confirmed",
        "The stated scope conditions were not confirmed.",
    ),
    (
        "working_holiday_maker.band_unsettled",
        "Payments before are from $135,000.01 to $135,000.99, which none of Schedule 15's columns covers.",
    ),
    ("worksheet.unknown", "The worksheet does not exist."),
];
