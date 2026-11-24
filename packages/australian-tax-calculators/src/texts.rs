//! Scope statements and sources, generated from `austaxcalc.calculations`
//! at the base revision so their wording is exactly the original.

pub const SCOPES: &[(&str, &str)] = &[
    (
        "vic_property_taxes",
        "One Victorian tax on land for the 2025 or 2026 tax year, on values the operator states: vacant residential land tax on one parcel of residential land that met the classification and vacancy conditions of Land Tax Act 2005 (Vic) ss 34B and 34C as in force for that tax year, is not exempt and is not in an alpine resort as defined for that year (including Sch 3 cl 23), as the tax on the parcel for its liable assessed owner (not a deemed owner's personal liability, s 34E); the commercial and industrial property tax on land that entered the tax reform scheme on a stated date, with the earliest tax year it could be taxed if it then meets every condition of s 3; or windfall gains tax on one WGT event that took effect in that year and is not a rezoning excluded by Windfall Gains Tax Act 2021 (Vic) s 3 or a Gazette declaration, with no exemption under ss 37 to 40A or waiver under s 41, on every title rezoned by that one event owned by one assessment unit (one owner, the same joint owners, one trust or a complete group) whose grouping and capacity are already settled under ss 17 to 25. Excluded: separate occupancy valuations (s 14), land not separately valued (s 15), residential land of more than 2 hectares (s 37(2)) and events with a correcting component (s 38(3A)). Values are the Valuer-General's; deferral, interest and apportionment of the tax to titles (s 16(2)) are not worked.",
    ),
    (
        "motor_vehicle_duty",
        "The duty on an application to register, or to transfer the registration of, one passenger vehicle in New South Wales, Victoria, Queensland, Western Australia, South Australia, Tasmania, the Australian Capital Territory or the Northern Territory, for a duty date in 2026-27 (1 July 2026 to 30 June 2027). The operator establishes the vehicle's dutiable value under the jurisdiction's Act and that it is a passenger vehicle there, and, where they set the rate, its Queensland engine row (Duties Act 2001 (Qld) Sch 4C), whether it is a Victorian green car or primary producer passenger car, its ACT emissions category and whether it is new, a Tasmanian manufacturer's fleet discount, and whether it is a Northern Territory electric motor vehicle (Stamp Duty Act 1978 (NT) s 92A). Each scale is the Act's or determination's in force or made on 24 November 2026 (the ACT's DI2025-152 to 31 January 2027 and DI2026-154 from 1 February 2027; New South Wales from Revenue NSW's and Service NSW's pages), charged per $100 (Victoria $200) or part, with South Australia's $5 minimum, Tasmania's $20 for a vehicle of $600 or less, and Western Australia's sliding rate rounded down to 5 cents (Duties Act 2008 (WA) s 236). Excludes motorcycles, goods and commercial vehicles, buses, heavy vehicles, trailers and special vehicles; deciding the dutiable value, class or category; exemptions, concessions and refunds other than the rates named (family and dealer transfers, disability, veterans and Victoria's s 218(2) modification reduction); Queensland's s 384 reduction; South Australia's component for a compulsory third party policy on registration (Stamp Duties Act 1923 (SA) Sch 2 cl 2(3)) and concessional transfers; ACT deemed emissions and the Commissioner's discretion; and penalty tax and interest.",
    ),
    (
        "insurance_duty",
        "The duty on one premium for general insurance in New South Wales, Victoria, Queensland, Western Australia, South Australia, Tasmania, the Australian Capital Territory or the Northern Territory, for a contract effected or renewed in 2026-27 (1 July 2026 to 30 June 2027), at the rate each Act sets for the class the operator states: New South Wales Types A, B and C (from Revenue NSW's pages; Type C 2.5% for a policy taken out before 1 January 2018 and exempt from then), Victoria's general and business insurance (Duties Act 2000 (Vic) s 179), Queensland's class 1, class 2 and CTP insurance (Duties Act 2001 (Qld) s 362), and general insurance elsewhere (in South Australia any insurance other than life insurance, personal accident insurance included; the ACT's duty repealed from 1 July 2017). The operator establishes the class, that the premium is dutiable and not exempt, and the premium as the Act defines it, apportioned to the jurisdiction and the class. Each rate is the one in force on 24 November 2026; the duty is shown to the cent with the exact figure beside it. Excludes life insurance, Queensland accident insurance (Duties Act 2001 (Qld) s 362(3)), CTP insurance outside Queensland, a Northern Territory policy for more than one year, exemptions, apportionment, who is liable, returns and refunds, and penalty tax and interest.",
    ),
    (
        "alcohol_excise",
        "Excise duty on beer, spirits and other excisable beverages manufactured in Australia and entered for home consumption from 1 July 2022 to 31 January 2027, in one financial year from 2022-23 to 2026-27, at the substituted rate the Commissioner published for the day (Excise Tariff Act 1921 Schedule items 1 to 3, s 6A), with the remission for alcohol manufacturers under its yearly cap and the refund of duty paid where it was available (Excise Regulation 2015 Sch 1 cl 2(1) item 10). The operator establishes each line's product, dutiable volume and strength at 20 degrees Celsius under the measurement determinations, container and entry day, and the remission conditions other than the cap. Excludes licences, approvals and the free subitems 3.5 to 3.8, quotas in a declared period, other remissions, refunds and drawbacks, imported goods (customs duty), wine (wine equalisation tax), tobacco and fuel.",
    ),
    (
        "asic_fees",
        "ASIC's standard fee in 2026-27 for registering a company with or without share capital, reserving a company name, a company's annual review (public, proprietary, or special purpose public or proprietary) or registering or renewing a business name for 1 or 3 years, as indexed from 1 July 2026; and for a late payment of an annual review fee or a late lodgement of a document, the late fee for the tier the day ASIC received it falls in: on or before the due date nil, within one month after it $102, more than one month after it $428. The month starts the day after the due date (Acts Interpretation Act 1901 s 2G(1) and s 36(1) item 6). The operator gives the due date: the worksheet never works it out from a review date, and refuses a received date that a move of the due date, or of the month's end, past a weekend could change (AIA s 36(2), Corporations Act s 105(3)). Excludes other ASIC fees, public holidays, fee waivers and refunds, and dates outside 2026-27.",
    ),
    (
        "fuel_tax_credits",
        "Fuel tax credits for taxable fuel acquired from 1 July 2022 to 31 January 2027, in one income year from 2022-23 to 2026-27, by an entity registered or required to be registered for GST, for use in carrying on its enterprise (Fuel Tax Act 2006 s 41-5). The operator establishes each line: the fuel (diesel, petrol, B5, B20, E10, E85, LPG, LNG or CNG, or B100), the day it was acquired, its quantity in litres (LNG and CNG in kilograms) and its use: in a heavy vehicle (gross vehicle mass above 4.5 tonnes) travelling on a public road, including fuel to power a bus's or coach's passenger air-conditioning (QC107638 note 1); in all other business uses, including a heavy vehicle's other auxiliary equipment, travel off public roads and a heavy vehicle whose public road travel is incidental to its main use (s 43-10(4)); or in a vehicle of 4.5 tonnes GVM or less on a public road, which gets no credit (s 41-20). Each line takes the ATO's published rate for the day it was acquired (s 43-5(2) and (2A) item 1; for a heavy vehicle on a public road the rate net of the road user charge, s 43-10(3) and (6)), from the 16 rate periods bundled with this command, each with its ATO page and the day it was read; its credit is the quantity times the rate in cents, divided by 100 and rounded half up to the cent, and the lines' credits are added and the whole dollars, cents dropped, are the amount for BAS label 7D. A small claimant (credits of $10,000 or less in the previous 12 months and expected to be so in the current 12 months) may use the basic method for heavy vehicles for diesel (PCG 2021/2: public road litres are the public road kilometres, actual or 98.4% of the total, times the diesel consumption rate, from the ATO's rigid truck table by gross weight or the engine's diagnostic report; the rest of the diesel takes the other rate), PCG 2016/2 Method 1 (a liquid fuel's litres are its GST inclusive cost divided by the average GST inclusive price the operator reads for its fuel type, area and tax period from the Australian Institute of Petroleum's weekly retail prices) and Method 2 (every liquid fuel line at the rate on the last day of a tax period in which the rate changed only because of indexation). Given the day the return is given, lines acquired more than 4 years before it are flagged under s 47-5(2). Refuses a year outside 2022-23 to 2026-27, an acquisition outside the income year or after 31 January 2027 (the rates change at the February 2027 indexation), a basic method line for a fuel other than diesel, a Method 1 line for LPG, LNG or CNG, a rigid truck of 4.5 tonnes or less, public road litres above the diesel, Method 2 across a change other than indexation alone (29 September 2022, 1 April 2026 and 3 August 2026, and each 1 July) or for LPG, LNG or CNG, and a simplified method without the small claimant statement. Excludes deciding entitlement and registration, who acquired the fuel in a hire or supply chain, whether a road is public or travel incidental, the environmental criteria (s 41-25), apportioning fuel between uses (PCG 2016/8, FTD 2010/1), the auxiliary equipment percentages (PCG 2016/11) and the list of heavy vehicles used mainly off public roads, consumption rates for articulated combinations (give litres_per_km from the engine's report), the average price itself, converting LPG kilograms to litres, aviation fuel, fuel for packaging or supply, domestic heating, electricity generation and non-business claims, fuel manufactured or imported (s 43-5(2A) item 2), fuel tax adjustments (Division 44), attributing credits to a tax period (s 65-5), the s 47-5(1) limit and the s 47-10 exception, and substituted accounting periods.",
    ),
    (
        "gst",
        "One ordinary taxable supply at 10%, already classified by the operator. Excludes mixed supplies, exemptions, margin schemes, adjustments, tax invoice rounding across line items, input-credit entitlement and BAS preparation.",
    ),
    (
        "resident_tax",
        "Australian resident individual, ordinary taxable income only; for a part-year resident (part_year_residency_months, with eligible_pensioner), the tax-free threshold pro-rated under Income Tax Rates Act 1986 s 20 unless s 18(2)(a) applies; with no part-year residency period under s 18(1) (residency beginning and ending within one month, or a 1 July resident ceasing during July) the inputs are omitted and the full threshold applies. Excludes minors' special rates, working holiday makers, offsets, Medicare levy, surcharge, HELP, PAYG credits and tax payable/refundable. Supply established whole-dollar taxable income; this is basic tax before offsets and levies.",
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
        "cgt_parcels",
        "Shares in a company or units in a unit trust held on capital account and disposed of (CGT event A1, ITAA 1997 s 104-10) under contracts made in the 2025-26 or 2026-27 income year, before Act No. 49 of 2026's changes from the income year that includes 1 July 2027, by an individual or a trust that was an Australian resident throughout and to which none of ss 115-105 to 115-120 applies, a complying superannuation entity or a company other than a life insurance company (whose gains on complying superannuation assets can be discount capital gains, ss 115-10(d) and 115-100(b)(ii)). Each parcel is units of one security acquired together, after 21 September 1999. The operator establishes each parcel's acquisition date (the contract date, s 109-5) and cost base elements (s 110-25), after any adjustment another provision requires, such as for bonus shares, rights, demergers, returns of capital and AMIT cost base adjustments, and each disposal's contract date, units, capital proceeds and incidental costs. Units are matched to parcels first in, first out by acquisition date (the earlier listed first on the same day), or as the taxpayer selects, both of which TD 33 accepts. A parcel's elements and a disposal's proceeds and selling costs are apportioned by units to the cent, half up, the last units taking what is left (ss 112-30 and 116-40). A gain is the capital proceeds above the cost base and a loss the reduced cost base, which leaves out costs of owning, above the capital proceeds (ss 104-10(4) and 110-55(2)). A gain is a discount capital gain when the parcel was acquired at least 12 months before the disposal, counting neither day (s 115-25(1)), unless the entity is a company (s 115-10). Disposals in earlier years use up units and are not counted. The year's capital losses, then earlier net capital losses, reduce the gains that are not discount capital gains first and then the discount capital gains (s 102-5(1) steps 1 and 2), and the discount percentage (50%, or 33 1/3% for a complying superannuation entity, s 115-100) reduces each discount capital gain left (step 3). The other gains and losses are the year's established capital gains and losses from other assets, and prior_net_capital_losses the earlier years' unapplied net capital losses, none from collectables or personal use assets. total_capital_gains, net_capital_gain and net_capital_losses_carried_forward are question 18 labels H, A and V of an individual's return when every capital gain and loss of the year is stated. Refuses a parcel disposed of in the year that was acquired at or before 21 September 1999 (indexation under ss 114-1 and 114-10, or pre-CGT under s 104-10(5)). Excludes shares held as trading stock or on revenue account, employee share scheme interests, interests to which s 115-45 applies, wash sales, other CGT events (such as C2, or E4 and G1 where a payment exceeds the cost base), foreign and temporary residents, the small business concessions (step 4), the affordable housing discount and CGT payable.",
    ),
    (
        "cgt_cost_base_adjustments",
        "Parcels of shares in a company or units in a trust held on capital account by an Australian resident individual, trust, complying superannuation entity or company other than a life insurance company (whose gains on complying superannuation assets can be discount capital gains, ss 115-10(d) and 115-100(b)(ii)), each acquired from 20 September 1985, and the events that adjust their cost base and reduced cost base from 1 July 2018 to the end of the income year (2018-19 to 2026-27): a company's non-assessable payment (CGT event G1, ITAA 1997 s 104-135), a non-AMIT trust's non-assessable part for a year (CGT event E4, ss 104-70 and 104-71), an AMIT cost base net amount (ss 104-107A to 104-107C, CGT event E10) and the Wesfarmers demerger of Coles (2018) and Woolworths demerger of Endeavour (2021) under their class rulings (Subdivision 125-B). The operator establishes each parcel's cost base just before the first event and the units held when each event happened (a demerger takes the parcels acquired by its record date), the non-assessable part after s 104-71 and any tax-free amount from the trust's statement, the AMIT cost base net amount from the AMMA statement, the parcels a year's amount was paid for when one was acquired in that year, the date of any other CGT event to the units in that year after the payment (an E4 or AMIT amount then happens just before it, ss 104-70(3)(b) and 104-107B(4)(b)), and the roll-over choice. A parcel takes no E4 amount and an AMIT amount in the same year, nor 2 AMIT amounts (ss 104-70(1A) and 104-107C). A parcel acquired at or before 21 September 1999 is worked without indexation (refused for a company); pre-CGT parcels, other demergers, partial roll-over (s 125-80(8)), foreign residents, revenue assets and trading stock (ss 104-107G and 104-107H), liquidator payments (s 104-135(6)), s 104-70(8) and (9), employee share scheme interests and the disposals themselves (cgt-parcels) are outside it. From 2027-28 Act No. 49 of 2026 changes how capital gains are worked out, so 2027-28 is refused.",
    ),
    (
        "main_residence",
        "An individual who was an Australian resident when the CGT event happened, disposing of (CGT event A1) one dwelling, with any adjacent land used mainly for private or domestic purposes with it and of up to 2 hectares in total, acquired on or after 20 September 1985 and held as the same ownership interest throughout, with the disposal in the 2024-25, 2025-26 or 2026-27 income year, before the changes from 1 July 2027. The operator establishes the gain (cost base) or loss (reduced cost base) on the whole interest held, before any exemption or discount; the ownership period's first and last days; the days the dwelling was their main residence, including any treated as such from acquisition under s 118-135; the days it or part of it produced assessable income where interest would have been deductible, with the part of the floor area used; the days covered by a choice under s 118-145; and when a new home that became their main residence was acquired. Every day is counted, both ends included. Each day is taxable in full, in part (the floor area used while living there, s 118-190) or not at all. A choice period keeps the dwelling exempt, except a part used to produce income before the move, which stays taxable; income use is covered for 6 years per absence, to the sixth anniversary of its first day inclusive, as the ATO counts it, and counted across breaks within the same absence; later income-producing days are taxable. Under s 118-140 the last 6 months of ownership, from the same day of the month 6 months earlier (or the first of the next month where that day does not exist), or from the new home's acquisition if later, are exempt when the dwelling was the main residence for a continuous 3 months in the 12 months before ownership ended, which start the day after the same date a year earlier, and produced no income in that time while not the main residence. Under s 118-192, when income use first began after 20 August 1996, a full exemption would have applied just before, and the dwelling is only partly exempt because of that use, the ownership period starts on that day and the gain or loss must be worked out from the market value then. The assessable part is the gain or loss times the taxable days over the days in the ownership period (s 118-185). Excludes deciding residence, main residence status, interest deductibility, floor area, the choice itself and market value; inherited dwellings (s 118-195); a spouse or dependant child with a different main residence (s 118-170); building, repairing or renovating (s 118-150) and destroyed dwellings (s 118-155); adjacent land over 2 hectares or sold separately; compulsory acquisitions; marriage or relationship breakdown rollovers; granny flat arrangements; another dwelling treated as the main residence during a choice period except under s 118-140; the CGT discount and its 12-month test; and small business concessions. Refuses income use of part of the dwelling that changed, paused or stopped between its start and a move that starts a choice period, and a partial exemption with causes besides income use where s 118-192 would otherwise apply.",
    ),
    (
        "hire_purchase_schedule",
        "One hire purchase agreement whose hirer is the notional buyer (ITAA 1997 s 240-17(2)), starting in the income year given, with instalments paid in arrears (at each period's end) or in advance (at its start) and any balloon paid at the end of the term. The instalments are one level amount weekly, fortnightly, monthly, quarterly or annually, or dated segments of level amounts (a nil amount for a payment holiday), all counted in weeks or all in months, over at most 30 years. The operator establishes the amount financed (the notional loan principal: the consideration less any deposit paid or trade-in credited at or before the start, s 240-25(3)), the instalments, the balloon, the start date and any deposit, trade-in, payout and GST facts. A deposit or trade-in is taken as paid or credited on the start date, and the goods and the credit component as recorded on one tax invoice. Months run from the start date's day of the month, or the month's last day when it is shorter; weeks are 7 days. Each instalment's period is an arrangement payment period (s 240-70(1)), and an annual period is 2 periods of 6 months (s 240-70(2)). Under s 240-60(1) each period's notional interest is the outstanding notional loan principal, less any instalment due at the period's start, times the implicit interest rate for the period: (1 + r)^n - 1 for a period of n months (or weeks), where r is the compound rate a month (or a week) at which the payments repay the amount financed, found to 64 halvings of 0 to 100%. The payments themselves are not deductible but are taken into account in the notional interest (s 240-55). Each period shows the notional interest to date rounded half up to cents, less what the periods before show, so the periods add up to the total charges, each is within a cent of its amount at the rate found and the last balance is nil. The balloon is shown apart from the last instalment, on its own date. Journal lines for the start, each period's interest, each payment on its day and any payout carry the liability at the notional loan principal outstanding, and are totalled by income year: the interest by its period, with the periods spanning 30 June set apart, and each payment by its day. An early payout on the end of a period, neither known nor expected at the start, replaces every payment due from its date: the worksheet gives the notional interest and payments to that date, the balance outstanding, the difference and, under Subdivision 240-G (ss 240-105 and 240-110), which way the difference goes, flagged for review. With every GST fact given, it gives a registered hirer's input tax credit for an agreement entered into from 1 July 2012: 1/11 of every component of the supply, the notional interest included (the ATO's 'GST – Hire purchase and leasing'), less any luxury car tax for a car (GST Act s 9-75(2)), rounded to the nearest cent, 0.5 up (s 9-90), times the extent of creditable purpose (s 11-30(3)), attributable up front whether or not the hirer accounts on a cash basis (ss 29-10(1) and 158-5). A passenger car whose GST inclusive market value exceeds the car limit for the financial year it is first used ($69,674 for 2024-25 and 2025-26, $69,883 for 2026-27), where the hirer may not quote an ABN for luxury car tax, needs a held tax invoice with separate goods and finance consideration and GST, all reconciled to the agreement. Section 69-10(1) caps the goods credit only. The worksheet gives no GST figure without the invoice split and makes no GST attribution or journal entry for separately invoiced supplies. Refuses a rate below nil or of 100% a month (or week) or more, a rate that cannot be found closely enough to leave a nil balance to the cent, a term over 30 years, a nil amount financed or instalment, segments that do not run on from the start or mix weeks and months, a payout not on the end of a period before the term ends, a payout with GST facts, a payout known or expected at the start, a car first used in a year whose car limit is not published, and luxury car tax above the price. Excludes deciding whether Division 240 applies and what is financed; fees, charges and stamp duty; payments off a period boundary; substituted accounting periods; termination amounts other than the balloon and a stated payout, and estimates of them (s 240-60(3) and (4)); apportioning a period that spans 30 June between income years (s 240-60(2)); whether the notional interest is deductible (s 240-50) and any reduction of it for the input tax credit; the car's cost for Division 40 (ss 27-80 and 40-230); agreements entered into before 1 July 2012; GST adjustments after the start; the trade-in's own GST; and the notional seller.",
    ),
    (
        "foreign_purchaser_surcharge",
        "The foreign purchaser duty surcharge on one buyer's share of one property, for a transaction from 1 July 2026 to 30 June 2027, and the foreign or absentee owner land tax surcharge for a full land tax year (2026 in New South Wales and Victoria, 2026-27 in Queensland, Tasmania and the ACT), in one of the 8 states and territories, at the rates in force when read on 24 November 2026. The operator establishes whether the buyer is a foreign person (or foreign corporation or trust) under the jurisdiction's duty surcharge test and whether the owner is a foreign person or absentee under its land tax surcharge test, what is residential or primary production property, and the values. Duty surcharges: New South Wales 9% (Duties Act 1997 s 104U), Victoria 8% (Duties Act 2000 s 28A), Queensland 8% (Duties Act 2001 s 244), Western Australia 7% (Duties Act 2008 s 205O), South Australia 7% (Stamp Duties Act 1923 s 72) and Tasmania 8%, or 1.5% on primary production property (Duties Act 2001 ss 30C and 30E), of the share's dutiable value; none in the ACT or the Northern Territory. Land tax surcharges: New South Wales 5% of the foreign owner's share of the residential land's value; Victoria 4% of the whole taxable value from $50,000 (the absentee owner rates less the general rates); Queensland 3.0c for each $1 more than $349,999 from $350,000; Tasmania 2% of the foreign-owned residential land, rounded to 10 cents (Land Tax Rating Act 2000 s 7), on land acquired by a foreign person, or held by a company or trust that became foreign, on or after 1 July 2022; the ACT 0.75% of the AUV; none in Western Australia, South Australia or the Northern Territory. New South Wales, Tasmania and the ACT charge the land tax surcharge on residential land only. Amounts are worked exactly and shown to the cent, half up. Refuses a trust that is a foreign owner in Victoria, whose absentee surcharge turns on its beneficiaries' interests (Land Tax Act 2005 s 46IA). Excludes who is foreign or absentee, exemptions, refunds and the Commissioners' discretions; the general duty and land tax (see transfer-duty and land-tax); Tasmania's lower rates for agreements made before 1 April 2020 (Duties Act 2001 ss 30C(2A) and 30E(2A)); surcharge charged or reassessed after a buyer becomes foreign or the use changes (Stamp Duties Act 1923 (SA) s 72(7), Duties Act 2001 (Tas) ss 30D and 30F); the ACT's quarterly surcharge for part of a year or on units (Land Tax Act 2004 s 17E(3)(a)); FIRB fees; and any change made after 24 November 2026.",
    ),
    (
        "dasp_tax",
        "One departing Australia superannuation payment paid in a financial year from 2025-26 to 2027-28: the tax on it by component (Superannuation (Departing Australia Superannuation Payments Tax) Act 2007 s 5). The operator establishes that the payment is a departing Australia superannuation payment (ITAA 1997 s 301-170: a superannuation lump sum paid to a person who has departed Australia under the specified regulations, s 67A of the Small Superannuation Accounts Act 1995, an exempt public sector scheme's substantially similar rules, or s 20H of the Superannuation (Unclaimed Money and Lost Members) Act 1999), its tax free component and the elements taxed and untaxed in the fund of its taxable component, as the paying fund states them, whether it includes amounts attributable to contributions made whilst the person was a working holiday maker, and whether it is a roll-over superannuation benefit paid under s 20H(2), (2AA) or (2A) of the Superannuation (Unclaimed Money and Lost Members) Act 1999. The tax free component bears nil; the element taxed in the fund 35% and the element untaxed in the fund 45% (s 5(1)); where the payment includes working holiday maker contributions, both elements bear 65% (s 5(3)). The tax is the person's final liability on the payment, which is not assessable income and not exempt income (ITAA 1997 s 301-175). Refuses a payment of nil, and such a roll-over with an element untaxed in the fund. Excludes deciding whether the payment is a departing Australia superannuation payment, its components and whether contributions were made whilst a working holiday maker; a roll-over superannuation benefit paid under s 20H(2), (2AA) or (2A) of that Act with an excess untaxed roll-over amount (s 5(2): taxed at nil here and under the Superannuation (Excess Untaxed Roll-over Amounts Tax) Act 2007 instead); payments in a temporary budget repair levy year (s 6: 2014-15 to 2016-17); the payer's withholding, payment summary and reporting duties; and a refund of tax wrongly withheld.",
    ),
    (
        "super_benefit_tax",
        "One superannuation lump sum paid in a financial year from 2020-21 to 2026-27 from a complying superannuation plan to a member, a death benefits dependant or a non-dependant: its components by the proportioning rule and the most income tax each part can bear before Medicare levy (ITAA 1997 Divisions 301, 302 and 307). The operator establishes that the payment is a superannuation lump sum paid on a condition of release, not a roll-over, not a commutation of an income stream and not a benefit paid in breach of the payment rules (s 304-10 makes such a benefit assessable in full), the value of the interest and its tax free component at the time s 307-125(3) fixes, the element untaxed in the fund of the benefit's taxable component as the fund states it, whether the recipient is the member, a death benefits dependant (s 302-195) or neither, the payment date, and for a member the date of birth, whether a terminal medical condition exists (s 303-10), whether the benefit is a disability superannuation benefit, any requirement the regulations set for a lump sum under $200 (s 301-225(1)(d)) and the low rate cap and untaxed plan cap amounts counted for earlier lump sums. The tax free component is the benefit times the interest's tax free component over its value (s 307-125(2)), rounded to the cent, and the taxable component the rest; its element taxed in the fund is the taxable component less the element untaxed in the fund (s 307-275). A member's age is taken on the payment date and the preservation age from the date of birth (SISR reg 6.01(2)). A member 60 or over pays nothing on the tax free component and the element taxed in the fund (s 301-10), and on the element untaxed in the fund at most 15% up to the untaxed plan cap left and the top rate above it (s 301-95). A member at or above preservation age and under 60, possible only to 2023-24, pays nothing on the tax free component (s 301-15), at most 0% on the element taxed in the fund up to the low rate cap left and 15% above it (s 301-20), and on the element untaxed in the fund at most 15% up to what is left of the low rate cap after the element taxed in the fund, 30% up to the untaxed plan cap left and the top rate above it (s 301-105). A member under preservation age pays nothing on the tax free component (s 301-30), at most 20% on the element taxed in the fund (s 301-35) and on the element untaxed in the fund at most 30% up to the untaxed plan cap left and the top rate above it (s 301-115). A member's lump sum with a terminal medical condition (s 303-10), a member's lump sum under $200 that leaves the interest's value nil (s 301-225(1)) and a death benefit to a dependant (s 302-60) are wholly not assessable income and not exempt income. A member's wholly tax free lump sum still counts its element untaxed in the fund against the untaxed plan cap (s 307-350). A non-dependant pays nothing on the tax free component (s 302-140) and at most 15% on the element taxed in the fund and 30% on the element untaxed in the fund (s 302-145). Each part's tax is its amount times the rate, rounded to the cent; maximum_tax is their sum, the most the benefit can bear before Medicare levy, as the person's marginal rates can make it less. The low rate cap left is the year's cap less the amounts counted for earlier lump sums (s 307-345), and the untaxed plan cap left the year's cap for the plan less the elements untaxed in the fund counted from it (s 307-350); the amounts this benefit counts against each are shown. Refuses a nil benefit or interest, a benefit above the interest's value, a tax free component of the interest above its value, an element untaxed in the fund above the taxable component, a payment date outside the year, a date of birth not before it, a disability superannuation benefit (s 307-145), an element untaxed in the fund with service before 1 July 1983 (s 307-150), cap amounts counted above the year's cap and member facts for a death benefit. Excludes income streams of every kind, including death benefit, reversionary and capped defined benefit income streams and the defined benefit income cap; transfer balance questions; roll-overs and excess untaxed roll-over amounts; departing Australia superannuation payments (dasp-tax); death benefit termination payments (Division 82) and other employment termination payments; benefits from a non-complying or foreign superannuation fund (Division 305); a death benefit paid to the trustee of a deceased estate (s 302-10); structured settlements and personal injury payments; an alternative method under s 307-125(4); deciding the components, dependency and a terminal medical condition; the Medicare levy and the tax at the person's marginal rates; and the payer's withholding and reporting.",
    ),
    (
        "super_withholding",
        "The amount a payer withholds under Schedule 12 of the PAYG withholding schedules from one superannuation lump sum or roll-over paid from 13 October 2020 to 30 June 2027 in a financial year from 2020-21 to 2026-27. The operator establishes the facts super-benefit-tax takes, whether the payee had given the payer their TFN, whether the payee is a foreign resident and, if so, that no tax treaty makes the payment assessable only in another country. The components, ages, treatments and caps are those of super-benefit-tax. With a TFN, each part is withheld at Table A's rate: the tax offset's ceiling rate plus the 2% Medicare levy for an Australian resident (22% on a member's element taxed in the fund under preservation age; nil up to the low rate cap left and 17% above it from preservation age to 59, possible only to 2023-24; nil at 60 and above; on the element untaxed in the fund 32% under preservation age, 17% up to the low rate cap left then 32% from preservation age to 59, 17% at 60 and above, each up to the untaxed plan cap left, and 47% above it; 17% and 32% on a non-dependant's elements), the ceiling rate itself for a foreign resident, and the total is rounded to the nearest dollar, 50 cents up. Nothing is withheld from a terminal medical condition payment, a member benefit under $200 that is the payee's entire benefit, a death benefit to a dependant or a death benefit to the trustee of a deceased estate. A roll-over has 47% withheld from the element untaxed in the fund above the untaxed plan cap left and nothing else, with or without a TFN and for a foreign resident; without a TFN, cents are ignored. Without a TFN, 47% (45% for a foreign resident) of the taxable component is withheld from a member under 60, or of the element untaxed in the fund from a member 60 or over, ignoring cents. Refused: a payment before 13 October 2020; no TFN for a non-dependant or a member benefit under $200, for which the schedule states no rate; and everything super-benefit-tax refuses. Excluded: tax treaties, commutations of income streams, departing Australia superannuation payments (dasp-tax), the tax on the payment (super-benefit-tax) and study and training support loans, from which nothing is withheld for these payments. With income_stream_period (weekly, fortnightly, monthly or quarterly), the payment is instead one superannuation income stream payment in 2024-25 to 2026-27 under Schedule 13 Part A: an account-based stream, or a capped defined benefit stream to a payee under 60, paid to the member or a death benefits dependant. Its components are proportioned on the interest's value and tax free component when the stream commenced (s 307-125(3)(a)). Cents in the stream are ignored; withholding applies to the taxable component under 60 and the element untaxed in the fund at 60 or over; the PAYG withholding tax table (Schedule 1, scale 2 with the tax-free threshold, scale 1 without) gives the withholding amount; the tax offset per payment is 15% of the element taxed in the fund for a disability stream under 60 or a dependant under 60 whose deceased died under 60, 10% of the element untaxed in the fund at 60 or over, and otherwise nil; where it is above nil, the offset adjustment on the weekly equivalent covers the Medicare levy; the greater of the withholding amount less the offset and the adjustment is withheld, rounded to the nearest dollar, 50 cents up. Without a TFN, 47% (45% for a foreign resident) of the amount withholding applies to, ignoring cents; for an income stream, tfn_provided is false only when the payee has not quoted a TFN, has not claimed an exemption from quoting one and has not told the payer of a TFN application or enquiry (or 28 days have passed since), and true for an exemption or an application within the 28 days, which Schedule 13 withholds from like a quoted TFN. Refused for an income stream: years before 2024-25, a capped defined benefit stream to a payee 60 or over (Parts B to E), a dependant under 60 whose deceased died aged 60 or over, a foreign resident with a TFN and an annual payment. Excluded for an income stream: a SAPTO claim through withholding (Schedule 9), Medicare levy variation declarations, study and training support loan debts (Schedule 8) and tax treaties.",
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
        "franking_account",
        "One corporate tax entity's franking account for an income year from 1 July to 30 June, 2025-26 or 2026-27, under ITAA 1997 Division 205 as read on the ATO's legal database on 19 November 2026, for a company, corporate limited partnership or public trading trust that satisfies the residency requirement (s 205-25) for the year and for every event, and is a franking entity throughout it. The operator establishes the balance at the end of the previous year (a deficit as a negative amount, which that year's franking deficit tax credits at the start of this one: s 205-15(1) item 5); the corporate tax rate for imputation purposes, 25% or 30%; the R&D deferral left from earlier years (s 205-15(4)); each event with its date and amount: PAYG instalments and income tax paid (items 1 and 2, reduced by the R&D deferral), franked distributions received and flowing indirectly (items 3 and 4, the franking credit or its share), item 4A, exploration credits (item 7) and Australian DMT tax paid (item 9), and the debits for income tax refunds (item 2, less the part attributable to a refundable R&D tax offset, s 205-30(2)(b), which joins the deferral), linked distributions, tax-exempt bonus shares, streaming determinations, share capital tainting and untainting, buy-backs, selective capital reductions and DMT refunds (items 5, 6, 7, 7A, 7B, 9, 9A, 9B and 14), each at the amount the provision gives; and each frankable distribution, in date order, with the franking credit allocated. A distribution's franking credit is the allocated credit up to its maximum, the distribution over the gross-up rate, (100% less the rate) over the rate (ss 202-60 and 202-65), and is debited on its day (item 1); its franking percentage is the allocated credit over the maximum, at most 100% (s 203-35). The first frankable distribution in each franking period (the income year for a private company, otherwise its first 6 months and the rest: ss 203-40(2) and 203-45) sets the benchmark (s 203-30); a later one franked above the benchmark, or a percentage the Commissioner permitted (s 203-55), gives over-franking tax and one below it a franking debit (item 3), each the distribution times the differential over the gross-up rate (s 203-50; Over-franking Tax Act 2002 s 5), unless s 203-20 exempts the entity. A refund paid in the 3 months after the year's end is taken to be received just before it when it would put the account into deficit or further into it (s 205-50(2)), and is otherwise left for the next year. Franking deficit tax is the deficit at the year's end (s 205-45(2); Franking Deficit Tax Act 2002 s 5). The year's offset (s 205-70(2) step 1) is that tax less 30% of the part of it attributable to item 1, 2, 3, 5 and 6 debits when that part exceeds 10% of the year's credits, by the ATO's method: the part is the opening surplus and the year's credits less those debits, when below nil, and item 2 debits count only beside an item 1, 3, 5 or 6 debit (s 205-70(8)). It is not reduced for a private company that meets s 205-70(5) or under a Commissioner's determination (s 205-70(6)). With income_tax, the offset and the offset brought forward (s 205-70(1)(c)) are applied to the income tax left after the offsets s 63-10 applies first, and the rest is carried forward. Each franking period's benchmark is compared with the last one for a significant variation: more than 20 percentage points for each franking period after that one, up to and including this one (s 204-70), not where the benchmark rule does not apply. The franking return is required where there is franking deficit tax, over-franking tax or a significant variation (the ATO's 'How to get'); it and the tax are due on the last day of the month after the year's end (s 214-150(1)), tax from a refund after the year's end with an outstanding return or 14 days after the refund (s 214-150(4)), and general interest charge runs from the due date (s 214-155). Amounts are worked exactly and shown to the cent, half up; the return's whole-dollar label figures are not worked out. Excludes deciding any event's amount or item, whether a distribution is frankable, the residency requirement and the corporate tax rate for imputation purposes; an entity that ceases to be a franking entity (item 4 debit, s 205-45(3) and s 205-50(3)), joins or leaves a consolidated group, or is a New Zealand franking company, a pooled development fund, a friendly society (item 6 credit, items 10 to 12 debits), a life insurance company or an exempting or former exempting entity (Division 208); items 2A, 6A, 6B, 8 and 13 (diverted profits tax); an earlier year in which the residency requirement was not met (s 205-70(1)(b)); RSA components (ss 205-15(3) and 205-30(2)(a)); substituted accounting periods and late balancers' 30 June election; deciding whether a late distribution statement was allowed (s 202-75(4)), which it flags; and the lodgment itself.",
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
        "land_tax",
        "One owner's land tax for a full land tax year in one of New South Wales and Victoria (the 2026 land tax year, on land owned at midnight on 31 December 2025) and Queensland, Western Australia, South Australia, Tasmania and the Australian Capital Territory (2026-27); the Northern Territory has no land tax. The operator establishes the jurisdiction's total taxable value of the owner's land after exemptions and concessions (in New South Wales the 3-year average land value, in Victoria and South Australia the site value, in the ACT one parcel's average unimproved value), whether the owner pays trust rates (a New South Wales special trust, Victorian trust surcharge rates, Queensland company and trustee rates other than for the trustees Land Tax Act 2010 (Qld) s 33 taxes at Sch 1, or South Australian trust rates under Land Tax Act 1936 (SA) s 8A(1a) after the exceptions in s 8A(1b) and (1c)), whether the owner is a foreign person or absentee who pays the jurisdiction's surcharge on all of the land (in the ACT, a parcel wholly foreign-owned: s 17F apportions), the value a New South Wales or Tasmanian surcharge applies to (in Tasmania the foreign-owned part of one folio, Land Tax Act 2000 (Tas) s 16B(1)), and in Western Australia whether all of the land is in the Perth metropolitan region. Each scale is the Act's or the revenue office's, with the surcharges of New South Wales (5% of the surcharge value), Victoria (the absentee owner rates less the general or trust rates), Queensland (3.0c for each $1 over $349,999 from $350,000, Land Tax Act 2010 (Qld) Schs 2 and 3), Tasmania (2% of the surcharge value) and the ACT (0.75%), and Western Australia's metropolitan region improvement tax (0.14% over $300,000). Tasmania's amounts are rounded to 10 cents (Land Tax Rating Act 2000 (Tas) s 7), and a case where rounding each amount and their sum differ is refused; South Australia's land tax under $20 is nil (Land Tax Act 1936 (SA) s 6); other amounts are worked exactly and shown to the cent, half up, the worksheet's convention, before any rounding a revenue office applies to the amount it collects (such as Taxation Administration Act 2001 (Qld) s 152). The ACT figure is for a taxable parcel liable in all 4 quarters. Excludes deciding which land is taxable, exemptions and concessions, who is a foreign person or absentee, and the taxable and surcharge values; joint ownership, grouping of related companies, single holding assessments, New South Wales companies that are not their group's concessional company, Victorian absentee trusts (s 46IA), Queensland build-to-rent land (s 58B), an exempt ACT parcel, the Victorian COVID-19 debt temporary surcharge on single holdings, Queensland trust aggregation, land only partly in the Perth metropolitan region, the ACT's unit entitlement and quarterly instalments, apportionment between properties, and part-year or supplementary assessments.",
    ),
    (
        "workers_compensation_premium",
        "One employer's workers compensation premium for a 12-month policy or premium period in 2026-27 in one of New South Wales, Victoria, Queensland, Western Australia, South Australia, Tasmania, the Australian Capital Territory and the Northern Territory, on the wages it estimates and the industry or premium rate its insurer gives it, with the premium worked again on its actual wages and the same scheme facts and the adjustment. The operator establishes the wages as the scheme defines them, the rate, the apprentice wages and each scheme's facts. New South Wales: a small employer (average performance premium, wages x the WIC rate, of $30,000 or less) under icare's formula with no apprentice incentive, the average performance premium less the Safe Employer Reward, plus GST, then the dust diseases contribution at a rate that includes GST (the gst amount excludes the GST within that contribution); a premium under icare's $240 minimum is refused. Victoria: a new employer (cl 12(5), including its related-employer and predecessor conditions) or small employer under the WorkCover Premiums Order (No. 34) 2026/2027 cl 6 to 12, with one premium rate and one remuneration figure (one workplace and coverage period), N / 365 to 6 places and the $400 minimum premium, 0.1 buy-out value and 15,500 deductible amount of Gazette S 284, plus GST. Queensland: a new policy (not reinstated under s 11 or carried on by a successor under s 12) that commences to insure in 2026-27 under WorkCover Queensland Notice (No. 1) of 2026 ss 3.2, 4.1, 5.2, 8.3 and 10.1 with no further premium under s 3.1, plus GST and insurance duty of 5% of the premium and its GST (Duties Act 2001 (Qld) s 362(3)), with s 4.1's minimum including both. South Australia: ReturnToWorkSA's BP x (1 - D) + C - A with its discount bands, claims costs capped at 3 x D x BP and no discount or claims costs while the employer has not completed a full financial year registered, at least $200 and rounded down to the dollar, plus GST; the supplementary underwriting rate is nil. Western Australia, Tasmania, the ACT and the Northern Territory: a policy whose premium is the insurer's rate times this employer's wages, plus GST, with no minimum, fixed charge, other adjustment or group allocation. Each step is worked exactly and shown to the cent, half up, except where a scheme rounds otherwise. Refuses experience-rated and performance-rated premiums, group members in New South Wales, Victoria, Queensland and South Australia, a New South Wales apprentice incentive, a Victorian remuneration that rounds to $0, a Queensland policy that commenced before 1 July 2026, a Queensland premium that rounding the estimated premium and apprentice discount first would move, Queensland actual wages, and a South Australian base premium above a band's bound less a cent and below the bound (ReturnToWorkSA's table and calculator differ). Excludes the industry classification and rate, what counts as wages, exemption from holding a policy, the Mine Safety Fund adjustment, catastrophic claims contribution and performance discount in New South Wales, default assessments, policies carried on from an earlier policy, business transfers in South Australia, the South Australian work health and safety fee, Comcare and self-insurers, household workers, early payment discounts and instalment interest.",
    ),
    (
        "payroll_tax",
        "One employer's payroll tax for the 2026-27 financial year (1 July 2026 to 30 June 2027) in one of New South Wales, Victoria, Queensland, Western Australia, South Australia, Tasmania, the Australian Capital Territory and the Northern Territory, for an employer that is not a member of a group and paid or was liable to pay wages on every day of the year. The operator establishes the employer's taxable wages in the jurisdiction (the wages its Act makes taxable there, after its exemptions) and its Australian wages (those taxable wages with its interstate wages, the wages taxable in every other jurisdiction), and in Victoria whether it is a regional employer (Payroll Tax Act 2007 (Vic) s 3A). The result is the annual liability under each Act's schedule before any rebate, discount or cap, such as Queensland's regional discount (Payroll Tax Act 1971 (Qld) s 10A) and the 6.85% cap for an eligible university in the ACT (DI2026-151 s 5(c)(i)), and before the amounts paid in periodic returns.",
    ),
    (
        "transfer_duty",
        "2026-27 transfer (stamp) duty on one dutiable transaction in one of New South Wales, Victoria, Queensland, Western Australia, South Australia, Tasmania, the Australian Capital Territory and the Northern Territory, occurring (the contract or agreement the Act charges, or the transfer where there is none) from 1 July 2026 to 30 June 2027, at the rates and concessions in force when read on 24 November 2026. The operator establishes the dutiable value (generally the greater of the consideration and the property's unencumbered or market value), whether the dutiable property is wholly residential land (in New South Wales one parcel of residential land under Duties Act 1997 (NSW) s 32A(3), no more than 2 hectares and not partly used for other purposes, s 32B and s 32C being outside this worksheet), and, for a first home buyer, that every condition of the jurisdiction's first home concession is met (citizenship or residency, never having owned a home, age, the residence requirement) and that the dutiable property is the home and its land alone. The general duty is the jurisdiction's general rate, with New South Wales' premium rate on residential land over $3,870,000 and its $20 minimum (s 273), and Victoria's duty to the nearest whole dollar, 50 cents down (Duties Act 2000 (Vic) s 28(1)). A first home buyer of a home pays: in New South Wales nil to $800,000 and below $1,000,000 the s 78A amount, the bracket rounded to the cent as Revenue NSW's calculator does; in Victoria nil to $600,000 and to $750,000 the s 57JA share; in Queensland, for an established home, the s 91 home concession duty less the Sch 4A amount, and nil for a new home (s 92A); in Western Australia nil to $600,000 and to $800,000 $16.15 for every $100 or part over $600,000 (Duties Act 2008 (WA) Sch 2 Div 2); in South Australia nil for a new home (Stamp Duties Act 1923 (SA) s 71DD) and the general rate for an established one; in the ACT nil under the Home Buyer Concession Scheme (DI2026-157), which is open to buyers without an interest in land for 5 years; and in Tasmania and the Northern Territory the general rate, as neither has a first home concession for 2026-27. Other amounts are worked exactly and shown to the cent, half up, before any rounding a revenue office applies to the amount it collects. Refuses land that is not wholly residential in South Australia (qualifying land) and the ACT (commercial rates), in New South Wales above $3,870,000 (where s 32B or s 32C may charge premium duty), and for a first home buyer. Excludes whether a transaction is dutiable, its dutiable value and who is liable; eligibility for a concession; every other concession, exemption and surcharge (Victoria's principal place of residence, off-the-plan and pensioner concessions, Queensland's home concession for a buyer who is not a first home buyer, Western Australia's concessional rate to $200,000, the ACT's owner occupier rates, the Northern Territory's house and land package exemption, first home buyers of vacant land, shared equity and every foreign purchaser surcharge); landholder, corporate trustee and trust acquisition duty; partial interests, joint tenants and aggregation; and any change made after 24 November 2026 to a rate or concession for later transactions in 2026-27.",
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
        "One loan's borrowing expenses under ITAA 1997 s 25-25 for one income year from 2017-18 to 2027-28. The operator establishes the expenditure the taxpayer incurred for borrowing the money, all in the income year the loan began; the extent the borrowed money is used to produce assessable income, the same throughout the loan; the first day the money was borrowed; the last day of the period the original loan contract specifies; any repayment day; and the income-producing share of the taxpayer's other borrowing expenses incurred in that year. Under s 25-25(6), when the money is used to produce income and this loan's income-producing share plus those other amounts is $100 or less, that share is deducted in the year incurred and later years are refused. Otherwise the period of the loan is the shortest of the contract's period, the period to the repayment day and 5 years from the first day borrowed (s 25-25(5)). Each income year's maximum deduction is the remaining expenditure over the days left in that period, times those days in the year, as determined at the end of the year, so the year of repayment takes all that remains (s 25-25(4)); each maximum is rounded to cents and carried forward at that amount, as the ATO's examples are, and the deduction is the maximum times the income-producing share (s 25-25(2) and (3)). Every day is counted, both ends included, as the ATO and the Library count: 5 years from 3 July end on 2 July, and from 29 February on 28 February (Acts Interpretation Act 1901 ss 2G(2) and 36(1)). From 2027-28 the result is flagged, not changed: s 26-155 may quarantine it. Excludes deciding which costs are borrowing expenses and the money's use; interest; refinancing, redraws and changes in use (borrowing_expenses_schedule takes refinancing and use by year); expenditure incurred before the income year the loan began; leisure facilities such as holiday homes unless s 26-50(3) applies at all times in the year; commercial debt forgiveness (Subdivision 245-E); private company loans (Division 7A); and applying the limits under ss 26-102 and 26-155. The deduction amounts are the s 25-25 calculation before either limit.",
    ),
    (
        "borrowing_expenses_schedule",
        "Several loans' borrowing expenses under ITAA 1997 s 25-25, each loan first drawn from 1 July 2017 to the end of the income year asked about, from 2017-18 to 2027-28: that year's amounts across the loans, and every income year of each loan with the totals by year. The operator establishes, for each loan, the expenditure incurred for borrowing the money, all in the income year the loan began; the first day the money was borrowed; the last day of the period the original loan contract specifies; any repayment day; the extent the money is used to produce assessable income, for every year or for each income year of the loan; and, for a loan that refinances an earlier one, which loan it repaid, on the day it was first drawn; and the income-producing share of borrowing expenses on loans not listed, by the income year incurred. Under s 25-25(6), the income-producing shares of everything incurred for borrowing in an income year, at $100 or less in total, are deducted that year; a loan whose money is not used to produce income at all that year is spread. Otherwise each loan is spread over its own period of the loan as borrowing_expenses spreads one loan (s 25-25(4) and (5)), each year's deduction being its maximum times that year's income-producing use (s 25-25(2) and (3)). A refinanced loan is repaid on the refinancing day, so its period ends then and that income year deducts what remains (s 25-25(5)(b)). A row from 2027-28 is flagged, not changed, because s 26-155 may quarantine it, and a row after 2027-28 applies s 25-25 as read on 19 November 2026 and is marked projected. Excludes deciding which costs are borrowing expenses, the money's use and the tracing of redraws; interest; expenditure incurred after the income year a loan began; loans first drawn before 1 July 2017; leisure facilities such as holiday homes unless s 26-50(3) applies at all times in each year (such a loan is refused); commercial debt forgiveness (Subdivision 245-E); private company loans (Division 7A); and applying the limits under ss 26-102 and 26-155. The deduction amounts are the s 25-25 calculation before either limit.",
    ),
    (
        "fbt",
        "Ordinary taxable employer, established type 1 and type 2 taxable values for the year ended 31 March 2026 or 2027, both at the 47% FBT rate. Uses the formulas in FBTAA s 5B(1B) and (1C) for gross-up. Excludes benefit valuation, exemptions, rebates and not-for-profit caps. Retains gross-up precision for the estimate, then presents the amounts to cents. The return_item figures follow the FBT return instead: items 14A and 14B rounded to the nearest dollar, as the ATO's own worked example does, item 15 as their sum and item 16 at 47%, before any optional rounding down to 5 cents. For 2027 the items follow the FBT return 2026 instructions until the ATO publishes the 2027 return.",
    ),
    (
        "fbt_car_statutory",
        "One car held by an ordinary taxable employer, as the only provider and for a single holding period, and used or available for an employee's private use in the FBT year ended 31 March 2026 or 2027, with no commitment to provide it made before 7.30 pm AEST on 10 May 2011, so the 20% statutory percentage applies. The operator establishes the base value under s 9(2) as the employer's guide (7.8.1) describes: cost_price holds the part the one-third reduction applies to (where the provider or an associate owned the car at the earliest holding time, its cost price to them with dealer delivery and non-business accessories fitted at acquisition; otherwise its leased car value then, which is the lessor's cost price only if the lease began then and the lessor bought the car at or about that time, and otherwise its arm's-length purchase price), and later_accessories holds non-business accessories fitted after the earliest holding time and still fitted while the car was held in the year, which it never reduces. one_third_reduction is true only from the FBT year starting after the car has been held for 4 years. Days used or available for private use, recipient's payments and whether the electric cars exemption applies are established facts. Excludes deciding whether a car benefit or exemption arises, the base value itself, arm's-length deeming, dealer demonstrator cars, the type 1 or type 2 choice, gross-up and FBT payable. Only the gross taxable value is rounded to cents, before the recipient's payment, which cannot take the taxable value below nil; an unused excess is shown and reduces no other benefit. An exempt electric car's taxable value is nil, and taxable_value_before_exemption is the notional value its reportable fringe benefits amount is worked out from, before gross-up.",
    ),
    (
        "fbt_instalment",
        "One FBT instalment of one employer for one quarter of the FBT year ended 31 March 2026 or 2027, on the activity statement (FBTAA Part VII Division 2, compilation 97, 1 July 2026). Not varying, label 6A is the ATO instalment amount at label F1, taken as the activity statement shows it: the ATO works it out from the notional tax amount, generally the tax on the most recent assessment (s 110(1)), and decides whether instalments are payable at all (s 111(2)). Varying, the employer's estimate of the year's tax (label F2, s 112(1)) is the notional tax amount from the end of the quarter (s 110(5)); the instalment is that estimate x the relevant fraction (0.25, 0.50, 0.75 or 1.00 for the quarters ending 30 June, 30 September, 31 December and 31 March, s 109), in cents 0.5 up, less the instalments for the year that fell due before it and plus the credits claimed for them (s 111(1)); a positive amount is label F3 and 6A, otherwise both are nil, and a negative amount is a credit claimable at label 6B (s 112A). Label F4 is the reason code given, one of the ATO's 4 (22, 30, 31 and 32). The operator establishes F1, the estimate and the earlier instalments and credits. Excludes: the notional tax amount, the base year and the instalment threshold, which are the ATO's; the Commissioner's own estimate under s 112(3); the general interest charge under s 112B where the estimate is less than 90% of the tax assessed; due dates (s 103); rounding a payment down to a multiple of 5 cents, which the ATO allows.",
    ),
    (
        "fbt_ev_exemption_check",
        "Whether the car benefits from one car, provided in the FBT year ended 31 March 2026 or 2027, are exempt under FBTAA s 8A (the electric cars exemption), as compiled on 1 July 2026, with the application and transitional rules of the Treasury Laws Amendment (Electric Car Discount) Act 2022. The operator establishes that the car's private use or availability is a car benefit under s 7(1), and each fact the command takes: whether the vehicle is a car (s 136(1)), a battery electric, hydrogen fuel cell electric or plug-in hybrid electric vehicle (s 8A(3) to (5)) or another, whether the benefit is for a current employee, the earliest day anyone both held and used the car, the financial year of its first retail sale and its luxury car tax value then (LCT Act s 5-20), whether luxury car tax became payable on any other supply or importation of it, and for a plug-in hybrid the facts of Schedule 2 item 5(2). Each condition is shown as met or not, and the benefit is exempt only when all are met: a car (s 136(1)); a current employee (s 8A(1)(a)); a battery electric or hydrogen fuel cell electric vehicle (s 8A(1)(b) and (2)), or a plug-in hybrid electric vehicle whose benefit falls under Schedule 2 item 5(2): a commitment made before 1 April 2025 for a period that began before then and includes the benefit, no new commitment from 1 April 2025, and an exempt car benefit for the car before 1 April 2025; the car first both held and used on or after 1 July 2022 (Schedule 1 item 7(2)); no luxury car tax at the first retail sale, its luxury car tax value not exceeding the fuel-efficient car threshold the ATO publishes for that financial year (LCT Act s 25-1(1) and (4); $91,387 for 2024-25 and 2025-26 and $91,661 for 2026-27); and none on any other supply or importation (s 8A(1)(c)). Gives electric_car_exempt for fbt-car-statutory and fbt-car-operating-cost, and what the exemption means for reporting and car expenses (ss 53(1) and 135P(3)). Refuses a first retail sale in a financial year the ATO's table does not hold (before 2012-13 or after 2026-27), a car first held and used or first sold after the FBT year ends, plug-in hybrid facts for another vehicle, and an exempt plug-in hybrid benefit before 1 April 2025 for a car first held and used later. Excludes deciding whether a car benefit arises, the vehicle's type, the dates, the luxury car tax value and the commitments; a car that was not a fuel-efficient car under LCT Act s 25-1(4) as it read at its first retail sale (whose threshold is the other vehicles' one); a car not first sold in a retail sale in Australia, such as one imported by its user; the taxable value, which fbt-car-statutory and fbt-car-operating-cost work out; and FBT years from 1 April 2027, for which the government announced on 5 May 2026 a change that was not law when this was checked.",
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
        "beneficiary_tax_offset",
        "One individual's beneficiary tax offset (ITAA 1936 s 160AAA(3)) for an income year from 2020-21 to 2026-27, from the rebatable benefit included in assessable income as the operator establishes it: a benefit under Part 2.11, 2.11A, 2.12, 2.15 or 2.23B of the Social Security Act 1991, parenting payment (partnered) not exempt under ITAA 1997 Division 52, a Commonwealth education or training payment, or another payment s 160AAA(1) and (2) list. Whether a payment is one, and the Centrelink facts behind it, are the operator's. The offset is 15% of the rebatable benefit amount (rounded down to the dollar) above $6,000, plus 15% of it above $45,000, the top of the lowest marginal rate, rounded up to the dollar (Income Tax Assessment (1936 Act) Regulations 2025 s 13 from 2026-27, and the 2015 Regulation s 13 before it, whose formula is the same). A person also entitled to the seniors and pensioners tax offset gets only the greater (s 160AAA(4)), which this worksheet does not compare. Excludes a trust beneficiary whose trustee has a s 160AAAB rebate for them (s 160AAA(4A)) and applying the offset, which is non-refundable (ITAA 1997 s 63-10(1) item 15), against tax.",
    ),
    (
        "invalid_carer_tax_offset",
        "One individual's invalid and invalid carer tax offset (ITAA 1997 Subdivision 61-A) for one dependant and an income year from 2020-21 to 2025-26: a spouse, a parent or the spouse's parent, or a child, brother or sister of the person or the spouse aged 16 or over, whom the person maintained, who is paid a disability support pension or special needs disability support pension or an invalidity service pension, or who is a spouse or parent paid carer allowance or carer payment for, or wholly engaged in caring for, such a person aged 16 or over (s 61-10), meeting the residence condition (s 61-10(1)(c)); those facts, the dependant's and the person's adjusted taxable income for offsets and the days that count are the operator's. No offset where the person's adjusted taxable income, with that of a spouse who is not the dependant (scaled by days with that spouse), is more than the FTB Part B limit (s 61-20). The amount is the indexed amount the ATO publishes for the year, or for part of the year days x the ATO's daily rate, counting for a spouse only days when neither was eligible for FTB Part B or received parental leave pay (ss 61-25 and 61-40), less $1 for every $4 of the dependant's adjusted taxable income above $282 (s 61-45), and nil below $1. Run once for each dependant; the offsets add (s 61-10(5)). Refuses a dependant also maintained by another person (s 61-40(1)(b), whose order against s 61-45 the ATO's worksheets do not follow), FTB Part B at a shared-care rate (s 61-35), a different spouse during different parts of the year (s 61-20(3)), and 2026-27, whose indexed amounts were unpublished on 22 November 2026. Excludes 2 or more spouses at once (s 61-15) and applying the offset, which is non-refundable, against tax.",
    ),
    (
        "zone_tax_offset",
        "One individual's zone tax offset (ITAA 1936 s 79A) for a usual place of residence in one area, a special area in Zone A or B, Zone A or Zone B, or overseas forces tax offset (s 79B) for a Defence Force member's service at overseas localities, for an income year from 2020-21 to 2026-27. The area is the operator's, from ITAA 1936 Schedule 2 and s 79A(3D) to (3F) and the ATO's Australian zone list, which is not exhaustive; the overseas localities are the Minister's declarations. A resident of the area is one whose usual place of residence was there for more than one-half of the year, or less with days carried from the immediately preceding year or a relevant preceding year under s 79A(3B)(d) or (e), which the operator establishes the person was not a resident of the area in; the 2023-24 year has 366 days, so 183 days is refused, as the ATO says 183 days or more. The amount is the fixed amount ($1,173 special area, $338 Zone A and overseas forces, $57 Zone B) plus 50% (20% for Zone B) of the relevant rebate amount: the invalid and invalid carer tax offset as given, the dependant notional tax offsets ($376 for each student and the oldest non-student child, $282 for each other child, apportioned by days and reduced by the dependant's adjusted taxable income over $282) and the sole parent notional tax offset ($1,607, or by days), as ITAA 1997 Division 961 and the ATO's T4 worksheets give them. The zone offset is reduced by the remote area allowance (s 79A(2A)); overseas forces service of not more than one-half of the year gives the amount x days / 183, the ATO's method for the Commissioner's amount (s 79B(2)(b)). Cents are dropped. Excludes residence in more than one zone or special area, and a zone and an overseas locality together, in one run (the Commissioner's amount under s 79A(2)(f)); a person who died in the year (s 79A(3B)(c)); a dependant also maintained by another person; United Nations armed force service (s 23AB); choosing between the zone and overseas forces offsets, of which only one is allowed (s 79B(4) and (4A)); and applying the offset, which is non-refundable, against tax.",
    ),
    (
        "net_medical_expenses_tax_offset",
        "One individual's net medical expenses tax offset (ITAA 1936 ss 159P and 159Q, compilation 161) for 2016-17, 2017-18 or 2018-19, the last year it ran (s 159P(1A)): expenses for disability aids, attendant care for a person with a disability or aged care from an approved provider (s 159P(1B)), paid in the year for the person or a resident dependant, less refunds from a government, public authority, society, association or fund (s 159P(1)), as the operator establishes them. The offset is 20% of the net expenses over the lower phase-in limit, or 10% over the higher limit where the adjusted taxable income for rebates (with the spouse's for a person married on 30 June, or at death for a person who died in the year, s 159Q(2)) exceeds the threshold: $90,000 for a single person, or for a person so married or with a dependant on any day $180,000 plus $1,500 for each dependent child after the first (s 159Q(1)), as the ATO's return instructions publish them. Excludes a trustee's offset (s 159P(3)), 2015-16 and earlier years, and applying the offset, which is non-refundable, against tax.",
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
        "aml_cdd_review",
        "Deterministic Australian AML/CTF Tranche 2 customer due diligence (CDD) checks for 2026-27 (designated services provided at or through a permanent establishment in Australia, AML/CTF Act 2006 s 30(2)), using zero personal data. question country_risk applies AUSTRAC's Basel AML Index country-risk bands (0-5 low, 5.01-6 medium, 6.01-10.00 high) and FATF grey/blacklist and Australian country-sanctions high-risk overrides, noting that statutory enhanced CDD under s 32(d) applies only where a covered person is physically present in or formed in a FATF Call-for-Action jurisdiction. question beneficial_ownership applies AML/CTF Act 2006 ss 5, 11 and 28(2)(d) and AML/CTF Rules 2025 ss 6-7(1)-(1C) and 6-8 using exact scaled-integer arithmetic across indirect ownership chains (25% threshold) and control limbs. question cdd_review applies only to a customer in an ongoing business relationship (s 30(2)(b)-(c)), excluding occasional-only transactions: it evaluates statutory CDD tier (ss 31-32 and Rules s 6-16), unconditional vs relevance-conditioned source of wealth and source of funds (Rules ss 6-21 to 6-23), mandatory immediate review triggers (s 30(2)(b)-(c) and Rules s 6-24), and periodic review dates under either AUSTRAC's Accountant Program Starter Kit defaults (12, 24 or 36 months, non-statutory guidance), a firm's custom policy interval, or statutory_only (no fixed statutory calendar interval). Excludes occasional-only customer ongoing reviews, pre-commencement customers (s 36), delayed initial CDD (s 29), foreign permanent establishments (Rules ss 6-23(3) and 6-24(2)), third-party reliance (ss 37A to 38), individual DFAT sanctions or PEP name screening, and assigning a customer's inherent ML/TF risk rating.",
    ),
    (
        "ppsr_timing",
        "What the Personal Property Securities Register's law decides without a search, for a registration, possession or attachment, or security agreement in the 2025-26 or 2026-27 income year, in ACT legal time (PPSA s 294). question registration gives a financing statement's latest end time under PPSA s 153 item 5 (consumer property or property described by serial number: the end of the day 7 years after the registration time; other collateral: no stated end time, or the end of the day 25 years after), the end time s 153(2) and (3) apply (a requested end time later than that, earlier than the registration time, or missing where one is required is taken to be the default time), and the registration fee of the Personal Property Securities (Fees) Determination 2018 s 4 items 1 to 3. question pmsi gives the time by which a purchase money security interest must be perfected by registration for priority under PPSA s 62: for inventory, by the time the grantor obtains possession of goods or the interest attaches to other property (s 62(2)(b)); otherwise before the end of 15 business days after that day (s 62(3)(b)), with a stated registration tested against it and the s 153 item 7 statement. question company_grantor gives the end of 20 business days, in the place concerned (Corporations Act s 9), after a company's security agreement came into force (s 588FL(2)(b)(ii)), and, with a critical time from 1 July 2025 to 30 June 2027, the latest of the s 588FL(2)(b) times and whether a stated registration time is after it, so that the interest vests in the company (s 588FL(4)); a registration after the critical time is left undecided, as s 588FL(2)(a) then turns on when the interest arose. Business days exclude weekends, the days the caller lists, and for the PPSA the days between Christmas Day and New Year's Day; the regulations prescribe no other day. The operator establishes that the arrangement is a security interest and, for pmsi, a purchase money security interest (PPSA s 14), the collateral's class and description, the possession or attachment time, the critical time, and that every public holiday the count spans is listed. Excludes searches and anything only the register holds, priority between interests that are not purchase money security interests, perfection by possession or control, transitional registrations, amendments and their fees, ending a registration (ss 151, 167 and 179), a foreign-law agreement's 56 day time (s 588FL(2)(b)(iii)), s 588FN's exceptions and court extensions under PPSA s 293.",
    ),
    (
        "fbt_minor_benefit_check",
        "A screen of one benefit provided to an employee or an associate in respect of the employee's employment, in the FBT year ended 31 March 2026 or 2027, against the minor benefits exemption of FBTAA s 58P. The operator establishes the benefit's category, its notional taxable value in the current year (for one benefit to one person: TR 2007/12 paras 17 and 18), whether it is under a salary sacrifice arrangement, each criterion of s 58P(1)(f) as a stated fact, and the facts of each category's exclusion. The conditions are decided and shown as met or not: an expense payment, property or residual benefit that would not be an in-house fringe benefit (s 58P(1)(c)); tax-exempt body entertainment only within s 58P(1)(d)(i) or (ii); meal entertainment or entertainment facility leasing never qualifies under the 50/50 split method (Division 9A or s 152B: TR 2007/12 para 14), and is refused without it, to be screened under its own category; no salary sacrifice arrangement (TR 2007/12 para 16); and a notional taxable value less than $300 (s 58P(1)(e)). The criteria of s 58P(1)(f) are each shown with which way the stated fact weighs, and weighed as the Commissioner's examples in TR 2007/12 weigh them: exempt where identical or similar benefits are not provided frequently, neither total is substantial and the benefit is not both regular and a reward for services (examples 1 to 3, 5 to 7 and 9 to 11), not exempt where they are provided frequently as a reward for services, their total is substantial, they are readily valued and not for an unexpected event (example 8), and otherwise the employer's judgement, shown as such. TR 2007/12 is a public ruling, the Commissioner's view of the law, and its para 11 still lists airline transport benefits, whose exclusion (s 58P(1)(b)) Act No. 88 of 2013 repealed. Excludes deciding the benefit's category, its notional taxable value, whether benefits are associated (s 58P(2)), and the criteria themselves; benefits exempt under another provision; and what an exempt entertainment benefit means for income tax and GST.",
    ),
    (
        "fhss",
        "The First Home Super Saver scheme for an individual whose first home super saver determination is made from 1 July 2018 to 31 December 2026. With contributions, an estimate of the determination (TAA 1953 Sch 1 s 138-10(1)): the operator establishes that the individual is 18 or older, has never held a legal interest in an estate in fee simple in real property in Australia, a lease of land in Australia within ITAA 1997 s 104-115(1)(b) or a company title interest in land in Australia, or has a financial hardship determination (s 138-10(2) and (2A)), and has made no earlier valid release request that was not withdrawn or revoked (s 138-10(2C)); that each contribution listed is an eligible one (s 138-35(2)): made in respect of the individual in the financial year, a voluntary employer contribution that is not mandated or a member contribution made by the individual, not required by a law or the fund's rules, not to a defined benefit interest and not to a constitutionally protected fund; and that the individual had no excess concessional or non-concessional contributions in a year counted (s 138-35(3) and (4) are not applied). Contributions are counted earliest first, a non-concessional one before a concessional one made at the same time, and in a year where the individual personally made both kinds, the year's non-concessional ones first (s 138-30(2) and (3)); a year's deducted personal contributions are taken to be its latest. At most $15,000 of a financial year's contributions and $50,000 in all count, or $30,000 in all for a determination requested before 1 July 2022, tested on contributions in full (s 138-35(1); GN 2024/1 section 3). The releasable contributions amount is the non-concessional contributions counted plus 85% of the concessional (s 138-30(1)). Associated earnings (s 138-40) are worked out for each contribution on the amount counted in the releasable contributions amount, daily at the shortfall interest charge rate (s 280-105(2): the quarter's annual rate the ATO publishes over the days in the calendar year), from 1 July 2017 for a 2017-18 contribution or otherwise the first day of its month, to the determination day, rounded down to the dollar; they are an estimate, as the Commissioner's determination is what counts. With determination, its stated components are taken as given. On release of amount_requested (the maximum unless given; never more, s 131-10(1) item 4): the assessable FHSS released amount (ITAA 1997 s 313-20), the tax withheld (TAA Regulations 2017 reg 53A: the Commissioner's estimate, which GN 2024/1 section 7 gives as the estimated marginal rate plus 2% Medicare levy less 30%, or 17% where the operator states that he cannot estimate one; one of the 2 must be given; rounded down to the dollar as GN 2024/1 examples 11 to 13 are; withholding_basis is estimate for the formula and reg_53A_1_b for the 17%), the 30% non-refundable offset (s 313-25), the FHSS tax at 20% if neither a contract nor a recontribution is notified in time (s 313-60; First Home Super Saver Tax Act 2017 s 4) and the recontribution that avoids it (s 313-50: the assessable amount less the tax withheld). Excludes the deadlines for a contract, notice and recontribution, KiwiSaver and other foreign fund transfers, excess contributions, amended determinations, a release after an action under the transitional rules of item 30 of Schedule 4 to the Treasury Laws Amendment (2023 Measures No. 3) Act 2023 for a determination made before 15 September 2024 (outside Subdivisions 313-C to 313-E, so no FHSS tax or recontribution applies: flagged for every such determination), foreign residents, a Medicare levy exemption or reduction in the withholding estimate, and the income tax payable on the assessable amount.",
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
        "notice_period",
        "The minimum notice of termination an employer must give a national system employee under the National Employment Standards (Fair Work Act 2009 s 117), for written notice given from 27 March 2021 to 30 June 2027. Continuous service runs from the first day to the end of the notice day, less the days the operator states do not count (unauthorised absence and prescribed periods, s 22(4); casual periods with the employer, s 117(4)); unpaid leave counts. s 117(3)(a): 1 week for not more than 1 year, 2 for more than 1 to 3, 3 for more than 3 to 5 and 4 for more than 5, a period of years ending on the day before its anniversary (Acts Interpretation Act 1901 s 2G); s 117(3)(b): 1 more week for an employee over 45 years old with at least 2 years of continuous service at the end of the notice day. The notice ends no earlier than the notice day plus 7 days a week. With a stated full rate of pay (s 18) and hours a week, the payment in lieu of s 117(2)(b) is the weeks times the hours times the rate, half up to the cent. The operator establishes the dates, the days not counted, whether the employee is a national system employee and which s 123 exclusion applies (a specified period, task or season unless it was to avoid the Division, serious misconduct, a casual, a training arrangement, a daily or weekly hire employee in building and construction or meat, or one the regulations prescribe); an excluded employee has no s 117 notice. Excludes longer notice under an award, enterprise agreement or contract, notice by the employee (s 118), redundancy pay, unfair dismissal, other State or Territory laws and a full rate of pay that varies from week to week. Refuses an employee aged 45 in completed years, as 'over 45 years old' may mean older than 45 years or aged above 45.",
    ),
    (
        "work_expenses",
        "An employee's deduction for one of 4 kinds of work expense in an income year from 2020-21 to 2026-27, from amounts the taxpayer incurred and bore, none reimbursed or paid for by an employer or anyone else and none covered by an allowance. home_office: working from home running expenses by the fixed rate (52c an hour before 2022-23, with a dedicated home office; PCG 2023/1 Table 1 from 2022-23, 67c to 2023-24 and 70c from 1 July 2024) plus the expenses it does not cover, the 80c shortcut (PCG 2020/3) in 2020-21 and 2021-22, and the actual cost, each with its return amount and the higher named; the operator establishes the recorded hours and the records each method needs (PCG 2023/1 paras 18 and 56 to 63) and apportions each expense's work-related amount. car_cents_per_kilometre: ITAA 1997 s 28-25, the first 5,000 business kilometres at the rate the Commissioner determines for the year. car_log_book: s 28-90, each car expense deductible outside Division 28 (decline in value included, as worked out under Division 40 with the car limit) at the business use percentage, with a log book of at least 12 continuous weeks (s 28-120(1)) kept for the year or one of the 4 before it (s 28-115(2)). Both car methods are for one car the taxpayer owned or leased (s 28-12(1)). self_education: the connection to current employment when the expense was incurred (ITAA 1997 s 8-1, TR 2024/3) and, before 2022-23, the $250 reduction of ITAA 1936 s 82A for a prescribed course of education, over the ATO's categories A to E. In 2026-27, given the assessable labour income, the other work-related deductions and residence, it adds the standard deduction for work-related expenses (ITAA 1997 s 25-130): the lesser of $1,000 and that income for a resident, reduced, but not below zero, by the covered deductions claimed, and the year's covered deductions with and without this method's claim. Excludes occupancy expenses (rent, mortgage interest, rates, house insurance) and whether the home is a place of business; the substantiation rules of Division 900 and their exceptions; reasonable allowance amounts and occupation-specific guidance; a log book notice or an added car requiring a new log book (s 28-115(3) and (4)), a car held under 12 weeks, cars taken on hire (s 28-165) and joint ownership splits; working out decline in value, the electric vehicle home charging rate and the apportionment of each expense; HELP, VET student loan and similar repayments, which are not deductible; foreign employment income; and the tax effect of the deduction.",
    ),
    (
        "ato_interest_penalties",
        "One question before remission: the general interest charge on one unpaid amount from its due day, with its payments; the shortfall interest charge on the additional income tax of one amended assessment; the failure to lodge on time penalty on one document; the false or misleading statement and not reasonably arguable penalties on one statement's or one period's parts; or the interest the Commissioner pays on one early payment, or on one refund of relevant tax overpaid as a result of a decision to which the Overpayments Act applies. The behaviour behind a shortfall, whether a position is reasonably arguable, whether a disclosure was voluntary or saved resources, the entity's size and significant global entity status are the operator's established facts. Excludes remission (the Commissioner's discretion), Division 296 GIC, scheme penalties, trust, partnership, petroleum resource rent tax and minimum tax positions, the s 284-75(3) penalty, part of a payment refunded before its due day, and other credit interest (Overpayments Act Parts IIB to IIG, IIIA and IIIAA).",
    ),
    (
        "transfer_balance",
        "One person's transfer balance account from 1 July 2017 to a day in 2017-18 to 2026-27, from its credits (income streams existing on 1 July 2017, new retirement phase income streams, LRBA repayments) and debits (commutations, s 294-85 events, payment splits, income streams leaving retirement phase): the personal transfer balance cap with proportional indexation, each day's excess, excess transfer balance earnings at the GIC rate credited daily, and excess transfer balance tax for each excess transfer balance period. An estimate before any determination: reversionary, child recipient, capped defined benefit, structured settlement and regulation events and the Commissioner's determinations, authorities and notices are refused. The person is alive on every day to the day worked: a transfer balance account ends on death (ITAA 1997 s 294-45).",
    ),
    (
        "excess_contributions",
        "One question: concessional_charge, the excess concessional contributions charge for 2013-14 to 2020-21 on the additional income tax from including excess concessional contributions, from 1 July to the day before tax under the first assessment is due; or non_concessional_release, the associated earnings, total release amount, assessable amount and 15% tax offset for an excess of non-concessional contributions for a contributions year from 2013-14 to 2025-26, released in full, to an assumed first determination date. Estimates by the method TAA 1953 Sch 1 s 97-30 sets out, which can differ by some dollars from a determination using the ATO's published Table 5 rates.",
    ),
    (
        "rental_property",
        "One residential rental property for one owner and one income year from 2020-21 to 2026-27: the owner's share of rent and other rental income (ITAA 1997 s 6-5) and of the expenses by the ATO worksheet's lines, apportioned by the days let or available for rent on commercial terms over the days held except those relating solely to renting (s 8-1; Rental properties guide 2026, 'Time-based method'), travel shown and denied (s 26-31); interest on the owner's own loans at the share of principal used for the rental (TR 2000/2); borrowing expenses through borrowing-expenses-schedule (s 25-25); capital works at 2.5% or 4% by when they began, for the days used and within the undeducted construction expenditure (Division 43); each depreciating asset's decline in value as given, nil where s 40-27 reduces it; the net rent; and, with other taxable income for 2024-25 to 2026-27, the tax at the resident scale without and with it. The operator establishes that the owner is an individual not carrying on a business of letting and not an entity excepted by ss 26-31(2), 40-27(3) or 26-102(5); the owner's legal interest; that the property is in Australia and the whole dwelling is let as residential accommodation at a market rent, with no area-based apportionment and no non-commercial letting; that the days counted as available exclude any day the land was vacant under s 26-102 and any private use; the line each expense belongs on, that it is revenue (not capital, an initial repair or an improvement) and whether it relates solely to renting; each loan's interest and principal split with no redraw or further drawing in the year; the construction expenditure from a builder's contract or a qualified estimate, whether each item is a building or a structural improvement (not earthworks s 43-20(4) excludes), when it began and was first used and whether s 43-220 applies; and each asset's decline in value and the facts s 40-27 turns on. Excludes holiday homes whose costs s 26-50 denies (refused); the 4% manner (build to rent and short-term traveller accommodation); GST on short-stay letting; s 26-155 from 2027-28; Medicare levy, the surcharge, offsets and tax losses in the effect on tax; and working out the decline in value or the construction expenditure.",
    ),
    (
        "foreign_resident_withholding",
        "Foreign resident capital gains withholding only: the amount a purchaser pays the Commissioner under TAA 1953 Sch 1 s 14-200 on acquiring taxable Australian real property or a company title interest from one or more vendors, the acquisition (contract) date from 1 July 2017 to 30 June 2027, at 12.5% with the $750,000 market value exclusion before 1 January 2025 and 15% with none from then; clearance certificates, notices of variation (varied rate and maximum price), class variations to nil and s 14-215 exclusions are stated facts. Excludes other indirect Australian real property interests, options or rights as the asset, look-through earnouts (s 14-205), and interest, dividend, royalty, managed investment trust and s 12-315 withholding.",
    ),
    (
        "sb_cgt_concessions",
        "A capital gain from CGT event A1 (a disposal, ITAA 1997 s 104-10) of a CGT asset acquired from 20 September 1985 that is not a share in a company, an interest in a trust (s 152-10(2)) or a right or interest in a partnership's income or capital (s 152-10(2C)), under a contract made in the 2025-26 or 2026-27 income year, before Act No. 49 of 2026's changes to Division 152 from the income year that includes 1 July 2027 (Schedule 1 item 82), by an individual, a company or a trust that is not a public entity (s 152-305(3)), none of them acting for a deceased estate (s 152-80). The operator establishes whether the business entity (you; the partnership for an interest in a partnership asset; or the affiliate, connected entity or partnership whose business uses a passively held asset) carried on a business in the year, its aggregated turnovers (s 328-115, with connected entities and affiliates), the net value of the CGT assets under ss 152-15 and 152-20, the total days in the test period on which the asset was an active asset (s 152-40), whether the business ceased and when, and the retirement, incapacity and significant individual facts. The CGT small business entity test is s 328-110 with $2 million for each $10 million (s 152-10(1AA)). The test period counts both its first and last days; a total that may or may not be 7.5 years (2,737 to 2,740 days) is refused, as is an asset acquired on the same date 15 years before the event. The 15-year exemption, the 50% active asset reduction, the retirement exemption and the roll-over are each shown open or not, with the conditions not met; the amounts, the choices (ss 103-25, 152-220 and 152-315), the payments a company or trust must make (s 152-325), each stakeholder's limit (s 152-320(2)), the replacement asset (s 152-420), CGT events J2, J5 and J6, and look-through earnout rights other than their extension of the replacement asset period (s 104-190(1A)(b)(ii)) are outside it.",
    ),
    (
        "private_health_insurance_rebate",
        "One individual's private health insurance tax offset or excess premium reduction for an income year from 2020-21 to 2026-27, worked out from the lines of the private health insurance statements for complying health insurance policies under which the person was an adult insured (a PHIIB, Private Health Insurance Act 2007 s 22-5(1)), every person insured being an eligible person under Health Insurance Act 1973 s 3 (ITAA 1997 s 61-205(1)(c)). Each line gives the person's share of the premiums paid in the period, divided among the adults insured (s 22-20) and without lifetime health cover loading (s 22-15(6)), the person's share of the premium reduction received, and the benefit code for the period (1 July to 31 March, or 1 April to 30 June) and the oldest insured person's age band; a premium is in the period it was paid, and the operator splits premiums by age band where the oldest person turned 65 or 70 in the period and applies s 22-25 to a person 65 or over who left the policy. The tier comes from income for surcharge purposes (ITAA 1997 s 995-1) as established, with the spouse's for a person married on 30 June, against the singles thresholds, or the family thresholds for a person married on 30 June or who maintained a dependent person who is their child or dependent sibling in the year (s 22-30). Each line is worked as a whole, as the statement and the assessment take it; give a line for each premium where the reductions within a period differed, to show the offset and the excess on each (ITAA 1997 s 61-210 and PHI Act s 282-18 work premium by premium). Excludes dependant-only policies (s 22-5(3) and (4)), a trustee's offset (ITAA 1997 s 61-205(2)), the choice to claim a spouse's offset (s 61-215) and the reallocation of the excess between spouses (PHI Act s 282-18(8) to (10)), and the lifetime health cover loading itself. Refuses premiums paid from 1 April 2027, whose percentages were not published.",
    ),
    (
        "depreciation_low_value_pool",
        "One holder's low-value pool (ITAA 1997 Subdivision 40-E) for an income year from 2020-21 to 2026-27: the decline in value at 18.75% of the taxable use percentage of the cost of low-cost assets allocated for the year and of second element amounts for the year, and at 37.5% of the previous closing pool balance and the taxable use percentage of the opening adjustable values of low-value assets allocated for the year (s 40-440), then the closing pool balance, reduced by the taxable use percentage of termination values, with any excess assessable (s 40-445). The operator establishes which assets may be and are allocated (s 40-425: low-cost assets cost under $1,000; low-value assets were written down by diminishing value to an opening adjustable value under $1,000; not assets of a small business entity under Subdivision 328-D, assets under s 40-82, R&D assets, assets for assessable labour income or assets costing $300 or less used mainly for non-business income) and each taxable use percentage (s 40-435). Excludes whether temporary full expensing applied to an asset in 2020-21 to 2022-23, the sampling rule, the capital gain or loss on the non-taxable part of a disposal and primary production apportionment.",
    ),
    (
        "depreciation_small_business_pool",
        "A small business entity's general small business pool (ITAA 1997 Subdivision 328-D) for an income year from 2020-21 to 2026-27, for an entity that chose the simplified depreciation rules for the year or continues its pool under s 328-220. The pool's s 328-210(2) amount (opening pool balance, plus the taxable purpose proportions of the adjustable values of assets allocated for the year and of cost addition amounts, less those of termination values) is written off when above zero and below the year's low pool value threshold: no limit for 2020-21 to 2022-23 (IT(TP)A 1997 s 328-181(5)), $20,000 for 2023-24 to 2025-26 (IT(TP)A s 328-180(6)(e)) and $20,000 for 2026-27 (s 328-210(1) as amended by Act No. 71 of 2026). Otherwise 30% of the opening pool balance and 15% of the year's additions and cost addition amounts are deducted (s 328-190) and the closing pool balance follows (s 328-200); a balance or amount below zero is assessable and the pool closes at zero (s 328-215). The operator establishes the entity's eligibility, which assets the instant asset write-off (s 328-180) or temporary full expensing took instead of the pool, each taxable purpose proportion and any s 328-225 adjustment, which belongs in the opening pool balance. Excludes roll-over relief (ss 328-243 to 328-247), assets excluded from simplified depreciation, the car limit, and the 57.5% backing business investment rate, refused in the one 2020-21 case it could change the result.",
    ),
    (
        "capital_works",
        "One area's capital works deduction under ITAA 1997 Division 43 for an income year from 2020-21 to 2026-27, for capital works begun on or after 16 September 1987 that qualify for a deduction for their start date and type (the operator establishes this, for example from the Rental properties guide 2026 table 1: structural improvements begun before 27 February 1992 do not). The rate is 4% for the part used in the 4% manner (s 43-145, including the 4% build to rent manner) for works begun after 26 February 1992, and 2.5% otherwise (s 43-210 steps 1 and 3; s 43-215 step 1(b) for works begun 16 September 1987 to 26 February 1992), on the construction expenditure for the part, for the days in the year it was owned or leased and used in that way over 365 (366 days are allowed in 2023-24), reduced for use only partly for producing assessable income, and limited to the undeducted construction expenditure (s 43-210 step 6, s 43-215 step 4). The operator establishes the construction expenditure attributable to their own area (a co-owner's own part), the days, the taxable use and the undeducted construction expenditure (s 43-235). Excludes works begun before 16 September 1987, works taken by s 43-220 to have begun before then, hotel and apartment buildings begun before 27 February 1992 (s 43-215 step 2), the calculation of undeducted construction expenditure, destruction (s 43-40) and splitting expenditure amongst co-owners.",
    ),
    (
        "instant_asset_write_off",
        "One asset's instant asset write-off for the income year from 2020-21 to 2026-27 in which it is first used, or installed ready for use, for a taxable purpose: for a small business entity that chose the simplified depreciation rules (ITAA 1997 s 328-180) for that year and the year it started to hold the asset, with the cost limit for its dates (IT(TP)A 1997 ss 328-180(4) and (4A) and 328-181(2), Act No. 71 of 2026 Sch 2), or for a medium sized business under s 40-82(2A). The operator establishes the entity's status and turnover for both years, the dates, the asset's Division 40 cost at the test time and its taxable purpose proportion. Excludes assets excluded from the simplified depreciation rules (s 328-175), cars whose cost the car limit reduces, an asset first used for any purpose in an earlier year, later second element amounts, balancing adjustments, split ownership and, for other than a small business entity, any asset temporary full expensing may take.",
    ),
    (
        "small_business_income_offset",
        "The small business income tax offset (ITAA 1997 Subdivision 328-F) of one individual, not as a trustee, who is not a prescribed person under s 102AC of the ITAA 1936, for an income year from 2020-21 to 2026-27: net small business income (s 328-365), each partnership or trust amount and its s 328-370 deductions, the basic income tax liability and the offsets applied before this one are the operator's established figures, and each partnership or trust is a small business entity at $5 million that is not a corporate tax entity.",
    ),
    (
        "trading_stock",
        "The change in value of the trading stock of one entity carrying on a business, for an income year from 1 July to 30 June, 2020-21 to 2026-27, under ITAA 1997 s 70-35: the excess of the value of all trading stock on hand at the end of the year over its value at the start is assessable, and the excess of the start value over the end value is deductible. The start value is the amount taken into account at the end of the last year (s 70-40). Each item on hand at the end is valued at its cost, market selling value or replacement value as elected for that item (s 70-45(1)), the least of the 3 for lowest, or below all 3 where obsolescence or other special circumstances warrant it (s 70-50), each net of input tax credits (s 70-45(1A)). A small business entity, or from 2021-22 an entity covered by s 328-285(2) ($50 million; Act No. 92 of 2020 Sch 3 item 40(8)), may choose not to account for the change when its reasonable estimate of the end value is within $5,000 of the start value (s 328-285(1)); the end value is then the start value (s 328-295). The operator establishes the entity's status, what is trading stock, each value, the elected methods, the obsolescence value and that it is reasonable, and the estimate. Excludes live stock acquired by natural increase and horse breeding stock (ss 70-55 to 70-65), stock taken for private use, disposals outside the ordinary course of business, assets becoming or ceasing to be trading stock (ss 70-30, 70-90 to 70-115), the cases in the s 70-45(2) table, primary producers' Subdivision 385, absorption costing and substituted accounting periods.",
    ),
    (
        "prepayments",
        "When one amount of expenditure incurred under an agreement in an income year from 1 July to 30 June, 2020-21 to 2026-27, is deductible under ITAA 1936 Subdivision H, where ITAA 1997 s 8-1 would otherwise allow it in full in the year incurred. Excluded expenditure (under $1,000, required by a law or court order, or under a contract of service: s 82KZL(1)) is deductible when incurred. The eligible service period runs from the later of the first day of the thing to be done and the day the expenditure is incurred to the last day, at most 10 years (s 82KZL(1)), months being read under Acts Interpretation Act 1901 s 2G(2). A thing wholly done in the expenditure year is deductible then. Where the operator states s 82KZME applies, s 82KZMF apportions the deduction by days. A small business entity or an entity covered by s 82KZM(1A) ($50 million, for expenditure incurred on or after 1 July 2020: Act No. 92 of 2020 Sch 3 item 40(5)) that has not chosen s 82KZMD, and an individual's non-business expenditure, deduct the whole amount in the expenditure year when the eligible service period is no longer than 12 months and ends by the last day of the next income year (s 82KZM(1)); otherwise s 82KZM, or for other taxpayers s 82KZMD, apportions it by days. Each year's amount is rounded to the cent half up on the running total. The operator establishes the deductible amount net of input tax credits, the taxpayer's status, the dates, the excluded expenditure limb and whether s 82KZME applies. Refuses agreements entered into on or before 11 November 1999. Excludes R&D expenditure (ss 355-205 and 355-480), Division 230 financial arrangements (s 82KZLA), Division 245, general insurers' expenditure, capital, private or domestic amounts, the STS accounting method and substituted accounting periods.",
    ),
    (
        "non_commercial_losses",
        "Whether ITAA 1997 Division 35 defers the loss from one business activity, or a group of similar ones, of one individual carrying it on alone or in partnership, for an income year from 2020-21 to 2026-27: the activity's income, deductions, asset values and earlier years and the income requirement amounts are the operator's established figures; not a bankruptcy (s 35-20), not section 40-880 expenditure (s 35-10(2A) and (2B)), and the Commissioner's discretion (s 35-55) is not decided.",
    ),
    (
        "minors_income",
        "The tax on one individual's taxable income under Division 6AA of Part III of the ITAA 1936 (minors' unearned income) for an income year from 2024-25 to 2026-27 ending on 30 June, on the individual's own return, not a trustee's under s 98 (Income Tax Rates Act 1986 Schedule 12): a full-year Australian resident, or a full-year foreign resident who is a prescribed non-resident, with ordinary taxable income only (no special income component, superannuation or employment termination remainder, or part-year tax-free threshold). Whether the individual is an excepted person, which income is excepted assessable income, the part of each trust share to which the Division applies and every deduction the Act relates to eligible income are the operator's established facts, as the Commissioner's opinion settles them. Basic income tax before offsets and levies; years before 2024-25 are outside it because the ordinary scales it uses start then.",
    ),
    (
        "smsf_tax",
        "The income tax calculation statement (item 13) of one complying self managed superannuation fund that is an Australian superannuation fund, for an income year from 2020-21 to 2026-27, from the operator's established figures: assessable contributions (ITAA 1997 Subdivision 295-C) and the no-TFN-quoted part (s 295-610); net non-arm's length income (s 295-550(1), (2), (4) and (5), less the deductions attributable to it; label U) and the general expense amounts of s 295-550(8) and (9); income from segregated current pension assets (s 295-385), other income to which the actuary's proportion applies (s 295-390), and income from segregated non-current assets; the actuary's proportion; deductions, already apportioned for exempt income, and the part attributable to contributions; the tax offsets, credits, PAYG instalments and s 102AAM interest charge; and whether the fund was newly registered or wound up in the year. It works out exempt current pension income (the proportion's product rounded down to whole dollars), taxable income, the non-arm's length component (s 295-545(2A)) taxed at 45% and the low tax component at 15% (Income Tax Rates Act 1986 s 26(1)), the 32% extra tax on no-TFN-quoted contributions (s 29), gross tax, the offsets in the return's order, the supervisory levy lines L, M and N ($259 a year, Supervisory Levy Imposition Regulations 2018 s 6) and the amount due or refundable. Excludes non-complying and foreign funds, a change of tax status (ss 295-320 and 295-325), funds other than SMSFs, the actuary's certificate and every judgement it rests on (which assets are segregated, the s 295-385(9) choice, the disregarded small fund assets test), whether income is non-arm's length (including under a limited recourse borrowing arrangement), deduction apportionment (TR 93/17), tax losses (Division 36) and capital losses, and the offsets' own rules.",
    ),
    (
        "ess_discount",
        "What ITAA 1997 Division 83A includes in one individual's assessable income for an income year from 2020-21 to 2026-27 for employee share scheme interests (shares, stapled securities and rights to acquire them) acquired from 1 July 2015 in relation to employment in Australia, by an individual who was an Australian resident, not a temporary resident, throughout: the discount, the start-up concession (s 83A-33), the upfront inclusion (s 83A-25) with the $1,000 reduction and its income test (s 83A-35), deferral (s 83A-105) to the taxing point (ss 83A-115 and 83A-120, with cessation of employment before 1 July 2022) and the amount then (s 83A-110), the value of an unlisted right under the Income Tax Assessment (1997 Act) Regulations 2021 on a day from 1 July 2021 where chosen, and the s 83A-310 exclusion for a forfeited interest. Market values (including of unlisted shares), consideration, cost bases, dates and the scheme and personal conditions are the operator's established facts. Excludes a share acquired by exercising an ESS right (s 83A-20(2): list the right), takeovers and restructures (s 83A-130), indeterminate rights (s 83A-340), the employer's deduction (Subdivision 83A-D), CGT on a later disposal and the amendment itself.",
    ),
    (
        "first_home_owner_grant",
        "The first home owner grant in one of the 8 states and territories for a new home bought or built under a contract made from 1 July 2025 to 22 November 2026: the amount on the contract date, the value cap and the lesser-of-consideration rule each Act sets, from 4 eligibility facts the operator states and does not have tested; no grant for an established home except a Northern Territory contract from 1 July to 30 September 2025. Owner-builders, substantially renovated homes, exemptions, the Commissioner's discretions, transfer duty and other home buyer schemes are outside it.",
    ),
    (
        "psi_test",
        "Whether the personal services income rules (ITAA 1997 Part 2-42) apply to one individual's personal services income for an income year from 2020-21 to 2026-27, from the operator's answers: whether the income is mainly a reward for personal efforts or skills, who receives it, a determination in force, each payer with its associates and what it paid for, the principal work by others, an apprentice and the business premises; s 87-40 is not applied to an agent on 75% or more commission, Australian government agencies are entered as s 87-35 treats them, and whether the Commissioner would make a determination is not decided.",
    ),
    (
        "payg_instalment",
        "One quarterly PAYG instalment of a quarterly payer who pays 4 instalments a year, for the 2025-26 or 2026-27 income year, under Division 45 of Schedule 1 to the TAA 1953: the T7 amount, an amount varied from estimated tax, instalment income times the T2 rate, or a varied rate with any credit for earlier quarters. The figures, estimates and method are the operator's; not monthly, 2-a-year or annual instalments, a multi-rate trustee or a consolidated group; no GDP adjustment is applied (it is in the T7 amount already), and neither general interest charge on a low variation (ss 45-230 and 45-232), the right to claim a credit nor a due date is decided.",
    ),
    (
        "pcg_2021_4",
        "The ATO's risk zone under PCG 2021/4 for one individual professional practitioner's (IPP's) allocation of professional firm profits, in 2020-21 (the rates of the guideline's examples) or 2022-23 to 2026-27 (it applies from 1 July 2022, paragraph 17), for an arrangement the operator has established passes Gateways 1 and 2 (paragraphs 10 to 12). Either the operator states the income the IPP and its associated entities are entitled to from the whole of firm group, the IPP's share and the total tax paragraph 85 counts, or gives the IPP and up to 5 associates, each an Australian resident individual aged 18 or over all year, a base rate entity company or another company, with its firm income, firm-related deductions and other income less other deductions; each one's tax is then the larger of the tax on its firm income less firm-related deductions alone and its tax on all income less the tax without them (paragraph 85), on the year's resident scale or company rate, with no offset or levy (footnote 6). Factors 1 and 2, and factor 3 against the operator's commercial benchmark when chosen, are scored on Table 1 (paragraph 76) and zoned on Table 2 (paragraph 78); an IPP that returns all of it is low risk (paragraph 81). Excludes testing the gateways and whether the guideline applies (paragraph 28), full-time equivalents for a part-time IPP (paragraph 71), superannuation contributions and fringe benefits (paragraphs 72, 73 and 95 to 99), tax offsets, trustees taxed under ITAA 1936 s 98 or 99A, minors under Division 6AA, non-residents and super funds as recipients, the benchmark itself, and Part IVA (paragraphs 36 and 37).",
    ),
    (
        "government_super_amounts",
        "The super co-contribution (Superannuation (Government Co-contribution for Low Income Earners) Act 2003, the Co-contribution Act, Part 2), the low income superannuation tax offset (LISTO, Part 2A, s 12C(1) only) and the spouse contribution tax offset (ITAA 1997 Subdivision 290-D) for one individual who was an Australian resident for the whole income year, in 2020-21 to 2026-27. Total income (Co-contribution Act s 8) and adjusted taxable income (A New Tax System (Family Assistance) Act 1999 Sch 3, disregarding cls 3 and 3A) are income-tests' measures, from the same established and classified components. The operator establishes the eligible income for the 10% test (s 6(1)(b), (2) and s 8(3)), the eligible personal contributions the fund received in the year (s 7; not deducted, not a downsizer contribution), the concessional contributions for the financial year (a person with contributions or amounts Co-contribution Act s 12E(3) disregards, for constitutionally protected funds or under ITAA 1997 s 291-165(1)(c), is out of scope, as s 12C(1)(a) counts them and s 12E(2)(a) does not), whether a return is lodged, the age, temporary visa, non-concessional cap and total super balance facts, and for a spouse the contributions s 290-230 counts (made whilst spouses, both resident and not living apart, to a complying fund the spouse could contribute to, not deductible and not a Family Law Act payment) and the spouse's income components and cap and balance facts. Thresholds are the ATO's published ones (Table 25). The payable co-contribution and LISTO are increased to a multiple of 5 cents (Regulations 2022 reg 15(2), 2004 reg 24(2)). Refuses a temporary visa held in the year, as the New Zealand citizen and prescribed visa exceptions are not worked out, a balance or cap fact not established where it decides an amount, and nil total income where a contribution makes the 10% test decide. Excludes contributions not yet received by the fund, LISTO the Commissioner estimates without a return (s 12C(2)), a deceased person's annualised income, interest on late payments (ss 12, 21 and 22), the co-contribution for an earlier year, the order and timing of payment, and 2027-28, when Act No. 8 of 2026 Schedule 4 changes LISTO.",
    ),
    (
        "horticulture_shearing_withholding",
        "Schedule 2 withholding (TAA 1953 Sch 1 s 12-35) for payments made from 1 July 2024 to 30 June 2027 to an individual in the horticultural industry (any process of producing, cultivating or harvesting a horticultural crop, on the grower's property) or the shearing industry (shearer, crutcher, wool classer, cook, shed hand or presser) who has not worked for the same grower (horticulture), or the same employer (shearing, payments to 30 June 2026) or woolgrower (shearing, payments from 1 July 2026), for a continuous period of more than 6 months and, when a TFN is quoted, has given a TFN declaration claiming the tax-free threshold. 13% of the earnings for the pay period with cents ignored, or 30% for a foreign resident, rounded to the dollar; without a TFN 47%, or 45% for a foreign resident, with cents ignored. No study and training support loan amount, Medicare levy adjustment or tax offset applies. Refuses a working holiday maker (Schedule 15), a worker under the Seasonal Worker Programme, Pacific Labour Scheme or Pacific Australia Labour Mobility scheme, and a TFN-quoting worker who has not claimed the tax-free threshold (Schedule 1). Excludes a variation the Commissioner has made and 2021-22 to 2023-24, whose schedules were not read for this worksheet.",
    ),
    (
        "upward_variation_withholding",
        "Schedule 14 withholding for a payment from 1 July 2024 to 30 June 2027 covered by Schedule 1, where the payee asked in writing for an additional amount to be withheld and the payer agreed: the Schedule 1 amount for the pay period, as payg-withholding works it out from the same inputs (its scale, residency, tax offsets, Medicare levy adjustment and 53 or 27 pay options), plus the additional amount. The normal amount under any other schedule (Schedules 2, 3, 5, 9, 15 and the others) is outside it: work that out with its own command and add the additional amount. Excludes a Commissioner's variation and 2021-22 to 2023-24, for which the Schedule 1 coefficients are not held.",
    ),
    (
        "daily_casual_withholding",
        "The tax table for daily and casual workers (NAT 1024) for payments from 1 July 2024 to 30 June 2027 of salary, wages, allowances, leave loading, paid parental leave, directors' fees, office holders' pay or payments to labour-hire workers or religious practitioners, to a payee engaged on a daily or casual basis who has claimed the tax-free threshold: Schedule 1 scale 2 on 5 times the daily earnings with cents ignored plus 99 cents, rounded to the dollar, divided by 5 and rounded to the dollar, less 0.38% of any yearly tax offsets claimed, rounded to the dollar. Without a TFN, 47%, or 45% for a foreign resident, of the earnings with cents ignored. No study and training support loan amount or Medicare levy variation applies. Refuses a working holiday maker (Schedule 15), a TFN-quoting payee who has not claimed the tax-free threshold, and a foreign resident who quoted a TFN, whose weekly rates the table gives with no step from daily earnings. Excludes a payee entitled to the seniors and pensioners tax offset (Schedule 9 coefficients: seniors-pensioners-withholding), a Commissioner's variation and 2021-22 to 2023-24.",
    ),
    (
        "no_abn_withholding",
        "Withholding from a payment from 1 July 2024 to 30 June 2027 for a supply made in the course or furtherance of an enterprise the supplier carries on in Australia, when the supplier has not quoted an ABN (TAA 1953 Sch 1 s 12-190(1)): the top rate, 45% plus the 2% Medicare levy (Taxation Administration Regulations 2017 ss 4 and 38), of the payment with cents disregarded, cents in the result disregarded (s 26); nil when the payments for the supply, disregarding GST, do not exceed $75 (s 12-190(4)(b), GST Regulations 2019 s 29-80.01). The operator establishes that s 12-190(1) applies and that none of its other exceptions does: an ABN quoted on an invoice or other document by the supplier or an agent, an incorrectly quoted ABN the payer had no reason to doubt, a payment outside the payer's own enterprise in Australia, a local governing body member's supply, a wholly input taxed supply, an investment or closely held trust payment under ss 12-140 to 12-180, or an individual's written hobby or private statement the payer has no reason to doubt; also the ATO's further cases (a supplier whose income is exempt, an individual under 18 paid $350 or less a week). Excludes a Commissioner's variation and 2021-22 to 2023-24, which were not read for this worksheet.",
    ),
    (
        "entertainer_withholding",
        "Schedule 3 withholding (TAA 1953 Sch 1 s 12-35) for payments made from 1 July 2024 to 30 June 2027 to an employee who is an actor, variety artist or other entertainer, for performances: the earnings for one performance with cents ignored, times the performances in the week, plus 99 cents; Table A (tax-free threshold claimed) or Table B, rounded to the dollar; divided by the week's performances and multiplied by the day's, rounded to the dollar; less any tax offsets claimed divided by 52 and by the week's performances, rounded to the dollar. Without a TFN, 47%, or 45% for a foreign resident, of the payment for the day's performances with cents ignored. Each result is for one payment covering performances_per_day performances; give 1 when each performance is paid separately. No study and training support loan amount or Medicare levy variation applies. Refuses a working holiday maker (Schedule 15), a foreign resident who quoted a TFN (the schedule gives weekly foreign resident rates with no step from performance earnings), tax offsets without a TFN or the tax-free threshold, and tax offsets with more than one performance a day, for which the schedule gives no daily value. Excludes a promotional activity, for which the Commissioner has varied the rate to 20%, a foreign resident engaged as a contractor (foreign resident withholding), the Withholding lookup tool's amounts, a Commissioner's variation and 2021-22 to 2023-24, whose schedules were not read for this worksheet.",
    ),
    (
        "return_to_work_withholding",
        "Schedule 4 withholding (TAA 1953 Sch 1 s 12-50) for a payment made from 1 July 2024 to 30 June 2027 that is included in the individual's assessable income under ITAA 1997 s 15-3 (a return to work payment): 32% of the payment, for residents and foreign residents alike, rounded to the dollar; without a TFN, 47%, or 45% for a foreign resident, of the payment with cents ignored. Refuses a working holiday maker (Schedule 15). Excludes a Commissioner's variation and 2021-22 to 2023-24, whose schedules were not read for this worksheet.",
    ),
    (
        "annuity_withholding",
        "Schedule 6 withholding (TAA 1953 Sch 1 s 12-80(b)) for a payment made from 1 July 2024 to 30 June 2027 of an annuity bought with non-superannuation money: the payment less the deductible amount (ITAA 1936 s 27H) divided by the instalments payable in the income year, then the Schedule 1 amount on that income for the pay period, as payg-withholding works it out from the same scale and options; without a TFN (scale 4), 47%, or 45% for a foreign resident, of the annuity payment with cents ignored. The operator establishes the deductible amount, the payee's share where 2 or more people hold the annuity. Refuses a deductible amount per instalment above the payment, an income amount not in whole cents, which the schedule does not round, and the additional amount for 53 or 27 pays on scale 4, of which the no TFN rule says nothing. Excludes an annuity bought with superannuation money (Schedule 13), a payee claiming the seniors and pensioners tax offset (Schedule 9 on the income amount, seniors-pensioners-withholding), a Commissioner's variation and 2021-22 to 2023-24, for which the Schedule 1 coefficients are not held.",
    ),
    (
        "voluntary_agreement_withholding",
        "Schedule 10 withholding (TAA 1953 Sch 1 s 12-55) for a payment made from 1 July 2024 to 30 June 2027 to an individual under an arrangement involving work or services that no other provision of Division 12 covers, under a voluntary agreement: the payee's Commissioner's instalment rate when it is 20% or more or the agreement says it applies, otherwise 20%, of the payment excluding GST, rounded to the dollar. Refuses an agreement that is not in force (s 12-55(1)(c), or terminated under s 12-55(3)), a payee whose ABN is not in force or not quoted in the agreement (s 12-55(1)(d)), and an agreement applying the instalment rate with no rate given. The operator establishes that the agreement is in the approved form and covers the arrangement, and that no other provision requires withholding. Excludes a Commissioner's variation and 2021-22 to 2023-24, whose schedules were not read for this worksheet.",
    ),
    (
        "residency_tests",
        "Whether one individual (not a company, trust or fund) was a resident of Australia at some time in an income year from 2020-21 to 2026-27 (1 July to 30 June, no substituted accounting period) under each of the 4 tests in ITAA 1936 s 6(1), resident (a): resides, domicile, 183 days and the Commonwealth superannuation fund test, from the operator's own judgements on residing, domicile, places of abode, intention and fund membership, and the dates in Australia, which it counts. It does not set the dates residency began or ended, pro-rate the tax-free threshold or apply a tax treaty.",
    ),
    (
        "lodgment_required",
        "Whether one individual must lodge their own income tax return for an income year from 2020-21 to 2025-26 under that year's lodgment instrument, made under ITAA 1936 s 161: not as a trustee, partner, agent, ship's master or person controlling a non-resident's money, not a full self-assessment taxpayer and without a substituted accounting period. The residency status, the conditions in s 6(2) that need a judgement and the exceptions' facts are the operator's answers. It does not apply the separate instrument for parents liable for or entitled to child support, or say whether lodging would be worthwhile.",
    ),
    (
        "esic",
        "One investor's early stage investor tax offset under ITAA 1997 Subdivision 360-A for shares one company issued to it in an income year from 2016-17 to 2026-27, the investor being an individual or a company that is not widely held, and the investor's and the company's income years running from 1 July to 30 June. The company is an early stage innovation company immediately after the issue (s 360-40(1)) where it was incorporated in Australia or registered in the Australian Business Register within the last 3 income years, or incorporated in Australia within the last 6 with total expenses of $1 million or less across the 3 income years before the current year (from 2018-19; refused where it decides an earlier year); its total expenses in the previous income year were $1 million or less and its assessable income $200,000 or less, disregarding an Accelerating Commercialisation Grant and, from 2022-23, a s 355-450(1) amount (s 360-40(2)); it is not listed; it is not a foreign company (from 2018-19); and it has at least 100 points under s 360-45(1) from the table items the operator states apply, or meets the principles-based test as the operator states. The investor is not an affiliate of the company, the issue is not of ESS interests and the investor does not hold more than 30% (s 360-15(1)); without a disclosure exemption under s 708(8), (10) or (11) of the Corporations Act 2001 for an offer, more than $50,000 paid takes away the offset and the modified CGT treatment (s 360-20), and a case where this company's amount and the ATO's count across all companies (which an exempt offer by another ESIC lifts), or the money paid and the money receivable or a non-cash benefit, fall on different sides of $50,000 is refused. The offset is 20% of the money received or receivable and the market value of non-cash benefits for the issue (s 360-25(1); before 2018-19 the total amount paid, and a non-cash benefit is refused), reduced so that it, the other offsets for the year of the investor and its affiliates and the offsets they carry forward to the year do not exceed $200,000 (s 360-25(2)). Amounts are worked exactly and shown to the cent, half up. The modified CGT treatment (s 360-50) is reported as applying whenever the investor is entitled, however much the cap reduces the offset. The shares issued are equity interests (s 360-15(1)(b)), and every issue by the company to the investor in the year meets the same tests on one set of facts. The operator establishes every fact: the company's figures with its 100% subsidiaries, the points items, the principles-based test, affiliates, the 30% holding, disclosure exemptions and the other and carried forward offsets. Excludes members of trusts and partnerships and trustees (ss 360-15(2), (3), 360-30 and 360-35), ESVCLPs before 2018-19, substituted accounting periods, applying or carrying forward the non-refundable offset (Division 65), the CGT consequences themselves, and schemes to obtain the offset.",
    ),
    (
        "rd_tax_offset",
        "One R&D entity's tax offset under ITAA 1997 s 355-100 for an income year starting on or after 1 July 2021, from 2021-22 to 2026-27 (Act No. 92 of 2020 Sch 4 item 14), and the amount s 355-450 includes in assessable income for a clawback amount attributable to that year. The entity is a company whose corporate tax rate is 25% where Income Tax Rates Act 1986 s 23(2)(a) applies to it (a base rate entity to which ss 23(3) to (5) and 23A do not apply) and otherwise 30% (ITAA 1997 s 995-1 'corporate tax rate'). The offset is worked on the s 355-100(1) total, or under $20,000 on the research service provider and CRC expenditure only (s 355-100(2)): where aggregated turnover is under $20 million and no exempt entity control applies, the corporate tax rate plus 18.5 percentage points, refundable (item 1, s 67-30(1)); otherwise the corporate tax rate plus 8.5% of the part up to 2% of total expenses (s 355-115) and 16.5% of the part above it, non-refundable (items 2 and 3, s 355-100(1A), s 63-10(1) item 35); the part above $150 million at the corporate tax rate (s 355-100(3)). The clawback amount is the s 355-440 recoupment amount given plus the lesser of feedstock revenue and the attributable feedstock expenditure (s 355-445(2)); the adjusted offset reduces the s 355-100(1) total by it, the deduction amount is it at the offset year's corporate tax rate, and the amount included is the starting offset less both, over the present year's corporate tax rate (s 355-450(1)). Amounts are worked exactly and shown to the cent, half up. The operator establishes the notional deductions, total expenses, turnover, control and the clawback inputs. Excludes deciding whether activities are core or supporting R&D activities or registered with AusIndustry (ss 355-20 to 355-35; Industry Research and Development Act 1986 s 27A), whether the entity is an R&D entity, whether expenditure to an associate was paid (s 355-480) or was on activities overseas or under an overseas finding, the s 355-440 amount itself (including the grant cap and repayments), balancing adjustment clawbacks (ss 355-446 to 355-449) and catch up deductions (s 355-475), an offset year with an earlier clawback or catch up amount (s 355-450(2)), R&D partnerships, entities that are not companies, substituted accounting periods starting before 1 July 2021, earlier years' offsets and carrying forward a non-refundable offset (Division 65).",
    ),
    (
        "gst_creditable_purpose_change",
        "One acquisition's adjustment under GST Act Division 129 for the tax period ending on 30 June of a year from 2020-21 to 2026-27, for an entity whose tax periods are calendar months, quarters or the financial year: whether that tax period is an adjustment period (s 129-20, with the number of them by GST exclusive value and business finance, and s 129-25 after a disposal, loss or expiry), the value thresholds of s 129-10, and the increasing or decreasing adjustment of ss 129-70 and 129-75. The operator establishes the full input tax credit with any Division 19, 21, 133 and 134 adjustments (s 129-80), the intended or former and the actual applications as percentages by a method they state, disregarding changes in applying the thing to financial supplies unless the entity exceeds the financial acquisitions threshold (s 129-5(2)), a reduced credit acquisition's extent under s 129-40(3), and that no Division 130 adjustment was had for the acquisition (s 129-15). Excludes judging whether the method is fair or reasonable, substituted and concluding tax periods (ss 27-39, 27-40, 151-55 and 162-85), a Commissioner's determination of fewer adjustment periods, gifts to charities (s 129-45) and importations' GST exclusive value other than as stated.",
    ),
    (
        "gst_annual_apportionment",
        "One entity's eligibility to make an annual apportionment election (GST Act s 131-5), and one partly creditable acquisition's input tax credit with and without the election (ss 131-40 and 11-30) and the increasing adjustment of s 131-55, for a tax period in a year from 2020-21 to 2026-27. The operator establishes small business entity status for the income year (ITAA 1997 s 328-110, not because of s 328-110(4)), GST turnover for an entity that carries on no business, the elections in effect, the GST payable on the supply taking into account Division 19, 21 and 134 adjustments, and the extents of creditable purpose, non-input-taxed purpose and consideration. A GST group's representative member can elect only if each member is eligible (s 131-15). Excludes cars (s 131-50), reduced credit acquisitions (s 131-40(5)), importations (s 131-45), when the election took effect or ceased (ss 131-10 and 131-20) and the tax period the adjustment is attributable to (s 131-60), which is stated, not worked out.",
    ),
    (
        "gst_financial_acquisitions_threshold",
        "Whether an entity, or a GST group member, exceeds the financial acquisitions threshold at a time in a month in a year from 2020-21 to 2026-27 (GST Act Division 189): the current acquisitions test over the 12 months ending at the end of the month (s 189-5) and the future acquisitions test over the month and the next 11 months (s 189-10), each exceeded when the input tax credits for financial acquisitions, treated as solely for a creditable purpose, are more than $150,000 or more than 10% of the input tax credits for all acquisitions and importations of the same months. The operator establishes which acquisitions are financial acquisitions (s 189-15: relating to financial supplies other than borrowings) and the credit amounts, with a GST group's taken across every member. Excludes which supplies are financial supplies and the reduced credit acquisitions an entity over the threshold may still claim.",
    ),
    (
        "gst_going_concern",
        "Whether one supply of a going concern is GST-free under GST Act s 38-325 in a year from 2020-21 to 2026-27, condition by condition, on facts the operator states: consideration, the recipient's registration, the written agreement and its timing (GSTR 2002/5 para 182), the supply of all things necessary for the continued operation of the enterprise and the supplier carrying it on until the day of the supply. The operator establishes each fact; the command never infers whether a supply is of a going concern from facts not stated. Excludes farm land supplied for farming (Subdivision 38-O), supplies partly outside the arrangement (s 9-80), the price and the recipient's s 135-5 and s 135-10 adjustments, which are noted, not worked out.",
    ),
    (
        "gst_second_hand_goods",
        "One acquisition of second-hand goods under GST Act Subdivision 66-A in a year from 2020-21 to 2026-27: the conditions of s 66-5 and the s 195-1 definition, and the notional input tax credit of s 66-10, 1/11 of the consideration, capped above $300 at the GST on your taxable supply of the goods. The operator establishes the facts, the consideration and that GST. Excludes goods divided for re-supply (Subdivision 66-B), goods of valuable metal, goods leased before sale (GSTD 2012/6), the tax period the credit is attributable to (s 66-15), which is noted, not worked out, and the records s 66-17 requires.",
    ),
    (
        "gst_bad_debt",
        "One bad debt adjustment under GST Act Division 21 in a year from 2020-21 to 2026-27, for the supplier (ss 21-5 and 21-10) or the recipient (ss 21-15 and 21-20): 1/11 of the amount written off, overdue for 12 months or more, recovered or paid, reduced for a partly taxable supply (s 136-5) or a partly creditable acquisition (s 136-10(1)). The operator establishes the amount, that it is overdue (s 195-1), the earlier adjustment for a recovery, and the proportion or extents. Excludes supplies and acquisitions taxable or creditable at less than 1/11 of the price (Subdivision 136-B, such as the margin scheme), reduced credit acquisitions (s 136-10(2)), credits worked out under Division 131 (s 136-10(3)), a change of accounting basis (Division 159) and the tax period the adjustment is attributable to.",
    ),
    (
        "deferred_tax",
        "One Australian resident company's tax effect worksheet under AASB 112 for an income year from 2020-21 to 2026-27, the company taxed at the Income Tax Rates Act 1986 s 23(2) rates (not a life insurance company, RSA provider, PDF, non-profit company under s 23(6) or credit union under s 23(7)) on Australian income tax alone. Each item's temporary difference is its carrying amount less its tax base for an asset, or its tax base less its carrying amount for a liability (paras 5, 7 and 8): positive is taxable, negative deductible. A deferred tax liability is taken on every taxable difference (para 15) and a deferred tax asset on deductible differences and unused tax losses less those the preparer states have no deferred tax asset, as future taxable profit is not probable (paras 24, 34 to 36 and 56), each at 25% or 30% as the preparer expects the company to be a base rate entity or not when they reverse (paras 47 and 48; s 23(2), s 23AA), undiscounted (para 53). The movement is split into the rate change on opening balances (para 80(d)), the recognition change (para 80(f) and (g)), with previously unrecognised amounts used in the year shown apart (para 80(e)), and origination and reversal (para 80(c)). Current tax is taxable income at the year's s 23(2) rate (25%, 26% in 2020-21, for a base rate entity; 30% otherwise) less non-refundable offsets the preparer has worked out, and the para 81(c)(i) reconciliation starts from accounting profit at that rate, with the average effective tax rate (para 86). The preparer establishes the carrying amounts, tax bases, unused tax losses, the probability judgement, the expected rates, taxable income, non-deductible expenses, non-assessable income and offsets, and leaves out of the items the differences the initial recognition exception covers (paras 15 and 24, as amended by AASB 2021-5 from 1 January 2023) and those on investments in subsidiaries, branches, associates and joint arrangements (paras 39 and 44). Excludes unused tax credits, adjustments for prior periods' current tax (para 80(b)), Pillar Two top-up taxes (para 4A), offsetting in the statement of financial position (para 74) and presentation. Refuses business combinations (paras 19 and 66 to 68), uncertain tax treatments (AASB Interpretation 23), tax-consolidated group members (AASB Interpretation 1052), tax on items outside profit or loss (paras 61A to 63), a change in the expected rate with opening balances when the year's rate is not the closing rate (the reversals in the year need a schedule) and, in 2020-21, opening balances measured at the base rate entity rate.",
    ),
    (
        "long_service_leave",
        "One employee's long service leave under the general long service leave Act of one of New South Wales, Victoria, Queensland, Western Australia, South Australia, Tasmania, the Australian Capital Territory and the Northern Territory, at a last day from 1 July 2020 to 30 June 2027 and not before the Act's text as read (New South Wales 25 March 2021, Victoria 1 July 2021, Western Australia 20 June 2022): the weeks an employee still employed can take, or the weeks paid when employment ends for the reason given, less leave already taken or paid, and their value at the weekly pay. Service runs from the first day to the end of the last day, less excluded days, in completed years, then completed months, then days, an anniversary on a missing day falling on the 1st of the next month; a part year is months over 12 plus days over 365.25, or in Queensland days over 364. New South Wales s 4(2)(a): 8 2/3 weeks for 10 years and 4 1/3 for each further 5; on ending after 10 to 15 years for any reason a proportion on 3 months for 15 years, after 15 each completed year at 2 months for 10, and after 5 to 10 years for the employer ending it other than for serious and wilful misconduct, illness, incapacity or a pressing necessity, or death, a proportion on 2 months for 10. Victoria ss 6, 9 and 10: after 7 years 1/60 of the continuous employment. Queensland s 95: 8.6667 weeks for 10 years and the proportion for a further 5 or more, and on ending after 7 years a proportion on 8.6667 weeks for 10, before 10 only for the reasons in s 95(4). Western Australia s 8, South Australia s 5, Tasmania ss 8 and 9, the ACT ss 3, 4, 11A and 11C and the Northern Territory ss 8 and 10 as the README sets out. The operator establishes the dates, the excluded days, the leave taken, the reason and the ordinary weekly pay. Weeks are shown to 4 places, half up; the value is the exact weeks less leave taken times the pay, half up to the cent once. Refuses a fixed term ending before the full period in New South Wales, Queensland, Tasmania, the ACT and the Northern Territory, a death under 10 years in the Northern Territory, leaving without notice between 7 and 10 years in South Australia, a dismissal found unfair outside Queensland where the reason decides, leave taken in Tasmania with service from before 1 July 2012, leave taken above the exact weeks, a start before 23 June 1990 and a last day outside the dates covered. Excludes awards, agreements and portable schemes that displace the Act, Queensland's seasonal employees, mining and casual industry schemes, what counts as service or ordinary pay, leave in advance, cashing out and public holidays during leave.",
    ),
    (
        "leave_provisions",
        "One entity's annual leave and long service leave provisions under AASB 119 at a reporting date ending in a financial year from 2020-21 to 2026-27, from lines of accumulated unused leave the preparer has established. A line is short-term if the preparer expects the benefit it belongs to to be settled wholly before 12 months after the reporting date, and other long-term otherwise, its part due within 12 months included (para 8; paras 9 and 153). A short-term line's provision is its hours times the hourly rate the entity expects to pay for them times the probability of payment, undiscounted (paras 11, 13(a), 15 and 16). An other long-term line's is its hours times the hourly rate at the reporting date, grown by the stated salary growth rate for its whole years to settlement (1 to 50), times the probability of vesting or payment, and discounted at the stated discount rate for those years (paras 155, 67, 68, 70, 72 and 83 to 87). Each amount is rounded half up to the cent. A short-term line is current (AASB 101 paras 69(a) and 70); a long-term line is current when the entity has no right at the reporting date to defer settling it for 12 months (para 69(d)), and non-current otherwise. For periods beginning before 1 January 2024 that right must be unconditional, unless AASB 2020-1 and AASB 2022-6 were applied early; from 1 January 2024 (AASB 2020-1 and AASB 2022-6) it must have substance and exist at the reporting date (para 72A), whatever the entity expects (para 75A). The probability measures a line and the right to defer classifies it, so a line without a right to defer takes any probability. The preparer establishes the hours, rates, the expectation of settlement, the right to defer and every actuarial assumption: the discount rate from market yields on high quality corporate bonds (government bonds where there is no deep market, and for a not-for-profit public sector entity, Aus83.1) at the reporting date for the term (para 83), and unbiased, mutually compatible salary growth and probabilities (paras 75 to 80); none is defaulted. Excludes on-costs (payroll tax, workers compensation and superannuation on leave), the profit or loss split into service cost, interest and remeasurements (para 156), the year's movement, fractional years to settlement and AASB 18. Refuses defined benefit plans and other post-employment benefits (paras 26 to 152), plan assets and reimbursement rights (paras 113 to 119), leave other than annual and long service leave (sick, parental, jury and military service leave, long-term disability and termination benefits), and a long-term line under 1 year to settlement (1 is the nearest whole year it takes).",
    ),
    (
        "foreign_income_conversion",
        "Foreign currency amounts of one taxpayer translated into Australian dollars under ITAA 1997 s 960-50 for an income year from 2020-21 to 2026-27 that starts on 1 July: ordinary income (s 960-50(6) item 6), statutory income other than a Division 102 amount (item 7), deductions other than under s 25-35 or Division 40 (item 8 as the regulations substitute it), a depreciating asset's cost (item 2), amounts relevant to CGT (item 5) and payments no other item covers, such as foreign income tax paid (item 11), each at the rate for the day its item names or, under item 12, at an average of the rates over a period of at most 12 months containing that day (Income Tax Assessment Regulations 1997 Schedule 2 clause 1.3 for 2020-21; Income Tax Assessment (1997 Act) Regulations 2021 Schedule 2 clause 3 from 2021-22). The operator states each rate as the foreign currency equal to 1 AUD, with its source; the command neither fetches nor assumes one, and refuses a specific rate for any other day. The operator establishes each amount's kind and dates, that an average rate reasonably approximates the item's rate and comes from a source that is not an associate or the taxpayer, and that the same time of day is used for daily rates. Excludes functional currency (Subdivision 960-D), CFC attribution, branch income of an overseas permanent establishment, forex gains and losses (Division 775), bad debts (item 8A), trading stock (items 3 and 4), withholding (item 10), film and digital game amounts, spot contracts, item 11A amounts, the financial report and records consistency rules, and which country is an amount's source.",
    ),
    (
        "foreign_income_tax_offset",
        "The foreign income tax offset (ITAA 1997 Division 770) for an income year from 2020-21 to 2026-27: the foreign income tax that counts towards it (s 770-70) and its limit (s 770-75), either $1,000 without working out s 770-75(2)(b) (note 1), or the greater of $1,000 and the income tax payable less that payable on the s 770-75(4) assumptions, each disregarding tax offsets and including the Medicare levy and surcharge (the ATO's Guide to foreign income tax offset rules, steps 1 to 3), raised by tax on s 23AI and 23AK income (s 770-80). The 2 income tax figures are stated, for any taxpayer, or worked out for an individual who was an Australian resident for the whole year or part of it, not a working holiday maker or trustee, in 2024-25 to 2026-27: basic income tax at the year's resident rates on whole-dollar taxable income with no amount taxed at another rate, with the part-year tax-free threshold of the Income Tax Rates Act 1986 ss 18 and 20, plus the Medicare levy and surcharge the operator gives for each income. The operator establishes which foreign income tax counts (s 770-10, including what a treaty allows, that foreign IIR and UTPR tax does not count where s 770-10(6) applies, that it was paid, s 770-130, and not refunded, s 770-140), translated into AUD, and the disregarded income and reasonably related deductions. The offset is applied against basic income tax, then the levy and the surcharge, and the rest is lost (s 63-10(1) item 22); applying it is outside it. Excludes CFC attributed tax (s 770-135), the Joint Petroleum Development Area rule, life insurance classes, deferred non-commercial losses' adjustment, pre-commencement excess foreign income tax and amendments (s 770-190).",
    ),
    (
        "income_tests",
        "The income measures of one individual who was an Australian resident for the whole income year and alive at its end, each the person's own for the whole year, from components the operator has established and classified: income for surcharge purposes (ITAA 1997 s 995-1), Division 293 income (s 293-20(1)(a)), rebate income (ITAA 1936 s 6(1)), repayment income for study and training loans (HESA s 154-5), adjusted taxable income for offsets (ITAA 1997 s 995-1, ITAA 1936 s 6(1) and A New Tax System (Family Assistance) Act 1999 Sch 3, disregarding cls 3 and 3A) and total income for the super co-contribution (Superannuation (Government Co-contribution for Low Income Earners) Act 2003 s 8). A spouse's measures come from the spouse's own components. A dependant's adjusted taxable income for part of a year, a deceased person's annualised adjusted taxable income (Sch 3 cl 2(2)), the co-contribution's 10% eligible income test, the combined family amounts and every threshold and eligibility test the measures feed are outside it. Income for surcharge purposes follows s 995-1, which does not add exempt foreign employment income; the ATO's surcharge page adds it where taxable income is $1 or more, and the result warns when that changes the figure. Measures another agency applies are outside it: the Commonwealth Seniors Health Card income test and family assistance adjusted taxable income are Services Australia's (https://www.servicesaustralia.gov.au/income-test-for-commonwealth-seniors-health-card). Refuses part-year residents, working holiday makers and foreign residents, and components that cannot all be true together.",
    ),
    (
        "help_indexation",
        "One person's accumulated HELP debt under HESA Division 140 for 1 June 2020 to 1 June 2026 (financial years 2019-20 to 2025-26): the preceding accumulated debt, debts incurred in the last 6 months of the preceding year, voluntary and compulsory repayments and amended assessments in the period, indexation at the ATO's published factor, the 20% reduction of ss 137-19A and 140-26 for 2025, debts incurred in the first 6 months of the year and the s 140-30 rounding; and what a voluntary repayment's timing does to the result. The operator supplies the balance, debts, repayments and compulsory repayment amounts; the command never estimates them. It does not take a location-preferred HELP debtor's reduced indexation or reduction, other study and training loans (VET Student Loans Act, SSL, ABSTUDY SSL, AASL, SFSS), payment processing time or HELP balance re-credits.",
    ),
    (
        "defined_benefit_income_cap",
        "The defined benefit income cap of one person for an income year from 2017-18 to 2026-27 (ITAA 1997 s 303-4), the 50% of the excess that is assessable (s 303-2) and the untaxed element offset after its reduction (s 303-3, with ss 301-100(2) and 302-85(2)). Which benefits are defined benefit income, and which get concessional treatment, are the operator's statements in the amounts given; the person's other income, tax and the transfer balance cap are not worked out.",
    ),
    (
        "foreign_pension_upp",
        "The deductible amount of the undeducted purchase price of one foreign pension or annuity for an income year: ITAA 1936 s 27H(2) from the stated purchase price, residual capital value and relevant number (2007-08 to 2026-27), or the ATO's 2026 D11 shortcuts for a UK State Pension or a Dutch SVB pension (2025-26 only), with the amount excluded and the assessable rest. It does not hold the Life Tables, apportion a part year, translate currency or make the Commissioner's determination under s 27H(3) or (3A).",
    ),
    (
        "scholarship_exemption",
        "Whether one scholarship, bursary, educational allowance or educational assistance paid to or for a student in an income year from 2017-18 to 2026-27 is exempt under ITAA 1997 s 51-10 item 2.1A, given the student's stated facts against s 51-35(c) to (e). The Commonwealth education and training payments s 51-35(a), (b) and (f) exclude, and friendly society plans, are refused, as the ATO's tool leaves them out; a scholarship the Commonwealth funds or pays is otherwise tested like any other.",
    ),
    (
        "super_crystallisation",
        "The crystallised segment of one taxed superannuation interest just before 1 July 2007 (ITTPA 1997 s 307-225): the stated concessional, post-June 1994 invalidity and CGT exempt components and undeducted contributions, with the pre-July 83 component worked out under former ITAA 1936 s 27AA(1)(d), and the taxable component left. Interests from an untaxed source and income streams first paid before 1 July 2007 are refused; the components are the fund's records, not worked out.",
    ),
    (
        "endorsement_review",
        "One of the ATO's 3 endorsement review worksheets (income tax exempt charities, Worksheet 1 for a DGR endorsed as a whole, Worksheet 2 for a DGR endorsed for a fund, authority or institution) from the entity's own yes or no answers, each tied to the provision it tests as read on 24 November 2026, with what each 'No' means and what to do. It checks no register; use dgr-check for a recipient's listed DGR status.",
    ),
];

