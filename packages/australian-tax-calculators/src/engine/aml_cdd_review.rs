// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `aml_cdd_review`: deterministic Australian AML/CTF Tranche 2 customer due
//! diligence (CDD) helper for 2026-27 with zero personal data. Covers 3
//! questions: `country_risk` (AUSTRAC Basel AML Index 0-10 bands with FATF and
//! Australian country-sanctions overrides and conditional s 32(d) notice),
//! `beneficial_ownership` (AML/CTF Act 2006 ss 5, 11 and 28(2)(d) exact 25%
//! indirect chain ownership and control tests with Rules ss 6-7(1)-(1C) and
//! 6-8), and `cdd_review` (business-relationship ongoing CDD under ss 30 to 32
//! and Rules ss 5-2, 6-16 and 6-20 to 6-24, cleanly separated from AUSTRAC
//! Accountant Program Starter Kit 1/2/3-year defaults and custom firm policies).
//!
//! Read on 24 November 2026: Anti-Money Laundering and Counter-Terrorism
//! Financing Act 2006 (C2006A00169, compilation 63, 1 October 2026);
//! Anti-Money Laundering and Counter-Terrorism Financing Rules 2025
//! (F2025L01026, compilation 1, 31 March 2026); AUSTRAC Step 2: Identify and
//! assess your risks; AUSTRAC Accountant program starter kit: Step 2; AUSTRAC
//! Determining ownership and control structures; AUSTRAC Reviewing and
//! updating customers' ML/TF risk and KYC information (29 October 2026).

use std::collections::BTreeSet;

use num_bigint::BigUint;
use serde_json::{Value, json};

use super::{Arg, Result, d, date_arg, flag, int, precise, rate, refuse, result, scope};
use crate::dates;
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "aml_cdd_review";
const QUESTIONS: [&str; 3] = ["country_risk", "beneficial_ownership", "cdd_review"];
const MAX_CHAINS: usize = 16;
const MAX_LAYERS: usize = 8;
const SCALE_EXPONENT: u32 = 46; // 6 * MAX_LAYERS - 2

const COUNTRY_KEYS: [&str; 4] = [
    "basel_aml_score",
    "fatf_call_for_action",
    "fatf_increased_monitoring",
    "australian_sanctioned_jurisdiction",
];
const BO_BODY_KEYS: [&str; 7] = [
    "entity_kind",
    "customer_exemption",
    "ownership_chains",
    "board_control",
    "financial_operating_policy_control",
    "voting_power_over_50",
    "issued_share_capital_over_50",
];
const BO_NON_BODY_KEYS: [&str; 5] = [
    "entity_kind",
    "customer_exemption",
    "ownership_chains",
    "board_control",
    "financial_operating_policy_control",
];
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

struct Given<'a> {
    question: &'a str,
    named: &'a [(String, Arg)],
}

impl<'a> Given<'a> {
    fn check_unique(&self) -> Result<()> {
        let mut seen = BTreeSet::new();
        for (k, _) in self.named {
            if !seen.insert(k.as_str()) {
                return refuse(
                    "aml_cdd_review.duplicate_field",
                    format!(
                        "{k} was supplied more than once for question {}.",
                        self.question
                    ),
                );
            }
        }
        Ok(())
    }

    fn get(&self, key: &str) -> Option<&'a Arg> {
        self.named.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    fn need(&self, key: &str) -> Result<&'a Arg> {
        match self.get(key) {
            Some(arg) => Ok(arg),
            None => refuse(
                "input.missing_field",
                format!("{key} is required for question {}.", self.question),
            ),
        }
    }

    fn need_flag(&self, key: &str) -> Result<bool> {
        flag(self.need(key)?, key)
    }

    fn only(&self, takes: &[&str]) -> Result<()> {
        match self
            .named
            .iter()
            .find(|(k, _)| !takes.contains(&k.as_str()))
        {
            Some((key, _)) => refuse(
                "input.field_unknown",
                format!("{key} is not an input of question {}.", self.question),
            ),
            None => Ok(()),
        }
    }
}

