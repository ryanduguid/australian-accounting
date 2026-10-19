//! JSON boundary for the removed Python import API. Amounts are decimal strings;
//! facts are true, false or null. README.md documents each schema.
use crate::cli;
use crate::date::Date;
use crate::decimal::Decimal;
use crate::engine::{
    self, BeneficiaryEntitlement, Error, Fact, ForeignTrustReceipt, Section100AFacts,
    TrustIncomeAssessment, TrustResolutionSchedule,
};
use serde_json::{Value, json};

pub const COMMANDS: &[&str] = &[
    "proportionate-share",
    "s100a-risk",
    "s99b-liability",
    "trust-resolution",
    "validate-amount",
];

type Result<T> = std::result::Result<T, String>;

fn text<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{key} must be a string"))
}

fn integer(v: &Value, key: &str) -> Result<i64> {
    v.get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("{key} must be an integer"))
}

fn is_absent(v: &Value, key: &str) -> bool {
    v.get(key).is_none_or(Value::is_null)
}

fn decimal(v: &Value, key: &str) -> Result<Decimal> {
    let t = text(v, key)?;
    if t.len() > 8192 {
        return Err(format!("{key}: decimal input exceeds 8192 bytes"));
    }
    // No exponent cap: the engine range-checks and overflows astronomical amounts
    // itself, and its tests rely on that.
    Decimal::parse(t).map_err(|_| format!("{key}: not a decimal amount"))
}

fn optional_decimal(v: &Value, key: &str) -> Result<Option<Decimal>> {
    if is_absent(v, key) {
        Ok(None)
    } else {
        decimal(v, key).map(Some)
    }
}

/// A decimal with the dataclass default of 0.00 when absent.
fn amount(v: &Value, key: &str) -> Result<Decimal> {
    Ok(optional_decimal(v, key)?.unwrap_or_else(|| engine::d("0.00")))
}

/// A fact as supplied: the engine refuses a value that is not true, false or null.
fn fact(v: &Value, key: &str) -> Fact {
    match v.get(key) {
        None | Some(Value::Null) => Fact::Unknown,
        Some(Value::Bool(b)) => Fact::Known(*b),
        Some(_) => Fact::NotBool,
    }
}

fn date(v: &Value, key: &str) -> Result<Date> {
    Date::parse(text(v, key)?).ok_or_else(|| format!("{key} must be a YYYY-MM-DD date"))
}

fn array<'a>(v: &'a Value, key: &str) -> Result<&'a [Value]> {
    match v.get(key) {
        None => Ok(&[]),
        Some(Value::Array(a)) => Ok(a),
        _ => Err(format!("{key} must be an array")),
    }
}

fn engine_error(e: Error) -> String {
    e.to_string()
}

fn beneficiary(v: &Value) -> Result<BeneficiaryEntitlement> {
    Ok(BeneficiaryEntitlement {
        beneficiary_name: text(v, "beneficiary_name")?.into(),
        is_resident: fact(v, "is_resident"),
        is_under_legal_disability: fact(v, "is_under_legal_disability"),
        fixed_entitlement_amount: optional_decimal(v, "fixed_entitlement_amount")?,
        percentage_entitlement: optional_decimal(v, "percentage_entitlement")?,
        specifically_streamed_capital_gains: amount(v, "specifically_streamed_capital_gains")?,
        specifically_streamed_franked_dividends: amount(
            v,
            "specifically_streamed_franked_dividends",
        )?,
    })
}

fn proportionate_share(v: &Value) -> Result<Value> {
    let assessment = TrustIncomeAssessment {
        trust_accounting_income: decimal(v, "trust_accounting_income")?,
        section95_net_taxable_income: decimal(v, "section95_net_taxable_income")?,
        net_capital_gains: amount(v, "net_capital_gains")?,
        franked_dividends: amount(v, "franked_dividends")?,
        franking_credits: amount(v, "franking_credits")?,
        beneficiaries: array(v, "beneficiaries")?
            .iter()
            .map(beneficiary)
            .collect::<Result<_>>()?,
    };
    let shares = engine::calculate_proportionate_share(&assessment).map_err(engine_error)?;
    Ok(shares
        .iter()
        .map(|s| {
            json!({
                "beneficiary_name": s.beneficiary_name,
                "trust_income_entitlement": s.trust_income_entitlement.to_string(),
                "proportion_percentage": s.proportion_percentage.to_string(),
                "section95_net_income_share": s.section95_net_income_share.to_string(),
                "streamed_capital_gains": s.streamed_capital_gains.to_string(),
                "streamed_franked_dividends": s.streamed_franked_dividends.to_string(),
                "franking_credit_grossup": s.franking_credit_grossup.to_string(),
                "total_taxable_component": s.total_taxable_component.to_string(),
                "assessed_under_section": s.assessed_under_section,
            })
        })
        .collect())
}

