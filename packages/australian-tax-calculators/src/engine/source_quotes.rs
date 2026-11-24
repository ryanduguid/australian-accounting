// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The passage of each Act, determination or revenue office page that states
//! the rate, scale or grant a state duty, surcharge, land tax or first home
//! owner grant command applies, quoted word for word as read at the official
//! site on [`VERIFIED`] (docs/calculation-evidence.md#quoted-source-passages).
//! Line and table row breaks are kept as newlines and table cells are joined
//! by a tab; non-breaking spaces are written as spaces and zero-width
//! characters are left out.

use serde_json::{Value, json};

/// The day every passage below was read at its URL.
pub(crate) const VERIFIED: &str = "2026-11-24";

pub(crate) struct Quote {
    pub kind: &'static str,
    pub jurisdiction: &'static str,
    pub provision: &'static str,
    pub source: &'static str,
    pub url: &'static str,
    pub passage: &'static str,
}

/// The commands whose results carry `source_quotes`, by API command name.
const COMMANDS: [(&str, &str); 4] = [
    ("transfer-duty", "transfer_duty"),
    ("foreign-purchaser-surcharge", "foreign_purchaser_surcharge"),
    ("land-tax", "land_tax"),
    ("first-home-owner-grant", "first_home_owner_grant"),
];

