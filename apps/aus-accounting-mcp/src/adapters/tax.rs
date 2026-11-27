// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

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

/// The linked engine's scope words for periods and inputs the input schema refuses
/// (b4a70733, d8f34dc2, 149f98ad, 0be34fb0, 17188fa6, 87a3f76d, the contribution caps
/// carry-forward by year and depreciation's later-year opening value), each with the
/// published 0.2.0 words the frozen contracts hold. Remove an entry when the schema opens them.
const NARROWED_SCOPE: [(&str, &str, &str); 7] = [
    (
        "medicare_levy",
        "Australian resident individual for the full income year, or for part of it. The operator establishes the days in the year, if any, for which they qualified for a full or a half Medicare levy exemption: a full exemption as a prescribed person under ITAA 1936 s 251U(1) in the medical category (a blind pensioner, or entitled to full free medical treatment under Defence Force or Veterans' Affairs arrangements) or as one not entitled to Medicare benefits, with no dependant outside the exemption categories; a half exemption in the medical category with such a dependant (s 251U(3)). For a part-year resident the operator also establishes the days they were not a resident on which every dependant was also a prescribed person (s 251U(1)(d) and (2)): each is a whole prescribed day, as s 251U(3) gives no half under para (d), and a day is counted once. A person not resident at any time in the year pays no levy (s 251S(1)(a)) and is outside this worksheet. The operator also establishes whether they are entitled to the seniors and pensioners tax offset (SAPTO) for the year at more than nil; whether they had a spouse on the last day of the year as s 3(3) reads it (a spouse living separately and apart does not count, but one living apart only because of illness, as in the ATO's Ashton example, does; so does one who died in the year), and if so the spouse's whole-dollar taxable income and SAPTO entitlement; without a spouse, whether they are entitled to the invalid and invalid carer tax offset for their child (ITAA 1997 Subdivision 61-A) or the sole parent notional tax offset (Subdivision 961-B); and the number of dependent children and students (s 961-5(1)(c)) for whom they or their spouse are entitled to the Subdivision 961-A notional tax offset, counting, without a spouse, only those for whom family tax benefit was payable to them. A spouse must also be an Australian resident for the full year, not entitled to any exemption and not a person to whom s 9A applies. Omitting those inputs states a person without SAPTO, spouse or those offsets. The operator must also establish whether s 9A applies: from 2024-25 it takes eligible lump sums in arrears out of taxable income when they are at least 10% of normal taxable income and its accrual-year tests are met. A person to whom s 9A applies is refused. Supply established whole-dollar taxable income. Applies the 2% rate and the low-income reduction (s 7): no levy at or below the threshold amount, and up to the phase-in limit no more than 10% of the excess over it, at SAPTO thresholds for a person entitled to SAPTO. With a spouse or one of those offsets the family reduction (s 8) follows: no levy when family taxable income is at or below the family income threshold, which rises for each dependent child; above it the levy is reduced by 2% of the threshold less 8% of the excess, in the person's share of family taxable income when the spouse's own income is above the spouse's threshold amount, and any part of the spouse's reduction above the spouse's levy also reduces this levy (s 8(4)). The levy after ss 7 and 8 is then reduced by the share of the year's days for which the person was a prescribed person (s 9(1)): each full exemption day, and half of each half exemption day; a full exemption for the whole year leaves no levy (s 251T). Excludes foreign residents, the Medicare levy surcharge and trustees.",
        "Australian resident individual for the full income year, single with no spouse or dependants at any time in the year, not entitled to the seniors and pensioners tax offset, and not entitled to any full or half Medicare levy exemption. The operator must also establish whether s 9A applies: from 2024-25 it takes eligible lump sums in arrears out of taxable income when they are at least 10% of normal taxable income and its accrual-year tests are met. A person to whom s 9A applies is refused. Supply established whole-dollar taxable income. Applies the 2% rate and the low-income reduction: no levy at or below the threshold amount, and up to the phase-in limit no more than 10% of the excess over it. Excludes the family income reduction, SAPTO thresholds, exemptions, the Medicare levy surcharge, trustees and part-year residence.",
    ),
    (
        "study_loan_withholding",
        "One regular weekly, fortnightly, monthly or quarterly payment of salary or wages made from 24 September 2025, in the income year given, to a payee who gave a TFN declaration or withholding declaration saying they have a study and training support loan debt, on scale 1, 2, 3, 5 or 6 as the operator established. The year selects the Schedule 1 coefficients and the Schedule 8 component coefficients for payments made in it; a 2025-26 payment needs its pay date, because the Schedule 8 coefficients of F2025L01016 apply from 24 September 2025. Scale 1 uses the Schedule 8 component for a payee who did not claim the tax-free threshold; scales 2, 3, 5 and 6 use the one for a payee who claimed it or is a foreign resident. Earnings include allowances subject to withholding. The result is the Schedule 1 amount plus the Schedule 8 component, each rounded to the dollar; the ATO's combined coefficients can differ by a dollar and it accepts either method. Excludes 2025-26 payments made before 24 September 2025, scale 4, a Medicare levy variation for a spouse or dependants, tax offsets, extra amounts for 53 or 27 pays, lump sum termination payments, back payments, bonuses and other schedules.",
        "One regular weekly, fortnightly or monthly payment of salary or wages made from 1 July 2026 to a payee who gave a TFN declaration or withholding declaration saying they have a study and training support loan debt, on scale 1, 2, 3, 5 or 6 as the operator established. Scale 1 uses the Schedule 8 component for a payee who did not claim the tax-free threshold; scales 2, 3, 5 and 6 use the one for a payee who claimed it or is a foreign resident. Earnings include allowances subject to withholding. The result is the Schedule 1 amount plus the Schedule 8 component, each rounded to the dollar; the ATO's combined coefficients can differ by a dollar and it accepts either method. Excludes scale 4, a Medicare levy variation for a spouse or dependants, tax offsets, extra amounts for 53 or 27 pays, quarterly pays, lump sum termination payments, back payments, bonuses and other schedules.",
    ),
    (
        "payg_withholding",
        "One regular weekly, fortnightly, monthly or quarterly payment of salary or wages made from 1 July 2024, in the income year given, on the scale the operator established from the payee's declarations: 1, 2, 3, 5 or 6 for a payee who gave a TFN, or 4 for one who did not, as a resident or foreign resident. The year selects the Schedule 1 coefficients and rules for payments made in it. Earnings include allowances subject to withholding. When given, the yearly tax offsets claimed on a Withholding declaration reduce scales 2, 5 and 6, the Medicare levy adjustment claimed on a Medicare levy variation declaration for a spouse or dependent children reduces scales 2 and 6, and the additional amount a payee asks for in a year of 53 weekly or 27 fortnightly pays is added. Reductions that exceed the amount withheld before them are refused. Excludes a Medicare levy variation that increases withholding, study and training support loans, bi-monthly, daily and casual pays, back payments, bonuses, termination and other schedules.",
        "One regular weekly, fortnightly or monthly payment of salary or wages made from 1 July 2026 to a payee who gave a TFN, on scale 1, 2, 3, 5 or 6 as the operator established from the payee's declarations. Earnings include allowances subject to withholding. Excludes scale 4, tax offsets, Medicare levy adjustments, study and training support loans, extra amounts for 53 or 27 pays, quarterly and bi-monthly pays, back payments, bonuses, termination and other schedules.",
    ),
    (
        "contribution_caps",
        ". Unused concessional cap is either the established total of unexpired amounts from the previous 5 years or, given each earlier year's established concessional contributions from at least the 5th year before (and the balance at the 30 June before any year whose contributions exceed its cap), worked out year by year from 2018-19 under ITAA 1997 s 291-20(3) to (7): each year's unused cap, the amounts applied earliest year first, the amount carried into the year, and, after the year's contributions and any proposed one, what is left of each year's amount and the year at whose end it lapses. Cap arithmetic only: it does not say what to contribute. The carry-forward by year excludes defined benefit interests and notional taxed contributions.",
        "; unused concessional cap is the established total of unexpired amounts from the previous 5 years.",
    ),
    (
        "depreciation",
        "Ordinary tangible Division 40 asset first held on or after 10 May 2006, in the income year its start time occurs or, given its established opening adjustable value for the year, a later year. Established cost, effective life and taxable-use proportion. Days run from first use or installation ready for use. No second-element costs after the start year, disposals, changed life or other s 40-75(2) adjustment, pooling,",
        "First year only, ordinary tangible Division 40 asset first held on or after 10 May 2006. Established cost, effective life and taxable-use proportion. Days run from first use or installation ready for use. No second-element costs, disposals, changed life, pooling,",
    ),
    (
        "fbt",
        "for the year ended 31 March 2026 or 2027, both at the 47% FBT rate. Uses the formulas in FBTAA s 5B(1B) and (1C) for gross-up. Excludes",
        "for the year ended 31 March 2026 at the 47% FBT rate. Uses the formulas in FBTAA s 5B(1B) and (1C) for gross-up. Excludes",
    ),
    (
        "fbt",
        "before any optional rounding down to 5 cents. For 2027 the items follow the FBT return 2026 instructions until the ATO publishes the 2027 return.",
        "before any optional rounding down to 5 cents.",
    ),
];

