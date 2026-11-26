// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Payroll tax grouping for 2026-27 against the Acts' own tracing examples
//! (Payroll Tax Act 2007 (Vic) ss 77(5) and 78(3)), SRO Victoria's 'Grouping'
//! examples and QRO's 'Payroll tax deductions', all read on 24 November 2026,
//! with each group deduction worked from the Act's group clause by hand
//! (docs/calculation-evidence.md#payroll-tax-group). Every person here is
//! fabricated.
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn input(place: &str, employer: &str, persons: Value) -> Value {
    json!({
        "jurisdiction": place, "employer": employer, "persons": persons,
        "employed_all_year": true, "year": "2026-27", "scope_confirmed": true,
    })
}

fn with(mut v: Value, key: &str, value: Value) -> Value {
    v[key] = value;
    v
}

fn run(input: &Value) -> Value {
    let result = api::evaluate("payroll-tax-group", input).unwrap();
    let envelope = json!({"command": "payroll-tax-group", "input": input});
    let doc = api::evaluate("derive", &envelope).unwrap();
    assert_eq!(doc["result"], result, "derive {input}");
    result
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("payroll-tax-group", input)
        .unwrap_err()
        .code
}

fn amount(result: &Value, key: &str) -> String {
    result["amounts"][key].as_str().unwrap().to_string()
}

fn wages(list: &[(&str, &str)]) -> Value {
    list.iter()
        .map(|(id, w)| json!({"id": id, "taxable_wages": w}))
        .collect()
}

fn holders(list: &[(&str, &str)]) -> Value {
    list.iter()
        .map(|(id, w)| json!({"id": id, "taxable_wages": w, "carries_on_business": w != &"0"}))
        .collect()
}

fn vote(holder: &str, corporation: &str, percent: &str) -> Value {
    json!({"holders": [holder], "corporation": corporation, "percent": percent})
}

fn control(holder: &[&str], business: &str, test: &str, percent: Option<&str>) -> Value {
    let mut v = json!({"holders": holder, "business": business, "test": test});
    if let Some(p) = percent {
        v["percent"] = json!(p);
    }
    v
}

/// Payroll Tax Act 2007 (Vic) s 77(5): 80% x 70% = 56% is a controlling
/// interest; 80% x 70% x 40% = 22.4% is not.
#[test]
fn tracing_follows_the_acts_chain_example() {
    let persons = holders(&[
        ("E", "0"),
        ("A", "500000"),
        ("B", "1200000"),
        ("C", "700000"),
    ]);
    let votes = json!([
        vote("E", "A", "80"),
        vote("A", "B", "70"),
        vote("B", "C", "40")
    ]);
    let v = with(input("vic", "B", persons), "voting_interests", votes);
    let result = run(&with(v.clone(), "designated_group_employers", json!(["B"])));
    assert_eq!(result["group"], json!(["E", "A", "B"]));
    assert_eq!(
        result["bases"][0]["basis"],
        "tracing of interests in corporations"
    );
    assert_eq!(
        result["bases"][0]["provision"],
        "Payroll Tax Act 2007 (Vic) s 73 and ss 76 to 78"
    );
    assert_eq!(
        code(&with(v, "employer", json!("C"))),
        "payroll_tax_group.not_grouped"
    );
}

/// Payroll Tax Act 2007 (Vic) s 78(3): a 40% direct interest and a 25% x 60%
/// indirect one aggregate to 55%. Here the middle corporation's own interest
/// is 50%, not more than 50%, so only the aggregate groups.
#[test]
fn tracing_aggregates_direct_and_indirect_interests() {
    let persons = holders(&[("E", "0"), ("A", "300000"), ("B", "900000")]);
    let votes = |direct: &str| {
        json!([
            vote("E", "B", direct),
            vote("E", "A", "30"),
            vote("A", "B", "50")
        ])
    };
    let v = input("vic", "B", persons);
    let grouped = run(&with(v.clone(), "voting_interests", votes("40")));
    assert_eq!(grouped["group"], json!(["E", "B"]));
    // 35% + 30% x 50% = 50%: not more than 50%.
    let at_half = with(v, "voting_interests", votes("35"));
    assert_eq!(code(&at_half), "payroll_tax_group.not_grouped");
}

