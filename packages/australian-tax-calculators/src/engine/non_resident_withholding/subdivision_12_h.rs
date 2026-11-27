// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use serde_json::{Value, json};

use super::{Facts, Worked, step};
use crate::decimal::Decimal;
use crate::engine::{Result, cents, d, flag, int, money, rate, refuse};

const MIT_EOI_RATE: &str = "0.15";
const MIT_CLEAN_BUILDING_RATE: &str = "0.10";
const MIT_NON_CONCESSIONAL_RATE: &str = "0.30";
const MIT_NON_EOI_RATE: &str = "0.30";

struct MitOutcome {
    withholding: Decimal,
    components: Value,
    rates: Vec<(String, String)>,
}

fn mit_component(
    name: &str,
    amount: &Decimal,
    rate_str: &str,
    wh: &Decimal,
    authority: &str,
) -> Result<Value> {
    Ok(json!({
        "component": name,
        "amount": cents(amount.clone())?.to_string(),
        "rate": rate_str,
        "withholding": cents(wh.clone())?.to_string(),
        "authority": authority
    }))
}

fn mit_zero_case(
    payment: &Decimal,
    component: &str,
    step_auth: &str,
    comp_auth: &str,
    steps: &mut Vec<Value>,
) -> Result<MitOutcome> {
    let zero = int(0);
    steps.push(step("withholding", step_auth, &zero));
    let comp = mit_component(component, payment, "0", &zero, comp_auth)?;
    Ok(MitOutcome {
        withholding: zero,
        components: json!([comp]),
        rates: vec![rate("applied_rate", "0")],
    })
}

fn mit_non_eoi_case(payment: &Decimal, steps: &mut Vec<Value>) -> Result<MitOutcome> {
    let r = d(MIT_NON_EOI_RATE);
    let wh = cents(r.mul(payment)?)?;
    steps.push(step(
        "withholding",
        "TAA Sch 1 ss 12-385(2), (3)(b) and 12-390(2), (3)(b), (5), (6)(b); IT(MITWHT)A 2008 s 4(1)(b): 30% for a recipient not in an information exchange country (rounded half up to the cent; s 15-10(3) does not engage TAR 2017 s 26)",
        &wh,
    ));
    let comp = mit_component(
        "non_eoi_fund_payment",
        payment,
        MIT_NON_EOI_RATE,
        &wh,
        "TAA Sch 1 ss 12-385(3)(b), 12-390(3)(b) and (6)(b); IT(MITWHT)A 2008 s 4(1)(b)",
    );
    let comp = comp?;
    Ok(MitOutcome {
        withholding: wh,
        components: json!([comp]),
        rates: vec![rate("non_eoi_rate", MIT_NON_EOI_RATE)],
    })
}

struct MitParts {
    standard: Decimal,
    clean: Decimal,
    ncmi: Decimal,
    company_non_agent: bool,
    exempt: bool,
    eoi: bool,
}

fn mit_eoi_components(
    parts: &MitParts,
    std_wh: &Decimal,
    clean_wh: &Decimal,
    ncmi_wh: &Decimal,
) -> Result<Value> {
    Ok(json!([
        mit_component(
            "standard",
            &parts.standard,
            MIT_EOI_RATE,
            std_wh,
            "TAA Sch 1 ss 12-385(3)(a)(i), 12-390(3)(a)(i) and (6)(a)(i); IT(MITWHT)A 2008 s 4(1)(a)(i)",
        )?,
        mit_component(
            "clean_building_mit",
            &parts.clean,
            MIT_CLEAN_BUILDING_RATE,
            clean_wh,
            "TAA Sch 1 ss 12-385(3)(a)(ii), 12-390(3)(a)(ii), (6)(a)(ii) and 12-425; IT(MITWHT)A 2008 s 4(1)(a)(ii)",
        )?,
        mit_component(
            "non_concessional_mit_income",
            &parts.ncmi,
            MIT_NON_CONCESSIONAL_RATE,
            ncmi_wh,
            "TAA Sch 1 ss 12-385(3)(a)(iii), 12-390(3)(a)(iii), (6)(a)(iii) and 12-435; IT(MITWHT)A 2008 s 4(1)(a)(iii)",
        )?,
    ]))
}