pub const SOURCES: &[(&str, &str)] = &[
    (
        "vic_property_taxes",
        "https://www.legislation.vic.gov.au/in-force/acts/windfall-gains-tax-act-2021/010",
    ),
    (
        "motor_vehicle_duty",
        "https://www.legislation.qld.gov.au/view/whole/html/inforce/current/act-2001-071",
    ),
    (
        "insurance_duty",
        "https://www.legislation.qld.gov.au/view/whole/html/inforce/current/act-2001-071",
    ),
    (
        "alcohol_excise",
        "https://www.ato.gov.au/businesses-and-organisations/gst-excise-and-indirect-taxes/excise-on-alcohol/excise-duty-rates-for-alcohol",
    ),
    (
        "asic_fees",
        "https://www.asic.gov.au/for-business-and-companies/forms-and-fees/all-fees/fees-for-commonly-lodged-documents",
    ),
    (
        "fuel_tax_credits",
        "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/incentives-and-concessions/fuel-schemes/fuel-tax-credits-business/rates-business",
    ),
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
        "cgt_parcels",
        "https://www.legislation.gov.au/C2004A05138/2026-05-21/2026-05-21/text/original/epub/OEBPS/document_3/document_3.html",
    ),
    (
        "cgt_cost_base_adjustments",
        "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/104-135",
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
        "foreign_purchaser_surcharge",
        "https://www.revenue.nsw.gov.au/taxes-duties-levies-royalties/transfer-duty/surcharge-purchaser-duty/what-is-surcharge-purchaser-duty",
    ),
    (
        "dasp_tax",
        "https://www.legislation.gov.au/C2007A00013/latest/text",
    ),
    (
        "super_benefit_tax",
        "https://www.ato.gov.au/tax-rates-and-codes/key-superannuation-rates-and-thresholds/payments-from-super",
    ),
    (
        "super_withholding",
        "https://www.legislation.gov.au/F2026L00716/asmade/text",
    ),
    (
        "land_tax",
        "https://www.legislation.vic.gov.au/in-force/acts/land-tax-act-2005/086",
    ),
    (
        "workers_compensation_premium",
        "https://www.worksafe.vic.gov.au/resources/workcover-insurance-premiums-order",
    ),
    (
        "payroll_tax",
        "https://www.revenue.nsw.gov.au/taxes-duties-levies-royalties/payroll-tax/lodge-and-pay-returns/thresholds-and-rates",
    ),
    (
        "transfer_duty",
        "https://www.revenue.nsw.gov.au/taxes-duties-levies-royalties/transfer-duty/understanding-transfer-duty/calculate-transfer-duty",
    ),
    (
        "gst_margin_scheme",
        "https://www.ato.gov.au/law/view/document?DocID=PAC/19990055/75-5",
    ),
    (
        "franking_account",
        "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/205-70",
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
    ("fbt_ev_exemption_check", FBTAA_PART_1),
    ("fbt_instalment", FBTAA_PART_2),
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
        "beneficiary_tax_offset",
        "https://www.legislation.gov.au/F2025L01060/asmade/text",
    ),
    (
        "invalid_carer_tax_offset",
        "https://www.ato.gov.au/forms-and-instructions/individual-supplementary-tax-return-2026-instructions/tax-offset-questions-t3-t9-supplementary-tax-return-2026/t5-invalid-and-invalid-carer-2026",
    ),
    (
        "zone_tax_offset",
        "https://www.ato.gov.au/forms-and-instructions/individual-supplementary-tax-return-2026-instructions/tax-offset-questions-t3-t9-supplementary-tax-return-2026/t4-zone-or-overseas-forces-2026",
    ),
    (
        "net_medical_expenses_tax_offset",
        "https://www.ato.gov.au/individuals-and-families/your-tax-return/instructions-to-complete-your-tax-return/paper-tax-return-instructions/2019/supplementary-tax-return/tax-offset-questions-t3-t11/t5-total-net-medical-expenses-for-disability-aids-attendant-care-or-aged-care-2019",
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
        "fhss",
        "https://www.ato.gov.au/law/view/document?DocID=PAC/19530001/SCH1-138-40",
    ),
    (
        "aml_cdd_review",
        "https://www.legislation.gov.au/C2006A00169/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_1/document_1.html",
    ),
    (
        "ppsr_timing",
        "https://www.legislation.gov.au/C2009A00130/2024-10-14/2024-10-14/text/original/epub/OEBPS/document_1/document_1.html",
    ),
    (
        "fbt_entertainment_classifier",
        "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_1/document_1.html",
    ),
    (
        "income_tests",
        "https://www.ato.gov.au/individuals-and-families/income-deductions-offsets-and-records/income-tests",
    ),
    (
        "government_super_amounts",
        "https://www.legislation.gov.au/C2004A01195/2018-07-01/2018-07-01/text/original/epub/OEBPS/document_1/document_1.html",
    ),
    (
        "ato_interest_penalties",
        "https://www.ato.gov.au/tax-rates-and-codes/general-interest-charge-rates",
    ),
    (
        "transfer_balance",
        "https://www.ato.gov.au/individuals-and-families/super-for-individuals-and-families/super/withdrawing-and-using-your-super/retirement-withdrawal-lump-sum-or-income-stream/calculating-your-personal-transfer-balance-cap",
    ),
    (
        "excess_contributions",
        "https://www.ato.gov.au/tax-rates-and-codes/key-superannuation-rates-and-thresholds/contributions-caps",
    ),
    (
        "sb_cgt_concessions",
        "https://www.legislation.gov.au/C2004A05138/2026-05-21/2026-05-21/text/original/epub/OEBPS/document_4/document_4.html",
    ),
    (
        "notice_period",
        "https://www.legislation.gov.au/C2009A00028/latest/text",
    ),
    (
        "work_expenses",
        "https://www.ato.gov.au/law/view/document?docid=COG/PCG20231/NAT/ATO/00001",
    ),
    (
        "private_health_insurance_rebate",
        "https://www.ato.gov.au/individuals-and-families/medicare-and-private-health-insurance/private-health-insurance-rebate/income-thresholds-and-rates-for-the-private-health-insurance-rebate",
    ),
    (
        "depreciation_low_value_pool",
        "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/40-440",
    ),
    (
        "depreciation_small_business_pool",
        "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/328-210",
    ),
    (
        "capital_works",
        "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/43-210",
    ),
    (
        "instant_asset_write_off",
        "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/depreciation-and-capital-expenses-and-allowances/simpler-depreciation-for-small-business/instant-asset-write-off",
    ),
    (
        "small_business_income_offset",
        "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_7/document_7.html",
    ),
    (
        "trading_stock",
        "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_3/document_3.html",
    ),
    (
        "prepayments",
        "https://www.legislation.gov.au/C1936A00027/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_1/document_1.html",
    ),
    (
        "non_commercial_losses",
        "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_1/document_1.html",
    ),
    (
        "minors_income",
        "https://www.legislation.gov.au/C1936A00027/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_2/document_2.html",
    ),
    (
        "ess_discount",
        "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_3/document_3.html",
    ),
    (
        "first_home_owner_grant",
        "https://www.revenue.nsw.gov.au/grants-schemes/first-home-owner-new-homes-grant",
    ),
    (
        "psi_test",
        "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_3/document_3.html",
    ),
    (
        "payg_instalment",
        "https://www.legislation.gov.au/C1953A00001/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_2/document_2.html",
    ),
    (
        "pcg_2021_4",
        "https://www.ato.gov.au/law/view/document?docid=COG/PCG20214/NAT/ATO/00001",
    ),
    (
        "rental_property",
        "https://www.ato.gov.au/forms-and-instructions/rental-properties-2026",
    ),
    (
        "foreign_resident_withholding",
        "https://www.legislation.gov.au/C1953A00001/latest/text",
    ),
    (
        "horticulture_shearing_withholding",
        "https://www.ato.gov.au/tax-rates-and-codes/payg-withholding-schedule-2-tax-table-for-the-horticultural-or-shearing-industry",
    ),
    (
        "upward_variation_withholding",
        "https://www.ato.gov.au/tax-rates-and-codes/sch-14-table-of-additional-amounts-to-withhold-as-a-result-of-an-agreement-to-increase-withholding",
    ),
    (
        "daily_casual_withholding",
        "https://www.ato.gov.au/tax-rates-and-codes/tax-table-daily-and-casual-workers",
    ),
    (
        "no_abn_withholding",
        "https://www.ato.gov.au/law/view/document?docid=PAC/19530001/Sch1-12-190",
    ),
    (
        "entertainer_withholding",
        "https://www.ato.gov.au/tax-rates-and-codes/payg-withholding-schedule-3-tax-table-for-actors-variety-artists-and-other-entertainers",
    ),
    (
        "return_to_work_withholding",
        "https://www.ato.gov.au/tax-rates-and-codes/schedule-4-tax-table-for-return-to-work-payments",
    ),
    (
        "annuity_withholding",
        "https://www.ato.gov.au/tax-rates-and-codes/payg-withholding-schedule-6-tax-table-for-annuities",
    ),
    (
        "voluntary_agreement_withholding",
        "https://www.ato.gov.au/tax-rates-and-codes/payg-withholding-schedule-10-tax-table-for-payments-made-under-voluntary-agreements",
    ),
    (
        "residency_tests",
        "https://www.legislation.gov.au/C1936A00027/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_1/document_1.html",
    ),
    (
        "lodgment_required",
        "https://www.legislation.gov.au/F2026L00618/asmade/text",
    ),
    (
        "esic",
        "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/360-15",
    ),
    (
        "rd_tax_offset",
        "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/355-100",
    ),
    (
        "gst_creditable_purpose_change",
        "https://www.legislation.gov.au/C2004A00446/2026-01-01/2026-01-01/text/original/epub/OEBPS/document_2/document_2.html",
    ),
    (
        "gst_annual_apportionment",
        "https://www.legislation.gov.au/C2004A00446/2026-01-01/2026-01-01/text/original/epub/OEBPS/document_2/document_2.html",
    ),
    (
        "gst_financial_acquisitions_threshold",
        "https://www.legislation.gov.au/C2004A00446/2026-01-01/2026-01-01/text/original/epub/OEBPS/document_2/document_2.html",
    ),
    (
        "gst_going_concern",
        "https://www.legislation.gov.au/C2004A00446/2026-01-01/2026-01-01/text/original/epub/OEBPS/document_1/document_1.html",
    ),
    (
        "gst_second_hand_goods",
        "https://www.legislation.gov.au/C2004A00446/2026-01-01/2026-01-01/text/original/epub/OEBPS/document_1/document_1.html",
    ),
    (
        "gst_bad_debt",
        "https://www.legislation.gov.au/C2004A00446/2026-01-01/2026-01-01/text/original/epub/OEBPS/document_1/document_1.html",
    ),
    (
        "deferred_tax",
        "https://standards.aasb.gov.au/aasb-112-sep-2023",
    ),
    (
        "leave_provisions",
        "https://www.aasb.gov.au/admin/file/content105/c9/AASB119_08-15_COMPdec22_01-23.pdf",
    ),
    (
        "long_service_leave",
        "https://www.legislation.vic.gov.au/in-force/acts/long-service-leave-act-2018",
    ),
    (
        "foreign_income_conversion",
        "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/960-50",
    ),
    (
        "foreign_income_tax_offset",
        "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/770-75",
    ),
    (
        "smsf_tax",
        "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_6/document_6.html",
    ),
    (
        "help_indexation",
        "https://www.ato.gov.au/tax-rates-and-codes/study-and-training-loan-indexation-rates",
    ),
    (
        "defined_benefit_income_cap",
        "https://www.ato.gov.au/law/view/document?DocID=PAC/19970038/303-4",
    ),
    (
        "foreign_pension_upp",
        "https://www.ato.gov.au/law/view/document?DocID=PAC/19360027/27H",
    ),
    (
        "scholarship_exemption",
        "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/pdf/2",
    ),
    (
        "super_crystallisation",
        "https://www.ato.gov.au/law/view/document?DocID=PAC/19970038/307-225",
    ),
    (
        "endorsement_review",
        "https://www.ato.gov.au/law/view/document?DocID=PAC/19970038/30-125",
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
        "vic_property_taxes",
        &[
            "https://www.legislation.vic.gov.au/in-force/acts/land-tax-act-2005/086",
            "https://www.legislation.vic.gov.au/in-force/acts/commercial-and-industrial-property-tax-reform-act-2024/004",
            "https://www.legislation.vic.gov.au/in-force/acts/interpretation-legislation-act-1984/134",
            "https://www.legislation.vic.gov.au/as-made/acts/state-taxation-acts-and-other-acts-amendment-act-2023",
            "https://www.sro.vic.gov.au/about-us/rates-and-statistics/current-rates/vacant-residential-land-tax-current-rates",
            "https://www.sro.vic.gov.au/owning-property/vacant-residential-land-tax/understanding-vacant-residential-land-tax",
            "https://www.sro.vic.gov.au/businesses-and-organisations/commercial-and-industrial-property-tax/understanding-commercial-and-industrial-property-tax",
            "https://www.sro.vic.gov.au/owning-property/windfall-gains-tax/understanding-windfall-gains-tax",
        ],
    ),
    (
        "motor_vehicle_duty",
        &[
            "https://www.revenue.nsw.gov.au/taxes-duties-levies-royalties/motor-vehicle-duty",
            "https://www.service.nsw.gov.au/transaction/check-motor-vehicle-stamp-duty",
            "https://content.legislation.vic.gov.au/sites/default/files/2026-06/00-79aa141-authorised.pdf",
            "https://www.sro.vic.gov.au/about-us/rates-and-statistics/current-rates/motor-vehicle-duty-current-rates",
            "https://www.ato.gov.au/tax-rates-and-codes/luxury-car-tax-rate-and-thresholds",
            "https://www.qld.gov.au/transport/registration/costs/duty/rates",
            "https://www.qld.gov.au/transport/registration/costs/duty/help",
            "https://www.legislation.wa.gov.au/legislation/statutes.nsf/RedirectURL?OpenAgent&query=mrdoc_49928.pdf",
            "https://www.wa.gov.au/organisation/department-of-treasury-and-finance/vehicle-licence-duty",
            "https://www.legislation.sa.gov.au/_legislation-documents/lz/c/a/stamp-duties-act-1923/current/1923.1569.auth.pdf",
            "https://www.legislation.tas.gov.au/view/whole/html/inforce/current/act-2001-015",
            "https://www.sro.tas.gov.au/motor-vehicle-duty/rates-of-duty",
            "https://www.legislation.act.gov.au/DownloadFile/a/1999-7/current/PDF/1999-7.PDF",
            "https://www.legislation.act.gov.au/DownloadFile/di/2025-152/current/PDF/2025-152.PDF",
            "https://www.legislation.act.gov.au/DownloadFile/di/2026-154/current/PDF/2026-154.PDF",
            "https://www.revenue.act.gov.au/motor-vehicle-duty",
            "https://legislation.nt.gov.au/en/Legislation/STAMP-DUTY-ACT-1978",
            "https://treasury.nt.gov.au/dtf/territory-revenue-office/stamp-duty",
        ],
    ),
    (
        "insurance_duty",
        &[
            "https://www.revenue.nsw.gov.au/taxes-duties-levies-royalties/insurance-duty/types-of-insurance",
            "https://content.legislation.vic.gov.au/sites/default/files/2026-06/00-79aa141-authorised.pdf",
            "https://www.sro.vic.gov.au/about-us/rates-and-statistics/current-rates/insurance-policies-current-rates",
            "https://qro.qld.gov.au/duties/insurance-duty/rates/",
            "https://www.legislation.wa.gov.au/legislation/statutes.nsf/RedirectURL?OpenAgent&query=mrdoc_49928.pdf",
            "https://www.wa.gov.au/government/publications/self-assess-and-pay-insurance-duty",
            "https://www.legislation.sa.gov.au/_legislation-documents/lz/c/a/stamp-duties-act-1923/current/1923.1569.auth.pdf",
            "https://www.legislation.tas.gov.au/view/whole/html/inforce/current/act-2001-015",
            "https://www.sro.tas.gov.au/insurance-duty/rates-of-duty",
            "https://www.legislation.act.gov.au/DownloadFile/a/1999-7/current/PDF/1999-7.PDF",
            "https://legislation.nt.gov.au/en/Legislation/STAMP-DUTY-ACT-1978",
        ],
    ),
    (
        "alcohol_excise",
        &[
            "https://www.ato.gov.au/businesses-and-organisations/gst-excise-and-indirect-taxes/excise-on-alcohol/calculating-alcohol-excise-duty",
            "https://www.ato.gov.au/businesses-and-organisations/gst-excise-and-indirect-taxes/excise-on-alcohol/excise-remissions-for-excisable-alcohol/remission-scheme-for-alcohol-manufacturers",
            "https://www.ato.gov.au/law/view/document?DocID=SAV/ALCOHOL/00007",
            "https://www.legislation.gov.au/C1921A00026/2026-07-01/2026-07-01/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.legislation.gov.au/C1901A00009/2026-08-27/2026-08-27/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.legislation.gov.au/F2015L00355/2025-11-29/2025-11-29/text/original/epub/OEBPS/document_1/document_1.html",
            "https://data.gov.au/data/dataset/excise-data",
        ],
    ),
    (
        "asic_fees",
        &[
            "https://download.asic.gov.au/asic-nga/data/fees-data-current.json",
            "https://www.asic.gov.au/for-business-and-companies/forms-and-fees/all-fees/fee-indexation",
            "https://www.asic.gov.au/for-business-and-companies/forms-and-fees/late-fees",
            "https://www.legislation.gov.au/F2001B00275/latest/text",
            "https://www.legislation.gov.au/F2003B00173/latest/text",
            "https://www.legislation.gov.au/F2022L00349/latest/text",
            "https://www.legislation.gov.au/C1901A00002/latest/text",
        ],
    ),
    (
        "fuel_tax_credits",
        &[
            "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/incentives-and-concessions/fuel-schemes/fuel-tax-credits-business/rates-business/from-1-july-2026-to-30-june-2027",
            "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/incentives-and-concessions/fuel-schemes/fuel-tax-credits-business/rates-business/from-1-july-2025-to-30-june-2026",
            "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/incentives-and-concessions/fuel-schemes/fuel-tax-credits-business/rates-business/from-1-july-2024-to-30-june-2025",
            "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/incentives-and-concessions/fuel-schemes/fuel-tax-credits-business/rates-business/from-1-july-2023-to-30-june-2024",
            "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/incentives-and-concessions/fuel-schemes/fuel-tax-credits-business/rates-business/from-1-july-2022-to-30-june-2023",
            "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/incentives-and-concessions/fuel-schemes/fuel-tax-credits-business/working-out-your-fuel-tax-credits",
            "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/incentives-and-concessions/fuel-schemes/fuel-tax-credits-business/working-out-your-fuel-tax-credits/simplified-fuel-tax-credits",
            "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/incentives-and-concessions/fuel-schemes/fuel-tax-credits-business/working-out-your-fuel-tax-credits/basic-method-for-heavy-vehicles",
            "https://www.ato.gov.au/law/view/document?docid=COG/PCG20162/NAT/ATO/00001",
            "https://www.ato.gov.au/law/view/document?docid=COG/PCG20212/NAT/ATO/00001",
            "https://www.ato.gov.au/law/view/document?docid=PAC/20060072/41-5",
            "https://www.ato.gov.au/law/view/document?docid=PAC/20060072/41-20",
            "https://www.ato.gov.au/law/view/document?docid=PAC/20060072/43-5",
            "https://www.ato.gov.au/law/view/document?docid=PAC/20060072/43-10",
            "https://www.ato.gov.au/law/view/document?docid=PAC/20060072/47-5",
            "https://www.ato.gov.au/law/view/document?docid=PAC/20060072/47-10",
            "https://data.gov.au/data/dataset/excise-data",
            "https://aip.com.au/pricing/pump-prices-retail/",
        ],
    ),
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
            "https://www.ato.gov.au/individuals-and-families/investments-and-assets/property-and-land/land-vacant-land-and-subdividing/deductions-for-vacant-land",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/26-155",
            "https://www.legislation.gov.au/C1901A00002/2026-03-28/text",
        ],
    ),
    (
        "borrowing_expenses_schedule",
        &[
            "https://www.ato.gov.au/individuals-and-families/investments-and-assets/property-and-land/residential-rental-properties/rental-expenses/borrowing-expenses",
            "https://www.ato.gov.au/individuals-and-families/investments-and-assets/property-and-land/land-vacant-land-and-subdividing/deductions-for-vacant-land",
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
        "foreign_purchaser_surcharge",
        &[
            "https://classic.austlii.edu.au/au/legis/nsw/consol_act/da199793/s104u.html",
            "https://www.revenue.nsw.gov.au/taxes-duties-levies-royalties/land-tax/surcharge-land-tax/what-is-surcharge-land-tax",
            "https://www.legislation.vic.gov.au/in-force/acts/duties-act-2000/141",
            "https://www.sro.vic.gov.au/buying-property/foreign-purchasers-property/calculating-foreign-purchaser-additional-duty",
            "https://www.sro.vic.gov.au/owning-property/land-tax/absentee-owner-surcharge/absentee-individuals",
            "https://www.legislation.qld.gov.au/view/whole/pdf/inforce/current/act-2001-071",
            "https://qro.qld.gov.au/duties/investors/afad/",
            "https://qro.qld.gov.au/land-tax/calculate/absentee/",
            "https://classic.austlii.edu.au/au/legis/wa/consol_act/da200893/s205o.html",
            "https://www.wa.gov.au/system/files/2026-10/foreign-transfer-duty-fact-sheet.pdf",
            "https://www.legislation.sa.gov.au/_legislation-documents/lz/c/a/stamp-duties-act-1923/current/1923.1569.auth.pdf",
            "https://www.revenuesa.sa.gov.au/stampduty/FOS",
            "https://www.legislation.tas.gov.au/view/whole/html/inforce/current/act-2001-015",
            "https://www.sro.tas.gov.au/property-transfer-duties/foreign-investor-duty-surcharge/rates-of-surcharge",
            "https://www.legislation.act.gov.au/View/di/2026-155/current/html/2026-155.html",
            "https://www.revenue.act.gov.au/rates-and-property-charges/land-tax/foreign-ownership-surcharge-for-land-tax",
            "https://nt.gov.au/property/land/buying-and-selling-land/land-taxes",
        ],
    ),
    (
        "dasp_tax",
        &[
            "https://www.ato.gov.au/law/view/document?DocID=PAC/19970038/301-170",
            "https://www.ato.gov.au/law/view/document?DocID=PAC/19970038/301-175",
            "https://www.ato.gov.au/tax-rates-and-codes/key-superannuation-rates-and-thresholds/payments-from-super",
        ],
    ),
    (
        "super_benefit_tax",
        &[
            "https://www.ato.gov.au/law/view/document?DocID=PAC/19970038/301-20",
            "https://www.ato.gov.au/law/view/document?DocID=PAC/19970038/301-105",
            "https://www.ato.gov.au/law/view/document?DocID=PAC/19970038/302-145",
            "https://www.ato.gov.au/law/view/document?DocID=PAC/19970038/307-125",
            "https://www.ato.gov.au/law/view/document?DocID=PAC/19970038/307-345",
            "https://www.ato.gov.au/law/view/document?DocID=PAC/19970038/307-350",
            "https://www.ato.gov.au/individuals-and-families/super-for-individuals-and-families/super/withdrawing-and-using-your-super/early-access-to-super/tax-on-super-benefits",
            "https://www.legislation.gov.au/F2026L00716/asmade/text",
        ],
    ),
    (
        "super_withholding",
        &[
            "https://www.ato.gov.au/tax-rates-and-codes/paygw-schedule-12-tax-table-from-13-october-2020-to-30-june-2024",
            "https://www.legislation.gov.au/F2024L00664/asmade/text",
            "https://www.legislation.gov.au/F2025L00647/asmade/text",
            "https://www.legislation.gov.au/F2025L01016/asmade/text",
            "https://www.ato.gov.au/tax-rates-and-codes/key-superannuation-rates-and-thresholds/payments-from-super",
        ],
    ),
    (
        "land_tax",
        &[
            "https://www.revenue.nsw.gov.au/taxes-duties-levies-royalties/land-tax/understanding-land-tax/thresholds-and-rates",
            "https://www.revenue.nsw.gov.au/taxes-duties-levies-royalties/land-tax/understanding-land-tax/types-of-landowners/trusts",
            "https://www.revenue.nsw.gov.au/taxes-duties-levies-royalties/land-tax/surcharge-land-tax/what-is-surcharge-land-tax",
            "https://www.sro.vic.gov.au/about-us/rates-and-statistics/current-rates/land-tax-current-rates",
            "https://www.legislation.qld.gov.au/view/whole/html/inforce/current/act-2010-015",
            "https://qro.qld.gov.au/land-tax/calculate/individual/",
            "https://qro.qld.gov.au/land-tax/calculate/company-trust/",
            "https://qro.qld.gov.au/land-tax/calculate/absentee/",
            "https://qro.qld.gov.au/land-tax/calculate/foreign-company-trust/",
            "https://www.legislation.wa.gov.au/legislation/prod/filestore.nsf/FileURL/mrdoc_28122.htm/$FILE/Land%20Tax%20Act%202002%20-%20%5B02-d0-04%5D.html",
            "https://www.legislation.wa.gov.au/legislation/prod/filestore.nsf/FileURL/mrdoc_17548.htm/$FILE/Metropolitan%20Region%20Improvement%20Tax%20Act%201959%20-%20%5B04-a0-08%5D.html",
            "https://www.wa.gov.au/organisation/department-of-treasury-and-finance/land-tax-assessment",
            "https://www.legislation.sa.gov.au/_legislation-documents/lz/c/a/land-tax-act-1936/current/1936.2318.auth.pdf",
            "https://www.revenuesa.sa.gov.au/landtax/rates-and-thresholds",
            "https://www.legislation.tas.gov.au/view/whole/html/inforce/current/act-2000-073",
            "https://www.sro.tas.gov.au/land-tax/rates-of-land-tax",
            "https://www.sro.tas.gov.au/Documents/foreign-investor-land-tax-surcharge-guideline.pdf",
            "https://legislation.act.gov.au/DownloadFile/a/2004-4/current/PDF/2004-4.PDF",
            "https://www.legislation.act.gov.au/View/di/2026-152/current/html/2026-152.html",
            "https://nt.gov.au/property/land/buying-and-selling-land/land-taxes",
        ],
    ),
    (
        "workers_compensation_premium",
        &[
            "https://content-v2.api.worksafe.vic.gov.au/sites/default/files/2026-06/Workcover-premiums-order-34-2026-06.pdf",
            "https://www.gazette.vic.gov.au/gazette/Gazettes2026/GG2026S284.pdf",
            "https://www.icare.nsw.gov.au/workers-compensation/employers/new-policy/premium-calculation-for-small-employers",
            "https://www.icare.nsw.gov.au/workers-compensation/employers/new-policy/understanding-your-premium",
            "https://cdn.workcoverqld.com.au/api/public/content/Queensland-Government-Gazette-2026.pdf",
            "https://www.legislation.qld.gov.au/view/whole/html/inforce/current/act-2001-071",
            "https://qro.qld.gov.au/resource/da356-1/",
            "https://www.rtwsa.com/insurance/insurance-with-us/premium-calculations/your-premium-explained",
            "https://www.rtwsa.com/insurance/insurance-with-us/employer-remuneration-return/apprentice-and-trainee-incentive",
            "https://www.rtwsa.com/media/documents/Your-work-injury-insurance-booklet-June-2026_web.pdf",
            "https://worksafe.nt.gov.au/forms-and-resources/bulletins/understanding-workers-compensation-premiums-northern-territory",
            "https://worksafe.tas.gov.au/topics/compensation/workers-compensation/information-for-employers/workers-compensation-insurance",
            "https://www.act.gov.au/open/act-workers-compensation-scheme-suggested-reasonable-premium-rates",
            "https://www.workcover.wa.gov.au/resources/rates-fees-payments/",
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
        "transfer_duty",
        &[
            "https://classic.austlii.edu.au/au/legis/nsw/consol_act/da199793/",
            "https://www.revenue.nsw.gov.au/grants-schemes/assistance-scheme",
            "https://www.nsw.gov.au/housing-and-construction/buying-and-selling-property/home-buying-assistance/first-home-buyers-assistance-scheme",
            "https://www.legislation.vic.gov.au/in-force/acts/duties-act-2000/141",
            "https://www.sro.vic.gov.au/about-us/rates-and-statistics/current-rates/land-transfer-duty-non-principal-place-residence-current-rates",
            "https://www.sro.vic.gov.au/buying-property/land-transfer-stamp-duty/concessions-exemptions-and-waivers/first-home-buyers/first-home-buyer-duty-exemption-or-concession",
            "https://www.legislation.qld.gov.au/view/whole/pdf/inforce/current/act-2001-071",
            "https://qro.qld.gov.au/duties/transfer-duty/calculate/rates/",
            "https://qro.qld.gov.au/duties/transfer-duty/calculate/concession-rates/",
            "https://classic.austlii.edu.au/au/legis/wa/consol_act/da200893/sch2.html",
            "https://www.wa.gov.au/organisation/department-of-treasury-and-finance/transfer-duty-assessment",
            "https://www.wa.gov.au/government/publications/duties-fact-sheet-first-home-owner-rate",
            "https://www.legislation.sa.gov.au/_legislation-documents/lz/c/a/stamp-duties-act-1923/current/1923.1569.auth.pdf",
            "https://revenuesa.sa.gov.au/stamp-duty/rate-of-stamp-duty",
            "https://www.revenuesa.sa.gov.au/stamp-duty/first-home-buyer-relief/relief-available",
            "https://www.legislation.tas.gov.au/view/whole/html/inforce/current/act-2001-015",
            "https://www.sro.tas.gov.au/property-transfer-duties/concessions-exemptions/first-home-buyers-of-established-homes-duty-relief",
            "https://www.legislation.act.gov.au/View/di/2026-155/current/html/2026-155.html",
            "https://www.legislation.act.gov.au/View/di/2026-157/current/html/2026-157.html",
            "https://www.revenue.act.gov.au/home-buyer-assistance/home-buyer-concession-scheme/about-the-home-buyer-concession-scheme",
            "https://legislation.nt.gov.au/en/Legislation/STAMP-DUTY-ACT-1978",
            "https://nt.gov.au/property/home-owner-assistance/stamp-duty-exemption",
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
        "franking_account",
        &[
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/205-15",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/205-30",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/205-45",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/205-50",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/202-60",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/203-30",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/203-35",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/203-40",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/203-50",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/204-70",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/214-150",
            "https://www.legislation.gov.au/C2004A00982/latest/text",
            "https://www.legislation.gov.au/C2004A00981/latest/text",
            "https://www.ato.gov.au/forms-and-instructions/franking-account-tax-return-2026-instructions/how-to-get-the-franking-account-tax-return-2026",
            "https://www.ato.gov.au/forms-and-instructions/franking-account-tax-return-2026-instructions/about-the-franking-account-tax-return",
            "https://www.ato.gov.au/forms-and-instructions/franking-account-tax-return-2026-instructions/instructions-to-complete-the-franking-account-tax-return-2026/section-b-franking-deficit-tax-and-over-franking-tax",
            "https://www.ato.gov.au/forms-and-instructions/franking-account-tax-return-2026-instructions/instructions-to-complete-the-franking-account-tax-return-2026/section-c-significant-variation-in-benchmark-franking-percentage",
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
        "cgt_parcels",
        &[
            "https://www.legislation.gov.au/C2026A00049/asmade/text",
            "https://www.ato.gov.au/law/view/document?DocID=CGD/TD33/NAT/ATO/00001",
            "https://www.ato.gov.au/individuals-and-families/investments-and-assets/capital-gains-tax/shares-and-similar-investments/keeping-records-of-shares-and-units",
            "https://www.ato.gov.au/individuals-and-families/investments-and-assets/capital-gains-tax/cgt-discount",
            "https://www.ato.gov.au/forms-and-instructions/individual-tax-return-instructions-supplement-2026/income-supplementary-section/18-capital-gains-2026",
        ],
    ),
    (
        "cgt_cost_base_adjustments",
        &[
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/104-70",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/104-71",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/104-107A",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/104-107B",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/125-80",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/125-85",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/115-30",
            "https://www.ato.gov.au/law/view/document?DocID=CLR/CR201859/NAT/ATO/00001",
            "https://www.ato.gov.au/law/view/document?DocID=CLR/CR202150/NAT/ATO/00001",
            "https://www.ato.gov.au/individuals-and-families/your-tax-return/instructions-to-complete-your-tax-return/paper-tax-return-instructions/2019/supporting-information/wesfarmers-limited-demerger-of-coles-group-limited-2018",
            "https://www.ato.gov.au/law/view/print?DocID=SAV/GCGT/00004&PiT=99991231235958",
            "https://www.ato.gov.au/law/view/print?DocID=SAV/PIG/00007&PiT=99991231235958",
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
        "fbt_instalment",
        &[
            "https://www.ato.gov.au/businesses-and-organisations/preparing-lodging-and-paying/business-activity-statements-bas/fringe-benefits-tax-fbt-instalment",
            "https://www.ato.gov.au/law/view/document?DocID=SAV/FBTGEMP/00004",
        ],
    ),
    (
        "fbt_ev_exemption_check",
        &[
            FBTAA_PART_2,
            "https://www.legislation.gov.au/C2022A00086/asmade/text",
            "https://www.legislation.gov.au/C2004A00463/latest/text",
            "https://www.ato.gov.au/businesses-and-organisations/hiring-and-paying-your-workers/fringe-benefits-tax/types-of-fringe-benefits/fbt-on-cars-other-vehicles-parking-and-tolls/electric-cars-exemption",
            "https://www.ato.gov.au/businesses-and-organisations/hiring-and-paying-your-workers/fringe-benefits-tax/types-of-fringe-benefits/fbt-on-cars-other-vehicles-parking-and-tolls/fbt-on-plug-in-hybrid-electric-vehicles",
            "https://www.ato.gov.au/tax-rates-and-codes/luxury-car-tax-rate-and-thresholds",
            "https://www.ato.gov.au/about-ato/new-legislation/in-detail/businesses/electric-car-discount-more-sustainable-fbt-treatment-of-electric-cars",
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
        "beneficiary_tax_offset",
        &[
            "https://www.legislation.gov.au/F2015L01415/2022-12-09/text",
            "https://www.legislation.gov.au/C1936A00027/2026-10-01/text",
            "https://www.legislation.gov.au/C2004A03348/latest/text",
            "https://www.ato.gov.au/individuals-and-families/income-deductions-offsets-and-records/tax-offsets/beneficiary-tax-offset",
        ],
    ),
    (
        "invalid_carer_tax_offset",
        &[
            "https://www.legislation.gov.au/C2004A05138/2026-10-01/text",
            "https://www.ato.gov.au/forms-and-instructions/individual-supplementary-tax-return-2025-instructions/tax-offset-questions-t3-t9-supplementary-tax-return-2025/t5-invalid-and-invalid-carer-2025",
            "https://www.ato.gov.au/individuals-and-families/your-tax-return/instructions-to-complete-your-tax-return/paper-tax-return-instructions/2021/supplementary-tax-return/tax-offset-questions-t3-t9/t5-invalid-and-invalid-carer-2021",
            "https://www.ato.gov.au/individuals-and-families/your-tax-return/instructions-to-complete-your-tax-return/paper-tax-return-instructions/2022/supplementary-tax-return/tax-offset-questions-t3-t9/t5-invalid-and-invalid-carer-2022",
            "https://www.ato.gov.au/individuals-and-families/your-tax-return/instructions-to-complete-your-tax-return/paper-tax-return-instructions/2023/supplementary-tax-return/tax-offset-questions-t3-t9/t5-invalid-and-invalid-carer-2023",
            "https://www.ato.gov.au/individuals-and-families/your-tax-return/instructions-to-complete-your-tax-return/paper-tax-return-instructions/2024/supplementary-tax-return/tax-offset-questions-t3-t9/t5-invalid-and-invalid-carer-2024",
            "https://www.ato.gov.au/individuals-and-families/income-deductions-offsets-and-records/tax-offsets/offset-for-maintaining-an-invalid-or-invalid-carer",
        ],
    ),
    (
        "zone_tax_offset",
        &[
            "https://www.legislation.gov.au/C1936A00027/2026-10-01/text",
            "https://www.legislation.gov.au/C2004A05138/2026-10-01/text",
            "https://www.ato.gov.au/forms-and-instructions/individual-supplementary-tax-return-2025-instructions/tax-offset-questions-t3-t9-supplementary-tax-return-2025/t4-zone-or-overseas-forces-2025",
            "https://www.ato.gov.au/individuals-and-families/your-tax-return/instructions-to-complete-your-tax-return/paper-tax-return-instructions/2021/supplementary-tax-return/tax-offset-questions-t3-t9/t4-zone-or-overseas-forces-2021",
            "https://www.ato.gov.au/individuals-and-families/your-tax-return/instructions-to-complete-your-tax-return/paper-tax-return-instructions/2022/supplementary-tax-return/tax-offset-questions-t3-t9/t4-zone-or-overseas-forces-2022",
            "https://www.ato.gov.au/individuals-and-families/your-tax-return/instructions-to-complete-your-tax-return/paper-tax-return-instructions/2023/supplementary-tax-return/tax-offset-questions-t3-t9/t4-zone-or-overseas-forces-2023",
            "https://www.ato.gov.au/individuals-and-families/your-tax-return/instructions-to-complete-your-tax-return/paper-tax-return-instructions/2024/supplementary-tax-return/tax-offset-questions-t3-t9/t4-zone-or-overseas-forces-2024",
            "https://www.ato.gov.au/individuals-and-families/income-deductions-offsets-and-records/tax-offsets/zone-or-overseas-forces-tax-offsets/zone-tax-offset",
            "https://www.ato.gov.au/individuals-and-families/income-deductions-offsets-and-records/tax-offsets/zone-or-overseas-forces-tax-offsets/overseas-forces-tax-offset",
            "https://www.ato.gov.au/calculators-and-tools/tax-offsets-australian-zones",
        ],
    ),
    (
        "net_medical_expenses_tax_offset",
        &[
            "https://www.legislation.gov.au/C1936A00027/2019-04-06/text",
            "https://www.ato.gov.au/individuals-and-families/your-tax-return/instructions-to-complete-your-tax-return/paper-tax-return-instructions/2017/supplementary-tax-return/tax-offset-questions-t3-t11/t5-total-net-medical-expenses-for-disability-aids-attendant-care-or-aged-care-2017",
            "https://www.ato.gov.au/individuals-and-families/your-tax-return/instructions-to-complete-your-tax-return/paper-tax-return-instructions/2018/supplementary-tax-return/tax-offset-questions-t3-t11/t5-total-net-medical-expenses-for-disability-aids-attendant-care-or-aged-care-2018",
            "https://www.ato.gov.au/forms-and-instructions/medical-expenses-tax-offset",
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
        "aml_cdd_review",
        &[
            "https://www.legislation.gov.au/F2025L01026/2026-03-31/2026-03-31/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.austrac.gov.au/industry-and-business/obligations-and-guidance/your-amlctf-program/develop-your-amlctf-programs/step-2-identify-and-assess-your-risks",
            "https://www.austrac.gov.au/industry-and-business/obligations-and-guidance/program-starter-kits/accountant-program-starter-kit/step-2-use-your-accounting-program",
            "https://www.austrac.gov.au/industry-and-business/obligations-and-guidance/additional-guidance/determining-ownership-and-control-structures",
            "https://www.austrac.gov.au/industry-and-business/obligations-and-guidance/your-amlctf-program/customer-due-diligence/ongoing-customer-due-diligence/reviewing-and-updating-customers-mltf-risk-and-kyc-information",
        ],
    ),
    (
        "ppsr_timing",
        &[
            "https://www.legislation.gov.au/F2010L03071/2022-07-01/2022-07-01/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.legislation.gov.au/F2018L01001/asmade/2018-07-04/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.legislation.gov.au/C2004A00818/2026-09-19/2026-09-19/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.legislation.gov.au/C2004A00818/2026-09-19/2026-09-19/text/original/epub/OEBPS/document_2/document_2.html",
            "https://www.ppsr.gov.au/fees",
        ],
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
        "fhss",
        &[
            "https://www.ato.gov.au/law/view/document?DocID=PAC/19530001/SCH1-138-30",
            "https://www.ato.gov.au/law/view/document?DocID=PAC/19530001/SCH1-138-35",
            "https://www.ato.gov.au/law/view/document?DocID=PAC/19530001/SCH1-280-105",
            "https://www.ato.gov.au/law/view/document?DocID=PAC/19530001/SCH1-131-10",
            "https://www.ato.gov.au/law/view/document?DocID=PAC/19970038/313-20",
            "https://www.ato.gov.au/law/view/document?DocID=PAC/19970038/313-25",
            "https://www.legislation.gov.au/F2017L01227/latest/text",
            "https://www.legislation.gov.au/C2023A00075/asmade/2023-09-20/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.ato.gov.au/law/view/document?DocID=GDN/GDN20241/NAT/ATO/00001",
            "https://www.ato.gov.au/tax-rates-and-codes/shortfall-interest-charge-rates",
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
        "notice_period",
        &[
            "https://www.legislation.gov.au/C2021A00025/asmade/text",
            "https://www.fairwork.gov.au/ending-employment/notice-and-final-pay/dismissal-how-much-notice",
            "https://www.fairwork.gov.au/tools-and-resources/fact-sheets/minimum-workplace-entitlements/notice-of-termination-and-redundancy-pay",
            "https://www.fairwork.gov.au/ending-employment/dismissal/who-doesnt-get-notice",
        ],
    ),
    (
        "work_expenses",
        &[
            "https://www.ato.gov.au/individuals-and-families/income-deductions-offsets-and-records/deductions-you-can-claim/work-related-deductions/working-from-home-expenses/fixed-rate-method",
            "https://www.ato.gov.au/forms-and-instructions/fixed-rate-method-52-cents",
            "https://www.ato.gov.au/forms-and-instructions/shortcut-method",
            "https://www.ato.gov.au/individuals-and-families/income-deductions-offsets-and-records/deductions-you-can-claim/work-related-deductions/working-from-home-expenses/actual-cost-method",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/28-25",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/28-90",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/28-115",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/28-120",
            "https://www.ato.gov.au/individuals-and-families/income-deductions-offsets-and-records/deductions-you-can-claim/work-related-deductions/cars-transport-and-travel/motor-vehicle-and-car-expenses/expenses-for-a-car-you-own-or-lease/cents-per-kilometre-method",
            "https://www.ato.gov.au/individuals-and-families/income-deductions-offsets-and-records/deductions-you-can-claim/work-related-deductions/cars-transport-and-travel/motor-vehicle-and-car-expenses/expenses-for-a-car-you-own-or-lease/logbook-method",
            "https://www.legislation.gov.au/F2026L00785/asmade/text",
            "https://www.ato.gov.au/individuals-and-families/income-deductions-offsets-and-records/deductions-you-can-claim/work-related-deductions/education-training-and-seminars/self-education-expenses",
            "https://www.ato.gov.au/individuals-and-families/income-deductions-offsets-and-records/deductions-you-can-claim/work-related-deductions/education-training-and-seminars/self-education-expenses/self-education-reduction-in-expenses",
            "https://www.ato.gov.au/law/view/document?docid=TXR/TR989/NAT/ATO/00001",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/25-130",
            "https://www.legislation.gov.au/C2026A00049/asmade/text",
            "https://www.ato.gov.au/individuals-and-families/income-deductions-offsets-and-records/deductions-you-can-claim/work-related-deductions/standard-deduction-for-work-related-expenses",
        ],
    ),
    (
        "ato_interest_penalties",
        &[
            "https://www.ato.gov.au/tax-rates-and-codes/shortfall-interest-charge-rates",
            "https://www.ato.gov.au/tax-rates-and-codes/credit-interest-rates-and-calculation",
            "https://www.ato.gov.au/individuals-and-families/paying-the-ato/interest-and-penalties/penalties/penalty-units",
            "https://www.ato.gov.au/businesses-and-organisations/corporate-tax-measures-and-assurance/public-business-and-international/significant-global-entities/significant-global-entities-penalties",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19530001/8AAD",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19530001/8AAC",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/5-15",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19530001/Sch1-280-105",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19530001/Sch1-286-80",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19530001/Sch1-284-90",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19530001/Sch1-284-225",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19140012/4AA",
            "https://www.legislation.gov.au/C2004A02729/2024-10-14/2024-10-14/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.ato.gov.au/law/view/document?DocID=PSR/PS201119/NAT/ATO/00001",
            "https://www.ato.gov.au/law/view/document?DocID=PSR/PS201123/NAT/ATO/00001",
            "https://www.ato.gov.au/law/view/document?docid=PSR/PS20125/NAT/ATO/00001",
            "https://www.ato.gov.au/law/view/document?docid=MXR/MT20082/NAT/ATO/00001",
            "https://www.ato.gov.au/law/view/document?docid=MXR/MT20123/NAT/ATO/00001",
            "https://www.ato.gov.au/law/view/document?Docid=PSR/PS20068/NAT/ATO/00001",
        ],
    ),
    (
        "transfer_balance",
        &[
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/294-25",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/294-30",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/294-35",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/294-40",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/294-80",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/294-230",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/294-235",
            "https://www.legislation.gov.au/C2016A00080/latest/text",
            "https://www.legislation.gov.au/C2004A05128/latest/text",
            "https://www.ato.gov.au/individuals-and-families/super-for-individuals-and-families/super/withdrawing-and-using-your-super/retirement-withdrawal-lump-sum-or-income-stream/transfer-balance-account",
            "https://www.ato.gov.au/individuals-and-families/super-for-individuals-and-families/super/withdrawing-and-using-your-super/retirement-withdrawal-lump-sum-or-income-stream/excess-transfer-balance",
            "https://www.ato.gov.au/tax-rates-and-codes/general-interest-charge-rates",
        ],
    ),
    (
        "excess_contributions",
        &[
            "https://www.legislation.gov.au/C1953A00001/latest/text",
            "https://www.legislation.gov.au/C2021C00276/asmade/text",
            "https://www.legislation.gov.au/C2013A00116/latest/text",
            "https://www.legislation.gov.au/C2021A00045/asmade/text",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/292-25",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/292-30",
            "https://www.ato.gov.au/individuals-and-families/super-for-individuals-and-families/super/growing-and-keeping-track-of-your-super/caps-limits-and-tax-on-super-contributions/non-concessional-contributions-cap",
        ],
    ),
    (
        "private_health_insurance_rebate",
        &[
            "https://www.legislation.gov.au/C2007A00031/2026-09-19/2026-09-19/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.ato.gov.au/forms-and-instructions/income-thresholds-and-rates-for-the-phi-rebate-2021",
            "https://www.ato.gov.au/forms-and-instructions/income-thresholds-and-rates-for-the-phi-rebate-2022",
            "https://www.ato.gov.au/forms-and-instructions/income-thresholds-and-rates-for-the-phi-rebate-2023",
            "https://www.ato.gov.au/forms-and-instructions/income-thresholds-and-rates-for-the-phi-rebate-2024",
            "https://www.ato.gov.au/individuals-and-families/medicare-and-private-health-insurance/private-health-insurance-rebate/claiming-the-private-health-insurance-rebate",
            "https://www.ato.gov.au/individuals-and-families/medicare-and-private-health-insurance/private-health-insurance-rebate/your-private-health-insurance-statement",
        ],
    ),
    (
        "depreciation_low_value_pool",
        &[
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/40-425",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/40-435",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/40-445",
            "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/depreciation-and-capital-expenses-and-allowances/general-depreciation-rules-capital-allowances/low-value-assets-pool",
        ],
    ),
    (
        "depreciation_small_business_pool",
        &[
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/328-190",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/328-200",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/328-215",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970040/328-180",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970040/328-181",
            "https://www.legislation.gov.au/C2026A00071/asmade/text",
            "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/depreciation-and-capital-expenses-and-allowances/simpler-depreciation-for-small-business/small-business-pool-calculations",
        ],
    ),
    (
        "capital_works",
        &[
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/43-140",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/43-145",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/43-215",
            "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/depreciation-and-capital-expenses-and-allowances/capital-works-deductions",
        ],
    ),
    (
        "instant_asset_write_off",
        &[
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/328-180",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/40-82",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970040/328-180",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970040/328-181",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970040/40-160",
            "https://www.ato.gov.au/law/view/pdf/acts/20260071.pdf",
            "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/depreciation-and-capital-expenses-and-allowances/temporary-full-expensing/tfe-for-small-business-entities-using-simplified-depreciation",
        ],
    ),
    (
        "rental_property",
        &[
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/8-1",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/26-31",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/40-27",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/43-25",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/43-210",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/43-230",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/43-235",
            "https://www.legislation.gov.au/C2017A00126/asmade/text",
            "https://www.ato.gov.au/law/view/document?docid=TXR/TR20002/NAT/ATO/00001",
            "https://www.ato.gov.au/law/view/document?docid=TXR/TR9332/NAT/ATO/00001",
        ],
    ),
    (
        "foreign_resident_withholding",
        &[
            "https://www.legislation.gov.au/C2017A00057/asmade/text",
            "https://www.legislation.gov.au/C2024A00135/asmade/text",
            "https://www.legislation.gov.au/F2016L01123/asmade/text",
            "https://www.legislation.gov.au/F2026L01231/asmade/text",
            "https://www.ato.gov.au/individuals-and-families/investments-and-assets/capital-gains-tax/foreign-residents-and-capital-gains-tax/foreign-resident-capital-gains-withholding",
        ],
    ),
    (
        "horticulture_shearing_withholding",
        &[
            "https://www.legislation.gov.au/F2026L00716/asmade/text",
            "https://www.legislation.gov.au/F2025L01016/asmade/text",
            "https://www.legislation.gov.au/F2025L00647/asmade/text",
            "https://www.legislation.gov.au/F2024L00664/asmade/text",
        ],
    ),
    (
        "upward_variation_withholding",
        &[
            "https://www.legislation.gov.au/F2026L00716/asmade/text",
            "https://www.legislation.gov.au/F2025L01016/asmade/text",
            "https://www.legislation.gov.au/F2024L00664/asmade/text",
            "https://www.ato.gov.au/tax-rates-and-codes/payg-withholding-schedule-1-statement-of-formulas-for-calculating-amounts-to-be-withheld",
        ],
    ),
    (
        "ess_discount",
        &[
            "https://www.legislation.gov.au/C2004A05138/2022-07-01/2022-07-01/text/original/epub/OEBPS/document_3/document_3.html",
            "https://www.legislation.gov.au/C2004A05138/2020-07-01/2020-07-01/text/original/epub/OEBPS/document_3/document_3.html",
            "https://www.legislation.gov.au/C2022A00008/asmade/2021-04-04/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.legislation.gov.au/F2021L00206/2026-06-27/2026-06-27/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.ato.gov.au/law/view/print?DocID=REG/20210206/83A-315.08&PiT=99991231235958",
            "https://www.ato.gov.au/businesses-and-organisations/corporate-tax-measures-and-assurance/employee-share-schemes/in-detail/market-value/market-value-of-unlisted-rights-to-acquire-listed-shares-and-stapled-securities",
            "https://www.ato.gov.au/businesses-and-organisations/corporate-tax-measures-and-assurance/employee-share-schemes/employees/ess-and-your-tax/income-test-for-the-upfront-concession-1000-dollar-reduction",
            "https://www.ato.gov.au/businesses-and-organisations/corporate-tax-measures-and-assurance/employee-share-schemes/employers/types-of-ess/concessional-ess/start-up-concession-interests-acquired-after-30-june-2015",
            "https://www.ato.gov.au/businesses-and-organisations/corporate-tax-measures-and-assurance/employee-share-schemes/in-detail/restrictions-and-forfeitures/genuine-disposal-restrictions-and-deferred-taxing-points",
            "https://www.ato.gov.au/forms-and-instructions/individual-tax-return-2025-instructions/income-questions-1-12-individual-tax-return-2025/12-employee-share-schemes-2025",
        ],
    ),
    (
        "daily_casual_withholding",
        &[
            "https://www.ato.gov.au/tax-rates-and-codes/tax-table-daily-and-casual-workers-01-july-2024-to-30-june-2026",
            "https://www.legislation.gov.au/F2026L00716/asmade/text",
            "https://www.legislation.gov.au/F2025L01016/asmade/text",
            "https://www.legislation.gov.au/F2024L00664/asmade/text",
        ],
    ),
    (
        "no_abn_withholding",
        &[
            "https://www.ato.gov.au/law/view/document?docid=PAC/19530001/Sch1-15-10",
            "https://www.legislation.gov.au/F2017L01227/latest/text",
            "https://www.legislation.gov.au/F2019L00417/latest/text",
            "https://www.ato.gov.au/businesses-and-organisations/hiring-and-paying-your-workers/payg-withholding/payments-you-need-to-withhold-from/withholding-from-suppliers/payments-you-don-t-withhold-from",
            "https://www.ato.gov.au/forms-and-instructions/rental-properties-2025/rental-expenses",
        ],
    ),
    (
        "entertainer_withholding",
        &[
            "https://www.legislation.gov.au/F2026L00716/asmade/text",
            "https://www.legislation.gov.au/F2025L01016/asmade/text",
            "https://www.legislation.gov.au/F2025L00647/asmade/text",
            "https://www.legislation.gov.au/F2024L00664/asmade/text",
        ],
    ),
    (
        "return_to_work_withholding",
        &[
            "https://www.legislation.gov.au/F2026L00716/asmade/text",
            "https://www.legislation.gov.au/F2025L01016/asmade/text",
            "https://www.legislation.gov.au/F2025L00647/asmade/text",
            "https://www.legislation.gov.au/F2024L00664/asmade/text",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19530001/Sch1-12-50",
        ],
    ),
    (
        "annuity_withholding",
        &[
            "https://www.legislation.gov.au/F2026L00716/asmade/text",
            "https://www.legislation.gov.au/F2025L01016/asmade/text",
            "https://www.legislation.gov.au/F2025L00647/asmade/text",
            "https://www.legislation.gov.au/F2024L00664/asmade/text",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19530001/Sch1-12-80",
            "https://www.ato.gov.au/tax-rates-and-codes/payg-withholding-schedule-1-statement-of-formulas-for-calculating-amounts-to-be-withheld",
        ],
    ),
    (
        "voluntary_agreement_withholding",
        &[
            "https://www.legislation.gov.au/F2026L00716/asmade/text",
            "https://www.legislation.gov.au/F2025L01016/asmade/text",
            "https://www.legislation.gov.au/F2025L00647/asmade/text",
            "https://www.legislation.gov.au/F2024L00664/asmade/text",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19530001/Sch1-12-55",
        ],
    ),
    (
        "esic",
        &[
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/360-20",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/360-25",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/360-40",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/360-45",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/360-50",
            "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/incentives-and-concessions/tax-incentives-for-innovation/tax-incentives-for-early-stage-investors/calculating-the-early-stage-investor-tax-offset",
            "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/incentives-and-concessions/tax-incentives-for-innovation/tax-incentives-for-early-stage-investors/qualifying-as-an-early-stage-innovation-company",
            "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/incentives-and-concessions/tax-incentives-for-innovation/tax-incentives-for-early-stage-investors/limits-for-investors-who-dont-meet-the-sophisticated-investor-test",
        ],
    ),
    (
        "rd_tax_offset",
        &[
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/355-115",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/355-440",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/355-445",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/355-450",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19970038/67-30",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19860107/23",
            "https://www.legislation.gov.au/C2020A00092/asmade/text",
            "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/incentives-and-concessions/research-and-development-tax-incentive/clawback-of-r-d-tax-incentive-offset-and-catch-up-deductions/clawback-of-feedstock-adjustments",
            "https://www.ato.gov.au/forms-and-instructions/research-and-development-tax-incentive-schedule-2022-instructions/part-e-rd-tax-offset-calculation",
            "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/incentives-and-concessions/research-and-development-tax-incentive-and-concessions/research-and-development-tax-incentive/rates-of-r-d-tax-incentive-offset",
            "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/incentives-and-concessions/research-and-development-tax-incentive-and-concessions/research-and-development-tax-incentive/in-detail/clawback-guide-government-grants-and-reimbursements/calculate-the-claw-back-adjustment",
        ],
    ),
    (
        "sb_cgt_concessions",
        &[
            "https://www.legislation.gov.au/C2004A05138/2026-05-21/2026-05-21/text/original/epub/OEBPS/document_3/document_3.html",
            "https://www.legislation.gov.au/C2004A05138/2026-05-21/2026-05-21/text/original/epub/OEBPS/document_7/document_7.html",
            "https://www.legislation.gov.au/C2026A00049/asmade/text",
        ],
    ),
    (
        "gst_creditable_purpose_change",
        &[
            "https://www.legislation.gov.au/C2004A00446/2020-04-01/2020-04-01/text/original/epub/OEBPS/document_2/document_2.html",
            "https://www.ato.gov.au/law/view/document?docid=GST/GSTR200024/NAT/ATO/00001",
        ],
    ),
    (
        "gst_annual_apportionment",
        &[
            "https://www.legislation.gov.au/C2004A00446/2026-01-01/2026-01-01/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.legislation.gov.au/C2004A00446/2020-04-01/2020-04-01/text/original/epub/OEBPS/document_2/document_2.html",
            "https://www.legislation.gov.au/F2019L00417/latest/text",
            "https://www.ato.gov.au/businesses-and-organisations/gst-excise-and-indirect-taxes/gst/in-detail/managing-gst-in-your-business/reporting-paying-and-activity-statements/annual-private-apportionment-of-gst/eligibility",
        ],
    ),
    (
        "gst_financial_acquisitions_threshold",
        &[
            "https://www.legislation.gov.au/C2004A00446/2026-01-01/2026-01-01/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.legislation.gov.au/C2004A00446/2020-04-01/2020-04-01/text/original/epub/OEBPS/document_2/document_2.html",
            "https://www.legislation.gov.au/F2019L00417/latest/text",
            "https://www.ato.gov.au/law/view/document?docid=GST/GSTR20039/NAT/ATO/00001",
        ],
    ),
    (
        "gst_going_concern",
        &[
            "https://www.legislation.gov.au/C2004A00446/2026-01-01/2026-01-01/text/original/epub/OEBPS/document_2/document_2.html",
            "https://www.legislation.gov.au/C2004A00446/2020-04-01/2020-04-01/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.ato.gov.au/law/view/document?docid=GST/GSTR20025/NAT/ATO/00001",
        ],
    ),
    (
        "gst_second_hand_goods",
        &[
            "https://www.legislation.gov.au/C2004A00446/2026-01-01/2026-01-01/text/original/epub/OEBPS/document_2/document_2.html",
            "https://www.legislation.gov.au/C2004A00446/2020-04-01/2020-04-01/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.ato.gov.au/law/view/document?docid=GSD/GSTD20126/NAT/ATO/00001",
        ],
    ),
    (
        "gst_bad_debt",
        &[
            "https://www.legislation.gov.au/C2004A00446/2026-01-01/2026-01-01/text/original/epub/OEBPS/document_2/document_2.html",
            "https://www.legislation.gov.au/C2004A00446/2020-04-01/2020-04-01/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.legislation.gov.au/C2004A00446/2020-04-01/2020-04-01/text/original/epub/OEBPS/document_2/document_2.html",
            "https://www.ato.gov.au/law/view/document?docid=GST/GSTR20002/NAT/ATO/00001",
        ],
    ),
    (
        "small_business_income_offset",
        &[
            "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_2/document_2.html",
            "https://www.legislation.gov.au/C2004A05138/2020-07-01/2020-07-01/text/original/epub/OEBPS/document_7/document_7.html",
            "https://www.legislation.gov.au/C2004A05138/2021-07-01/2021-07-01/text/original/epub/OEBPS/document_7/document_7.html",
            "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/income-and-deductions-for-business/concessions-offsets-and-rebates/small-business-income-tax-offset",
        ],
    ),
    (
        "trading_stock",
        &[
            "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_7/document_7.html",
            "https://www.legislation.gov.au/C2004A05138/2020-07-01/2020-07-01/text/original/epub/OEBPS/document_7/document_7.html",
            "https://www.legislation.gov.au/C2020A00092/asmade/2020-10-14/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/income-and-deductions-for-business/accounting-for-trading-stock/simplified-trading-stock-rules",
            "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/income-and-deductions-for-business/accounting-for-trading-stock/valuing-trading-stock",
        ],
    ),
    (
        "prepayments",
        &[
            "https://www.legislation.gov.au/C1936A00027/2020-07-01/2020-07-01/text/original/epub/OEBPS/document_2/document_2.html",
            "https://www.legislation.gov.au/C2020A00092/asmade/2020-10-14/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.legislation.gov.au/C1901A00002/2026-03-28/2026-03-28/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.ato.gov.au/api/public/content/0-3a8a1b7d-086d-40b2-abfe-55615b4f3528",
            "https://www.ato.gov.au/api/public/content/0-347c6a18-4343-461e-b4f3-9f12a5ddb904",
        ],
    ),
    (
        "non_commercial_losses",
        &[
            "https://www.legislation.gov.au/C2004A05138/2020-07-01/2020-07-01/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/losses/non-commercial-losses/offset-or-defer-the-loss-individuals-or-sole-traders/four-tests",
            "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/losses/non-commercial-losses/offset-or-defer-the-loss-individuals-or-sole-traders/the-income-requirement",
            "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/losses/non-commercial-losses/offset-or-defer-the-loss-individuals-or-sole-traders/excepted-business-activities",
            "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/losses/non-commercial-losses/how-to-defer-your-losses",
        ],
    ),
    (
        "minors_income",
        &[
            "https://www.legislation.gov.au/C2004A03348/2026-07-01/2026-07-01/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_2/document_2.html",
            "https://www.legislation.gov.au/C1936A00027/2020-07-01/2020-07-01/text/original/epub/OEBPS/document_2/document_2.html",
            "https://www.legislation.gov.au/C2004A03348/2020-07-01/2020-07-01/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.ato.gov.au/tax-rates-and-codes/tax-rates-if-you-re-under-18-years-old",
            "https://www.ato.gov.au/forms-and-instructions/individual-tax-return-2026-instructions/adjustment-questions-a1-a4-individual-tax-return-2026/a1-under-18-2026",
        ],
    ),
    (
        "smsf_tax",
        &[
            "https://www.legislation.gov.au/C2004A03348/latest/text",
            "https://www.legislation.gov.au/F2018L00605/latest/text",
            "https://www.legislation.gov.au/C2024A00052/asmade/text",
            "https://www.ato.gov.au/forms-and-instructions/self-managed-superannuation-fund-annual-return-2026-instructions/instructions-to-complete-your-smsf-annual-return-2026/section-d-income-tax-calculation-statement-item-13",
            "https://www.ato.gov.au/individuals-and-families/super-for-individuals-and-families/self-managed-super-funds-smsf/smsf-administration-and-reporting/exempt-current-pension-income",
        ],
    ),
    (
        "first_home_owner_grant",
        &[
            "https://content.legislation.vic.gov.au/sites/default/files/2025-11/00-5aa053-authorised.pdf",
            "https://www.sro.vic.gov.au/buying-property/first-home-owner-grant/understanding-first-home-owner-grant",
            "https://www.legislation.qld.gov.au/view/pdf/current/act-2000-014",
            "https://qro.qld.gov.au/property-concessions-grants/first-home-grant/eligibility/",
            "https://www.legislation.wa.gov.au/legislation/statutes.nsf/law_a2000.html",
            "https://www.wa.gov.au/system/files/2026-10/first-home-owner-grant.pdf",
            "https://www.legislation.sa.gov.au/_legislation-documents/lz/c/a/first-home-and-housing-construction-grants-act-2000/current/2000.25.auth.pdf",
            "https://www.revenuesa.sa.gov.au/FHOG/grant-details",
            "https://www.legislation.tas.gov.au/view/html/inforce/current/act-2000-019",
            "https://www.sro.tas.gov.au/first-home-owner/eligibility",
            "https://legislation.nt.gov.au/api/sitecore/Act/PDF?id=11819",
            "https://nt.gov.au/property/home-owner-assistance/buy-build-new-home",
            "https://www.legislation.act.gov.au/a/2000-23/",
            "https://www.revenue.act.gov.au/home-buyer-assistance/first-home-owner-grant",
        ],
    ),
    (
        "psi_test",
        &[
            "https://www.legislation.gov.au/C2004A05138/2020-07-01/2020-07-01/text/original/epub/OEBPS/document_3/document_3.html",
            "https://www.ato.gov.au/law/view/document?docid=TXR/TR20223/NAT/ATO/00001",
            "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/personal-services-income/working-out-if-the-psi-rules-apply/results-test",
        ],
    ),
    (
        "payg_instalment",
        &[
            "https://www.legislation.gov.au/C1953A00001/2025-07-01/2025-07-01/text/original/epub/OEBPS/document_2/document_2.html",
            "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/payg-instalments/how-to-vary-your-payg-instalments",
            "https://www.ato.gov.au/forms-and-instructions/payg-instalments-how-to-complete-your-activity-statement",
            "https://www.ato.gov.au/businesses-and-organisations/gst-excise-and-indirect-taxes/gst/completing-your-bas-for-gst/complete-your-bas",
            "https://www.ato.gov.au/businesses-and-organisations/income-deductions-and-concessions/payg-instalments/calculate-your-payg-instalments/how-we-calculate-your-payg-instalment-amount-or-rate",
        ],
    ),
    (
        "pcg_2021_4",
        &[
            "https://www.ato.gov.au/tax-rates-and-codes/tax-rates-australian-residents",
            "https://www.ato.gov.au/tax-rates-and-codes/company-tax-rates/tax-rates-201112-to-202021",
            "https://www.ato.gov.au/tax-rates-and-codes/company-tax-rates/tax-rates-2025-26",
            "https://www.legislation.gov.au/C2004A03348/latest/text",
        ],
    ),
    (
        "government_super_amounts",
        &[
            "https://www.ato.gov.au/tax-rates-and-codes/key-superannuation-rates-and-thresholds/government-contributions",
            "https://www.ato.gov.au/individuals-and-families/super-for-individuals-and-families/super/growing-and-keeping-track-of-your-super/how-to-save-more-in-your-super/government-super-contributions/super-co-contribution",
            "https://www.ato.gov.au/individuals-and-families/super-for-individuals-and-families/super/growing-and-keeping-track-of-your-super/how-to-save-more-in-your-super/government-super-contributions/low-income-super-tax-offset",
            "https://www.ato.gov.au/individuals-and-families/income-deductions-offsets-and-records/tax-offsets/superannuation-related-tax-offsets",
            "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_6/document_6.html",
            "https://www.legislation.gov.au/F2022L00155/asmade/2022-02-18/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.legislation.gov.au/C2026A00008/asmade/text",
        ],
    ),
    (
        "residency_tests",
        &[
            "https://www.legislation.gov.au/C1936A00027/2020-07-01/2020-07-01/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.ato.gov.au/law/view/document?docid=TXR/TR20231/NAT/ATO/00001",
        ],
    ),
    (
        "lodgment_required",
        &[
            "https://www.legislation.gov.au/F2021L00577/asmade/text",
            "https://www.legislation.gov.au/F2022L00508/asmade/text",
            "https://www.legislation.gov.au/F2023L00533/asmade/text",
            "https://www.legislation.gov.au/F2024L00550/asmade/text",
            "https://www.legislation.gov.au/F2025L00570/asmade/text",
        ],
    ),
    (
        "deferred_tax",
        &[
            "https://www.aasb.gov.au/admin/file/content105/c9/AASB112_08-15_COMPmar20_07-21.pdf",
            "https://standards.aasb.gov.au/sites/default/files/2024-06/AASB18_06-24_AppendixD_0.pdf",
            "https://standards.aasb.gov.au/ad-aasb-112-12-12-dec-2012",
            "https://www.legislation.gov.au/C2004A03348/2026-07-01/2026-07-01/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.ato.gov.au/tax-rates-and-codes/company-tax-rates/tax-rates-201112-to-202021",
            "https://www.ato.gov.au/tax-rates-and-codes/company-tax-rates/tax-rates-2025-26",
        ],
    ),
    (
        "long_service_leave",
        &[
            "https://legislation.nsw.gov.au/view/whole/html/inforce/current/act-1955-038",
            "https://www.nsw.gov.au/employment/rights-responsibilities/leave/long-service-leave",
            "https://www.legislation.qld.gov.au/view/whole/html/inforce/current/act-2016-063",
            "https://www.legislation.qld.gov.au/view/whole/html/asmade/act-2020-019",
            "https://www.legislation.wa.gov.au/legislation/statutes.nsf/main_mrtitle_548_homepage.html",
            "https://www.legislation.sa.gov.au/lz?path=%2FC%2FA%2FLONG%20SERVICE%20LEAVE%20ACT%201987",
            "https://www.legislation.tas.gov.au/view/whole/html/inforce/current/act-1976-095",
            "https://www.legislation.act.gov.au/a/1976-27/",
            "https://www.worksafe.act.gov.au/__data/assets/pdf_file/0009/1673964/Long-service-leave-calculation-guidelines.pdf",
            "https://legislation.nt.gov.au/Legislation/LONG-SERVICE-LEAVE-ACT-1981",
        ],
    ),
    (
        "leave_provisions",
        &[
            "https://standards.aasb.gov.au/sites/default/files/2026-11/AASB119_08-15_COMPdec22_07-26.pdf",
            "https://aasb.gov.au/admin/file/content105/c9/AASB2022-9_12-22.pdf",
            "https://www.aasb.gov.au/admin/file/content105/c9/AASB101_07-15_COMPdec22_01-24.pdf",
            "https://www.aasb.gov.au/admin/file/content105/c9/AASB101_07-15_ACOMPdec22_01-23.pdf",
        ],
    ),
    (
        "foreign_income_conversion",
        &[
            "https://www.legislation.gov.au/F2021L00206/latest/text",
            "https://www.legislation.gov.au/F1997B02703/2020-04-01/2020-04-01/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.ato.gov.au/businesses-and-organisations/corporate-tax-measures-and-assurance/foreign-exchange-gains-and-losses/translation-conversion-rules",
            "https://www.ato.gov.au/businesses-and-organisations/corporate-tax-measures-and-assurance/foreign-exchange-gains-and-losses/in-detail/general-information-on-average-rates",
            "https://www.ato.gov.au/tax-rates-and-codes/foreign-exchange-rates-overview",
        ],
    ),
    (
        "foreign_income_tax_offset",
        &[
            "https://www.ato.gov.au/law/view/document?docid=PAC/19860107/20",
            "https://www.ato.gov.au/law/view/document?docid=PAC/19860107/18",
            "https://www.ato.gov.au/individuals-and-families/coming-to-australia-or-going-overseas/coming-to-australia/tax-free-threshold-for-newcomers-to-australia",
            "https://www.ato.gov.au/individuals-and-families/coming-to-australia-or-going-overseas/living-overseas-and-becoming-a-foreign-tax-resident",
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
    (
        "help_indexation",
        &[
            "https://www.legislation.gov.au/C2004A01234/2026-10-17/2026-10-17/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.legislation.gov.au/C2024A00108/asmade/2024-12-05/text/2/epub/OEBPS/document_1/document_1.html",
            "https://www.legislation.gov.au/C2025A00030/asmade/2025-08-02/text/original/epub/OEBPS/document_1/document_1.html",
            "https://www.ato.gov.au/individuals-and-families/study-and-training-support-loans/view-your-study-loan-account-online",
            "https://www.ato.gov.au/individuals-and-families/study-and-training-support-loans/voluntary-repayments",
        ],
    ),
    (
        "defined_benefit_income_cap",
        &[
            "https://www.ato.gov.au/law/view/document?DocID=PAC/19970038/303-2",
            "https://www.ato.gov.au/law/view/document?DocID=PAC/19970038/303-3",
            "https://www.ato.gov.au/law/view/document?DocID=COG/LCR20171/NAT/ATO/00001",
            "https://www.ato.gov.au/tax-rates-and-codes/key-superannuation-rates-and-thresholds/transfer-balance-cap",
        ],
    ),
    (
        "foreign_pension_upp",
        &[
            "https://www.legislation.gov.au/C1936A00027/2007-05-11/2007-05-11/text/original/pdf/1",
            "https://www.legislation.gov.au/F2025L01060/asmade/2025-09-09/text/original/pdf",
            "https://www.ato.gov.au/forms-and-instructions/individual-supplementary-tax-return-2026-instructions/deduction-questions-d11-d15-supplementary-tax-return-2026/d11-deductible-amount-of-upp-of-a-foreign-pension-or-annuity-2026",
        ],
    ),
    (
        "scholarship_exemption",
        &[
            "https://www.legislation.gov.au/C2004A05138/2017-07-01/2017-07-01/text/original/pdf/2",
            "https://www.ato.gov.au/individuals-and-families/income-deductions-offsets-and-records/income-you-must-declare/scholarships-prizes-and-awards/scholarship-payments",
            "https://www.ato.gov.au/calculators-and-tools/is-my-scholarship-taxable",
        ],
    ),
    (
        "super_crystallisation",
        &[
            "https://www.legislation.gov.au/C1936A00027/2007-02-19/2007-02-19/text/original/pdf/1",
            "https://www.ato.gov.au/tax-and-super-professionals/for-superannuation-professionals/apra-regulated-funds/managing-member-benefits/managing-and-calculating-member-benefits/calculating-components-of-a-super-benefit",
            "https://www.ato.gov.au/calculators-and-tools/super-crystallisation-calculator",
        ],
    ),
    (
        "endorsement_review",
        &[
            "https://www.ato.gov.au/law/view/document?DocID=PAC/19970038/50-50",
            "https://www.ato.gov.au/law/view/document?DocID=PAC/19530001/Sch1-426-45",
            "https://www.ato.gov.au/calculators-and-tools/charities-income-tax-exempt-charities-endorsement-review-worksheet",
            "https://www.ato.gov.au/calculators-and-tools/charities-review-of-a-dgr-endorsed-as-a-whole-worksheet-1",
            "https://www.ato.gov.au/calculators-and-tools/charities-review-of-a-dgr-endorsed-as-a-part-of-an-organisation-worksheet-2",
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
    (
        "motor_vehicle_duty",
        &[
            "Each revenue office or registration authority decides the dutiable value, the vehicle's class and any exemption or concession; this worksheet takes the value and class as established.",
            "The scales are those in force or made on 24 November 2026; an amendment for a later duty date is not reflected.",
        ],
    ),
    (
        "insurance_duty",
        &[
            "Each revenue office decides the class, any exemption and how a premium is apportioned between jurisdictions and classes; this worksheet takes the class and the apportioned premium as established.",
            "No Act sets a rounding rule for one premium's duty: each is paid on a return of total premiums. amounts.duty is rates.exact_duty to the cent, for display; add exact figures when totalling a return.",
            "The rates are those in force on 24 November 2026; an amendment for a later effected date is not reflected.",
        ],
    ),
    (
        "long_service_leave",
        &[
            "An award, enterprise agreement, pre-modern federal award or portable long service leave scheme can displace the Act, and what counts as service and ordinary pay are the operator's facts: this worksheet applies the Act to them and does not decide them.",
        ],
    ),
    (
        "esic",
        &[
            "The investor must work out whether the company qualifies at the test time; the ATO can rule on the principles-based test, and this worksheet takes it and the points items as stated.",
            "The offset is non-refundable and can be carried forward (Division 65); this worksheet does not apply it against tax.",
        ],
    ),
    (
        "leave_provisions",
        &[
            "The discount rate, salary growth, probabilities, settlement expectations and rights to defer are the preparer's stated assumptions and judgements (AASB 119 paras 75 to 87; AASB 101 para 69); the worksheet applies them and does not test them.",
        ],
    ),
    (
        "vic_property_taxes",
        &[
            "The State Revenue Office decides whether land is vacant residential land, has a qualifying use or is exempt, and the Valuer-General sets its values; this worksheet takes those facts and values as established.",
            "No land leaves the commercial and industrial property tax's 10-year transition period before 1 July 2034, so the tax is nil in 2025 and 2026. The earliest possible tax year shown is not a finding of future liability: the land must then still be tax reform scheme land with a qualifying use that is taxable for land tax (s 3), and a future entry date is a hypothetical.",
            "Each amount is the statutory formula amount shown to the cent, half up, before any administrative adjustment the Commissioner makes; it is not an assessed amount.",
        ],
    ),
    (
        "asic_fees",
        &[
            "Public holidays are not checked: if the due date or the last day of the first month after it is one where the payment or document reaches ASIC, check ASIC's invoice. A late fee is on top of the fee itself. GST does not apply to ASIC fees.",
        ],
    ),
    (
        "fuel_tax_credits",
        &[
            "Each line's rate is the one the ATO publishes for its acquisition day; the ATO's fuel tax credit calculator remains the authority for an amount claimed.",
        ],
    ),
    (
        "notice_period",
        &[
            "An award, enterprise agreement or contract can require longer notice than the National Employment Standards, and whether the employee is a national system employee, the days not counted and any s 123 exclusion are the operator's facts: this worksheet applies s 117 to them and does not decide them.",
        ],
    ),
    (
        "work_expenses",
        &[
            "The higher method is named for comparison only: each method has its own records (PCG 2023/1 paras 56 to 63, ITAA 1997 Division 900 and Subdivision 28-G), and the taxpayer chooses one for the year.",
        ],
    ),
    (
        "ato_interest_penalties",
        &[
            "Every amount is before remission, which is the Commissioner's discretion (TAA 1953 s 8AAG, Sch 1 ss 280-160 and 298-20). GIC and SIC are worked exactly and rounded once, so a statement that rounds daily may differ by a few cents.",
        ],
    ),
    (
        "private_health_insurance_rebate",
        &[
            "Each line's share of the PHII benefit is rounded to the cent, as the ATO's worked examples do; no provision sets the rounding. An offset on one line and an excess on another are both shown, as ITAA 1997 s 61-210 and PHI Act s 282-18 each work on a premium; a line that sums premiums with different reductions shows only their net, not each premium's offset or excess, and as its share is rounded once its net can differ from theirs by a cent; give such premiums as separate lines.",
        ],
    ),
    (
        "depreciation_small_business_pool",
        &[
            "Which assets the instant asset write-off or temporary full expensing took instead of the pool is the operator's to establish; this worksheet works out the pool only.",
        ],
    ),
    (
        "capital_works",
        &[
            "undeducted_construction_expenditure is not carried forward here: it falls by more than the deduction where the area was not used to produce income (s 43-235).",
        ],
    ),
    (
        "rental_property",
        &[
            "Each line is the operator's classification: an expense that is capital, an initial repair or an improvement is not deductible under s 8-1 and goes to the cost base or capital works instead.",
            "The amounts are to the cent; a return shows whole dollars, and the ATO's examples round the same figures to the dollar.",
        ],
    ),
    (
        "foreign_resident_withholding",
        &[
            "A clearance certificate counts only if the vendor gave it before you pay and its period covers the contract date (s 14-210(2)); a notice of variation only for the asset and up to its maximum price. Check each document's names against the title.",
            "Where every vendor gave a document and one is a notice of variation, the class variations' 'not all' condition is not met; this follows the ATO's joint owner example and works each vendor's share alike.",
            "The amount is due on or before the day you become the owner (s 14-200(2)), with the purchaser payment notification; the vendor claims it as a credit in their return.",
        ],
    ),
    (
        "sb_cgt_concessions",
        &[
            "An open concession is one whose conditions the facts given meet; the choices, the order (s 152-210) and the amounts are the taxpayer's, and a company or trust must pay its CGT concession stakeholders within the time s 152-325 sets.",
        ],
    ),
    (
        "gst_creditable_purpose_change",
        &[
            "application_basis is the operator's statement of the apportionment method; the command does not judge whether it is fair or reasonable (GSTR 2006/4 and GSTR 2006/3 discuss direct and indirect methods).",
        ],
    ),
    (
        "small_business_income_offset",
        &[
            "The ATO works out the offset from the return and shows it on the notice of assessment; this is a check of that amount, not a claim.",
        ],
    ),
    (
        "trading_stock",
        &[
            "Making or not making the s 328-285 choice, and each item's valuation method, are the taxpayer's elections; the result shows the consequence of the ones given.",
        ],
    ),
    (
        "prepayments",
        &[
            "Whether s 82KZME applies is taken as stated; the worksheet does not test the tax shelter conditions.",
        ],
    ),
    (
        "non_commercial_losses",
        &[
            "The tests are applied to the figures given; whether the activity is a business, which activities are similar, what is used on a continuing basis and whether an estimate is reasonable stay with the operator.",
        ],
    ),
    (
        "minors_income",
        &[
            "A prescribed person's low income tax offset cannot exceed the basic income tax liability not attributable to eligible taxable income (ITAA 1997 s 61-115(2)); tax_on_other_income is that part where the special rates apply. No offset or levy is applied here.",
        ],
    ),
    (
        "smsf_tax",
        &[
            "The ATO assesses the fund from its return; this is a check of the return's item 13, not a lodgment. The ATO has not published the SMSF annual return 2027 instructions, so 2026-27 follows the law in force on 22 November 2026 and the 2026 return's labels.",
        ],
    ),
    (
        "ess_discount",
        &[
            "The employer's ESS statement reports the discounts; this checks them from the facts given. A forfeiture exclusion is claimed by asking the ATO to amend the assessment for the year included, which has no time limit for this.",
        ],
    ),
    (
        "first_home_owner_grant",
        &[
            "The grant is paid only on an application the revenue office or an approved agent decides; confirm it with them before relying on it. The New South Wales rules rest on Revenue NSW's page: the Act could not be read on 23 November 2026.",
        ],
    ),
    (
        "psi_test",
        &[
            "A personal services business is still subject to Part IVA of the ITAA 1936 on splitting the income (TR 2022/3 paragraphs 25 and 160); the PSI is still reported in the tax return.",
        ],
    ),
    (
        "pcg_2021_4",
        &[
            "A green zone is not a ruling: Part IVA can still apply (PCG 2021/4 paragraphs 36 and 37). Whether the gateways are passed is the operator's answer.",
        ],
    ),
    (
        "government_super_amounts",
        &[
            "The Commissioner determines the co-contribution and LISTO from the return and the fund's reports and pays them to the fund; these are estimates from the facts given.",
        ],
    ),
    (
        "residency_tests",
        &[
            "Residing, domicile, the places of abode and intention are your judgements under TR 2023/1; the worksheet applies them and counts days, and does not test them.",
            "A resident under any test is a resident for at least part of the year, and a resident for at least one day is taxed at resident rates; when residency began or ended, and the part-year tax-free threshold, are not worked out (TR 2023/1 paragraphs 105 to 107).",
        ],
    ),
    (
        "lodgment_required",
        &[
            "A person not required to lodge may still choose to, for example to claim a refund of tax withheld; the ATO may also ask a person to lodge.",
            "The due date is the instrument's for lodging yourself; a registered tax agent's lodgment program may allow a later date.",
        ],
    ),
    (
        "foreign_income_conversion",
        &[
            "Each Australian dollar amount is rounded half up to cents and the totals add the rounded amounts; keep the rate used and its source with your records.",
        ],
    ),
    (
        "foreign_income_tax_offset",
        &[
            "The ATO works out the offset from the return; this checks the amount, and does not decide which foreign tax counts or what a tax treaty allows.",
        ],
    ),
    ("fbt_car_statutory", &[CAR_REPORTABLE_NOTE]),
    (
        "fbt_instalment",
        &[
            "If the instalments are varied on an estimate less than 90% of the tax assessed for the year, the general interest charge may apply to the shortfall (FBTAA s 112B); this worksheet does not work it out.",
        ],
    ),
    (
        "fbt_ev_exemption_check",
        &[
            "On 5 May 2026 the government announced that from 1 April 2027 the exemption would continue in full only for an electric car costing $75,000 or less, with a 25% discount for one costing more but below the luxury car tax threshold; the ATO said it was not yet law when this was checked on 19 November 2026, so the year ended 31 March 2028 is refused.",
        ],
    ),
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
        "foreign_purchaser_surcharge",
        &[
            "Each revenue office decides who is a foreign person or absentee, what property the surcharge applies to and any exemption or refund; this worksheet takes the facts as established.",
            "The duty surcharge is paid on the transaction and the land tax surcharge every land tax year; they are not added together.",
        ],
    ),
    (
        "dasp_tax",
        &[
            "Each fund decides the rate for the payment it makes from the contributions it holds: a person who held a working holiday maker visa can have the 65% rate on one fund's payment and the ordinary rates on another's.",
        ],
    ),
    (
        "super_benefit_tax",
        &[
            "maximum_tax excludes the Medicare levy, which the ATO's tables add at 2% to each rate, and is a ceiling: each offset ensures the rate on its part does not exceed the rate shown, so the person's marginal rates on that part can give less. Put assessable_amount in the tax return; the ATO works out the lump sum offsets.",
        ],
    ),
    (
        "super_withholding",
        &[
            "The amount withheld is a credit against the payee's tax, which super-benefit-tax works out; it is not the tax. A payee with a tax treaty that makes the payment assessable only in another country has nothing withheld, which this command does not decide.",
        ],
    ),
    (
        "land_tax",
        &[
            "Each revenue office decides which land is taxable, its value and whether an owner is a foreign person or absentee; this worksheet takes the taxable value and any surcharge value as established.",
            "The ACT assesses land tax each quarter on the days in the quarter; this worksheet gives the full year's amount, which the quarterly assessments can differ from by cents.",
        ],
    ),
    (
        "workers_compensation_premium",
        &[
            "The insurer's notice is the premium: this worksheet takes the wages, the industry or premium rate and each scheme's facts as established, and holds no rate table.",
            "An employer may be exempt from holding a policy, which this worksheet does not decide: in New South Wales with wages of $7,500 or less and no apprentice or trainee, outside a group; in Victoria with rateable remuneration of $7,500 or less for 2026-27 (Premiums Order (No. 34) cl 7), unless it employs an apprentice or trainee.",
            "South Australia's work health and safety fee, which carries no GST, is added to the premium and is not worked here.",
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
        "transfer_duty",
        &[
            "Each revenue office decides whether a transaction is dutiable, its dutiable value and whether a buyer meets a first home concession's conditions; this worksheet takes them as established.",
            "Only the first home concession asked for is applied; a buyer may be entitled to another concession or exemption, or liable to a foreign purchaser surcharge, which this worksheet does not apply.",
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
        "cgt_parcels",
        &[
            "The ATO accepts first in, first out where the shares or units sold cannot be identified, and the taxpayer's own selection (TD 33 para 4); records must support the parcels chosen.",
        ],
    ),
    (
        "cgt_cost_base_adjustments",
        &[
            "The amounts adjust the parcels' cost bases only: give discount_capital_gains and other_capital_gains to cgt-parcels as its other_discount_capital_gains and other_capital_gains for the same year, and its parcels as cgt-parcels' parcels.",
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
        "invalid_carer_tax_offset",
        &[
            "For part of the year the offset is days x the ATO's published daily rate, the ATO's reasonable apportionment (ITAA 1997 s 61-40(2)); the Commissioner may apportion otherwise.",
        ],
    ),
    (
        "zone_tax_offset",
        &[
            "The area is the operator's: the ATO's Australian zone list is not exhaustive, and ITAA 1936 Schedule 2 and s 79A(3D) to (3F) define the zones and special areas. For 2026-27 the ATO had not published its T4 instructions on 22 November 2026: the amounts are the law's, and the part-year sole parent daily rate is $1,607 / 365, the rule its earlier rates follow.",
        ],
    ),
    (
        "lump_sum_arrears_tax_offset",
        &[
            "The Medicare levy and Medicare levy surcharge are separate questions this worksheet does not answer. From 2024-25 Medicare Levy Act 1986 s 9A leaves the total arrears amount out of taxable income for the levy when it is at least 10% of normal taxable income, the arrears included, and the levy in each of the 2 latest accrual years meets a condition of s 9A(1)(c); ITAA 1997 Subdivision 61-L gives a tax offset against the surcharge on them.",
        ],
    ),
    (
        "aml_cdd_review",
        &[
            "AML/CTF Act 2006 s 30(2)(c) sets no fixed calendar interval for periodic KYC reviews; the 12, 24 and 36 month intervals are AUSTRAC Accountant Program Starter Kit defaults for small accounting practices, not statutory mandates.",
            "This worksheet processes no personal names and performs no individual DFAT sanctions, PEP or adverse-media screening; carry out screening separately through current authoritative lists.",
        ],
    ),
    (
        "ppsr_timing",
        &[
            "For PPSA s 10, list public holidays for the whole of any State, the ACT or the NT and dates of Registrar suspensions under s 147(5). For Corporations Act s 9, list public and bank holidays in the place concerned. Omitting an excluded date can make the calculated deadline too early. Listing a date that should count can make it too late.",
            "A court may extend the PPSA s 62(3) period (s 293) or fix a later s 588FL time (s 588FM); the times given are those without an order, unless court_ordered_time is given.",
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
        "fhss",
        &[
            "associated_earnings worked out from contributions is an estimate: GN 2024/1 example 6 (Jill) states $5,190 of associated earnings, which s 138-40 at the ATO's published SIC rates does not reproduce on any reading tried (the counted amount at 85% or in full, either contribution order); the example also counts 2024-25 contributions ahead of 2023-24 ones, which s 138-30(2) counts first. The Commissioner's determination is the figure to use.",
            "The assessable FHSS released amount is included in the income year that corresponds to the financial year in which the release was requested, even if it is paid in the next (s 313-20(1)); income-tests leaves it out of the income tests it lists.",
        ],
    ),
    (
        "fbt_entertainment_classifier",
        &[
            "TR 97/17 and TR 2007/12 are public rulings: the Commissioner's view of the law, not the law, and their paragraphs are cited as that view. FBTAA s 58P(1)(e) requires a minor benefit's notional taxable value to be less than $300, which this classifier applies; the $100 threshold in TR 97/17 para 27 is superseded and not applied.",
            "A minor benefit is shown only as possible: whether it would be unreasonable to treat it as a fringe benefit (FBTAA s 58P(1)(f)) is the employer's judgement, and the totals treat it as a fringe benefit. undetermined_cost is the cost left out of the totals for want of a stated fact.",
        ],
    ),
    (
        "help_indexation",
        &[
            "A repayment counts from the day the ATO receives and processes it: the ATO says electronic and Australia Post payments can take up to 4 business days, and indexation applies if a payment is not received and processed before 1 June. This command takes the dates as given.",
            "The factors are the ATO's published rates plus one; the CPI and WPI factors behind them were not recomputed from ABS index numbers here.",
        ],
    ),
    (
        "defined_benefit_income_cap",
        &[
            "Whether an income stream is a capped defined benefit income stream, and whether each benefit gets concessional treatment, are your statements; ask the fund, which reports them on the PAYG payment summary.",
        ],
    ),
    (
        "foreign_pension_upp",
        &[
            "The ATO decides a foreign pension's deductible amount by private ruling on a request for a determination (NAT 16543); the formula's amount rests on the purchase price and relevant number you give, and the Commissioner may find it inappropriate (s 27H(3)).",
        ],
    ),
    (
        "scholarship_exemption",
        &[
            "Whether the student is full-time, the payment's conditions and its educational purpose are your judgements on the scholarship agreement; the ATO's examples show how it reads work experience.",
        ],
    ),
    (
        "super_crystallisation",
        &[
            "The non-qualifying and excessive components are taken as nil, as the ATO's calculator takes neither; the pre-July 83 component is shown to the cent, the law giving no rounding rule.",
        ],
    ),
    (
        "endorsement_review",
        &[
            "The worksheets date from 2021; each answer is tied to the provision as read on 24 November 2026, and conditions they do not ask about are listed under not_asked.",
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
/// The vacant-land limit for holding costs incurred from 1 July 2019.
pub const BORROWING_S26_102: &str = "ITAA 1997 s 26-102 can limit deductions for vacant land holding costs incurred from 1 July 2019, including ongoing costs of borrowing to acquire the land. The s 25-25 amounts are before any applicable limit. Check which costs and exceptions apply.";

/// The flag on a borrowing expenses year from 2027-28 (ITAA 1997 s 26-155,
/// inserted by Act No 49 of 2026 for the 2027-28 income year and later years,
/// read on 19 November 2026).
pub const BORROWING_S26_155: &str = "From 2027-28 ITAA 1997 s 26-155(1) quarantines the amount by which deductions relating to using or holding residential dwellings as residential accommodation exceed the assessable income from them, and this deduction is one of them where the money is used that way. The amount here is s 25-25's before s 26-155: the worksheet neither applies s 26-155 nor decides its exceptions, such as a dwelling last acquired before 7.30 pm on 12 May 2026 (ACT legal time) or a new residential dwelling (s 26-155(2)), or an excepted entity (s 26-155(4)).";

/// The refusal of a leisure facility under s 26-50.
pub const BORROWING_S26_50: &str = "Section 26-50 denies specified leisure-facility expenses, subject to the full-year exception in subsection (3), the reasonable part-year share in subsection (4) and the fringe-benefit exception in subsection (8). The anti-avoidance rule in subsection (7) can disapply the use exception. Establish whether section 26-50 restricts this deduction before using the worksheet.";

/// The note on a borrowing expenses schedule with a row after the last year read.
pub const BORROWING_PROJECTED: &str = "A row marked projected is after 2027-28, the last income year read: it applies ITAA 1997 s 25-25 as read on 19 November 2026, which sets no figure by year, and a later amendment would change it.";

/// Why a worksheet that takes an income year refuses 2027-28, added to its
/// `period.unsupported` message: the figures not yet published, or the change
/// in the law it does not apply.
pub const UNPUBLISHED_2027_28: &[(&str, &str)] = &[
    (
        "aml_cdd_review",
        "The AML/CTF Act 2006, AML/CTF Rules 2025 and AUSTRAC guidance were read for 2026-27 on 24 November 2026; 2027-28 amendments and guidance updates are not yet read.",
    ),
    (
        "alcohol_excise",
        "The excise duty rates for goods entered from 1 February 2027 had not been published when this worksheet was checked, so no 2027-28 rate is held.",
    ),
    (
        "asic_fees",
        "ASIC's fees from 1 July 2027 are indexed on the March 2027 quarter CPI and had not been published when this worksheet was checked on 24 November 2026.",
    ),
    (
        "work_expenses",
        "No cents per kilometre rate for 2027-28 had been determined under ITAA 1997 s 28-25(4) when this worksheet was checked.",
    ),
    (
        "super_benefit_tax",
        "The untaxed plan cap for 2027-28 was not published when this worksheet was checked.",
    ),
    (
        "super_withholding",
        "The untaxed plan cap for 2027-28 was not published when this worksheet was checked.",
    ),
    ("capital_gains", CGT_2027_28),
    ("main_residence", CGT_2027_28),
    ("cgt_parcels", CGT_2027_28),
    ("cgt_cost_base_adjustments", CGT_2027_28),
    ("balancing_adjustment", CGT_2027_28),
    (
        "rental_property",
        "From 2027-28 ITAA 1997 s 26-155 (Act No. 49 of 2026) can quarantine residential rental deductions above the rental income, which this worksheet does not apply.",
    ),
    (
        "sb_cgt_concessions",
        "From 2027-28 Schedule 1 to Act No. 49 of 2026 changes Division 152, and this worksheet has not been checked against it.",
    ),
    (
        "government_super_amounts",
        "From 2027-28 Schedule 4 to Act No. 8 of 2026 replaces LISTO's $37,000 limit and $500 maximum, and the Commissioner has not published the 2027-28 co-contribution thresholds.",
    ),
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
        "2027-28 has 366 days, and this worksheet has been checked for a year of 366 days only in 2023-24.",
    ),
    ("payg_withholding", WITHHOLDING_2027_28),
    ("horticulture_shearing_withholding", WITHHOLDING_2027_28),
    ("upward_variation_withholding", WITHHOLDING_2027_28),
    ("daily_casual_withholding", WITHHOLDING_2027_28),
    ("entertainer_withholding", WITHHOLDING_2027_28),
    ("return_to_work_withholding", WITHHOLDING_2027_28),
    ("annuity_withholding", WITHHOLDING_2027_28),
    ("voluntary_agreement_withholding", WITHHOLDING_2027_28),
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
        "private_health_insurance_rebate",
        "The 2027-28 private health insurance income thresholds, and the rebate percentages from 1 April 2027, had not been published when this was checked on 22 November 2026.",
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
/// Adds a source and a warning to a result whose optional input brought in a
/// rule its fixed scope, sources and warnings do not cover, and rewrites the
/// scope's exclusion of that rule with each `(from, to)` pair.
pub fn add_note(
    out: &mut serde_json::Value,
    (source, warning): (&str, &str),
    scope: &[(&str, &str)],
) {
    for (key, text) in [("sources", source), ("warnings", warning)] {
        if let Some(list) = out[key].as_array_mut() {
            list.push(serde_json::Value::from(text));
        }
    }
    if let Some(text) = out["scope"].as_str() {
        let rewritten = scope.iter().fold(text.to_string(), |text, (from, to)| {
            text.replacen(from, to, 1)
        });
        out["scope"] = serde_json::Value::from(rewritten);
    }
}

/// Added to a `contribution-caps` result given the downsizer inputs, whose
/// other results are unchanged.
pub const DOWNSIZER_SOURCE: &str =
    "https://www.ato.gov.au/law/view/document?DocID=PAC/19970038/292-102";
pub const DOWNSIZER_WARNING: &str = "downsizer_covered is the part of the contribution s 292-102 covers, which does not count towards the concessional or non-concessional cap; the operator states the criteria of s 292-102(1)(b) to (f), (h), (i) and (2). downsizer_not_covered is not a downsizer contribution and is not added to non_concessional_contributions: whether the fund accepts it as a personal contribution, which would count towards the non-concessional cap, is the fund's decision.";

/// Added to a `pension-minimum` result for a transition to retirement income
/// stream, whose other results are unchanged.
pub const TTR_MAXIMUM_SOURCE: &str = "https://www.legislation.gov.au/F1996B00580/latest/text";
/// The rewrites of the `contribution-caps` scope given the downsizer inputs.
pub const DOWNSIZER_SCOPE: &[(&str, &str)] = &[(
    "the CGT cap, downsizer and other excluded contributions,",
    "the CGT cap, other excluded contributions (the downsizer contribution given is tested under ITAA 1997 s 292-102 on the stated age, dates, limits and conditions),",
)];
/// The rewrites of the `pension-minimum` scope for a transition to retirement
/// income stream.
pub const TTR_MAXIMUM_SCOPE: &[(&str, &str)] = &[
    (
        "Excludes pensions under",
        "Includes the transition to retirement 10% maximum of SISR reg 6.01(2). Excludes pensions under",
    ),
    ("the transition to retirement 10% maximum, ", ""),
];
pub const TTR_MAXIMUM_WARNING: &str = "maximum_payment is 10% of the account balance on 1 July, or on the commencement day in the first year, under SISR reg 6.01(2) 'transition to retirement income stream'; it is not reduced for a part year, it is shown down to the cent so a payment of it stays within the limit, it excludes commutations and includes payment splits, and it ends once the pensioner satisfies a condition of release with a nil cashing restriction.";

pub const REFUSAL_CODES: &[(&str, &str)] = &[
    (
        "alcohol_excise.classification_unsettled",
        "A line's labelled strength is above its actual strength and would put the goods in another subitem, which Excise Act 1901 s 77FA does not settle.",
    ),
    (
        "alcohol_excise.date_outside_year",
        "A line's date is outside the financial year given.",
    ),
    (
        "alcohol_excise.fact_conflict",
        "manufacturer_from is after a line marked remission.",
    ),
    ("alcohol_excise.line_invalid", "A line is not an object."),
    (
        "alcohol_excise.lines_invalid",
        "lines is not a list of 1 to 1,000 line objects.",
    ),
    (
        "alcohol_excise.not_excisable",
        "The goods are wine, non-commercial liquor made with non-commercial equipment, or beer or another beverage of 1.15% alcohol by volume or less, none of which is in Excise Tariff Act 1921 Schedule items 1 to 3.",
    ),
    (
        "alcohol_excise.product_invalid",
        "A line's product is not beer, brandy, spirit or other_excisable_beverage.",
    ),
    (
        "alcohol_excise.product_out_of_scope",
        "The line is tobacco, fuel, spirit under the free subitems 3.5 to 3.8 (an approval or determination), or imported goods (customs duty), none of which the worksheet works.",
    ),
    (
        "alcohol_excise.quantity_invalid",
        "A volume, container or strength is not a decimal string above 0 within its limit and decimal places.",
    ),
    (
        "alcohol_excise.rate_unpublished",
        "A line is on or after 1 February 2027, when the next substituted rates apply and none is held.",
    ),
    (
        "aml_cdd_review.basel_score_invalid",
        "basel_aml_score is not a decimal string from 0.00 to 10.00 with at most 2 decimal places.",
    ),
    (
        "aml_cdd_review.cadence_basis_invalid",
        "cadence_basis is not austrac_accountant_starter_kit, custom_policy or statutory_only.",
    ),
    (
        "aml_cdd_review.chains_invalid",
        "ownership_chains is not a list of up to 16 chains, each holding 1 to 8 percentage strings above 0 to 100 with at most 4 decimal places.",
    ),
    (
        "aml_cdd_review.date_outside_year",
        "last_cdd_date does not fall within 2026-07-01 to 2027-06-30 for period 2026-27.",
    ),
    (
        "aml_cdd_review.duplicate_field",
        "A named input field was supplied more than once to aml-cdd-review.",
    ),
    (
        "aml_cdd_review.entity_kind_invalid",
        "entity_kind is not body_corporate or non_body_corporate.",
    ),
    (
        "aml_cdd_review.exemption_inconsistent",
        "listed_public_company exemption was supplied for a non_body_corporate customer.",
    ),
    (
        "aml_cdd_review.exemption_invalid",
        "customer_exemption is not none, listed_public_company, government_body, controlled_by_listed_or_government or owned_through_listed_or_government_only.",
    ),
    (
        "aml_cdd_review.months_invalid",
        "policy_review_months is not an integer from 1 to 60.",
    ),
    (
        "aml_cdd_review.ownership_above_100",
        "Combined effective ownership across ownership_chains exceeds 100%.",
    ),
    (
        "aml_cdd_review.pep_change_inconsistent",
        "new_pep_status_change is true whilst pep_status is none.",
    ),
    (
        "aml_cdd_review.pep_invalid",
        "pep_status is not none, foreign, domestic or international_organisation.",
    ),
    (
        "aml_cdd_review.question_invalid",
        "question is not country_risk, beneficial_ownership or cdd_review.",
    ),
    (
        "aml_cdd_review.risk_invalid",
        "customer_risk is not low, medium or high.",
    ),
    (
        "aml_cdd_review.starter_kit_risk_inconsistent",
        "Worksheet safeguard: austrac_accountant_starter_kit was chosen with a mandatory enhanced CDD trigger under s 32 whilst customer_risk is not high.",
    ),
    (
        "annuity_withholding.deductible_above_payment",
        "The deductible amount divided by the instalments exceeds the annuity payment.",
    ),
    (
        "annuity_withholding.extra_pay_without_tfn",
        "The additional amount for 53 weekly or 27 fortnightly pays was asked for on scale 4, for which Schedule 6 gives only the no TFN rate.",
    ),
    (
        "annuity_withholding.income_not_whole_cents",
        "The annuity payment less the deductible amount divided by the instalments does not come to whole cents, which Schedule 6 does not round.",
    ),
    (
        "asic_fees.dates_outside_year",
        "A due date or received date falls outside 2026-27, whose amounts the worksheet holds.",
    ),
    (
        "asic_fees.fee_unknown",
        "fee is not one of the ASIC fees the worksheet holds.",
    ),
    (
        "asic_fees.weekend_undecided",
        "A move of the due date, or of the end of the month after it, past a weekend could change the late fee tier, which the worksheet does not decide.",
    ),
    (
        "ato_interest_penalties.date_order",
        "A date is out of the order the question's period needs.",
    ),
    (
        "ato_interest_penalties.fact_invalid",
        "A fact is not of the form the question takes.",
    ),
    (
        "ato_interest_penalties.increase_with_nil",
        "A statement penalty part has an increase and a voluntary disclosure that reduces its base penalty amount to nil, which the law does not settle.",
    ),
    (
        "ato_interest_penalties.item_not_worked",
        "A trust's or partnership's position that is not reasonably arguable (s 284-90(1) items 5 and 6) is not worked.",
    ),
    (
        "ato_interest_penalties.liability_not_listed",
        "The early payment is of an amount Overpayments Act s 8A(1)(a) does not list.",
    ),
    (
        "ato_interest_penalties.not_entitled",
        "The amount is not relevant tax overpaid as a result of a decision to which the Overpayments Act applies.",
    ),
    (
        "ato_interest_penalties.overpaid",
        "A GIC payment is more than the amount owing that day, or a refund more than the payments.",
    ),
    (
        "ato_interest_penalties.penalty_unit_unpublished",
        "The failure to lodge due date is before 28 December 2012, where the ATO's penalty unit table starts its values.",
    ),
    (
        "ato_interest_penalties.question_invalid",
        "question is not one the command answers.",
    ),
    (
        "ato_interest_penalties.rate_unpublished",
        "A day falls outside the ATO's published GIC, SIC or base interest rates, and for GIC no assumed rate was given.",
    ),
    (
        "ato_interest_penalties.remission_discretionary",
        "Remission is the Commissioner's discretion: amounts are given before it and it is not estimated.",
    ),
    (
        "ato_interest_penalties.scheme_penalty",
        "Scheme penalties (TAA Sch 1 Subdiv 284-C) are not worked.",
    ),
    (
        "ato_interest_penalties.sge_before_2017",
        "The significant global entity failure to lodge penalty applies only to documents due on or after 1 July 2017.",
    ),
    (
        "ato_interest_penalties.year_mismatch",
        "year is not the financial year of the question's end date.",
    ),
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
        "The deduction is restricted by s 26-50, or the facts needed to apply its exceptions are outside the worksheet's scope.",
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
        "capital_works.begun_before_16_september_1987",
        "The capital works began before 16 September 1987, under rates and rules of ITAA 1997 s 43-215 the worksheet does not take.",
    ),
    (
        "capital_works.four_percent_before_27_february_1992",
        "The 4% manner was given for capital works begun before 27 February 1992, which have only the 2.5% rate (ITAA 1997 s 43-215 step 1(b)).",
    ),
    (
        "capital_works.precision_exceeded",
        "taxable_use has more than 20 significant digits.",
    ),
    (
        "capital_works.undeducted_above_expenditure",
        "The undeducted construction expenditure is more than the construction expenditure.",
    ),
    (
        "cgt_cost_base.below_reduced_cost_base",
        "An event would cut a parcel's reduced cost base by more than it is, without the cut to the cost base being above the cost base.",
    ),
    (
        "cgt_cost_base.company_indexation",
        "A company's parcel acquired at or before 21 September 1999, whose cost base is indexed, is touched by an event.",
    ),
    (
        "cgt_cost_base.demerger_unknown",
        "The demerger is not one whose class ruling the command carries.",
    ),
    (
        "cgt_cost_base.event_year_unsupported",
        "An event is before 1 July 2018.",
    ),
    (
        "cgt_cost_base.pre_cgt",
        "A parcel acquired before 20 September 1985 is touched by an event.",
    ),
    (
        "cgt_parcels.entity_invalid",
        "entity is not individual, trust, complying_superannuation_entity or company.",
    ),
    (
        "cgt_parcels.fact_conflict",
        "Parcels or disposals contradict each other or the income year: a name used twice, a parcel of another security or acquired after the disposal or the year, selected units that do not add up, or a company's discount gains.",
    ),
    (
        "cgt_parcels.fact_invalid",
        "A parcel's or disposal's fact has the wrong type or is out of range.",
    ),
    (
        "cgt_parcels.fact_missing",
        "A parcel or disposal leaves out a fact it needs.",
    ),
    (
        "cgt_parcels.identification_invalid",
        "identification is not first_in_first_out or selected.",
    ),
    (
        "cgt_parcels.indexation_or_pre_cgt",
        "A parcel disposed of in the year was acquired at or before 21 September 1999, so indexation or the pre-CGT rule may apply.",
    ),
    (
        "cgt_parcels.list_invalid",
        "parcels or disposals is not a list of 1 to 1,000 objects.",
    ),
    (
        "cgt_parcels.units_not_held",
        "A disposal takes more units than the parcels acquired by then still hold.",
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
        "daily_casual_withholding.foreign_resident_tfn",
        "A foreign resident who quoted a TFN was given: the tax table for daily and casual workers gives weekly foreign resident rates with no step from daily earnings to them.",
    ),
    (
        "daily_casual_withholding.offsets_exceed_withholding",
        "The daily value of the tax offsets exceeds the daily amount withheld before them.",
    ),
    (
        "daily_casual_withholding.tax_offsets_not_allowed",
        "Tax offsets were given for a payee who has not provided a TFN.",
    ),
    (
        "daily_casual_withholding.threshold_not_claimed",
        "A payee who quoted a TFN has not claimed the tax-free threshold, which the tax table for daily and casual workers requires.",
    ),
    (
        "dasp_tax.payment_nil",
        "Every component of the departing Australia superannuation payment is nil.",
    ),
    (
        "dasp_tax.unclaimed_money_rollover",
        "A roll-over under s 20H(2), (2AA) or (2A) of the Superannuation (Unclaimed Money and Lost Members) Act 1999 with an element untaxed in the fund was given; s 5(2) is outside the command.",
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
        "deferred_tax.business_combination",
        "Deferred tax arising in a business combination was stated (AASB 112 paras 19 and 66 to 68).",
    ),
    (
        "deferred_tax.expected_rate_invalid",
        "expected_rate or opening_expected_rate is not base_rate_entity or standard.",
    ),
    (
        "deferred_tax.item_invalid",
        "items is not a list of 1 to 500 objects, each with a label, a class of asset or liability and 4 non-negative amounts, and no other field.",
    ),
    (
        "deferred_tax.offsets_above_tax",
        "tax_offsets is more than current tax before offsets.",
    ),
    (
        "deferred_tax.opening_rate_unscheduled",
        "2020-21 opening balances measured at the base rate entity rate, which reverse partly at 26% and partly at 25%.",
    ),
    (
        "deferred_tax.outside_profit_or_loss",
        "Tax on items recognised in other comprehensive income or equity was stated (AASB 112 paras 61A to 63).",
    ),
    (
        "deferred_tax.rate_change_unscheduled",
        "The expected rate changed while the year's rate is not the closing rate, with opening balances to remeasure.",
    ),
    (
        "deferred_tax.tax_consolidated_group",
        "A tax-consolidated group member was stated (AASB Interpretation 1052).",
    ),
    (
        "deferred_tax.uncertain_tax_treatment",
        "An uncertain tax treatment was stated (AASB Interpretation 23).",
    ),
    (
        "deferred_tax.unrecognised_above_deductible",
        "An unrecognised amount is more than that date's deductible temporary differences and unused tax losses.",
    ),
    (
        "deferred_tax.unrecognised_used_above_opening",
        "unrecognised_used is more than opening_unrecognised_deductible.",
    ),
    (
        "defined_benefit_income_cap.first_day_outside_year",
        "first_day is not in the income year.",
    ),
    (
        "depreciation.precision_exceeded",
        "An effective life's significant digits and the taxable use's decimal places total more than 22.",
    ),
    (
        "depreciation_small_business_pool.backing_business_investment",
        "In 2020-21 a pool's s 328-210(2) amount is exactly zero and additions are above zero, so the 57.5% backing business investment rate could change the result.",
    ),
    (
        "derivation.unsupported",
        "derive was asked to explain a command whose amounts it does not derive.",
    ),
    (
        "downsizer.contribution_before_settlement",
        "downsizer_contribution_date is before downsizer_settlement_date, where s 292-102(1)(g) allows 90 days after the change of ownership.",
    ),
    (
        "downsizer.date_outside_year",
        "downsizer_contribution_date is not in the income year given.",
    ),
    (
        "downsizer.settlement_before_contract",
        "downsizer_settlement_date is before downsizer_contract_date.",
    ),
    (
        "endorsement_review.answers_invalid",
        "answers is not an object of exactly the worksheet's booleans.",
    ),
    (
        "endorsement_review.worksheet_invalid",
        "worksheet is not one of the 3 endorsement review worksheets.",
    ),
    (
        "entertainer_withholding.foreign_resident_tfn",
        "A foreign resident who quoted a TFN was given: Schedule 3 gives weekly foreign resident rates with no step from performance earnings to them.",
    ),
    (
        "entertainer_withholding.offsets_exceed_withholding",
        "The per performance value of the tax offsets exceeds the amount withheld before them.",
    ),
    (
        "entertainer_withholding.offsets_several_performances",
        "Tax offsets were given with more than one performance a day, for which Schedule 3 gives no daily value.",
    ),
    (
        "entertainer_withholding.tax_offsets_not_allowed",
        "Tax offsets were given for a performer who has not provided a TFN or has not claimed the tax-free threshold.",
    ),
    (
        "esic.amount_above_income",
        "accelerating_commercialisation_grant and rd_clawback_income together are more than assessable_income_previous_year, of which they are part.",
    ),
    (
        "esic.clawback_year_unsettled",
        "A s 355-450(1) amount is given for a 2021-22 test, where whether s 360-40(2)(b) (assessments for income years commencing on or after 1 July 2021) disregards it is unsettled.",
    ),
    (
        "esic.incorporation_after_year",
        "incorporated_year or abr_registered_year is after the income year of the test.",
    ),
    (
        "esic.innovation_items_conflict",
        "innovation_items gives both item 1 (at least 50% of total expenses) and item 3 (at least 15% but less than 50%).",
    ),
    (
        "esic.innovation_items_invalid",
        "innovation_items is not a list of distinct whole numbers from 1 to 8.",
    ),
    (
        "esic.investor_invalid",
        "investor is not individual, company, widely_held_company, esvclp, trust or partnership.",
    ),
    (
        "esic.non_cash_before_2018_19",
        "A non-cash benefit is given before 2018-19, when s 360-25(1) took the total amount paid for the shares.",
    ),
    (
        "esic.retail_limit_unsettled",
        "Without a disclosure exemption, this company's amount and the ATO's count across all companies (lifted by another ESIC's exempt offer), or the money paid and the money receivable or a non-cash benefit, fall on different sides of the $50,000 limit.",
    ),
    (
        "esic.six_year_route_before_2018_19",
        "Before 2018-19 the 6-year route of s 360-40(1)(a)(ii) counted expenses across 3 years including the current one, which this worksheet does not take.",
    ),
    (
        "esic.trust_or_partnership_unsupported",
        "The investor is a trust or partnership (or an ESVCLP before 2018-19), whose members and trustee take ss 360-15(2), (3), 360-30 and 360-35, not worked.",
    ),
    (
        "esic.year_invalid",
        "incorporated_year or abr_registered_year is not an income year such as 2024-25 or none.",
    ),
    (
        "ess_discount.acquired_before_2015",
        "An interest was acquired before 1 July 2015, when the 7-year deferral, no start-up concession and a narrower forfeiture rule applied.",
    ),
    (
        "ess_discount.dates_inconsistent",
        "An interest's dates cannot all be true together: a date before acquisition, a share's restrictions lifted without or before an exercise, a last exercise day before acquisition, or a disposal before the deferred taxing point.",
    ),
    (
        "ess_discount.employment_end_not_stated",
        "A deferred interest acquired before 1 July 2022 does not state when employment ended (or null), which was a taxing point before then (former ss 83A-115(5) and 83A-120(5)).",
    ),
    (
        "ess_discount.foreign_employment",
        "A discount relates to employment outside Australia, or the individual was a foreign or temporary resident (ss 83A-25(2) and 83A-110(2)).",
    ),
    (
        "ess_discount.income_test_invalid",
        "income_test is not null or an object of the 4 amounts s 83A-35(2)(b) adds.",
    ),
    (
        "ess_discount.interests_invalid",
        "interests is not a list of 1 to 50 objects, or an interest, its conditions, start-up facts, value or forfeiture is malformed.",
    ),
    (
        "ess_discount.salary_sacrifice_unclear",
        "In one income year salary sacrificed interests at a real risk of forfeiture, whose discount equals their market value, or forfeited interests sit beside shares relying on s 83A-105(4), whose $5,000 limit the worksheet does not settle for them together.",
    ),
    (
        "ess_discount.taxing_point_value",
        "A deferred taxing point falls in the year without the value at it, or the value is stated on another day.",
    ),
    (
        "ess_discount.valuation_unavailable",
        "The regulations' value was chosen where they do not apply: a valuation day before 1 July 2021, a right exercisable more than 15 years after acquisition, a right already exercised, or a taxing point that is a disposal.",
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
        "excess_contributions.charge_abolished",
        "An excess concessional contributions charge was asked for a year from 2021-22, when the charge no longer applies.",
    ),
    (
        "excess_contributions.date_order",
        "A date is out of the order the question's period needs.",
    ),
    (
        "excess_contributions.fact_invalid",
        "A fact is not of the form the question takes.",
    ),
    (
        "excess_contributions.question_invalid",
        "The question is not one the command answers.",
    ),
    (
        "excess_contributions.rate_unpublished",
        "A day's charge rate, or a base interest rate the proxy rate averages, is not yet published.",
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
        "fbt_car_parking.capped_contribution_invalid",
        "A capped parking contribution has missing or extra fields or exceeds the total gross taxable value.",
    ),
    (
        "fbt_car_parking.contribution_allocation_needed",
        "A combined parking contribution exceeds one benefit's value, so the amounts need capping per benefit before combining.",
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
        "fbt_ev_exemption.fact_conflict",
        "The car's dates contradict the FBT year or the plug-in hybrid facts.",
    ),
    (
        "fbt_ev_exemption.sale_within_year",
        "Luxury car tax was payable on a first retail sale within the FBT year after the car was first held and used, so the benefits before and after it differ.",
    ),
    (
        "fbt_ev_exemption.threshold_year_unsupported",
        "first_retail_sale is before the financial years whose fuel-efficient luxury car tax threshold the ATO's table lists.",
    ),
    (
        "fbt_ev_exemption.vehicle_invalid",
        "vehicle is not battery_electric, hydrogen_fuel_cell_electric, plug_in_hybrid_electric or other.",
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
        "fbt_instalment.ato_instalment_missing",
        "fbt-instalment was given neither the ATO instalment amount at label F1 nor an estimate at F2 to vary it.",
    ),
    (
        "fbt_instalment.quarter_invalid",
        "fbt-instalment was given a quarter other than 1 to 4.",
    ),
    (
        "fbt_instalment.variation_invalid",
        "fbt-instalment was given a variation without a reason code from the ATO's list or without the earlier instalments it needs, or variation facts without an estimate.",
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
        "fbt_return.employer_kind_unsupported",
        "fbt-return was given an employer_kind it does not compose, such as a government body under FBTAA s 57A(2) or an organisation partly a public benevolent institution and partly rebatable.",
    ),
    (
        "fbt_return.entertainment_invalid",
        "fbt-return was given a benefit's entertainment marker that is not salary_packaged or not_salary_packaged, or that contradicts its worksheet or whether it is excluded.",
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
        "fbt_return.nfp_exempt_car_unsupported",
        "fbt-return was given an exempt electric car for an FBTAA s 57A employer, whose place under the caps no ATO example shows.",
    ),
    (
        "fbt_return.payroll_tax_unsupported",
        "fbt-return was asked for a payroll tax gross-up, which each state and territory sets and it does not work out.",
    ),
    (
        "fbt_return.rebatable_days_invalid",
        "fbt-return was given rebatable_days and employer_days that are not 2 whole numbers, rebatable_days at most employer_days and employer_days at most the days in the FBT year.",
    ),
    (
        "fbt_tax_exempt_body_entertainment.heads_invalid",
        "The employees and associates entertained are not a JSON integer from 0 to everyone entertained, or everyone entertained is not from 1 to 1,000,000.",
    ),
    (
        "fhss.basis_invalid",
        "fhss was given both contributions and determination, or neither.",
    ),
    (
        "fhss.contribution_kind_invalid",
        "A contribution's kind is not employer_voluntary or personal, the kinds eligible to be released (TAA 1953 Sch 1 s 138-35(2)).",
    ),
    (
        "fhss.contribution_out_of_period",
        "A contribution is dated before 1 July 2017 or after the determination day.",
    ),
    (
        "fhss.contributions_invalid",
        "contributions is not a list of 1 to 10000 objects.",
    ),
    (
        "fhss.deductions_invalid",
        "personal_deductions is not a list of financial years and amounts, gives a year twice or deducts more than that year's personal contributions.",
    ),
    (
        "fhss.determination_invalid",
        "determination is not an object of concessional, non_concessional and associated_earnings.",
    ),
    (
        "fhss.determination_requested_invalid",
        "determination_requested is before 1 July 2018 or after the determination day.",
    ),
    (
        "fhss.rate_unpublished",
        "determination_date is after 31 December 2026, the last quarter with a published SIC rate.",
    ),
    (
        "fhss.request_exceeds_maximum",
        "amount_requested is more than the FHSS maximum release amount (TAA 1953 Sch 1 s 131-10(1), table item 4).",
    ),
    (
        "fhss.year_mismatch",
        "determination_date is not in the financial year given as year.",
    ),
    (
        "first_home_owner_grant.below_grant",
        "The value is below the grant where the source read gives no lesser-of-consideration rule, or for a home built, whose building contract price must be at least the grant.",
    ),
    (
        "first_home_owner_grant.date_invalid",
        "contract_date is not a YYYY-MM-DD date in the financial year given as year.",
    ),
    (
        "first_home_owner_grant.date_uncovered",
        "The contract date is before 1 July 2025 or after 22 November 2026, the day the grant rules were read.",
    ),
    (
        "first_home_owner_grant.fact_needed",
        "A fact the transaction needs is not stated: the building contract price for a home built, or the Northern Territory or Tasmanian conditions.",
    ),
    (
        "first_home_owner_grant.facts_invalid",
        "facts is not an object of booleans.",
    ),
    (
        "first_home_owner_grant.facts_not_stated",
        "One of the 4 eligibility facts every grant needs is not stated.",
    ),
    (
        "first_home_owner_grant.home_invalid",
        "home is not bought, built or established.",
    ),
    (
        "first_home_owner_grant.jurisdiction_invalid",
        "jurisdiction is not one of the 8 states and territories.",
    ),
    (
        "first_home_owner_grant.location_needed",
        "A Western Australian home's location, north_of_26th, is not stated.",
    ),
    (
        "first_home_owner_grant.value_basis",
        "A Victorian home built over $750,000 in total: the Act caps the building work's consideration, which the total does not show.",
    ),
    (
        "first_home_owner_grant.value_invalid",
        "total_value or cap_value is not whole dollars from 1 to 1,000,000,000.",
    ),
    (
        "foreign_income_conversion.average_period",
        "An average rate's period is longer than 12 months, runs backwards or does not contain the day the amount is translated on.",
    ),
    (
        "foreign_income_conversion.basis_invalid",
        "basis is not specific or average.",
    ),
    (
        "foreign_income_conversion.currency_invalid",
        "currency is not a 3-letter code other than AUD.",
    ),
    (
        "foreign_income_conversion.items_invalid",
        "items is not a list of 1 to 200 objects.",
    ),
    (
        "foreign_income_conversion.kind_invalid",
        "kind is not one of the amounts the worksheet translates.",
    ),
    (
        "foreign_income_conversion.rate_date",
        "A specific rate is for a day other than the one the amount's translation rule names.",
    ),
    (
        "foreign_income_conversion.text_invalid",
        "rate_source is not text of 1 to 200 characters.",
    ),
    (
        "foreign_income_tax_offset.limit_method_invalid",
        "limit_method is not default, stated or worked.",
    ),
    (
        "foreign_income_tax_offset.residency_periods_invalid",
        "residency_periods is not a list of 1 to 12 periods inside the income year, each running forwards.",
    ),
    (
        "foreign_income_tax_offset.worked_year",
        "The worked limit was asked for a year resident-tax has no rates for.",
    ),
    (
        "foreign_pension_upp.commuted",
        "Part of the annuity has been commuted (ITAA 1936 s 27H(3A)).",
    ),
    (
        "foreign_pension_upp.method_invalid",
        "method is not formula, uk_state_pension or dutch_svb_fallback.",
    ),
    (
        "foreign_pension_upp.method_year_unsupported",
        "A shortcut method was asked for a year other than 2025-26.",
    ),
    (
        "foreign_pension_upp.part_year",
        "The annuity was not derived for the whole income year, which s 27H(2) does not apportion.",
    ),
    (
        "foreign_pension_upp.relevant_number_invalid",
        "relevant_number is not a term of years or a life expectation factor above 0.",
    ),
    (
        "foreign_pension_upp.residual_exceeds_price",
        "The residual capital value exceeds the undeducted purchase price.",
    ),
    (
        "foreign_pension_upp.share_invalid",
        "total_annuity_derived is less than annuity_derived or under $1.",
    ),
    (
        "foreign_pension_upp.uk_category_invalid",
        "uk_category is not a, b_widow, c or d.",
    ),
    (
        "foreign_purchaser_surcharge.absentee_trust_unsupported",
        "A trust that is a foreign owner was given in vic, whose absentee surcharge turns on its beneficiaries' interests.",
    ),
    (
        "foreign_purchaser_surcharge.dutiable_value_nil",
        "The dutiable value is nil.",
    ),
    (
        "foreign_purchaser_surcharge.jurisdiction_invalid",
        "jurisdiction is not nsw, vic, qld, wa, sa, tas, act or nt.",
    ),
    (
        "foreign_purchaser_surcharge.owner_invalid",
        "owner is not individual, company or trust.",
    ),
    (
        "foreign_purchaser_surcharge.use_invalid",
        "property_use is not residential, primary_production or other.",
    ),
    (
        "foreign_resident_withholding.asset_unsupported",
        "The asset is not taxable Australian real property or a company title interest.",
    ),
    (
        "foreign_resident_withholding.date_outside_year",
        "acquisition_date is outside the income year given.",
    ),
    (
        "foreign_resident_withholding.exclusion_invalid",
        "exclusion is not null or one of the s 14-215 paragraphs the command names.",
    ),
    (
        "foreign_resident_withholding.option_exceeds_price",
        "option_payments exceed the first element of the cost base.",
    ),
    (
        "foreign_resident_withholding.payment_unsupported",
        "The payment is not capital_gains: the s 14-205 earnout amount, Subdivisions 12-F and 12-H and s 12-315 withholding are not built.",
    ),
    (
        "foreign_resident_withholding.variation_invalid",
        "A varied rate exceeds the full rate for the acquisition date.",
    ),
    (
        "foreign_resident_withholding.variation_maximum_unsettled",
        "A notice's maximum sale price lies between the base and the GST inclusive sale_price, so whether it applies turns on the maximum's GST basis.",
    ),
    (
        "foreign_resident_withholding.vendors_invalid",
        "vendors is not a list of 1 to 50 vendor objects with an interest above nil and a known document.",
    ),
    (
        "franking_account.distribution_invalid",
        "A distribution is not an object of date (in the income year, in date order), amount above nil and franking_credit, with optional permitted_percentage (from 0 to 100, after the first in its franking period) and statement_after_distribution (a private company's only), or distributions is not a list of at most 10,000.",
    ),
    (
        "franking_account.event_invalid",
        "An event is not an object of date (in the income year, or for an income tax refund the 3 months after it), a type the command takes and an amount above nil, with rd_offset_part for an income tax refund only, or events is not a list of at most 10,000.",
    ),
    (
        "franking_account.facts_conflict",
        "A private company was said to be exempt from the benchmark rule, which s 203-20 gives only to a listed public company and its 100% subsidiaries.",
    ),
    (
        "franking_account.income_tax_invalid",
        "income_tax is not an object of before_fdt_offset and offset_brought_forward in AUD, with earlier_year_liability for a private company only.",
    ),
    (
        "franking_account.income_tax_needed",
        "The offset reduction would apply to a private company, and s 205-70(5) needs its income tax facts to decide whether it does.",
    ),
    (
        "franking_account.last_benchmark_invalid",
        "last_benchmark is not null or an object of percentage (0 to 100) and periods_since (a whole number from 1).",
    ),
    (
        "franking_account.rate_invalid",
        "The corporate tax rate for imputation purposes is not 0.25 or 0.30.",
    ),
    (
        "fuel_tax_credits.date_outside_year",
        "A line's acquisition day is outside the income year given.",
    ),
    (
        "fuel_tax_credits.fact_conflict",
        "The lines' facts contradict each other, the tax period or the return's day, or a basic method line's public road litres exceed its diesel.",
    ),
    (
        "fuel_tax_credits.fuel_invalid",
        "A line's fuel is not one of the fuels the ATO's rate table lists.",
    ),
    (
        "fuel_tax_credits.lines_invalid",
        "lines is not a list of 1 to 1,000 line objects.",
    ),
    (
        "fuel_tax_credits.method_invalid",
        "A line's method is not basic_heavy_vehicle or litres_from_cost.",
    ),
    (
        "fuel_tax_credits.period_invalid",
        "rate_at_period_end is not an object with the tax period's start and end.",
    ),
    (
        "fuel_tax_credits.quantity_invalid",
        "A line's quantity, litres, kilometres, weight or consumption rate is not a decimal string in range.",
    ),
    (
        "fuel_tax_credits.rate_unpublished",
        "A line's fuel was acquired, or a Method 2 tax period ends, after the last day the bundled ATO rates cover.",
    ),
    (
        "fuel_tax_credits.simplified_method_unavailable",
        "A simplified method was asked for where its guideline does not allow it.",
    ),
    (
        "fuel_tax_credits.use_invalid",
        "A line's use is not heavy_vehicle_public_road, other_business or light_vehicle_public_road.",
    ),
    (
        "government_super_amounts.fact_conflict",
        "Facts that cannot all be true: eligible income above total income before business deductions, or a spouse's assessable FHSS released amount or excess concessional contributions above the spouse's assessable income.",
    ),
    (
        "government_super_amounts.fact_not_supplied",
        "A total super balance or non-concessional cap fact given as null would decide an amount.",
    ),
    (
        "government_super_amounts.no_total_income",
        "Total income is nil, so the 10% eligible income test has no share to measure, and it would decide an amount.",
    ),
    (
        "government_super_amounts.temporary_visa",
        "A temporary visa was held in the income year; the New Zealand citizen and prescribed visa exceptions in s 6(1)(f) are not worked out.",
    ),
    (
        "gst_annual_apportionment.car",
        "The acquisition is of a car, whose credit s 131-50 limits to the s 69-10 car limit amount.",
    ),
    (
        "gst_annual_apportionment.election_status_unsettled",
        "election_made is true but the entity test of s 131-5(1)(a) fails, and whether the election ceased under s 131-20(1)(c) or (d) is not worked out.",
    ),
    (
        "gst_annual_apportionment.fact_conflict",
        "creditable_purpose is above non_input_taxed_extent, or a small business entity carries on no business.",
    ),
    (
        "gst_annual_apportionment.not_partly_creditable",
        "The acquisition is not partly creditable: creditable_purpose or consideration_extent is nil, or both are 1.",
    ),
    (
        "gst_annual_apportionment.reduced_credit_acquisition",
        "The acquisition is to any extent a reduced credit acquisition, which s 131-40(5) excludes.",
    ),
    (
        "gst_bad_debt.accounting_basis_changed",
        "A recovery or payment is stated on a cash basis, so the accounting basis changed after the earlier adjustment, which Division 159 governs.",
    ),
    (
        "gst_bad_debt.annual_apportionment",
        "The recipient's credit was worked out under Division 131, where s 136-10(3) turns on the tax period of the s 131-55 adjustment.",
    ),
    (
        "gst_bad_debt.dates_invalid",
        "as_at is before overdue_from.",
    ),
    (
        "gst_bad_debt.fact_conflict",
        "A taxable proportion, extent of creditable purpose or extent of consideration of nil contradicts a taxable supply or creditable acquisition.",
    ),
    (
        "gst_bad_debt.fact_invalid",
        "party, event or special_rule is not one of its allowed values.",
    ),
    (
        "gst_bad_debt.reduced_credit_acquisition",
        "The recipient's acquisition is a reduced credit acquisition, which s 136-10(2) reduces by the Division 70 percentage.",
    ),
    (
        "gst_bad_debt.special_rule",
        "A special rule displaces or changes Division 21 for the debt: Subdivision 136-B, Division 126 (gambling), an assigned debt, GST Transition Act s 15G or Division 58 (representatives).",
    ),
    (
        "gst_creditable_purpose_change.basis_missing",
        "application_basis is not text of 1 to 200 characters stating the method the applications rest on.",
    ),
    (
        "gst_creditable_purpose_change.last_period_unsettled",
        "ended_on falls in a tax period that itself ends on 30 June, where s 129-25(1)'s next tax period ending on 30 June is not settled.",
    ),
    (
        "gst_creditable_purpose_change.period_end_invalid",
        "attributed_period_end is not the last day of a month.",
    ),
    (
        "gst_creditable_purpose_change.tax_period_unsupported",
        "tax_period_months is not 1, 3 or 12.",
    ),
    (
        "gst_financial_acquisitions_threshold.fact_conflict",
        "The financial acquisitions' credits are more than the total credits that include them.",
    ),
    (
        "gst_going_concern.fact_invalid",
        "recipient_registration or written_agreement is not one of its allowed values.",
    ),
    (
        "gst_going_concern.farm_land",
        "The supply is of farm land for farming, which Subdivision 38-O governs.",
    ),
    (
        "gst_going_concern.part_outside_arrangement",
        "Part of the supply is outside the arrangement, so s 9-80 apportions it.",
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
        "Item 2 of s 75-10(3) overlaps with item 3 or 4 for the facts given, and gst-margin-scheme does not choose between their valuation days.",
    ),
    (
        "gst_margin_scheme.valuation_unavailable",
        "An approved valuation was given for an interest acquired on or after 1 July 2000.",
    ),
    (
        "gst_second_hand_goods.divided_for_resupply",
        "The goods are to be divided for re-supply, where Subdivision 66-B may apply instead.",
    ),
    (
        "gst_second_hand_goods.fact_invalid",
        "supply_to_you is not one of its allowed values.",
    ),
    (
        "gst_second_hand_goods.leased_before_sale",
        "The goods are leased before they are sold, where GSTD 2012/6 attributes the credit in parts.",
    ),
    (
        "gst_second_hand_goods.valuable_metal",
        "The goods consist to any extent of valuable metal, which s 195-1 excludes from second-hand goods unless incidental.",
    ),
    (
        "help_indexation.date_outside_period",
        "A debt is dated outside its half-year, or a voluntary repayment outside 1 June of the preceding financial year to 31 May.",
    ),
    (
        "help_indexation.debt_missing",
        "previous_accumulated_debt was not supplied; the command never estimates a balance.",
    ),
    (
        "help_indexation.entries_invalid",
        "A debt or repayment list is not a list of at most 100 objects, or an entry's date or group is malformed.",
    ),
    (
        "help_indexation.location_preferred",
        "A location-preferred HELP debtor's reduced indexation or reduction applies (HESA ss 140-10(1)(a), 142-10, 142-15, 144-5 and 144-10), which the command does not take.",
    ),
    (
        "help_indexation.reduction_already_applied",
        "Amounts the command reduces by 20% (every amount for 2024-25, and debts incurred from 1 January to 1 June 2025) were given already reduced; it applies HESA ss 140-26 and 137-19A itself.",
    ),
    (
        "help_indexation.repayments_exceed_debt",
        "The repayments exceed the debts they reduce under HESA s 140-5(1) or s 140-25(1).",
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
        "horticulture_shearing_withholding.labour_mobility_scheme",
        "The worker is employed under the Seasonal Worker Programme, Pacific Labour Scheme or Pacific Australia Labour Mobility scheme, to which Schedule 2 does not apply.",
    ),
    (
        "horticulture_shearing_withholding.threshold_not_claimed",
        "A worker who quoted a TFN has not claimed the tax-free threshold, which Schedule 2 requires.",
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
        "input.count_invalid",
        "A count of performances, instalments or part-year residency months is not an integer in the range the input allows.",
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
        "A field the worksheet or command does not take, or does not take beside the other inputs given, was given (fbt-loan, fbt-meal-entertainment, fbt-car-parking, fbt-debt-waiver, fbt-expense-payment, fbt-property, fbt-residual, fbt-housing, fbt-lafha, fbt-board, fbt-tax-exempt-body-entertainment, fbt-entertainment-classifier, fbt-meal-entertainment-register and its entries, fbt-minor-benefit-check, aml-cdd-review, ppsr-timing, fhss and its contributions, government-super-amounts, personal_deductions and determination, division-296, depreciation, fuel-tax-credits and its lines, alcohol-excise and its lines, asic-fees, foreign-resident-tax, medicare-levy, medicare-levy-surcharge, payg-withholding, payg-gross-up, termination-payment, etp-tax-free-component, unused-leave-payment, the Schedule 5 methods, working-holiday-maker-withholding, study-loan-withholding, seniors-pensioners-withholding, seniors-pensioners-tax-offset, lump-sum-arrears-tax-offset, hire-purchase-schedule, chattel-mortgage-schedule, loan-transactions and its transactions and rate_changes, borrowing-expenses-schedule and its loans, business-capital-expenditure and its items, balancing-adjustment, its years and car_limit, trade-in, franking-account and its events, distributions and income_tax, foreign-purchaser-surcharge, dasp-tax, super-benefit-tax, super-withholding, gst-margin-scheme, wine-equalisation-tax and its dealings, luxury-car-tax and its lines, foreign-resident-withholding and its vendors, payroll-tax, land-tax, vic-property-taxes and its parcels, workers-compensation-premium, motor-vehicle-duty, insurance-duty, transfer-duty, cgt-parcels, its parcels and disposals, cgt-cost-base-adjustments, its parcels and events, fbt-ev-exemption-check, fbt-instalment, sb-cgt-concessions, small-business-income-offset, ess-discount, pcg-2021-4, residency-tests, lodgment-required, trading-stock and its items, prepayments, depreciation-low-value-pool, gst-creditable-purpose-change, gst-annual-apportionment, gst-financial-acquisitions-threshold, gst-going-concern, gst-second-hand-goods, gst-bad-debt, depreciation-small-business-pool, capital-works, instant-asset-write-off, non-commercial-losses, first-home-owner-grant, psi-test, minors-income, smsf-tax, payg-instalment, beneficiary-tax-offset, invalid-carer-tax-offset, zone-tax-offset and its dependants, net-medical-expenses-tax-offset, work-expenses, notice-period, ato-interest-penalties and its payments and parts, private-health-insurance-rebate, rental-property and its loans, capital_works and depreciating_assets, horticulture-shearing-withholding, upward-variation-withholding, daily-casual-withholding, no-abn-withholding, entertainer-withholding, return-to-work-withholding, annuity-withholding, voluntary-agreement-withholding, rd-tax-offset, esic, deferred-tax, leave-provisions, long-service-leave, foreign-income-conversion, foreign-income-tax-offset, transfer-balance and its events, excess-contributions, help-indexation and its debts and repayments, defined-benefit-income-cap, foreign-pension-upp and the inputs of each method, scholarship-exemption, super-crystallisation, endorsement-review, contribution-caps given a carry-forward input, derive, personal-tax-estimate, dgr-check, income-tests and fbt-return only).",
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
    (
        "input.percentage_invalid",
        "A percentage is outside 0 to 100.",
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
        "instant_asset_write_off.acquired_after_first_use",
        "The asset's acquisition date is after its first use.",
    ),
    (
        "instant_asset_write_off.budget_time_day",
        "The asset was acquired on the day of a 7.30 pm budget time the result turns on, and the command takes dates only.",
    ),
    (
        "instant_asset_write_off.entity_invalid",
        "entity is not small_business_simplified, medium_business or other.",
    ),
    (
        "instant_asset_write_off.first_use_outside_year",
        "The asset's first use for a taxable purpose is outside the income year.",
    ),
    (
        "instant_asset_write_off.precision_exceeded",
        "taxable_use has more than 26 significant digits.",
    ),
    (
        "instant_asset_write_off.temporary_full_expensing",
        "Temporary full expensing (IT(TP)A 1997 Subdivision 40-BB) may take the asset, which displaces s 40-82, and the command does not decide whether it does.",
    ),
    (
        "insurance_duty.class_invalid",
        "class is not one the jurisdiction's insurance duty has (nsw type_a, type_b or type_c; vic general or business; qld class_1, class_2 or ctp; general elsewhere).",
    ),
    (
        "insurance_duty.date_outside_year",
        "effected_date is not in the year asked for.",
    ),
    (
        "insurance_duty.jurisdiction_invalid",
        "jurisdiction is not nsw, vic, qld, wa, sa, tas, act or nt.",
    ),
    (
        "insurance_duty.term_unsettled",
        "A Northern Territory policy issued or renewed for more than one year, whose duty under Sch 1 cl 6(3)(b)(ii) is unsettled for a premium for the whole term.",
    ),
    (
        "invalid_carer_tax_offset.dependant_invalid",
        "dependant is not spouse or other.",
    ),
    (
        "invalid_carer_tax_offset.ftb_shared_care",
        "You or your spouse were eligible for FTB Part B at a shared-care rate, whose reduction (ITAA 1997 s 61-35) needs rates this worksheet does not work out.",
    ),
    (
        "invalid_carer_tax_offset.rate_unpublished",
        "The year is 2026-27, whose indexed offset amount and FTB Part B limit had not been published.",
    ),
    (
        "invalid_carer_tax_offset.shared_maintenance",
        "Another person also maintained the dependant: ITAA 1997 s 61-40(1)(b) and the ATO's worksheets order the apportionment and the income reduction differently.",
    ),
    (
        "invalid_carer_tax_offset.successive_spouses",
        "The person had a different spouse during different parts of the year, whose incomes ITAA 1997 s 61-20(3) adds separately.",
    ),
    (
        "land_tax.absentee_trust_unsupported",
        "A Victorian trust was given as an absentee owner: s 46IA charges a fixed or unit trust's surcharge on its absentee beneficiaries' share, which this worksheet does not take.",
    ),
    (
        "land_tax.jurisdiction_invalid",
        "jurisdiction is not nsw, vic, qld, wa, sa, tas or act; the Northern Territory has no land tax.",
    ),
    (
        "land_tax.metropolitan_region_invalid",
        "metropolitan_region is not all, none or part.",
    ),
    (
        "land_tax.metropolitan_region_part",
        "The land is partly in the Perth metropolitan region, whose improvement tax on that part alone is not worked.",
    ),
    (
        "land_tax.rounding_unsettled",
        "Tasmania's s 7 rounding to 10 cents gives a different total applied to the land tax and the foreign investor land tax apart than to their sum, and which it requires is unsettled.",
    ),
    (
        "land_tax.surcharge_unavailable",
        "A foreign or absentee owner was given in a jurisdiction whose surcharge this worksheet does not hold (wa, sa).",
    ),
    (
        "land_tax.surcharge_value_above_taxable_value",
        "A Tasmanian surcharge value is more than the taxable value, though the land it applies to is part of the owner's taxable land.",
    ),
    (
        "land_tax.trust_rates_unavailable",
        "Trust rates were asked for in a jurisdiction whose trust rates this worksheet does not hold (wa, tas, act).",
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
        "leave_provisions.defined_benefit_plan",
        "A defined benefit plan or other post-employment benefit was stated (AASB 119 paras 26 to 152).",
    ),
    (
        "leave_provisions.line_invalid",
        "lines is not a list of 1 to 500 objects, each annual or long service leave with its hours, rate, probability and, for a line not settled wholly within 12 months, 1 to 50 whole years to settlement, salary growth, discount rate and right to defer, and no other field.",
    ),
    (
        "leave_provisions.plan_assets",
        "Plan assets or a reimbursement right for other long-term benefits were stated (AASB 119 paras 113 to 119 and 155).",
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
        "lodgment_required.child_support_parent",
        "This instrument does not require a return, and the parent may be required to lodge under the separate child support instrument, which the worksheet does not apply.",
    ),
    (
        "lodgment_required.group_invalid",
        "under_18, study_loan, working_holiday_maker, listed_payment or sapto is not null or an object of its facts.",
    ),
    (
        "lodgment_required.instrument_not_made",
        "No lodgment instrument for the year was on the Federal Register of Legislation when the worksheet was checked.",
    ),
    (
        "lodgment_required.residency_invalid",
        "residency is not whole_year, part_year with 1 to 12 months, or none with taxable_australian_income.",
    ),
    (
        "lodgment_required.triggers_invalid",
        "triggers is not a list of the conditions the worksheet takes.",
    ),
    (
        "long_service_leave.before_current_text",
        "The last day is before the Act's text this worksheet read began (New South Wales 25 March 2021, Victoria 1 July 2021, Western Australia 20 June 2022).",
    ),
    (
        "long_service_leave.end_before_start",
        "The last day is before the first day of service.",
    ),
    (
        "long_service_leave.excluded_invalid",
        "excluded_days is not a whole number from 0 to 36,500 fewer than the days of service.",
    ),
    (
        "long_service_leave.hours_invalid",
        "hours_per_week is not above 0 to 168 to at most 2 decimal places.",
    ),
    (
        "long_service_leave.jurisdiction_invalid",
        "jurisdiction is not nsw, vic, qld, wa, sa, tas, act or nt.",
    ),
    (
        "long_service_leave.pay_invalid",
        "weekly_pay is more than $1,000,000.",
    ),
    (
        "long_service_leave.reason_invalid",
        "reason is not one of the 10 endings the worksheet takes.",
    ),
    (
        "long_service_leave.start_too_early",
        "The first day of service is before 23 June 1990.",
    ),
    (
        "long_service_leave.status_invalid",
        "status is not employed or ends.",
    ),
    (
        "long_service_leave.taken_above_weeks",
        "Leave taken or paid is more than the exact weeks of the entitlement.",
    ),
    (
        "long_service_leave.taken_invalid",
        "leave_taken is not weeks from 0 to 1,000 to at most 4 decimal places.",
    ),
    (
        "long_service_leave.unsettled",
        "The Act does not settle the ending before its full qualifying period: a fixed term ending in New South Wales, Tasmania, the ACT or the Northern Territory, or a death under 10 years in the Northern Territory.",
    ),
    (
        "long_service_leave.year_mismatch",
        "The last day does not fall in the year given.",
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
        "minors_income.excepted_income_invalid",
        "excepted_income is not a list of at most 50 objects, each with a ground ITAA 1936 s 102AE(2) lists and an amount.",
    ),
    (
        "minors_income.excepted_person_invalid",
        "excepted_person is not none or a ground of ITAA 1936 s 102AC(2).",
    ),
    (
        "minors_income.facts_inconsistent",
        "Taxable income exceeds the assessable income given, or eligible taxable income exceeds taxable income (a loss or deduction against excepted income, which the worksheet does not take).",
    ),
    (
        "minors_income.residency_invalid",
        "residency is not resident or foreign_resident.",
    ),
    (
        "minors_income.special_income_component",
        "Taxable income includes a special income component, which Income Tax Rates Act 1986 Schedule 11 clause 3 taxes by a formula the worksheet does not take.",
    ),
    (
        "minors_income.trust_shares_invalid",
        "trust_shares is not a list of at most 50 objects, each with amount and division_6aa_part, the part no more than the amount.",
    ),
    (
        "motor_vehicle_duty.date_outside_year",
        "duty_date is not in the year asked for.",
    ),
    (
        "motor_vehicle_duty.emissions_category_invalid",
        "An ACT emissions_category is not aaa, aa, a, b, c, d or non_rated.",
    ),
    (
        "motor_vehicle_duty.engine_invalid",
        "A Queensland engine is not hybrid_or_electric, up_to_4_cylinders, 5_or_6_cylinders or 7_or_more_cylinders.",
    ),
    (
        "motor_vehicle_duty.jurisdiction_invalid",
        "jurisdiction is not nsw, vic, qld, wa, sa, tas, act or nt.",
    ),
    (
        "motor_vehicle_duty.vehicle_class_unsupported",
        "vehicle_class is not passenger; other classes' rates are not worked.",
    ),
    (
        "no_abn_withholding.gst_above_payment",
        "The GST included is more than the payment.",
    ),
    (
        "no_abn_withholding.total_below_payment",
        "The total of the payments for the supply, disregarding GST, is less than this payment without its GST.",
    ),
    (
        "non_commercial_losses.activity_invalid",
        "activity is not other, primary_production or professional_arts.",
    ),
    (
        "non_commercial_losses.earlier_years_invalid",
        "earlier_years is not a list of 4 entries, each null or an object with assessable_income and deductions.",
    ),
    (
        "notice_period.age_unsettled",
        "The employee is 45 in completed years at the end of the notice day, where the readings of 'over 45 years old' in s 117(3)(b) differ.",
    ),
    (
        "notice_period.before_current_text",
        "The notice day is before 27 March 2021, when s 117(4) began; the earlier text was not read.",
    ),
    (
        "notice_period.birth_after_start",
        "The date of birth is not before the first day of service.",
    ),
    (
        "notice_period.excluded_invalid",
        "excluded_days is not a whole number from 0 to 36,500 fewer than the days of service.",
    ),
    (
        "notice_period.exclusion_invalid",
        "exclusion is not none or one of the s 123 exclusions the worksheet takes.",
    ),
    (
        "notice_period.hours_invalid",
        "weekly_hours is not above 0 to 168 to at most 2 decimal places.",
    ),
    (
        "notice_period.start_after_notice",
        "The first day of service is after the notice day.",
    ),
    (
        "notice_period.year_mismatch",
        "The notice day does not fall in the year given.",
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
        "payg_instalment.credits_before_third",
        "Credits were claimed before the second instalment, when none can exist (s 45-410(2) and s 45-215(1)(b) of Schedule 1 to the TAA 1953).",
    ),
    (
        "payg_instalment.credits_exceed_earlier",
        "credits_claimed is more than earlier_instalments.",
    ),
    (
        "payg_instalment.first_has_earlier",
        "The first instalment of the year was given earlier instalments.",
    ),
    (
        "payg_instalment.instalment_number_invalid",
        "instalment_number is not a JSON integer from 1 to 4.",
    ),
    (
        "payg_instalment.method_invalid",
        "method is not amount, varied_amount, rate or varied_rate.",
    ),
    (
        "payg_instalment.rate_invalid",
        "instalment_rate or previous_rate is not a per cent from 0 to 100 with at most 2 decimal places.",
    ),
    (
        "payg_instalment.tax_exceeds_income",
        "estimated_tax is more than estimated_instalment_income, a varied rate over 100%.",
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
        "pcg_2021_4.benchmark_nil",
        "Factor 3 was chosen with a nil commercial benchmark.",
    ),
    (
        "pcg_2021_4.entitlement_nil",
        "The profit entitlement of the IPP and associates from the whole of firm group is nil.",
    ),
    (
        "pcg_2021_4.gateway_failed",
        "The arrangement fails Gateway 1 or 2, so PCG 2021/4 gives it no risk score (paragraphs 10 to 12).",
    ),
    (
        "pcg_2021_4.recipient_kind_unsupported",
        "A recipient is not an individual, a base rate entity company or another company, whose tax the worksheet works out.",
    ),
    (
        "pcg_2021_4.recipients_invalid",
        "recipients is not a list of 1 to 6 objects with the IPP, an individual, first.",
    ),
    (
        "pcg_2021_4.returned_above_entitlement",
        "The amount the IPP returns is more than the profit entitlement.",
    ),
    (
        "pcg_2021_4.tax_above_entitlement",
        "The tax counted is more than the profit entitlement it is paid on.",
    ),
    (
        "period.unsupported",
        "The worksheet has no rates for the period.",
    ),
    (
        "ppsr_timing.date_outside_year",
        "The registration, possession or attachment, or agreement time is outside the income year given.",
    ),
    (
        "ppsr_timing.date_unsettled",
        "A 7 or 25 year anniversary, or the day 6 months before a critical time, does not exist in the calendar (29 February, or a month too short), and the Act read gives no rule for it.",
    ),
    (
        "ppsr_timing.days_invalid",
        "excluded_days or place_holidays is not a list of YYYY-MM-DD dates.",
    ),
    (
        "ppsr_timing.fact_invalid",
        "question or collateral is not one of the values ppsr-timing takes.",
    ),
    (
        "ppsr_timing.foreign_law_unsupported",
        "The security agreement came into force under a foreign law; Corporations Act s 588FL(2)(b)(iii)'s 56 day time is not computed.",
    ),
    (
        "ppsr_timing.holidays_incomplete",
        "A business day count spans 25 December or 1 January on a weekday that the holiday list does not hold.",
    ),
    (
        "ppsr_timing.law_unread",
        "The critical time is outside 1 July 2025 to 30 June 2027, the years whose Corporations Act text was read.",
    ),
    (
        "ppsr_timing.time_invalid",
        "A time is not YYYY-MM-DDTHH:MM.",
    ),
    (
        "ppsr_timing.time_unresolved",
        "An input or calculated time falls in the skipped or repeated ACT hour at a daylight saving change.",
    ),
    (
        "prepayments.agreement_before_cutoff",
        "The agreement was entered into on or before 11 November 1999, when a pre-RBT obligation (ITAA 1936 s 82KZL(1)) or a pre-existing obligation (s 82KZME(8)) may apply.",
    ),
    (
        "prepayments.choice_unavailable",
        "chose_82kzmd is true for a taxpayer that is neither a small business entity nor covered by ITAA 1936 s 82KZMA(2A), or for expenditure s 82KZMD cannot apply to (ss 82KZMA(3) and (4)).",
    ),
    (
        "prepayments.dates_invalid",
        "incurred is outside the income year, service_end is before service_start, or agreement_date is after incurred.",
    ),
    (
        "prepayments.exclusion_invalid",
        "exclusion is not none, law_or_court_order or contract_of_service.",
    ),
    (
        "prepayments.tax_shelter_conflict",
        "tax_shelter is true for excluded expenditure or a thing done by the end of the expenditure year, which s 82KZME cannot cover (ss 82KZME(2)(b) and (7)).",
    ),
    (
        "prepayments.taxpayer_invalid",
        "taxpayer is not small_business_entity, medium_business_entity or other.",
    ),
    (
        "private_health_insurance_rebate.lhc_loading",
        "A statement line gave a lifetime health cover loading, which is outside the rebate (PHI Act s 22-15(6)) and is not worked out.",
    ),
    (
        "private_health_insurance_rebate.lines_invalid",
        "The statement lines are not a list of 1 to 100 objects with premiums_eligible, rebate_received and a benefit code of 30, 31, 35, 36, 40 or 41.",
    ),
    (
        "private_health_insurance_rebate.rate_unpublished",
        "A line is for premiums paid from 1 April 2027, whose rebate percentages had not been published.",
    ),
    (
        "psi_test.agent_payments_invalid",
        "An agent's principal payments are nil or less than the commission given.",
    ),
    (
        "psi_test.agent_rules",
        "At least 75% of an agent's payments from the principal are commission or performance fees, so s 87-40 may apply; the worksheet does not apply it.",
    ),
    (
        "psi_test.earner_invalid",
        "earner is not employee, religious_practitioner, sole_trader or entity.",
    ),
    ("psi_test.no_psi", "The payers' PSI totals nil."),
    (
        "psi_test.others_exceed_psi",
        "The market value of principal work by others exceeds the PSI given.",
    ),
    (
        "psi_test.payers_invalid",
        "payers is not a list of 1 to 50 payer objects.",
    ),
    (
        "psi_test.premises_invalid",
        "premises is not an object of the 5 business premises answers.",
    ),
    (
        "rd_tax_offset.clawback_above_total",
        "The clawback amount is more than notional_deductions, the total the adjusted offset reduces.",
    ),
    (
        "rd_tax_offset.clawback_below_threshold",
        "There is a clawback amount and notional_deductions, or notional_deductions less the clawback amount, is under $20,000, where s 355-100(2) and the ATO's feedstock example differ.",
    ),
    (
        "rd_tax_offset.feedstock_expenditure_above_total",
        "feedstock_expenditure is more than notional_deductions, of which it is part.",
    ),
    (
        "rd_tax_offset.threshold_expenditure_above_total",
        "rsp_crc_expenditure is more than notional_deductions, of which it is part.",
    ),
    (
        "rental_property.asset_acquired_9_may_2017",
        "A depreciating asset was acquired on 9 May 2017, and s 40-27 applies only from 7.30 pm that day.",
    ),
    (
        "rental_property.days_unclear",
        "Capital works or a depreciating asset were first used after the property was first held in the year, and it was not available on every day held, so their days of rental use are not known.",
    ),
    (
        "rental_property.fact_conflict",
        "The facts contradict each other or the income year: dates outside it or out of order, days available over the days held, or principal for the rental over the total.",
    ),
    (
        "rental_property.fact_invalid",
        "A rental property input is not the kind of value it takes: a list, an object of amounts, a date, a whole number of days, a kind of capital works or an ownership share.",
    ),
    (
        "rental_property.interest_redraw",
        "A loan had a redraw or further drawing in the year, which changes its share of principal used for the rental from that day.",
    ),
    (
        "rental_property.s26_50_denied",
        "The property is a holiday home whose ownership costs s 26-50 denies.",
    ),
    (
        "rental_property.tax_effect_year",
        "other_taxable_income was given for a year before 2024-25, for which resident-tax holds no scale.",
    ),
    (
        "rental_property.works_intended_use",
        "A building begun from 22 August 1979 to 17 July 1985 qualifies only if intended on completion for short-term traveller accommodation or non-residential use, which the worksheet does not take.",
    ),
    (
        "residency_tests.presence_invalid",
        "presence is not a list of at most 400 periods, each with from and to as dates, to not before from.",
    ),
    (
        "residency_tests.super_invalid",
        "commonwealth_super is not none, pss_member, css_eligible_employee, partner_of_member or child_under_16_of_member.",
    ),
    (
        "residency_tests.treaty_tie_breaker",
        "The individual is a resident under ITAA 1936 s 6(1) and also of another country under its law; a tax treaty's tie-breaker may apply, and the worksheet does not apply treaties.",
    ),
    (
        "sb_cgt_concessions.active_period_unclear",
        "The asset was owned for more than 15 years and its active days, 2,737 to 2,740, may or may not be 7.5 years.",
    ),
    (
        "sb_cgt_concessions.entity_invalid",
        "entity is not individual, company or trust.",
    ),
    (
        "sb_cgt_concessions.fact_conflict",
        "The facts contradict each other or the income year: dates out of order, active days beyond the test period, a partnership asset that is also passively held, or an exempt amount over the limit.",
    ),
    (
        "sb_cgt_concessions.fact_invalid",
        "business or passively_held is not one of the answers the worksheet takes.",
    ),
    (
        "sb_cgt_concessions.fifteen_years_same_day",
        "The asset was acquired on the same date 15 years before the CGT event, so the 15 years turn on the time of day.",
    ),
    (
        "sb_cgt_concessions.pre_cgt_asset",
        "The asset was acquired before 20 September 1985, so the gain is disregarded and no concession is needed.",
    ),
    (
        "sb_cgt_concessions.share_or_trust_interest",
        "The asset is a share in a company or an interest in a trust, whose additional basic conditions the worksheet does not work out.",
    ),
    (
        "scholarship_exemption.commonwealth_payment",
        "The payment is one of the Commonwealth payments ITAA 1997 s 51-35(a), (b) or (f) excludes from item 2.1A.",
    ),
    (
        "scholarship_exemption.commonwealth_payment_invalid",
        "commonwealth_payment is not one of its 4 values.",
    ),
    (
        "scholarship_exemption.friendly_society_plan",
        "The payment is an education benefit under a friendly society scholarship plan.",
    ),
    (
        "scholarship_exemption.payment_type_invalid",
        "payment_type is not a payment item 2.1A covers.",
    ),
    (
        "scope.not_confirmed",
        "The stated scope conditions were not confirmed.",
    ),
    (
        "small_business_income_offset.business_invalid",
        "business is not carried_on or winding_up.",
    ),
    (
        "small_business_income_offset.prescribed_person",
        "The individual is a prescribed person under s 102AC of the ITAA 1936, whose total net small business income s 328-375 works out from business income the worksheet does not take.",
    ),
    (
        "small_business_income_offset.shares_invalid",
        "shares is not a list of at most 50 objects, each with amount and deductions.",
    ),
    (
        "smsf_tax.disregarded_small_fund_assets",
        "Income from segregated current pension assets was given for a fund with disregarded small fund assets (ITAA 1997 s 295-385(7)).",
    ),
    (
        "smsf_tax.foreign_fund",
        "The fund is not an Australian superannuation fund.",
    ),
    (
        "smsf_tax.non_complying",
        "The fund is not a complying superannuation fund for the year.",
    ),
    (
        "smsf_tax.part_exceeds_total",
        "No-TFN contributions exceed assessable contributions, or contribution deductions exceed deductions or assessable contributions.",
    ),
    (
        "smsf_tax.proportion_invalid",
        "exempt_proportion is not a decimal from 0 to 1.",
    ),
    (
        "super_benefit_tax.benefit_nil",
        "The superannuation lump sum or the value of the interest it is paid from is nil.",
    ),
    (
        "super_benefit_tax.disability_benefit",
        "A disability superannuation benefit was given; s 307-145's increase to its tax free component is outside the command.",
    ),
    (
        "super_benefit_tax.fact_conflict",
        "The facts given contradict each other: a benefit above the interest's value, a tax free component above the interest's value, an element untaxed in the fund above the taxable component, a payment date outside the year, a date of birth not before the payment date, or cap amounts counted above the year's cap.",
    ),
    (
        "super_benefit_tax.other_lump_sums_this_year",
        "Another lump sum in the same income year at or above preservation age and under 60 was given with some of the low rate cap left; ss 301-20(3) and 301-105(5) and (6) share the year's low rate cap across the lump sums, which the command does not take.",
    ),
    (
        "super_benefit_tax.pre_july_1983_service",
        "An element untaxed in the fund with service before 1 July 1983 was given; s 307-150's modification is outside the command.",
    ),
    (
        "super_benefit_tax.recipient_invalid",
        "recipient is not member, death_benefits_dependant or non_dependant.",
    ),
    (
        "super_crystallisation.components_exceed_value",
        "The stated components and undeducted contributions add to more than the interest's value.",
    ),
    (
        "super_crystallisation.dates_invalid",
        "eligible_service_start is after 30 June 2007.",
    ),
    (
        "super_crystallisation.income_stream_before_july_2007",
        "The interest supports an income stream first paid before 1 July 2007 (ITTPA 1997 s 307-125).",
    ),
    (
        "super_crystallisation.untaxed_source",
        "The interest is from an untaxed source (ITTPA 1997 s 307-225(3)).",
    ),
    (
        "super_withholding.capped_defined_benefit",
        "A capped defined benefit income stream was given for a payee 60 or over; Schedule 13 Parts B to E and the defined benefit income cap are outside the command.",
    ),
    (
        "super_withholding.deceased_60_recipient_under_60",
        "A death benefit income stream was given to a dependant under 60 whose deceased died aged 60 or over; Schedule 13 Part A and ITAA 1997 s 302-65(b) treat its element taxed in the fund differently, and the command does not choose between them.",
    ),
    (
        "super_withholding.income_stream_foreign_resident",
        "An income stream payment to a foreign resident with a TFN was given; Schedule 13 names no scale for it.",
    ),
    (
        "super_withholding.income_stream_period_invalid",
        "income_stream_period is not weekly, fortnightly, monthly or quarterly.",
    ),
    (
        "super_withholding.income_stream_year",
        "An income stream payment was given for a year before 2024-25; Schedule 13 as it stood then was not read for this command.",
    ),
    (
        "super_withholding.no_tfn_unstated",
        "No TFN was given for a payment the no-TFN rates of Schedule 12 do not state: a non-dependant's death benefit or a member benefit under $200.",
    ),
    (
        "super_withholding.recipient_invalid",
        "recipient is not member, death_benefits_dependant, non_dependant, rollover or deceased_estate_trustee.",
    ),
    (
        "super_withholding.schedule_not_read",
        "The payment date is before 13 October 2020; Schedule 12 as it stood then was not read for this command.",
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
        "trading_stock.entity_invalid",
        "entity is not small_business_entity, medium_business_entity or other.",
    ),
    (
        "trading_stock.items_invalid",
        "items is not a list of at most 500 objects.",
    ),
    (
        "trading_stock.method_invalid",
        "An item's method is not cost, market_selling_value, replacement_value, lowest or obsolescence.",
    ),
    (
        "trading_stock.obsolescence_not_below",
        "An item's obsolescence value is not below its cost, market selling value and replacement value (ITAA 1997 s 70-50).",
    ),
    (
        "trading_stock.simplified_unavailable",
        "The s 328-285 choice was made by an entity that cannot make it for the year, or with the estimated closing value more than $5,000 from the opening value.",
    ),
    (
        "trading_stock.value_missing",
        "An item's elected value is null, or lowest or obsolescence lacks one of the 3 values.",
    ),
    (
        "transfer_balance.date_order",
        "An event or date is out of the order the account needs.",
    ),
    (
        "transfer_balance.event_refused",
        "An event of a kind the command does not work, with the reason.",
    ),
    (
        "transfer_balance.fact_invalid",
        "An event or fact is not of the form the command takes.",
    ),
    (
        "transfer_balance.rate_unpublished",
        "A day with excess needs a GIC rate the ATO has not yet published.",
    ),
    (
        "transfer_balance.transition_undecided",
        "IT(TP)A 1997 s 294-30 may disregard the 2017 excess, which only the debits to 31 December 2017 decide.",
    ),
    (
        "transfer_balance.year_mismatch",
        "The year is not the financial year of the last day worked.",
    ),
    (
        "transfer_duty.buyer_invalid",
        "buyer is not general, first_home_established or first_home_new.",
    ),
    (
        "transfer_duty.dutiable_value_nil",
        "The dutiable value is nil.",
    ),
    (
        "transfer_duty.first_home_not_residential",
        "A first home buyer was given for land that is not wholly residential; the concessions apply to a home.",
    ),
    (
        "transfer_duty.jurisdiction_invalid",
        "jurisdiction is not nsw, vic, qld, wa, sa, tas, act or nt.",
    ),
    (
        "transfer_duty.non_residential_unsupported",
        "Land that is not wholly residential was given in sa or act, whose rates for it this worksheet does not hold, or in nsw above $3,870,000, where s 32B or s 32C may charge premium duty.",
    ),
    (
        "vic_property_taxes.concession_unavailable",
        "The s 88EB concessional rate of vacant residential land tax was claimed for 2025; it needs an s 88EA exemption in a preceding tax year, and s 88EA commenced on 1 January 2025.",
    ),
    (
        "vic_property_taxes.entry_date_before_reform",
        "The land's entry date into the commercial and industrial property tax reform scheme is before 1 July 2024, when the Act commenced.",
    ),
    (
        "vic_property_taxes.parcels_invalid",
        "parcels is not a list of 1 to 100 objects, each with civ_before and civ_after as whole-dollar amounts of 0 or more.",
    ),
    (
        "vic_property_taxes.residential_land_invalid",
        "residential_land is not vacant_home, construction, uninhabitable or undeveloped.",
    ),
    (
        "vic_property_taxes.tax_invalid",
        "tax is not vacant_residential_land, commercial_industrial or windfall_gains.",
    ),
    (
        "vic_property_taxes.transition_end_unsettled",
        "The entry date is 31 December, so the transition period ends at the midnight that fixes liability; the Acts do not settle whether the land is still within it then.",
    ),
    (
        "vic_property_taxes.undeveloped_before_2026",
        "Vacant residential land tax on undeveloped land (s 34B(2B)) was asked for the 2025 tax year; it applies from 2026.",
    ),
    (
        "voluntary_agreement_withholding.abn_not_quoted",
        "The payee's ABN is not in force or not quoted in the voluntary agreement (TAA 1953 Sch 1 s 12-55(1)(d)).",
    ),
    (
        "voluntary_agreement_withholding.agreement_not_in_force",
        "The voluntary agreement is not in force (TAA 1953 Sch 1 s 12-55(1)(c)).",
    ),
    (
        "voluntary_agreement_withholding.gst_above_payment",
        "The GST included is more than the payment.",
    ),
    (
        "voluntary_agreement_withholding.instalment_rate_missing",
        "The agreement applies the payee's Commissioner's instalment rate, but no instalment_rate was given.",
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
        "withholding.working_holiday_maker",
        "The payee is a working holiday maker, for whom Schedule 15 governs every payment.",
    ),
    (
        "work_expenses.allowance_received",
        "An allowance was received for the expenses, so the reasonable allowance exceptions and occupation-specific guidance the worksheet does not apply would be needed.",
    ),
    (
        "work_expenses.car_not_held",
        "The car was neither owned nor leased by the taxpayer, so neither Division 28 method applies (ITAA 1997 s 28-12(1)).",
    ),
    (
        "work_expenses.diary_year_unsupported",
        "An hours diary was given for a year before 2022-23; it is read only for the years PCG 2023/1 covers.",
    ),
    (
        "work_expenses.fact_conflict",
        "The facts contradict each other or the year: hours beyond the year, business kilometres above total kilometres, nil total kilometres, a log book after the year claimed, an amount for a running expense when none was incurred, category D cents per kilometre car expenses beside log book car expenses in category A, hours beside an hours diary, a diary date outside the year or over 24 hours, a log book beside log_book_weeks or business_kilometres, or a log book whose dates or odometer readings run backwards, overlap or fall outside its period.",
    ),
    (
        "work_expenses.fact_invalid",
        "connection, log_book_year or log_book_weeks is not one of the answers the worksheet takes, or an hours diary or log book is not the list or object of dated rows the worksheet reads.",
    ),
    (
        "work_expenses.log_book_entry_incomplete",
        "A journey in the log book does not say why it was made (ITAA 1997 s 28-125(2)(d)).",
    ),
    (
        "work_expenses.log_book_expired",
        "The log book was kept more than 4 income years before the year claimed (ITAA 1997 s 28-115(2)).",
    ),
    (
        "work_expenses.log_book_outside_year",
        "The log book period includes no part of the income year it is kept for (ITAA 1997 s 28-120(2)).",
    ),
    (
        "work_expenses.log_book_too_short",
        "The log book covers fewer than 12 continuous weeks (ITAA 1997 s 28-120(1)).",
    ),
    (
        "work_expenses.log_book_unrepresentative",
        "The log book no longer represents the car's business use, as after a change of job or home, so a new one is needed (ATO 'Logbook method').",
    ),
    (
        "work_expenses.method_invalid",
        "method is not home_office, car_cents_per_kilometre, car_log_book or self_education.",
    ),
    (
        "work_expenses.reimbursed",
        "The expenses were reimbursed or paid for by someone else, so the taxpayer did not bear them.",
    ),
    (
        "work_expenses.standard_deduction_unavailable",
        "The standard deduction facts were given for a year before 2026-27, to which ITAA 1997 s 25-130 does not apply.",
    ),
    (
        "workers_compensation_premium.adjustment_unsettled",
        "Actual wages were given for a Queensland policy commencing in 2026-27, for which the notice sets no year-end adjustment.",
    ),
    (
        "workers_compensation_premium.apprentice_above_wages",
        "Apprentice wages are more than the wages they are part of.",
    ),
    (
        "workers_compensation_premium.apprentice_unavailable",
        "Apprentice wages above nil were given for a scheme this worksheet deducts none for (vic, wa, tas, act, nt).",
    ),
    (
        "workers_compensation_premium.apprentice_unsettled",
        "Apprentice wages above nil were given in New South Wales, where icare does not say whether its apprentice incentive comes off before or after GST.",
    ),
    (
        "workers_compensation_premium.band_edge_unsettled",
        "A South Australian base premium is above a discount band's bound less a cent and below the bound, where ReturnToWorkSA's table and its calculator give different bands.",
    ),
    (
        "workers_compensation_premium.below_minimum",
        "A New South Wales premium is below icare's $240 minimum, whose GST treatment icare does not state.",
    ),
    (
        "workers_compensation_premium.days_invalid",
        "days_registered is not an integer from 1 to 365.",
    ),
    (
        "workers_compensation_premium.employer_invalid",
        "employer is not new, small or other.",
    ),
    (
        "workers_compensation_premium.experience_rated",
        "The premium is experience or performance rated: a New South Wales average performance premium over $30,000, or a Victorian employer that is neither new nor small.",
    ),
    (
        "workers_compensation_premium.group_rated",
        "The employer is a member of a group in a scheme that rates the group together (nsw, vic, qld, sa).",
    ),
    (
        "workers_compensation_premium.jurisdiction_invalid",
        "jurisdiction is not nsw, vic, qld, wa, sa, tas, act or nt.",
    ),
    (
        "workers_compensation_premium.policy_after_year",
        "A Queensland policy_start is after 30 June 2027.",
    ),
    (
        "workers_compensation_premium.policy_before_year",
        "A Queensland policy commenced before 1 July 2026, so its premium rate and prior-year adjustment come from WorkCover Queensland's models.",
    ),
    (
        "workers_compensation_premium.rate_invalid",
        "A rate is not a percentage in its range to at most 4 decimal places.",
    ),
    (
        "workers_compensation_premium.remuneration_nil",
        "A Victorian remuneration rounds to $0, and the Premiums Order's deductible divides by it.",
    ),
    (
        "workers_compensation_premium.rounding_unsettled",
        "A Queensland premium differs by a cent with the estimated premium and apprentice discount each rounded to the cent first, and the notice does not say which.",
    ),
    (
        "working_holiday_maker.band_unsettled",
        "Payments before are from $135,000.01 to $135,000.99, which none of Schedule 15's columns covers.",
    ),
    ("worksheet.unknown", "The worksheet does not exist."),
    (
        "zone_tax_offset.area_invalid",
        "area is not special_area, zone_a, zone_b or overseas_forces.",
    ),
    (
        "zone_tax_offset.dependants_invalid",
        "dependants is not a list of up to 50 objects with kind student or child, adjusted_taxable_income and optional days.",
    ),
    (
        "zone_tax_offset.half_year_unclear",
        "The usual place of residence was in the area for 183 days of 2023-24, exactly one-half of the year, which ITAA 1936 s 79A(3B)(a) and the ATO's guidance treat differently.",
    ),
    (
        "zone_tax_offset.prior_year_invalid",
        "prior_year is not an income year 1 to 4 years before, or prior_days is more than one-half of it.",
    ),
    (
        "zone_tax_offset.prior_year_unclear",
        "prior_year is 5 years before, a relevant preceding year on the ATO's reading of ITAA 1936 s 79A(4) but not on its words.",
    ),
    (
        "zone_tax_offset.shared_maintenance",
        "A dependant was also maintained by another person, whose apportionment the ATO does not publish.",
    ),
    (
        "zone_tax_offset.sole_parent_without_dependant",
        "sole_parent_days was given without a dependant notional tax offset of at least $1 (ITAA 1997 s 961-55(1)(b)).",
    ),
];

/// `lodgment_required`: each condition, its key, its provision in the instruments from 2023-24
/// and in the Notices before, and what it says.
pub const LODGMENT_PROVISIONS: [(&str, &str, &str, &str); 23] = [
    (
        "payg_withheld",
        "paragraph 6(2)(a)",
        "Table A item (1)",
        "an amount withheld under PAYG withholding, other than from the payments the paragraph excludes",
    ),
    (
        "tax_loss",
        "paragraph 6(2)(b)",
        "Table A item (2)",
        "a tax loss incurred, or an entitlement to deduct one",
    ),
    (
        "net_capital_loss",
        "paragraph 6(2)(c)",
        "Table A item (3)",
        "a net capital loss made, or an entitlement to apply one of an earlier year",
    ),
    (
        "business",
        "paragraph 6(2)(f)",
        "Table A item (6)",
        "a business carried on",
    ),
    (
        "primary_production_trust",
        "paragraph 6(2)(g)",
        "Table A item (7)",
        "income as a beneficiary of a trust that operated a primary production business in Australia",
    ),
    (
        "primary_production_partnership",
        "paragraph 6(2)(h)",
        "Table A item (8)",
        "an individual interest in the net income or partnership loss of a partnership that operated a primary production business in Australia",
    ),
    (
        "under_18",
        "paragraph 6(2)(i)",
        "Table A item (9)",
        "under 18 at all times, with income other than from work, or from dividends and franking credits, over $416",
    ),
    (
        "exempt_overseas_income",
        "paragraph 6(2)(j)",
        "Table A item (10)",
        "income under ITAA 1936 s 23AF or 23AG and $1 or more of other income",
    ),
    (
        "payg_instalment",
        "paragraph 6(2)(k)",
        "Table A item (11)",
        "a PAYG instalment paid",
    ),
    (
        "special_professional",
        "paragraph 6(2)(l)",
        "Table A item (12)",
        "a special professional under ITAA 1997 Division 405",
    ),
    (
        "private_health_offset",
        "paragraph 6(2)(m)",
        "Table A item (13)",
        "entitled to the private health insurance offset and not claiming the correct offset as a premium reduction, unless a choice was made under ITAA 1997 s 61-215",
    ),
    (
        "reportable_amounts",
        "paragraph 6(2)(n)",
        "Table A item (14)",
        "reportable fringe benefits or reportable employer superannuation contributions on a payment summary or income statement",
    ),
    (
        "dividends",
        "paragraph 6(2)(o)",
        "Table A item (15)",
        "assessable income from dividends or distributions and franking credits over $18,200",
    ),
    (
        "co_contribution",
        "paragraph 6(2)(p)",
        "Table A item (16)",
        "personal contributions that will receive a super co-contribution",
    ),
    (
        "contributions_cap",
        "paragraph 6(2)(q)",
        "Table A item (17)",
        "a concessional or non-concessional contributions cap exceeded",
    ),
    (
        "super_lump_sum",
        "paragraph 6(2)(r)",
        "Table A item (18)",
        "an Australian superannuation lump sum with an untaxed element at 60 or over, or a taxed or untaxed element under 60",
    ),
    (
        "resident_whole_year",
        "subparagraph 6(3)(a)(i)",
        "Table B item (1)(a)",
        "a resident for the whole year with taxable income over $18,200",
    ),
    (
        "resident_part_year",
        "subparagraph 6(3)(a)(ii)",
        "Table B item (1)(b)",
        "a resident for part of the year with taxable income over the lesser of $18,200 and $13,464 plus $395 a month",
    ),
    (
        "not_resident",
        "paragraph 6(3)(b)",
        "Table B item (2)",
        "not a resident at any time, with income taxable in Australia other than the excluded withholding payments",
    ),
    (
        "study_loan",
        "subsection 6(10)",
        "Table J",
        "a foreign resident with a study or training loan debt on 1 June before the year and income over the year's amount",
    ),
    (
        "working_holiday_maker",
        "paragraph 7(1)(f)",
        "Table P",
        "a working holiday maker with no other assessable income and taxable income, without super or ETP remainders, under $45,001",
    ),
    (
        "listed_payment",
        "subsection 7(2)",
        "Table K item (1)",
        "a payment the instrument lists, and no other assessable income or taxable income under the year's amount",
    ),
    (
        "sapto",
        "subsection 7(3)",
        "Table K item (2)",
        "eligible for the seniors and pensioners tax offset, with rebate income under the year's amount",
    ),
];

/// `smsf_tax`: (id, authority, formula, return label) for each amount, in order.
pub const SMSF_TAX_STEPS: [(&str, &str, &str, &str); 20] = [
    (
        "gross_income",
        "ITAA 1997 s 295-10(1) step 2 and SAR 2026 label W: assessable contributions (Subdivision 295-C), net non-arm's length income (label U) and the fund's other ordinary and statutory income",
        "assessable_contributions + net_non_arms_length_income + segregated_pension_income + unsegregated_income + segregated_non_current_income",
        "W",
    ),
    (
        "exempt_current_pension_income",
        "ITAA 1997 s 295-385(1) (income from segregated current pension assets) and s 295-390(1) and (3) (the actuary's proportion of other income, not non-arm's length income, contributions or income from segregated non-current assets, s 295-390(2)); SAR 2026 label Y, in whole dollars, the proportion's product rounded down",
        "segregated_pension_income + floor(unsegregated_income x exempt_proportion)",
        "Y",
    ),
    (
        "total_assessable_income",
        "SAR 2026 label V: gross income less exempt current pension income",
        "gross_income - exempt_current_pension_income",
        "V",
    ),
    (
        "taxable_income",
        "ITAA 1997 s 4-15 and SAR 2026 labels O and A: total assessable income less deductions; nil for a loss",
        "max(0, total_assessable_income - deductions)",
        "A",
    ),
    (
        "loss",
        "SAR 2026 label O: the loss, which Section E carries forward (Division 36, not worked out here)",
        "max(0, deductions - total_assessable_income)",
        "",
    ),
    (
        "non_arms_length_component",
        "ITAA 1997 s 295-545(2A): the lesser of net non-arm's length income under s 295-550(1), (2), (4) and (5) plus that under s 295-550(8) and (9), and taxable income less assessable contributions plus the deductions attributable to them (not below nil)",
        "min(net_non_arms_length_income + general_expense_non_arms_length_income, max(0, taxable_income - assessable_contributions + contribution_deductions))",
        "",
    ),
    (
        "low_tax_component",
        "ITAA 1997 s 295-545(3): the rest of taxable income",
        "taxable_income - non_arms_length_component",
        "",
    ),
    (
        "tax_on_taxable_income",
        "Income Tax Rates Act 1986 s 26(1): 15% of the low tax component and 45% of the non-arm's length component",
        "low_tax_component x low_tax_rate + non_arms_length_component x non_arms_length_rate",
        "T1",
    ),
    (
        "no_tfn_contributions_tax",
        "ITAA 1997 ss 295-605 and 295-610 and Income Tax Rates Act 1986 s 29(2): the 45% top rate plus 2% less 15%",
        "no_tfn_contributions x no_tfn_rate",
        "J",
    ),
    (
        "gross_tax",
        "ITAA 1997 s 295-10(1) steps 1 and 5; SAR 2026 label B",
        "tax_on_taxable_income + no_tfn_contributions_tax",
        "B",
    ),
    (
        "subtotal_1",
        "ITAA 1997 s 295-10(1) step 6; SAR 2026 label T2: gross tax less non-refundable non-carry forward tax offsets (label C), not below nil",
        "max(0, gross_tax - non_refundable_offsets)",
        "T2",
    ),
    (
        "non_refundable_offsets_lost",
        "SAR 2026 label T2: the non-refundable non-carry forward tax offsets above gross tax are lost",
        "max(0, non_refundable_offsets - gross_tax)",
        "",
    ),
    (
        "subtotal_2",
        "SAR 2026 label T3: subtotal 1 less non-refundable carry forward tax offsets (label D), not below nil",
        "max(0, subtotal_1 - carry_forward_offsets)",
        "T3",
    ),
    (
        "carry_forward_offsets_unused",
        "SAR 2026 label T3: the carry forward tax offsets above subtotal 1, which Division 65 may carry forward",
        "max(0, carry_forward_offsets - subtotal_1)",
        "",
    ),
    (
        "tax_payable",
        "SAR 2026 label T5: subtotal 2 less refundable tax offsets (label E), not below nil",
        "max(0, subtotal_2 - refundable_offsets)",
        "T5",
    ),
    (
        "tax_offset_refunds",
        "SAR 2026 label I: the refundable tax offsets above subtotal 2",
        "max(0, refundable_offsets - subtotal_2)",
        "I",
    ),
    (
        "supervisory_levy",
        "Superannuation (Self Managed Superannuation Funds) Supervisory Levy Imposition Regulations 2018 s 6; SAR 2026 label L: the levy for the next year, paid with this return",
        "levy",
        "L",
    ),
    (
        "levy_adjustment_wound_up",
        "SAR 2026 label M and table 9: an existing fund wound up in the year pays no levy for the next year",
        "levy",
        "M",
    ),
    (
        "levy_adjustment_new_fund",
        "SAR 2026 label N and table 9: a newly registered fund not wound up in the year also pays the levy for this year",
        "levy",
        "N",
    ),
    (
        "amount_due_or_refundable",
        "SAR 2026 label S: tax payable, the s 102AAM interest charge (G), the levy (L) and the new fund adjustment (N), less eligible credits (H), tax offset refunds (I), PAYG instalments (K) and the wound up adjustment (M); negative is a refund",
        "tax_payable + interest_charge + supervisory_levy + levy_adjustment_new_fund - eligible_credits - tax_offset_refunds - payg_instalments - levy_adjustment_wound_up",
        "S",
    ),
];