/// SRO Victoria, 'Grouping': Ian controls Company A, which controls
/// Partnership B, so Ian controls B; Ian is not a business and stays out.
#[test]
fn control_passes_along_a_chain() {
    let persons = holders(&[
        ("Ian", "0"),
        ("A", "500000"),
        ("B", "400000"),
        ("C", "300000"),
    ]);
    let facts = json!([
        control(&["Ian"], "A", "voting_shares", Some("60")),
        control(&["A"], "B", "partnership_capital", Some("60")),
        control(&["Ian"], "C", "director_votes", Some("51")),
    ]);
    let result = run(&with(
        input("vic", "C", persons),
        "controlling_interests",
        facts,
    ));
    assert_eq!(result["group"], json!(["A", "B", "C"]));
    assert_eq!(result["designated_group_employer"], Value::Null);
    // With no designated group employer no member deducts (Sch 1 Pt 4).
    assert_eq!(amount(&result, "group_deduction"), "0.00");
    assert_eq!(amount(&result, "taxable_wages"), "1200000.00");
}

/// SRO Victoria, 'Grouping': Lucy, an object of a discretionary trust whose
/// trustee controls Trading Co, controls Trading Co (s 72(5), (6)).
#[test]
fn a_discretionary_object_controls_what_the_trustee_controls() {
    let persons = holders(&[
        ("Lucy", "0"),
        ("T", "0"),
        ("Trading", "800000"),
        ("Cafe", "300000"),
    ]);
    let facts = json!([
        control(&["Lucy"], "T", "discretionary_object", None),
        control(&["T"], "Trading", "voting_shares", Some("80")),
        control(&["Lucy"], "Cafe", "sole_owner", None),
    ]);
    let result = run(&with(
        input("vic", "Cafe", persons),
        "controlling_interests",
        facts,
    ));
    // The trust's business is carried on by its trustee, T, which joins too.
    assert_eq!(result["group"], json!(["T", "Trading", "Cafe"]));
    assert_eq!(result["bases"][0]["basis"], "common control");
}

#[test]
fn exactly_half_is_not_a_controlling_interest() {
    let persons = holders(&[("Ian", "0"), ("A", "500000"), ("B", "400000")]);
    let facts = |p: &str| {
        json!([
            control(&["Ian"], "A", "voting_shares", Some(p)),
            control(&["Ian"], "B", "voting_shares", Some("51")),
        ])
    };
    let v = input("vic", "A", persons);
    assert_eq!(
        code(&with(v.clone(), "controlling_interests", facts("50"))),
        "payroll_tax_group.not_grouped"
    );
    let grouped = run(&with(v, "controlling_interests", facts("50.01")));
    assert_eq!(grouped["group"], json!(["A", "B"]));
}

