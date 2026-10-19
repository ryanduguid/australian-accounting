//! Trust distribution engine: Division 6 proportionate allocation, s 100A risk
//! zoning, s 99B assessment and trust resolution checks. Ported from Python
//! package louisgoldberg 0.2.0 without changing a rule, rounding step or message.

use crate::date::Date;
use crate::decimal::{Decimal, Rounding, Signal};

/// A refusal: Python's ValueError with its message, or a trapped decimal signal.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Value(String),
    Decimal(Signal),
}

impl From<Signal> for Error {
    fn from(signal: Signal) -> Error {
        Error::Decimal(signal)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Value(message) => f.write_str(message),
            Error::Decimal(signal) => write!(f, "decimal.{}", signal.name()),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

fn refuse<T>(message: impl Into<String>) -> Result<T> {
    Err(Error::Value(message.into()))
}

pub fn d(text: &str) -> Decimal {
    Decimal::parse(text).expect("decimal literal")
}

/// A tristate fact as supplied: True, False, None, or a value of another type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fact {
    Known(bool),
    Unknown,
    NotBool,
}

impl Fact {
    fn is(self, value: bool) -> bool {
        self == Fact::Known(value)
    }
}

/// `validate_boolean_facts`: unknown facts pass, other types are refused.
fn validate_boolean_facts(facts: &[(&str, Fact)]) -> Result<()> {
    match facts.iter().find(|(_, fact)| *fact == Fact::NotBool) {
        Some((name, _)) => refuse(format!("{name} must be True, False or None")),
        None => Ok(()),
    }
}

/// The default context's largest exponent; larger amounts are refused before
/// they are written out in fixed point.
const MAX_SUPPORTED_ADJUSTED_EXPONENT: i128 = 999_999;

pub fn validate_printable_amount(name: &str, value: &Decimal) -> Result<()> {
    if !value.is_zero() && value.adjusted() > MAX_SUPPORTED_ADJUSTED_EXPONENT {
        return refuse(format!(
            "{name} {value} is outside the supported range; amounts must be below 1E+{}",
            MAX_SUPPORTED_ADJUSTED_EXPONENT + 1
        ));
    }
    Ok(())
}

/// Python's `max(a, b)`: `b` only when `b > a`.
fn py_max(a: Decimal, b: Decimal) -> Result<Decimal> {
    Ok(if b.gt(&a)? { b } else { a })
}

fn cents(value: &Decimal) -> Result<Decimal> {
    Ok(value.quantize(-2, Rounding::HalfUp)?)
}

/// Indices ordered by key, largest first; ties keep their order, as Python's
/// stable `sorted(..., reverse=True)` does.
fn descending(indices: Vec<usize>, keys: &[Decimal]) -> Vec<usize> {
    let mut indices = indices;
    indices.sort_by(|&a, &b| keys[b].compare(&keys[a]).expect("finite sort keys"));
    indices
}

// Division 6 -----------------------------------------------------------------

pub struct BeneficiaryEntitlement {
    pub beneficiary_name: String,
    pub is_resident: Fact,
    pub is_under_legal_disability: Fact,
    pub fixed_entitlement_amount: Option<Decimal>,
    pub percentage_entitlement: Option<Decimal>,
    pub specifically_streamed_capital_gains: Decimal,
    pub specifically_streamed_franked_dividends: Decimal,
}

pub struct TrustIncomeAssessment {
    pub trust_accounting_income: Decimal,
    pub section95_net_taxable_income: Decimal,
    pub net_capital_gains: Decimal,
    pub franked_dividends: Decimal,
    pub franking_credits: Decimal,
    pub beneficiaries: Vec<BeneficiaryEntitlement>,
}

#[derive(Clone)]
pub struct BeneficiaryTaxShare {
    pub beneficiary_name: String,
    pub trust_income_entitlement: Decimal,
    pub proportion_percentage: Decimal,
    pub section95_net_income_share: Decimal,
    pub streamed_capital_gains: Decimal,
    pub streamed_franked_dividends: Decimal,
    pub franking_credit_grossup: Decimal,
    pub total_taxable_component: Decimal,
    pub assessed_under_section: &'static str,
}

/// `calculate_proportionate_share`: Bamford proportionate allocation of s 95
/// net income, refusing the cases the model does not implement.
pub fn calculate_proportionate_share(
    assessment: &TrustIncomeAssessment,
) -> Result<Vec<BeneficiaryTaxShare>> {
    let total = &assessment.trust_accounting_income;
    let s95 = &assessment.section95_net_taxable_income;
    let zero = d("0.00");

    if assessment.beneficiaries.is_empty() {
        return refuse(
            "no presently entitled beneficiary: the trustee is assessed under \
             s 99 or s 99A ITAA 1936, which this module does not compute",
        );
    }
    if total.le(&zero)? {
        return refuse(
            "income of the trust estate is nil or negative: with no income to be \
             presently entitled to, the proportionate approach cannot allocate the \
             s 95 net income and the trustee is assessed under s 99 or s 99A",
        );
    }
    if s95.lt(&zero)? {
        return refuse(
            "s 95 net income is a loss: a loss is not allocated to beneficiaries \
             as negative assessable shares, it is carried forward by the trust \
             against its own later net income",
        );
    }
    if assessment.franking_credits.lt(&zero)? {
        return refuse("franking credits must be non-negative");
    }
    for (name, value) in [
        ("net_capital_gains", &assessment.net_capital_gains),
        ("franked_dividends", &assessment.franked_dividends),
    ] {
        if !value.is_finite() {
            return refuse(format!("{name} must be a finite amount"));
        }
    }
    if assessment.net_capital_gains.gt(&zero)? || assessment.franked_dividends.gt(&zero)? {
        return refuse(
            "the trust has net capital gains or franked distributions: Division 6E \
             with Subdivisions 115-C and 207-B applies to them whether or not they \
             are streamed, and is not implemented",
        );
    }
    for b in &assessment.beneficiaries {
        validate_boolean_facts(&[
            ("is_resident", b.is_resident),
            ("is_under_legal_disability", b.is_under_legal_disability),
        ])?;
        let name = &b.beneficiary_name;
        if !b.is_resident.is(true) {
            return refuse(format!(
                "{name} is a non-resident: the trustee is assessed \
                 under s 98(2A) or s 98(3) ITAA 1936, which this module does not compute"
            ));
        }
        if b.is_under_legal_disability == Fact::Unknown {
            return refuse(format!(
                "{name}: whether the beneficiary is under a legal \
                 disability is not established. s 97 ITAA 1936 assesses the beneficiary \
                 and s 98 assesses the trustee on the beneficiary's behalf, so the \
                 assessed taxpayer is not known; state the fact as True or False"
            ));
        }
        if b.specifically_streamed_capital_gains.gt(&zero)?
            || b.specifically_streamed_franked_dividends.gt(&zero)?
        {
            return refuse(format!(
                "{name} has specifically streamed amounts: the Division 6E \
                 carve-out with Subdivisions 115-C and 207-B is not implemented, so the \
                 proportionate allocation would be wrong"
            ));
        }
    }

    // Unrounded ratios; the 2dp implied percentage is a report field only.
    let hundred = d("100.00");
    let mut ratios = Vec::new();
    let mut implied = Vec::new();
    let mut pct_total = d("0.00");
    let mut fixed_total = d("0.00");
    for b in &assessment.beneficiaries {
        let name = &b.beneficiary_name;
        if let Some(pct) = &b.percentage_entitlement {
            if !(zero.lt(pct)? && pct.le(&hundred)?) {
                return refuse(format!(
                    "{name} has a {pct}% entitlement; \
                     a present entitlement must be above 0 and at most 100 per cent"
                ));
            }
            ratios.push(pct.div(&hundred)?);
            implied.push(pct.clone());
            pct_total = pct_total.add(pct)?;
        } else if let Some(fixed) = &b.fixed_entitlement_amount {
            if fixed.le(&zero)? {
                return refuse(format!(
                    "{name} has a fixed entitlement of {fixed}; it must be positive"
                ));
            }
            if !fixed.equals(&fixed.quantize(-2, Rounding::HalfEven)?)? {
                return refuse(format!(
                    "{name} has a fixed entitlement of {fixed}; it must be stated in whole cents"
                ));
            }
            ratios.push(fixed.div(total)?);
            implied.push(cents(&fixed.div(total)?.mul(&hundred)?)?);
            fixed_total = fixed_total.add(fixed)?;
        } else {
            return refuse(format!(
                "{name} has neither a percentage nor a fixed entitlement"
            ));
        }
    }

    // Reconciliation to the cent, not a tolerance.
    let entitled_total = fixed_total.add(&cents(&pct_total.mul(total)?.div(&d("100"))?)?)?;
    if !entitled_total.equals(total)? {
        let gap = cents(&total.sub(&entitled_total)?)?;
        let summary = if fixed_total.equals(&zero)? {
            format!("beneficiary entitlements sum to {pct_total}%, not 100%")
        } else if pct_total.equals(&zero)? {
            format!(
                "fixed entitlements sum to {fixed_total}, not the {total} \
                 income of the trust estate"
            )
        } else {
            format!(
                "beneficiary entitlements sum to {fixed_total} plus {pct_total}% of the \
                 {total} income of the trust estate"
            )
        };
        let direction = if gap.gt(&zero)? {
            format!("{gap} of the income of the trust estate is unallocated")
        } else {
            format!(
                "the entitlements over-allocate the income of the trust estate by {}",
                gap.neg()?
            )
        };
        return refuse(format!(
            "{summary}: {direction}, \
             and the proportionate approach will not allocate the s 95 net income \
             until the entitlements reconcile to the cent"
        ));
    }

    let mut residual = cents(s95)?;
    let mut credit_residual = cents(&assessment.franking_credits)?;
    let mut shares = Vec::new();
    for ((b, ratio), pct) in assessment.beneficiaries.iter().zip(&ratios).zip(&implied) {
        let entitlement = cents(&total.mul(ratio)?)?;
        let s95_share = cents(&s95.mul(ratio)?)?;
        residual = residual.sub(&s95_share)?;
        let fc_share = cents(&assessment.franking_credits.mul(ratio)?)?;
        credit_residual = credit_residual.sub(&fc_share)?;
        let section_ref = if b.is_under_legal_disability.is(true) {
            "s 98 ITAA 1936 (Trustee assessed on behalf of beneficiary under legal disability)"
        } else {
            "s 97 ITAA 1936 (Beneficiary presently entitled and not under legal disability)"
        };
        shares.push(BeneficiaryTaxShare {
            beneficiary_name: b.beneficiary_name.clone(),
            trust_income_entitlement: entitlement,
            proportion_percentage: pct.clone(),
            section95_net_income_share: s95_share.clone(),
            streamed_capital_gains: b.specifically_streamed_capital_gains.clone(),
            streamed_franked_dividends: b.specifically_streamed_franked_dividends.clone(),
            franking_credit_grossup: fc_share,
            total_taxable_component: s95_share,
            assessed_under_section: section_ref,
        });
    }

    let all: Vec<usize> = (0..shares.len()).collect();
    let s95_keys: Vec<Decimal> = shares
        .iter()
        .map(|s| s.section95_net_income_share.clone())
        .collect();
    for i in descending(all.clone(), &s95_keys) {
        if residual.is_zero() {
            break;
        }
        let share = &mut shares[i];
        let adjustment = py_max(share.section95_net_income_share.neg()?, residual.clone())?;
        share.section95_net_income_share = share.section95_net_income_share.add(&adjustment)?;
        share.total_taxable_component = share.total_taxable_component.add(&adjustment)?;
        residual = residual.sub(&adjustment)?;
    }

    for i in descending(all, &ratios) {
        if credit_residual.is_zero() {
            break;
        }
        let share = &mut shares[i];
        let adjustment = py_max(
            share.franking_credit_grossup.neg()?,
            credit_residual.clone(),
        )?;
        share.franking_credit_grossup = share.franking_credit_grossup.add(&adjustment)?;
        credit_residual = credit_residual.sub(&adjustment)?;
    }

    // The reported entitlement column foots among the percentage shares only.
    let mut entitled = d("0.00");
    for s in &shares {
        entitled = entitled.add(&s.trust_income_entitlement)?;
    }
    let mut entitlement_residual = total.sub(&entitled)?;
    let percentage_shares: Vec<usize> = assessment
        .beneficiaries
        .iter()
        .enumerate()
        .filter(|(_, b)| b.percentage_entitlement.is_some())
        .map(|(i, _)| i)
        .collect();
    for i in descending(percentage_shares, &ratios) {
        if entitlement_residual.is_zero() {
            break;
        }
        let share = &mut shares[i];
        let adjustment = py_max(
            share.trust_income_entitlement.neg()?,
            entitlement_residual.clone(),
        )?;
        share.trust_income_entitlement = share.trust_income_entitlement.add(&adjustment)?;
        entitlement_residual = entitlement_residual.sub(&adjustment)?;
    }

    Ok(shares)
}

// Section 100A ---------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RiskZone {
    Green,
    OutsideGreen,
    Red,
    FactsNotEstablished,
}

