// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `partnership-net-income`: a partnership's net income or partnership loss
//! (ITAA 1936 s 90) and each partner's share of it, and of exempt and
//! non-assessable non-exempt income (s 92). Added after the Python port.
use crate::api::{array, text};
use crate::decimal::Decimal;
use crate::engine::d;
use crate::figures::{
    Result, amount, fact, money, out_of_scope, percentage, refuse, share_out, year,
};
use serde_json::{Value, json};

const OUT_OF_SCOPE: [(&str, &str, &str); 6] = [
    (
        "corporate_limited_partnership",
        "a corporate limited partnership is taxed as a company",
        "Div 5A ITAA 1936",
    ),
    (
        "venture_capital_partnership",
        "a VCLP, ESVCLP, AFOF or VCMP limits limited partners' loss deductions",
        "ss 92(2AA), 92A ITAA 1936",
    ),
    (
        "foreign_hybrid",
        "a foreign hybrid limited partnership limits partners' loss deductions",
        "s 92(2) ITAA 1936; s 830-45 ITAA 1997",
    ),
    (
        "uncontrolled_partnership_income",
        "a partner without real and effective control of their share is taxed under s 94",
        "s 94 ITAA 1936",
    ),
    (
        "segregated_or_pension_asset_interest",
        "a partner's interest that is a segregated exempt asset or segregated current pension asset takes no loss deduction",
        "s 92(2A) ITAA 1936",
    ),
    (
        "partner_salaries_or_interest",
        "salary or interest allocated to partners changes each partner's individual interest and is not modelled",
        "s 92 ITAA 1936",
    ),
];

struct Partner {
    name: String,
    residency: &'static str,
    interest: Decimal,
}

fn partner(p: &Value, australian: Option<bool>) -> Result<Partner> {
    let residency = match text(p, "residency")? {
        "resident" => "resident",
        "non_resident" if australian == Some(true) => "non_resident",
        "non_resident" => {
            return Err(refuse(
                "a non-resident partner's share is limited to income from Australian sources; state all_income_from_australian_sources true or leave the partner out",
                "s 92(1)(b) ITAA 1936",
            ));
        }
        "part_year" => {
            return Err(refuse(
                "a partner resident for part of the year takes shares attributable to periods",
                "s 92(1) ITAA 1936",
            ));
        }
        _ => return Err("residency must be resident, non_resident or part_year".into()),
    };
    Ok(Partner {
        name: text(p, "name")?.into(),
        residency,
        interest: percentage(p, "interest_percentage")?,
    })
}

fn partners(v: &Value) -> Result<Vec<Partner>> {
    let australian = fact(v, "all_income_from_australian_sources")?;
    let list = array(v, "partners")?
        .iter()
        .map(|p| partner(p, australian))
        .collect::<Result<Vec<_>>>()?;
    let total = list.iter().try_fold(d("0"), |t, p| t.add(&p.interest))?;
    if list.is_empty() || !total.equals(&d("100"))? {
        return Err(format!(
            "partners' interest_percentage must total 100, not {total}"
        ));
    }
    Ok(list)
}

const STEPS: [&str; 5] = [
    "The partnership lodges a return but is not liable to tax on its income (s 91).",
    "Net income: assessable income less deductions other than those under s 290-150 or Division 36 ITAA 1997, as if the partnership were a resident taxpayer; a partnership loss is the excess of those deductions (s 90).",
    "Each partner includes their individual interest in the net income, or deducts their interest in the partnership loss (s 92(1), (2)), and includes their interest in exempt and non-assessable non-exempt income (s 92(3), (4)).",
    "A non-resident partner takes only the part from Australian sources; this command decides that only when all the partnership's income is from Australian sources (s 92(1)(b) to (4)(b)).",
    "Shares are rounded half up to the cent; cents left over go to the largest shares first, so shares add up to the partnership's figure.",
];

/// Assessable income, counted deductions, excluded deductions, net income and partnership loss (s 90).
fn net_income(v: &Value) -> Result<[Decimal; 5]> {
    let assessable = money(v, "assessable_income")?;
    let deductions = money(v, "deductions")?;
    let excluded = money(v, "excluded_deductions")?;
    if excluded.gt(&deductions)? {
        return Err("excluded_deductions is part of deductions and cannot exceed it".into());
    }
    let counted = deductions.sub(&excluded)?;
    let (net, loss) = if assessable.lt(&counted)? {
        (d("0"), counted.sub(&assessable)?)
    } else {
        (assessable.sub(&counted)?, d("0"))
    };
    Ok([assessable, counted, excluded, net, loss])
}

pub fn evaluate(v: &Value) -> Result<Value> {
    let fy = year(v)?;
    out_of_scope(v, &OUT_OF_SCOPE)?;
    let [assessable, counted, excluded, net, loss] = net_income(v)?;
    let exempt = money(v, "exempt_income")?;
    let nane = money(v, "non_assessable_non_exempt_income")?;
    let list = partners(v)?;
    let weights = list
        .iter()
        .map(|p| p.interest.div(&d("100")))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let split = [&net, &loss, &exempt, &nane]
        .iter()
        .map(|x| share_out(x, &weights))
        .collect::<Result<Vec<_>>>()?;
    let rows: Vec<Value> = list
        .iter()
        .enumerate()
        .map(|(i, p)| {
            json!({"name": p.name, "residency": p.residency,
                "interest_percentage": p.interest.to_string(),
                "assessable_income": amount(&split[0][i]), "deductible_loss": amount(&split[1][i]),
                "exempt_income": amount(&split[2][i]),
                "non_assessable_non_exempt_income": amount(&split[3][i])})
        })
        .collect();
    Ok(
        json!({"financial_year": fy, "assessable_income": amount(&assessable),
        "deductions_counted": amount(&counted), "excluded_deductions": amount(&excluded),
        "net_income": amount(&net), "partnership_loss": amount(&loss),
        "exempt_income": amount(&exempt), "non_assessable_non_exempt_income": amount(&nane),
        "partners": rows, "steps": STEPS}),
    )
}
