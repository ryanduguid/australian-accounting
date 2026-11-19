// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Capital gains and losses from parcels of shares or units (ITAA 1997
//! ss 102-5, 104-10, 110-25, 110-55, 115-25 and 115-100, compilation 264)
//! against the ATO's Kathleen examples (2026 supplement, question 18), Boris's
//! example ('Keeping records of shares and units') and the 12-month count of
//! 'CGT discount', all read on 19 November 2026, the Library's David Jones
//! example (Tax Examples, CGT, 2-240) and hand-worked cases
//! (docs/calculation-evidence.md#cgt-parcels).
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn run(input: &Value) -> Value {
    let result = api::evaluate("cgt-parcels", input).unwrap();
    // derive gives the same result.
    let envelope = json!({"command": "cgt-parcels", "input": input});
    let doc = api::evaluate("derive", &envelope).unwrap();
    assert_eq!(doc["result"], result, "derive {input}");
    result
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("cgt-parcels", input).unwrap_err().code
}

fn amounts<'a, const N: usize>(result: &'a Value, keys: [&str; N]) -> [&'a str; N] {
    keys.map(|key| result["amounts"][key].as_str().unwrap())
}

const NET: [&str; 8] = [
    "total_capital_gains",
    "losses_applied_to_other_gains",
    "losses_applied_to_discount_gains",
    "prior_losses_applied_to_other_gains",
    "prior_losses_applied_to_discount_gains",
    "discount",
    "net_capital_gain",
    "net_capital_losses_carried_forward",
];

fn parcel(id: &str, security: &str, acquired: &str, units: &str, cost: &str) -> Value {
    json!({"parcel": id, "security": security, "acquired": acquired, "units": units,
           "acquisition_cost": cost, "incidental_costs": "0.00"})
}

fn sale(security: &str, date: &str, units: &str, proceeds: &str) -> Value {
    json!({"security": security, "date": date, "units": units,
           "capital_proceeds": proceeds, "incidental_costs": "0.00"})
}

fn facts(entity: &str, parcels: Value, disposals: Value) -> Value {
    json!({
        "entity": entity, "identification": "first_in_first_out",
        "parcels": parcels, "disposals": disposals,
        "other_discount_capital_gains": "0.00", "other_capital_gains": "0.00",
        "other_capital_losses": "0.00", "prior_net_capital_losses": "0.00",
        "year": "2025-26", "scope_confirmed": true,
    })
}

/// Kathleen (examples 1 to 4): $3,000 by the discount method and $520 by the
/// other method make label H $3,520; her $600 share loss takes the $520 first
/// and $80 of the discount gain, her $400 of earlier losses take $400 more,
/// and half of the $2,520 left is her $1,260 net capital gain. Her collectables
/// loss, which only collectables' gains can use, is left out.
fn kathleen() -> Value {
    let mut input = facts(
        "individual",
        json!([
            parcel("K1", "Company A", "2000-11-17", "1000", "3000.00"),
            parcel("K2", "Company B", "2025-10-10", "130", "520.00"),
            parcel("K3", "Company C", "2025-10-10", "600", "2400.00"),
        ]),
        json!([
            sale("Company A", "2025-12-17", "1000", "6000.00"),
            sale("Company B", "2026-02-27", "130", "1040.00"),
            sale("Company C", "2026-06-25", "600", "1800.00"),
        ]),
    );
    input["prior_net_capital_losses"] = json!("400.00");
    input
}

