//! The Python 0.1.8 operations, in the same evaluation and rounding order.
use crate::date::Date;
use crate::decimal::{Decimal as D, Rounding};
use serde_json::{Value, json};

pub type Result<T> = std::result::Result<T, String>;
pub fn d(text: &str) -> D {
    D::parse(text).expect("decimal literal")
}
pub fn q(value: &D, rounding: Rounding) -> Result<D> {
    value
        .quantize(-2, rounding)
        .map_err(|_| "decimal.InvalidOperation".into())
}
pub fn cents(value: &D) -> Result<D> {
    q(value, Rounding::HalfUp)
}
pub fn nonnegative(name: &str, value: &D) -> Result<()> {
    if !value.is_finite() || value < &d("0.00") {
        return Err(format!(
            "{name} must be a non-negative finite amount, got {value}"
        ));
    }
    Ok(())
}
pub fn rate(value: Option<&D>, where_: &str) -> Result<D> {
    let prefix = if where_.is_empty() {
        String::new()
    } else {
        format!("{where_}: ")
    };
    let value = value.ok_or_else(|| format!("{prefix}corporate_tax_rate is required. The entity's corporate tax rate is a fact to be supplied, not a base rate to be assumed"))?;
    if !value.is_finite() || value <= &d("0.00") || value >= &d("1.00") {
        return Err(format!(
            "{prefix}corporate_tax_rate must be a finite fraction between 0 and 1, got {value}"
        ));
    }
    Ok(value.clone())
}
pub fn threshold(fy: i64) -> Result<D> {
    match fy {
        2018 => Ok(d("25000000.00")),
        2019..=2027 => Ok(d("50000000.00")),
        _ => Err(format!(
            "No legislated aggregated turnover threshold exists for FY{fy}"
        )),
    }
}
#[derive(Clone, Debug)]
pub struct Bre {
    pub fy: i64,
    pub turnover: D,
    pub assessable: D,
    pub passive: D,
}
impl Bre {
    pub fn validate(&self) -> Result<()> {
        for (name, value) in [
            ("aggregated_turnover", &self.turnover),
            ("assessable_income", &self.assessable),
            ("passive_income", &self.passive),
        ] {
            nonnegative(name, value)?;
        }
        if self.passive > self.assessable {
            return Err("passive_income must not exceed assessable_income".into());
        }
        Ok(())
    }
    pub fn turnover_eligible(&self) -> Result<bool> {
        Ok(self.turnover < threshold(self.fy)?)
    }
    pub fn brepi(&self) -> Option<bool> {
        (self.assessable > d("0.00"))
            .then(|| self.passive.mul(&d("100.00")) <= d("80.00").mul(&self.assessable))
    }
    pub fn is_bre(&self) -> Result<Option<bool>> {
        Ok(if self.turnover_eligible()? {
            self.brepi()
        } else {
            Some(false)
        })
    }
    pub fn passive_pct(&self) -> Result<Option<D>> {
        if self.assessable <= d("0.00") {
            return Ok(None);
        }
        Ok(Some(cents(
            &self.passive.div(&self.assessable).mul(&d("100.00")),
        )?))
    }
    pub fn corporate_tax(&self) -> Result<CorporateTax> {
        self.validate()?;
        if !(2018..=2027).contains(&self.fy) {
            return Err(format!(
                "No legislated company-rate table exists for FY{}",
                self.fy
            ));
        }
        let bre = self.is_bre()?.ok_or_else(|| format!("FY{}: base rate entity status is not determinable because there is no assessable income to run the s 23AB passive-income test against; supply assessable_income for the year", self.fy))?;
        let (rate, description, basis) = if bre {
            let rate = d(match self.fy {
                2018..=2020 => "0.275",
                2021 => "0.260",
                _ => "0.250",
            });
            let desc = format!(
                "Base Rate Entity ({}%)",
                rate.mul(&d("100")).fixed(1, false)
            );
            let basis = format!(
                "s 23AA Income Tax Rates Act 1986; turnover < ${}M and BREPI <= 80%",
                threshold(self.fy)?.div(&d("1000000")).fixed(0, false)
            );
            (rate, desc, basis)
        } else {
            (
                d("0.300"),
                "Standard Corporate Tax Rate (30.0%)".into(),
                "s 23(2) Income Tax Rates Act 1986; exceeds turnover or BREPI threshold".into(),
            )
        };
        Ok(CorporateTax {
            fy: self.fy,
            bre,
            rate,
            description,
            basis,
        })
    }
}
pub struct CorporateTax {
    pub fy: i64,
    pub bre: bool,
    pub rate: D,
    pub description: String,
    pub basis: String,
}
impl CorporateTax {
    pub fn json(&self) -> Value {
        json!({"financial_year":self.fy,"is_base_rate_entity":self.bre,"applicable_rate":self.rate.to_string(),"rate_description":self.description,"statutory_basis":self.basis})
    }
}
pub fn max_franking_rate(fy: i64, prior: Option<Bre>) -> Result<D> {
    let mut prior = prior.ok_or_else(|| {
        format!("prior_year_test is required for FY{fy}; the maximum franking rate is not assumed")
    })?;
    prior.validate()?;
    let required = i128::from(fy) - 1;
    if i128::from(prior.fy) != required {
        return Err(format!(
            "prior_year_test must be for FY{}, got FY{}",
            required, prior.fy
        ));
    }
    prior.fy = fy;
    Ok(prior.corporate_tax()?.rate)
}

