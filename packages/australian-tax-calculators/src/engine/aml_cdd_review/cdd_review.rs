// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use serde_json::{Value, json};

use super::{Given, KIND, choice};
use crate::dates;
use crate::engine::{Arg, Result, date_arg, precise, rate, refuse, result};

const CDD_KEYS: [&str; 16] = [
    "last_cdd_date",
    "customer_risk",
    "simplified_cdd_policy_in_place",
    "pep_status",
    "acting_representative_foreign_pep",
    "smr_continue_relationship",
    "unusual_smr_behaviour",
    "fatf_call_for_action_jurisdiction",
    "nested_services_relationship",
    "unusual_service_request",
    "virtual_asset_physical_currency",
    "kyc_veracity_doubt",
    "significant_risk_change",
    "new_pep_status_change",
    "cadence_basis",
    "policy_review_months",
];

struct CddFlags {
    simplified_policy: bool,
    acting_foreign_pep: bool,
    smr_continue: bool,
    unusual_smr_behaviour: bool,
    fatf_call: bool,
    nested: bool,
    unusual: bool,
    va_cash: bool,
    kyc_doubt: bool,
    sig_change: bool,
    new_pep: bool,
}

struct CddFacts {
    last_day: i64,
    risk: &'static str,
    pep: &'static str,
    flags: CddFlags,
    cadence_basis: &'static str,
    policy_months: Option<i64>,
}

fn parse_policy_months(given: &Given<'_>, cadence_basis: &str) -> Result<Option<i64>> {
    match (cadence_basis, given.get("policy_review_months")) {
        ("custom_policy", Some(Arg::Int(m))) if (1..=60).contains(m) => Ok(Some(*m)),
        ("custom_policy", Some(_)) => refuse(
            "aml_cdd_review.months_invalid",
            "policy_review_months must be an integer from 1 to 60.",
        ),
        ("custom_policy", None) => refuse(
            "input.missing_field",
            "policy_review_months is required when cadence_basis is custom_policy.",
        ),
        (_, Some(_)) => refuse(
            "input.field_unknown",
            "policy_review_months is taken only when cadence_basis is custom_policy.",
        ),
        (_, None) => Ok(None),
    }
}

fn parse_cdd_date(given: &Given<'_>) -> Result<i64> {
    let last_day = date_arg(given.need("last_cdd_date")?, "last_cdd_date")?;
    let start = dates::parse("2026-07-01").expect("valid date");
    let end = dates::parse("2027-06-30").expect("valid date");
    if !(start..=end).contains(&last_day) {
        return refuse(
            "aml_cdd_review.date_outside_year",
            "last_cdd_date must fall within 2026-07-01 to 2027-06-30 for period 2026-27.",
        );
    }
    Ok(last_day)
}

fn parse_cdd_enums(given: &Given<'_>) -> Result<(&'static str, &'static str, &'static str)> {
    let Some(risk) = choice(given.need("customer_risk")?, &["low", "medium", "high"]) else {
        return refuse(
            "aml_cdd_review.risk_invalid",
            "customer_risk must be low, medium or high.",
        );
    };
    let Some(pep) = choice(
        given.need("pep_status")?,
        &["none", "foreign", "domestic", "international_organisation"],
    ) else {
        return refuse(
            "aml_cdd_review.pep_invalid",
            "pep_status must be none, foreign, domestic or international_organisation.",
        );
    };
    let Some(cadence_basis) = choice(
        given.need("cadence_basis")?,
        &[
            "austrac_accountant_starter_kit",
            "custom_policy",
            "statutory_only",
        ],
    ) else {
        return refuse(
            "aml_cdd_review.cadence_basis_invalid",
            "cadence_basis must be austrac_accountant_starter_kit, custom_policy or statutory_only.",
        );
    };
    Ok((risk, pep, cadence_basis))
}

fn parse_cdd_flags(given: &Given<'_>) -> Result<CddFlags> {
    Ok(CddFlags {
        simplified_policy: given.need_flag("simplified_cdd_policy_in_place")?,
        acting_foreign_pep: given.need_flag("acting_representative_foreign_pep")?,
        smr_continue: given.need_flag("smr_continue_relationship")?,
        unusual_smr_behaviour: given.need_flag("unusual_smr_behaviour")?,
        fatf_call: given.need_flag("fatf_call_for_action_jurisdiction")?,
        nested: given.need_flag("nested_services_relationship")?,
        unusual: given.need_flag("unusual_service_request")?,
        va_cash: given.need_flag("virtual_asset_physical_currency")?,
        kyc_doubt: given.need_flag("kyc_veracity_doubt")?,
        sig_change: given.need_flag("significant_risk_change")?,
        new_pep: given.need_flag("new_pep_status_change")?,
    })
}

