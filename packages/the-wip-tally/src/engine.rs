//! AASB 15 WIP arithmetic for a single unit of account, ported from Python
//! `wiptally.schedule` and `wiptally.model`. Subsequent corrections are recorded
//! in RELEASE_NOTES.md. It computes; it does not decide transfer over time,
//! claim enforceability or whether a contract is onerous.

use crate::decimal::{Calc, Decimal, Signal};
use crate::money::{as_money, as_points, d};
use crate::pytext;

pub const PROGRESS_COST_TO_COST: &str = "cost_to_cost";
pub const PROGRESS_OUTPUT: &str = "output";
pub const PROGRESS_RIGHT_TO_INVOICE: &str = "right_to_invoice";
/// `sorted(PROGRESS_METHODS)`.
pub const PROGRESS_METHODS: [&str; 3] = [
    PROGRESS_COST_TO_COST,
    PROGRESS_OUTPUT,
    PROGRESS_RIGHT_TO_INVOICE,
];
pub const RETENTION_RECEIVABLE: &str = "receivable";
pub const RETENTION_REVIEW: &str = "review";
/// `sorted(RETENTION_CLASSES)`.
pub const RETENTION_CLASSES: [&str; 3] = ["conditional", RETENTION_RECEIVABLE, RETENTION_REVIEW];

/// Flags that force exit code 2. Informational flags stay on the row.
pub const REVIEW_FLAGS: [&str; 12] = [
    "profit_fade",
    "stale_cost_to_complete",
    "onerous_contract_review_aasb_137",
    "impair_contract_assets_before_provision_aasb_137_69",
    "outcome_not_reasonably_measurable",
    "variable_consideration_constrained",
    "committed_exceeds_etc",
    "etc_has_no_commitments",
    "uncertified_claims_present",
    "retention_classification_review",
    "progress_method_not_cost_to_cost",
    "negative_margin_at_completion",
];

/// A refusal. `Decimal` is a trapped signal the Python engine raised uncaught.
#[derive(Debug, PartialEq, Eq)]
pub enum ScheduleError {
    Message(String),
    Decimal(Signal),
}

impl From<Signal> for ScheduleError {
    fn from(signal: Signal) -> Self {
        ScheduleError::Decimal(signal)
    }
}

pub type Result<T> = std::result::Result<T, ScheduleError>;

/// One unit of account. Combination under AASB 15 para 17 is an operator fact.
#[derive(Clone, Debug)]
pub struct ContractInput {
    pub contract_id: String,
    /// The source row, as Python's `int` printed it in messages.
    pub line_number: String,
    pub customer: String,
    pub description: String,
    pub original_contract_sum: Decimal,
    pub approved_variations: Decimal,
    pub unapproved_variations_estimate: Decimal,
    pub constraint_include_ratio: Decimal,
    pub costs_incurred: Decimal,
    pub inefficiency_rework_wastage: Decimal,
    pub uninstalled_materials: Decimal,
    pub estimated_cost_to_complete: Decimal,
    pub certified_billings: Decimal,
    pub uncertified_claims: Decimal,
    pub retention_withheld: Decimal,
    pub retention_classification: String,
    pub committed_outstanding: Decimal,
    /// None is not stated, which is refused rather than read as yes.
    pub outcome_reasonably_measurable: Option<bool>,
    pub recoverable_costs: Option<Decimal>,
    pub progress_method: String,
    pub output_percent: Option<Decimal>,
    pub prior_transaction_price: Option<Decimal>,
    pub prior_estimated_cost_at_completion: Option<Decimal>,
    pub prior_costs_incurred: Option<Decimal>,
    pub prior_estimated_cost_to_complete: Option<Decimal>,
    pub prior_revenue_to_date: Option<Decimal>,
    pub gst_rate: Decimal,
    pub assets_used_carrying: Option<Decimal>,
}

/// The computed WIP position for one contract. Money fields are cent-quantised.
#[derive(Clone, Debug)]
pub struct ContractPosition {
    pub contract: ContractInput,
    pub approved_price: Decimal,
    pub variable_consideration_included: Decimal,
    pub variable_consideration_excluded: Decimal,
    pub transaction_price: Decimal,
    pub progress_cost: Decimal,
    pub estimated_cost_at_completion: Decimal,
    pub progress_eac: Decimal,
    pub percent_complete: Decimal,
    pub revenue_to_date: Decimal,
    pub contract_asset: Decimal,
    pub contract_liability: Decimal,
    pub gross_profit_at_completion: Decimal,
    pub margin_at_completion: Option<Decimal>,
    pub prior_margin_at_completion: Option<Decimal>,
    pub profit_fade_points: Option<Decimal>,
    pub period_revenue: Option<Decimal>,
    pub gst_on_certified_billings: Decimal,
    pub gst_on_retention: Decimal,
    pub flags: Vec<&'static str>,
}

