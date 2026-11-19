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
        "One hire purchase agreement whose hirer is the notional buyer (ITAA 1997 s 240-17(2)), starting in the income year given, with instalments paid in arrears (at each period's end) or in advance (at its start) and any balloon paid at the end of the term. The instalments are one level amount weekly, fortnightly, monthly, quarterly or annually, or dated segments of level amounts (a nil amount for a payment holiday), all counted in weeks or all in months, over at most 30 years. The operator establishes the amount financed (the notional loan principal: the consideration less any deposit paid or trade-in credited at or before the start, s 240-25(3)), the instalments, the balloon, the start date and any deposit, trade-in, payout and GST facts. A deposit or trade-in is taken as paid or credited on the start date, and the goods and the credit component as recorded on one tax invoice. Months run from the start date's day of the month, or the month's last day when it is shorter; weeks are 7 days. Each instalment's period is an arrangement payment period (s 240-70(1)), and an annual period is 2 periods of 6 months (s 240-70(2)). Under s 240-60(1) each period's notional interest is the outstanding notional loan principal, less any instalment due at the period's start, times the implicit interest rate for the period: (1 + r)^n - 1 for a period of n months (or weeks), where r is the compound rate a month (or a week) at which the payments repay the amount financed, found to 64 halvings of 0 to 100%. The payments themselves are not deductible but are taken into account in the notional interest (s 240-55). Each period shows the notional interest to date rounded half up to cents, less what the periods before show, so the periods add up to the total charges, each is within a cent of its amount at the rate found and the last balance is nil. The balloon is shown apart from the last instalment, on its own date. Journal lines for the start, each period's interest, each payment on its day and any payout carry the liability at the notional loan principal outstanding, and are totalled by income year: the interest by its period, with the periods spanning 30 June set apart, and each payment by its day. An early payout on the end of a period, neither known nor expected at the start, replaces every payment due from its date: the worksheet gives the notional interest and payments to that date, the balance outstanding, the difference and, under Subdivision 240-G (ss 240-105 and 240-110), which way the difference goes, flagged for review. With every GST fact given, it gives a registered hirer's input tax credit for an agreement entered into from 1 July 2012: 1/11 of every component of the supply, the notional interest included (the ATO's 'GST – Hire purchase and leasing'), less any luxury car tax for a car (GST Act s 9-75(2)), rounded to the nearest cent, 0.5 up (s 9-90), times the extent of creditable purpose (s 11-30(3)), attributable up front whether or not the hirer accounts on a cash basis (ss 29-10(1) and 158-5). A passenger car whose GST inclusive market value exceeds the car limit for the financial year it is first used ($69,674 for 2024-25 and 2025-26, $69,883 for 2026-27), where the hirer may not quote an ABN for luxury car tax, gives no GST figure and is flagged for review: s 69-10(1) caps the credit on the car, and the sources read do not settle whether the cap takes in the GST on the credit component. Refuses a rate below nil or of 100% a month (or week) or more, a rate that cannot be found closely enough to leave a nil balance to the cent, a term over 30 years, a nil amount financed or instalment, segments that do not run on from the start or mix weeks and months, a payout not on the end of a period before the term ends, a payout with GST facts, a payout known or expected at the start, a car first used in a year whose car limit is not published, and luxury car tax above the price. Excludes deciding whether Division 240 applies and what is financed; fees, charges and stamp duty; payments off a period boundary; substituted accounting periods; termination amounts other than the balloon and a stated payout, and estimates of them (s 240-60(3) and (4)); apportioning a period that spans 30 June between income years (s 240-60(2)); whether the notional interest is deductible (s 240-50) and any reduction of it for the input tax credit; the car's cost for Division 40 (ss 27-80 and 40-230); agreements entered into before 1 July 2012; GST adjustments after the start; the trade-in's own GST; and the notional seller.",
    ),
    (
        "gst_margin_scheme",
        "One taxable supply of real property in a financial year from 2025-26 to 2027-28, made under a contract entered into on or after 9 December 2008 and not under an option or right granted before then: whether the margin scheme of GST Act Division 75 can apply, and the GST with and without it. The operator establishes that the supply is a taxable supply apart from Division 75 (s 9-5: such as new residential premises or land sold in the course of an enterprise, not input taxed residential premises or a GST-free supply), what it sells or grants, the written agreement, how the supplier acquired the entire interest, unit or lease, and the base: the consideration for that acquisition and any of it unpaid, or an approved valuation (s 75-35) with the facts that fix its day. Each condition of s 75-5 is shown as met or not: a freehold interest, a stratum unit or a long-term lease (s 75-5(1)); a written agreement made on or before the supply or within a further period the Commissioner allows (s 75-5(1A)); and the interest not acquired through a taxable supply on which the GST was worked out without the margin scheme (s 75-5(2) and (3)(a)). The margin is the amount by which the consideration for the supply exceeds the base, or nil (s 75-10(2) and (3)): the acquisition's consideration less any of it unpaid (s 75-12), disregarding improvements and development (s 75-14), or for an interest acquired before 1 July 2000 an approved valuation as at the day s 75-10(3) items 1 to 4 fix, checked against the facts given; for a supply of part of the land, only the corresponding proportion of the base, the share of the whole given (s 75-15). The GST under the margin scheme is 1/11 of the margin (s 75-10(1)) and without it 1/11 of the consideration (ss 9-70 and 9-75(1)), each rounded to the nearest cent, 0.5 cents up, as for one supply on one invoice (s 9-90(1)); gst is the first when every condition is met and the second otherwise. Refuses an acquisition from a fellow GST group member, a GST joint venture operator or an associate, by inheritance, as a GST-free going concern or farm land, through a supply GST-free under s 38-445(1A), or in part through an ineligible supply (ss 75-5(3)(b) to (g), 75-10(3) item 2A, 75-11 and 75-22), a supply to an associate (s 75-13), an approved valuation for an interest acquired from 1 July 2000, a valuation day s 75-10(3) does not fix for the facts, a supply items 2 and 4 both describe, unpaid consideration above the consideration and a share above the whole. Excludes deciding whether the supply is taxable, the agreement's terms, the valuation and its compliance with the Commissioner's requirements, the reasonableness of the apportionment, the time of the supply, GST at settlement withholding (Schedule 1 to the Taxation Administration Act 1953 s 14-250), the increasing and decreasing adjustments of ss 75-22 and 75-27, contracts before 9 December 2008, and several supplies recorded on one invoice (s 9-90(2)).",
    ),
    (
        "chattel_mortgage_schedule",
        "One loan to buy goods, secured by a chattel mortgage over them and made on the day of the purchase in the income year given, under which the purchaser takes title at the purchase, so the agreement is not a hire purchase agreement (ITAA 1997 s 995-1) and Division 240 does not apply (s 240-10). The operator establishes the amount financed (paid to the supplier for the goods), the instalments, the balloon, the start date, any deposit and trade-in (taken as paid or credited on the start date, with the goods on one tax invoice) and any GST facts, and either the instalment or a stated annual rate. The instalments are one level amount weekly, fortnightly, monthly, quarterly or annually, in arrears (at each period's end) or in advance (at its start), with any balloon paid at the end of the term, over at most 30 years; months run from the start date's day of the month (or the month's last day when it is shorter) and weeks are 7 days. Each period's interest is the balance owing, less any instalment due at the period's start, times the rate for the period (the actuarial method, TR 93/16 para 18). Given the instalment, the rate is the compound rate a month (or week) at which the payments repay the amount financed, found to 64 halvings of 0 to 100%, and a period of n months (or weeks) has the rate (1 + r)^n - 1. Given annual_rate, for monthly or weekly instalments in arrears, the rate a period is annual_rate / 12 (or / 52), the instalment is the level one that repays the amount financed with the balloon at that rate, rounded half up to the cent, and the balance that rounding leaves, rounded to the cent, is added to the last instalment. Each period shows the interest to date rounded half up to cents, less what the periods before show, so the periods add up to the total interest and the last balance is nil. The interest is totalled by income year for the periods wholly in each year; a period spanning 30 June is listed apart with its days in each year. Interest charged is deductible under s 8-1 to the extent the borrowed money is used to gain or produce assessable income and is not capital, private or domestic; where the lender's statements give the interest, that figure is used (TR 93/16 para 2). With every GST fact given, it gives a registered purchaser's input tax credit on the price (the amount financed, deposit and trade-in): 1/11 of the price, less any luxury car tax for a car (GST Act s 9-75(2)), rounded to the nearest cent, 0.5 up (s 9-90(1)); for a passenger car whose GST inclusive market value exceeds the car limit for the financial year it is first used ($69,674 for 2024-25 and 2025-26, $69,883 for 2026-27), where the purchaser may not quote an ABN for luxury car tax, no more than 1/11 of that limit (s 69-10(1)); times the extent of creditable purpose (ss 11-30(3) and 69-10(3)). The interest gives no credit, as the loan is an input taxed financial supply (s 40-5(1)). The credit is attributable to the tax period of the purchase: on an accruals basis the earlier of the invoice and any payment (s 29-10(1)), on a cash basis the payment of the full price with the borrowed funds (s 29-10(2)(a)), as the ATO's GST issues register 'Financial services - questions and answers' (issue 1.21) states. Refuses both or neither of instalment and annual_rate, annual_rate with instalments in advance or other than monthly or weekly, a rate below nil or of 100% a month (or week) or more, a rate that cannot be found closely enough to leave a nil balance to the cent, a term over 30 years, a nil amount financed or instalment, a balloon that alone repays the loan at the stated rate, a rounded instalment that moves the last instalment by more than a whole instalment (below nil or above twice the instalment), a car first used in a year whose car limit is not published, and luxury car tax above the price. Excludes deciding whether the agreement is a chattel mortgage, the extent of income use and creditable purpose; fees, charges and stamp duty; irregular or changing instalments and rates (loan_transactions takes dated rates); interest charged daily; an early payout; substituted accounting periods; apportioning a period that spans 30 June between income years; the goods' decline in value and their cost for Division 40 (ss 27-80 and 40-230); the trade-in's own GST; GST adjustments after the purchase; and the lender.",
    ),
    (
        "luxury_car_tax",
        "Luxury car tax on one tax period's taxable supplies and taxable importations of luxury cars in a financial year from 2019-20 to 2026-27, by an entity registered or required to be registered for GST, with its price and change of use adjustments and the primary producer and tourism operator refunds (A New Tax System (Luxury Car Tax) Act 1999). The operator establishes each line: that the vehicle is a car (s 27-1) and not one s 25-1(2) excludes (emergency vehicles, cars fitted out for wheelchairs, commercial vehicles not designed mainly to carry passengers, motor homes and campervans), that the supply or importation is taxable (no quote, s 9-5; not GST-free, input taxed or otherwise exempt), its luxury car tax value including GST and customs duty but not luxury car tax (ss 5-20 and 7-15(2)), and its combined fuel consumption rating. The threshold is the luxury car tax threshold for the financial year of the supply or importation, or the fuel-efficient car limit for a car rated at most 3.5 litres per 100 km for supplies and importations from 1 July 2025 (7 litres before, and after where item 4(2) of Schedule 1 to Act No. 29 of 2025 continues it), from the ATO's published table (QC38161) for 2012-13 to 2026-27. The tax is 33% x 10/11 x (value - threshold), nil at or below the threshold, rounded half up to the cent (s 5-15(1)); a sale's is less tax payable on an earlier supply or importation of the car, not below nil (s 5-15(2)). Importation tax is shown apart, as it is paid to the Department of Home Affairs. A price adjustment is the corrected tax on the sale against the tax previously attributed (ss 15-20 and 15-25); a change of use adjustment is the tax that would have been payable or was payable on the supply (s 15-30(1) to (4)). Label 1E (sales and increasing adjustments) and label 1F (decreasing adjustments) are rounded down to whole dollars. A refund is the lesser of the tax borne and $10,000 on a refund-eligible car supplied or imported from 1 July 2019 (ss 18-5 and 18-10), one car a financial year for a primary producer. Refuses a year outside 2019-20 to 2026-27, a supply year outside the table, a refund for a car supplied before 1 July 2019 and a second primary producer refund. Excludes deciding whether a vehicle is a car or a luxury car, quoting, the luxury car tax value of a car with other goods or of a demonstrator, whether a car is refund-eligible or the refund conditions are met, the s 15-30(1A) and (3A) adjustments for cars imported for the purposes of s 7-10(3)(ba), bad debts (Subdivision 15-C), attribution of a line to a tax period, deferred luxury car tax on importations and the refund claim itself, made on the approved form and not the BAS.",
    ),
    (
        "loan_transactions",
        "One loan's statement from start_date to end_date, starting in the income year given and ending less than 30 years later. The operator establishes the balance owing at the start, the annual interest rate from the start and each dated change to it, and each drawing and repayment with its date and amount. A day's rate change and transactions take effect on that day. Each day's interest is the balance owing at the end of the day, when above nil, times the annual rate in force that day divided by 365, in leap years too; the interest accrued is debited to the loan on the last day of each month and on end_date, rounded half up to the cent, so each income year's interest is debited by its 30 June and the interest by income year is the interest debited in it. The statement lists the opening balance, each rate change, drawing, repayment and interest debit with the running balance, as rows and as a printable table. Interest charged on money borrowed is deductible under ITAA 1997 s 8-1(1) to the extent the money is used to gain or produce assessable income, and not to the extent it is capital, private or domestic (s 8-1(2)). Refuses a start outside the income year, an end before the start or 30 years or more after it, more than 5,000 transactions or 1,000 rate changes, an entry not in date order or outside the statement, 2 rate changes on one day or one on start_date, a rate outside 0 to 1, and a balance below nil at the end of a day (a loan in credit). Excludes deciding the extent of income use, whether any amount is deductible and the purpose of the borrowing; interest on another day count, compounding or rest, or charged in advance; fees and charges; offset accounts and redraw limits; interest on a balance in credit; Division 7A loans (div7a-loan-review); substituted accounting periods; and the lender.",
    ),
    (
        "wine_equalisation_tax",
        "Wine tax on one tax period's taxable dealings in wine in a financial year from 2023-24 to 2027-28, by an entity registered or required to be registered for GST, and the producer rebate on them (A New Tax System (Wine Equalisation Tax) Act 1999). The operator establishes each dealing: what it is, that it is in assessable wine in the indirect tax zone, that no exemption other than quoting applies, and its price. Its taxable value is: for a wholesale sale, the price excluding wine tax and GST (s 5-5, items AD1a, AD1b and AD11b); for a retail sale that is an assessable dealing, and for an application to own use connected with retail sales (consuming or giving away wine in connection with retail sales, such as tastings), the notional wholesale selling price by the half retail price method, 50% of the retail price including wine tax and GST (ss 9-30 and 9-35; WETR 2009/1 para 143), or for grape wine, when chosen, by the average wholesale price method, the weighted average price excluding wine tax and GST of the tax period's wholesale sales of the same vintage and grape varieties (ss 9-25 and 9-40), allowed only where at least 10% by value of those sales are wholesale sales; and for an application to own use of wine purchased under quote, the purchase price excluding GST (items AD3c(a) and AD13c(a)). Each notional wholesale selling price is rounded half up to the cent. The wine tax is 29% of the taxable value, rounded half up to the cent (s 5-5(3)); a sale under quote is not taxable (s 7-10). For a wholesale sale, a sale under quote and a retail sale the GST is shown too, on the WET inclusive price or within the retail price, assuming a taxable supply. A dealing given with rebate_conditions earns the producer rebate, 29% of the price excluding wine tax and GST of a wholesale sale or sale under quote, or of the notional wholesale selling price of a retail sale or application to own use (s 19-15(1)), when the operator states that the entity is the wine's producer, owns the source product for at least 85% of the wine throughout its making and has it in a container of at most 5 litres (51 litres for cider or perry) suitable for retail sale and branded with its trade mark (s 19-5(1), (3) to (5) and (7)), and the dealing is one it is liable to wine tax on or, for a sale under quote, the quote states none of the intentions in s 5-50(1)(b). The rebates are capped for the financial year at $350,000 to 2025-26 and $400,000 from 2026-27 (s 19-15(2), as amended by Act No. 12 of 2026 for assessable dealings from 1 July 2026), for the producer or its group of associated producers (ss 19-15(3) and 19-20), less the rebates claimed in the year's earlier tax periods (WETR 2009/2 para 67). The amounts for BAS label 1C (wine tax payable) and 1D (wine tax refundable, here only the producer rebate) are the totals rounded down to whole dollars. Refuses a year outside 2023-24 to 2027-28, the average wholesale price method for wine that is not grape wine or without the 10% share, and a rebate on an application to own use of wine purchased under quote (its rebate would be on the notional wholesale selling price). Excludes deciding what a dealing is, whether the wine is assessable wine or grape wine, the price (arm's length, s 27-10; packages with other goods, s 27-15; containers, s 9-65), quoting entitlements, the rebate conditions and associated producers; importations and other customs dealings (paid to the Department of Home Affairs), royalty-inclusive dealings, indirect marketing schemes' notional prices, airport shop goods and other dealings valued under s 9-45; wine tax credits other than the producer rebate (s 17-5), such as bad debts and earlier wine tax; New Zealand participants; the transitional rules for 2017 and earlier vintage wine and for wine being made into fortified wine on 1 January 2018; excess claims under s 19-25; correcting earlier errors; and attributing a dealing to a tax period.",
    ),
    (
        "trade_in",
        "Trading in one depreciating asset for a replacement in an income year from 2025-26 to 2026-27, for a taxpayer with an income year from 1 July to 30 June, at arm's length, where the replacement is started to be held and first used in that income year. The traded-in asset's facts are balancing_adjustment's, and the operator establishes them as it states. Its termination value is the trade-in allowance, the amount by which the replacement's price is reduced (ITAA 1997 s 40-305(1)(b) item 2), less the GST payable on the trade-in where it is a taxable supply (1/11 of the allowance: GST Act ss 9-70 and 9-75(1); ITAA 1997 s 27-95(1)), plus the GST Act s 132-5 decreasing adjustment for an asset acquired partly for a private purpose or for making financial supplies, from its input tax credit as adjusted and the full one (s 27-95(3)); its balancing adjustment, reduction for non-taxable use and CGT event K7 amount are then worked as balancing_adjustment works them. The replacement's cost is its price before the trade-in, including GST and any luxury car tax (ss 40-180 and 40-185(1)(b) item 2), less its input tax credit (s 27-80(1)): for a creditable acquisition the GST payable, 1/11 of the price less any luxury car tax, times the extent of creditable purpose (GST Act ss 9-75, 11-25 and 11-30), and for a passenger car no more than 1/11 of the car limit times that extent (s 69-10(1) and (3)). A passenger car's cost is then reduced to the car limit for the year (ITAA 1997 s 40-230(1)): $69,674 for 2025-26 and $69,883 for 2026-27, as the ATO publishes them. Amounts are worked exactly and shown to the cent, half up. Refuses a trade-in not at arm's length (s 40-300(2) item 6, s 27-95(2) and s 40-225 would apply), an involuntary disposal (whose s 40-365 choice is not made), a taxable trade-in with an adjusted input tax credit above the full one, a creditable acquisition with a nil creditable purpose, and luxury car tax above the price. Excludes deciding whether either supply is taxable, the allowance, price, creditable purpose and input tax credits; a car whose purchaser may quote an ABN for luxury car tax, a car that is not a luxury car under s 25-1(2) of the Luxury Car Tax Act, a car leased or hired, and a car excluded from the car limit by s 40-230(2); a trade-in of a luxury car on which luxury car tax is payable; a replacement first held or used in another year; a decreasing adjustment arising in a later income year (s 27-95(5)) and increasing adjustments (s 27-95(4)); assets in a pool or written off under s 328-180 or temporary full expensing; and everything balancing_adjustment excludes.",
    ),
    (
        "payroll_tax",
        "One employer's payroll tax for the 2026-27 financial year (1 July 2026 to 30 June 2027) in one of New South Wales, Victoria, Queensland, Western Australia, South Australia, Tasmania, the Australian Capital Territory and the Northern Territory, for an employer that is not a member of a group and paid or was liable to pay wages on every day of the year. The operator establishes the employer's taxable wages in the jurisdiction (the wages its Act makes taxable there, after its exemptions) and its Australian wages (those taxable wages with its interstate wages, the wages taxable in every other jurisdiction), and in Victoria whether it is a regional employer (Payroll Tax Act 2007 (Vic) s 3A). The result is the annual liability under each Act's schedule before any rebate, discount or cap, such as Queensland's regional discount (Payroll Tax Act 1971 (Qld) s 10A) and the 6.85% cap for an eligible university in the ACT (DI2026-151 s 5(c)(i)), and before the amounts paid in periodic returns.",
    ),
    (
        "balancing_adjustment",
        "The balancing adjustment for one depreciating asset when a balancing adjustment event occurs for it in an income year from 2024-25 to 2026-27, under ITAA 1997 Subdivision 40-D as compiled on 1 October 2026, for a taxpayer with an income year from 1 July to 30 June whose decline in value was worked out under Subdivision 40-B. The operator establishes that a balancing adjustment event occurred (s 40-295); the termination value under ss 40-300 to 40-320, after Subdivision 27-B; the cost (both elements, after Subdivision 27-B and, for a car, the car limit of s 40-230); whether the asset is a car (s 995-1), a motor cycle or similar vehicle, or another asset; and, for each income year it was held from its start time to the event's year, the decline in value (s 40-25(1)) and the deduction after the reductions s 40-25(2) to (4) required. A car states each year's car expense method (log book or cents per kilometre, from 2015-16) and days held; for a cents per kilometre year the decline is the notional decline s 40-370(3) assumes, worked on the same basis as the other years; and, where s 40-230 reduced its cost, the car limit for the financial year it was first used, the second element of cost and its total cost ignoring the car limit after Subdivision 27-B (s 40-325). The adjustable value just before the event is cost less the years' declines (s 40-85). The balancing adjustment amount is the termination value less the adjustable value: assessable if positive, deductible if negative (s 40-285), reduced by itself times the sum of the s 40-25 reductions over the total decline (s 40-290(2)). A car whose car expenses were claimed by the log book method in some years and the cents per kilometre method in others is worked under s 40-370 instead: the cents per kilometre years count 80% of their notional decline as reductions (s 40-370(4)), and the reduced amount is scaled by the days the decline was deducted over the days held. A car claimed only by the cents per kilometre method has nothing assessable or deductible (s 40-370(1) Note 2). A car's termination value is first multiplied by the s 40-325 fraction where s 40-230 reduced its cost. The capital gain or loss of CGT event K7 is (termination value less cost, or cost less termination value) times the same fraction (s 104-240), disregarded for a car, motor cycle or similar vehicle (s 118-5); it is before any other exemption, the CGT discount and the changes from 1 July 2027. Amounts are worked exactly with each fraction divided once, last; the result's amounts are shown to the cent, half up. Refuses a year after the event's year, a year before 2001-02, a car year before 2015-16 (when the one-third of actual expenses and 12% of original value methods could apply, which s 40-370 as compiled now does not count), a year given twice, years that skip one or stop before the event's, a deduction above its year's decline, declines totalling more than cost or nil, car limit facts that show s 40-230 did not reduce the cost, and a car's cost other than the car limit plus the second element (as for a car first used in a later financial year than the one it was acquired in, or held with another entity). Excludes deciding whether an event occurred and the termination value, cost, decline, use and method; assets in a low-value pool, a software development pool, a project pool or a small business pool, and assets written off under s 328-180 or temporary full expensing (Subdivision 328-D and the rules for those pools apply instead, and K7 is disregarded for a Division 328 asset, s 104-235(4)(b)); the fixed 50% reduction chosen under s 40-291A from 2026-27; second-hand assets in residential rental property (s 40-291); R&D use (ss 40-292 and 40-293); split or merged assets (s 40-290(3) and (4)); roll-over relief (s 40-340) and an asset held as a legal personal representative, whose earlier holder's reductions count; involuntary disposal roll-over (s 40-365); the deferral for vessels (s 40-285(5)); a car acquired at a discount (s 40-225, which s 40-320 adds back to the termination value); misappropriation (s 25-47); pre-CGT assets and assets acquired before 21 September 1999 or 1 July 2001 whose cost base exceeds cost; other CGT exemptions, such as collectables and personal use assets (ss 108-20 and 118-10); substituted accounting periods; and the tax payable on either amount.",
    ),
    (
        "business_capital_expenditure",
        "Business capital expenditure under ITAA 1997 s 40-880 (black hole and start-up costs), for an income year from 2020-21 to 2027-28, item by item, each incurred from 1 July 2020 (when s 40-880 as amended by Act No 92 of 2020 began to apply) to the end of the income year asked about, by a taxpayer with an income year from 1 July to 30 June. The operator establishes each item's amount, the income year it was incurred and that it is capital expenditure; the business it is in relation to, under s 40-880(2)(a) (your business), (b) (one that used to be carried on), (c) (one proposed to be carried on) or (d) (liquidating, deregistering or winding up a company, partnership or trust that carried on a business), and for a former or proposed business whether you or another entity carries it on; the extent the business is, was or is proposed to be carried on for a taxable purpose (s 40-880(3) and (4)(a)), tested when the expenditure is incurred for a current or proposed business and over a period that reasonably reflects a former one (TR 2011/6 paras 157 and 160); for another entity's business, and for a wind-up, whether the expenditure is in connection with your deriving assessable income from the business and with the business (s 40-880(4)(b)); for a proposed business, whether it is reasonable to conclude that it is proposed to be carried on within a reasonable time (s 40-880(7)), whether the expenditure is advice or services relating to its proposed structure or operation or a government agency's fee, tax or charge relating to establishing it or its operating structure (s 40-880(2A)(b)), and which entity test of s 40-880(2A)(c) and (2B) the taxpayer meets for the income year incurred; and which of s 40-880(5)(a) to (j), (8), (9)(a) and (9)(b) exclude it, with s 40-880(6) for (5)(d) and (f). An item excluded by s 40-880(4)(b), (5), (7), (8) or (9) has nothing to deduct. Otherwise its deductible expenditure is its amount times the taxable purpose extent, deducted in full in the year incurred under s 40-880(2A) when that subsection's conditions are met, or else one fifth in each of the 5 income years starting in the year incurred (s 40-880(2)), with no apportionment for part of a year. Eligibility is fixed when an item is incurred, so its later fifths continue if the business ceases (TR 2011/6 para 10). Amounts are worked exactly; the result's amounts are shown to the cent, half up. Refuses expenditure that forms part of the cost of a depreciating asset (s 40-880(5)(a): Division 40) or that could be taken into account in working out a capital gain or loss (s 40-880(5)(f): the CGT provisions), a revenue outlay, an item incurred before 1 July 2020 or after the year asked about, and a fact a stated item needs that is missing. Excludes deciding whether an outlay is capital or business related, the taxable purpose extent, the entity tests and each exclusion; splitting an outlay that serves more than one business or purpose (state each part as an item); expenditure partly within an exclusion (state the excluded part apart); non-arm's length amounts (s 40-885); Division 250; non-commercial losses (Division 35), which can defer or deny an individual's deduction; from 2027-28, s 26-155's limit on deductions relating to using or holding residential dwellings as residential accommodation; personal services income (s 85-10); a taxpayer that ceases to exist before the 5 years end (TR 2011/6 example 41); substituted accounting periods; consolidated groups; and website and software costs.",
    ),
    (
        "borrowing_expenses",
        "One loan's borrowing expenses under ITAA 1997 s 25-25 for one income year from 2017-18 to 2027-28. The operator establishes the expenditure the taxpayer incurred for borrowing the money, all in the income year the loan began; the extent the borrowed money is used to produce assessable income, the same throughout the loan; the first day the money was borrowed; the last day of the period the original loan contract specifies; any repayment day; and the income-producing share of the taxpayer's other borrowing expenses incurred in that year. Under s 25-25(6), when the money is used to produce income and this loan's income-producing share plus those other amounts is $100 or less, that share is deducted in the year incurred and later years are refused. Otherwise the period of the loan is the shortest of the contract's period, the period to the repayment day and 5 years from the first day borrowed (s 25-25(5)). Each income year's maximum deduction is the remaining expenditure over the days left in that period, times those days in the year, as determined at the end of the year, so the year of repayment takes all that remains (s 25-25(4)); each maximum is rounded to cents and carried forward at that amount, as the ATO's examples are, and the deduction is the maximum times the income-producing share (s 25-25(2) and (3)). Every day is counted, both ends included, as the ATO and the Library count: 5 years from 3 July end on 2 July, and from 29 February on 28 February (Acts Interpretation Act 1901 ss 2G(2) and 36(1)). From 2027-28 the result is flagged, not changed: s 26-155 may quarantine it. Excludes deciding which costs are borrowing expenses and the money's use; interest; refinancing, redraws and changes in use (borrowing_expenses_schedule takes refinancing and use by year); expenditure incurred before the income year the loan began; leisure facilities such as holiday homes unless s 26-50(3) applies at all times in the year; commercial debt forgiveness (Subdivision 245-E); private company loans (Division 7A); and applying s 26-155.",
    ),
    (
        "borrowing_expenses_schedule",
        "Several loans' borrowing expenses under ITAA 1997 s 25-25, each loan first drawn from 1 July 2017 to the end of the income year asked about, from 2017-18 to 2027-28: that year's amounts across the loans, and every income year of each loan with the totals by year. The operator establishes, for each loan, the expenditure incurred for borrowing the money, all in the income year the loan began; the first day the money was borrowed; the last day of the period the original loan contract specifies; any repayment day; the extent the money is used to produce assessable income, for every year or for each income year of the loan; and, for a loan that refinances an earlier one, which loan it repaid, on the day it was first drawn; and the income-producing share of borrowing expenses on loans not listed, by the income year incurred. Under s 25-25(6), the income-producing shares of everything incurred for borrowing in an income year, at $100 or less in total, are deducted that year; a loan whose money is not used to produce income at all that year is spread. Otherwise each loan is spread over its own period of the loan as borrowing_expenses spreads one loan (s 25-25(4) and (5)), each year's deduction being its maximum times that year's income-producing use (s 25-25(2) and (3)). A refinanced loan is repaid on the refinancing day, so its period ends then and that income year deducts what remains (s 25-25(5)(b)). A row from 2027-28 is flagged, not changed, because s 26-155 may quarantine it, and a row after 2027-28 applies s 25-25 as read on 19 November 2026 and is marked projected. Excludes deciding which costs are borrowing expenses, the money's use and the tracing of redraws; interest; expenditure incurred after the income year a loan began; loans first drawn before 1 July 2017; leisure facilities such as holiday homes unless s 26-50(3) applies at all times in each year (such a loan is refused); commercial debt forgiveness (Subdivision 245-E); private company loans (Division 7A); and applying s 26-155.",
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
        "fbt_meal_entertainment_register",
        "Whether an employer's 12 week meal entertainment register is valid for the FBT year ended 31 March 2026 or 2027 under FBTAA Division 9A Subdivision C (ss 37CA to 37CF), and the values fbt_meal_entertainment takes from it for the register percentage (s 37CB(2)). It checks that the register period is continuous and at least 12 weeks (84 days, both ends counted) and that the register records meal entertainment (s 37CC(1) and (3)); that every entry records the date, whether each recipient is an employee or an associate of one, the cost, the kind of meal entertainment, where it was provided and, on the employer's premises, whether in an in-house dining facility (s 37CE(1)); and the FBT years it is valid for: the year its period ends in, when the period begins and ends in that year, or the second year, when it spans 2, and the 4 years after (s 37CD(1) and (2)), unless a later year's meal entertainment expenses were more than 20% higher than the first valid year's, when it ceases to be valid at the end of that year (s 37CD(3)). Each check is reported with its authority and the entries it fails on; the register is valid only when every check passes, and then the result gives fbt_meal_entertainment's register inputs. The register's total value of meal entertainment is the recorded costs and its total value of meal entertainment fringe benefits the fringe_benefits_value of each entry, which the operator works out: the part provided to employees and their associates, less any exempt benefit, such as a minor benefit, which the guide says can apply under this method (the guide points to TD 94/25 on apportioning entertainment between employees and others). The register percentage is kept exact and shown to the cent. Refuses an entry dated outside the period, a fringe benefits value above the cost or above nil with no employee or associate among the recipients, a period spanning more than 2 FBT years, and an in-house dining facility answer for meal entertainment off the employer's premises. The operator establishes that the employer elects Division 9A and Subdivision C (s 37CA), that the register was kept, and meal entertainment provided, throughout the period (s 37CC(1)), that the period is representative of the first FBT year the register is valid for (s 37CC(2)), that no entry is false or misleading in a material particular (s 37CF), that each entry was made as soon as practicable (s 37CE(2)), and that no later valid register replaces this one for the year (s 37CD(3)). Excludes the car parking register (ss 39GA to 39GF), the taxable value itself (fbt_meal_entertainment works it out) and judging whether the period is representative.",
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
        "The tax-free part of a genuine redundancy or early retirement scheme payment and the PAYG withholding from one life benefit employment termination payment (ETP) under Schedule 11, made in 2026-27 to a resident employee who gave a TFN and is not a working holiday maker, within 12 months of the termination. The operator establishes the redundancy payment as a genuine redundancy payment (ITAA 1997 s 83-175, including dismissal before pension age) or an early retirement scheme payment (s 83-180), above what voluntary termination would have paid; the completed years of service (s 83-170(3)); the rest of the ETP's taxable component, split into excluded payments (s 82-10(6)) and non-excluded payments; and the payer's other taxable payments to the employee in the year. The redundancy payment is tax-free up to $13,598 plus $6,801 for each completed year of service (s 83-170) and the rest of it is an excluded ETP part. The excluded part is taken first and capped at the $270,000 ETP cap; the non-excluded part is capped at the smaller of the ETP cap less the excluded part within it and the whole-of-income cap, $180,000 less the other taxable payments, not below nil (s 82-10(4) and (7); Schedule 11). Withholding is 17% up to the cap if the employee is their preservation age or older on 30 June 2027 (SISR reg 6.01(2)), 32% if younger, and 47% above the cap, each part rounded to the nearest dollar, 50 cents up. Excludes classifying the payments, death benefit ETPs, working out an ETP's tax-free component (an invalidity segment or employment before 1 July 1983; etp_tax_free_component does, and gives the taxable component to classify here), earlier ETPs to the employee in the income year or for the same termination, unused annual and long service leave payments, delayed termination payments, payees without a TFN, foreign residents, working holiday makers and the income tax assessed on the ETP.",
    ),
    (
        "etp_tax_free_component",
        "The tax free component of one life benefit employment termination payment (ETP) and the taxable component left, for a payment made in 2026-27 or 2027-28 under ITAA 1997 ss 82-140 to 82-155 as compiled on 19 November 2026, which set no figure by year. The invalidity segment (s 82-150) is the payment times the days to retirement over the employment days plus the days to retirement, where the payment was made because the employee stopped being gainfully employed through ill-health, before their last retirement day, and 2 legally qualified medical practitioners have certified that they are unlikely ever to be gainfully employed in a capacity for which they are reasonably qualified; it is nil otherwise. The days to retirement run from the day after the termination to the last retirement day, both counted; the employment days from the first day of the employment to the termination day, both counted, as the ATO's worksheet NAT 71949 counts them. The pre-July 83 segment (s 82-155) is the payment less the invalidity segment, times the employment days before 1 July 1983 over all the employment days. Each segment is rounded half up to the cent, and the taxable component is the payment less both (s 82-145), so the components add up to the payment. Whether the payment includes an invalidity segment, or would but for its timing, is shown as invalidity_excluded_payment: such a payment is an excluded payment (s 82-10(6)(c)), so its taxable component is termination_payment's excluded_payment. Refuses an employment start after the termination day. The operator establishes that the payment is a life benefit ETP (s 82-130) and its amount, the ill-health, the medical certificates and the last retirement day (s 995-1(1)), and that the payment relates to one continuous period of employment from employment_start to termination_date. Excludes death benefit ETPs, which cannot include an invalidity segment; payments that relate to only part of that period or to broken periods of employment, whose employment days are not the days between the 2 dates; unused annual and long service leave (Division 83); superannuation benefits, whose disability benefits increase the tax free component under s 307-145 instead; and the tax or withholding on the taxable component, which termination_payment works out.",
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
        "lump_sum_arrears_tax_offset",
        "The tax offset for lump sum payments in arrears (ITAA 1936 Part III Division 17 Subdivision AB, ss 159ZR to 159ZRD) of an individual, not a trustee (s 159ZRA(2)), whose assessable income of the year of receipt includes eligible lump sums the operator has classified (s 159ZR(1)): lump sum payments, received on or after 1 July 1986, of salary or wages (Taxation Administration Act 1953 Schedule 1 ss 12-35, 12-40 except to a director who is an associate of the company, 12-45, 12-80, 12-110, 12-115 and 12-120) to the extent they accrued in a period ending more than 12 months before payment, of salary or wages for a period of suspension paid after reinstatement, of a payment covered by s 12-80 or 12-120, of a Commonwealth education or training payment, or of a payment covered by ITAA 1997 Division 52, 53 or 55 that is not exempt, or a similar payment under a foreign law, less any part taken into account in an act of grace tax reimbursement; a BSWAT payment amount counts as one (s 159ZR(2)). The operator gives each earlier income year in which part of them accrued, once, with its annual arrears amount from every payer. The individual was an Australian resident for the whole of the year of receipt and of each recent accrual year, entitled to the whole tax-free threshold and taxed at the resident rates on all of that taxable income: none of it a minor's eligible taxable income (Division 6AA) or a superannuation or employment termination remainder taxed at another rate. None of those years' taxable income includes an amount under ITAA 1997 Division 82 (employment termination payments), s 83-10 or 83-80 (unused annual or long service leave) or Division 301 or 302 (superannuation benefits) or Income Tax (Transitional Provisions) Act 1997 Division 82, or above-average special professional income, and none had the primary producers' averaging tax offset (s 392-35(2)), so that rebated tax and gross tax (s 159ZR(1)) are the tax at the year's resident rates and normal taxable income is taxable income less any net capital gain. The offset is due when the total arrears amount is at least 10% of normal taxable income less that amount (s 159ZRA(1)(b)). It is the tax on the arrears, the tax on the year's taxable income less the tax on it without the arrears (s 159ZRB), less the notional tax: for each recent accrual year (the latest 2 when there are 3 or more accrual years, otherwise each one), the tax on its taxable income with its arrears added less the tax without them (s 159ZRC), and for the other, distant accrual years their arrears at the average of the recent accrual years' rates, each rate the increase in tax on normal taxable income over the arrears, to 3 decimal places, increased by 0.001 when the fourth decimal place would be more than 4 (s 159ZRD); nil when the notional tax is the greater. A recent accrual year that was itself a rebate year, or in which part of an earlier rebate year's total arrears amount accrued, is adjusted as ss 159ZRC and 159ZRD direct, from the amounts given. Each year's resident rates are the ATO's published rates from 2012-13, with the temporary budget repair levy of 2% above $180,000, which Income Tax (Transitional Provisions) Act 1997 s 4-11 makes extra income tax, for 2014-15 to 2016-17; a recent accrual year before 2012-13 is refused. Excludes foreign and part-year residents, employment termination payments and superannuation lump sums, which are not eligible income, an amount the operator has not classified as eligible income, the Medicare levy exemption for lump sums in arrears (Medicare Levy Act 1986 s 9A), the Medicare levy surcharge tax offset for them (ITAA 1997 Subdivision 61-L), and applying the offset against tax payable.",
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
        "fbt_minor_benefit_check",
        "A screen of one benefit provided to an employee or an associate in respect of the employee's employment, in the FBT year ended 31 March 2026 or 2027, against the minor benefits exemption of FBTAA s 58P. The operator establishes the benefit's category, its notional taxable value in the current year (for one benefit to one person: TR 2007/12 paras 17 and 18), whether it is under a salary sacrifice arrangement, each criterion of s 58P(1)(f) as a stated fact, and the facts of each category's exclusion. The conditions are decided and shown as met or not: an expense payment, property or residual benefit that would not be an in-house fringe benefit (s 58P(1)(c)); tax-exempt body entertainment only within s 58P(1)(d)(i) or (ii); meal entertainment or entertainment facility leasing never qualifies under the 50/50 split method (Division 9A or s 152B: TR 2007/12 para 14), and is refused without it, to be screened under its own category; no salary sacrifice arrangement (TR 2007/12 para 16); and a notional taxable value less than $300 (s 58P(1)(e)). The criteria of s 58P(1)(f) are each shown with which way the stated fact weighs, and weighed as the Commissioner's examples in TR 2007/12 weigh them: exempt where identical or similar benefits are not provided frequently, neither total is substantial and the benefit is not both regular and a reward for services (examples 1 to 3, 5 to 7 and 9 to 11), not exempt where they are provided frequently as a reward for services, their total is substantial, they are readily valued and not for an unexpected event (example 8), and otherwise the employer's judgement, shown as such. TR 2007/12 is a public ruling, the Commissioner's view of the law, and its para 11 still lists airline transport benefits, whose exclusion (s 58P(1)(b)) Act No. 88 of 2013 repealed. Excludes deciding the benefit's category, its notional taxable value, whether benefits are associated (s 58P(2)), and the criteria themselves; benefits exempt under another provision; and what an exempt entertainment benefit means for income tax and GST.",
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
        "borrowing_expenses_schedule",
        "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/25-25",
    ),
    (
        "hire_purchase_schedule",
        "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/240-60",
    ),
    (
        "luxury_car_tax",
        "https://www.ato.gov.au/law/view/document?docid=PAC/19990076/5-15",
    ),
    (
        "chattel_mortgage_schedule",
        "https://www.ato.gov.au/law/view/print?DocID=GIR%2FFinancial-services-ch1&PiT=99991231235958",
    ),
    (
        "loan_transactions",
        "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_1/document_1.html",
    ),
    (
        "wine_equalisation_tax",
        "https://www.ato.gov.au/law/view/document?docid=PAC/19990062/5-5",
    ),
    (
        "balancing_adjustment",
        "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_2/document_2.html",
    ),
    (
        "payroll_tax",
        "https://www.revenue.nsw.gov.au/taxes-duties-levies-royalties/payroll-tax/lodge-and-pay-returns/thresholds-and-rates",
    ),
    (
        "gst_margin_scheme",
        "https://www.ato.gov.au/law/view/document?DocID=PAC/19990055/75-5",
    ),
    (
        "trade_in",
        "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_2/document_2.html",
    ),
    (
        "business_capital_expenditure",
        "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_2/document_2.html",
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
        "fbt_meal_entertainment_register",
        "https://www.ato.gov.au/law/view/document?docid=PAC/19860039/37CC",
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
        "etp_tax_free_component",
        "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/82-150",
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
        "lump_sum_arrears_tax_offset",
        "https://www.ato.gov.au/individuals-and-families/income-deductions-offsets-and-records/income-you-must-declare/lump-sum-payment-in-arrears",
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
        "fbt_minor_benefit_check",
        "https://www.ato.gov.au/law/view/document?DocID=PAC/19860039/58P",
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
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/26-155",
            "https://www.legislation.gov.au/C1901A00002/2026-03-28/text",
        ],
    ),
    (
        "borrowing_expenses_schedule",
        &[
            "https://www.ato.gov.au/individuals-and-families/investments-and-assets/property-and-land/residential-rental-properties/rental-expenses/borrowing-expenses",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/26-155",
            "https://www.legislation.gov.au/C1901A00002/2026-03-28/text",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/26-50",
        ],
    ),
    (
        "hire_purchase_schedule",
        &[
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/240-17",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/240-55",
            "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_5/document_5.html",
            "https://www.legislation.gov.au/C2004A00446/2026-01-01/2026-01-01/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.ato.gov.au/businesses-and-organisations/gst-excise-and-indirect-taxes/gst/in-detail/rules-for-specific-transactions/agent-consignment-and-progressive-transactions/gst-hire-purchase-and-leasing",
            "https://www.ato.gov.au/businesses-and-organisations/gst-excise-and-indirect-taxes/gst/in-detail/your-industry/motor-vehicle-and-transport/gst-and-motor-vehicles/purchasing-a-motor-vehicle",
            "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/depreciation-and-capital-expenses-and-allowances/simpler-depreciation-for-small-business/assets-and-exclusions",
            "https://www.ato.gov.au/law/view/print?DocID=GIR%2FFinancial-services-ch1&PiT=99991231235958",
        ],
    ),
    (
        "chattel_mortgage_schedule",
        &[
            "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_5/document_5.html",
            "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_10/document_10.html",
            "https://www.legislation.gov.au/C2004A00446/2026-01-01/2026-01-01/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.ato.gov.au/law/view/print?DocID=TXR/TR9316/NAT/ATO/00001&PiT=99991231235958",
            "https://www.ato.gov.au/businesses-and-organisations/gst-excise-and-indirect-taxes/gst/in-detail/your-industry/motor-vehicle-and-transport/gst-and-motor-vehicles/purchasing-a-motor-vehicle",
            "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/depreciation-and-capital-expenses-and-allowances/simpler-depreciation-for-small-business/assets-and-exclusions",
        ],
    ),
    (
        "balancing_adjustment",
        &[
            "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_3/document_3.html",
            "https://www.ato.gov.au/law/view/document?docid=SAV/DEPRECIATING/00001",
            "https://www.legislation.gov.au/C2026A00049/asmade/text",
        ],
    ),
    (
        "payroll_tax",
        &[
            "https://legislation.nsw.gov.au/view/whole/html/inforce/current/act-2007-021",
            "https://www.sro.vic.gov.au/about-us/rates-and-statistics/current-rates/payroll-tax-current-rates",
            "https://www.sro.vic.gov.au/businesses-and-organisations/payroll-tax/thresholds-and-grouping/payroll-tax-surcharges",
            "https://www.legislation.vic.gov.au/in-force/acts/payroll-tax-act-2007",
            "https://qro.qld.gov.au/payroll-tax/calculate/rates-thresholds/",
            "https://qro.qld.gov.au/payroll-tax/calculate/deductions/",
            "https://qro.qld.gov.au/payroll-tax/mental-health-levy/calculating/",
            "https://www.legislation.qld.gov.au/view/whole/pdf/inforce/2026-10-22/act-1971-037",
            "https://www.wa.gov.au/government/multi-step-guides/payroll-tax-employer-guide/calculation-payroll-tax-employer-guide",
            "https://www.legislation.wa.gov.au/legislation/statutes.nsf/law_a6335.html",
            "https://www.legislation.wa.gov.au/legislation/statutes.nsf/law_a6336.html",
            "https://www.revenuesa.sa.gov.au/payrolltax/rates-and-thresholds",
            "https://www.legislation.sa.gov.au/_legislation-documents/lz/c/a/payroll-tax-act-2009/current/2009.22.auth.pdf",
            "https://www.sro.tas.gov.au/payroll-tax/rates-thresholds",
            "https://www.legislation.tas.gov.au/view/whole/html/inforce/current/act-2008-016",
            "https://www.revenue.act.gov.au/business-taxes-and-levies/payroll-tax/about-payroll-tax",
            "https://www.revenue.act.gov.au/business-taxes-and-levies/payroll-tax/calculating-payroll-tax",
            "https://www.legislation.act.gov.au/View/di/2026-151/current/html/2026-151.html",
            "https://www.legislation.act.gov.au/a/2011-18/",
            "https://treasury.nt.gov.au/dtf/territory-revenue-office/payroll-tax/payroll-tax-rates-and-thresholds",
            "https://treasury.nt.gov.au/dtf/territory-revenue-office/payroll-tax/payroll-changes-from-1-july-2026",
            "https://legislation.nt.gov.au/en/Legislation/PAYROLL-TAX-ACT-2009",
        ],
    ),
    (
        "gst_margin_scheme",
        &[
            "https://www.ato.gov.au/law/view/document?DocID=PAC/19990055/75-10",
            "https://www.ato.gov.au/law/view/document?DocID=PAC/19990055/75-11",
            "https://www.ato.gov.au/law/view/document?DocID=PAC/19990055/75-12",
            "https://www.ato.gov.au/law/view/document?DocID=PAC/19990055/75-13",
            "https://www.ato.gov.au/law/view/document?DocID=PAC/19990055/75-14",
            "https://www.ato.gov.au/law/view/document?DocID=PAC/19990055/75-15",
            "https://www.ato.gov.au/law/view/document?DocID=PAC/19990055/75-20",
            "https://www.ato.gov.au/law/view/document?DocID=PAC/19990055/75-35",
            "https://www.ato.gov.au/law/view/document?DocID=PAC/19990055/9-90",
            "https://www.ato.gov.au/law/view/document?DocID=GST/GSTR20068/NAT/ATO/00001",
            "https://www.ato.gov.au/businesses-and-organisations/gst-excise-and-indirect-taxes/gst/in-detail/your-industry/property/gst-and-the-margin-scheme",
        ],
    ),
    (
        "wine_equalisation_tax",
        &[
            "https://www.ato.gov.au/law/view/document?docid=PAC/19990062/9-25",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19990062/9-35",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19990062/9-40",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19990062/19-5",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19990062/19-15",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19990062/19-20",
            "https://www.legislation.gov.au/C2004A00452/2026-04-01/2026-04-01/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.legislation.gov.au/C2026A00012/asmade/2026-03-26/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.ato.gov.au/law/view/pdf/pbr/wetr2009-001.pdf",
            "https://www.ato.gov.au/law/view/pdf/pbr/wetr2009-002.pdf",
            "https://www.ato.gov.au/businesses-and-organisations/gst-excise-and-indirect-taxes/wine-equalisation-tax/how-much-to-pay/wholesale-sales",
            "https://www.ato.gov.au/businesses-and-organisations/gst-excise-and-indirect-taxes/wine-equalisation-tax/how-much-to-pay/retail-sales-and-own-use",
            "https://www.ato.gov.au/businesses-and-organisations/gst-excise-and-indirect-taxes/wine-equalisation-tax/producer-rebate",
            "https://www.ato.gov.au/businesses-and-organisations/gst-excise-and-indirect-taxes/wine-equalisation-tax/producer-rebate/calculate-and-claim-your-rebate",
            "https://www.ato.gov.au/businesses-and-organisations/gst-excise-and-indirect-taxes/wine-equalisation-tax/reporting-and-paying-wet",
            "https://www.ato.gov.au/businesses-and-organisations/preparing-lodging-and-paying/business-activity-statements-bas/wine-equalisation-tax-wet",
            "https://www.ato.gov.au/businesses-and-organisations/gst-excise-and-indirect-taxes/gst/completing-your-bas-for-gst/complete-your-bas",
        ],
    ),
    (
        "luxury_car_tax",
        &[
            "https://www.ato.gov.au/law/view/document?docid=PAC/19990076/5-20",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19990076/7-15",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19990076/15-20",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19990076/15-30",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19990076/18-5",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19990076/18-10",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19990076/25-1",
            "https://www.legislation.gov.au/C2025A00029/asmade/2025-03-27/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.ato.gov.au/tax-rates-and-codes/luxury-car-tax-rate-and-thresholds",
            "https://www.ato.gov.au/businesses-and-organisations/gst-excise-and-indirect-taxes/luxury-car-tax/working-out-the-lct-amount/working-out-the-lct-on-a-sale",
            "https://www.ato.gov.au/businesses-and-organisations/gst-excise-and-indirect-taxes/luxury-car-tax/working-out-the-lct-amount/working-out-the-lct-on-an-import",
            "https://www.ato.gov.au/businesses-and-organisations/gst-excise-and-indirect-taxes/luxury-car-tax/adjustments-credits-and-refunds/credits-and-refunds",
            "https://www.ato.gov.au/businesses-and-organisations/preparing-lodging-and-paying/business-activity-statements-bas/luxury-car-tax-lct",
            "https://www.ato.gov.au/businesses-and-organisations/gst-excise-and-indirect-taxes/gst/completing-your-bas-for-gst/complete-your-bas",
        ],
    ),
    (
        "trade_in",
        &[
            "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_3/document_3.html",
            "https://www.legislation.gov.au/C2004A00446/2026-01-01/2026-01-01/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.legislation.gov.au/C2004A00446/2026-01-01/2026-01-01/text/original/epub/OEBPS/document_2/document_2.html",
            "https://www.ato.gov.au/businesses-and-organisations/gst-excise-and-indirect-taxes/gst/in-detail/your-industry/motor-vehicle-and-transport/gst-and-motor-vehicles/purchasing-a-motor-vehicle",
            "https://www.ato.gov.au/businesses-and-organisations/gst-excise-and-indirect-taxes/gst/in-detail/your-industry/motor-vehicle-and-transport/gst-and-motor-vehicles/disposing-of-a-motor-vehicle",
            "https://www.ato.gov.au/businesses-and-organisations/gst-excise-and-indirect-taxes/gst/in-detail/rules-for-specific-transactions/business-asset-transactions/gst-and-the-disposal-of-capital-assets",
            "https://www.ato.gov.au/law/view/document?docid=SAV/DEPRECIATING/00001",
            "https://www.legislation.gov.au/C2026A00049/asmade/text",
        ],
    ),
    (
        "business_capital_expenditure",
        &[
            "https://www.legislation.gov.au/C2020A00092/asmade/text",
            "https://www.ato.gov.au/law/view/document?docid=TXR/TR20116/NAT/ATO/00001",
            "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/income-and-deductions-for-business/deductions/deductions-for-depreciating-assets-and-capital-expenses",
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
        "fbt_meal_entertainment_register",
        &[
            "https://www.ato.gov.au/law/view/document?docid=PAC/19860039/37CA",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19860039/37CB",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19860039/37CD",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19860039/37CE",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19860039/37CF",
            "https://www.ato.gov.au/law/view/document?DocID=SAV/FBTGEMP/00015",
        ],
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
        "etp_tax_free_component",
        &[
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/82-140",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/82-145",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/82-155",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/82-10",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/995-1",
            "https://www.ato.gov.au/businesses-and-organisations/hiring-and-paying-your-workers/engaging-a-worker/when-a-worker-leaves-your-business/taxation-of-termination-payments/tax-free-component-of-etps",
            "https://www.ato.gov.au/forms-and-instructions/employment-termination-payments-with-an-invalidity-segment-worksheet",
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
        "lump_sum_arrears_tax_offset",
        &[
            "https://www.legislation.gov.au/C1936A00027/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_3/document_3.html",
            "https://www.ato.gov.au/tax-rates-and-codes/tax-rates-australian-residents",
            "https://www.legislation.gov.au/C2004A05128/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.legislation.gov.au/C2014A00045/asmade/text",
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
        "fbt_minor_benefit_check",
        &[
            "https://www.ato.gov.au/law/view/document?DocID=TXR/TR200712/NAT/ATO/00001",
            "https://www.ato.gov.au/law/view/document?DocID=PAC/19860039/136(1)",
            "https://www.ato.gov.au/law/view/document?DocID=PAC/19860039/152B",
        ],
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
            "interest_by_income_year totals only the periods wholly in each income year. A period spanning 30 June is listed in interest_across_30_june with its days on each side, not split: s 240-60(2) relates the part of a period that falls in an income year to it in accordance with generally accepted accounting principles, which this worksheet does not apply. The journals set the same periods apart.",
        ],
    ),
    (
        "chattel_mortgage_schedule",
        &[
            "interest_by_income_year totals only the periods wholly in each income year. A period spanning 30 June is listed in interest_across_30_june with its days on each side, not split: the operator establishes how much of its interest each income year incurs (ITAA 1997 s 8-1), from the lender's statements where they give it (TR 93/16 para 2).",
        ],
    ),
    (
        "balancing_adjustment",
        &[
            "From 2026-27 a taxpayer who deducted an amount under s 25-130 may instead choose the fixed 50% reduction of s 40-291A for an asset used to produce assessable labour income; this worksheet applies s 40-290 and does not make that choice.",
        ],
    ),
    (
        "payroll_tax",
        &[
            "Each revenue office decides which payments are taxable wages and in which jurisdiction they are taxable; this worksheet takes both wage totals as established.",
            "The annual liability is reconciled against the payroll tax and any surcharge or levy already paid in periodic returns; this worksheet does not make that reconciliation.",
        ],
    ),
    (
        "gst_margin_scheme",
        &[
            "The margin, both GST amounts and the conditions are worked whether or not the margin scheme applies, so the 2 can be compared; gst is the GST payable only on the facts given.",
            "A supply of new residential premises or potential residential land may require the purchaser to pay an amount to the ATO at settlement (Schedule 1 to the Taxation Administration Act 1953 s 14-250); this worksheet does not work it out.",
        ],
    ),
    (
        "trade_in",
        &[
            "From 2026-27 a taxpayer who deducted an amount under s 25-130 may instead choose the fixed 50% reduction of s 40-291A for an asset used to produce assessable labour income; this worksheet applies s 40-290 and does not make that choice.",
            "The decreasing adjustment reduces the net amount of GST for the tax period, not the GST payable on the trade-in; it increases the termination value only when it arises in the income year of the trade-in (ITAA 1997 s 27-95(3)), and in a later year it is assessable instead (s 27-95(5)).",
        ],
    ),
    (
        "business_capital_expenditure",
        &[
            "Each item's schedule runs 5 income years from the year incurred under s 40-880 as compiled on 1 October 2026; a later amendment could change any year not ended by then, 2026-27 and 2027-28 included.",
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
        "lump_sum_arrears_tax_offset",
        &[
            "The Medicare levy and Medicare levy surcharge are separate questions this worksheet does not answer. From 2024-25 Medicare Levy Act 1986 s 9A leaves the total arrears amount out of taxable income for the levy when it is at least 10% of normal taxable income, the arrears included, and the levy in each of the 2 latest accrual years meets a condition of s 9A(1)(c); ITAA 1997 Subdivision 61-L gives a tax offset against the surcharge on them.",
        ],
    ),
    (
        "fbt_minor_benefit_check",
        &[
            "s 58P(1)(f) asks whether, having regard to all its criteria, it would be concluded that it would be unreasonable to treat the benefit as a fringe benefit; the criteria are the operator's statements, and the screen weighs them as TR 2007/12, the Commissioner's view, weighs its examples. A judgement_required result is the employer's to decide.",
            "Where the employer uses the 12 week register method for meal entertainment, minor benefits reduce the register's meal entertainment fringe benefits value (TR 2007/12 para 15); fbt-meal-entertainment takes that value as given.",
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
/// The flag on a borrowing expenses year from 2027-28 (ITAA 1997 s 26-155,
/// inserted by Act No 49 of 2026 for the 2027-28 income year and later years,
/// read on 19 November 2026).
pub const BORROWING_S26_155: &str = "From 2027-28 ITAA 1997 s 26-155(1) quarantines the amount by which deductions relating to using or holding residential dwellings as residential accommodation exceed the assessable income from them, and this deduction is one of them where the money is used that way. The amount here is s 25-25's before s 26-155: the worksheet neither applies s 26-155 nor decides its exceptions, such as a dwelling last acquired before 7.30 pm on 12 May 2026 (ACT legal time) or a new residential dwelling (s 26-155(2)), or an excepted entity (s 26-155(4)).";

/// The refusal of a leisure facility under s 26-50.
pub const BORROWING_S26_50: &str = "Section 26-50 denies deductions for a leisure facility, such as a holiday home, unless s 26-50(3) applies at all times in the income year; part-year use under s 26-50(4) is out of scope.";

/// The note on a borrowing expenses schedule with a row after the last year read.
pub const BORROWING_PROJECTED: &str = "A row marked projected is after 2027-28, the last income year read: it applies ITAA 1997 s 25-25 as read on 19 November 2026, which sets no figure by year, and a later amendment would change it.";

/// Why a worksheet that takes an income year refuses 2027-28, added to its
/// `period.unsupported` message: the figures not yet published, or the change
/// in the law it does not apply.
pub const UNPUBLISHED_2027_28: &[(&str, &str)] = &[
    ("capital_gains", CGT_2027_28),
    ("main_residence", CGT_2027_28),
    ("balancing_adjustment", CGT_2027_28),
    (
        "trade_in",
        "The car limit for 2027-28 was not published when this worksheet was checked, and from 2027-28 Schedule 1 to Act No. 49 of 2026 changes how capital gains are worked out.",
    ),
    (
        "lump_sum_arrears_tax_offset",
        "This worksheet has not been checked against 2027-28, when Act No. 49 of 2026 brings in the working Australians tax offset and changes how capital gains are worked out.",
    ),
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
        "balancing.asset_invalid",
        "asset is not car, motor_cycle_or_similar or other.",
    ),
    (
        "balancing.car_limit_cost_mismatch",
        "cost is not car_limit's limit plus second_element_cost.",
    ),
    (
        "balancing.car_limit_invalid",
        "car_limit is not null or an object.",
    ),
    (
        "balancing.car_limit_not_reduced",
        "The car limit facts show that s 40-230 did not reduce the car's cost, so s 40-325 does not apply.",
    ),
    (
        "balancing.decline_above_cost",
        "The years' declines in value total more than cost.",
    ),
    (
        "balancing.deduction_above_decline",
        "A year's deduction exceeds its decline in value.",
    ),
    (
        "balancing.fact_invalid",
        "A row of years states a fact that is not of the form or one of the values it takes.",
    ),
    (
        "balancing.fact_missing",
        "A row of years or car_limit does not state a fact it needs.",
    ),
    (
        "balancing.no_decline",
        "The years' declines in value total nil, so the s 40-290 fraction is undefined.",
    ),
    (
        "balancing.year_after_event",
        "A row's income year is after the year of the balancing adjustment event.",
    ),
    (
        "balancing.year_before_scope",
        "A row's income year is before 2001-02, or for a car before 2015-16.",
    ),
    (
        "balancing.year_repeated",
        "A row's income year is given twice.",
    ),
    (
        "balancing.years_incomplete",
        "The rows do not give every income year from the first held to the event's.",
    ),
    (
        "balancing.years_invalid",
        "years is not a non-empty list of objects.",
    ),
    (
        "borrowing.income_use_invalid",
        "A loan's income_use is neither a fraction from 0 to 1 nor an object giving one for each income year of the loan.",
    ),
    (
        "borrowing.loan_before_2017_18",
        "A loan was first drawn before 1 July 2017.",
    ),
    (
        "borrowing.loan_date_invalid",
        "A loan date is neither null nor a date on or after the start of the loan.",
    ),
    (
        "borrowing.loan_fact_missing",
        "A loan does not state a fact the schedule requires.",
    ),
    (
        "borrowing.loan_period_ended",
        "The period of the loan ended before the income year began.",
    ),
    (
        "borrowing.loan_starts_after_year",
        "The loan starts after the income year ended.",
    ),
    (
        "borrowing.loans_invalid",
        "loans is not a non-empty list of objects.",
    ),
    (
        "borrowing.other_expenses_invalid",
        "other_borrowing_expenses is not an object of income years and amounts.",
    ),
    (
        "borrowing.refinance_invalid",
        "refinances does not name an earlier loan repaid on the day the loan was first drawn and refinanced by no other.",
    ),
    (
        "borrowing.s25_25_6_not_spread",
        "s 25-25(6) applied in the income year the expenditure was incurred, so nothing spreads to this year.",
    ),
    (
        "borrowing.s26_50_denied",
        "s 26-50 denies the deduction for a leisure facility, or the worksheet does not cover a part-year exception.",
    ),
    (
        "business_capital.cgt_amount",
        "An item could be taken into account in working out a capital gain or loss (s 40-880(5)(f)) and s 40-880(6) does not apply.",
    ),
    (
        "business_capital.depreciating_asset_cost",
        "An item forms part of the cost of a depreciating asset (s 40-880(5)(a)).",
    ),
    (
        "business_capital.fact_invalid",
        "An item's stated fact is not of the form or one of the values it takes.",
    ),
    (
        "business_capital.fact_missing",
        "An item does not state a fact its business or exclusions need.",
    ),
    (
        "business_capital.incurred_after_year",
        "An item was incurred after the income year asked about.",
    ),
    (
        "business_capital.incurred_before_july_2020",
        "An item was incurred before 1 July 2020, before s 40-880's current form applies.",
    ),
    (
        "business_capital.items_invalid",
        "items is not a non-empty list of objects.",
    ),
    (
        "business_capital.not_capital",
        "An item is stated not to be capital expenditure.",
    ),
    (
        "chattel_mortgage.balloon_too_large",
        "At a chattel mortgage's stated annual_rate the balloon alone repays the amount financed or more.",
    ),
    (
        "chattel_mortgage.final_instalment_unsupported",
        "Rounding a chattel mortgage's instalment to the cent at its stated annual_rate moves the last instalment by more than a whole instalment: below nil or above twice the instalment.",
    ),
    (
        "chattel_mortgage.stated_rate_unsupported",
        "A chattel mortgage's annual_rate is given for instalments in advance, or other than monthly or weekly.",
    ),
    (
        "chattel_mortgage.terms_conflict",
        "A chattel mortgage is given both or neither of instalment and annual_rate.",
    ),
    ("command.unknown", "The command does not exist."),
    (
        "contract_register.argument_invalid",
        "A contract-register argument is missing, unknown, repeated or not valid.",
    ),
    (
        "contract_register.column_invalid",
        "A contracts file column is unknown, repeated or missing, or a contract's cell is empty where its kind needs it, filled where its kind does not take it, or not valid.",
    ),
    (
        "contract_register.contract_duplicate",
        "Two lines of the contracts file name the same contract.",
    ),
    (
        "contract_register.csv_invalid",
        "The contracts file is not UTF-8 CSV with a header and 1 to 5,000 contracts, one a line, each with the header's fields.",
    ),
    (
        "contract_register.kind_unsupported",
        "A contract's kind is not one the register sets out.",
    ),
    (
        "contract_register.text_invalid",
        "A contract, client or financier name is empty, too long, holds a control character or begins with a character a spreadsheet reads as a formula.",
    ),
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
        "etp_tax_free_component.dates_invalid",
        "employment_start is after termination_date.",
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
        "fbt_meal_entertainment_register.entries_invalid",
        "The register's entries are not a list of at most 10,000 entry objects.",
    ),
    (
        "fbt_meal_entertainment_register.entry_invalid",
        "A register entry has a detail that cannot be read, is dated outside the register period, or values its fringe benefits above its cost or above nil with no employee or associate among its recipients.",
    ),
    (
        "fbt_meal_entertainment_register.period_invalid",
        "The register period ends before it starts or spans more than 2 FBT years.",
    ),
    (
        "fbt_minor_benefit.category_unsettled",
        "Meal entertainment or entertainment facility leasing not valued by the 50/50 split method was given; it is screened under its own category.",
    ),
    (
        "fbt_minor_benefit.fact_invalid",
        "category, frequency or tax_exempt_body_exception is not one of the values fbt-minor-benefit-check takes.",
    ),
    (
        "fbt_return.benefit_command_unsupported",
        "fbt-return was given a benefit whose command is not one of the FBT benefit worksheets.",
    ),
    (
        "fbt_return.car_statutory_lower",
        "fbt-return was given an operating cost car whose statutory formula value is lower, so FBTAA s 10(5) values it under s 9 and fbt-car-statutory gives its item 23 amounts.",
    ),
    (
        "fbt_return.exclusion_conflict",
        "fbt-return was told a car parking or meal entertainment benefit is not excluded from reporting, which FBTAA s 5E(3) excludes.",
    ),
    (
        "fbt_return.gst_type_missing",
        "fbt-return was given a benefit without a gst_type of type_1 or type_2.",
    ),
    (
        "fbt_return.mixed_years",
        "fbt-return was given a benefit for another FBT year than the return's.",
    ),
    (
        "fbt_return.multiple_employers",
        "fbt-return was given more than one employer.",
    ),
    (
        "fbt_return.payroll_tax_unsupported",
        "fbt-return was asked for a payroll tax gross-up, which each state and territory sets and it does not work out.",
    ),
    (
        "fbt_tax_exempt_body_entertainment.heads_invalid",
        "The employees and associates entertained are not a JSON integer from 0 to everyone entertained, or everyone entertained is not from 1 to 1,000,000.",
    ),
    (
        "gst_margin_scheme.acquisition_unsupported",
        "The interest was acquired in a way whose eligibility or margin s 75-5(3)(b) to (g), s 75-10(3) item 2A, s 75-11 or s 75-22 governs, which gst-margin-scheme does not work.",
    ),
    (
        "gst_margin_scheme.fact_conflict",
        "The unpaid consideration exceeds the acquisition consideration, or the subdivision share is nil or exceeds the whole.",
    ),
    (
        "gst_margin_scheme.fact_invalid",
        "interest, agreement, acquisition or method is not one of the values gst-margin-scheme takes.",
    ),
    (
        "gst_margin_scheme.supply_to_associate",
        "The supply is to the supplier's associate, whose margin s 75-13 works from the GST inclusive market value, which gst-margin-scheme does not work.",
    ),
    (
        "gst_margin_scheme.valuation_day_invalid",
        "valuation_day is not the day s 75-10(3) fixes for the facts given.",
    ),
    (
        "gst_margin_scheme.valuation_item_unsettled",
        "Both s 75-10(3) item 2 and item 4 describe the supply, and gst-margin-scheme does not choose between their valuation days.",
    ),
    (
        "gst_margin_scheme.valuation_unavailable",
        "An approved valuation was given for an interest acquired on or after 1 July 2000.",
    ),
    (
        "hire_purchase.amount_nil",
        "A hire purchase agreement's or chattel mortgage's amount financed or instalment is nil.",
    ),
    (
        "hire_purchase.car_limit_unknown",
        "A car's first_used_year (hire purchase or chattel mortgage GST facts) is one whose car limit the ATO has not published, or this worksheet does not hold.",
    ),
    (
        "hire_purchase.frequencies_mixed",
        "Hire purchase segments mix weekly or fortnightly with monthly, quarterly or annual instalments.",
    ),
    (
        "hire_purchase.frequency_invalid",
        "A hire purchase or chattel mortgage frequency is not weekly, fortnightly, monthly, quarterly or annual.",
    ),
    (
        "hire_purchase.gst_fact_conflict",
        "Hire purchase or chattel mortgage GST facts conflict: luxury car tax above the price.",
    ),
    (
        "hire_purchase.gst_fact_invalid",
        "The hire purchase or chattel mortgage gst facts are not an object, or one of them is not a value it takes.",
    ),
    (
        "hire_purchase.instalments_unsupported",
        "The number of hire purchase or chattel mortgage instalments is not an integer from 1 to 30 years of them at their frequency, or the segments run past 30 years.",
    ),
    (
        "hire_purchase.payout_date_unsupported",
        "A hire purchase payout date is not the end of an arrangement payment period before the end of the term.",
    ),
    (
        "hire_purchase.payout_gst_unsupported",
        "A hire purchase payout is given with GST facts.",
    ),
    (
        "hire_purchase.payout_invalid",
        "A hire purchase payout is not an object with a date, an amount and known_at_start.",
    ),
    (
        "hire_purchase.payout_known_at_start",
        "A hire purchase payout was known or expected at the start of the arrangement, or could not then be reasonably estimated.",
    ),
    (
        "hire_purchase.rate_imprecise",
        "A hire purchase or chattel mortgage implicit rate cannot be found closely enough to leave a nil balance to the cent, at a high rate over a long term.",
    ),
    (
        "hire_purchase.rate_unsupported",
        "A hire purchase agreement's or chattel mortgage's payments imply an interest rate below nil or of 100% a month (or week) or more.",
    ),
    (
        "hire_purchase.segments_invalid",
        "Hire purchase segments are not a non-empty list of objects, or a segment's start, count, amount or frequency is not a value it takes.",
    ),
    (
        "hire_purchase.segments_not_contiguous",
        "A hire purchase segment does not start on the start date or where the segment before it ends.",
    ),
    (
        "hire_purchase.start_outside_year",
        "A hire purchase agreement or chattel mortgage starts outside the income year given.",
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
        "A field the worksheet or command does not take, or does not take beside the other inputs given, was given (fbt-loan, fbt-meal-entertainment, fbt-car-parking, fbt-debt-waiver, fbt-expense-payment, fbt-property, fbt-residual, fbt-housing, fbt-lafha, fbt-board, fbt-tax-exempt-body-entertainment, fbt-entertainment-classifier, fbt-meal-entertainment-register and its entries, fbt-minor-benefit-check, division-296, depreciation, foreign-resident-tax, medicare-levy, medicare-levy-surcharge, payg-withholding, payg-gross-up, termination-payment, etp-tax-free-component, unused-leave-payment, the Schedule 5 methods, working-holiday-maker-withholding, study-loan-withholding, seniors-pensioners-withholding, seniors-pensioners-tax-offset, lump-sum-arrears-tax-offset, hire-purchase-schedule, chattel-mortgage-schedule, loan-transactions and its transactions and rate_changes, borrowing-expenses-schedule and its loans, business-capital-expenditure and its items, balancing-adjustment, its years and car_limit, trade-in, gst-margin-scheme, wine-equalisation-tax and its dealings, luxury-car-tax and its lines, payroll-tax, contribution-caps given a carry-forward input, derive, personal-tax-estimate, dgr-check, income-tests and fbt-return only).",
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
        "loan_transactions.balance_below_nil",
        "A loan's balance is below nil at the end of a day.",
    ),
    (
        "loan_transactions.date_outside_statement",
        "A loan transaction or rate change is dated outside the statement (a rate change on start_date included).",
    ),
    (
        "loan_transactions.end_date_invalid",
        "A loan statement's end_date is before start_date, or 30 years or more after it.",
    ),
    (
        "loan_transactions.entry_invalid",
        "Loan transactions or rate changes are not a list of objects within the limit, or an entry's date, kind, amount or annual_rate is not a value it takes.",
    ),
    (
        "loan_transactions.not_in_date_order",
        "Loan transactions or rate changes are not in date order, or 2 rate changes fall on one day.",
    ),
    (
        "loan_transactions.start_outside_year",
        "A loan statement starts outside the income year given.",
    ),
    (
        "lump_sum_arrears.accrual_years_invalid",
        "accrual_years is not a list of distinct income years before the year of receipt, each with its arrears and the fields its kind of accrual year takes, as whole-dollar decimal strings.",
    ),
    (
        "lump_sum_arrears.recent_year_unsupported",
        "A recent accrual year is earlier than 2012-13, the first year whose resident rates the worksheet holds.",
    ),
    (
        "luxury_car_tax.consumption_invalid",
        "A fuel consumption rating is not null or a decimal string from 0 to 100 with at most 2 decimal places.",
    ),
    (
        "luxury_car_tax.kind_invalid",
        "A line's kind is not sale, importation, price_adjustment, change_of_use_increasing, change_of_use_decreasing or refund.",
    ),
    ("luxury_car_tax.line_invalid", "A line is not an object."),
    (
        "luxury_car_tax.lines_invalid",
        "lines is not a list of 1 to 1,000 line objects.",
    ),
    (
        "luxury_car_tax.refund_invalid",
        "A refund's claimant is not primary_producer or tourism_operator, or the car was supplied or imported before 1 July 2019.",
    ),
    (
        "luxury_car_tax.refund_limit",
        "More than one primary producer refund on a refund-eligible car was given for the financial year (s 18-5(4)).",
    ),
    (
        "luxury_car_tax.year_invalid",
        "A line's supplied_in is not a financial year whose thresholds the ATO publishes, or is after the tax period's.",
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
        "payroll_tax.grouped",
        "The employer is a member of a group, whose threshold, rate and deduction are not worked.",
    ),
    (
        "payroll_tax.jurisdiction_invalid",
        "jurisdiction is not nsw, vic, qld, wa, sa, tas, act or nt.",
    ),
    (
        "payroll_tax.part_year",
        "The employer did not pay wages for the whole year, and the threshold reduced by days is not worked.",
    ),
    (
        "payroll_tax.wages_conflict",
        "australian_wages is less than taxable_wages, which it includes.",
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
        "trade_in.asset_invalid",
        "replacement_asset is not passenger_car or other.",
    ),
    (
        "trade_in.fact_conflict",
        "The trade-in's facts contradict each other or leave a GST Act fraction undefined.",
    ),
    (
        "trade_in.involuntary_disposal",
        "The traded-in asset was disposed of involuntarily, and the s 40-365 choice is not worked.",
    ),
    (
        "trade_in.not_arms_length",
        "The trade-in is not at arm's length, so market value rules apply that are not worked.",
    ),
    (
        "wine_equalisation_tax.dealing_invalid",
        "A dealing, its average_wholesale, wholesale_sales or rebate_conditions is not the object or list it must be.",
    ),
    (
        "wine_equalisation_tax.dealings_invalid",
        "dealings is not a list of 1 to 1,000 dealing objects.",
    ),
    (
        "wine_equalisation_tax.intention_invalid",
        "A sale under quote's purchaser_intention is not taxable_dealing, manufacture, gst_free_supply or sale_under_quote.",
    ),
    (
        "wine_equalisation_tax.kind_invalid",
        "A dealing's kind is not wholesale_sale, retail_sale, own_use, quoted_sale or own_use_purchased_under_quote.",
    ),
    (
        "wine_equalisation_tax.method_unavailable",
        "The average wholesale price method was asked for wine that is not grape wine, or without at least 10% of the sales by value being wholesale sales.",
    ),
    (
        "wine_equalisation_tax.quantity_invalid",
        "An average wholesale price quantity is not a decimal string above 0, at most 1e12 and 4 decimal places.",
    ),
    (
        "working_holiday_maker.band_unsettled",
        "Payments before are from $135,000.01 to $135,000.99, which none of Schedule 15's columns covers.",
    ),
    ("worksheet.unknown", "The worksheet does not exist."),
];