impl RiskZone {
    pub fn value(self) -> &'static str {
        match self {
            RiskZone::Green => "GREEN",
            RiskZone::OutsideGreen => "OUTSIDE_GREEN",
            RiskZone::Red => "RED",
            RiskZone::FactsNotEstablished => "FACTS_NOT_ESTABLISHED",
        }
    }
}

/// The facts PCG 2022/2 zoning turns on, in the function's parameter order.
pub struct Section100AFacts {
    pub beneficiary_is_adult_child: Fact,
    pub funds_retained_by_parents_without_loan: Fact,
    pub circular_flow_of_funds: Fact,
    pub corporate_beneficiary_unpaid_present_entitlement: Fact,
    pub beneficiary_actually_received_funds: Fact,
    pub funds_used_for_beneficiary_direct_benefit: Fact,
    pub commercial_loan_agreement_in_place: Fact,
    pub entitlement_applied_to_pre_18_expenses: Fact,
    pub received_within_two_years: Fact,
    pub retention_scenario_conditions_met: Fact,
    pub paragraph_32_exclusion_present: Fact,
}

impl Section100AFacts {
    pub fn unstated() -> Section100AFacts {
        let u = Fact::Unknown;
        Section100AFacts {
            beneficiary_is_adult_child: u,
            funds_retained_by_parents_without_loan: u,
            circular_flow_of_funds: u,
            corporate_beneficiary_unpaid_present_entitlement: u,
            beneficiary_actually_received_funds: u,
            funds_used_for_beneficiary_direct_benefit: u,
            commercial_loan_agreement_in_place: u,
            entitlement_applied_to_pre_18_expenses: u,
            received_within_two_years: u,
            retention_scenario_conditions_met: u,
            paragraph_32_exclusion_present: u,
        }
    }