#[test]
fn kathleen_gives_labels_h_a_and_v() {
    let r = run(&kathleen());
    assert_eq!(
        amounts(&r, NET),
        [
            "3520.00", "520.00", "80.00", "0.00", "400.00", "1260.00", "1260.00", "0.00"
        ]
    );
    let matched: Vec<(&str, &str, bool)> = r["matches"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| {
            let gain_or_loss = if m["capital_loss"] == "0.00" {
                &m["capital_gain"]
            } else {
                &m["capital_loss"]
            };
            (
                m["parcel"].as_str().unwrap(),
                gain_or_loss.as_str().unwrap(),
                m["discount"]
                    .as_str()
                    .is_some_and(|d| d.starts_with("discount")),
            )
        })
        .collect();
    assert_eq!(
        matched,
        [
            ("K1", "3000.00", true),
            ("K2", "520.00", false),
            ("K3", "600.00", false)
        ]
    );
    assert_eq!(r["holdings"], json!([]));
    // The catalogue's example is this case.
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let mut example = catalogue["cgt_parcels"]["example"]["facts"].clone();
    example.as_object_mut().unwrap().remove("kind");
    example["scope_confirmed"] = json!(true);
    assert_eq!(example, kathleen());
}

/// Boris bought 1,000 shares at $5 in 2024 and 3,000 at $10 in 2025, and sold
/// 1,500 at $8 in 2026. Selecting the 2025 shares gives a $3,000 loss and
/// leaves 1,000 at $5 and 1,500 at $10; first in, first out sells the 1,000
/// at $5 ($3,000 gain, discounted) and 500 at $10 ($1,000 loss) instead. The
/// days are fabricated within the ATO's years.
fn boris(identification: &str) -> Value {
    let mut sold = sale("Company", "2026-03-02", "1500", "12000.00");
    if identification == "selected" {
        sold["parcels"] = json!([{"parcel": "2025", "units": "1500"}]);
    }
    let mut input = facts(
        "individual",
        json!([
            parcel("2024", "Company", "2024-03-01", "1000", "5000.00"),
            parcel("2025", "Company", "2025-03-03", "3000", "30000.00"),
        ]),
        json!([sold]),
    );
    input["identification"] = json!(identification);
    input
}

#[test]
fn boris_selects_the_parcel_that_gives_a_loss() {
    let r = run(&boris("selected"));
    assert_eq!(
        amounts(
            &r,
            [
                "parcel_losses",
                "net_capital_gain",
                "net_capital_loss",
                "net_capital_losses_carried_forward"
            ]
        ),
        ["3000.00", "0.00", "3000.00", "3000.00"]
    );
    assert_eq!(
        r["holdings"],
        json!([
            {"parcel": "2024", "security": "Company", "acquired": "2024-03-01", "units": "1000",
             "cost_base": "5000.00", "reduced_cost_base": "5000.00"},
            {"parcel": "2025", "security": "Company", "acquired": "2025-03-03", "units": "1500",
             "cost_base": "15000.00", "reduced_cost_base": "15000.00"},
        ])
    );
    let fifo = run(&boris("first_in_first_out"));
    assert_eq!(
        amounts(
            &fifo,
            [
                "parcel_discount_gains",
                "parcel_losses",
                "discount",
                "net_capital_gain"
            ]
        ),
        ["3000.00", "1000.00", "1000.00", "1000.00"]
    );
    assert_eq!(fifo["holdings"][0]["parcel"], "2025");
    assert_eq!(fifo["holdings"][0]["units"], "2500");
}