fn mit_eoi_case(parts: &MitParts, steps: &mut Vec<Value>) -> Result<MitOutcome> {
    let std_wh = cents(d(MIT_EOI_RATE).mul(&parts.standard)?)?;
    let clean_wh = cents(d(MIT_CLEAN_BUILDING_RATE).mul(&parts.clean)?)?;
    let ncmi_wh = cents(d(MIT_NON_CONCESSIONAL_RATE).mul(&parts.ncmi)?)?;
    let total = std_wh.add(&clean_wh)?.add(&ncmi_wh)?;
    steps.push(step(
        "standard_withholding",
        "TAA Sch 1 ss 12-385(3)(a)(i) and 12-390(3)(a)(i), (6)(a)(i); IT(MITWHT)A 2008 s 4(1)(a)(i): 15% on standard fund payment (rounded half up to the cent)",
        &std_wh,
    ));
    steps.push(step(
        "clean_building_withholding",
        "TAA Sch 1 ss 12-385(3)(a)(ii), 12-390(3)(a)(ii), (6)(a)(ii) and 12-425; IT(MITWHT)A 2008 s 4(1)(a)(ii): 10% on clean building MIT fund payment (rounded half up to the cent)",
        &clean_wh,
    ));
    steps.push(step(
        "ncmi_withholding",
        "TAA Sch 1 ss 12-385(3)(a)(iii), 12-390(3)(a)(iii), (6)(a)(iii) and 12-435; IT(MITWHT)A 2008 s 4(1)(a)(iii): 30% on non-concessional MIT income (rounded half up to the cent)",
        &ncmi_wh,
    ));
    steps.push(step(
        "withholding",
        "TAA Sch 1 ss 12-385(2) and 12-390(2), (5): sum of the Subdivision 12-H rate parts",
        &total,
    ));
    let comps = mit_eoi_components(parts, &std_wh, &clean_wh, &ncmi_wh)?;
    let rates = vec![
        rate("standard_rate", MIT_EOI_RATE),
        rate("clean_building_rate", MIT_CLEAN_BUILDING_RATE),
        rate("non_concessional_rate", MIT_NON_CONCESSIONAL_RATE),
    ];
    Ok(MitOutcome {
        withholding: total,
        components: comps,
        rates,
    })
}

fn mit_select_branch(
    payment: &Decimal,
    parts: &MitParts,
    steps: &mut Vec<Value>,
) -> Result<MitOutcome> {
    if parts.company_non_agent {
        mit_zero_case(
            payment,
            "company_not_acting_as_agent_exception",
            "TAA Sch 1 s 12-390(10)(a): section 12-390 does not apply to a company unless the company would, apart from section 12-420, be acting in the capacity as agent for the recipient",
            "TAA Sch 1 s 12-390(10)(a)",
            steps,
        )
    } else if parts.exempt {
        mit_zero_case(
            payment,
            "exempt_fund_payment",
            "TAA Sch 1 ss 12-385(5) and 12-390(10)(b): no amount is required to be withheld when no managed investment trust withholding tax is payable",
            "TAA Sch 1 ss 12-385(5) and 12-390(10)(b)",
            steps,
        )
    } else if !parts.eoi {
        mit_non_eoi_case(payment, steps)
    } else {
        mit_eoi_case(parts, steps)
    }
}

fn parse_mit_parts(payment: &Decimal, facts: &Facts) -> Result<MitParts> {
    let eoi = flag(facts.get(3), "information_exchange_country")?;
    let clean = money(facts.get(4))?;
    let ncmi = money(facts.get(5))?;
    let company_non_agent = flag(facts.get(6), "company_not_acting_as_agent")?;
    let exempt = flag(facts.get(11), "exempt")?;
    let special = clean.add(&ncmi)?;
    if special.gt(payment)? {
        return refuse(
            "non_resident_withholding.mit_parts_exceed_payment",
            "clean_building_mit_amount plus non_concessional_mit_income exceeds payment.",
        );
    }
    Ok(MitParts {
        standard: payment.sub(&special)?,
        clean,
        ncmi,
        company_non_agent,
        exempt,
        eoi,
    })
}

/// Subdivision 12-H: withholding MIT fund payments under TAA Sch 1 ss 12-385
/// and 12-390 and Income Tax (Managed Investment Trust Withholding Tax) Act
/// 2008 s 4. TAR 2017 s 26 does not apply (s 15-10(3)); each statutory rate
/// part is rounded half up to the cent.
pub(super) fn subdivision_12_h(payment: &Decimal, facts: &Facts) -> Result<Worked> {
    let parts = parse_mit_parts(payment, facts)?;
    let mut steps = vec![step(
        "standard_fund_payment",
        "TAA Sch 1 ss 12-385(3)(a)(i) and 12-390(3)(a)(i): fund payment other than clean building MIT and non-concessional MIT income parts",
        &parts.standard,
    )];
    let outcome = mit_select_branch(payment, &parts, &mut steps)?;
    let net = payment.sub(&outcome.withholding)?;
    steps.push(step(
        "net_payment",
        "Fund payment less withholding required under Subdivision 12-H",
        &net,
    ));
    Ok(Worked {
        subdivision: "12-H",
        final_tax: true,
        amounts: vec![
            ("standard_fund_payment", parts.standard),
            ("clean_building_mit_amount", parts.clean),
            ("non_concessional_mit_income", parts.ncmi),
            ("withholding", outcome.withholding),
            ("net_payment", net),
        ],
        rates: outcome.rates,
        steps,
        components: Some(outcome.components),
    })
}
