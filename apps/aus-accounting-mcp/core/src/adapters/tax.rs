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
/// (b4a70733, d8f34dc2, 149f98ad, 0be34fb0, 17188fa6, 87a3f76d and the contribution caps
/// carry-forward by year), each with the published 0.2.0 words, or for study loan withholding
/// the words before 0be34fb0, for the Medicare levy the single person without SAPTO, offsets
/// or exemption its facts state, and for contribution caps the supplied unused total alone,
/// for what it admits. Remove an entry when the schema opens them.
const NARROWED_SCOPE: [(&str, &str, &str); 7] = [
    (
        "medicare_levy",
        "Australian resident individual for the full income year, or for part of it. The operator establishes the days in the year, if any, for which they qualified for a full or a half Medicare levy exemption: a full exemption as a prescribed person under ITAA 1936 s 251U(1) in the medical category (a blind pensioner, or entitled to full free medical treatment under Defence Force or Veterans' Affairs arrangements) or as one not entitled to Medicare benefits, with no dependant outside the exemption categories; a half exemption in the medical category with such a dependant (s 251U(3)). For a part-year resident the operator also establishes the days they were not a resident on which every dependant was also a prescribed person (s 251U(1)(d) and (2)): each is a whole prescribed day, as s 251U(3) gives no half under para (d), and a day is counted once. A person not resident at any time in the year pays no levy (s 251S(1)(a)) and is outside this worksheet. The operator also establishes whether they are entitled to the seniors and pensioners tax offset (SAPTO) for the year at more than nil; whether they had a spouse on the last day of the year as s 3(3) reads it (a spouse living separately and apart does not count, but one living apart only because of illness, as in the ATO's Ashton example, does; so does one who died in the year), and if so the spouse's whole-dollar taxable income and SAPTO entitlement; without a spouse, whether they are entitled to the invalid and invalid carer tax offset for their child (ITAA 1997 Subdivision 61-A) or the sole parent notional tax offset (Subdivision 961-B); and the number of dependent children and students (s 961-5(1)(c)) for whom they or their spouse are entitled to the Subdivision 961-A notional tax offset, counting, without a spouse, only those for whom family tax benefit was payable to them. A spouse must also be an Australian resident for the full year, not entitled to any exemption and not a person to whom s 9A applies. Omitting those inputs states a person without SAPTO, spouse or those offsets. The operator must also establish whether",
        "Australian resident individual for the full income year, not entitled to any full or half Medicare levy exemption or to the seniors and pensioners tax offset, with no spouse on the last day of the year, and not entitled to the invalid and invalid carer tax offset for a child, the sole parent notional tax offset or a notional tax offset for dependent children or students. The operator establishes whether",
    ),
    (
        "medicare_levy",
        ", at SAPTO thresholds for a person entitled to SAPTO. With a spouse or one of those offsets the family reduction (s 8) follows: no levy when family taxable income is at or below the family income threshold, which rises for each dependent child; above it the levy is reduced by 2% of the threshold less 8% of the excess, in the person's share of family taxable income when the spouse's own income is above the spouse's threshold amount, and any part of the spouse's reduction above the spouse's levy also reduces this levy (s 8(4)). The levy after ss 7 and 8 is then reduced by the share of the year's days for which the person was a prescribed person (s 9(1)): each full exemption day, and half of each half exemption day; a full exemption for the whole year leaves no levy (s 251T). Excludes foreign residents, the Medicare levy surcharge and trustees.",
        ". Excludes exemptions, the Medicare levy surcharge, trustees and part-year residence.",
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
        "Unused concessional cap is either the established total of unexpired amounts from the previous 5 years or, given each earlier year's established concessional contributions from at least the 5th year before (and the balance at the 30 June before any year whose contributions exceed its cap), worked out year by year from 2018-19 under ITAA 1997 s 291-20(3) to (7): each year's unused cap, the amounts applied earliest year first, the amount carried into the year, and, after the year's contributions and any proposed one, what is left of each year's amount and the year at whose end it lapses. Cap arithmetic only: it does not say what to contribute. The carry-forward by year excludes defined benefit interests and notional taxed contributions.",
        "Unused concessional cap is the established total of unexpired amounts from the previous 5 years. Cap arithmetic only: it does not say what to contribute.",
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
/// schema admits.
fn served_scope(kind: &str, record: &mut Value) {
    if let Some(text) = record["scope"].as_str() {
        let narrowed = NARROWED_SCOPE
            .iter()
            .filter(|(narrowed, ..)| *narrowed == kind)
            .fold(text.to_owned(), |text, (_, wide, admitted)| {
                text.replace(wide, admitted)
            });
        record["scope"] = json!(narrowed);
    }
}

// WORKSHEETS: each kind of the linked engine's worksheet catalogue, with its entry and
// the entry's freshness fields as JSON text (build.rs).
include!(concat!(env!("OUT_DIR"), "/worksheets.rs"));

const OTHER_FACTS: &str = "WorksheetFacts";

/// The linked engine's worksheet kinds, in its catalogue's order.
pub(crate) fn kinds() -> impl Iterator<Item = &'static str> {
    WORKSHEETS.iter().map(|(kind, ..)| *kind)
}

/// One kind's engine catalogue entry, parsed alone.
fn entry(kind: &str) -> Option<Value> {
    let (_, text, _) = WORKSHEETS.iter().find(|(named, ..)| *named == kind)?;
    Some(serde_json::from_str(text).expect("the build's catalogue entry"))
}

/// Give every engine worksheet without typed facts to `WorksheetFacts`, which takes
/// what the kind's catalogue entry lists, as the local server serves them.
pub(crate) fn open_other_kinds(schema: &mut Value) {
    let typed = &schema["properties"]["facts"]["discriminator"]["mapping"];
    let other: Vec<&str> = kinds().filter(|kind| typed.get(*kind).is_none()).collect();
    schema["$defs"][OTHER_FACTS]["properties"]["kind"]["enum"] = json!(other);
}

/// The kinds `WorksheetFacts` serves.
pub(crate) fn other_kinds(schema: &Value) -> impl Iterator<Item = &str> {
    let kinds = schema["$defs"][OTHER_FACTS]["properties"]["kind"]["enum"].as_array();
    kinds.into_iter().flatten().filter_map(Value::as_str)
}

/// The facts variant of `kind` in the tool's input schema, if it serves the kind.
fn variant<'a>(schema: &'a Value, kind: &str) -> Option<&'a Value> {
    match schema["properties"]["facts"]["discriminator"]["mapping"][kind].as_str() {
        Some(reference) => schema.pointer(reference.strip_prefix('#')?),
        None => other_kinds(schema)
            .any(|other| other == kind)
            .then(|| &schema["$defs"][OTHER_FACTS]),
    }
}