impl ContractPosition {
    pub fn needs_review(&self) -> bool {
        self.flags.iter().any(|f| REVIEW_FLAGS.contains(f))
    }
}

/// All contracts in the file, and the portfolio totals that must not be netted.
pub struct Schedule {
    pub as_at: String,
    pub positions: Vec<ContractPosition>,
}

impl Schedule {
    fn total(&self, field: impl Fn(&ContractPosition) -> &Decimal) -> Calc<Decimal> {
        self.positions
            .iter()
            .try_fold(d("0.00"), |sum, row| sum.add(field(row)))
    }

    pub fn total_contract_assets(&self) -> Calc<Decimal> {
        self.total(|row| &row.contract_asset)
    }

    pub fn total_contract_liabilities(&self) -> Calc<Decimal> {
        self.total(|row| &row.contract_liability)
    }

    pub fn total_revenue_to_date(&self) -> Calc<Decimal> {
        self.total(|row| &row.revenue_to_date)
    }

    pub fn review_rows(&self) -> Vec<&ContractPosition> {
        self.positions.iter().filter(|r| r.needs_review()).collect()
    }
}

fn where_(contract: &ContractInput, field: &str) -> String {
    format!(
        "row {} ({}): {field}",
        contract.line_number, contract.contract_id
    )
}

fn refuse<T>(message: String) -> Result<T> {
    Err(ScheduleError::Message(message))
}

fn require_non_negative(contract: &ContractInput, value: &Decimal, field: &str) -> Result<()> {
    if value.lt(&d("0"))? {
        return refuse(format!("{} is negative", where_(contract, field)));
    }
    Ok(())
}

fn require_ratio(
    contract: &ContractInput,
    value: &Decimal,
    field: &str,
    description: &str,
) -> Result<()> {
    if !value.is_finite() || !(d("0.00").le(value)? && value.le(&d("1"))?) {
        return refuse(format!(
            "{} is {value}; {description} must be between 0 and 1",
            where_(contract, field)
        ));
    }
    Ok(())
}

