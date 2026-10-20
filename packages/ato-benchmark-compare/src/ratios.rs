//! The ATO benchmark ratio calculations (QC 37143, last updated 16 March
//! 2026): every ratio is a percentage of turnover excluding GST; turnover
//! falls back to total business income; associates are deducted once.

use crate::Failure;
use crate::decimal::{Decimal, Rounding};
use crate::mapping::{BUCKETS, EXPENSE_BUCKETS, is_bucket};

pub const TURNOVER_FROM_SALES: &str = "sales of goods and services";
pub const TURNOVER_FROM_TOTAL_INCOME: &str = "total business income";

/// A human-readable message and the fields needed to state it.
#[derive(Clone, Debug)]
pub struct EvidenceMessage {
    pub code: String,
    pub text: String,
    pub required_fields: Vec<String>,
}

impl EvidenceMessage {
    pub fn new(code: &str, text: String, fields: &[&str]) -> EvidenceMessage {
        EvidenceMessage {
            code: code.into(),
            text,
            required_fields: fields.iter().map(|f| f.to_string()).collect(),
        }
    }

    /// `required_fields <= known`.
    pub fn stated_by(&self, known: &[String]) -> bool {
        self.required_fields.iter().all(|f| known.contains(f))
    }
}

#[derive(Clone, Debug)]
pub struct Figures {
    /// Every bucket's amount, in `BUCKETS` order.
    pub totals: Vec<(String, Decimal)>,
    pub turnover: Decimal,
    pub turnover_basis: &'static str,
    pub trading_sales: Decimal,
    pub other_income: Decimal,
    pub total_business_income: Decimal,
    pub total_expenses_reported: Decimal,
    pub total_expenses_for_ratio: Decimal,
    pub cost_of_sales_for_ratio: Decimal,
    pub labour: Decimal,
    /// The five ratios, in the order Python built them.
    pub ratios: Vec<(String, Decimal)>,
    pub warnings: Vec<String>,
    pub warning_details: Vec<EvidenceMessage>,
    pub supplied_fields: Vec<String>,
}

impl Figures {
    pub fn amount(&self, bucket: &str) -> &Decimal {
        &self.totals.iter().find(|(b, _)| b == bucket).unwrap().1
    }

    pub fn ratio(&self, name: &str) -> Option<&Decimal> {
        self.ratios.iter().find(|(k, _)| k == name).map(|(_, v)| v)
    }
}

fn amount_error(value: &Decimal) -> Failure {
    Failure::Value(format!("{value} has more digits than this tool can report"))
}

/// Round a ratio to 4 places, refusing one the decimal context cannot hold.
pub fn quantise(value: &Decimal) -> Result<Decimal, Failure> {
    value
        .quantize(-4, Rounding::HalfUp)
        .map_err(|_| amount_error(value))
}

fn ratio_error(message: &str) -> Failure {
    Failure::Value(message.into())
}