fn input_schema() -> Result<&'static Value, String> {
    let spec =
        crate::contracts::tool("calculate_tax_worksheet").ok_or("calculate_tax_worksheet")?;
    Ok(&spec["inputSchema"])
}

/// The engine's worksheets, periods and optional inputs this server's input schema can
/// reach. The linked engine can hold periods outside a year the schema fixes or
/// bounds, and optional inputs that the published contract does not advertise yet.
pub fn advertised_worksheets() -> Result<Value, String> {
    advertised(|(_, entry, _)| entry)
}

/// [`advertised_worksheets`] with only the fields the data freshness report reads.
pub(crate) fn advertised_freshness() -> Result<Value, String> {
    advertised(|(_, _, freshness)| freshness)
}

type Worksheet = (&'static str, &'static str, &'static str);

fn advertised(text: fn(&Worksheet) -> &'static str) -> Result<Value, String> {
    let schema = input_schema()?;
    let mut catalogue = serde_json::Map::new();
    for worksheet in WORKSHEETS {
        let kind = worksheet.0;
        let Some(variant) = variant(schema, kind) else {
            continue;
        };
        let mut entry: Value = serde_json::from_str(text(worksheet)).map_err(|e| e.to_string())?;
        served_scope(kind, &mut entry);
        let typed = variant["additionalProperties"] == false;
        let fields = entry.as_object_mut().ok_or("worksheet entry")?;
        if typed && let Some(Value::Object(optional)) = fields.get_mut("optional_inputs") {
            optional.retain(|name, _| variant["properties"].get(name).is_some());
            if optional.is_empty() {
                fields.remove("optional_inputs");
            }
        }
        if let Some(periods) = entry["supported_periods"].as_array_mut() {
            periods.retain(|period| {
                period["arguments"].as_object().is_none_or(|arguments| {
                    arguments
                        .iter()
                        .all(|(name, value)| admits(&variant["properties"][name], value))
                })
            });
        }
        catalogue.insert(kind.to_owned(), entry);
    }
    Ok(Value::Object(catalogue))
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
    let kind = object.remove("kind").unwrap().as_str().unwrap().to_owned();
    if variant(input_schema()?, &kind).is_some_and(|v| v["additionalProperties"] == true) {
        other_inputs(&kind, object)?;
    } else {
        typed_inputs(object)?;
    }
    let mut result = australian_tax_calculators::api::evaluate(&kind.replace('_', "-"), &facts)?;
    served_scope(&kind, &mut result);
    if input["file_note"] == true {
        result["file_note"] = json!(file_note(&input["facts"], &result)?);
    }
    Ok(result)
}

fn typed_inputs(object: &mut serde_json::Map<String, Value>) -> Result<(), String> {
    for (field, value) in object {
        if MONEY.contains(&field.as_str()) {
            *value = json!(super::money::amount(value.as_str().unwrap(), field)?.to_sci_string());
        } else if ["effective_life", "taxable_use"].contains(&field.as_str()) {
            let parsed = div7a_loan_review::decimal::Decimal::parse(value.as_str().unwrap())
                .map_err(|_| "[<class 'decimal.ConversionSyntax'>]".to_string())?;
            *value = json!(parsed.to_sci_string());
        }
    }
    Ok(())
}

/// The local server's checks of a kind without typed facts (`calculate` and `_plain`
/// in its adapters/tax.py): only the inputs its catalogue entry lists, and each
/// decimal string, however deep, in plain notation and stripped. The engine
/// validates and refuses each input.
fn other_inputs(kind: &str, object: &mut serde_json::Map<String, Value>) -> Result<(), String> {
    let entry = entry(kind).ok_or("worksheet catalogue")?;
    let listed = |name: &str| {
        ["required_inputs", "optional_inputs"]
            .iter()
            .any(|inputs| entry[*inputs].get(name).is_some())
    };
    let mut unknown: Vec<&str> = object
        .keys()
        .map(String::as_str)
        .filter(|name| !listed(name))
        .collect();
    if !unknown.is_empty() {
        unknown.sort_unstable();
        return Err(format!(
            "{kind} does not take {}; calculation_worksheets lists its required_inputs and optional_inputs.",
            unknown.join(", ")
        ));
    }
    object
        .iter_mut()
        .try_for_each(|(name, value)| plain(value, name))
}

fn plain(value: &mut Value, path: &str) -> Result<(), String> {
    static PLAIN: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r"^\s*-?[0-9]+(?:\.[0-9]+)?\s*$").unwrap()
    });
    match value {
        Value::Object(items) => items
            .iter_mut()
            .try_for_each(|(key, item)| plain(item, &format!("{path}.{key}"))),
        Value::Array(items) => items
            .iter_mut()
            .enumerate()
            .try_for_each(|(index, item)| plain(item, &format!("{path}[{index}]"))),
        Value::String(text) => match div7a_loan_review::decimal::Decimal::parse(text) {
            Ok(number) if number.is_finite() && !PLAIN.is_match(text) => Err(format!(
                "{path}: {} is not a decimal amount",
                div7a_loan_review::raw::Raw::text(text).py_repr()
            )),
            Ok(_) => {
                *text = text.trim().to_owned();
                Ok(())
            }
            Err(_) => Ok(()),
        },
        _ => Ok(()),
    }
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
    let calculation = result["calculation"].as_str().unwrap_or_default();
    let passage = shown(&entry(calculation).unwrap_or_default()["source_passage"]);
    let mut lines = facts_relied_on(calculation, facts, result);
    lines.extend([String::new(), "## Calculation".into(), String::new()]);
    lines.extend(table("Amount (AUD)", &result["amounts"]));
    lines.extend(table("Rate", &result["rates"]));
    lines.extend(sources_and_issues(&passage, result));
    lines.push(String::new());
    Ok(lines.join("\n"))
}