/// `measure`: the WIP position for one contract.
pub fn measure(contract: &ContractInput) -> Result<ContractPosition> {
    let c = contract;
    for (value, field) in [
        (&c.original_contract_sum, "original_contract_sum"),
        (&c.approved_variations, "approved_variations"),
        (
            &c.unapproved_variations_estimate,
            "unapproved_variations_estimate",
        ),
        (&c.costs_incurred, "costs_incurred"),
        (
            &c.inefficiency_rework_wastage,
            "inefficiency_rework_wastage",
        ),
        (&c.uninstalled_materials, "uninstalled_materials"),
        (&c.estimated_cost_to_complete, "estimated_cost_to_complete"),
        (&c.certified_billings, "certified_billings"),
        (&c.uncertified_claims, "uncertified_claims"),
        (&c.retention_withheld, "retention_withheld"),
        (&c.committed_outstanding, "committed_outstanding"),
    ] {
        require_non_negative(c, value, field)?;
    }

    // AASB 15 paras 44-45: an unstated outcome is not a yes.
    let Some(measurable) = c.outcome_reasonably_measurable else {
        return refuse(format!(
            "{} is not stated; paras 44-45 decide between percentage-of-completion revenue and \
             revenue limited to recoverable cost, so state yes or no for this contract",
            where_(c, "outcome_reasonably_measurable")
        ));
    };

    let excluded = c
        .inefficiency_rework_wastage
        .add(&c.uninstalled_materials)?;
    if excluded.gt(&c.costs_incurred)? {
        return refuse(format!(
            "{} ({excluded}) exceed costs incurred ({})",
            where_(c, "B19 exclusions"),
            c.costs_incurred
        ));
    }

    let approved_price = as_money(&c.original_contract_sum.add(&c.approved_variations)?)?;
    require_ratio(
        c,
        &c.constraint_include_ratio,
        "constraint_include_ratio",
        "constraint include ratio",
    )?;
    let variable_included = as_money(
        &c.unapproved_variations_estimate
            .mul(&c.constraint_include_ratio)?,
    )?;
    let variable_excluded = as_money(&c.unapproved_variations_estimate.sub(&variable_included)?)?;
    let transaction_price = as_money(&approved_price.add(&variable_included)?)?;

    let progress_cost = as_money(&c.costs_incurred.sub(&excluded)?)?;
    let eac = as_money(&c.costs_incurred.add(&c.estimated_cost_to_complete)?)?;
    let progress_eac = as_money(&eac.sub(&excluded)?)?;

    let mut flags: Vec<&'static str> = Vec::new();
    let method = c.progress_method.as_str();
    if !PROGRESS_METHODS.contains(&method) {
        return refuse(format!(
            "{} {} is not supported",
            where_(c, "progress_method"),
            pytext::repr(method)
        ));
    }
    if method != PROGRESS_COST_TO_COST {
        flags.push("progress_method_not_cost_to_cost");
    }
    // A supplied output percentage is checked even where para 45 leaves it unused.
    if method == PROGRESS_OUTPUT
        && let Some(output) = &c.output_percent
    {
        require_ratio(c, output, "output_percent", "output progress")?;
    }

    // Use the same cents for measurable right-to-invoice revenue and balance.
    // Keep the supplied contract intact as evidence and leave other methods'
    // rounding order unchanged.
    let certified_billings = if measurable && method == PROGRESS_RIGHT_TO_INVOICE {
        as_money(&c.certified_billings)?
    } else {
        c.certified_billings.clone()
    };
    let zero = d("0.00");
    let (percent_complete, revenue_to_date) = if !measurable {
        recoverable_cost_revenue(c, &transaction_price, &progress_cost, &mut flags)?
    } else if method == PROGRESS_COST_TO_COST {
        cost_to_cost_revenue(c, &transaction_price, &progress_cost, &progress_eac)?
    } else if method == PROGRESS_OUTPUT {
        let Some(output) = &c.output_percent else {
            return refuse(format!(
                "{} is required when progress_method is output",
                where_(c, "output_percent")
            ));
        };
        (output.clone(), as_money(&transaction_price.mul(output)?)?)
    } else {
        let percent = if progress_eac.le(&d("0"))? {
            zero.clone()
        } else {
            progress_cost.div(&progress_eac)?
        };
        (percent, certified_billings.clone())
    };

    let net = as_money(&revenue_to_date.sub(&certified_billings)?)?;
    let (contract_asset, contract_liability) = if net.gt(&d("0"))? {
        (net, zero.clone())
    } else if net.lt(&d("0"))? {
        (zero.clone(), as_money(&net.neg()?)?)
    } else {
        (zero.clone(), zero.clone())
    };

    let gross_profit = as_money(&transaction_price.sub(&eac)?)?;
    let margin = if transaction_price.equals(&d("0"))? {
        None
    } else {
        Some(gross_profit.div(&transaction_price)?)
    };
    let prior_margin = prior_margin(c)?;
    let fade_points = match (&margin, &prior_margin) {
        (Some(m), Some(p)) => Some(as_points(&p.sub(m)?.mul(&d("100"))?)?),
        _ => None,
    };
    let period_revenue = match &c.prior_revenue_to_date {
        Some(prior) => Some(as_money(&revenue_to_date.sub(prior)?)?),
        None => None,
    };

    require_ratio(c, &c.gst_rate, "gst_rate", "GST rate")?;
    let gst_on_certified = as_money(&c.certified_billings.mul(&c.gst_rate)?)?;
    let gst_on_retention = as_money(&c.retention_withheld.mul(&c.gst_rate)?)?;

    flag_commercial(
        c,
        &mut flags,
        &variable_excluded,
        &fade_points,
        &gross_profit,
    )?;

    Ok(ContractPosition {
        contract: c.clone(),
        approved_price,
        variable_consideration_included: variable_included,
        variable_consideration_excluded: variable_excluded,
        transaction_price,
        progress_cost,
        estimated_cost_at_completion: eac,
        progress_eac,
        percent_complete,
        revenue_to_date,
        contract_asset,
        contract_liability,
        gross_profit_at_completion: gross_profit,
        margin_at_completion: margin,
        prior_margin_at_completion: prior_margin,
        profit_fade_points: fade_points,
        period_revenue,
        gst_on_certified_billings: gst_on_certified,
        gst_on_retention,
        flags,
    })
}

/// AASB 15 para 45: revenue only to the extent of recoverable costs incurred.
fn recoverable_cost_revenue(
    c: &ContractInput,
    transaction_price: &Decimal,
    progress_cost: &Decimal,
    flags: &mut Vec<&'static str>,
) -> Result<(Decimal, Decimal)> {
    flags.push("outcome_not_reasonably_measurable");
    let recoverable = c.recoverable_costs.as_ref().unwrap_or(progress_cost);
    require_non_negative(c, recoverable, "recoverable_costs")?;
    if recoverable.gt(&c.costs_incurred)? {
        return refuse(format!(
            "{} {recoverable} exceeds costs incurred {}; para 45 recognises \
             revenue only to the extent of the costs incurred",
            where_(c, "recoverable_costs"),
            c.costs_incurred
        ));
    }
    if recoverable.gt(transaction_price)? {
        return refuse(format!(
            "{} {recoverable} exceeds the transaction price {transaction_price}; revenue is \
             recognised only from the transaction price",
            where_(c, "recoverable_costs")
        ));
    }
    Ok((d("0.00"), as_money(recoverable)?))
}