    /// The facts keyed by parameter name, in order.
    pub fn named(&self) -> [(&'static str, Fact); 11] {
        [
            (
                "beneficiary_is_adult_child",
                self.beneficiary_is_adult_child,
            ),
            (
                "funds_retained_by_parents_without_loan",
                self.funds_retained_by_parents_without_loan,
            ),
            ("circular_flow_of_funds", self.circular_flow_of_funds),
            (
                "corporate_beneficiary_unpaid_present_entitlement",
                self.corporate_beneficiary_unpaid_present_entitlement,
            ),
            (
                "beneficiary_actually_received_funds",
                self.beneficiary_actually_received_funds,
            ),
            (
                "funds_used_for_beneficiary_direct_benefit",
                self.funds_used_for_beneficiary_direct_benefit,
            ),
            (
                "commercial_loan_agreement_in_place",
                self.commercial_loan_agreement_in_place,
            ),
            (
                "entitlement_applied_to_pre_18_expenses",
                self.entitlement_applied_to_pre_18_expenses,
            ),
            ("received_within_two_years", self.received_within_two_years),
            (
                "retention_scenario_conditions_met",
                self.retention_scenario_conditions_met,
            ),
            (
                "paragraph_32_exclusion_present",
                self.paragraph_32_exclusion_present,
            ),
        ]
    }
}

pub struct Section100AAssessment {
    pub beneficiary_name: String,
    pub distribution_amount: Decimal,
    pub risk_zone: RiskZone,
    /// Always undecided: no zone decides the s 100A(13) exception.
    pub is_ordinary_family_dealing: Option<bool>,
    pub risk_factors_identified: Vec<String>,
    pub mitigating_factors: Vec<String>,
    pub tax_consequence_summary: String,
    pub statutory_reference: &'static str,
    pub unestablished_facts: Vec<&'static str>,
}

/// `evaluate_section100a_risk` under PCG 2022/2.
pub fn evaluate_section100a_risk(
    beneficiary_name: &str,
    distribution_amount: &Decimal,
    facts: &Section100AFacts,
) -> Result<Section100AAssessment> {
    if !distribution_amount.is_finite() || distribution_amount.le(&d("0"))? {
        return refuse("distribution amount must be positive and finite");
    }
    let named = facts.named();
    validate_boolean_facts(&named)?;
    let unestablished: Vec<&'static str> = named
        .iter()
        .filter(|(_, fact)| *fact == Fact::Unknown)
        .map(|(name, _)| *name)
        .collect();
    let f = facts;
    let mut risk_factors: Vec<String> = Vec::new();
    let mut mitigating: Vec<String> = Vec::new();

    // Red zone scenarios (PCG 2022/2 paragraphs 34 to 48).
    if f.circular_flow_of_funds.is(true) {
        risk_factors.push(
            "Trust income returned to the trust as a franked dividend from the beneficiary company \
             (PCG 2022/2 red zone scenario 2, paragraph 36)"
                .into(),
        );
    }
    if f.beneficiary_is_adult_child.is(true) && f.entitlement_applied_to_pre_18_expenses.is(true) {
        risk_factors.push(
            "Adult child's entitlement paid to a parent or caregiver, or set against a debit account, \
             for expenses incurred before the beneficiary turned 18 \
             (PCG 2022/2 red zone scenario 1, paragraph 34(a) and (b))"
                .into(),
        );
    }

    // Green zone scenarios (PCG 2022/2 paragraphs 20 to 30).
    if f.beneficiary_actually_received_funds.is(true)
        && f.received_within_two_years.is(true)
        && f.funds_retained_by_parents_without_loan.is(false)
    {
        mitigating.push(
            "Beneficiary received the entitlement within 2 years and used it (green zone scenario 2, paragraph 22)"
                .into(),
        );
    }
    if f.funds_used_for_beneficiary_direct_benefit.is(true) {
        mitigating.push(
            "Funds paid to the beneficiary and used for the beneficiary, their spouse or dependants \
             (green zone scenario 1, paragraph 20)"
                .into(),
        );
    }
    if f.commercial_loan_agreement_in_place.is(true) && f.retention_scenario_conditions_met.is(true)
    {
        mitigating.push(
            "Entitlement retained by the trustee under a loan on paragraph 25(e) commercial terms, \
             with the rest of green zone scenario 3A or 3B met"
                .into(),
        );
    }
    let green_blocked = f.paragraph_32_exclusion_present.is(true)
        || f.funds_retained_by_parents_without_loan.is(true)
        || (f.corporate_beneficiary_unpaid_present_entitlement.is(true)
            && f.commercial_loan_agreement_in_place.is(false));

    let (zone, consequence) = if !risk_factors.is_empty() {
        (
            RiskZone::Red,
            "HIGH RISK: the ATO prioritises review of red-zone arrangements. This does not \
             determine whether the ordinary family dealing exception or s 100A applies. \
             If s 100A applies, the entitlement is disregarded \
             and the trustee is assessed at the top rate applying under s 99A of the ITAA 1936."
                .to_string(),
        )
    } else if !unestablished.is_empty() {
        (
            RiskZone::FactsNotEstablished,
            format!(
                "FACTS NOT ESTABLISHED: no zone is assigned because these facts were not \
                 stated: {}. The green zone turns on every one of \
                 them, so it is not available here, and this is not a finding that the \
                 arrangement meets no green zone criterion. State each fact as True or \
                 False from the trust's records, or treat the arrangement as unzoned and \
                 make the factual inquiry.",
                unestablished.join(", ")
            ),
        )
    } else if !mitigating.is_empty() && !green_blocked {
        (
            RiskZone::Green,
            "GREEN ZONE: the arrangement matches a green zone example, so the ATO states it \
             will not dedicate compliance resources to it, subject to the guideline's own \
             exclusions. That is not a determination that s 100A does not apply, and it does \
             not decide the s 100A(13) ordinary family dealing exception, which turns on the \
             facts of the dealing."
                .to_string(),
        )
    } else {
        (
            RiskZone::OutsideGreen,
            "OUTSIDE THE GREEN ZONE: the arrangement meets no green zone scenario, or a green zone \
             exclusion applies, and it matches no red zone scenario, so PCG 2022/2 assigns it no zone. \
             Further factual inquiry and contemporaneous \
             documentation required."
                .to_string(),
        )
    };

    // Recorded after zoning: these facts must not drive the zone.
    if f.beneficiary_actually_received_funds.is(false) {
        risk_factors.push(
            "Beneficiary did not receive the funds representing the present entitlement".into(),
        );
    }
    if f.funds_retained_by_parents_without_loan.is(true) {
        risk_factors.push(
            "Entitlement retained by parents without a loan: outside the green zone; red only where \
             it matches paragraph 34 (expenses incurred before the beneficiary turned 18)"
                .into(),
        );
    }
    if f.corporate_beneficiary_unpaid_present_entitlement.is(true)
        && f.commercial_loan_agreement_in_place.is(false)
    {
        risk_factors.push(
            "Corporate beneficiary UPE retained without a loan on commercial terms: outside green zone \
             scenario 3B (paragraph 28(f)). Not a red zone scenario. After Commissioner of Taxation v \
             Bendel [2026] HCA 18 an unpaid entitlement is not by itself a Division 7A loan; \
             Subdivision EA can still reach dealings with the funds"
                .into(),
        );
    }

    Ok(Section100AAssessment {
        beneficiary_name: beneficiary_name.to_string(),
        distribution_amount: distribution_amount.clone(),
        risk_zone: zone,
        is_ordinary_family_dealing: None,
        risk_factors_identified: risk_factors,
        mitigating_factors: mitigating,
        tax_consequence_summary: consequence,
        statutory_reference: "s 100A ITAA 1936; ATO PCG 2022/2; TR 2022/4",
        unestablished_facts: unestablished,
    })
}

// Section 99B ----------------------------------------------------------------

pub struct ForeignTrustReceipt {
    pub beneficiary_name: String,
    pub gross_amount_received_aud: Decimal,
    pub beneficiary_was_resident_during_year: Fact,
    pub corpus_amount_aud: Decimal,
    pub corpus_attributable_to_notional_assessable_income_aud: Decimal,
    pub not_assessable_to_resident_aud: Decimal,
    pub already_assessed_under_div6_aud: Decimal,
}

pub struct Section99BAssessment {
    pub beneficiary_name: String,
    pub gross_receipt: Decimal,
    pub corpus_exemption: Decimal,
    pub not_assessable_to_resident_exemption: Decimal,
    pub prior_assessed_exemption: Decimal,
    pub assessable_income_under_s99b: Decimal,
    pub statutory_basis: String,
    pub caveats: Vec<String>,
}

/// `evaluate_section99b_liability`: s 99B(1) less the s 99B(2)(a), (b) and (c) reductions.
pub fn evaluate_section99b_liability(
    receipt: &ForeignTrustReceipt,
) -> Result<Section99BAssessment> {
    let zero = d("0.00");
    for (name, value) in [
        (
            "gross_amount_received_aud",
            &receipt.gross_amount_received_aud,
        ),
        ("corpus_amount_aud", &receipt.corpus_amount_aud),
        (
            "corpus_attributable_to_notional_assessable_income_aud",
            &receipt.corpus_attributable_to_notional_assessable_income_aud,
        ),
        (
            "not_assessable_to_resident_aud",
            &receipt.not_assessable_to_resident_aud,
        ),
        (
            "already_assessed_under_div6_aud",
            &receipt.already_assessed_under_div6_aud,
        ),
    ] {
        if !value.is_finite() || value.lt(&zero)? {
            return refuse(format!(
                "{name} must be a non-negative finite amount, got {value}"
            ));
        }
    }
    validate_boolean_facts(&[(
        "beneficiary_was_resident_during_year",
        receipt.beneficiary_was_resident_during_year,
    )])?;
    match receipt.beneficiary_was_resident_during_year {
        Fact::Known(true) => {}
        Fact::Known(false) => {
            return refuse(
                "s 99B(1) ITAA 1936 applies only where the beneficiary was a resident \
                 at some time during the year of income; this receipt is outside it",
            );
        }
        _ => {
            return refuse(
                "residency during the year of income is not established; s 99B(1) ITAA \
                 1936 turns on it, so state it as True or False rather than assessing \
                 the receipt on an assumption",
            );
        }
    }

    let gross = &receipt.gross_amount_received_aud;
    let corpus = &receipt.corpus_amount_aud;
    let attributable = &receipt.corpus_attributable_to_notional_assessable_income_aud;
    if attributable.gt(corpus)? {
        return refuse(format!(
            "corpus attributable to notionally assessable income cannot exceed \
             the corpus amount ({attributable} > {corpus})"
        ));
    }
    let corpus_exempt = corpus.sub(attributable)?;
    let not_assessable = &receipt.not_assessable_to_resident_aud;
    let prior_taxed = &receipt.already_assessed_under_div6_aud;

    let exemptions = corpus_exempt.add(not_assessable)?.add(prior_taxed)?;
    if exemptions.gt(gross)? {
        return refuse(format!(
            "exemptions (${}) exceed the gross receipt (${}); \
             check the inputs rather than assuming a nil assessable amount",
            exemptions.fixed(2, true),
            gross.fixed(2, true)
        ));
    }
    let assessable = gross.sub(&exemptions)?;
    for (name, value) in [
        ("gross_amount_received_aud", gross),
        ("corpus_exemption", &corpus_exempt),
        (
            "corpus_attributable_to_notional_assessable_income_aud",
            attributable,
        ),
        ("not_assessable_to_resident_aud", not_assessable),
        ("already_assessed_under_div6_aud", prior_taxed),
        ("assessable_income_under_s99b", &assessable),
    ] {
        validate_printable_amount(name, value)?;
    }

    let mut nil_exemptions = Vec::new();
    for (name, value) in [
        ("corpus_amount_aud", corpus),
        ("not_assessable_to_resident_aud", not_assessable),
        ("already_assessed_under_div6_aud", prior_taxed),
    ] {
        if value.equals(&zero)? {
            nil_exemptions.push(name);
        }
    }
    let nil_addback = corpus.gt(&zero)? && attributable.equals(&zero)?;
    let mut parts = Vec::new();
    if !nil_exemptions.is_empty() {
        parts.push(format!(
            "nil exemption amounts, each of which gives the largest assessable amount: {}",
            nil_exemptions.join(", ")
        ));
    }
    if nil_addback {
        parts.push(
            "a nil s 99B(2)(a) add-back, which leaves the whole corpus exempt and gives \
             the smallest assessable amount: corpus_attributable_to_notional_assessable_income_aud"
                .to_string(),
        );
    }
    let mut caveats = Vec::new();
    if !parts.is_empty() {
        caveats.push(format!(
            "A nil default cannot be told apart from a figure that was never supplied. \
             This result carries {}. Establish each figure from \
             the trust's records before relying on this result.",
            parts.join("; and ")
        ));
    }
    let basis = format!(
        "s 99B(1) ITAA 1936: gross receipt ${} less corpus exemption \
         ${} (s 99B(2)(a), net of ${} attributable to \
         amounts that would have been assessable to a resident), less ${} \
         (s 99B(2)(b)) and ${} already taxed (s 99B(2)(c)). \
         Assessable: ${}. Review aid only, not advice.",
        gross.fixed(2, true),
        corpus_exempt.fixed(2, true),
        attributable.fixed(2, true),
        not_assessable.fixed(2, true),
        prior_taxed.fixed(2, true),
        assessable.fixed(2, true)
    );

    Ok(Section99BAssessment {
        beneficiary_name: receipt.beneficiary_name.clone(),
        gross_receipt: gross.clone(),
        corpus_exemption: corpus_exempt,
        not_assessable_to_resident_exemption: not_assessable.clone(),
        prior_assessed_exemption: prior_taxed.clone(),
        assessable_income_under_s99b: assessable,
        statutory_basis: basis,
        caveats,
    })
}

// Trust resolution -----------------------------------------------------------

pub struct TrustResolutionSchedule {
    pub financial_year: i64,
    pub resolution_date: Date,
    pub is_signed_by_trustee: Fact,
    pub streaming_powers_in_deed: Fact,
    pub default_beneficiary_clause_exists: Fact,
    pub allocated_percentages_total: Decimal,
    pub uses_specific_streaming: Fact,
    pub deed_resolution_deadline: Option<Date>,
}

impl TrustResolutionSchedule {
    fn year_end(&self) -> Result<Date> {
        Date::new(self.financial_year, 6, 30).map_err(Error::Value)
    }