/// The note's heading and its facts relied on, as supplied.
fn facts_relied_on(calculation: &str, facts: &Value, result: &Value) -> Vec<String> {
    let text = |key: &str| shown(&result[key]);
    let mut lines = vec![
        format!("# File note: {calculation} worksheet, {}", text("period")),
        String::new(),
        format!(
            "Drafted from the {} {} result, which it restates without adding any figure.",
            text("engine"),
            text("engine_version")
        ),
        String::new(),
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
    lines
}

/// The note's assumptions, provisions and dated sources, and open issues.
fn sources_and_issues(passage: &str, result: &Value) -> Vec<String> {
    let text = |key: &str| shown(&result[key]);
    let items = |key: &str| -> Vec<String> {
        result[key]
            .as_array()
            .into_iter()
            .flatten()
            .map(shown)
            .collect()
    };
    let mut lines = vec![
        "## Assumptions".into(),
        String::new(),
        "The operator confirmed this scope:".into(),
        String::new(),
        text("scope"),
        String::new(),
        "## Provisions and sources".into(),
        String::new(),
        format!("The engine's source check covered: {passage}."),
        String::new(),
    ];
    let checked = text("source_checked");
    lines.extend(
        items("sources")
            .into_iter()
            .map(|url| format!("- {url} (checked {checked})")),
    );
    lines.extend([String::new(), "## Open issues".into(), String::new()]);
    lines.extend(
        items("warnings")
            .into_iter()
            .map(|warning| format!("- {warning}")),
    );
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

    /// Each narrowing still finds its words in the engine's scope, so an engine change
    /// that rewords a narrowed scope fails here instead of serving the wider words.
    #[test]
    fn every_narrowed_scope_is_in_the_engines_words() {
        let catalogue =
            australian_tax_calculators::api::evaluate("worksheet-catalogue", &json!({})).unwrap();
        for (kind, wide, _) in super::NARROWED_SCOPE {
            let scope = catalogue[kind]["scope"].as_str().unwrap();
            assert!(scope.contains(wide), "{kind}: {wide}");
        }
    }

    /// The published Medicare levy facts take no exemption days, so the served scope
    /// still excludes exemptions, in the catalogue and in a result.
    #[test]
    fn the_medicare_levy_scope_excludes_the_exemptions_its_facts_cannot_give() {
        let catalogue = super::advertised_worksheets().unwrap();
        let entry = &catalogue["medicare_levy"];
        let mut facts = entry["example"]["facts"].clone();
        let first = entry["supported_periods"][0]["arguments"]
            .as_object()
            .unwrap();
        facts.as_object_mut().unwrap().extend(first.clone());
        let result = super::calculate(&json!({ "facts": facts })).unwrap();
        for scope in [&entry["scope"], &result["scope"]] {
            let scope = scope.as_str().unwrap();
            assert!(scope.contains("not entitled to any full or half Medicare levy exemption"));
            assert!(scope.contains("Excludes exemptions,"), "{scope}");
        }
    }

    /// The published Medicare levy facts take no SAPTO, spouse or dependant inputs, so the
    /// served scope neither asks for them nor describes the reductions they drive.
    #[test]
    fn the_medicare_levy_scope_names_no_family_inputs_its_facts_cannot_give() {
        let catalogue = super::advertised_worksheets().unwrap();
        let scope = catalogue["medicare_levy"]["scope"].as_str().unwrap();
        for wide in ["SAPTO", "spouse's", "family reduction"] {
            assert!(!scope.contains(wide), "{wide}: {scope}");
        }
    }

    /// The served catalogue offers no optional input its kind's published facts refuse,
    /// such as the contribution caps carry-forward inputs.
    #[test]
    fn every_served_optional_input_is_a_published_fact() {
        let catalogue = super::advertised_worksheets().unwrap();
        let spec = crate::contracts::tool("calculate_tax_worksheet").unwrap();
        let schema = &spec["inputSchema"];
        let mapping = &schema["properties"]["facts"]["discriminator"]["mapping"];
        for (kind, entry) in catalogue.as_object().unwrap() {
            let pointer = mapping[kind].as_str().unwrap().strip_prefix('#').unwrap();
            let facts = &schema.pointer(pointer).unwrap()["properties"];
            let optional = entry.get("optional_inputs").and_then(Value::as_object);
            for name in optional.into_iter().flat_map(|names| names.keys()) {
                assert!(facts.get(name).is_some(), "{kind} offers {name}");
            }
        }
        assert!(
            catalogue["contribution_caps"]
                .get("optional_inputs")
                .is_none()
        );
    }

    /// The published contribution caps facts take only the unused total, so the served
    /// scope does not describe the carry-forward by year their schema refuses.
    #[test]
    fn the_contribution_caps_scope_describes_only_the_supplied_unused_total() {
        let catalogue = super::advertised_worksheets().unwrap();
        let entry = &catalogue["contribution_caps"];
        let mut facts = entry["example"]["facts"].clone();
        let first = entry["supported_periods"][0]["arguments"]
            .as_object()
            .unwrap();
        facts.as_object_mut().unwrap().extend(first.clone());
        let result = super::calculate(&json!({ "facts": facts })).unwrap();
        for scope in [&entry["scope"], &result["scope"]] {
            let scope = scope.as_str().unwrap();
            assert!(scope.contains("Unused concessional cap is the established total"));
            for wide in ["year by year", "proposed", "defined benefit"] {
                assert!(!scope.contains(wide), "{wide}: {scope}");
            }
        }
    }

    const GST: &str = r#"{"kind":"gst","scope_confirmed":true,"year":"2025-26","amount":"1100","gst_inclusive":true}"#;
    const HEADINGS: [&str; 5] = [
        "## Facts relied on",
        "## Calculation",
        "## Assumptions",
        "## Provisions and sources",
        "## Open issues",
    ];

    /// A worksheet result with its file note taken out.
    fn noted(facts: &Value) -> (Value, String) {
        let input = json!({ "facts": facts, "file_note": true });
        let mut result = super::calculate(&input).unwrap();
        let note = result.as_object_mut().unwrap().remove("file_note").unwrap();
        (result, note.as_str().unwrap().to_owned())
    }

    fn headings(note: &str) -> Vec<&str> {
        note.lines()
            .filter(|line| line.starts_with("## "))
            .collect()
    }

    /// As the Python server's tests/test_file_note.py has it.
    #[test]
    fn the_note_is_opt_in_and_leaves_the_result_unchanged() {
        let facts: Value = serde_json::from_str(GST).unwrap();
        let plain = super::calculate(&json!({ "facts": facts })).unwrap();
        assert!(plain.get("file_note").is_none());
        let declined = super::calculate(&json!({ "facts": facts, "file_note": false }));
        assert_eq!(declined.unwrap(), plain);
        assert_eq!(noted(&facts).0, plain);
    }

    /// The whole note for $1,100 including GST, in the Python server's
    /// render_file_note order: GST is 1,100 / 11 = $100.00. The scope, sources,
    /// warnings and source passage are the engine's own words.
    #[test]
    fn a_gst_note_restates_the_facts_figures_scope_sources_and_warnings() {
        let (result, note) = noted(&serde_json::from_str(GST).unwrap());
        let text = |key: &str| result[key].as_str().unwrap().to_owned();
        let list = |key: &str| result[key].as_array().unwrap().iter().map(text_of);
        let catalogue =
            australian_tax_calculators::api::evaluate("worksheet-catalogue", &json!({})).unwrap();
        let passage = catalogue["gst"]["source_passage"].as_str().unwrap();
        let mut expected = vec![
            "# File note: gst worksheet, 2025-26".to_owned(),
            String::new(),
            format!(
                "Drafted from the {} {} result, which it restates without adding any figure.",
                text("engine"),
                text("engine_version")
            ),
        ];
        expected.extend(GST_FACTS_AND_TABLES.lines().map(str::to_owned));
        expected.push(text("scope"));
        expected.extend(["", "## Provisions and sources", ""].map(str::to_owned));
        expected.push(format!("The engine's source check covered: {passage}."));
        expected.push(String::new());
        let checked = text("source_checked");
        expected.extend(list("sources").map(|url| format!("- {url} (checked {checked})")));
        expected.extend(["", "## Open issues", ""].map(str::to_owned));
        expected.extend(list("warnings").map(|warning| format!("- {warning}")));
        expected.push(String::new());
        assert_eq!(note, expected.join("\n"));
        assert_eq!(headings(&note), HEADINGS);
    }

    fn text_of(value: &Value) -> String {
        value.as_str().unwrap().to_owned()
    }

    const GST_FACTS_AND_TABLES: &str = "
## Facts relied on

Established by the operator and not verified by this server.

- `kind`: gst
- `scope_confirmed`: true
- `year`: 2025-26
- `amount`: 1100
- `gst_inclusive`: true

## Calculation

| Amount (AUD) | Value |
| --- | --- |
| `gst` | 100.00 |
| `exclusive` | 1000.00 |
| `inclusive` | 1100.00 |

| Rate | Value |
| --- | --- |
| `gst_rate` | 0.10 |

## Assumptions

The operator confirmed this scope:

";

    fn numbers(text: &str) -> std::collections::BTreeSet<String> {
        let number = regex::Regex::new(r"\d+(?:\.\d+)?").unwrap();
        number
            .find_iter(text)
            .map(|m| m.as_str().to_owned())
            .collect()
    }

    /// Every advertised kind's note has the 5 sections in order and every amount,
    /// and states no number beyond the facts, the result and the source passage.
    #[test]
    fn every_kind_yields_a_note_whose_every_number_comes_from_the_result() {
        let catalogue = super::advertised_worksheets().unwrap();
        for (kind, entry) in catalogue.as_object().unwrap() {
            let mut facts = entry["example"]["facts"].clone();
            let first = entry["supported_periods"][0]["arguments"]
                .as_object()
                .unwrap();
            facts.as_object_mut().unwrap().extend(first.clone());
            let (result, note) = noted(&facts);
            assert_eq!(headings(&note), HEADINGS, "{kind}");
            for (name, value) in result["amounts"].as_object().unwrap() {
                let row = format!("| `{name}` | {} |", super::shown(value));
                assert!(note.contains(&row), "{kind}: {row}");
            }
            let passage = entry["source_passage"].as_str().unwrap_or_default();
            let stated = numbers(&format!("{facts}{result}{passage}"));
            let invented: Vec<_> = numbers(&note).difference(&stated).cloned().collect();
            assert!(invented.is_empty(), "{kind} states {invented:?}");
        }
    }
}
