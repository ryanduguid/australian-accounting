//! The trial entry points' identifiers and refusals, without a provider:
//! `depreciation.evaluate_range`'s `case_id or ...` identifier (the retired
//! Python adapter's rule) and the formula worksheet's period lookup.
use lodgeit_calculator_adapter::decimal::Decimal;
use lodgeit_calculator_adapter::trials::Evaluation;
use lodgeit_calculator_adapter::trials::depreciation::{self, Asset};
use lodgeit_calculator_adapter::trials::fbt::{self, CarBenefit};

fn text(points: &[u32]) -> String {
    points.iter().filter_map(|&c| char::from_u32(c)).collect()
}

fn dec(text: &str) -> Decimal {
    Decimal::parse_exact(text).expect("a literal")
}

fn asset() -> Asset {
    Asset {
        case_id: "DEP-fixture".to_owned(),
        cost: dec("1000.00"),
        acquisition_date: "2024-07-01".to_owned(),
        useful_life_years: 5,
        method: "prime_cost".to_owned(),
    }
}

#[test]
fn a_given_case_id_names_the_range_and_an_empty_one_falls_back() {
    // Python: `case_id or f"{asset.case_id}:{from_date}..{to_date}:{day_count}"`.
    for (given, expected) in [
        (Some("given-id"), "given-id"),
        (Some(""), "DEP-fixture:2024-07-01..2025-06-30:actual/actual"),
        (None, "DEP-fixture:2024-07-01..2025-06-30:actual/actual"),
    ] {
        let comparison = depreciation::evaluate_range(
            &asset(),
            "2024-07-01",
            "2025-06-30",
            "actual/actual",
            None,
            None,
            given,
        )
        .expect("no provider, no call");
        assert_eq!(text(&comparison.case_id), expected, "{given:?}");
        assert_eq!(comparison.evaluation, Evaluation::NotRun, "{given:?}");
        assert_eq!(
            comparison
                .reasons
                .iter()
                .map(|r| text(r))
                .collect::<Vec<_>>(),
            ["No client supplied; the provider was not called."],
            "{given:?}"
        );
    }
}

#[test]
fn the_formula_worksheet_refuses_a_period_it_has_no_year_for() {
    // `PERIOD_MAP[uri]` for a missing key: KeyError, whose str is the key's repr.
    let benefit = CarBenefit {
        case_id: "FBT-fixture".to_owned(),
        base_value: dec("40000.00"),
        days_available: 365,
        accessories: dec("0.00"),
        employee_contribution: dec("0.00"),
        fbt_type: "Type 2".to_owned(),
        period_uri: "urn:fixture:period:none".to_owned(),
    };
    let error = fbt::run_local_statutory(&benefit).expect_err("no worksheet year");
    assert_eq!(error.class, "KeyError");
    assert_eq!(text(&error.text), "'urn:fixture:period:none'");
}