/// Members that together control a business bring it in (s 74(2)).
#[test]
fn members_together_bring_in_a_business_they_control() {
    let persons = holders(&[("A", "500000"), ("B", "400000"), ("D", "300000")]);
    let v = with(
        input("vic", "D", persons),
        "related_bodies_corporate",
        json!([["A", "B"]]),
    );
    let facts = json!([control(&["A", "B"], "D", "board_members", Some("60"))]);
    let result = run(&with(v, "controlling_interests", facts));
    assert_eq!(result["group"], json!(["A", "B", "D"]));
    let kinds: Vec<&str> = result["bases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["basis"].as_str().unwrap())
        .collect();
    assert_eq!(
        kinds,
        ["related bodies corporate", "smaller groups subsumed"]
    );
}

/// Sch 1 cl 7A: below the $3,000,000 phase-out start the DGE deducts the whole
/// $1,000,000; at $4,000,000 it falls by half the excess, to $500,000; with
/// half the wages interstate it is shared to $250,000.
#[test]
fn victoria_works_the_deduction_on_the_groups_wages() {
    let rbc = json!([["A", "B"]]);
    let case = |a: &str, b: &str, giw: &str| {
        let mut persons = wages(&[("A", a), ("B", b)]);
        persons[1]["interstate_wages"] = json!(giw);
        let v = with(
            input("vic", "B", persons),
            "related_bodies_corporate",
            rbc.clone(),
        );
        run(&with(v, "designated_group_employers", json!(["A"])))
    };
    assert_eq!(
        amount(&case("1400000.00", "600000.00", "0"), "group_deduction"),
        "1000000.00"
    );
    assert_eq!(
        amount(&case("3000000.00", "1000000.00", "0"), "group_deduction"),
        "500000.00"
    );
    let shared = case("1500000.00", "500000.00", "2000000.00");
    assert_eq!(amount(&shared, "interstate_wages"), "2000000.00");
    assert_eq!(amount(&shared, "group_deduction"), "250000.00");
    assert_eq!(shared["designated_group_employer"], "A");
}

/// QRO, 'Payroll tax deductions': only the DGE claims, and an excess goes to
/// a nominated member. $1,300,000 - $100,000 / 7 = $1,285,714.29; the DGE's
/// $1,000,000 of wages leaves $285,714.29 for B (ss 45, 48(2)).
#[test]
fn queensland_shares_the_excess_deduction_in_order() {
    let persons = wages(&[("A", "1000000.00"), ("B", "400000.00")]);
    let v = with(
        input("qld", "A", persons),
        "related_bodies_corporate",
        json!([["A", "B"]]),
    );
    let v = with(v, "designated_group_employers", json!(["A"]));
    let shared = run(&with(v.clone(), "excess_deduction_order", json!(["B"])));
    assert_eq!(amount(&shared, "group_deduction"), "1285714.29");
    assert_eq!(amount(&shared, "dge_deduction"), "1000000.00");
    assert_eq!(amount(&shared, "excess_deduction"), "285714.29");
    assert_eq!(amount(&shared, "excess_unshared"), "0.00");
    assert_eq!(
        shared["excess_shares"],
        json!([{"person": "B", "share": "285714.29"}])
    );
    // With no order stated the commissioner may determine one (s 47).
    assert_eq!(code(&v), "payroll_tax_group.excess_unnominated");
    // A member with no Queensland wages has no annual liability to share it.
    let mut idle = with(v.clone(), "excess_deduction_order", json!(["B"]));
    idle["persons"][1]["taxable_wages"] = json!("0");
    assert_eq!(code(&idle), "payroll_tax_group.excess_order_invalid");
    // B's own wages cap its share; what is left stays unshared (s 48(2)).
    // Splitting B's wages with C keeps the group's wages and the excess.
    let mut small = with(v.clone(), "excess_deduction_order", json!(["B"]));
    small["persons"] = wages(&[("A", "1000000.00"), ("B", "200000.00"), ("C", "200000.00")]);
    small["related_bodies_corporate"] = json!([["A", "B"], ["A", "C"]]);
    let capped = run(&small);
    assert_eq!(
        capped["excess_shares"],
        json!([{"person": "B", "share": "200000.00"}])
    );
    assert_eq!(amount(&capped, "excess_unshared"), "85714.29");
    let none = run(&with(v, "designated_group_employers", json!([])));
    assert_eq!(amount(&none, "group_deduction"), "0.00");
}

/// Pay-roll Tax Assessment Act 2002 (WA) s 17(3A): the group's deductable
/// amount, with or without a DGE: $1,000,000 - $500,000 x 1,000,000 /
/// 6,500,000 = $923,076.92.
#[test]
fn western_australia_gives_the_deduction_to_the_group() {
    let persons = wages(&[("A", "800000.00"), ("B", "700000.00")]);
    let v = with(
        input("wa", "A", persons),
        "common_employees",
        json!([["A", "B"]]),
    );
    let result = run(&v);
    assert_eq!(amount(&result, "group_deduction"), "923076.92");
    assert_eq!(
        result["bases"][0]["provision"],
        "Pay-roll Tax Assessment Act 2002 (WA) s 31"
    );
    let designated = run(&with(v, "designated_group_employers", json!(["B"])));
    assert_eq!(amount(&designated, "group_deduction"), "923076.92");
}

/// SA Sch 1 cl 9(2): GTW / (GTW + GIW) x EA = $600,000; ACT Sch 1 s 1.9(2):
/// the $1,750,000 threshold, refused when the DGE's wages are less.
#[test]
fn south_australia_and_the_act_deduct_through_the_dge() {
    let pair = |place: &str, a: &str, b: &str, dge: Value| {
        let v = with(
            input(place, "A", wages(&[("A", a), ("B", b)])),
            "related_bodies_corporate",
            json!([["A", "B"]]),
        );
        with(v, "designated_group_employers", dge)
    };
    let sa = run(&pair("sa", "1000000.00", "800000.00", json!(["A"])));
    assert_eq!(amount(&sa, "group_deduction"), "600000.00");
    assert_eq!(
        code(&pair("sa", "1000000.00", "800000.00", json!([]))),
        "payroll_tax_group.no_dge_unsettled"
    );
    let act = run(&pair("act", "2000000.00", "500000.00", json!(["A"])));
    assert_eq!(amount(&act, "group_deduction"), "1750000.00");
    let short = pair("act", "1000000.00", "1000000.00", json!(["A"]));
    assert_eq!(code(&short), "payroll_tax_group.dge_wages_below_deduction");
}

/// A pair of related corporations A and B in `place`, B with `giw` of
/// interstate wages, with the DGEs `dge`.
fn pair(place: &str, a: &str, b: &str, giw: &str, dge: Value) -> Value {
    let mut persons = wages(&[("A", a), ("B", b)]);
    persons[1]["interstate_wages"] = json!(giw);
    let v = with(
        input(place, "A", persons),
        "related_bodies_corporate",
        json!([["A", "B"]]),
    );
    with(v, "designated_group_employers", dge)
}

/// The authority `derive` cites for step `id` of `input`'s working.
fn authority(input: &Value, id: &str) -> String {
    let envelope = json!({"command": "payroll-tax-group", "input": input});
    let doc = api::evaluate("derive", &envelope).unwrap();
    let steps = doc["steps"].as_array().unwrap();
    let step = steps.iter().find(|s| s["id"] == id).unwrap();
    step["authority"].as_str().unwrap().to_string()
}

/// Payroll Tax Act 2007 (NSW) Sch 1 cl 9(2): 3,000,000 / 4,000,000 x
/// $1,200,000 = $900,000; cl 8: $1,100,000 or exactly $1,200,000 leaves no
/// member liable; cl 12: with no DGE each member pays on its own wages.
#[test]
fn new_south_wales_works_sch_1_parts_3_and_4() {
    let dge = json!(["A"]);
    let over = pair("nsw", "2000000.00", "1000000.00", "1000000.00", dge.clone());
    let result = run(&over);
    assert_eq!(amount(&result, "group_deduction"), "900000.00");
    assert_eq!(result["members_liable"], true);
    let cited = authority(&over, "group_deduction");
    assert!(cited.starts_with("Payroll Tax Act 2007 (NSW) Sch 1 cl 9(2)"));
    assert_eq!(
        result["bases"][0]["provision"],
        "Payroll Tax Act 2007 (NSW) s 70"
    );
    for (a, b) in [("700000.00", "400000.00"), ("800000.00", "400000.00")] {
        let under = pair("nsw", a, b, "0", dge.clone());
        let result = run(&under);
        assert_eq!(amount(&result, "group_deduction"), "0.00");
        assert_eq!(result["members_liable"], false);
        let cited = authority(&under, "group_deduction");
        assert!(cited.starts_with("Payroll Tax Act 2007 (NSW) Sch 1 cl 8"));
    }
    for (a, b) in [("2000000.00", "1000000.00"), ("700000.00", "400000.00")] {
        let none = run(&pair("nsw", a, b, "0", json!([])));
        assert_eq!(amount(&none, "group_deduction"), "0.00");
        assert_eq!(none["members_liable"], true);
    }
    let short = pair("nsw", "500000.00", "1500000.00", "0", dge);
    assert_eq!(code(&short), "payroll_tax_group.dge_wages_below_deduction");
}

/// Payroll Tax Act 2008 (Tas) Sch 1 cl 9A(2): $1,500,000 deducts the whole
/// $1,250,000; cl 9B(2): 4,000,000 / 5,000,000 x $1,250,000 = $1,000,000 and
/// x $2,000,000 = $1,600,000 from the R2 wages; cl 9: $1,200,000 leaves no
/// member liable; cl 12: with no DGE no member deducts.
#[test]
fn tasmania_works_both_thresholds() {
    let dge = json!(["A"]);
    let lower = run(&pair("tas", "1300000.00", "200000.00", "0", dge.clone()));
    assert_eq!(amount(&lower, "group_deduction"), "1250000.00");
    assert!(lower["amounts"].get("top_bracket_deduction").is_none());
    let upper = pair("tas", "3000000.00", "1000000.00", "1000000.00", dge.clone());
    let result = run(&upper);
    assert_eq!(amount(&result, "group_deduction"), "1000000.00");
    assert_eq!(amount(&result, "top_bracket_deduction"), "1600000.00");
    let cited = authority(&upper, "top_bracket_deduction");
    assert!(cited.contains("cl 8(2) and 9B(2)"));
    let under = run(&pair("tas", "700000.00", "500000.00", "0", dge.clone()));
    assert_eq!(under["members_liable"], false);
    let none = pair("tas", "3000000.00", "1000000.00", "0", json!([]));
    assert_eq!(amount(&run(&none), "group_deduction"), "0.00");
    assert!(authority(&none, "group_deduction").contains("cl 12"));
    let short = pair("tas", "1500000.00", "2500000.00", "0", dge);
    assert_eq!(code(&short), "payroll_tax_group.dge_wages_below_deduction");
}

/// Payroll Tax Act 2009 (NT) Sch 1 cl 7: AD = $2,500,000 - ($5,000,000 -
/// $2,500,000) / 2 = $1,250,000; cl 9(2): x 4,000,000 / 5,000,000 =
/// $1,000,000, only with the Commissioner's permission (cl 9(1)(c)); AD is
/// nil from $7,500,000; cl 8: $2,500,000 leaves no member liable.
#[test]
fn the_northern_territory_needs_written_permission() {
    let dge = json!(["A"]);
    let over = pair("nt", "3000000.00", "1000000.00", "1000000.00", dge.clone());
    assert_eq!(code(&over), "payroll_tax_group.nt_annual_unallowed");
    let refused = with(over.clone(), "annual_calculation_allowed", json!(false));
    assert_eq!(code(&refused), "payroll_tax_group.nt_annual_unallowed");
    let allowed = with(over, "annual_calculation_allowed", json!(true));
    let result = run(&allowed);
    assert_eq!(amount(&result, "australian_deduction"), "1250000.00");
    assert_eq!(amount(&result, "group_deduction"), "1000000.00");
    let nil = pair("nt", "5000000.00", "2500000.00", "0", dge.clone());
    let nil = with(nil, "annual_calculation_allowed", json!(true));
    assert_eq!(amount(&run(&nil), "group_deduction"), "0.00");
    let under = run(&pair("nt", "1500000.00", "1000000.00", "0", dge));
    assert_eq!(under["members_liable"], false);
    let none = run(&pair("nt", "3000000.00", "1000000.00", "0", json!([])));
    assert_eq!(none["members_liable"], true);
    assert_eq!(amount(&none, "group_deduction"), "0.00");
}

/// Nothing is estimated: the days on which a member paid wages (C) are not
/// an input, so a part year is refused naming the clause that defines C.
#[test]
fn a_part_year_names_the_clause_for_c() {
    for (place, act) in [
        ("nsw", "Payroll Tax Act 2007 (NSW) Sch 1 cl 7"),
        ("tas", "Payroll Tax Act 2008 (Tas) Sch 1 cl 7"),
        ("nt", "Payroll Tax Act 2009 (NT) Sch 1 cl 7"),
    ] {
        let v = pair(place, "2000000.00", "1000000.00", "0", json!(["A"]));
        let v = with(v, "employed_all_year", json!(false));
        let err = api::evaluate_coded("payroll-tax-group", &v).unwrap_err();
        assert_eq!(err.code, "payroll_tax_group.part_year");
        assert!(err.message.contains(act), "{}", err.message);
    }
}

#[test]
fn exclusions_take_a_member_out_unless_the_act_bars_it() {
    let persons = wages(&[("A", "500000.00"), ("B", "400000.00"), ("C", "300000.00")]);
    let v = with(
        input("vic", "A", persons),
        "common_employees",
        json!([["A", "B"], ["A", "C"]]),
    );
    let result = run(&with(v.clone(), "exclusions", json!(["C"])));
    assert_eq!(result["group"], json!(["A", "B"]));
    assert_eq!(result["excluded"], json!(["C"]));
    // Without A, B and C have no link of their own.
    let split = with(v.clone(), "exclusions", json!(["A"]));
    let split = with(split, "employer", json!("B"));
    assert_eq!(code(&split), "payroll_tax_group.not_grouped");
    let related = with(v.clone(), "related_bodies_corporate", json!([["A", "C"]]));
    assert_eq!(
        code(&with(related, "exclusions", json!(["C"]))),
        "payroll_tax_group.exclusion_barred"
    );
    let out = with(v, "exclusions", json!(["A"]));
    assert_eq!(code(&out), "payroll_tax_group.not_grouped");
}

#[test]
fn refusals_name_what_is_not_worked() {
    let persons = wages(&[("A", "500000.00"), ("B", "400000.00")]);
    let v = with(
        input("vic", "A", persons),
        "related_bodies_corporate",
        json!([["A", "B"]]),
    );
    let cases = [
        (
            with(v.clone(), "annual_calculation_allowed", json!(true)),
            "payroll_tax_group.fact_invalid",
        ),
        (
            with(v.clone(), "jurisdiction", json!("xx")),
            "payroll_tax_group.jurisdiction_invalid",
        ),
        (
            with(v.clone(), "employed_all_year", json!(false)),
            "payroll_tax_group.part_year",
        ),
        (
            with(v.clone(), "designated_group_employers", json!(["A", "B"])),
            "payroll_tax_group.dge_conflict",
        ),
        (
            with(v.clone(), "excess_deduction_order", json!(["B"])),
            "payroll_tax_group.excess_order_invalid",
        ),
        (
            with(v.clone(), "year", json!("2025-26")),
            "period.unsupported",
        ),
    ];
    for (case, expected) in cases {
        assert_eq!(code(&case), expected, "{case}");
    }
}

#[test]
fn refusals_name_the_fact_that_is_wrong() {
    let persons = wages(&[("A", "500000.00"), ("B", "400000.00"), ("C", "0")]);
    let v = input("vic", "A", persons);
    let rbc = |pairs: Value| with(v.clone(), "related_bodies_corporate", pairs);
    let ci = |p: &str| {
        with(
            v.clone(),
            "controlling_interests",
            json!([control(&["C"], "A", "voting_shares", Some(p))]),
        )
    };
    let cases = [
        (rbc(json!([["A", "Z"]])), "payroll_tax_group.person_unknown"),
        (rbc(json!([["A"]])), "payroll_tax_group.fact_invalid"),
        (ci("0"), "payroll_tax_group.percent_invalid"),
        (ci("100.01"), "payroll_tax_group.percent_invalid"),
        (ci("sixty"), "payroll_tax_group.percent_invalid"),
        (
            with(
                v.clone(),
                "controlling_interests",
                json!([control(&["C"], "A", "owner", None)]),
            ),
            "payroll_tax_group.test_invalid",
        ),
        (
            with(
                v.clone(),
                "voting_interests",
                json!([vote("A", "B", "10"), vote("B", "A", "10")]),
            ),
            "payroll_tax_group.interest_cycle",
        ),
        (
            with(
                rbc(json!([["A", "B"]])),
                "designated_group_employers",
                json!(["C"]),
            ),
            "payroll_tax_group.dge_not_member",
        ),
    ];
    for (case, expected) in cases {
        assert_eq!(code(&case), expected, "{case}");
    }
}

#[test]
fn persons_are_listed_once_in_the_stated_form() {
    let v = input("vic", "A", wages(&[("A", "500000.00")]));
    let cases = [
        (
            with(v.clone(), "persons", json!([{"id": "A"}, {"id": "A"}])),
            "payroll_tax_group.person_duplicate",
        ),
        (
            with(v.clone(), "persons", json!("A")),
            "payroll_tax_group.fact_invalid",
        ),
        (
            with(
                v.clone(),
                "persons",
                json!([{"id": "A", "taxable_wage": "1"}]),
            ),
            "input.field_unknown",
        ),
    ];
    for (case, expected) in cases {
        assert_eq!(code(&case), expected, "{case}");
    }
}

/// Queensland s 74(4)(b) bars excluding a set only for a related body
/// corporate outside it; Victoria s 79(3) bars excluding either corporation.
#[test]
fn queensland_excludes_a_set_of_related_corporations() {
    let persons = wages(&[
        ("A", "500000.00"),
        ("B", "400000.00"),
        ("C", "300000.00"),
        ("D", "200000.00"),
    ]);
    let v = input("qld", "A", persons);
    let v = with(v, "common_employees", json!([["A", "B"], ["A", "D"]]));
    let v = with(v, "related_bodies_corporate", json!([["B", "C"]]));
    let v = with(v, "exclusions", json!(["B", "C"]));
    assert_eq!(run(&v)["group"], json!(["A", "D"]));
    let vic = with(v.clone(), "jurisdiction", json!("vic"));
    assert_eq!(code(&vic), "payroll_tax_group.exclusion_barred");
    let part = with(v, "exclusions", json!(["B"]));
    assert_eq!(code(&part), "payroll_tax_group.exclusion_barred");
}

#[test]
fn a_sole_holder_must_say_whether_it_carries_on_a_business() {
    let persons = wages(&[("A", "500000.00"), ("B", "400000.00")]);
    let facts = json!([control(&["A"], "B", "voting_shares", Some("60"))]);
    let v = with(input("vic", "A", persons), "controlling_interests", facts);
    assert_eq!(code(&v), "payroll_tax_group.business_unstated");
    let mut stated = v.clone();
    stated["persons"][0]["carries_on_business"] = json!(true);
    assert_eq!(run(&stated)["group"], json!(["A", "B"]));
    // A holder that carries on no business of its own groups nothing.
    stated["persons"][0]["carries_on_business"] = json!(false);
    assert_eq!(code(&stated), "payroll_tax_group.not_grouped");
}

#[test]
fn a_voting_interest_is_given_once() {
    let persons = holders(&[("E", "0"), ("B", "900000")]);
    let votes = json!([vote("E", "B", "30"), vote("E", "B", "30")]);
    let v = with(input("vic", "B", persons), "voting_interests", votes);
    assert_eq!(code(&v), "payroll_tax_group.fact_invalid");
}
