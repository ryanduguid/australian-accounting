//! Fixed fabricated demonstrations, not statutory calculations or SBR lodgments.
use div7a_loan_review::decimal::{Decimal, Rounding};
use serde_json::{Value, json};

fn dec(text: &str) -> Decimal {
    Decimal::parse(text).expect("fixed demonstration amount")
}
fn cents(value: Decimal) -> Decimal {
    value
        .quantize(-2, Rounding::HalfEven)
        .expect("bounded demonstration amount")
}
pub fn generate(input: &Value) -> Result<Value, String> {
    let amount = super::money::amount(
        input["revenue_or_sales"].as_str().unwrap(),
        "revenue_or_sales",
    )?;
    let name = &input["entity_name"];
    let raw = input["form_type"].as_str().unwrap();
    let kind = raw.trim().to_uppercase();
    let cost = cents(amount.mul(&dec("0.4")).unwrap());
    match kind.as_str() {
        "CTR" => {
            let expenses = cents(amount.mul(&dec("0.3")).unwrap());
            let entertainment = dec("15000.00");
            let gross = amount.sub(&cost).unwrap();
            let profit = gross.sub(&expenses).unwrap().sub(&entertainment).unwrap();
            let taxable = profit.add(&entertainment).unwrap();
            let liability = cents(taxable.mul(&dec("0.25")).unwrap());
            Ok(
                json!({"synthetic":true,"not_a_lodgment":true,"form_type":"CTR_AU_2025",
                "entity":{"legal_name":name,"tfn":"XXX-XXX-123","abn":"11 222 333 444","financial_year":2025,"base_rate_entity":true},
                "income_statement":{"total_business_income":amount.to_string(),"cost_of_sales":cost.to_string(),"gross_profit":gross.to_string(),"total_expenses":expenses.add(&entertainment).unwrap().to_string(),"operating_net_profit":profit.to_string()},
                "reconciliation":{"accounting_profit":profit.to_string(),"add_back_non_deductible":entertainment.to_string(),"taxable_income":taxable.to_string(),"applicable_tax_rate":"0.25","gross_tax_liability":liability.to_string()}}),
            )
        }
        "BAS" => {
            let capital = dec("11000.00");
            let collected = cents(amount.div(&dec("11.0")).unwrap());
            let purchases = cents(capital.add(&cost).unwrap().div(&dec("11.0")).unwrap());
            let net = collected.sub(&purchases).unwrap();
            Ok(
                json!({"synthetic":true,"not_a_lodgment":true,"form_type":"BAS_AU_ACTIVITY_STATEMENT",
                "entity":{"name":name,"abn":"11 222 333 444","quarter_ended":"2025-03-31"},
                "gst_labels":{"G1_total_sales":amount.to_string(),"G10_capital_purchases":capital.to_string(),"G11_non_capital_purchases":cost.to_string(),"1A_gst_on_sales":collected.to_string(),"1B_gst_on_purchases":purchases.to_string(),"net_gst":net.to_string()},
                "payg_withholding_labels":{"W1_total_salary_wages":"150000.00","W2_amounts_withheld":"37500.00"},
                "summary":{"total_payable_to_ato":net.add(&dec("37500.00")).unwrap().to_string()}}),
            )
        }
        _ => Err(format!(
            "Unknown form_type {}. Supported: CTR, BAS.",
            div7a_loan_review::raw::Raw::text(raw).py_repr()
        )),
    }
}