/// David Jones (Library, CGT 2-240, 2025-26): first in, first out sells 2,000
/// of his original CSR shares at $1 against their $2 cost, a $2,000 loss; 1,000
/// of his 5,000 WBC shares (bonus shares included, $60,000 in all) sell at $20
/// within 12 months, an $8,000 gain by the other method. His IBM loss ($6,660)
/// and TEL discount gain ($21,435, pre-1999 parcels) are given as other gains
/// and losses: $8,660 of losses take the $8,000 and $660 of the TEL gain, and
/// half of $20,775 is $10,387.50. The CSR bonus shares' $2,000 dividend amount
/// is fabricated; the example does not state it.
#[test]
fn david_jones_losses_go_to_the_other_gain_first() {
    let mut input = facts(
        "individual",
        json!([
            parcel("CSR original", "CSR Ltd", "2006-09-10", "6000", "12000.00"),
            parcel("CSR bonus", "CSR Ltd", "2026-04-01", "2000", "2000.00"),
            parcel("WBC", "WBC Ltd", "2025-12-14", "5000", "60000.00"),
        ]),
        json!([
            sale("CSR Ltd", "2026-06-01", "2000", "2000.00"),
            sale("WBC Ltd", "2026-06-01", "1000", "20000.00"),
        ]),
    );
    input["other_discount_capital_gains"] = json!("21435.00");
    input["other_capital_losses"] = json!("6660.00");
    let r = run(&input);
    assert_eq!(
        amounts(&r, NET),
        [
            "29435.00", "8000.00", "660.00", "0.00", "0.00", "10387.50", "10387.50", "0.00"
        ]
    );
    assert_eq!(
        amounts(&r, ["parcel_other_gains", "parcel_losses"]),
        ["8000.00", "2000.00"]
    );
    assert_eq!(r["holdings"][0]["units"], "4000");
    assert_eq!(r["holdings"][0]["cost_base"], "8000.00");
}

/// 'CGT discount' (last updated 29 June 2026): neither the day of acquisition
/// nor the day of the event counts, so shares bought on 2 February 2025 give a
/// discount capital gain from a contract on 3 February 2026, not 2 February.
#[test]
fn the_12_months_count_neither_day() {
    let gains = |date: &str| {
        let input = facts(
            "individual",
            json!([parcel("P", "S", "2025-02-02", "1", "800.00")]),
            json!([sale("S", date, "1", "1000.00")]),
        );
        let r = run(&input);
        amounts(
            &r,
            [
                "parcel_discount_gains",
                "parcel_other_gains",
                "net_capital_gain",
            ],
        )
        .map(str::to_string)
    };
    assert_eq!(gains("2026-02-03"), ["200.00", "0.00", "100.00"]);
    assert_eq!(gains("2026-02-02"), ["0.00", "200.00", "200.00"]);
}

/// Worked by hand: a $289,755 gain held over 12 months is discounted by a third
/// for a complying superannuation entity ($96,585) and not at all for a company.
#[test]
fn the_discount_percentage_follows_the_entity() {
    let one = |entity: &str| {
        let input = facts(
            entity,
            json!([parcel("P", "S", "2020-01-15", "1000", "206245.00")]),
            json!([sale("S", "2026-02-01", "1000", "496000.00")]),
        );
        let r = run(&input);
        amounts(&r, ["discount", "net_capital_gain"]).map(str::to_string)
    };
    assert_eq!(
        one("complying_superannuation_entity"),
        ["96585.00", "193170.00"]
    );
    assert_eq!(one("trust"), ["144877.50", "144877.50"]);
    assert_eq!(one("company"), ["0.00", "289755.00"]);
}

/// Worked by hand for a company in 2026-27: a parcel bought for $100.00 and
/// sold for nothing is a $100 loss, which takes $100 of the $150 of other
/// gains (step 1); $20 of earlier losses take $20 more (step 2), leaving a
/// $30 net capital gain and nothing to carry forward.
#[test]
fn a_company_loss_and_earlier_losses_reduce_its_other_gains() {
    let mut input = facts(
        "company",
        json!([parcel("P", "S", "2025-01-15", "10", "100.00")]),
        json!([sale("S", "2026-09-01", "10", "0.00")]),
    );
    input["year"] = json!("2026-27");
    input["other_capital_gains"] = json!("150.00");
    input["prior_net_capital_losses"] = json!("20.00");
    let r = run(&input);
    assert_eq!(
        amounts(&r, NET),
        ["150.00", "100.00", "0.00", "20.00", "0.00", "0.00", "30.00", "0.00"]
    );
    assert_eq!(r["matches"][0]["capital_loss"], "100.00");
}