    /// 30 June, or the deed's earlier date; None until the deed is read.
    pub fn resolution_deadline(&self) -> Result<Option<Date>> {
        let Some(deed) = &self.deed_resolution_deadline else {
            return Ok(None);
        };
        let year_end = self.year_end()?;
        Ok(Some(if deed < &year_end {
            deed.clone()
        } else {
            year_end
        }))
    }

    pub fn is_effective_by_year_end(&self) -> Result<Option<bool>> {
        if self.resolution_date > self.year_end()? {
            return Ok(Some(false));
        }
        Ok(self
            .resolution_deadline()?
            .map(|deadline| self.resolution_date <= deadline))
    }

    pub fn is_within_income_year(&self) -> Result<bool> {
        Ok(self.resolution_date
            >= Date::new(self.financial_year - 1, 7, 1).map_err(Error::Value)?)
    }
}

/// `validate_trust_resolution`: None while a fact the answer turns on is unestablished.
pub fn validate_trust_resolution(
    schedule: &TrustResolutionSchedule,
) -> Result<(Option<bool>, Vec<String>)> {
    let s = schedule;
    validate_boolean_facts(&[
        ("is_signed_by_trustee", s.is_signed_by_trustee),
        ("streaming_powers_in_deed", s.streaming_powers_in_deed),
        (
            "default_beneficiary_clause_exists",
            s.default_beneficiary_clause_exists,
        ),
        ("uses_specific_streaming", s.uses_specific_streaming),
    ])?;
    let mut issues = Vec::new();
    let fy = s.financial_year;
    let date = &s.resolution_date;

    if *date > s.year_end()? {
        issues.push(format!(
            "Resolution dated {date} is after 30 June {fy} deadline."
        ));
    } else if s.is_effective_by_year_end()? == Some(false) {
        issues.push(format!(
            "Resolution dated {date} is after the deed's deadline of {}.",
            s.resolution_deadline()?
                .expect("a deed deadline decided it")
        ));
    }
    if !s.is_within_income_year()? {
        issues.push(format!(
            "Resolution dated {date} predates the {fy} income year, which began on 1 July {}; \
             it does not distribute that year's income.",
            fy - 1
        ));
    }
    if s.is_signed_by_trustee.is(false) {
        issues.push("Trustee resolution is not executed/signed.".into());
    }
    if !s.allocated_percentages_total.equals(&d("100.00"))? {
        issues.push(format!(
            "Allocated income percentages sum to {}%, not 100%.",
            s.allocated_percentages_total
        ));
    }
    let streams = s.uses_specific_streaming;
    if streams.is(true) && s.streaming_powers_in_deed.is(false) {
        issues.push(
            "Resolution streams specific income but the deed does not record streaming powers."
                .into(),
        );
    }
    if s.default_beneficiary_clause_exists.is(false) {
        issues.push(
            "Deed has no default beneficiary clause; unresolved income may be taxed to the trustee."
                .into(),
        );
    }

    // Streaming powers matter only to a resolution that streams.
    let settled = streams.is(false) || s.streaming_powers_in_deed.is(true);
    let unknown = |fact: Fact| !settled && fact == Fact::Unknown;
    let mut unestablished = Vec::new();
    if s.is_signed_by_trustee == Fact::Unknown {
        unestablished.push("is_signed_by_trustee");
    }
    if unknown(streams) {
        unestablished.push("uses_specific_streaming");
    }
    if unknown(s.streaming_powers_in_deed) {
        unestablished.push("streaming_powers_in_deed");
    }
    if s.default_beneficiary_clause_exists == Fact::Unknown {
        unestablished.push("default_beneficiary_clause_exists");
    }
    if s.is_effective_by_year_end()?.is_none() {
        unestablished.push("deed_resolution_deadline");
    }
    if !unestablished.is_empty() {
        issues.push(format!(
            "Not established: {}. Each is a fact of the executed resolution or the deed. Read it and \
             state it as True or False; until then the resolution is neither validated \
             nor shown to be in breach.",
            unestablished.join(", ")
        ));
        return Ok((None, issues));
    }
    let valid = issues.is_empty();
    Ok((Some(valid), issues))
}