pub struct Statement {
    pub entity: String,
    pub acn: String,
    pub recipient: String,
    pub date: Date,
    pub franked: D,
    pub unfranked: D,
    pub credit: D,
    pub pct: D,
    pub rate: D,
}
impl Statement {
    pub fn new(
        entity: String,
        acn: String,
        recipient: String,
        date: Date,
        total: D,
        pct: D,
        tax_rate: Option<D>,
    ) -> Result<Self> {
        if !total.is_finite() || total <= d("0.00") {
            return Err(format!(
                "total_distribution must be a positive finite amount, got {total}"
            ));
        }
        if total != cents(&total)? {
            return Err(format!(
                "total_distribution must be a whole number of cents, got {total}"
            ));
        }
        if !pct.is_finite() || pct < d("0.00") || pct > d("100.00") {
            return Err(format!(
                "franking_percentage must be finite and between 0 and 100, got {pct}"
            ));
        }
        let rate = rate(tax_rate.as_ref(), "")?;
        let franked = cents(&total.mul(&pct.div(&d("100.00"))))?;
        let unfranked = cents(&total.sub(&franked).value)?;
        let multiplier = rate.div(&d("1.00").sub(&rate).value);
        let credit = q(&franked.mul(&multiplier), Rounding::Down)?;
        Ok(Self {
            entity,
            acn,
            recipient,
            date,
            franked,
            unfranked,
            credit,
            pct,
            rate,
        })
    }
    pub fn total(&self) -> D {
        self.franked.add(&self.unfranked).value
    }
    pub fn gross(&self) -> D {
        self.total().add(&self.credit).value
    }
    pub fn json(&self) -> Value {
        json!({"entity_name":self.entity,"abn_or_acn":self.acn,"recipient_name":self.recipient,"payment_date":self.date.to_string(),"franked_amount":self.franked.to_string(),"unfranked_amount":self.unfranked.to_string(),"franking_credit":self.credit.to_string(),"franking_percentage":self.pct.to_string(),"corporate_tax_rate":self.rate.to_string(),"withholding_tax_deducted":"0.00","total_distribution":self.total().to_string(),"gross_assessable_income":self.gross().to_string()})
    }
}