/// Worked by hand: 3 units bought for $100.00 and $10.00 brokerage, sold one by
/// one and then 2 together, take $36.67 and then the $73.33 left of the cost;
/// a sale of 3 units for $100.00 across parcels of 1 and 2 units gives them
/// $33.33 and $66.67. Each split rounds to the cent and the last units take
/// the rest, so nothing is lost or counted twice.
#[test]
fn apportioned_amounts_add_up_to_the_whole() {
    let mut bought = parcel("P", "S", "2025-08-01", "3", "100.00");
    bought["incidental_costs"] = json!("10.00");
    let input = facts(
        "individual",
        json!([bought]),
        json!([
            sale("S", "2025-09-01", "1", "50.00"),
            sale("S", "2025-10-01", "2", "100.00")
        ]),
    );
    let r = run(&input);
    let bases: Vec<&str> = r["matches"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["cost_base"].as_str().unwrap())
        .collect();
    assert_eq!(bases, ["36.67", "73.33"]);
    assert_eq!(amounts(&r, ["parcel_other_gains"]), ["40.00"]);
    let input = facts(
        "individual",
        json!([
            parcel("A", "S", "2025-08-01", "1", "30.00"),
            parcel("B", "S", "2025-08-02", "2", "60.00")
        ]),
        json!([sale("S", "2025-09-01", "3", "100.00")]),
    );
    let r = run(&input);
    let proceeds: Vec<&str> = r["matches"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["capital_proceeds"].as_str().unwrap())
        .collect();
    assert_eq!(proceeds, ["33.33", "66.67"]);
}

/// Worked by hand: costs of owning are in the cost base but not the reduced
/// cost base (ss 110-25(4) and 110-55(2)), so proceeds between the two give
/// neither a gain nor a loss.
#[test]
fn costs_of_owning_stay_out_of_the_reduced_cost_base() {
    let mut bought = parcel("P", "S", "2024-01-10", "10", "1000.00");
    bought["ownership_costs"] = json!("100.00");
    let r = run(&facts(
        "individual",
        json!([bought]),
        json!([sale("S", "2025-07-10", "10", "1050.00")]),
    ));
    assert_eq!(
        amounts(&r, ["total_capital_gains", "capital_losses"]),
        ["0.00", "0.00"]
    );
    assert_eq!(r["matches"][0]["cost_base"], "1100.00");
    assert_eq!(r["matches"][0]["reduced_cost_base"], "1000.00");
}

/// A sale in an earlier year uses up the earliest units but is not counted;
/// first in, first out then takes the next. Its parcel may be pre-1999.
#[test]
fn an_earlier_sale_uses_up_units_without_being_counted() {
    let input = facts(
        "individual",
        json!([
            parcel("old", "S", "1998-05-01", "100", "100.00"),
            parcel("new", "S", "2023-05-01", "100", "500.00")
        ]),
        json!([
            sale("S", "2024-05-01", "100", "300.00"),
            sale("S", "2025-08-01", "50", "400.00")
        ]),
    );
    let r = run(&input);
    assert_eq!(r["matches"][0]["counted"], false);
    assert_eq!(r["matches"][0]["income_year"], "2023-24");
    assert_eq!(r["matches"][1]["parcel"], "new");
    assert_eq!(
        amounts(&r, ["parcel_discount_gains", "net_capital_gain"]),
        ["150.00", "75.00"]
    );
    assert_eq!(r["holdings"][0]["units"], "50");
}