fn cost_to_cost_revenue(
    c: &ContractInput,
    transaction_price: &Decimal,
    progress_cost: &Decimal,
    progress_eac: &Decimal,
) -> Result<(Decimal, Decimal)> {
    if progress_eac.le(&d("0"))? {
        return refuse(format!(
            "{} is {progress_eac} after B19 exclusions; cost-to-cost progress is undefined",
            where_(c, "progress_eac")
        ));
    }
    if progress_cost.gt(progress_eac)? {
        return refuse(format!(
            "{} {progress_cost} exceeds progress EAC {progress_eac}",
            where_(c, "progress_cost")
        ));
    }
    let percent_complete = progress_cost.div(progress_eac)?;
    // B19(b): uninstalled materials at cost; the rest of the price follows POC.
    let poc_base = as_money(&transaction_price.sub(&c.uninstalled_materials)?)?;
    if poc_base.lt(&d("0"))? {
        return refuse(format!(
            "{} {} exceeds transaction price {transaction_price}",
            where_(c, "uninstalled_materials"),
            c.uninstalled_materials
        ));
    }
    let revenue = as_money(
        &c.uninstalled_materials
            .add(&poc_base.mul(&percent_complete)?)?,
    )?;
    Ok((percent_complete, revenue))
}

fn prior_margin(c: &ContractInput) -> Calc<Option<Decimal>> {
    let (Some(prior_tp), Some(prior_eac)) = (
        &c.prior_transaction_price,
        &c.prior_estimated_cost_at_completion,
    ) else {
        return Ok(None);
    };
    if prior_tp.equals(&d("0"))? {
        return Ok(None);
    }
    Ok(Some(as_money(&prior_tp.sub(prior_eac)?)?.div(prior_tp)?))
}

fn flag_commercial(
    c: &ContractInput,
    flags: &mut Vec<&'static str>,
    variable_excluded: &Decimal,
    fade_points: &Option<Decimal>,
    gross_profit: &Decimal,
) -> Calc<()> {
    let zero = d("0");
    if variable_excluded.gt(&zero)? {
        flags.push("variable_consideration_constrained");
    }
    if c.uncertified_claims.gt(&zero)? {
        flags.push("uncertified_claims_present");
    }
    let class = c.retention_classification.as_str();
    if class == RETENTION_REVIEW && c.retention_withheld.gt(&zero)? {
        flags.push("retention_classification_review");
    } else if class == RETENTION_RECEIVABLE && c.retention_withheld.gt(&zero)? {
        flags.push("retention_presented_as_receivable_confirm_para_108");
    } else if c.retention_withheld.gt(&zero)? {
        flags.push("retention_remains_in_contract_balance");
    }

    if let Some(fade) = fade_points
        && fade.compare(&d("1.00"))?.is_ge()
    {
        flags.push("profit_fade");
    }

    if let (Some(prior_costs), Some(prior_etc)) =
        (&c.prior_costs_incurred, &c.prior_estimated_cost_to_complete)
        && c.estimated_cost_to_complete.equals(prior_etc)?
        && as_money(&c.costs_incurred.sub(prior_costs)?)?
            .compare(&d("1000.00"))?
            .is_ge()
    {
        flags.push("stale_cost_to_complete");
    }

    // AASB 137 does not wait for the outcome to be reasonably measurable.
    if gross_profit.lt(&zero)? {
        flags.push("negative_margin_at_completion");
        flags.push("onerous_contract_review_aasb_137");
        if let Some(carrying) = &c.assets_used_carrying
            && !carrying.is_zero()
            && carrying.gt(&zero)?
        {
            flags.push("impair_contract_assets_before_provision_aasb_137_69");
        }
    }

    if c.committed_outstanding.gt(&c.estimated_cost_to_complete)? {
        flags.push("committed_exceeds_etc");
    }
    if c.estimated_cost_to_complete.gt(&zero)?
        && c.committed_outstanding.equals(&zero)?
        && c.costs_incurred.gt(&zero)?
    {
        flags.push("etc_has_no_commitments");
    }
    Ok(())
}