/// A worksheet's scope as this server serves it: no wider than the periods its input
/// schema admits. The engine writes a scope string into every worksheet result and
/// catalogue entry.
fn served_scope(kind: &str, record: &mut Value) -> Result<(), String> {
    let text = record["scope"].as_str().ok_or("worksheet scope")?;
    let narrowed = NARROWED_SCOPE
        .iter()
        .filter(|(narrowed, ..)| *narrowed == kind)
        .fold(text.to_owned(), |text, (_, wide, admitted)| {
            text.replace(wide, admitted)
        });
    record["scope"] = json!(narrowed);
    Ok(())
}

/// The engine's worksheets, periods and optional inputs this server's input schema can
/// reach. The linked engine can hold kinds, periods outside a year the schema fixes or
/// bounds, and optional inputs that the published contract does not advertise yet.
pub fn advertised_worksheets() -> Result<Value, String> {
    let mut catalogue =
        australian_tax_calculators::api::evaluate("worksheet-catalogue", &json!({}))?;
    let spec =
        crate::contracts::tool("calculate_tax_worksheet").ok_or("calculate_tax_worksheet")?;
    let schema = &spec["inputSchema"];
    let advertised = &schema["properties"]["facts"]["discriminator"]["mapping"];
    let kinds = catalogue.as_object_mut().ok_or("worksheet catalogue")?;
    kinds.retain(|kind, _| advertised.get(kind).is_some());
    for (kind, entry) in kinds {
        let reference = advertised[kind.as_str()].as_str().ok_or("facts mapping")?;
        let variant = reference
            .strip_prefix('#')
            .and_then(|pointer| schema.pointer(pointer))
            .ok_or("facts variant")?;
        served_scope(kind, entry)?;
        let fields = entry.as_object_mut().ok_or("worksheet entry")?;
        if let Some(serde_json::Value::Object(optional)) = fields.get_mut("optional_inputs") {
            optional.retain(|name, _| variant["properties"].get(name).is_some());
            if optional.is_empty() {
                fields.remove("optional_inputs");
            }
        }
        // The engine lists every entry's supported periods.
        let periods = entry["supported_periods"]
            .as_array_mut()
            .ok_or("worksheet periods")?;
        periods.retain(|period| {
            period["arguments"].as_object().is_none_or(|arguments| {
                arguments
                    .iter()
                    .all(|(name, value)| admits(&variant["properties"][name], value))
            })
        });
    }
    Ok(catalogue)
}