#[derive(Clone)]
pub struct Event {
    pub date: Date,
    pub recipient: String,
    pub amount: D,
    pub credit: D,
    pub rate: Option<D>,
}
impl Event {
    pub fn validate(&self) -> Result<()> {
        nonnegative("distribution_amount", &self.amount)?;
        nonnegative("franking_credit", &self.credit)?;
        if self.rate.is_some() {
            rate(self.rate.as_ref(), "DistributionEvent")?;
        }
        Ok(())
    }
    pub fn maximum(&self) -> Result<D> {
        let where_ = format!(
            "distribution to {} on {} (state a rate on the event, or add it through BenchmarkRuleValidator.add_distribution to take the validator's)",
            self.recipient, self.date
        );
        let rate = rate(self.rate.as_ref(), &where_)?;
        if self.amount <= d("0.00") {
            return Ok(d("0.00"));
        }
        Ok(self.amount.mul(&rate.div(&d("1.00").sub(&rate).value)))
    }
    pub fn ratio(&self) -> Result<D> {
        let max = self.maximum()?;
        if max <= d("0.00") {
            return Ok(d("0.00"));
        }
        let ratio = self.credit.div(&max);
        Ok(if ratio <= d("1.00") { ratio } else { d("1.00") })
    }
    pub fn pct(&self) -> Result<D> {
        cents(&self.ratio()?.mul(&d("100.00")))
    }
    pub fn json(&self) -> Result<Value> {
        Ok(
            json!({"event_date":self.date.to_string(),"recipient_name":self.recipient,"distribution_amount":self.amount.to_string(),"franking_credit":self.credit.to_string(),"corporate_tax_rate":self.rate.as_ref().map(ToString::to_string),"maximum_franking_credit":self.maximum()?.to_string(),"franking_ratio":self.ratio()?.to_string(),"franking_percentage":self.pct()?.to_string()}),
        )
    }
}
pub fn benchmark(tax_rate: Option<D>, mut events: Vec<Event>) -> Result<Value> {
    let tax_rate = rate(tax_rate.as_ref(), "BenchmarkRuleValidator")?;
    for event in &mut events {
        event.validate()?;
        if event.rate.is_none() {
            event.rate = Some(tax_rate.clone());
        }
    }
    let distributions = events.iter().map(Event::json).collect::<Result<Vec<_>>>()?;
    events.sort_by(|a, b| a.date.cmp(&b.date));
    let Some(start) = events.iter().position(|e| e.amount > d("0.00")) else {
        return Ok(
            json!({"benchmark_percentage":null,"compliant":null,"violations":[],"distributions":distributions}),
        );
    };
    let first = &events[start];
    let pct = first.pct()?;
    let ratio = first.ratio()?;
    let mut violations = Vec::new();
    for event in &events[start + 1..] {
        let actual = event.pct()?;
        let diff = actual.sub(&pct).value;
        let max = event.maximum()?;
        let benchmark_credit = ratio.mul(&max);
        let actual_credit = if event.credit < max {
            &event.credit
        } else {
            &max
        };
        let credit_diff = actual_credit.sub(&benchmark_credit).value;
        if credit_diff.abs() > d("0.01") {
            let over = credit_diff > d("0.00");
            violations.push(json!({"event_date":event.date.to_string(),"recipient_name":event.recipient,"benchmark_percentage":pct.to_string(),"actual_percentage":actual.to_string(),"variance_percentage":diff.to_string(),"consequence_type":if over {"OVER_FRANKING_TAX"} else {"FRANKING_DEBIT"},"penalty_or_debit_amount":cents(&credit_diff.abs())?.to_string(),"statutory_reference":if over {"s 203-50(1) ITAA 1997: Over-franking tax payable on excess franking credits."} else {"s 203-50(2) ITAA 1997: Franking account debit arises equal to shortfall."}}));
        }
    }
    Ok(
        json!({"benchmark_percentage":pct.to_string(),"compliant":violations.is_empty(),"violations":violations,"distributions":distributions}),
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryType {
    Payg,
    TaxPayment,
    DistributionReceived,
    Fdt,
    DistributionPaid,
    Refund,
    UnderFranking,
}
impl EntryType {
    pub fn parse(text: &str) -> Result<Self> {
        match text {
            "PAYG_INSTALMENT" => Ok(Self::Payg),
            "COMPANY_TAX_PAYMENT" => Ok(Self::TaxPayment),
            "FRANKED_DIST_REC" => Ok(Self::DistributionReceived),
            "FDT_LIABILITY" => Ok(Self::Fdt),
            "FRANKED_DIST_PAID" => Ok(Self::DistributionPaid),
            "TAX_REFUND" => Ok(Self::Refund),
            "UNDER_FRANKING_DEBIT" => Ok(Self::UnderFranking),
            _ => Err("entry_type must be a FrankingEntryType".into()),
        }
    }
    pub fn credit(self) -> bool {
        matches!(
            self,
            Self::Payg | Self::TaxPayment | Self::DistributionReceived | Self::Fdt
        )
    }
    pub fn details(self) -> (&'static str, &'static str, &'static str, &'static str) {
        match self {
            Self::Payg => (
                "PAYG_INSTALMENT",
                "amount",
                "PAYG instalment paid",
                "s 205-15 Item 1 ITAA 1997",
            ),
            Self::TaxPayment => (
                "COMPANY_TAX_PAYMENT",
                "amount",
                "Company tax assessment paid",
                "s 205-15 Item 2 ITAA 1997",
            ),
            Self::DistributionReceived => (
                "FRANKED_DIST_REC",
                "franking_credit",
                "Franked dividend received",
                "s 205-15 Item 3 ITAA 1997",
            ),
            Self::DistributionPaid => (
                "FRANKED_DIST_PAID",
                "franking_credit_attached",
                "Franked dividend paid",
                "s 205-30 Item 1 ITAA 1997",
            ),
            Self::Refund => (
                "TAX_REFUND",
                "refund_amount",
                "Income tax refund received",
                "s 205-30 Item 2 ITAA 1997",
            ),
            Self::UnderFranking => (
                "UNDER_FRANKING_DEBIT",
                "shortfall_amount",
                "Under-franking debit (benchmark rule breach)",
                "s 205-30 Item 3 / s 203-50(2) ITAA 1997",
            ),
            Self::Fdt => (
                "FDT_LIABILITY",
                "fdt_amount",
                "Franking deficit tax liability incurred",
                "s 205-15 ITAA 1997 (liability to franking deficit tax)",
            ),
        }
    }
}
pub struct Entry {
    pub date: Date,
    pub kind: EntryType,
    pub amount: D,
    pub description: String,
    pub reference: String,
}
impl Entry {
    pub fn new(
        date: Date,
        kind: EntryType,
        amount: D,
        description: String,
        reference: String,
    ) -> Result<Self> {
        if !amount.is_finite() || amount < d("0") {
            return Err("amount must be a non-negative finite Decimal".into());
        }
        Ok(Self {
            date,
            kind,
            amount,
            description,
            reference,
        })
    }
    pub fn json(&self) -> Value {
        json!({"entry_date":self.date.to_string(),"entry_type":self.kind.details().0,"amount":self.amount.to_string(),"description":self.description,"statutory_reference":self.reference,"is_credit":self.kind.credit(),"is_debit":!self.kind.credit()})
    }
}
pub struct Account {
    pub fy: i64,
    pub opening: Option<D>,
    pub entries: Vec<Entry>,
}
impl Account {
    pub fn validate(&self) -> Result<()> {
        if let Some(opening) = &self.opening
            && !opening.is_finite()
        {
            return Err(format!(
                "opening_balance must be a finite amount, got {opening}"
            ));
        }
        self.validate_periods()
    }
    fn validate_periods(&self) -> Result<()> {
        if !(2..=9999).contains(&self.fy) {
            return Err("year must be in 1..9999".into());
        }
        let start = format!("{:04}-07-01", self.fy - 1);
        let end = format!("{:04}-06-30", self.fy);
        for e in &self.entries {
            if e.date.0 < start || e.date.0 > end {
                return Err(format!(
                    "entry_date {} is outside FY{} ({} to {})",
                    e.date, self.fy, start, end
                ));
            }
        }
        Ok(())
    }
    pub fn record(
        &mut self,
        date: Date,
        kind: EntryType,
        amount: D,
        description: Option<String>,
        refund_classification: Option<bool>,
    ) -> Result<()> {
        if kind == EntryType::Refund && refund_classification != Some(false) {
            return Err("Refunds containing an R&D offset are unsupported; confirm includes_r_and_d_offset=False only for an ordinary refund".into());
        }
        let (_, name, desc, reference) = kind.details();
        if !amount.is_finite() || amount <= d("0.00") {
            return Err(format!(
                "{name} must be a positive finite amount, got {amount}"
            ));
        }
        let entry = Entry::new(
            date,
            kind,
            cents(&amount)?,
            description.unwrap_or_else(|| desc.into()),
            reference.into(),
        )?;
        // Validate before posting: a refused entry never changes this ledger.
        let single = Self {
            fy: self.fy,
            opening: None,
            entries: vec![entry],
        };
        single.validate_periods()?;
        self.entries.extend(single.entries);
        Ok(())
    }
    pub fn totals(&self) -> Result<(D, D)> {
        self.validate_periods()?;
        let mut credit = d("0.00");
        let mut debit = d("0.00");
        for entry in &self.entries {
            if entry.kind.credit() {
                credit = credit.add(&entry.amount).value;
            } else {
                debit = debit.add(&entry.amount).value;
            }
        }
        Ok((credit, debit))
    }
    pub fn closing(&self) -> Result<Option<D>> {
        let (credits, debits) = self.totals()?;
        self.opening
            .as_ref()
            .map(|o| cents(&o.add(&credits.sub(&debits).value).value))
            .transpose()
    }
    pub fn deficit(&self) -> Result<Value> {
        let balance = self.closing()?;
        let (credits, _) = self.totals()?;
        let Some(balance) = balance else {
            return Ok(
                json!({"closing_balance":null,"has_deficit":null,"franking_deficit_tax":null,"total_franking_credits_year":credits.to_string(),"fdt_offset_reduction_applies":null,"allowable_tax_offset":null,"statutory_basis":"s 205-45 ITAA 1997: not tested.","unknown_reason":format!("The FY{} opening franking account balance is not established, so there is no closing balance to test. State it, including a nil balance, from the prior year's reconciled account",self.fy)}),
            );
        };
        let deficit = balance < d("0.00");
        let fdt = if deficit { balance.abs() } else { d("0.00") };
        let reduction = deficit
            && self.entries.iter().any(|e| {
                matches!(
                    e.kind,
                    EntryType::DistributionPaid | EntryType::UnderFranking
                )
            })
            && fdt > credits.mul(&d("0.10"));
        let offset = if reduction {
            cents(&fdt.mul(&d("0.70")))?
        } else {
            fdt.clone()
        };
        let basis = if !deficit {
            "s 205-45 ITAA 1997: Surplus franking account balance; no FDT liability.".into()
        } else if reduction {
            format!(
                "s 205-45 ITAA 1997: FDT liability ${}. Deficit from debits covered by s 205-70(8) exceeds 10% of annual credits (${}); tax offset is reduced by 30% to ${} under s 205-70(2).",
                fdt.fixed(2, true),
                credits.fixed(2, true),
                offset.fixed(2, true)
            )
        } else {
            format!(
                "s 205-45 ITAA 1997: FDT liability ${}. 100% allowable as tax offset under s 205-70.",
                fdt.fixed(2, true)
            )
        };
        Ok(
            json!({"closing_balance":balance.to_string(),"has_deficit":deficit,"franking_deficit_tax":fdt.to_string(),"total_franking_credits_year":credits.to_string(),"fdt_offset_reduction_applies":reduction,"allowable_tax_offset":offset.to_string(),"statutory_basis":basis,"unknown_reason":null}),
        )
    }
    pub fn json(&self) -> Result<Value> {
        let (credits, debits) = self.totals()?;
        Ok(
            json!({"financial_year":self.fy,"opening_balance":self.opening.as_ref().map(ToString::to_string),"entries":self.entries.iter().map(Entry::json).collect::<Vec<_>>(),"total_credits":credits.to_string(),"total_debits":debits.to_string(),"closing_balance":self.closing()?.map(|d| d.to_string()),"deficit":self.deficit()?}),
        )
    }
}