/// Turn bucket totals into ATO benchmark ratios. A missing bucket is
/// zero-filled; `supplied` records which ones the caller established.
pub fn compute(
    totals: &[(String, Decimal)],
    supplied: &[String],
    w1: Option<&Decimal>,
) -> Result<Figures, Failure> {
    let zero = || Decimal::from_int(0);
    let amounts: Vec<(String, Decimal)> = BUCKETS
        .iter()
        .map(|(b, _)| {
            let value = totals
                .iter()
                .rev()
                .find(|(k, _)| k == b)
                .map_or_else(zero, |(_, v)| v.clone());
            (b.to_string(), value)
        })
        .collect();
    let get = |b: &str| amounts.iter().find(|(k, _)| k == b).unwrap().1.clone();
    let mut supplied_fields: Vec<String> = Vec::new();
    for name in supplied {
        if is_bucket(name) && !supplied_fields.contains(name) {
            supplied_fields.push(name.clone());
        }
    }
    if w1.is_some() {
        supplied_fields.push("w1".into());
    }
    let mut warnings = Vec::new();
    let mut details = Vec::new();
    let mut warn = |code: &str, text: String, fields: &[&str]| {
        warnings.push(text.clone());
        details.push(EvidenceMessage::new(code, text, fields));
    };
    let trading_sales = get("turnover");
    let other_income = get("other_income");
    let total_business_income = trading_sales.add(&other_income)?;
    let two = Decimal::from_int(2);
    let (turnover, basis) =
        if !trading_sales.gt(&zero())? || trading_sales.mul(&two)?.lt(&total_business_income)? {
            if !trading_sales.gt(&zero())? {
                warn(
                "turnover_no_positive_sales",
                "No positive sales of goods and services were mapped, so total business income \
                 is used as turnover. That is the ATO rule, but check the mapping."
                    .into(),
                &["turnover", "other_income"],
            );
            } else {
                warn(
                    "turnover_uses_total_income",
                    "Sales of goods and services are less than half of total business income, so \
                 total business income is used as turnover."
                        .into(),
                    &["turnover", "other_income"],
                );
            }
            (total_business_income.clone(), TURNOVER_FROM_TOTAL_INCOME)
        } else {
            (trading_sales.clone(), TURNOVER_FROM_SALES)
        };
    if !turnover.gt(&zero())? {
        return Err(ratio_error(
            "turnover is zero or negative, so no ratio can be calculated. Check that income \
             accounts are mapped to turnover or other_income and are positive.",
        ));
    }
    let mut total_expenses_reported = zero();
    for bucket in EXPENSE_BUCKETS {
        total_expenses_reported = total_expenses_reported.add(&get(bucket))?;
    }
    let associated = get("associated_persons");
    let total_expenses_for_ratio = total_expenses_reported.sub(&associated)?;
    let cost_of_sales_for_ratio = get("cost_of_sales");
    let salary_wages_mapped = get("salary_wages").add(&get("cost_of_sales_labour"))?;
    let mut salary_and_wages = salary_wages_mapped.add(&associated)?;
    if let Some(w1) = w1 {
        if w1.lt(&zero())? {
            return Err(ratio_error(
                "the activity statement W1 amount cannot be negative",
            ));
        }
        if w1.gt(&salary_and_wages)? {
            warn(
                "w1_used_for_labour",
                format!(
                    "Activity statement W1 ({w1}) is greater than the salary and wages label \
                     ({salary_and_wages}, the mapped salary and wages plus payments to \
                     associates), so W1 is used in the labour ratio."
                ),
                &[
                    "w1",
                    "salary_wages",
                    "cost_of_sales_labour",
                    "associated_persons",
                ],
            );
            salary_and_wages = w1.clone();
        }
    }
    let labour = salary_and_wages
        .sub(&associated)?
        .add(&get("contractor_commission"))?;
    for bucket in EXPENSE_BUCKETS {
        let value = get(bucket);
        if value.lt(&zero())? {
            warn(
                "negative_expense_bucket",
                format!(
                    "The {bucket} total is negative ({value}). Check the sign convention of the \
                     export, or use --flip-expense-signs."
                ),
                &[bucket],
            );
        }
    }
    if labour.lt(&zero())? {
        let mut fields = vec![
            "salary_wages",
            "cost_of_sales_labour",
            "contractor_commission",
        ];
        if w1.is_some() {
            fields.extend(["w1", "associated_persons"]);
        }
        warn(
            "negative_labour",
            "Labour is negative. Check the sign convention of the wage and contractor accounts, \
             or use --flip-expense-signs."
                .into(),
            &fields,
        );
    }
    if cost_of_sales_for_ratio.gt(&zero())? && get("cost_of_sales_labour").is_zero() {
        warn(
            "cost_of_sales_labour_zero",
            "No salary and wages were mapped inside cost of sales. The ATO excludes wages from \
             the cost of sales ratio, so confirm none are sitting in those accounts."
                .into(),
            &["cost_of_sales", "cost_of_sales_labour"],
        );
    }
    if w1.is_none() && salary_wages_mapped.gt(&zero())? {
        warn(
            "w1_not_supplied",
            "No activity statement W1 amount was supplied. The ATO uses W1 for the labour ratio \
             when it exceeds the salary and wages figure. Pass --w1 to apply that rule."
                .into(),
            &["salary_wages", "cost_of_sales_labour"],
        );
    }
    if associated.is_zero() {
        warn(
            "associated_persons_zero",
            "No payments to associated persons were mapped. Wages, directors fees and \
             management fees paid to associates are deducted from total expenses, so a zero here \
             raises the total expenses ratio."
                .into(),
            &["associated_persons"],
        );
    }
    let ratio = |value: &Decimal| -> Result<Decimal, Failure> { quantise(&value.div(&turnover)?) };
    let ratios = vec![
        (
            "total_expenses_to_turnover".to_string(),
            ratio(&total_expenses_for_ratio)?,
        ),
        (
            "cost_of_sales_to_turnover".to_string(),
            ratio(&cost_of_sales_for_ratio)?,
        ),
        ("labour_to_turnover".to_string(), ratio(&labour)?),
        ("rent_to_turnover".to_string(), ratio(&get("rent"))?),
        (
            "motor_vehicle_to_turnover".to_string(),
            ratio(&get("motor_vehicle"))?,
        ),
    ];
    Ok(Figures {
        totals: amounts.clone(),
        turnover,
        turnover_basis: basis,
        trading_sales,
        other_income,
        total_business_income,
        total_expenses_reported,
        total_expenses_for_ratio,
        cost_of_sales_for_ratio,
        labour,
        ratios,
        warnings,
        warning_details: details,
        supplied_fields,
    })
}