/// Whether a published facts property admits a period argument, by the keywords those
/// properties use: const, enum, minimum and maximum.
fn admits(property: &Value, value: &Value) -> bool {
    let number = value.as_f64();
    let bound = |key: &str, holds: fn(f64, f64) -> bool| {
        property[key]
            .as_f64()
            .is_none_or(|limit| number.is_some_and(|number| holds(number, limit)))
    };
    property.get("const").is_none_or(|fixed| fixed == value)
        && property["enum"]
            .as_array()
            .is_none_or(|allowed| allowed.contains(value))
        && bound("minimum", |number, limit| number >= limit)
        && bound("maximum", |number, limit| number <= limit)
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
    let mut result = australian_tax_calculators::api::evaluate(&kind, &facts)?;
    served_scope(&kind.replace('-', "_"), &mut result)?;
    if input["file_note"] == true {
        result["file_note"] = json!(file_note(&input["facts"], &result)?);
    }
    Ok(result)
}

/// A Python f-string's rendering of one supplied fact or result value.
fn shown(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Null => "None".into(),
        other => other.to_string(),
    }
}

fn table(heading: &str, values: &Value) -> Vec<String> {
    let values = values.as_object().into_iter().flatten();
    let rows: Vec<_> = values
        .map(|(name, value)| format!("| `{name}` | {} |", shown(value)))
        .collect();
    if rows.is_empty() {
        return rows;
    }
    [
        vec![format!("| {heading} | Value |"), "| --- | --- |".into()],
        rows,
        vec![String::new()],
    ]
    .concat()
}

