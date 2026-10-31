use serde_json::{Value, json};

const MONEY: &[&str] = &[
    "amount",
    "taxable_income",
    "other_gains",
    "discount_gains",
    "current_losses",
    "prior_losses",
    "type_one_value",
    "type_two_value",
    "cost",
    "ordinary_time_earnings",
    "qualifying_contributions",
    "earnings",
    "total_super_balance",
    "concessional_contributions",
    "unused_concessional_cap",
    "non_concessional_contributions",
    "account_balance",
    "repayment_income",
    "division_293_income",
    "low_tax_contributions",
];

/// The engine's worksheets this server's input schema can reach. The engine can
/// hold kinds the published contract does not advertise yet.
pub fn advertised_worksheets() -> Result<Value, String> {
    let mut catalogue =
        australian_tax_calculators::api::evaluate("worksheet-catalogue", &json!({}))?;
    let spec =
        crate::contracts::tool("calculate_tax_worksheet").ok_or("calculate_tax_worksheet")?;
    let advertised = &spec["inputSchema"]["properties"]["facts"]["discriminator"]["mapping"];
    catalogue
        .as_object_mut()
        .ok_or("worksheet catalogue")?
        .retain(|kind, _| advertised.get(kind).is_some());
    Ok(catalogue)
}

pub fn calculate(input: &Value) -> Result<Value, String> {
    let mut facts = input["facts"].clone();
    let object = facts.as_object_mut().unwrap();
    let kind = object
        .remove("kind")
        .unwrap()
        .as_str()
        .unwrap()
        .replace('_', "-");
    for (field, value) in object {
        if MONEY.contains(&field.as_str()) {
            *value = json!(super::money::amount(value.as_str().unwrap(), field)?.to_sci_string());
        } else if ["effective_life", "taxable_use"].contains(&field.as_str()) {
            let parsed = div7a_loan_review::decimal::Decimal::parse(value.as_str().unwrap())
                .map_err(|_| "[<class 'decimal.ConversionSyntax'>]".to_string())?;
            *value = json!(parsed.to_sci_string());
        }
    }
    australian_tax_calculators::api::evaluate(&kind, &facts)
}