#[test]
fn refusals_name_what_is_out_of_scope_or_wrong() {
    let base = || boris("selected");
    let with = |edit: &dyn Fn(&mut Value)| {
        let mut input = base();
        edit(&mut input);
        code(&input)
    };
    assert_eq!(
        with(&|v| v["entity"] = json!("partnership")),
        "cgt_parcels.entity_invalid"
    );
    assert_eq!(
        with(&|v| v["identification"] = json!("average")),
        "cgt_parcels.identification_invalid"
    );
    assert_eq!(
        with(&|v| v["parcels"] = json!([])),
        "cgt_parcels.list_invalid"
    );
    assert_eq!(
        with(&|v| v["year"] = json!("2027-28")),
        "period.unsupported"
    );
    assert_eq!(
        with(&|v| v["parcels"][1]["acquired"] = json!("1999-09-21")),
        "cgt_parcels.indexation_or_pre_cgt"
    );
    // The day after is not refused, nor is an unsold parcel acquired earlier.
    let mut later = base();
    later["parcels"][1]["acquired"] = json!("1999-09-22");
    later["parcels"][0]["acquired"] = json!("1985-01-01");
    assert!(api::evaluate("cgt-parcels", &later).is_ok());
    assert_eq!(
        with(&|v| v["disposals"][0]["parcels"][0]["units"] = json!("1400")),
        "cgt_parcels.fact_conflict"
    );
    assert_eq!(
        with(&|v| v["disposals"][0]["parcels"][0]["units"] = json!("3001")),
        "cgt_parcels.units_not_held"
    );
    assert_eq!(
        with(&|v| v["disposals"][0]["parcels"][0]["parcel"] = json!("2023")),
        "cgt_parcels.fact_conflict"
    );
    assert_eq!(
        with(&|v| v["disposals"][0]["security"] = json!("Other")),
        "cgt_parcels.fact_conflict"
    );
    assert_eq!(
        with(&|v| v["disposals"][0]["date"] = json!("2025-03-02")),
        "cgt_parcels.fact_conflict"
    );
    assert_eq!(
        with(&|v| v["disposals"][0]["date"] = json!("2026-07-01")),
        "cgt_parcels.fact_conflict"
    );
    assert_eq!(
        with(&|v| v["parcels"][1]["parcel"] = json!("2024")),
        "cgt_parcels.fact_conflict"
    );
    assert_eq!(
        with(&|v| v["parcels"][1]["acquired"] = json!("2026-07-01")),
        "cgt_parcels.fact_conflict"
    );
    assert_eq!(
        with(&|v| v["parcels"][0]["units"] = json!("0")),
        "cgt_parcels.fact_invalid"
    );
    assert_eq!(
        with(&|v| v["parcels"][0]["units"] = json!("1.0000001")),
        "cgt_parcels.fact_invalid"
    );
    assert_eq!(
        with(&|v| v["parcels"][0]["acquisition_cost"] = json!("1.001")),
        "cgt_parcels.fact_invalid"
    );
    assert_eq!(
        with(&|v| {
            v["parcels"][0].as_object_mut().unwrap().remove("security");
        }),
        "cgt_parcels.fact_missing"
    );
    assert_eq!(
        with(&|v| {
            v["disposals"][0].as_object_mut().unwrap().remove("parcels");
        }),
        "cgt_parcels.fact_missing"
    );
    assert_eq!(
        with(&|v| v["parcels"][0]["indexed"] = json!(true)),
        "input.field_unknown"
    );
    assert_eq!(
        with(&|v| v["identification"] = json!("first_in_first_out")),
        "input.field_unknown"
    );
    assert_eq!(
        with(&|v| {
            v["entity"] = json!("company");
            v["other_discount_capital_gains"] = json!("1.00");
        }),
        "cgt_parcels.fact_conflict"
    );
    // First in, first out: more units than the parcels acquired by then hold.
    let mut input = boris("first_in_first_out");
    input["disposals"][0]["units"] = json!("4001");
    input["disposals"][0]["capital_proceeds"] = json!("1.00");
    assert_eq!(code(&input), "cgt_parcels.units_not_held");
    input["disposals"][0]["units"] = json!("1001");
    input["disposals"][0]["date"] = json!("2025-03-02");
    assert_eq!(code(&input), "cgt_parcels.units_not_held");
}