pub(crate) const QUOTES: [Quote; 36] = [
    Quote {
        kind: "transfer_duty",
        jurisdiction: "nsw",
        provision: "Duties Act 1997 (NSW) s 32",
        source: "Revenue NSW, 'Calculate transfer duty', transfer duty rates for 2026/2027",
        url: "https://www.revenue.nsw.gov.au/taxes-duties-levies-royalties/transfer-duty/understanding-transfer-duty/calculate-transfer-duty",
        passage: "$0 to $18,000\n$1.25 for every $100 (minimum $20)\n$18,001 to $38,000\n$225 plus $1.50 for every $100 over $18,000\n$38,001 to $103,000\n$525 plus $1.75 for every $100 over $38,000\n$103,001 to $387,000\n$1,662 plus $3.50 for every $100 over $103,000\n$387,001 to $1,290,000\n$11,602 plus $4.50 for every $100 over $387,000\nOver $1,290,000\n$52,237 plus $5.50 for every $100 over $1,290,000",
    },
    Quote {
        kind: "transfer_duty",
        jurisdiction: "vic",
        provision: "Duties Act 2000 (Vic) s 28",
        source: "SRO Victoria, 'Land transfer duty – non-principal place of residence (current rates)'",
        url: "https://www.sro.vic.gov.au/about-us/rates-and-statistics/current-rates/land-transfer-duty-non-principal-place-residence-current-rates",
        passage: "$0 - $25,000\t1.4% of the dutiable value of the property\n>$25,000 - $130,000\t$350 plus 2.4% of the dutiable value in excess of $25,000\n>$130,000 - $960,000\t$2870 plus 6% of the dutiable value in excess of $130,000\n>$960,000 - $2,000,000\t5.5% of the dutiable value\nMore than $2,000,000\t$110,000 plus 6.5% of the dutiable value in excess of $2,000,000",
    },
    Quote {
        kind: "transfer_duty",
        jurisdiction: "qld",
        provision: "Duties Act 2001 (Qld) s 24(2) and Sch 3",
        source: "Queensland Revenue Office, 'Transfer duty rates', current transfer duty rates",
        url: "https://qro.qld.gov.au/duties/transfer-duty/calculate/rates/",
        passage: "Not more than $5,000\tNil\nMore than $5,000 up to $75,000\t$1.50 for each $100, or part of $100, over $5,000\n$75,000 to $540,000\t$1,050 plus $3.50 for each $100, or part of $100, over $75,000\n$540,000 to $1,000,000\t$17,325 plus $4.50 for each $100, or part of $100, over $540,000\nMore than $1,000,000\t$38,025 plus $5.75 for each $100, or part of $100, over $1,000,000",
    },
    Quote {
        kind: "transfer_duty",
        jurisdiction: "wa",
        provision: "Duties Act 2008 (WA) Sch 2",
        source: "Department of Treasury and Finance, 'Transfer duty assessment', general rate",
        url: "https://www.wa.gov.au/organisation/department-of-treasury-and-finance/transfer-duty-assessment",
        passage: "$0 - $120,000\t$1.90 per $100 or part thereof\n$120,001 - $150,000\t$2,280 + $2.85 per $100 or part thereof above $120,000\n$150,001 - $360,000\t$3,135 + $3.80 per $100 or part thereof above $150,000\n$360,001 - $725,000\t$11,115 + $4.75 per $100 or part thereof above $360,000\n$725,001 +\t$28,453 + $5.15 per $100 or part thereof above $725,000",
    },
    Quote {
        kind: "transfer_duty",
        jurisdiction: "sa",
        provision: "Stamp Duties Act 1923 (SA) Sch 2 item 3(b)",
        source: "RevenueSA, 'Rate of stamp duty', conveyances",
        url: "https://revenuesa.sa.gov.au/stamp-duty/rate-of-stamp-duty",
        passage: "Does not exceed $12,000\n$1.00 for every $100 or part of $100\nExceeds $12,000 but not $30,000\n$120 plus $2.00 for every $100 or part of $100 over $12,000\nExceeds $30,000 but not $50,000\n$480 plus $3.00 for every $100 or part of $100 over $30,000\nExceeds $50,000 but not $100,000\n$1,080 plus $3.50 for every $100 or part of $100 over $50,000\nExceeds $100,000 but not $200,000\n$2,830 plus $4.00 for every $100 or part of $100 over $100,000\nExceeds $200,000 but not $250,000\n$6,830 plus $4.25 for every $100 or part of $100 over $200,000\nExceeds $250,000 but not $300,000\n$8,955 plus $4.75 for every $100 or part of $100 over $250,000\nExceeds $300,000 but not $500,000\n$11,330 plus $5.00 for every $100 or part of $100 over $300,000\nExceeds $500,000\n$21,330 plus $5.50 for every $100 or part of $100 over $500,000",
    },
    Quote {
        kind: "transfer_duty",
        jurisdiction: "tas",
        provision: "Duties Act 2001 (Tas) s 29(1)",
        source: "Duties Act 2001 (Tas), current version, s 29",
        url: "https://www.legislation.tas.gov.au/view/whole/html/inforce/current/act-2001-015",
        passage: "Not more than $3 000\n$50\nMore than $3 000 but not more than $25 000\n$50 plus $1.75 for every $100, or part, by which the dutiable value exceeds $3 000\nMore than $25 000 but not more than $75 000\n$435 plus $2.25 for every $100, or part, by which the dutiable value exceeds $25 000\nMore than $75 000 but not more than $200 000\n$1 560 plus $3.50 for every $100, or part, by which the dutiable value exceeds $75 000\nMore than $200 000 but not more than $375 000\n$5 935 plus $4 for every $100, or part, by which the dutiable value exceeds $200 000\nMore than $375 000 but not more than $725 000\n$12 935 plus $4.25 for every $100, or part, by which the dutiable value exceeds $375 000\nMore than $725 000\n$27 810 plus $4.50 for every $100, or part, by which the dutiable value exceeds $725 000",
    },
    Quote {
        kind: "transfer_duty",
        jurisdiction: "act",
        provision: "Duties Act 1999 (ACT) s 31; DI2026-155 s 5(1)(b) and Table 2",
        source: "Taxation Administration (Amounts Payable—Duty) Determination 2026 (DI2026-155)",
        url: "https://www.legislation.act.gov.au/View/di/2026-155/current/html/2026-155.html",
        passage: "Table 2 Amounts payable (Transfer rate—transaction that is not an eligible owner occupier transaction)—Duties Act, s 31\ncolumn 1\ndutiable amount\ncolumn 2\nrate of duty (transaction that is not an eligible owner occupier transaction)\nless than or equal to $200 000\n$1.20 for every $100, or part of $100, of the dutiable amount\nmore than $200 000 but not more than $300 000\n$2,400 plus $2.20 for every $100, or part of $100, of the dutiable amount that is more than $200 000\nmore than $300 000 but not more than $500 000\n$4,600 plus $3.40 for every $100, or part of $100, of the dutiable amount that is more than $300 000\nmore than $500 000 but not more than $750 000\n$11,400 plus $4.32 for every $100, or part of $100, of the dutiable amount that is more than $500 000\nmore than $750 000 but not more than $1 000 000\n$22,200 plus $5.90 for every $100, or part of $100, of the dutiable amount that is more than $750 000\nmore than $1 000 000 but not more than $1 455 000\n$36,950 plus $6.40 for every $100, or part of $100, of the dutiable amount that is more than $1 000 000\nmore than $1 455 000\na flat rate of $4.54 per $100 applied to the total dutiable amount",
    },
    Quote {
        kind: "transfer_duty",
        jurisdiction: "nt",
        provision: "Stamp Duty Act 1978 (NT) Sch 1 cl 1(2)",
        source: "Stamp Duty Act 1978 (NT), as in force at 1 July 2025",
        url: "https://legislation.nt.gov.au/api/sitecore/Act/PDF?id=11906",
        passage: "(2) Subject to this clause, the duty payable on a conveyance of dutiable property is determined as follows:\n(a) if the dutiable value of the dutiable property subject to the conveyance does not exceed $525 000: D = (0.06571441 x V²) + 15V where: D is the duty (expressed in dollars) V is 1/1 000 of the dutiable value (expressed in dollars)\n(b) if the dutiable value exceeds $525 000 but is less than $3 000 000, the duty is 4.95% of the dutiable value;\n(c) if the dutiable value is $3 000 000 or more but is less than $5 000 000, the duty is 5.75% of the dutiable value;\n(d) if the dutiable value is $5 000 000 or more, the duty is 5.95% of the dutiable value.",
    },
    Quote {
        kind: "foreign_purchaser_surcharge",
        jurisdiction: "nsw",
        provision: "Duties Act 1997 (NSW) s 104U",
        source: "Revenue NSW, 'What is surcharge purchaser duty'",
        url: "https://www.revenue.nsw.gov.au/taxes-duties-levies-royalties/transfer-duty/surcharge-purchaser-duty/what-is-surcharge-purchaser-duty",
        passage: "The current surcharge purchaser duty rate is 9% of the dutiable value of the property.",
    },
    Quote {
        kind: "foreign_purchaser_surcharge",
        jurisdiction: "nsw",
        provision: "Land Tax Act 1956 (NSW), as Revenue NSW states it (the Act could not be read on 24 November 2026)",
        source: "Revenue NSW, 'What is surcharge land tax'",
        url: "https://www.revenue.nsw.gov.au/taxes-duties-levies-royalties/land-tax/surcharge-land-tax/what-is-surcharge-land-tax",
        passage: "2025 onwards\t5% of the land value of your property",
    },
    Quote {
        kind: "foreign_purchaser_surcharge",
        jurisdiction: "vic",
        provision: "Duties Act 2000 (Vic) s 28A",
        source: "SRO Victoria, 'Calculating foreign purchaser additional duty'",
        url: "https://www.sro.vic.gov.au/buying-property/foreign-purchasers-property/calculating-foreign-purchaser-additional-duty",
        passage: "The additional duty rate is 8%.",
    },
    Quote {
        kind: "foreign_purchaser_surcharge",
        jurisdiction: "vic",
        provision: "Land Tax Act 2005 (Vic) Sch 1 Table 4.5",
        source: "SRO Victoria, 'Absentee individuals'",
        url: "https://www.sro.vic.gov.au/owning-property/land-tax/absentee-owner-surcharge/absentee-individuals",
        passage: "This surcharge is 4% from the 2024 land tax year (previously 2% for 2020 to 2023, 1.5% for 2017 to 2019, and 0.5% for the 2016 land tax years).",
    },
    Quote {
        kind: "foreign_purchaser_surcharge",
        jurisdiction: "qld",
        provision: "Duties Act 2001 (Qld) s 244",
        source: "Queensland Revenue Office, 'Additional foreign acquirer duty'",
        url: "https://qro.qld.gov.au/duties/investors/afad/",
        passage: "AFAD is an extra 8% of (stamp) duty that applies to transactions that are liable for transfer duty, landholder duty or corporate trustee duty.",
    },
    Quote {
        kind: "foreign_purchaser_surcharge",
        jurisdiction: "qld",
        provision: "Land Tax Act 2010 (Qld) Sch 3 Pt 2",
        source: "Queensland Revenue Office, 'Land tax threshold and rates for absentees'",
        url: "https://qro.qld.gov.au/land-tax/calculate/absentee/",
        passage: "An absentee surcharge of 3% on taxable land valued at $350,000 or more applies in addition to the land tax rates.",
    },
    Quote {
        kind: "foreign_purchaser_surcharge",
        jurisdiction: "wa",
        provision: "Duties Act 2008 (WA) s 205O",
        source: "Western Australian Government, 'About foreign buyers duty'",
        url: "https://www.wa.gov.au/service/financial-management/taxation-and-duty/about-foreign-buyers-duty",
        passage: "Foreign buyers duty imposes additional duty of 7% on certain transactions and landholder acquisitions involving foreign persons acquiring residential property in Western Australia.",
    },
    Quote {
        kind: "foreign_purchaser_surcharge",
        jurisdiction: "sa",
        provision: "Stamp Duties Act 1923 (SA) s 72(3)",
        source: "RevenueSA, 'Foreign ownership surcharge'",
        url: "https://www.revenuesa.sa.gov.au/stampduty/FOS",
        passage: "Foreign persons (which includes natural persons and corporations) or foreign trusts that acquire an interest in residential land in South Australia are required to pay a foreign ownership surcharge (the “surcharge”) of 7% of the value of the interest in residential land.",
    },
    Quote {
        kind: "foreign_purchaser_surcharge",
        jurisdiction: "tas",
        provision: "Duties Act 2001 (Tas) ss 30C and 30E",
        source: "SRO Tasmania, 'Rates of surcharge'",
        url: "https://www.sro.tas.gov.au/property-transfer-duties/foreign-investor-duty-surcharge/rates-of-surcharge",
        passage: "For agreements (or transactions without a written agreement) on or after 1 April 2020.\nThe following surcharge rate applies:\n8 percent on the proportion of the dutiable value of residential property directly or indirectly acquired by\na foreign person; and\n1.5 per cent on the proportion of the dutiable value of primary productions property directly or indirectly acquired by a foreign person.",
    },
    Quote {
        kind: "foreign_purchaser_surcharge",
        jurisdiction: "act",
        provision: "Land Tax Act 2004 (ACT) s 17E(3)(b) and (4); DI2026-152 s 7",
        source: "Taxation Administration (Amounts Payable—Land Tax) Determination 2026 (DI2026-152)",
        url: "https://www.legislation.act.gov.au/View/di/2026-152/current/html/2026-152.html",
        passage: "For the purposes of section 17E (4) of the Act, I determine that P (or percentage rate) is 0.75% of the base value.",
    },
    Quote {
        kind: "foreign_purchaser_surcharge",
        jurisdiction: "nt",
        provision: "Stamp Duty Act 1978 (NT) Sch 1 cl 1(2), which sets no rate for a foreign purchaser",
        source: "Stamp Duty Act 1978 (NT), as in force at 1 July 2025",
        url: "https://legislation.nt.gov.au/api/sitecore/Act/PDF?id=11906",
        passage: "(2) Subject to this clause, the duty payable on a conveyance of dutiable property is determined as follows:\n(a) if the dutiable value of the dutiable property subject to the conveyance does not exceed $525 000: D = (0.06571441 x V²) + 15V where: D is the duty (expressed in dollars) V is 1/1 000 of the dutiable value (expressed in dollars)\n(b) if the dutiable value exceeds $525 000 but is less than $3 000 000, the duty is 4.95% of the dutiable value;\n(c) if the dutiable value is $3 000 000 or more but is less than $5 000 000, the duty is 5.75% of the dutiable value;\n(d) if the dutiable value is $5 000 000 or more, the duty is 5.95% of the dutiable value.",
    },
    Quote {
        kind: "foreign_purchaser_surcharge",
        jurisdiction: "nt",
        provision: "no Act: the Territory levies no land tax",
        source: "NT Government, 'Property taxes'",
        url: "https://nt.gov.au/property/land/buying-and-selling-land/land-taxes",
        passage: "There is no land tax in the NT.",
    },
    Quote {
        kind: "land_tax",
        jurisdiction: "nsw",
        provision: "Land Tax Act 1956 (NSW) and Land Tax Management Act 1956 (NSW), as Revenue NSW states them (the Acts could not be read on 24 November 2026)",
        source: "Revenue NSW, 'Land tax thresholds and rates'",
        url: "https://www.revenue.nsw.gov.au/taxes-duties-levies-royalties/land-tax/understanding-land-tax/thresholds-and-rates",
        passage: "General\tMore than $1,075,000 but less than $6,571,000\t$100 + 1.6% of the land value above $1,075,000\nPremium\tMore than $6,571,000\t$88,036 + 2% of the land value above $6,571,000",
    },
    Quote {
        kind: "land_tax",
        jurisdiction: "vic",
        provision: "Land Tax Act 2005 (Vic) Sch 1 Table 1.6",
        source: "SRO Victoria, 'Land tax (current rates)', general rates",
        url: "https://www.sro.vic.gov.au/about-us/rates-and-statistics/current-rates/land-tax-current-rates",
        passage: "< $50,000\tNil\n$50,000 to < $100,000\t$500\n$100,000 to < $300,000\t$975\n$300,000 to < $600,000\t$1350 plus 0.3% of amount > $300,000\n$600,000 to < $1,000,000\t$2250 plus 0.6% of amount > $600,000\n$1,000,000 to < $1,800,000\t$4650 plus 0.9% of amount > $1,000,000\n$1,800,000 to < $3,000,000\t$11,850 plus 1.65% of amount > $1,800,000\n$3,000,000 and over\t$31,650 plus 2.65% of amount > $3,000,000",
    },
    Quote {
        kind: "land_tax",
        jurisdiction: "qld",
        provision: "Land Tax Act 2010 (Qld) s 32(1)(a) and Sch 1",
        source: "Queensland Revenue Office, 'Land tax threshold and rates for individuals'",
        url: "https://qro.qld.gov.au/land-tax/calculate/individual/",
        passage: "$0–$599,999\t$0\n$600,000–$999,999\t$500 plus 1 cent for each $1 more than $600,000\n$1,000,000–$2,999,999\t$4,500 plus 1.65 cents for each $1 more than $1,000,000\n$3,000,000–$4,999,999\t$37,500 plus 1.25 cents for each $1 more than $3,000,000\n$5,000,000–$9,999,999\t$62,500 plus 1.75 cents for each $1 more than $5,000,000\n$10,000,000 or more\t$150,000 plus 2.25 cents for each $1 more than $10,000,000",
    },
    Quote {
        kind: "land_tax",
        jurisdiction: "wa",
        provision: "Land Tax Act 2002 (WA) s 5 Table 11",
        source: "Department of Treasury and Finance, 'Land tax assessment'",
        url: "https://www.wa.gov.au/organisation/department-of-treasury-and-finance/land-tax-assessment",
        passage: "$0 - $300,000\tNil\n$300,001 - $420,000\t$300\n$420,001 - $1,000,000\t$300 + 0.0025 dollars for each $1 in excess of $420,000\n$1,000,001 - $1,800,000\t$1,750 + 0.009 dollars for each $1 in excess of $1,000,000\n$1,800,001 - $5,000,000\t$8,950 + 0.018 dollars for each $1 in excess of $1,800,000\n$5,000,001 - $11,000,000\t$66,550 + 0.02 dollars for each $1 in excess of $5,000,000\n$11,000,001 +\t$186,550 + 0.0267 dollars for each $1 in excess of $11,000,000",
    },
    Quote {
        kind: "land_tax",
        jurisdiction: "sa",
        provision: "Land Tax Act 1936 (SA) s 8A(1) and Sch 1 Pt 2",
        source: "RevenueSA, 'Rates and thresholds', 2026-27 general rates",
        url: "https://www.revenuesa.sa.gov.au/landtax/rates-and-thresholds",
        passage: "2026-27 General Rates\nTotal Taxable Site Value\nAmount of Tax\nDoes not exceed $936,000\nNil\nExceeds $936,000 but not $1,504,000\n$0.50 for every $100 or part of $100 above\n$936,000\nExceeds $1,504,000 but not $2,188,000\n$2,840 plus $1.00 for every $100 or part of $100 above $1,504,000\nExceeds $2,188,000 but not $3,504,000\n$9,680 plus $2.00 for every $100 or part of $100 above $2,188,000\nExceeds $3,504,000\n$36,000 plus $2.40 for every $100 or part of $100 above $3,504,000",
    },
    Quote {
        kind: "land_tax",
        jurisdiction: "tas",
        provision: "Land Tax Rating Act 2000 (Tas) s 6 and Sch 1",
        source: "SRO Tasmania, 'Rates of land tax'",
        url: "https://www.sro.tas.gov.au/land-tax/rates-of-land-tax",
        passage: "Rates from 1 July 2025\nTotal Land Value\nCurrent Tax Scale\n$0 - $124 999.99\nNil\n$125 000 - $499 999.99\n$50 plus 0.45% of value above $125 000\n$500 000 and above\n$1 737.50 plus 1.5% of value above $500 000",
    },
    Quote {
        kind: "land_tax",
        jurisdiction: "act",
        provision: "Land Tax Act 2004 (ACT) s 9(3)(b); DI2026-152 s 6",
        source: "Taxation Administration (Amounts Payable—Land Tax) Determination 2026 (DI2026-152)",
        url: "https://www.legislation.act.gov.au/View/di/2026-152/current/html/2026-152.html",
        passage: "(a) FC (or fixed charge) is $1,778; and\n(b) P (or percentage rate) is the amount listed in column 2 of Table 1 opposite the base value listed in column 1.\nTable 1 Percentage rates—land tax\ncolumn 1\nbase value\ncolumn 2\nP or percentage rate\nless than or equal to $150 000\n0.54% of the base value\nmore than $150 000 but not more than $275 000\n$810 plus 0.64% of the part of the base value that is more than $150 000\nmore than $275 000 but not more than $1 000 000\n$1,610 plus 1.24% of the part of the base value that is more than $275 000\nmore than $1 000 000 but not more than $2 000 000\n$10,600 plus 1.25% of the part of the base value that is more than $1 000 000\nmore than $2 000 000\n$23,100 plus 1.26% of the part of the base value that is more than $2 000 000",
    },
    Quote {
        kind: "first_home_owner_grant",
        jurisdiction: "nsw",
        provision: "First Home Owner Grant (New Homes) Act 2000 (NSW), as Revenue NSW states it (the Act could not be read on 24 November 2026)",
        source: "Revenue NSW, 'First Home Owner (New Homes) Grant'",
        url: "https://www.revenue.nsw.gov.au/grants-schemes/first-home-owner-new-homes-grant",
        passage: "A $10,000 First Home Owner Grant (FHOG) is available when you buy or build your first new home.",
    },
    Quote {
        kind: "first_home_owner_grant",
        jurisdiction: "vic",
        provision: "First Home Owner Grant and Home Buyer Schemes Act 2000 (Vic) s 18(1A)",
        source: "SRO Victoria, 'Understanding the First Home Owner Grant'",
        url: "https://www.sro.vic.gov.au/buying-property/first-home-owner-grant/understanding-first-home-owner-grant",
        passage: "The First Home Owner Grant is a $10,000 payment to help eligible first home buyers buy or build a new home in Victoria.",
    },
    Quote {
        kind: "first_home_owner_grant",
        jurisdiction: "qld",
        provision: "First Home Owner Grant and Other Home Owner Grants Act 2000 (Qld) s 20(1)",
        source: "Queensland Revenue Office, 'First home owner grant eligibility'",
        url: "https://qro.qld.gov.au/property-concessions-grants/first-home-grant/eligibility/",
        passage: "$30,000 for contracts signed on or after 20 November 2023\n$15,000 for contracts signed before 20 November 2023.",
    },
    Quote {
        kind: "first_home_owner_grant",
        jurisdiction: "wa",
        provision: "First Home Owner Grant Act 2000 (WA) s 19",
        source: "Department of Treasury and Finance, 'First home owner grant' (October 2026)",
        url: "https://www.wa.gov.au/system/files/2026-10/first-home-owner-grant.pdf",
        passage: "The first home owner grant (FHOG) is a one-off payment of up to $10,000 to help eligible applicants buy or build their first new home in Western Australia.",
    },
    Quote {
        kind: "first_home_owner_grant",
        jurisdiction: "sa",
        provision: "First Home and Housing Construction Grants Act 2000 (SA) s 18(1)",
        source: "RevenueSA, 'Grant details'",
        url: "https://www.revenuesa.sa.gov.au/FHOG/grant-details",
        passage: "If you are a first home buyer, you may be eligible for the first home owner grant of up to $15,000 if you are purchasing or building a new home.",
    },
    Quote {
        kind: "first_home_owner_grant",
        jurisdiction: "tas",
        provision: "First Home Owner Grant Act 2000 (Tas) s 18(1) and (2)(al) and (am)",
        source: "SRO Tasmania, 'First Home Owner Grant eligibility'",
        url: "https://www.sro.tas.gov.au/first-home-owner/eligibility",
        passage: "Transaction commencement dates between:\nAmount ($)\n1 July 2026 and 30 June 2027\n$20 000\n1 July 2025 and 30 June 2026\n$30 000\n1 July 2024 and 30 June 2025\n$10 000",
    },
    Quote {
        kind: "first_home_owner_grant",
        jurisdiction: "act",
        provision: "First Home Owner Grant Act 2000 (ACT) s 18",
        source: "ACT Revenue Office, 'First Home Owner Grant'",
        url: "https://www.revenue.act.gov.au/home-buyer-assistance/first-home-owner-grant",
        passage: "First Home Owner Grant (FHOG) payments are not available for applicants who enter into a property purchase transaction with a commencement date from 1 July 2019.",
    },
    Quote {
        kind: "first_home_owner_grant",
        jurisdiction: "nt",
        provision: "First Home Owner Grant Act 2000 (NT) s 18(3D) and (3E)",
        source: "First Home Owner Grant Act 2000 (NT), as in force at 1 July 2026",
        url: "https://legislation.nt.gov.au/api/sitecore/Act/PDF?id=11819",
        passage: "(3D) If the commencement date for the transaction is on or after 1 October 2024 but before 1 October 2025, the amount for subsection (1)(b) is:\n(a) if the home is a new home – $50 000; or\n(b) otherwise – $10 000.\n(3E) If the commencement date for the transaction is on or after 1 October 2025 but before 1 October 2027, the amount for subsection (1)(b) is $50 000.",
    },
    Quote {
        kind: "first_home_owner_grant",
        jurisdiction: "nt",
        provision: "First Home Owner Grant Act 2000 (NT) s 18",
        source: "NT Government, 'Buy or build a new home'",
        url: "https://nt.gov.au/property/home-owner-assistance/buy-build-new-home",
        passage: "The HomeGrown Territory Grant offers $50,000 for first-home buyers to put towards building or buying their first home.",
    },
];

fn quote_json(quote: &Quote) -> Value {
    json!({
        "jurisdiction": quote.jurisdiction,
        "provision": quote.provision,
        "source": quote.source,
        "url": quote.url,
        "passage": quote.passage,
        "verified": VERIFIED,
    })
}

fn select(kind: &str, jurisdiction: Option<&str>) -> Vec<Value> {
    QUOTES
        .iter()
        .filter(|q| q.kind == kind && jurisdiction.is_none_or(|j| q.jurisdiction == j))
        .map(quote_json)
        .collect()
}

/// Every quoted passage of worksheet `kind`, for its catalogue entry, or
/// `None` for a worksheet without them.
pub(crate) fn catalogue(kind: &str) -> Option<Value> {
    let all = select(kind, None);
    (!all.is_empty()).then(|| Value::Array(all))
}

/// Adds to a successful `command` result the passages for the jurisdiction
/// its input names.
pub(crate) fn attach(command: &str, input: &Value, result: &mut Value) {
    let Some((_, kind)) = COMMANDS.iter().find(|(c, _)| *c == command) else {
        return;
    };
    if let Some(jurisdiction) = input["jurisdiction"].as_str() {
        result["source_quotes"] = Value::Array(select(kind, Some(jurisdiction)));
    }
}