fn parse_cdd_facts(given: &Given<'_>) -> Result<CddFacts> {
    given.only(&CDD_KEYS)?;
    let last_day = parse_cdd_date(given)?;
    let (risk, pep, cadence_basis) = parse_cdd_enums(given)?;
    let policy_months = parse_policy_months(given, cadence_basis)?;
    let flags = parse_cdd_flags(given)?;
    if flags.new_pep && pep == "none" {
        return refuse(
            "aml_cdd_review.pep_change_inconsistent",
            "new_pep_status_change cannot be true when pep_status is none.",
        );
    }
    Ok(CddFacts {
        last_day,
        risk,
        pep,
        flags,
        cadence_basis,
        policy_months,
    })
}

fn collect_ecdd_triggers(f: &CddFacts) -> Vec<&'static str> {
    let mut out = Vec::new();
    if f.risk == "high" {
        out.push("AML/CTF Act 2006 s 32(a): customer ML/TF risk is high");
    }
    if f.flags.smr_continue {
        out.push("AML/CTF Act 2006 s 32(b): suspicious matter reporting obligation arose and designated services continue");
    }
    if f.pep == "foreign" || f.flags.acting_foreign_pep {
        out.push("AML/CTF Act 2006 s 32(c): foreign politically exposed person (covered customer, beneficial owner, behalf person or acting representative)");
    }
    if f.flags.fatf_call {
        out.push("AML/CTF Act 2006 s 32(d): FATF high-risk jurisdiction calling for enhanced due diligence");
    }
    if f.flags.nested {
        out.push("AML/CTF Act 2006 s 32(e): nested services relationship");
    }
    if f.flags.unusual {
        out.push("AML/CTF Act 2006 s 32(f) and AML/CTF Rules 2025 s 6-20: unusual service request");
    }
    if f.flags.va_cash {
        out.push("AML/CTF Act 2006 s 32(f) and AML/CTF Rules 2025 s 6-22(1): physical currency in item 50A virtual asset service");
    }
    out
}

fn collect_sow_sof(f: &CddFacts) -> (&'static str, Vec<&'static str>) {
    let mut mandatory = Vec::new();
    let mut conditional = Vec::new();
    if f.pep == "foreign" {
        mandatory.push(
            "AML/CTF Rules 2025 s 6-23(1)(a): source of wealth and source of funds for foreign PEP",
        );
    } else if matches!(f.pep, "domestic" | "international_organisation") && f.risk == "high" {
        mandatory.push("AML/CTF Rules 2025 s 6-23(1)(b)-(c): source of wealth and source of funds for high-risk domestic or international organisation PEP");
    }
    if f.flags.va_cash {
        mandatory.push("AML/CTF Rules 2025 s 6-22(2)-(3): source of funds and ongoing source of wealth for physical currency in item 50A virtual asset service");
    }
    if f.risk == "high" || f.flags.smr_continue || f.flags.fatf_call {
        conditional.push("AML/CTF Rules 2025 s 6-21: source of wealth and source of funds each required only where relevant to the nature of the customer's ML/TF risk (s 32(a), (b) or (d))");
    }
    let status = if !mandatory.is_empty() {
        "mandatory"
    } else if !conditional.is_empty() {
        "conditional_on_relevance"
    } else {
        "not_mandated_by_rules"
    };
    mandatory.extend(conditional);
    (status, mandatory)
}