/// Restate one worksheet result as a Markdown file note, as the Python server's
/// render_file_note does. Every figure, provision, date and caveat is the engine's or
/// the operator's; the note adds none.
fn file_note(facts: &Value, result: &Value) -> Result<String, String> {
    let catalogue = australian_tax_calculators::api::evaluate("worksheet-catalogue", &json!({}))?;
    let calculation = result["calculation"].as_str().unwrap_or_default();
    let passage = shown(&catalogue[calculation]["source_passage"]);
    let mut lines = note_heading(result, calculation);
    lines.extend(note_facts(facts));
    lines.extend(note_calculation(result, &passage));
    lines.extend(note_issues(result));
    Ok(lines.join("\n"))
}

fn note_text(result: &Value, key: &str) -> String {
    shown(&result[key])
}

fn note_items(result: &Value, key: &str) -> Vec<String> {
    result[key]
        .as_array()
        .into_iter()
        .flatten()
        .map(shown)
        .collect()
}

fn note_heading(result: &Value, calculation: &str) -> Vec<String> {
    vec![
        format!(
            "# File note: {calculation} worksheet, {}",
            note_text(result, "period")
        ),
        String::new(),
        format!(
            "Drafted from the {} {} result, which it restates without adding any figure.",
            note_text(result, "engine"),
            note_text(result, "engine_version")
        ),
        String::new(),
    ]
}

fn note_facts(facts: &Value) -> Vec<String> {
    let mut lines = vec![
        "## Facts relied on".into(),
        String::new(),
        "Established by the operator and not verified by this server.".into(),
        String::new(),
    ];
    lines.extend(
        facts
            .as_object()
            .into_iter()
            .flatten()
            .map(|(name, value)| format!("- `{name}`: {}", shown(value))),
    );
    lines.extend([String::new(), "## Calculation".into(), String::new()]);
    lines
}

fn note_calculation(result: &Value, passage: &str) -> Vec<String> {
    let mut lines = table("Amount (AUD)", &result["amounts"]);
    lines.extend(table("Rate", &result["rates"]));
    lines.extend([
        "## Assumptions".into(),
        String::new(),
        "The operator confirmed this scope:".into(),
        String::new(),
        note_text(result, "scope"),
        String::new(),
        "## Provisions and sources".into(),
        String::new(),
        format!("The engine's source check covered: {passage}."),
        String::new(),
    ]);
    let checked = note_text(result, "source_checked");
    lines.extend(
        note_items(result, "sources")
            .into_iter()
            .map(|url| format!("- {url} (checked {checked})")),
    );
    lines.push(String::new());
    lines
}

