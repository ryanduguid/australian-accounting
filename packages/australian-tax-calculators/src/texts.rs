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
        "foreign_resident_tax",
        "Individual who was a foreign resident at all times in the income year and is a prescribed non-resident (Income Tax Rates Act 1986 s 3(1)): not one to whom, at any time in the year, compensation or a pension, allowance or benefit is payable under the Veterans' Entitlements Act 1986, subsection 4(6) of the Veterans' Entitlements (Transitional Provisions and Consequential Amendments) Act 1986, the Military Rehabilitation and Compensation Act 2004 or the Social Security Act 1991 in respect of which the person is liable to be assessed and to pay income tax in Australia, who pays resident rates. Ordinary taxable income only. There is no tax-free threshold: Schedule 7 Part II taxes the income to $135,000 at the second resident personal tax rate (30%), to $190,000 at the third (37%) and the rest at 45%. A foreign resident pays no Medicare levy, which ITAA 1936 s 251S(1) charges on a person resident at some time in the year, and is not entitled to the low income tax offset, which ITAA 1997 s 61-110(1) gives only to a resident, so medicare-levy and low-income-tax-offset do not apply. Excludes part-year residents, who are taxed as residents; a working holiday maker at any time in the year (clause 4); minors' special rates; the superannuation and employment termination remainders; a special income component (clauses 2 and 3); trustees and beneficiaries; working out which amounts are assessable to a foreign resident; other offsets, the Medicare levy surcharge, HELP, PAYG credits and tax payable or refundable. Supply established whole-dollar taxable income; this is basic tax before offsets.",
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
        "hire_purchase_schedule",
        "One hire purchase agreement whose hirer is the notional buyer (ITAA 1997 s 240-17(2)), starting in the income year given, with a level monthly instalment paid in arrears (at each month's end) or in advance (at its start) and any balloon paid at the end of the term. The operator establishes the amount financed (the notional loan principal: the price less any deposit or trade-in), the instalment, the number of instalments, the balloon and the start date. Each month runs from the start date's day of the month, or the month's last day when it is shorter. Under s 240-60(1) each month's notional interest is the outstanding notional loan principal, less any instalment due at the month's start, times the implicit interest rate: the monthly compound rate at which the payments repay the amount financed, found to 64 halvings of 0 to 100% a month. The payments themselves are not deductible but are taken into account in the notional interest (s 240-55). Each month shows the notional interest to date rounded half up to cents, less what the months before show, so the months add up to the total charges, each is within a cent of its amount at the rate found and the last balance is nil. The balloon is shown apart from the last instalment, on its own date. Refuses a rate below nil or of 100% a month or more, a rate that cannot be found closely enough to leave a nil balance to the cent, more than 360 instalments and a nil amount financed or instalment. Excludes deciding whether Division 240 applies and what is financed; fees, GST and stamp duty; irregular, skipped, weekly or quarterly payments; termination amounts other than the balloon and estimates of them (s 240-60(3) and (4)); apportioning a month that spans 30 June between income years (s 240-60(2)); whether the notional interest is deductible (s 240-50); and the notional seller.",
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
        "fbt_loan",
        "One loan an ordinary taxable employer made to a current employee, in the FBT year ended 31 March 2026 or 2027, at that year's statutory interest rate (8.62% for 2026, 8.27% for 2027). The operator establishes the loan's balances at the end of each day in the year the employee owed it, added together; the interest that accrued on it for the year; the share of the notional interest that would have been a once-only income tax deduction to the employee had they paid it; and the deduction that would have been allowable for the interest accrued. Notional interest is the summed daily balances times the rate over the days in the year (s 136(1)), rounded to cents; the gross taxable value is the amount by which it exceeds the interest accrued (s 18). The otherwise deductible rule (s 19) reduces that by the notional deduction: the gross deduction on the notional interest, rounded to cents, less the deduction for the interest accrued, never below nil; the taxable value never falls below nil. The rule applies only to a loan to an employee, not an associate, and the operator establishes that the employee's declaration or documents were obtained. Excludes deciding whether a loan benefit or an exemption (s 17) arises, interest deferred or unpaid (a further loan or a debt waiver), the deduction itself, the type 2 gross-up and FBT payable.",
    ),
    (
        "fbt_meal_entertainment",
        "An ordinary taxable employer's meal entertainment fringe benefits for the FBT year ended 31 March 2026 or 2027, where the employer elected to apply Division 9A, with no meal entertainment provided under a salary packaging arrangement. The operator establishes the method and the year's total expenses for providing meal entertainment to all persons, excluding contributions by employees or their associates that the employer did not reimburse (s 37AB). The 50/50 split method takes 50% of those expenses (s 37BA). The 12 week register method (s 37CB) applies the register percentage, the value of the register's meal entertainment fringe benefits over the value of all its meal entertainment, worked out exactly and shown to 2 places, to the expenses. A register is valid for the FBT year it was first valid for and the 4 after it, and ceases to be valid at the end of a year whose expenses were more than 20% higher than its first valid year's (s 37CD); the worksheet checks both from the expenses given for each year since its first valid year. The operator establishes that the register was kept as ss 37CC and 37CE require and that no later register replaced it. Excludes deciding what is meal entertainment, the elections themselves, the actual method, exempt benefits, splitting the value between type 1 and type 2 for fbt, gross-up and FBT payable.",
    ),
    (
        "fbt_car_parking",
        "An ordinary taxable employer's car parking fringe benefits for the FBT year ended 31 March 2026 or 2027, valued by one method: the commercial parking station method (s 39C), the market value method (s 39D), the average cost method (s 39DA), the statutory formula (spaces) method (s 39FA) or the 12 week record keeping method (s 39GB). The operator establishes that each benefit meets s 39A other than the threshold, such as a car parked for more than 4 hours in a daylight period on the provider's business or associated premises with a commercial parking station within 1 km, for one premises and the stations within 1 km of it (a worksheet for each premises), and the highest of those stations' lowest all-day fees on the first business day of the year: no benefit arises unless it is more than that year's car parking threshold ($11.03 for 2026, $11.48 for 2027; s 39A(1)(a)(iii)). Under the commercial parking station method the benefits are counted and each valued at the lowest all-day fee on its day, which must be the same on every day (a weekly or monthly fee becomes a daily rate under s 39E); under the average cost method each is valued at the average of the lowest fees on the days benefits were first and last provided in the year. Under both, the gross taxable value is rounded to cents before the recipients' payments, which the operator establishes in total, counting each benefit's payment only up to that benefit's value, since an excess on one benefit reduces no other; the payments cannot take the taxable value below nil, and an unused excess is shown. The market value method values each benefit at the arm's length amount a suitably qualified valuer at arm's length reported, the same for every benefit, less the payments as above; the operator establishes that the report was given before the declaration date in the approved form and that the values come from it (s 39D(3)). The spaces method takes, for each space with a car parking benefit for an employee the election covers, its daily rate amount (a benefit's value by the commercial parking station, market value or average cost method as if there were no recipients contribution; s 39FC) times its availability period's days over 366 times 228, in groups of spaces sharing a rate and period, each group with its own first business day fee above the threshold; their total, the total statutory benefit, divided by 366 once and rounded to cents (s 39FA(4)); where the employees the election covers average fewer than the eligible spaces, each average taken from the start and end of the parking period, that total times the one over the other, rounded to cents (s 39FB); less every relevant recipients contribution (s 39FE), never below nil. The operator establishes the election and whom it covers, that the fees behind the daily rates are representative (s 39FA(5)) and that the head counts are representative (s 39FB(6) and (7)), and gives the election's employees and spaces in one run. The 12 week register method takes the register's total value of car parking benefits (s 39GC), already net of payments, times 52 / 12, times the days in the car parking availability period over 366, the divisor the Act sets for every year (s 39GB). A register is valid for the FBT year it was first valid for and the 4 after it (s 39GF), which the worksheet checks; the operator establishes that it was kept as s 39GG requires, that the number of benefits rose by no more than 10% on any day of an earlier year since its first valid year (it ceases to be valid at the end of such a year) and that no later register replaced it. Excludes valuing the parking itself, head counts that are not representative (when s 39FB does not apply), exemptions such as the small business exemption (s 58GA), fees that are not representative (ss 39AA and 39AB), splitting the value between type 1 and type 2 for fbt, gross-up and FBT payable.",
    ),
    (
        "fbt_debt_waiver",
        "An ordinary taxable employer's debt waiver fringe benefits for the FBT year ended 31 March 2026 or 2027: the employer or an associate waived the obligation of an employee or an associate to pay or repay an amount (s 14). The taxable value is the amount waived, principal and interest together (s 15); the Act allows no recipient's contribution or otherwise deductible rule against it. The operator establishes the amount and that the waiver was for reasons related to the employment, not a genuine bad debt written off for reasons unrelated to it. Excludes deciding whether a waiver is a benefit, interest merely left unpaid (a loan benefit), gross-up and FBT payable.",
    ),
    (
        "fbt_expense_payment",
        "An ordinary taxable employer's expense payment fringe benefit for the FBT year ended 31 March 2026 or 2027: the employer or an associate paid a third party to discharge the recipient's obligation (s 20(a)) or reimbursed the recipient (s 20(b)). The operator establishes the recipient's expenditure, the amount paid or reimbursed, which cannot exceed it, and the recipient's contribution, which arises only for a payment. An external benefit's taxable value is the payment less the contribution, or the reimbursement (s 23), never below nil, with any excess contribution shown. For an employee, not an associate, the otherwise deductible rule (s 24) then reduces it by the notional deduction: the gross deduction on the recipient's expenditure, rounded to cents, less the deduction for the expenditure the recipient bore (the recipient's portion), never below nil; the operator establishes that the documents and declarations s 24 requires were obtained. An in-house property expense payment (s 22A(1)), for goods from the employer or an associate that, at or about the time it provides them, carries on a business that includes providing identical or similar goods principally to outsiders, or from a third party that acquired them from the employer or an associate when, at or about that time, both carry on such a business (s 136(1), paragraphs (c) and (d) of the definition), where the operator establishes that the recipient obtained documentary evidence of the expenditure and gave it, or a copy, to the employer before the declaration date (s 136(1), paragraph (e) of the definition), takes the property's value under the in-house rule the method names (s 42(1), as fbt_property describes) and reduces it by the recipient's portion, the expenditure less the payment net of the contribution, never below nil; then, unless the property was salary packaged, by the part of the employee's $1,000 in-house reduction for the year (s 62) that the operator establishes is left. An in-house residual expense payment (s 22A(2)), for a service or other residual benefit from the employer or an associate that, at or about the time that would be its comparison time were it a residual fringe benefit, carries on a business that includes providing identical or similar benefits principally to outsiders, or from a third party that purchased it from the employer or an associate when, at or about that time, both carry on such a business (s 136(1), paragraphs (b) and (c) of the definition), with the same documentary evidence (paragraph (d)), is valued the same way under the in-house residual rule the method names: the notional value when salary packaged, otherwise 75% of the lowest price charged to the public for an identical benefit or 75% of the notional value (ss 48 and 49, for a period benefit read with s 22A(3) for the recipient's overall benefit). Excludes airline transport, the otherwise deductible rule for in-house expense payments (s 24(1)(b)(i)) and for car expenses, benefits provided jointly to an employee and an associate (s 138(3)), exempt benefits (such as ss 20A, 21, 22 and 58P), other Division 14 reductions, gross-up and FBT payable.",
    ),
    (
        "fbt_property",
        "An ordinary taxable employer's property fringe benefit for the FBT year ended 31 March 2026 or 2027: property provided free or at a discount (s 40), valued under the rule the method names. In-house property, goods identical or similar to those the provider, or the seller it acquired them from, provides principally to outsiders, is valued under s 42(1): the notional value if salary packaged (aa); for property the provider manufactured, produced, processed or treated, the lowest arm's length price to manufacturers, wholesalers or retailers ((a)(i)), else 75% of the lowest price to the public ((a)(ii)), else 75% of the notional value ((a)(iii)); for property it acquired, the lesser of its arm's length price and the notional value (b); otherwise 75% of the notional value (c). External property is valued under s 43 at the cost price of property the employer or an associate bought at arm's length at or about the provision time (a), the arm's length expenditure the employer or an associate incurred to a third-party provider (b), or the notional value (c). 75% is rounded to cents. The recipient's contribution comes off, never below nil, with any excess shown. For an employee, not an associate, the otherwise deductible rule (s 44) then reduces the value by the notional deduction: the gross deduction on the value before the contribution, rounded to cents, less the deduction for the contribution, never below nil; the operator establishes that the declarations s 44 requires were obtained. An in-house benefit not salary packaged is then reduced by the part of the employee's $1,000 in-house reduction for the year (s 62) that the operator establishes is left. Excludes airline transport (s 42(1)(ab)), car property benefits under s 44's car rules, benefits provided jointly to an employee and an associate (s 138(3)), exempt benefits (such as ss 41 and 58P), other Division 14 reductions, gross-up and FBT payable.",
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
        "payg_gross_up",
        "The gross pay for one regular weekly, fortnightly, monthly or quarterly payment of salary or wages made from 1 July 2024, in the income year given, whose Schedule 1 withholding on the scale the operator established leaves the net pay given, as for pay agreed net of tax. It searches payg-withholding's own results, so payg-withholding's scope applies, with scale 4 taking foreign_resident as it does there. With study_loan, it searches study-loan-withholding's results instead, whose scope applies: Schedule 1 with Schedule 8's study and training support loan component, for the pay date where the year needs one, on scales 1, 2, 3, 5 and 6. Excludes tax offsets, the Medicare levy adjustment, the additional amount for 53 or 27 pays, super and other deductions from pay. Refused where more than one gross leaves the net, as Schedule 1 withholds whole dollars.",
    ),
    (
        "contribution_caps",
        "One individual's contributions received by all their funds in the income year, already classified by the operator as concessional or non-concessional, with no bring-forward period started in either of the 2 previous years. Total super balance is at the 30 June before the year. Unused concessional cap is either the established total of unexpired amounts from the previous 5 years or, given each earlier year's established concessional contributions from at least the 5th year before (and the balance at the 30 June before any year whose contributions exceed its cap), worked out year by year from 2018-19 under ITAA 1997 s 291-20(3) to (7): each year's unused cap, the amounts applied earliest year first, the amount carried into the year, and, after the year's contributions and any proposed one, what is left of each year's amount and the year at whose end it lapses. Cap arithmetic only: it does not say what to contribute. The carry-forward by year excludes defined benefit interests and notional taxed contributions. Excludes classification, fund acceptance and work-test rules, deductibility, the CGT cap, downsizer and other excluded contributions, excess concessional amounts counting as non-concessional, determinations, release elections, Division 293 tax and the transfer balance cap.",
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
        "One regular weekly, fortnightly, monthly or quarterly payment of salary or wages made from 24 September 2025, in the income year given, to a payee who gave a TFN declaration or withholding declaration saying they have a study and training support loan debt, on scale 1, 2, 3, 5 or 6 as the operator established. The year selects the Schedule 1 coefficients and the Schedule 8 component coefficients for payments made in it; a 2025-26 payment needs its pay date, because the Schedule 8 coefficients of F2025L01016 apply from 24 September 2025. Scale 1 uses the Schedule 8 component for a payee who did not claim the tax-free threshold; scales 2, 3, 5 and 6 use the one for a payee who claimed it or is a foreign resident. Earnings include allowances subject to withholding. The result is the Schedule 1 amount plus the Schedule 8 component, each rounded to the dollar; the ATO's combined coefficients can differ by a dollar and it accepts either method. Excludes 2025-26 payments made before 24 September 2025, scale 4, a Medicare levy variation for a spouse or dependants, tax offsets, extra amounts for 53 or 27 pays, lump sum termination payments, back payments, bonuses and other schedules.",
    ),
    (
        "medicare_levy",
        "Australian resident individual for the full income year. The operator establishes the days in the year, if any, for which they qualified for a full or a half Medicare levy exemption: a full exemption as a prescribed person under ITAA 1936 s 251U(1) in the medical category (a blind pensioner, or entitled to full free medical treatment under Defence Force or Veterans' Affairs arrangements) or as one not entitled to Medicare benefits, with no dependant outside the exemption categories; a half exemption in the medical category with such a dependant (s 251U(3)); a foreign resident is outside this worksheet. The operator also establishes whether they are entitled to the seniors and pensioners tax offset (SAPTO) for the year at more than nil; whether they had a spouse on the last day of the year as s 3(3) reads it (a spouse living separately and apart does not count, but one living apart only because of illness, as in the ATO's Ashton example, does; so does one who died in the year), and if so the spouse's whole-dollar taxable income and SAPTO entitlement; without a spouse, whether they are entitled to the invalid and invalid carer tax offset for their child (ITAA 1997 Subdivision 61-A) or the sole parent notional tax offset (Subdivision 961-B); and the number of dependent children and students (s 961-5(1)(c)) for whom they or their spouse are entitled to the Subdivision 961-A notional tax offset, counting, without a spouse, only those for whom family tax benefit was payable to them. A spouse must also be an Australian resident for the full year, not entitled to any exemption and not a person to whom s 9A applies. Omitting those inputs states a person without SAPTO, spouse or those offsets. The operator must also establish whether s 9A applies: from 2024-25 it takes eligible lump sums in arrears out of taxable income when they are at least 10% of normal taxable income and its accrual-year tests are met. A person to whom s 9A applies is refused. Supply established whole-dollar taxable income. Applies the 2% rate and the low-income reduction (s 7): no levy at or below the threshold amount, and up to the phase-in limit no more than 10% of the excess over it, at SAPTO thresholds for a person entitled to SAPTO. With a spouse or one of those offsets the family reduction (s 8) follows: no levy when family taxable income is at or below the family income threshold, which rises for each dependent child; above it the levy is reduced by 2% of the threshold less 8% of the excess, in the person's share of family taxable income when the spouse's own income is above the spouse's threshold amount, and any part of the spouse's reduction above the spouse's levy also reduces this levy (s 8(4)). The levy after ss 7 and 8 is then reduced by the share of the year's days for which the person was a prescribed person (s 9(1)): each full exemption day, and half of each half exemption day; a full exemption for the whole year leaves no levy (s 251T). Excludes foreign residents, the Medicare levy surcharge, trustees and part-year residence.",
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
        "unused_leave_payment",
        "PAYG withholding under Schedule 7 (payments from 1 July 2026) from the unused annual leave, leave loading and unused long service leave paid in 2026-27 in consequence of the termination of one employee's employment, with each part's assessable share and tax offset under ITAA 1997 Subdivisions 83-A and 83-B and its label for the employee's income statement. The employee is not a working holiday maker, has no withholding variation that covers unused leave payments, and is alive when paid. The operator establishes the reason for the termination, the first day of the leave's accrual, the parts of each total that accrued before 16 August 1978, from 16 August 1978 to 17 August 1993 and before 18 August 1993 (for long service leave, worked out by days under ss 83-95 to 83-115), and the employee's normal gross earnings for the last full pay period and their Schedule 1 scale, or scale 4 and their residency when no TFN was provided. Employment termination payments, unused sick leave, rostered days off, payments in lieu of notice, leave paid after death, the study and training support loan component (Schedule 7 withholds none for it), super an award or agreement requires, and the tax assessed are outside it.",
    ),
    (
        "schedule_5_method_a",
        "Withholding under Schedule 5 (NAT 3348) Method A from one regular weekly, fortnightly or monthly pay of salary or wages that includes back payments (including lump sums in arrears), commissions, bonuses or similar payments, made from 1 July 2024 in the income year given, to a payee who gave a TFN, on scale 1, 2, 3, 5 or 6 as the operator established from the payee's declarations. The year selects the Schedule 1 coefficients, as payg-withholding uses them. The operator establishes the pay's gross earnings excluding the additional payments, the additional payments in it and whether the payee declared a study and training support loan debt, when each table amount includes the Schedule 8 component, as Schedule 5 allows, with the Schedule 8 coefficients for the pay date, which a 2025-26 pay must give and which must be on or after 24 September 2025, when its coefficients apply. Cents are ignored in the earnings (so a monthly amount ending in 33 cents gains no cent) and in the additional payments divided by 52 weekly, 26 fortnightly or 12 monthly pay periods, with no adjustment for 53 or 27 pays, or, for a commission, bonus or similar payment for a defined period of less than 12 months, by the pay periods it relates to, which the operator chooses. The extra withheld from the earnings plus that share, times the same pay periods, is limited to 47% of the additional payments, with cents ignored and a negative result taken as nil; the pay's withholding adds it to the earnings' amount. A payment for a single pay period belongs in that pay under Schedule 1, so 1 pay period is refused. Excludes working holiday makers (Schedule 15), payees who gave no TFN (scale 4), withholding variations, tax offsets, Medicare levy adjustments, superannuation income streams, employment termination and unused leave payments, and other schedules.",
    ),
    (
        "schedule_5_method_b_i",
        "Withholding under Schedule 5 (NAT 3348) Method B(i) from one regular weekly, fortnightly or monthly pay of salary or wages that includes a back payment for earlier pay periods in the current financial year, made from 1 July 2024 in the income year given, to a payee who gave a TFN, on scale 1, 2, 3, 5 or 6 as the operator established from the payee's declarations. The year selects the Schedule 1 coefficients, as payg-withholding uses them. For each earlier pay period the back payment applies to, the operator establishes the normal earnings paid for it (including back payments already applied to it by Method B(i)), the part of the back payment for it and the amount withheld from it; and the current pay's gross earnings excluding additional payments and whether the payee declared a study and training support loan debt, when each table amount includes the Schedule 8 component, as Schedule 5 allows, for a year whose Schedule 8 coefficients apply from 1 July, and the amounts withheld include it too; 2025-26's changed on 24 September 2025, and Schedule 5 does not say which coefficients recalculate the earlier periods. Each period's withholding is recalculated on its earnings plus its part of the back payment, less the amount withheld from it; the total is the withholding on the back payment, a negative total taken as nil, with no 47% limit, and the pay's withholding adds it to the current earnings' amount. A financial year holds at most 53 weekly, 27 fortnightly or 12 monthly pays, so at most 52, 26 or 11 earlier periods. When Method B(ii) applies to another additional payment in the same pay, count the earnings' amount once. Excludes back payments for a prior financial year and commissions, bonuses and similar payments (Method A or B(ii)), working holiday makers (Schedule 15), payees who gave no TFN (scale 4), withholding variations, tax offsets, Medicare levy adjustments, superannuation income streams and other schedules.",
    ),
    (
        "schedule_5_method_b_ii",
        "Withholding under Schedule 5 (NAT 3348) Method B(ii) from one regular weekly, fortnightly or monthly pay of salary or wages that includes a back payment for a prior financial year, or a commission, bonus or similar payment that does not relate to a single pay period, made from 1 July 2024 in the income year given, to a payee who gave a TFN, on scale 1, 2, 3, 5 or 6 as the operator established from the payee's declarations. The year selects the Schedule 1 coefficients, as payg-withholding uses them. The operator establishes the normal earnings paid in the financial year to date, including the current pay, plus current-year back payments for which Method B(i) was used; the pay periods to date, including the current one; the additional payment in this pay; additional payments made earlier in the year for which Method B(ii) was used and the amounts withheld from them; the current pay's gross earnings excluding additional payments; and whether the payee declared a study and training support loan debt, when each table amount includes the Schedule 8 component, as Schedule 5 allows, with the Schedule 8 coefficients for the pay date, which a 2025-26 pay must give and which must be on or after 24 September 2025, when its coefficients apply, and the earlier amounts withheld include it too. Average total earnings are the earnings to date divided by the pay periods to date, cents ignored. All the year's Method B(ii) additional payments are divided by 52 weekly, 26 fortnightly or 12 monthly pay periods, cents ignored, with no adjustment for 53 or 27 pays; the extra withheld from the average plus that share, times those pay periods, less the earlier amounts withheld, is limited to 47% of this additional payment, with cents ignored and a negative result taken as nil; the pay's withholding adds it to the current earnings' amount. When Method B(i) applies to a back payment in the same pay, count the earnings' amount once. Excludes working holiday makers (Schedule 15), payees who gave no TFN (scale 4), withholding variations, tax offsets, Medicare levy adjustments, superannuation income streams, employment termination and unused leave payments, and other schedules.",
    ),
    (
        "working_holiday_maker_withholding",
        "Withholding under Schedule 15 from one payment to a working holiday maker, made from 1 July 2024 in the income year given: an individual who holds a working holiday visa (subclass 417), a work and holiday visa (subclass 462) or a bridging visa that Schedule 15 covers, as the operator established. The payment is the total for the pay period, including allowances, back payments, commissions, bonuses and other payments Schedule 15 covers; for an employment termination payment, only its taxable component. The operator establishes the payments made to the payee in the income year before this one, whether the employer is registered with the ATO to withhold at working holiday maker rates, and whether the payee quoted a TFN or, within the last 28 days, advised that they applied for one or made an enquiry with the ATO. A registered employer withholds the payment, cents ignored, times the Table A rate for the column the payments before it select: 15% for less than $45,001, 30% from $45,001 to $135,000, 37% from $135,001 to $190,000 and 45% for more than $190,000, rounded to the dollar with 50 cents up, or with cents ignored at 45%; payments before of $135,000.01 to $135,000.99, which no column covers, are refused. Without a TFN any employer withholds 45% of the payment, cents ignored. An unregistered employer withholds at foreign resident rates, the Schedule 1 scale 3 amount for a weekly, fortnightly or monthly pay, as payg-withholding gives it. Every payment needs a weekly, fortnightly or monthly pay period. Excludes a payee who claimed an exemption from quoting a TFN, Medicare levy adjustments and tax offsets, which do not reduce this withholding, the Seasonal Worker Programme, Pacific Labour Scheme and Pacific Australia Labour Mobility scheme, withholding variations, the employer's registration and the payee's visa, residency and annual return.",
    ),
    (
        "medicare_levy_surcharge",
        "Australian resident individual for the full income year who was not in a Medicare levy exemption category at any time in it, and who was either not married and without dependants for the whole year (Medicare Levy Act 1986 s 8B) or married for the whole year as s 3(3) and (3A) read it (s 8D: a spouse living separately and apart does not count; one who died in the year does). The operator establishes the person's whole-dollar taxable income, including any amount on which family trust distribution tax was paid (s 3(2A)); their reportable fringe benefits total; their income for surcharge purposes; the days in the year on which the person or, for a married person, the person, the spouse or a dependent child had no appropriate level of private patient hospital cover, not counting a spouse or child who was then a prescribed person, or would have been but for ITAA 1936 s 251U(2) (s 8D(1)(b)); and for a married person, the spouse's income for surcharge purposes and the number of MLS dependent children. The tier is set by income for surcharge purposes, combined with the spouse's for a married person, against the year's singles thresholds or the family thresholds, which are double them and rise by $1,500 for each dependent child after the first: above the tier 1 threshold the rate is 1%, for a tier 2 earner 1.25% and for a tier 3 earner 1.5%. A married person is liable only if their own income for surcharge purposes also exceeds the s 3(1) threshold amount ($27,222 for 2024-25, $28,011 from 2025-26). The rate applies to taxable income plus reportable fringe benefits only, not to the other parts of income for surcharge purposes, for the days without cover as a share of the year's days (ss 8B and 8D, and A New Tax System (Medicare Levy Surcharge - Fringe Benefits) Act 1999 ss 12, 15 and 16). Excludes a person with dependants but no spouse (s 8C), a change of marital or dependant status during the year, a person in an exemption category, trustees (ss 8E to 8G), part-year residence, the tax offset for the surcharge on lump sums in arrears (ITAA 1997 Subdivision 61-L) and the private health insurance rebate.",
    ),
    (
        "seniors_pensioners_withholding",
        "Withholding under Schedule 9 (NAT 4466), the tax table for seniors and pensioners, from one regular weekly, fortnightly or monthly payment made from 1 July 2024 in the income year given, to a payee the operator established it applies to: a senior or pensioner 67 or older at the end of the financial year, or a veteran with a service pension or a war widow or widower with an income support supplement from the Department of Veterans' Affairs who is at least 60, who gave a Withholding declaration, or its short version for seniors and pensioners, saying they are an Australian resident for tax purposes and answering yes to the questions the schedule names. The year selects the coefficients: 2024-25 and 2025-26 use the schedule for payments from 1 July 2024 to 30 June 2026, and 2026-27 the one for payments from 1 July 2026. Earnings include allowances and irregular payments subject to withholding. The payee's status (single, a member of an illness-separated couple or a member of a couple) and any full or half Medicare levy exemption claimed on a Medicare levy variation declaration select scales 1 to 9. The weekly amount is a x weekly earnings - b, on the weekly earnings in whole dollars plus 99 cents, rounded to the dollar with 50 cents up; a fortnightly pay uses half its earnings and twice the weekly amount, and a monthly pay 3/13 of its earnings, after adding a cent to an amount ending in 33 cents, and 13/3 of the weekly amount, rounded to the dollar. For a payee who declared a study and training support loan debt and claimed no Medicare levy reduction for a spouse or dependants and low family income, the schedule refers the payer to the study and training support loans weekly tax table, which adds Schedule 8's component for a payee claiming the tax-free threshold, as a Schedule 9 payee does: a x the same weekly earnings - b, rounded to the dollar and converted to the pay period in the same way, on the coefficients for the pay date, for 2026-27 and for 2025-26 pays made from 24 September 2025. Excludes other tax offsets, which the schedule values only by combining its ready reckoner's weekly amounts, the Medicare levy adjustment, which it leaves to the Medicare levy adjustment weekly tax table, quarterly pays, for which it has no rule, payees who gave no TFN, study and training support loans in 2024-25 and before 24 September 2025, whose Schedule 8 used a percentage of weekly earnings, lump sum leave loading (Schedule 5), super income streams (also Schedule 13), withholding variations and other schedules.",
    ),
    (
        "seniors_pensioners_tax_offset",
        "The seniors and pensioners tax offset (ITAA 1936 s 160AAAA) of an individual the operator established meets its first condition (s 160AAAA(2)): on a day in the year qualified for the age pension, or of veterans' pension age and eligible for a Veterans' Entitlements Act pension, allowance or benefit, or with an assessable social security or veterans' pension, education entry payment or income support supplement, and not in gaol that day. The operator establishes the whole-dollar rebate income (s 6(1)) and the status that gives the base rebate amount (regulations s 9(2)): $2,230 for a person who is not a spouse at any time in the year, $2,040 for a member of an illness-separated couple and $1,602 for a spouse, spouses living separately and apart permanently counting as not spouses. A person whose spouse status changed in the year is outside the worksheet. The second condition (s 160AAAA(3) and (4)) is rebate income, or for a spouse half the couple's rebate income, less than the cut-out: the base amount divided by 0.125 plus the rebate threshold, rounded up to the dollar (regulations s 10). The rebate threshold is (the $445 rebate maximum amount plus the base amount) divided by the year's lowest marginal tax rate plus the tax-free threshold, or above $37,000 the s 9(6) formula, rounded up to the dollar (s 9(4) to (8)). The offset is the base amount less 12.5 cents for each dollar of the person's own rebate income above the threshold (s 11), rounded up to the whole dollar as the ATO does, and nil without entitlement. With the spouse's taxable income, a spouse entitled to the offset whose whole taxable income is ordinary taxable income taxed at the resident scale (Income Tax Rates Act 1986 Sch 7 Part I) passes their unused base amount (s 12): all of it up to $6,000 of taxable income, then less 15% of the excess, increasing this person's base amount and threshold, but not the cut-out, when this person's own offset is below the base amount. Years from 2026-27 use the 2025 Regulations and earlier years the 2015 Regulation, whose rules are the same. Excludes trustees (s 160AAAB), a person also entitled to the beneficiary tax offset, of which s 160AAA(4) allows only the greater, a part-year spouse, the transfer from a foreign resident spouse or from a spouse with a superannuation or employment termination remainder taxed at 45% (Sch 7 Part I clause 1(a) and (aa)), whose tax can leave less than the whole base amount unused, the transfer out of this person's unused amount, and applying the offset, which is non-refundable, against tax payable.",
    ),
    (
        "fbt_residual",
        "A residual fringe benefit (FBTAA s 45) an ordinary taxable employer provided in respect of an employee's employment in the FBT year ended 31 March 2026 or 2027: a right, privilege, service or facility, or another benefit no other Division covers, such as the use of property. The operator establishes whether it is in-house (identical or similar to what the provider supplies to the public in the ordinary course of business) or external, whether it was provided under a salary packaging arrangement and, for a benefit provided over a period, the part provided in the year. An in-house benefit under a salary packaging arrangement is valued at its notional value (ss 48(aa) and 49(aa)); another in-house benefit at 75% of the lowest price the provider charged the public for an identical benefit (ss 48(a) and 49(a)) or, with no such sale, 75% of its notional value (ss 48(b) and 49(b)), rounded to cents; an external benefit at the arm's length amount the employer or its associate paid as provider (ss 50(a) and 51(a)), the arm's length expenditure it incurred to another provider (ss 50(b) and 51(b)) or otherwise its notional value (ss 50(c) and 51(c)). The recipients contribution reduces that value, never below nil; an unused excess is shown and reduces no other benefit. The otherwise deductible rule (s 52(1)) then takes off the notional deduction: the gross deduction, the share of the value before the contribution that would have been a once-only deduction to the employee had they paid it, rounded to cents, less the deduction for the contribution, never below nil and never taking the value below nil. It applies only to a benefit provided to the employee, not an associate, and the operator establishes the declaration or travel diary s 52 requires. For an in-house benefit outside a salary packaging arrangement the employee's $1,000 in-house reduction (s 62) then applies, as far as their other in-house fringe benefits have left it. Excludes car fringe benefits and benefits exempt under s 47 or Division 13, airline transport fringe benefits (ss 48(ab) and 49(ab)), car residual benefits (s 52(1)(da) and (e)), benefits deemed provided to the employee only under s 138(3) (s 52(5)), the ATO's cents per kilometre values for motor vehicles other than cars, apportioning a period benefit between years, splitting the value between type 1 and type 2 for fbt, gross-up and FBT payable.",
    ),
    (
        "fbt_housing",
        "A housing fringe benefit (FBTAA s 25): an ordinary taxable employer's housing right, a lease or licence to occupy a unit of accommodation as the employee's usual place of residence, subsisting in the FBT year ended 31 March 2026 or 2027, valued under s 26 by one method. outside_australia: a unit not in a State or internal Territory, at the market value of the recipient's current housing right (s 26(1)(a)). accommodation_business: a caravan or mobile home, or a unit in a hotel, motel, hostel or guesthouse, whose provider let identical or similar units to outsiders in the tenancy period, at that market value, or 75% of it, rounded to cents, where the benefit would otherwise be an in-house residual fringe benefit (s 26(1)(b)). Any other unit is valued at its statutory annual value times the whole days of the tenancy period over the days in the year, rounded to cents (s 26(1)(c)): base_year works that value out as the market value for the tenancy period times the days in the year over its whole days (s 26(2)(a)), and indexed as the year before's statutory annual value, or their weighted average, times the indexation factor for the year and the State or Territory, as the ATO's rates page publishes it (ss 26(2)(b) and 28), each rounded to cents, the amount the next year's prior_statutory_annual_value takes. The operator establishes the market value, disregarding any right to have the occupant's expenses paid and onerous conditions of employment (s 27), and whether the year is a base year: a year after one with no housing fringe benefit in respect of the right or an equivalent one (s 26(3)(b)), a year the employer elected (s 26(3)(aa)), a year after an alteration changed the market value by 10% or more (s 26(5)) or a year after 9 without a base year (s 26(3)(c)), which the worksheet checks. The recipient's rent then reduces the value, never below nil; an unused excess is shown. Excludes remote area housing benefits, which are exempt (s 58ZC), the reduction for temporary accommodation on relocation (s 61C), splitting the value between type 1 and type 2 for fbt, gross-up and FBT payable.",
    ),
    (
        "fbt_lafha",
        "An ordinary taxable employer's living-away-from-home allowance fringe benefit (FBTAA s 30(1)) for a period in the FBT year ended 31 March 2026 or 2027, valued by one method. maintains_home applies s 31, where the employee keeps a home in Australia they or their spouse have an ownership interest in, available for their use (s 31C), the allowance relates only to the first 12 months they are required to live away from it (s 31D) and the employee gave the declaration (s 31F); fly_in_fly_out applies s 31A, where the employee has accommodation at or near the usual place of employment, works on a fly-in fly-out or drive-in drive-out basis (s 31E) and gave the declaration. Under either, the taxable value is the allowance less the exempt accommodation component, the accommodation component up to the accommodation expenses substantiated under s 31G, and the exempt food component (s 31H): the food component less the applicable statutory food total, never below nil, up to the food expenses incurred, substantiated where s 31G requires. The applicable statutory food total is $42 a week for each eligible family member 12 or older before the year began and $21 a week for each other, for their days in the period, less any normal home food amount taken into account in the food component, never below nil. other applies s 31B, where neither section does, and the taxable value is the allowance. Run the worksheet once for each part of the allowance a different method applies to: under other for a period while the 12 months is paused (s 31(4)), and under maintains_home with food_expenses of nil for days the employee resumed living at the normal residence (s 31(3)). The operator establishes the components, the expenses, the eligible family members (s 136(1)) and the Commissioner's reasonable food amount. Excludes the allowance for an employee on an oil rig or other installation at sea (s 30(2)), living-away-from-home benefits other than an allowance, splitting the value between type 1 and type 2 for fbt, gross-up and FBT payable.",
    ),
    (
        "fbt_board",
        "An ordinary taxable employer's board fringe benefits (FBTAA s 35) in the FBT year ended 31 March 2026 or 2027: meals provided to employees, or their associates, that are board meals (s 136(1)), such as meals an employee entitled to accommodation and at least 2 meals a day is given, cooked and supplied on the employer's premises and not at a party, reception or other social function. Each meal is valued at $2.00 for a recipient who was 12 or older before the year began and $1.00 for another (s 36), less the recipients contribution for it, which the operator establishes in total, counting each meal's only up to its value; the contributions cannot take the value below nil, and an unused excess is shown. The otherwise deductible rule (s 37) then takes off the gross deduction, the share of the value before contributions that would have been deductible to the employee under ITAA 1997 s 8-1, rounded to cents, less the deduction for the contribution, never below nil and never taking the value below nil; it applies only to meals provided to the employee. Excludes deciding what is a board meal, exempt meals, such as those for an employee in a primary production business in a remote area (the employer's guide, 13.6), food or drink exempt under s 54, gross-up and FBT payable. Board fringe benefits are type 2 (the employer's guide, 13.2).",
    ),
    (
        "fbt_entertainment_classifier",
        "Classifies an ordinary taxable employer's entertainment expense lines for the FBT year ended 31 March 2026 or 2027; the employer is not exempt from income tax and is registered for GST. Each line states what was provided (food or drink, recreation, or accommodation or travel to do with either), for food or drink, and for accommodation or travel to do with it, the food or drink's occasion, the venue, whether the employer provided it or reimbursed an attendee, the attendees by class (employees, their associates, and others such as clients) with head counts, and its cost and whether that includes GST. Whether food or drink is entertainment follows TR 97/17's tests (para 23) for the occasion stated; recreation, and accommodation or travel to do with entertainment, are entertainment (ITAA 1997 s 32-10), and only food or drink, with the accommodation or travel to do with it, is meal entertainment (FBTAA s 37AD). A line's cost is split per head between its classes, the last class taking the rounding remainder, and a line of more than one class without every head count is refused. For each class the classifier gives the fringe benefit and its type (s 20 expense payment, s 40 property or s 45 residual; on a reimbursed line with more than one employee, the reimbursed employee's expense payment and each other employee's property or residual benefit, TR 97/17 para 98), the s 41 exemption, the minor benefits exemption only as a possibility (s 58P(1)(e)'s $300 threshold and the operator's statement of (f)(i); the rest of (f) is the employer's judgement), the deduction (ITAA 1997 ss 8-1, 32-5 and 32-20, or ITAA 1936 s 51AEA or 51AEB under an election, less the GST credit, s 27-5) and the GST credit, the deductible share of the GST in the cost (GST Act s 69-5), and, for each fringe benefit, whether it is an excluded fringe benefit left out of reporting: meal entertainment is (FBTAA s 5E(3)(a)) and other entertainment is not. Food or drink, or accommodation or travel to do with it, with no occasion stated, and an employee's share of something other than food or drink the employer provided on its business premises on a working day, are returned undetermined with the missing fact and left out of the totals. Under the 50/50 split or 12 week register method the meal entertainment lines' cost is totalled as meal_entertainment_expenditure for fbt-meal-entertainment, which values it; others such as clients receive no fringe benefit, though their meal entertainment counts in that total and is deductible at the method's share (FBTAA ss 37BA and 37CB(3); ITAA 1936 ss 51AEA and 51AEB). The operator establishes each line's facts, that every attendee on a travel_meal line is travelling for work, that nothing is provided under a salary packaging arrangement, as an in-house benefit or by an associate of the employer, and that no exception in ITAA 1997 ss 32-30 to 32-50 applies: in-house dining and dining facilities (items 1.1 to 1.3), overtime meals (items 1.4 and 5.1), recreation facilities for employees (item 1.5), other exempt food, drink or meals (items 1.6 and 1.7), allowances (item 1.8), seminars (item 2.1), the entertainment industry (items 3.1 and 3.2), promotion and advertising (items 4.1 to 4.3) and the disadvantaged public (item 5.2). Excludes tax-exempt bodies (fbt-tax-exempt-body-entertainment), entertainment facility leasing expenses (s 152B), benefits to a client's employees under an arrangement (TR 97/17 para 126), taxable values, contributions and the otherwise deductible rule (fbt-property, fbt-expense-payment and fbt-residual), each employee's reportable fringe benefits amount, gross-up and FBT payable.",
    ),
    (
        "fbt_tax_exempt_body_entertainment",
        "Tax-exempt body entertainment fringe benefits (FBTAA s 38) in the FBT year ended 31 March 2026 or 2027: non-deductible exempt entertainment expenditure (s 136(1)) an employer wholly or partly exempt from income tax, or without assessable income from the activities the entertainment relates to, incurred in entertaining employees or their associates. The taxable value is so much of the expenditure as is attributable to that entertainment (s 39): actual takes the amount the operator attributes to employees and associates; per_head apportions a function's expenditure by the employees and associates entertained over everyone entertained, rounded to cents, where the actual amount cannot easily be worked out (the employer's guide, 15.9). No recipient's contribution or reduction applies (FBT return 2026 instructions, item 23L). Excludes deciding what is entertainment or exempt (such as minor benefits, s 58P), meal entertainment valued under Division 9A (fbt-meal-entertainment), the 50/50 split for entertainment facility leasing expenses (s 152B), splitting the value between type 1 and type 2 for fbt, gross-up and FBT payable.",
    ),
    (
        "income_tests",
        "The income measures of one individual who was an Australian resident for the whole income year and alive at its end, each the person's own for the whole year, from components the operator has established and classified: income for surcharge purposes (ITAA 1997 s 995-1), Division 293 income (s 293-20(1)(a)), rebate income (ITAA 1936 s 6(1)), repayment income for study and training loans (HESA s 154-5), adjusted taxable income for offsets (ITAA 1997 s 995-1, ITAA 1936 s 6(1) and A New Tax System (Family Assistance) Act 1999 Sch 3, disregarding cls 3 and 3A) and total income for the super co-contribution (Superannuation (Government Co-contribution for Low Income Earners) Act 2003 s 8). A spouse's measures come from the spouse's own components. A dependant's adjusted taxable income for part of a year, a deceased person's annualised adjusted taxable income (Sch 3 cl 2(2)), the co-contribution's 10% eligible income test, the combined family amounts and every threshold and eligibility test the measures feed are outside it. Income for surcharge purposes follows s 995-1, which does not add exempt foreign employment income; the ATO's surcharge page adds it where taxable income is $1 or more, and the result warns when that changes the figure. Measures another agency applies are outside it: the Commonwealth Seniors Health Card income test and family assistance adjusted taxable income are Services Australia's (https://www.servicesaustralia.gov.au/income-test-for-commonwealth-seniors-health-card). Refuses part-year residents, working holiday makers and foreign residents, and components that cannot all be true together.",
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
        "foreign_resident_tax",
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
        "hire_purchase_schedule",
        "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/240-60",
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
        "fbt_loan",
        "https://www.ato.gov.au/law/view/document?DocID=SAV/FBTGEMP/00009",
    ),
    (
        "fbt_meal_entertainment",
        "https://www.ato.gov.au/law/view/document?DocID=SAV/FBTGEMP/00015",
    ),
    (
        "fbt_car_parking",
        "https://www.ato.gov.au/law/view/document?DocID=SAV/FBTGEMP/00017",
    ),
    (
        "fbt_debt_waiver",
        "https://www.ato.gov.au/law/view/document?DocID=SAV/FBTGEMP/00009",
    ),
    (
        "fbt_expense_payment",
        "https://www.ato.gov.au/law/view/document?DocID=SAV/FBTGEMP/00010",
    ),
    (
        "fbt_property",
        "https://www.ato.gov.au/law/view/document?DocID=SAV/FBTGEMP/00018",
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
        "payg_gross_up",
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
        "medicare_levy_surcharge",
        "https://www.ato.gov.au/individuals-and-families/medicare-and-private-health-insurance/medicare-levy-surcharge/medicare-levy-surcharge-income-thresholds-and-rates",
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
    (
        "unused_leave_payment",
        "https://www.ato.gov.au/tax-rates-and-codes/payg-withholding-schedule-7-tax-table-for-unused-leave-payments-on-termination-of-employment",
    ),
    ("schedule_5_method_a", SCHEDULE_5_METHODS),
    ("schedule_5_method_b_i", SCHEDULE_5_METHODS),
    ("schedule_5_method_b_ii", SCHEDULE_5_METHODS),
    (
        "working_holiday_maker_withholding",
        "https://www.ato.gov.au/tax-rates-and-codes/schedule-15-tax-table-for-working-holiday-makers",
    ),
    (
        "seniors_pensioners_withholding",
        "https://www.ato.gov.au/tax-rates-and-codes/schedule-9-tax-table-for-seniors-and-pensioners",
    ),
    (
        "seniors_pensioners_tax_offset",
        "https://www.ato.gov.au/individuals-and-families/income-deductions-offsets-and-records/tax-offsets/seniors-and-pensioners-tax-offset",
    ),
    (
        "fbt_residual",
        "https://www.ato.gov.au/law/view/document?DocID=SAV/FBTGEMP/00019",
    ),
    (
        "fbt_housing",
        "https://www.ato.gov.au/law/view/document?DocID=SAV/FBTGEMP/00011",
    ),
    (
        "fbt_lafha",
        "https://www.ato.gov.au/law/view/document?DocID=SAV/FBTGEMP/00012",
    ),
    (
        "fbt_board",
        "https://www.ato.gov.au/law/view/document?DocID=SAV/FBTGEMP/00014",
    ),
    (
        "fbt_tax_exempt_body_entertainment",
        "https://www.ato.gov.au/law/view/document?DocID=SAV/FBTGEMP/00016",
    ),
    (
        "fbt_entertainment_classifier",
        "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_1/document_1.html",
    ),
    (
        "income_tests",
        "https://www.ato.gov.au/individuals-and-families/income-deductions-offsets-and-records/income-tests",
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

/// The FBTAA as compiled on 1 July 2026 (compilation 97), the volume with
/// Divisions 4 to 10A, and the ATO pages the FBT benefit worksheets share.
const FBTAA_PART_1: &str = "https://www.legislation.gov.au/C2004A03280/2026-07-01/2026-07-01/text/original/epub/OEBPS/document_1/document_1.html";
const FBT_RETURN_ITEM_23: &str = "https://www.ato.gov.au/forms-and-instructions/fringe-benefits-tax-return-2026-instructions/item-23-fringe-benefit-categories-for-fbt-return-2026";
const FBT_RATES: &str =
    "https://www.ato.gov.au/tax-rates-and-codes/fringe-benefits-tax-rates-and-thresholds";
/// The FBTAA's volume with s 136, for the definitions the FBT benefit
/// worksheets apply, and the employer's guide chapter on reductions (19.5, the
/// in-house reduction), for the debt waiver, expense payment and property
/// worksheets.
const FBTAA_PART_2: &str = "https://www.legislation.gov.au/C2004A03280/2026-07-01/2026-07-01/text/original/epub/OEBPS/document_2/document_2.html";
const FBT_GUIDE_REDUCTIONS: &str =
    "https://www.ato.gov.au/law/view/document?DocID=SAV/FBTGEMP/00020";

pub const EXTRA_SOURCES: &[(&str, &[&str])] = &[
    (
        "contribution_caps",
        &[
            "https://www.ato.gov.au/individuals-and-families/super-for-individuals-and-families/super/growing-and-keeping-track-of-your-super/caps-limits-and-tax-on-super-contributions/concessional-contributions-cap",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/291-20",
        ],
    ),
    (
        "foreign_resident_tax",
        &[
            "https://www.ato.gov.au/tax-rates-and-codes/tax-rates-foreign-residents",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19360027/251S",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/61-110",
        ],
    ),
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
        "hire_purchase_schedule",
        &[
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/240-17",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/240-55",
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
        "fbt_loan",
        &[
            FBTAA_PART_1,
            "https://www.legislation.gov.au/C2004A03280/2026-07-01/2026-07-01/text/original/epub/OEBPS/document_2/document_2.html",
            FBT_RETURN_ITEM_23,
            FBT_RATES,
            "https://www.ato.gov.au/businesses-and-organisations/hiring-and-paying-your-workers/fringe-benefits-tax/types-of-fringe-benefits/loan-and-debt-waiver-fringe-benefits",
        ],
    ),
    (
        "fbt_meal_entertainment",
        &[FBTAA_PART_1, FBT_RETURN_ITEM_23],
    ),
    (
        "fbt_car_parking",
        &[
            FBTAA_PART_1,
            FBT_RETURN_ITEM_23,
            FBT_RATES,
            "https://www.ato.gov.au/businesses-and-organisations/hiring-and-paying-your-workers/fringe-benefits-tax/types-of-fringe-benefits/fbt-on-cars-other-vehicles-parking-and-tolls/car-parking-and-fbt",
        ],
    ),
    (
        "fbt_debt_waiver",
        &[
            FBTAA_PART_1,
            FBT_RETURN_ITEM_23,
            "https://www.ato.gov.au/businesses-and-organisations/hiring-and-paying-your-workers/fringe-benefits-tax/types-of-fringe-benefits/loan-and-debt-waiver-fringe-benefits",
        ],
    ),
    (
        "fbt_expense_payment",
        &[
            FBTAA_PART_1,
            FBTAA_PART_2,
            FBT_RETURN_ITEM_23,
            FBT_GUIDE_REDUCTIONS,
            "https://www.ato.gov.au/law/view/document?DocID=SAV/FBTGEMP/00018",
        ],
    ),
    (
        "fbt_property",
        &[
            FBTAA_PART_1,
            FBTAA_PART_2,
            FBT_RETURN_ITEM_23,
            FBT_GUIDE_REDUCTIONS,
        ],
    ),
    (
        "payg_withholding",
        &[
            "https://www.ato.gov.au/tax-rates-and-codes/schedule-1-tax-table-01-july-2024-to-30-june-2026",
        ],
    ),
    (
        "payg_gross_up",
        &[
            "https://www.ato.gov.au/tax-rates-and-codes/schedule-1-tax-table-01-july-2024-to-30-june-2026",
            "https://www.ato.gov.au/tax-rates-and-codes/schedule-8-statement-of-formulas-for-calculating-study-and-training-support-loans-components",
            "https://www.legislation.gov.au/F2025L01016/asmade/text",
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
            "https://www.legislation.gov.au/F2025L01016/asmade/text",
            "https://www.ato.gov.au/tax-rates-and-codes/schedule-1-tax-table-01-july-2024-to-30-june-2026",
        ],
    ),
    (
        "medicare_levy",
        &[
            "https://www.ato.gov.au/individuals-and-families/medicare-and-private-health-insurance/medicare-levy/medicare-levy-reduction/medicare-levy-reduction-for-low-income-earners",
            "https://www.ato.gov.au/individuals-and-families/your-tax-return/instructions-to-complete-your-tax-return/mytax-instructions/2025/medicare-and-private-health-insurance/medicare-levy-reduction-or-exemption",
            "https://www.ato.gov.au/individuals-and-families/income-deductions-offsets-and-records/income-you-must-declare/lump-sum-payment-in-arrears",
            "https://www.legislation.gov.au/C2026A00058/asmade/text",
            "https://www.ato.gov.au/individuals-and-families/medicare-and-private-health-insurance/medicare-levy/medicare-levy-reduction/medicare-levy-reduction-family-income",
            "https://www.legislation.gov.au/C2004A03351/2025-03-28/2025-03-28/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.legislation.gov.au/C2025A00028/asmade/text",
            "https://www.ato.gov.au/individuals-and-families/medicare-and-private-health-insurance/medicare-levy/medicare-levy-exemption/medical-exemption-from-medicare-levy",
            "https://www.ato.gov.au/individuals-and-families/medicare-and-private-health-insurance/medicare-levy/medicare-levy-exemption/not-entitled-to-medicare-benefits",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19360027/251U",
            "https://www.ato.gov.au/forms-and-instructions/individual-tax-return-2026-instructions/medicare-levy-questions-m1-m2-individual-tax-return-2026/m1-medicare-levy-reduction-or-exemption-2026",
        ],
    ),
    (
        "medicare_levy_surcharge",
        &[
            "https://www.ato.gov.au/forms-and-instructions/individual-tax-return-2026-instructions/medicare-levy-questions-m1-m2-individual-tax-return-2026/m2-medicare-levy-surcharge-2026",
            "https://www.legislation.gov.au/C2004A03351/2026-07-01/2026-07-01/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.legislation.gov.au/C2004A00412/2026-07-01/2026-07-01/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.legislation.gov.au/C2004A03351/2025-03-28/2025-03-28/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.legislation.gov.au/C2004A00412/2025-03-28/2025-03-28/text/original/epub/OEBPS/document_1/document_1.html",
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
    (
        "unused_leave_payment",
        &[
            "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_3/document_3.html",
            "https://www.ato.gov.au/businesses-and-organisations/hiring-and-paying-your-workers/single-touch-payroll/in-detail/single-touch-payroll-phase-2-employer-reporting-guidelines/when-an-employee-transfers-or-leaves",
            "https://www.ato.gov.au/businesses-and-organisations/super-for-employers/paying-super-on-payday/what-payments-are-qualifying-earnings",
            "https://www.ato.gov.au/forms-and-instructions/employment-termination-payments-withholding-from-unused-leave-payments/payments-made-after-death",
            "https://www.legislation.gov.au/C2004A04402/2026-07-01/2026-07-01/text/original/epub/OEBPS/document_1/document_1.html",
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
    (
        "seniors_pensioners_withholding",
        &[
            "https://www.ato.gov.au/tax-rates-and-codes/schedule-9-tax-table-01-july-2024-to-30-june-2026",
            "https://www.ato.gov.au/tax-rates-and-codes/payg-withholding-schedule-1-statement-of-formulas-for-calculating-amounts-to-be-withheld",
            "https://www.ato.gov.au/tax-rates-and-codes/study-and-training-support-loans-weekly-tax-table",
            "https://www.ato.gov.au/tax-rates-and-codes/schedule-8-statement-of-formulas-for-calculating-study-and-training-support-loans-components",
            "https://www.legislation.gov.au/F2025L01016/asmade/text",
        ],
    ),
    (
        "seniors_pensioners_tax_offset",
        &[
            "https://www.legislation.gov.au/F2025L01060/asmade/text",
            "https://www.legislation.gov.au/F2015L01415/2022-12-09/text",
            "https://www.legislation.gov.au/C1936A00027/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_3/document_3.html",
            "https://www.legislation.gov.au/C1936A00027/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.ato.gov.au/forms-and-instructions/individual-tax-return-2025-instructions/completing-the-individual-tax-return-2025/do-you-need-to-lodge-a-tax-return-2025",
        ],
    ),
    (
        "fbt_residual",
        &[FBTAA_PART_1, FBTAA_PART_2, FBT_RETURN_ITEM_23],
    ),
    (
        "fbt_housing",
        &[FBTAA_PART_1, FBTAA_PART_2, FBT_RETURN_ITEM_23, FBT_RATES],
    ),
    (
        "fbt_lafha",
        &[FBTAA_PART_1, FBTAA_PART_2, FBT_RETURN_ITEM_23],
    ),
    (
        "fbt_board",
        &[FBTAA_PART_1, FBTAA_PART_2, FBT_RETURN_ITEM_23],
    ),
    (
        "fbt_tax_exempt_body_entertainment",
        &[FBTAA_PART_1, FBTAA_PART_2, FBT_RETURN_ITEM_23],
    ),
    (
        "fbt_entertainment_classifier",
        &[
            "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_10/document_10.html",
            FBTAA_PART_1,
            FBTAA_PART_2,
            "https://www.legislation.gov.au/C1936A00027/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.legislation.gov.au/C2004A00446/2026-01-01/2026-01-01/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.legislation.gov.au/F2018L01284/2018-09-14/2018-09-14/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.ato.gov.au/law/view/document?docid=TXR/TR9717/NAT/ATO/00001",
            "https://www.ato.gov.au/law/view/document?docid=TXR/TR200712/NAT/ATO/00001",
        ],
    ),
    (
        "income_tests",
        &[
            "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_10/document_10.html",
            "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_6/document_6.html",
            "https://www.legislation.gov.au/C1936A00027/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.legislation.gov.au/C2004A01234/2026-10-17/2026-10-17/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.legislation.gov.au/C2004A00490/2026-07-01/2026-07-01/text/original/epub/OEBPS/document_1/document_1.html",
            FBTAA_PART_2,
            "https://www.legislation.gov.au/F1996B00580/2026-07-01/2026-07-01/text/original/epub/OEBPS/document_2/document_2.html",
            "https://www.legislation.gov.au/C2004A01195/2018-07-01/2018-07-01/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.ato.gov.au/forms-and-instructions/individual-tax-return-2026-instructions/supporting-information-individual-tax-return-2026/rebate-income-2026",
            "https://www.ato.gov.au/forms-and-instructions/individual-tax-return-2026-instructions/supporting-information-individual-tax-return-2026/adjusted-taxable-income-for-you-and-your-dependants-2026",
            "https://www.ato.gov.au/forms-and-instructions/individual-tax-return-2026-instructions/income-test-it1-it8-individual-tax-return-2026/it1-total-reportable-fringe-benefits-amounts-2026",
            "https://www.ato.gov.au/forms-and-instructions/individual-tax-return-2026-instructions/adjustment-questions-a1-a4-individual-tax-return-2026/a3-government-super-contributions-2026",
            "https://www.ato.gov.au/forms-and-instructions/individual-tax-return-2025-instructions/supporting-information-individual-tax-return-2025/rebate-income-2025",
            "https://www.ato.gov.au/forms-and-instructions/individual-tax-return-2025-instructions/supporting-information-individual-tax-return-2025/adjusted-taxable-income-for-you-and-your-dependants-2025",
            "https://www.ato.gov.au/forms-and-instructions/individual-tax-return-2025-instructions/income-test-it1-it8-individual-tax-return-2025/it1-total-reportable-fringe-benefits-amounts-2025",
            "https://www.ato.gov.au/forms-and-instructions/individual-tax-return-2025-instructions/adjustment-questions-a1-a4-individual-tax-return-2025/a3-government-super-contributions-2025",
            "https://www.ato.gov.au/individuals-and-families/medicare-and-private-health-insurance/medicare-levy-surcharge/medicare-levy-surcharge-income-thresholds-and-rates",
            "https://www.ato.gov.au/individuals-and-families/study-and-training-support-loans/when-must-you-repay-your-loan",
            "https://www.ato.gov.au/individuals-and-families/super-for-individuals-and-families/super/growing-and-keeping-track-of-your-super/caps-limits-and-tax-on-super-contributions/division-293-tax-on-concessional-contributions-by-high-income-earners",
            FBT_RATES,
        ],
    ),
];

/// The car worksheets' reportable fringe benefits note, read from the ATO's
/// reportable fringe benefits page on 30 October 2026. It follows the 2
/// warnings every worksheet carries.
const CAR_REPORTABLE_NOTE: &str = "reportable_taxable_value is this car benefit's taxable value before any exemption, not a reportable fringe benefits amount. The employee's reportable fringe benefits amount is the total reportable taxable value of all their reportable fringe benefits for the FBT year multiplied by the type 2 (lower) gross-up rate, when that total exceeds $2,000. A car benefit from the private use of a pooled or shared car is not reportable. This worksheet determines neither.";

/// Warnings added after the original's 2, by worksheet.
/// Which gross-up an expense payment or property benefit takes (the
/// employer's guide, 2.5 and 2.6).
const GST_CREDITABLE_NOTE: &str = "Give taxable_value to fbt as part of type_one_value when the provider, or a member of its GST group, is entitled to a GST credit for providing the benefit (a GST-creditable benefit), and as part of type_two_value otherwise.";

pub const EXTRA_WARNINGS: &[(&str, &[&str])] = &[
    ("fbt_car_statutory", &[CAR_REPORTABLE_NOTE]),
    (
        "hire_purchase_schedule",
        &[
            "interest_by_income_year totals only the months wholly in each income year. A month spanning 30 June is listed in interest_across_30_june with its days on each side, not split: s 240-60(2) relates the part of a month that falls in an income year to it in accordance with generally accepted accounting principles, which this worksheet does not apply.",
        ],
    ),
    ("fbt_car_operating_cost", &[CAR_REPORTABLE_NOTE]),
    (
        "fbt_loan",
        &[
            "A loan fringe benefit gives the employer no GST credit, so it is a type 2 benefit: give taxable_value to fbt as part of type_two_value.",
        ],
    ),
    (
        "fbt_debt_waiver",
        &[
            "A debt waiver fringe benefit gives the employer no GST credit, so it is a type 2 benefit: give taxable_value to fbt as part of type_two_value.",
        ],
    ),
    ("fbt_expense_payment", &[GST_CREDITABLE_NOTE]),
    ("fbt_property", &[GST_CREDITABLE_NOTE]),
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
    (
        "unused_leave_payment",
        &[
            "Withholding, not the tax payable: on assessment assessable_income is assessable, a tax offset limits the rate of tax on offset_capped_income to 30% (ITAA 1997 ss 83-15 and 83-85), and the Medicare levy applies; the rest is taxed at marginal rates.",
            "qualifying_earnings is the ATO's table for super guarantee from 1 July 2026; an award or agreement can still require super on these payments.",
        ],
    ),
    (
        "fbt_board",
        &[
            "A board fringe benefit is a type 2 benefit (the employer's guide, 13.2): give taxable_value to fbt as part of type_two_value.",
        ],
    ),
    (
        "fbt_entertainment_classifier",
        &[
            "TR 97/17 and TR 2007/12 are public rulings: the Commissioner's view of the law, not the law, and their paragraphs are cited as that view. FBTAA s 58P(1)(e) requires a minor benefit's notional taxable value to be less than $300, which this classifier applies; the $100 threshold in TR 97/17 para 27 is superseded and not applied.",
            "A minor benefit is shown only as possible: whether it would be unreasonable to treat it as a fringe benefit (FBTAA s 58P(1)(f)) is the employer's judgement, and the totals treat it as a fringe benefit. undetermined_cost is the cost left out of the totals for want of a stated fact.",
        ],
    ),
];

/// A `foreign_resident_tax` warning for a year whose rates the ATO had not
/// published when they were read from the Act.
pub const FOREIGN_RESIDENT_YEAR_WARNINGS: &[(&str, &str)] = &[
    (
        "2026-27",
        "2026-27 uses the rates Income Tax Rates Act 1986 Schedule 7 Part II sets for the 2024-25 and later years, with the second and third resident personal tax rates for 2026-27 (30% and 37%), as compiled at 1 July 2026 (compilation 66). The ATO had not published 2026-27 foreign resident rates when this was checked on 14 November 2026, and a later Act can change them for 2026-27 and change this result.",
    ),
    (
        "2027-28",
        "2027-28 uses the rates Income Tax Rates Act 1986 Schedule 7 Part II sets for the 2024-25 and later years, with the second and third resident personal tax rates for 2027-28 (30% and 37%), as compiled at 1 July 2026 (compilation 66). The ATO had not published 2027-28 foreign resident rates when this was checked on 16 November 2026, and a later Act can change them for 2027-28 and change this result.",
    ),
];

/// A `resident_tax` warning for a year whose rates the ATO had not published
/// when they were read from the Act.
pub const RESIDENT_TAX_YEAR_WARNINGS: &[(&str, &str)] = &[(
    "2027-28",
    "2027-28 uses the table Income Tax Rates Act 1986 Schedule 7 Part I sets for the 2027-28 and later years (14%, 30%, 37% and 45%), as compiled at 1 July 2026 (compilation 66). The ATO had not published 2027-28 resident rates when this was checked on 16 November 2026, and a later Act can change them for 2027-28 and change this result. From 2027-28 a resident can also owe extra income tax on a minimum tax capital gain (ITAA 1997 s 119-10, inserted by Act No. 49 of 2026), which this worksheet does not work out.",
)];

/// A `medicare_levy` warning for a year whose thresholds the ATO had not
/// published when they were read from the Act.
pub const MEDICARE_YEAR_WARNINGS: &[(&str, &str)] = &[
    (
        "2026-27",
        "2026-27 uses the threshold amounts, phase-in limits and family income thresholds Medicare Levy Act 1986 ss 3(1) and 8 set for 2025-26 and later years (Act No. 58, 2026, Sch 5 item 14). The ATO had not published 2026-27 thresholds when this was checked on 14 November 2026. That Act, assented on 30 June 2026, raised the figures for 2025-26, so a later Act can raise them for 2026-27 and change this result.",
    ),
    (
        "2027-28",
        "2027-28 uses the threshold amounts, phase-in limits and family income thresholds Medicare Levy Act 1986 ss 3(1) and 8 set for 2025-26 and later years (Act No. 58, 2026, Sch 5 item 14), as compiled at 1 July 2026 (compilation 53). The ATO had not published 2027-28 thresholds when this was checked on 16 November 2026, and a later Act can raise them for 2027-28 and change this result.",
    ),
];

/// An `income_tests` warning when exempt foreign employment income is given
/// with taxable income of $1 or more.
pub const INCOME_TESTS_EXEMPT_FOREIGN_NOTE: &str = "income_for_surcharge_purposes leaves out exempt_foreign_employment_income, as ITAA 1997 s 995-1 'income for surcharge purposes' (compilation 268) does. The ATO's 'Medicare levy surcharge income, thresholds and rates' page (last updated 22 June 2026) adds it where taxable income is $1 or more. Neither the Medicare Levy Act 1986 (compilation 53) nor the A New Tax System (Medicare Levy Surcharge—Fringe Benefits) Act 1999 (compilation 42) adds it: s 3(1) of each gives the term s 995-1's meaning. Only the lump sum offset's one-eleventh test (ITAA 1997 s 61-580(1)(d)) counts it beside those amounts. The ATO's Division 293 page leaves it out. Confirm with the ATO before using income_for_surcharge_purposes for the surcharge.";

/// A `seniors_pensioners_tax_offset` warning for a year whose thresholds the
/// ATO had not published when they were worked from the regulations.
pub const SAPTO_YEAR_NOTES: &[(&str, &str)] = &[
    (
        "2026-27",
        "2026-27 thresholds are worked from the Income Tax Assessment (1936 Act) Regulations 2025 ss 9 and 10 with the 15% lowest marginal tax rate the Income Tax Rates Act 1986 sets for 2026-27. The ATO had published thresholds only to 2025-26 when this was checked on 14 November 2026.",
    ),
    (
        "2027-28",
        "2027-28 thresholds are worked from the Income Tax Assessment (1936 Act) Regulations 2025 ss 9 and 10 with the 14% lowest marginal tax rate the Income Tax Rates Act 1986 sets for 2027-28. The ATO had published thresholds only to 2025-26 when this was checked on 16 November 2026.",
    ),
];

const WITHHOLDING_2027_28: &str = "The ATO had not published withholding schedules for payments from 1 July 2027 when this was checked on 16 November 2026.";
const CGT_2027_28: &str = "From 2027-28 Schedule 1 to Act No. 49 of 2026 changes how capital gains are worked out, and this worksheet has not been checked against it.";
const RENTAL_2027_28: &str = "From 2027-28 ITAA 1997 s 26-155 limits deductions for residential rental property, and this worksheet does not apply it.";

/// Why a worksheet that takes an income year refuses 2027-28, added to its
/// `period.unsupported` message: the figures not yet published, or the change
/// in the law it does not apply.
pub const UNPUBLISHED_2027_28: &[(&str, &str)] = &[
    ("capital_gains", CGT_2027_28),
    ("main_residence", CGT_2027_28),
    ("borrowing_expenses", RENTAL_2027_28),
    (
        "depreciation",
        "2027-28 has 366 days, and this worksheet takes at most 365 days held.",
    ),
    ("payg_withholding", WITHHOLDING_2027_28),
    ("payg_gross_up", WITHHOLDING_2027_28),
    ("study_loan_withholding", WITHHOLDING_2027_28),
    ("schedule_5_method_a", WITHHOLDING_2027_28),
    ("schedule_5_method_b_i", WITHHOLDING_2027_28),
    ("schedule_5_method_b_ii", WITHHOLDING_2027_28),
    ("working_holiday_maker_withholding", WITHHOLDING_2027_28),
    ("seniors_pensioners_withholding", WITHHOLDING_2027_28),
    ("unused_leave_payment", WITHHOLDING_2027_28),
    (
        "termination_payment",
        "The 2027-28 ETP cap amounts and genuine redundancy tax-free limits had not been published when this was checked on 16 November 2026.",
    ),
    (
        "contribution_caps",
        "The 2027-28 contribution caps had not been published when this was checked on 16 November 2026.",
    ),
    (
        "study_loan_repayment",
        "The 2027-28 study and training loan repayment thresholds had not been published when this was checked on 16 November 2026.",
    ),
    (
        "medicare_levy_surcharge",
        "The 2027-28 Medicare levy surcharge income thresholds had not been published when this was checked on 16 November 2026.",
    ),
    (
        "division_296",
        "The 2027-28 indexed superannuation balance thresholds had not been published when this was checked on 16 November 2026.",
    ),
];

/// A `medicare_levy` warning when s 8(3) shares the family reduction.
pub const MEDICARE_SHARED_REDUCTION_NOTE: &str = "The family reduction is shared in proportion to taxable income (s 8(3)) and worked out exactly; each amount is rounded half up to cents only for display, so the displayed levy can differ by a cent from the displayed amounts it is worked out from.";

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
        "contribution_caps.defined_benefit_interest",
        "A person with a defined benefit interest was given to the carry-forward by year (ITAA 1997 Subdivision 291-C and s 291-370).",
    ),
    (
        "contribution_caps.earlier_balance_missing",
        "An earlier year's concessional contributions exceed its cap and its total superannuation balance at the 30 June before it was not given (ITAA 1997 s 291-20(3)(b)).",
    ),
    (
        "contribution_caps.earlier_years_invalid",
        "The earlier years' concessional contributions are not one entry for each year from the 5th before the year to the one before it, without a gap.",
    ),
    (
        "contribution_caps.earlier_years_short",
        "An earlier year's cap was increased from unused cap of years before the first year given (ITAA 1997 s 291-20(4) and (5)).",
    ),
    (
        "contribution_caps.year_before_carry_forward",
        "An earlier year before 2018-19 was given, which has no unused concessional contributions cap (ITAA 1997 s 291-20(7)).",
    ),
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
        "estimate.family_mismatch",
        "personal-tax-estimate was given parts that describe different families: a spouse in one but not another, 2 spouse taxable incomes, or dependants without a spouse.",
    ),
    (
        "estimate.residency_unsupported",
        "personal-tax-estimate was given a part-year resident or a working holiday maker, or a foreign resident with a part it composes for a resident only.",
    ),
    (
        "estimate.scale_mismatch",
        "personal-tax-estimate was given a withholding scale for a foreign resident with a resident, or one for a resident with a foreign resident.",
    ),
    (
        "estimate.surcharge_with_exemption",
        "personal-tax-estimate was given Medicare levy exemption days beside a Medicare levy surcharge input, whose scope excludes them.",
    ),
    (
        "fbt.deduction_exceeds_contribution",
        "A deduction for a recipient's contribution to a property, residual or board fringe benefit exceeds the contribution.",
    ),
    (
        "fbt.in_house_reduction_invalid",
        "An in-house reduction left for a benefit is more than $1,000.",
    ),
    (
        "fbt.register_not_valid",
        "A 12 week register's first valid FBT year is not the year given or one of the 4 before it.",
    ),
    (
        "fbt_car.cost_basis_conflict",
        "A car has neither or both of a depreciated value and lease charges.",
    ),
    (
        "fbt_car_parking.head_counts_invalid",
        "A spaces method's employees or eligible spaces at the start or end of the parking period are not a JSON integer from 0 to 1,000,000.",
    ),
    (
        "fbt_car_parking.space_groups_invalid",
        "A spaces method's space groups are not a list of 1 to 1,000,000 objects each with exactly a first business day fee, a daily rate amount, availability days and spaces.",
    ),
    (
        "fbt_car_parking.spaces_without_benefit",
        "A spaces method's space group has a first business day fee not more than the car parking threshold, so its spaces gave rise to no car parking benefit.",
    ),
    (
        "fbt_entertainment_classifier.fact_invalid",
        "An entertainment line's stated fact (what was provided, the occasion, venue, payer or attendees) is not one the classifier reads.",
    ),
    (
        "fbt_entertainment_classifier.gst_status_invalid",
        "An entertainment line's cost does not state whether it includes GST.",
    ),
    (
        "fbt_entertainment_classifier.line_not_split",
        "An entertainment line's attendees span more than one class without a head count for each, so its cost cannot be split between them.",
    ),
    (
        "fbt_entertainment_classifier.lines_invalid",
        "The entertainment lines are not a list of 1 to 10,000 line objects.",
    ),
    (
        "fbt_entertainment_classifier.method_required",
        "A line is meal entertainment and the employer's meal entertainment valuation method is not stated.",
    ),
    (
        "fbt_entertainment_classifier.register_percentage_invalid",
        "Under the 12 week register method, the income tax register percentage is absent or outside 0 to 100.",
    ),
    (
        "fbt_entertainment_classifier.tax_exempt_body",
        "The employer is exempt from income tax, so fbt-tax-exempt-body-entertainment applies.",
    ),
    (
        "fbt_expense_payment.deduction_exceeds_portion",
        "The deduction for the expenditure an expense payment's recipient bore exceeds the recipient's portion.",
    ),
    (
        "fbt_expense_payment.in_house_deductible_not_covered",
        "An income use share or recipient's portion deduction was given for an in-house expense payment, whose otherwise deductible rule (FBTAA s 24(1)(b)(i)) the worksheet does not cover.",
    ),
    (
        "fbt_expense_payment.payment_exceeds_expenditure",
        "An expense payment or reimbursement exceeds the recipient's expenditure.",
    ),
    (
        "fbt_housing.base_year_invalid",
        "A housing right's last base year is not one of the 9 FBT years before the year given.",
    ),
    (
        "fbt_lafha.components_exceed_allowance",
        "A living-away-from-home allowance's accommodation and food components together exceed the allowance.",
    ),
    (
        "fbt_loan.deduction_exceeds_interest",
        "The deduction for the interest accrued on a loan exceeds that interest.",
    ),
    (
        "fbt_meal_entertainment.register_ceased",
        "A year since a meal entertainment register's first valid year had expenses more than 20% higher than that year's, so the register ceased to be valid (FBTAA s 37CD(3)).",
    ),
    (
        "fbt_meal_entertainment.register_values_invalid",
        "A meal entertainment register's total is nil or less than its fringe benefits' value.",
    ),
    (
        "fbt_meal_entertainment.register_years_invalid",
        "A meal entertainment register's yearly expenses are not a list of decimal strings, one for each FBT year from its first valid year to the year before the year given.",
    ),
    (
        "fbt_tax_exempt_body_entertainment.heads_invalid",
        "The employees and associates entertained are not a JSON integer from 0 to everyone entertained, or everyone entertained is not from 1 to 1,000,000.",
    ),
    (
        "hire_purchase.amount_nil",
        "A hire purchase agreement's amount financed or instalment is nil.",
    ),
    (
        "hire_purchase.instalments_unsupported",
        "The number of monthly instalments is not an integer from 1 to 360.",
    ),
    (
        "hire_purchase.rate_imprecise",
        "The implicit rate cannot be found closely enough to leave a nil balance to the cent, at a high rate over a long term.",
    ),
    (
        "hire_purchase.rate_unsupported",
        "The payments imply an interest rate below nil or of 100% a month or more.",
    ),
    (
        "hire_purchase.start_outside_year",
        "A hire purchase agreement starts outside the income year given.",
    ),
    (
        "income_tests.components_inconsistent",
        "income-tests was given a component larger than the amount that includes it: taxable income, the assessable FHSS released amount or excess concessional contributions above assessable income, or exempt foreign employment income above target foreign income.",
    ),
    (
        "income_tests.residency_unsupported",
        "income-tests was given a part-year resident, a working holiday maker or a foreign resident.",
    ),
    (
        "input.abn_invalid",
        "An ABN is not a string of 11 ASCII digits, optionally spaced, that pass ABN Lookup's check digit test.",
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
        "input.benefit_count_invalid",
        "A number of fringe benefits is not a JSON integer from 0.",
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
        "input.dependants_invalid",
        "A count of dependent children is not a non-negative JSON integer.",
    ),
    (
        "input.depreciation_method_invalid",
        "The method is neither prime_cost nor diminishing_value.",
    ),
    (
        "input.donations_invalid",
        "Donations are not a list of 1 to 10,000 objects.",
    ),
    (
        "input.effective_life_invalid",
        "An effective life is outside 0.01 to 1000 years.",
    ),
    (
        "input.fbt_method_invalid",
        "An FBT benefit worksheet's method is not one it offers.",
    ),
    (
        "input.field_unknown",
        "A field the worksheet or command does not take, or does not take beside the other inputs given, was given (fbt-loan, fbt-meal-entertainment, fbt-car-parking, fbt-debt-waiver, fbt-expense-payment, fbt-property, fbt-residual, fbt-housing, fbt-lafha, fbt-board, fbt-tax-exempt-body-entertainment, fbt-entertainment-classifier, division-296, depreciation, foreign-resident-tax, medicare-levy, medicare-levy-surcharge, payg-withholding, payg-gross-up, termination-payment, unused-leave-payment, the Schedule 5 methods, working-holiday-maker-withholding, study-loan-withholding, seniors-pensioners-withholding, seniors-pensioners-tax-offset, hire-purchase-schedule, contribution-caps given a carry-forward input, derive, personal-tax-estimate, dgr-check and income-tests only).",
    ),
    ("input.flag_invalid", "A flag is not a JSON boolean."),
    ("input.fraction_invalid", "A fraction is outside 0 to 1."),
    (
        "input.integer_too_long",
        "A JSON integer has more than 4,300 digits.",
    ),
    ("input.invalid_json", "The input is not JSON."),
    ("input.kind_invalid", "A worksheet kind is not a string."),
    (
        "input.medicare_levy_exemption_invalid",
        "A Medicare levy exemption is not none, full or half.",
    ),
    ("input.missing_field", "A required argument is absent."),
    (
        "input.name_invalid",
        "A name is not a string of 1 to 200 characters with an ASCII letter or digit.",
    ),
    ("input.not_object", "The JSON input is not an object."),
    (
        "input.opening_value_above_cost",
        "An opening adjustable value is more than the asset's cost.",
    ),
    (
        "input.pay_period_invalid",
        "The pay period is not weekly, fortnightly or monthly (or quarterly, for payg-withholding and study-loan-withholding).",
    ),
    (
        "input.pay_periods_invalid",
        "A count of pay periods is outside the range the method or the income year allows for the pay period.",
    ),
    ("input.quarter_invalid", "A quarter is not 1, 2, 3 or 4."),
    (
        "input.residency_invalid",
        "A residency is not full_year_resident, part_year_resident, working_holiday_maker or foreign_resident.",
    ),
    (
        "input.signed_amount_invalid",
        "An amount that may be negative is not a finite decimal of at most 2dp within 1e12 of nil.",
    ),
    (
        "input.state_invalid",
        "A State or Territory is not one of NSW, VIC, QLD, SA, WA, TAS, NT and ACT.",
    ),
    (
        "input.status_invalid",
        "A senior's or pensioner's status is not single, illness_separated or couple.",
    ),
    (
        "input.tax_scale_unsupported",
        "The withholding scale is not 1, 2, 3, 5 or 6 (or 4, for payg-withholding), or has no table for the period.",
    ),
    (
        "input.termination_reason_invalid",
        "A termination reason is not normal, genuine_redundancy, early_retirement_scheme, invalidity or death.",
    ),
    (
        "input.timing_invalid",
        "A payment timing is not in_arrears or in_advance.",
    ),
    ("input.too_large", "The JSON input exceeds 8 MiB."),
    ("input.unreadable", "Standard input could not be read."),
    (
        "input.years_of_service_invalid",
        "A count of completed years of service is not a whole number from 0 to 150.",
    ),
    (
        "leave.accrual_split_invalid",
        "A part of unused leave accrued before a date is more than its total, or is given above nil though the leave's accrual started on or after that date.",
    ),
    (
        "leave.accrual_split_missing",
        "A part of unused leave accrued before 16 August 1978 or 18 August 1993 is not given where the accrual start and termination reason make it change the tax.",
    ),
    (
        "leave.death",
        "Unused leave paid after an employee's death, which is not withheld from.",
    ),
    (
        "leave.under_300_with_pre_1978",
        "A post-17 August 1993 payment under $300 beside pre-16 August 1978 long service leave, which Schedule 7 does not say how to limit.",
    ),
    (
        "leave.working_holiday_maker",
        "Unused leave paid to a working holiday maker, whom Schedule 15 covers.",
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
        "Residency was given for a scale other than 4, which settles it itself, or beside a study loan, which scale 4 cannot have.",
    ),
    (
        "payg.gross_up_net_too_large",
        "A net pay to gross up is so large that a gross leaving it could exceed the largest amount the withholding worksheet takes.",
    ),
    (
        "payg.gross_up_not_unique",
        "More than one gross pay leaves the net pay given; the refusal lists them.",
    ),
    (
        "payg.medicare_adjustment_not_allowed",
        "A Medicare levy adjustment was claimed on a scale other than 2 or 6.",
    ),
    (
        "payg.pay_date_before_schedule",
        "The pay date is before the first payment the year's Schedule 8 coefficients apply to.",
    ),
    (
        "payg.pay_date_outside_year",
        "The pay date is outside the income year given.",
    ),
    (
        "payg.pay_date_without_study_loan",
        "A Schedule 5 or Schedule 9 pay date was given without a study loan, the only part it selects.",
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