fn s100a_risk(v: &Value) -> Result<Value> {
    let mut facts = Section100AFacts::unstated();
    for (slot, key) in [
        (
            &mut facts.beneficiary_is_adult_child,
            "beneficiary_is_adult_child",
        ),
        (
            &mut facts.funds_retained_by_parents_without_loan,
            "funds_retained_by_parents_without_loan",
        ),
        (&mut facts.circular_flow_of_funds, "circular_flow_of_funds"),
        (
            &mut facts.corporate_beneficiary_unpaid_present_entitlement,
            "corporate_beneficiary_unpaid_present_entitlement",
        ),
        (
            &mut facts.beneficiary_actually_received_funds,
            "beneficiary_actually_received_funds",
        ),
        (
            &mut facts.funds_used_for_beneficiary_direct_benefit,
            "funds_used_for_beneficiary_direct_benefit",
        ),
        (
            &mut facts.commercial_loan_agreement_in_place,
            "commercial_loan_agreement_in_place",
        ),
        (
            &mut facts.entitlement_applied_to_pre_18_expenses,
            "entitlement_applied_to_pre_18_expenses",
        ),
        (
            &mut facts.received_within_two_years,
            "received_within_two_years",
        ),
        (
            &mut facts.retention_scenario_conditions_met,
            "retention_scenario_conditions_met",
        ),
        (
            &mut facts.paragraph_32_exclusion_present,
            "paragraph_32_exclusion_present",
        ),
    ] {
        *slot = fact(v, key);
    }
    let a = engine::evaluate_section100a_risk(
        text(v, "beneficiary_name")?,
        &decimal(v, "distribution_amount")?,
        &facts,
    )
    .map_err(engine_error)?;
    Ok(json!({
        "beneficiary_name": a.beneficiary_name,
        "distribution_amount": a.distribution_amount.to_string(),
        "risk_zone": a.risk_zone.value(),
        "is_ordinary_family_dealing": a.is_ordinary_family_dealing,
        "risk_factors_identified": a.risk_factors_identified,
        "mitigating_factors": a.mitigating_factors,
        "tax_consequence_summary": a.tax_consequence_summary,
        "statutory_reference": a.statutory_reference,
        "unestablished_facts": a.unestablished_facts,
    }))
}

fn s99b_liability(v: &Value) -> Result<Value> {
    let receipt = ForeignTrustReceipt {
        beneficiary_name: text(v, "beneficiary_name")?.into(),
        gross_amount_received_aud: decimal(v, "gross_amount_received_aud")?,
        beneficiary_was_resident_during_year: fact(v, "beneficiary_was_resident_during_year"),
        corpus_amount_aud: amount(v, "corpus_amount_aud")?,
        corpus_attributable_to_notional_assessable_income_aud: amount(
            v,
            "corpus_attributable_to_notional_assessable_income_aud",
        )?,
        not_assessable_to_resident_aud: amount(v, "not_assessable_to_resident_aud")?,
        already_assessed_under_div6_aud: amount(v, "already_assessed_under_div6_aud")?,
    };
    let a = engine::evaluate_section99b_liability(&receipt).map_err(engine_error)?;
    Ok(json!({
        "beneficiary_name": a.beneficiary_name,
        "gross_receipt": a.gross_receipt.to_string(),
        "corpus_exemption": a.corpus_exemption.to_string(),
        "not_assessable_to_resident_exemption": a.not_assessable_to_resident_exemption.to_string(),
        "prior_assessed_exemption": a.prior_assessed_exemption.to_string(),
        "assessable_income_under_s99b": a.assessable_income_under_s99b.to_string(),
        "statutory_basis": a.statutory_basis,
        "caveats": a.caveats,
    }))
}

fn trust_resolution(v: &Value) -> Result<Value> {
    let schedule = TrustResolutionSchedule {
        financial_year: integer(v, "financial_year")?,
        resolution_date: date(v, "resolution_date")?,
        is_signed_by_trustee: fact(v, "is_signed_by_trustee"),
        streaming_powers_in_deed: fact(v, "streaming_powers_in_deed"),
        default_beneficiary_clause_exists: fact(v, "default_beneficiary_clause_exists"),
        allocated_percentages_total: decimal(v, "allocated_percentages_total")?,
        uses_specific_streaming: fact(v, "uses_specific_streaming"),
        deed_resolution_deadline: if is_absent(v, "deed_resolution_deadline") {
            None
        } else {
            Some(date(v, "deed_resolution_deadline")?)
        },
    };
    let (result, issues) = engine::validate_trust_resolution(&schedule).map_err(engine_error)?;
    // Validation has already evaluated each property, so none can fail here.
    let deadline = schedule.resolution_deadline().map_err(engine_error)?;
    Ok(json!({
        "result": result,
        "issues": issues,
        "resolution_deadline": deadline.map(|d| d.to_string()),
        "is_effective_by_year_end": schedule.is_effective_by_year_end().map_err(engine_error)?,
        "is_within_income_year": schedule.is_within_income_year().map_err(engine_error)?,
    }))
}

/// Run one JSON stdin command: the result value, or the refusal message.
pub fn evaluate(command: &str, v: &Value) -> Result<Value> {
    if !v.is_object() {
        return Err("input must be a JSON object".into());
    }
    match command {
        "proportionate-share" => proportionate_share(v),
        "s100a-risk" => s100a_risk(v),
        "s99b-liability" => s99b_liability(v),
        "trust-resolution" => trust_resolution(v),
        "validate-amount" => cli::decimal_type(text(v, "value")?).map(|d| json!(d.to_string())),
        _ => Err(format!("unknown command {command}")),
    }
}