fn note_issues(result: &Value) -> Vec<String> {
    let mut lines = vec!["## Open issues".into(), String::new()];
    lines.extend(
        note_items(result, "warnings")
            .into_iter()
            .map(|warning| format!("- {warning}")),
    );
    lines.push(String::new());
    lines
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    fn year(date: &Value) -> u32 {
        date.as_str().unwrap()[..4].parse().unwrap()
    }

    /// The years from `first` on that a text names. Earlier years name legislation or
    /// history, such as ITAA 1997 or assets first held from 10 May 2006, not periods.
    fn named_years(text: &str, first: u32) -> Vec<u32> {
        text.split(|c: char| !c.is_ascii_alphanumeric())
            .filter(|word| word.len() == 4)
            .filter_map(|word| word.parse().ok())
            .filter(|named| *named >= first)
            .collect()
    }

    /// The served Medicare levy scope excludes part-year residence exactly when the
    /// published facts cannot give a part-year resident's `non_resident_days`, and the
    /// engine's own scope, which takes them, no longer does: each narrowing matches.
    #[test]
    fn the_medicare_levy_scope_excludes_part_year_residence_only_without_its_days() {
        let engine =
            australian_tax_calculators::api::evaluate("worksheet-catalogue", &json!({})).unwrap();
        let served = super::advertised_worksheets().unwrap();
        for catalogue in [&engine, &served] {
            let entry = &catalogue["medicare_levy"];
            let takes_days = entry["optional_inputs"].get("non_resident_days").is_some();
            let scope = entry["scope"].as_str().unwrap();
            assert_eq!(
                scope.contains("part-year residence."),
                !takes_days,
                "{scope}"
            );
        }
        assert!(engine["medicare_levy"]["optional_inputs"]["non_resident_days"].is_string());
    }

    /// Every year a served worksheet scope names falls in one of the worksheet's
    /// advertised periods, in the scope resource and in each period's result.
    #[test]
    fn every_year_a_scope_names_is_an_advertised_period() {
        let catalogue = super::advertised_worksheets().unwrap();
        let catalogue = catalogue.as_object().unwrap();
        let periods = |entry: &Value| entry["supported_periods"].as_array().unwrap().clone();
        let first = catalogue
            .values()
            .flat_map(periods)
            .map(|period| year(&period["start"]))
            .min()
            .unwrap();
        for (kind, entry) in catalogue {
            let periods = periods(entry);
            let mut scopes = vec![entry["scope"].clone()];
            for period in &periods {
                let mut facts = entry["example"]["facts"].clone();
                let arguments = period["arguments"].as_object().unwrap();
                facts.as_object_mut().unwrap().extend(arguments.clone());
                let result = super::calculate(&json!({ "facts": facts })).unwrap();
                scopes.push(result["scope"].clone());
            }
            for scope in &scopes {
                let scope = scope.as_str().unwrap();
                for named in named_years(scope, first) {
                    assert!(
                        periods.iter().any(|period| {
                            (year(&period["start"])..=year(&period["end"])).contains(&named)
                        }),
                        "{kind} names {named}, outside its advertised periods: {scope}"
                    );
                }
            }
        }
    }

    /// The file note's tables print each value as the Python server's f-string does:
    /// a JSON null is Python's `None`, a string is bare and anything else is its JSON.
    #[test]
    fn a_null_fact_reads_as_python_none_in_the_file_note_table() {
        assert_eq!(
            super::table(
                "Fact",
                &json!({"carry_forward": null, "label": "Lot 4", "units": 2})
            ),
            [
                "| Fact | Value |",
                "| --- | --- |",
                "| `carry_forward` | None |",
                "| `label` | Lot 4 |",
                "| `units` | 2 |",
                "",
            ]
        );
    }
}