fn collect_immediate_triggers(f: &CddFacts) -> Vec<&'static str> {
    let mut out = Vec::new();
    if f.flags.sig_change {
        out.push(
            "AML/CTF Act 2006 s 30(2)(b)(i): significant change to s 28(4) ML/TF risk matters",
        );
    }
    if f.flags.unusual_smr_behaviour || f.flags.smr_continue {
        out.push("AML/CTF Act 2006 s 30(2)(b)(ii): unusual transactions or behaviours that may give rise to an SMR obligation");
    }
    if f.flags.kyc_doubt {
        out.push(
            "AML/CTF Act 2006 s 30(2)(c)(i): doubts about adequacy or veracity of KYC information",
        );
    }
    if f.flags.new_pep {
        match (f.pep, f.risk) {
            ("foreign", _) => out.push("AML/CTF Act 2006 s 30(2)(c)(ii) and AML/CTF Rules 2025 s 6-24(1)(a): became a foreign PEP"),
            ("domestic", "high") => out.push("AML/CTF Act 2006 s 30(2)(c)(ii) and AML/CTF Rules 2025 s 6-24(1)(b): became a domestic PEP with high ML/TF risk"),
            ("international_organisation", "high") => out.push("AML/CTF Act 2006 s 30(2)(c)(ii) and AML/CTF Rules 2025 s 6-24(1)(c): became an international organisation PEP with high ML/TF risk"),
            _ => {}
        }
    }
    out
}

fn resolve_cadence(
    f: &CddFacts,
    ecdd_mandatory: bool,
) -> Result<(Option<i64>, Option<String>, bool, &'static str)> {
    match f.cadence_basis {
        "austrac_accountant_starter_kit" => {
            if ecdd_mandatory && f.risk != "high" {
                return refuse(
                    "aml_cdd_review.starter_kit_risk_inconsistent",
                    "Worksheet safeguard: when a mandatory enhanced CDD obligation under AML/CTF Act 2006 s 32 applies, this worksheet refuses a low (36-month) or medium (24-month) starter-kit review interval; reassess customer_risk to high or use custom_policy or statutory_only.",
                );
            }
            let months = match f.risk {
                "high" => 12,
                "medium" => 24,
                _ => 36,
            };
            let next = dates::format(dates::add_months(f.last_day, months));
            Ok((
                Some(months),
                Some(next),
                false,
                "AUSTRAC Accountant Program Starter Kit (Step 2: Use your accounting program) default for small accounting practices (12/24/36 months); not a fixed statutory interval under AML/CTF Act 2006 s 30(2)(c)",
            ))
        }
        "custom_policy" => {
            let months = f.policy_months.expect("validated custom_policy months");
            let next = dates::format(dates::add_months(f.last_day, months));
            Ok((
                Some(months),
                Some(next),
                false,
                "Reporting entity's AML/CTF policies under AML/CTF Act 2006 s 30(2)(c) and AML/CTF Rules 2025 s 5-2(3); not a fixed statutory interval",
            ))
        }
        _ => Ok((
            None,
            None,
            true,
            "AML/CTF Act 2006 s 30(2)(c) requires review at a frequency appropriate to the customer's ML/TF risk and sets no fixed calendar interval",
        )),
    }
}

pub(super) fn cdd_review(given: &Given<'_>, period: &str) -> Result<Value> {
    let f = parse_cdd_facts(given)?;
    let ecdd = collect_ecdd_triggers(&f);
    let (sow_sof_status, sow_sof_triggers) = collect_sow_sof(&f);
    let immediate = collect_immediate_triggers(&f);
    let ecdd_mandatory = !ecdd.is_empty();
    let cdd_tier = if ecdd_mandatory {
        "enhanced_mandatory"
    } else if f.risk == "low" && f.flags.simplified_policy {
        "simplified_permitted"
    } else {
        "standard"
    };
    let (months, next_date, is_statutory, authority) = resolve_cadence(&f, ecdd_mandatory)?;
    let mut rates = vec![rate("last_cdd_date", dates::format(f.last_day))];
    if let Some(m) = months {
        rates.push(rate("review_interval_months", m));
    }
    if let Some(ref d_str) = next_date {
        rates.push(rate("next_periodic_review_date", d_str));
    }
    let mut out = precise(|| result(KIND, period, vec![], rates))?;
    out["question"] = json!("cdd_review");
    out["cdd_tier"] = json!(cdd_tier);
    out["ecdd_triggers"] = json!(ecdd);
    out["source_of_wealth_and_funds_status"] = json!(sow_sof_status);
    out["sow_sof_triggers"] = json!(sow_sof_triggers);
    out["immediate_review_required"] = json!(!immediate.is_empty());
    out["immediate_review_triggers"] = json!(immediate);
    out["cadence_basis"] = json!(f.cadence_basis);
    out["cadence_is_statutory"] = json!(is_statutory);
    out["review_interval_months"] = json!(months);
    out["next_periodic_review_date"] = json!(next_date);
    out["cadence_authority"] = json!(authority);
    Ok(out)
}