fn choice(arg: &Arg, allowed: &[&'static str]) -> Option<&'static str> {
    arg.text()
        .and_then(|t| allowed.iter().copied().find(|a| *a == t))
}

/// Strict unsigned decimal string (digits and optional single decimal point),
/// finite, with at most `max_dp` decimal places.
fn parse_plain_decimal(text: &str, max_dp: i128) -> Option<Decimal> {
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit() || b == b'.') {
        return None;
    }
    let val = Decimal::parse(text).ok()?;
    (val.is_finite() && val.exponent() >= -max_dp).then_some(val)
}

fn parse_basel_score(arg: &Arg) -> Result<Decimal> {
    let bad = || {
        refuse(
            "aml_cdd_review.basel_score_invalid",
            "basel_aml_score must be a decimal string from 0.00 to 10.00 with at most 2 decimal places.",
        )
    };
    let Some(val) = arg.text().and_then(|t| parse_plain_decimal(t, 2)) else {
        return bad();
    };
    if val.lt(&int(0))? || val.gt(&int(10))? {
        return bad();
    }
    Ok(val.quantize(-2, Rounding::HalfUp)?)
}

fn country_risk(given: &Given<'_>, period: &str) -> Result<Value> {
    given.only(&COUNTRY_KEYS)?;
    let score = parse_basel_score(given.need("basel_aml_score")?)?;
    let call_for_action = given.need_flag("fatf_call_for_action")?;
    let grey_list = given.need_flag("fatf_increased_monitoring")?;
    let sanctioned = given.need_flag("australian_sanctioned_jurisdiction")?;
    let basel_band = if score.le(&d("5.00"))? {
        "low"
    } else if score.le(&d("6.00"))? {
        "medium"
    } else {
        "high"
    };
    let override_applied = call_for_action || grey_list || sanctioned;
    let rating = if override_applied { "high" } else { basel_band };
    let mut provisions = Vec::new();
    if call_for_action {
        provisions.push("AML/CTF Act 2006 s 32(d): mandatory enhanced CDD applies only if a covered individual is physically present in, or a covered body corporate or legal arrangement was formed in, this FATF Call-for-Action jurisdiction");
        provisions.push("AML/CTF Rules 2025 s 6-21: if s 32(d) applies, establish and hold source of wealth and source of funds where relevant to the nature of the customer's ML/TF risk");
    }
    let rates = vec![
        rate("basel_aml_score", &score),
        rate("basel_low_max", "5.00"),
        rate("basel_medium_max", "6.00"),
        rate("basel_high_max", "10.00"),
    ];
    let mut out = precise(|| result(KIND, period, vec![], rates))?;
    out["question"] = json!("country_risk");
    out["basel_band"] = json!(basel_band);
    out["override_applied"] = json!(override_applied);
    out["country_risk_rating"] = json!(rating);
    out["fatf_call_for_action_ecdd_applies_if_covered_connection"] = json!(call_for_action);
    out["statutory_provisions"] = json!(provisions);
    out["guidance_authority"] = json!(
        "AUSTRAC, Step 2: Identify and assess your risks (Basel AML Index 0-5 low, 5.01-6 medium, 6.01-10.00 high; FATF grey/blacklist and Australian sanctions high-risk override)"
    );
    Ok(out)
}

/// Parse an exact percentage string (> 0 to 100, at most 4 decimal places) as
/// a scaled integer in ten-thousandths of 1% (1..=1_000_000).
fn parse_scaled_layer(item: &Value) -> Result<u64> {
    let bad = || {
        refuse(
            "aml_cdd_review.chains_invalid",
            "ownership_chains must be a list of up to 16 chains, each a list of 1 to 8 percentage strings above 0 to 100 with at most 4 decimal places.",
        )
    };
    let Some(s) = item.as_str() else {
        return bad();
    };
    let (whole_s, frac_s) = match s.split_once('.') {
        Some((w, f)) if !w.is_empty() && !f.is_empty() && f.len() <= 4 => (w, f),
        Some(_) => return bad(),
        None if !s.is_empty() => (s, ""),
        None => return bad(),
    };
    if !whole_s.bytes().all(|b| b.is_ascii_digit()) || !frac_s.bytes().all(|b| b.is_ascii_digit()) {
        return bad();
    }
    let Some(whole) = whole_s.parse::<u64>().ok().filter(|&w| w <= 100) else {
        return bad();
    };
    let frac = if frac_s.is_empty() {
        0
    } else {
        frac_s.parse::<u64>().unwrap() * 10u64.pow((4 - frac_s.len()) as u32)
    };
    let scaled = whole * 10_000 + frac;
    if !(1..=1_000_000).contains(&scaled) {
        return bad();
    }
    Ok(scaled)
}

/// Exact combined ownership numerator over `10^46`, with zero intermediate rounding.
fn compute_effective_ownership(arg: &Arg) -> Result<BigUint> {
    let bad = || {
        refuse(
            "aml_cdd_review.chains_invalid",
            "ownership_chains must be a list of up to 16 chains, each a list of 1 to 8 percentage strings above 0 to 100 with at most 4 decimal places.",
        )
    };
    let Arg::Json(Value::Array(chains)) = arg else {
        return bad();
    };
    if chains.len() > MAX_CHAINS {
        return bad();
    }
    let ten = BigUint::from(10u8);
    let mut total = BigUint::ZERO;
    for chain in chains {
        let Some(layers) = chain.as_array() else {
            return bad();
        };
        if layers.is_empty() || layers.len() > MAX_LAYERS {
            return bad();
        }
        let mut prod = BigUint::from(1u8);
        for layer in layers {
            prod *= BigUint::from(parse_scaled_layer(layer)?);
        }
        let lift = ten.pow(6 * (MAX_LAYERS - layers.len()) as u32);
        total += prod * lift;
    }
    let hundred_scaled = BigUint::from(100u8) * ten.pow(SCALE_EXPONENT);
    if total > hundred_scaled {
        return refuse(
            "aml_cdd_review.ownership_above_100",
            "Combined effective ownership across ownership_chains exceeds 100%.",
        );
    }
    Ok(total)
}

/// Format an exact integer scaled by `10^46` as a decimal string with at least
/// 2 decimal places and no superfluous trailing zeroes beyond the second place.
fn format_scaled_percent(num: &BigUint) -> String {
    let digits = num.to_string();
    let scale = SCALE_EXPONENT as usize;
    let (whole, frac) = if digits.len() <= scale {
        (
            "0".to_string(),
            format!("{}{digits}", "0".repeat(scale - digits.len())),
        )
    } else {
        let (w, f) = digits.split_at(digits.len() - scale);
        (w.to_string(), f.to_string())
    };
    let trimmed = frac.trim_end_matches('0');
    match trimmed.len() {
        0 => format!("{whole}.00"),
        1 => format!("{whole}.{trimmed}0"),
        _ => format!("{whole}.{trimmed}"),
    }
}

fn control_grounds(given: &Given<'_>, is_body: bool) -> Result<Vec<&'static str>> {
    let mut grounds = Vec::new();
    if is_body {
        if given.need_flag("voting_power_over_50")? {
            grounds.push(
                "AML/CTF Act 2006 s 11(1)(a): capacity to cast or control more than 50% of votes",
            );
        }
        if given.need_flag("issued_share_capital_over_50")? {
            grounds.push("AML/CTF Act 2006 s 11(1)(b): holds more than 50% of qualifying issued share capital");
        }
    }
    if given.need_flag("board_control")? {
        grounds.push(if is_body {
            "AML/CTF Act 2006 s 11(1)(c): capacity to control composition of board or governing body"
        } else {
            "AML/CTF Act 2006 s 11(2)(a): capacity to control composition of board or governing body"
        });
    }
    if given.need_flag("financial_operating_policy_control")? {
        grounds.push(if is_body {
            "AML/CTF Act 2006 s 11(1)(d): capacity to determine financial and operating policy decisions"
        } else {
            "AML/CTF Act 2006 s 11(2)(b): capacity to determine financial and operating policy decisions"
        });
    }
    Ok(grounds)
}

fn parse_bo_header(given: &Given<'_>) -> Result<(&'static str, &'static str)> {
    given.only(&BO_BODY_KEYS)?;
    let Some(entity_kind) = choice(
        given.need("entity_kind")?,
        &["body_corporate", "non_body_corporate"],
    ) else {
        return refuse(
            "aml_cdd_review.entity_kind_invalid",
            "entity_kind must be body_corporate or non_body_corporate.",
        );
    };
    if entity_kind == "non_body_corporate" {
        given.only(&BO_NON_BODY_KEYS)?;
    }
    let Some(exemption) = choice(
        given.need("customer_exemption")?,
        &[
            "none",
            "listed_public_company",
            "government_body",
            "controlled_by_listed_or_government",
            "owned_through_listed_or_government_only",
        ],
    ) else {
        return refuse(
            "aml_cdd_review.exemption_invalid",
            "customer_exemption must be none, listed_public_company, government_body, controlled_by_listed_or_government or owned_through_listed_or_government_only.",
        );
    };
    if exemption == "listed_public_company" && entity_kind != "body_corporate" {
        return refuse(
            "aml_cdd_review.exemption_inconsistent",
            "listed_public_company exemption applies only when entity_kind is body_corporate.",
        );
    }
    Ok((entity_kind, exemption))
}

fn exemption_authority(exemption: &str) -> Option<&'static str> {
    match exemption {
        "listed_public_company" => Some(
            "AML/CTF Rules 2025 s 6-7(1): listed public company subject to public disclosure requirements",
        ),
        "government_body" => Some("AML/CTF Rules 2025 s 6-7(1A): government body"),
        "controlled_by_listed_or_government" => Some(
            "AML/CTF Rules 2025 s 6-7(1B): customer controlled by a qualifying listed public company or government body",
        ),
        "owned_through_listed_or_government_only" => Some(
            "AML/CTF Rules 2025 s 6-7(1C): individual is a beneficial owner solely through a qualifying listed public company or government body owning part of the customer",
        ),
        _ => None,
    }
}

fn beneficial_ownership(given: &Given<'_>, period: &str) -> Result<Value> {
    let (entity_kind, exemption) = parse_bo_header(given)?;
    let is_body = entity_kind == "body_corporate";
    let effective = compute_effective_ownership(given.need("ownership_chains")?)?;
    let grounds = control_grounds(given, is_body)?;
    let threshold = BigUint::from(25u8) * BigUint::from(10u8).pow(SCALE_EXPONENT);
    let owns_25 = effective >= threshold;
    let controls = !grounds.is_empty();
    let is_bo = owns_25 || controls;
    let deemed = exemption != "none";
    let rates = vec![
        rate("beneficial_ownership_threshold_percent", "25.00"),
        rate(
            "effective_ownership_percent",
            format_scaled_percent(&effective),
        ),
    ];
    let mut out = precise(|| result(KIND, period, vec![], rates))?;
    out["question"] = json!("beneficial_ownership");
    out["entity_kind"] = json!(entity_kind);
    out["owns_25_or_more"] = json!(owns_25);
    out["controls_customer"] = json!(controls);
    out["control_grounds"] = json!(grounds);
    out["is_beneficial_owner"] = json!(is_bo);
    out["beneficial_owner_cdd_deemed_established"] = json!(deemed);
    out["exemption_authority"] = json!(exemption_authority(exemption));
    out["identification_required"] = json!(!deemed && is_bo);
    out["ceo_fallback"] = json!(
        "AML/CTF Rules 2025 s 6-8(1)-(3): where a body corporate, partnership or unincorporated association has no beneficial owners or none can be established after all reasonable steps, establish the identity of the chief executive officer (or equivalent)."
    );
    Ok(out)
}

struct CddFacts {
    last_day: i64,
    risk: &'static str,
    simplified_policy: bool,
    pep: &'static str,
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

fn parse_cdd_facts(given: &Given<'_>) -> Result<CddFacts> {
    given.only(&CDD_KEYS)?;
    let last_day = parse_cdd_date(given)?;
    let (risk, pep, cadence_basis) = parse_cdd_enums(given)?;
    let policy_months = parse_policy_months(given, cadence_basis)?;
    let new_pep = given.need_flag("new_pep_status_change")?;
    if new_pep && pep == "none" {
        return refuse(
            "aml_cdd_review.pep_change_inconsistent",
            "new_pep_status_change cannot be true when pep_status is none.",
        );
    }
    Ok(CddFacts {
        last_day,
        risk,
        simplified_policy: given.need_flag("simplified_cdd_policy_in_place")?,
        pep,
        acting_foreign_pep: given.need_flag("acting_representative_foreign_pep")?,
        smr_continue: given.need_flag("smr_continue_relationship")?,
        unusual_smr_behaviour: given.need_flag("unusual_smr_behaviour")?,
        fatf_call: given.need_flag("fatf_call_for_action_jurisdiction")?,
        nested: given.need_flag("nested_services_relationship")?,
        unusual: given.need_flag("unusual_service_request")?,
        va_cash: given.need_flag("virtual_asset_physical_currency")?,
        kyc_doubt: given.need_flag("kyc_veracity_doubt")?,
        sig_change: given.need_flag("significant_risk_change")?,
        new_pep,
        cadence_basis,
        policy_months,
    })
}

fn collect_ecdd_triggers(f: &CddFacts) -> Vec<&'static str> {
    let mut out = Vec::new();
    if f.risk == "high" {
        out.push("AML/CTF Act 2006 s 32(a): customer ML/TF risk is high");
    }
    if f.smr_continue {
        out.push("AML/CTF Act 2006 s 32(b): suspicious matter reporting obligation arose and designated services continue");
    }
    if f.pep == "foreign" || f.acting_foreign_pep {
        out.push("AML/CTF Act 2006 s 32(c): foreign politically exposed person (covered customer, beneficial owner, behalf person or acting representative)");
    }
    if f.fatf_call {
        out.push("AML/CTF Act 2006 s 32(d): FATF high-risk jurisdiction calling for enhanced due diligence");
    }
    if f.nested {
        out.push("AML/CTF Act 2006 s 32(e): nested services relationship");
    }
    if f.unusual {
        out.push("AML/CTF Act 2006 s 32(f) and AML/CTF Rules 2025 s 6-20: unusual service request");
    }
    if f.va_cash {
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
    if f.va_cash {
        mandatory.push("AML/CTF Rules 2025 s 6-22(2)-(3): source of funds and ongoing source of wealth for physical currency in item 50A virtual asset service");
    }
    if f.risk == "high" || f.smr_continue || f.fatf_call {
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
    if f.sig_change {
        out.push(
            "AML/CTF Act 2006 s 30(2)(b)(i): significant change to s 28(4) ML/TF risk matters",
        );
    }
    if f.unusual_smr_behaviour || f.smr_continue {
        out.push("AML/CTF Act 2006 s 30(2)(b)(ii): unusual transactions or behaviours that may give rise to an SMR obligation");
    }
    if f.kyc_doubt {
        out.push(
            "AML/CTF Act 2006 s 30(2)(c)(i): doubts about adequacy or veracity of KYC information",
        );
    }
    if f.new_pep {
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

fn cdd_review(given: &Given<'_>, period: &str) -> Result<Value> {
    let f = parse_cdd_facts(given)?;
    let ecdd = collect_ecdd_triggers(&f);
    let (sow_sof_status, sow_sof_triggers) = collect_sow_sof(&f);
    let immediate = collect_immediate_triggers(&f);
    let ecdd_mandatory = !ecdd.is_empty();
    let cdd_tier = if ecdd_mandatory {
        "enhanced_mandatory"
    } else if f.risk == "low" && f.simplified_policy {
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

pub fn aml_cdd_review(
    question: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
    named: &[(String, Arg)],
) -> Result<Value> {
    let period = scope(scope_confirmed, year, KIND)?;
    let Some(question) = choice(question, &QUESTIONS) else {
        return refuse(
            "aml_cdd_review.question_invalid",
            format!("question must be one of {}.", QUESTIONS.join(", ")),
        );
    };
    let given = Given { question, named };
    given.check_unique()?;
    match question {
        "country_risk" => country_risk(&given, &period),
        "beneficial_ownership" => beneficial_ownership(&given, &period),
        _ => cdd_review(&given, &period),
    }
}
