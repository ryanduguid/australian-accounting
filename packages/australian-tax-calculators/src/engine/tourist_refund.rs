// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `tourist_refund`: whether one traveller's purchases from one registered
//! supplier qualify for the tourist refund scheme when the goods leave as
//! accompanied baggage, and the refund of GST (GST Act s 168-5(1)) and of wine
//! tax borne (WET Act s 25-5(1)). The citations are to the GST Act
//! compilation 96 (C2026C00081), the GST Regulations 2019 compilation 5
//! (F2025C01015, from 1 November 2025) Division 168, the WET Act compilation
//! 32 (C2026C00149) and the WET Regulations 2019 (F2019L01154) Division 25,
//! all read on 24 November 2026. The External Territory route (s 168-5(1A))
//! is not worked.

use serde_json::{Value, json};

use super::{
    Arg, Result, Step, d, date_arg, derived_result, flag, int, precise, rate, refuse, scope, step,
};
use crate::dates;
use crate::decimal::Decimal;

mod items;
use items::Item;

const KIND: &str = "tourist_refund";
/// The GST Regulations compilation read starts on this day.
const FIRST_DEPARTURE: &str = "2025-11-01";

/// The facts as the command passes them; see the README for each.
pub struct TouristRefundArgs<'a> {
    pub departure_date: &'a Arg,
    pub departing_by: &'a Arg,
    pub departing_as: &'a Arg,
    pub trs_port: &'a Arg,
    pub accompanied_baggage: &'a Arg,
    pub minutes_before_departure: &'a Arg,
    pub acquired_by_traveller: &'a Arg,
    pub supplier_registered: &'a Arg,
    pub items: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

const ACQUIRED: &str = "GST Act s 168-5(1)(a): the traveller made the acquisition";
const REGISTERED: &str =
    "GST Regulations reg 168-5.03: the goods were acquired from a registered entity";
const DEPARTING_AS: &str = "GST Regulations reg 168-5.06: the traveller leaves other than in the course of employment as the person in charge or command of, or a member of the crew of, an aircraft or ship";
const PORT: &str = "GST Regulations reg 168-5.07: the traveller leaves at an airport or seaport at which the tourist refund scheme is administered";
const BAGGAGE: &str = "GST Act s 168-5(1)(c) and GST Regulations reg 168-5.08: the goods are exported as accompanied baggage";
const IN_TIME: &str = "GST Regulations reg 168-5.10(3): the traveller presents at, or lodges the claim at, the TRS verification facility at least 30 minutes (aircraft) or 60 minutes (ship) before the scheduled departure";
const THRESHOLD: &str = "GST Regulations reg 168-5.04: the purchase price of one item, or the total for 2 or more items from the same registered entity, is at least $300; only qualifying items count";
const TAXABLE: &str =
    "GST Act s 168-5(1)(a): the supply is a taxable supply, shown by GST on the tax invoice";
const GOODS_KIND: &str = "GST Regulations reg 168-5.02(a) to (c): not tobacco or tobacco products, and not an alcoholic beverage other than wine on which wine tax is taken to have been borne (WET Act s 31-10)";
const NOT_CONSUMED: &str =
    "GST Regulations reg 168-5.02(d): the goods are not partly consumed when the traveller leaves";
const INVOICE: &str =
    "GST Regulations reg 168-5.05(2): the acquisition is accompanied by a tax invoice";
const SIXTY_DAYS: &str = "GST Regulations reg 168-5.09: the goods are exported within 60 days after the day on which they were acquired";

/// One condition row.
fn condition(id: &str, authority: &str, met: bool) -> Value {
    json!({"condition": id, "authority": authority, "met": met})
}

/// The ids of the rows not met.
fn failing(rows: &[Value]) -> Vec<Value> {
    rows.iter()
        .filter(|row| row["met"] == false)
        .map(|row| row["condition"].clone())
        .collect()
}

/// One of `allowed`, or a refusal naming them.
fn choice(arg: &Arg, key: &str, allowed: &[&'static str]) -> Result<&'static str> {
    match allowed.iter().find(|a| arg.text() == Some(**a)) {
        Some(a) => Ok(a),
        None => refuse(
            "tourist_refund.fact_invalid",
            format!("{key} must be one of {}.", allowed.join(", ")),
        ),
    }
}

fn minutes(arg: &Arg) -> Result<i64> {
    match arg.int() {
        Some(Some(n)) if (0..=10_080).contains(&n) => Ok(n),
        _ => refuse(
            "tourist_refund.fact_invalid",
            "minutes_before_departure must be an integer from 0 to 10,080.",
        ),
    }
}

/// The departure day, inside `year` and on or after [`FIRST_DEPARTURE`].
fn departure(arg: &Arg, year: &str) -> Result<i64> {
    let day = date_arg(arg, "departure_date")?;
    let first = dates::parse(FIRST_DEPARTURE).expect("a date");
    if day < first {
        return refuse(
            "tourist_refund.departure_not_covered",
            "A departure before 1 November 2025 is not covered: only the GST Regulations 2019 compilation 5, in force from that day, was read.",
        );
    }
    let start: i64 = year[..4].parse().expect("a supported year");
    let (from, to) = (
        dates::parse(&format!("{start}-07-01")).expect("a date"),
        dates::parse(&format!("{}-06-30", start + 1)).expect("a date"),
    );
    if day < from || day > to {
        return refuse(
            "tourist_refund.departure_outside_year",
            format!("departure_date must fall in {year}."),
        );
    }
    Ok(day)
}

/// The 5 conditions each item must meet.
fn item_conditions(item: &Item, departure: i64) -> Vec<Value> {
    let goods_ok = match item.goods {
        "general" => true,
        "wine" => item.wine_tax_borne,
        _ => false,
    };
    vec![
        condition("taxable_supply", TAXABLE, !item.gst.is_zero()),
        condition("goods_kind", GOODS_KIND, goods_ok),
        condition("not_partly_consumed", NOT_CONSUMED, !item.partly_consumed),
        condition("tax_invoice", INVOICE, item.tax_invoice),
        condition("within_60_days", SIXTY_DAYS, departure - item.date <= 60),
    ]
}

/// A qualifying item: its number, facts and wine tax borne.
type Qualifying<'a> = (usize, &'a Item, Decimal);

/// WET Regulations reg 25-5.03: 29% of half the GST-inclusive price paid.
fn wine_tax(item: &Item) -> Result<Decimal> {
    if item.goods != "wine" {
        return Ok(int(0));
    }
    Ok(d("0.29").mul(&item.price.div(&int(2))?)?)
}

/// Each item's row, and the items that qualify.
fn assess(items: &[Item], departure: i64) -> Result<(Vec<Value>, Vec<Qualifying<'_>>)> {
    let mut rows = Vec::with_capacity(items.len());
    let mut qualifying = Vec::new();
    for (i, item) in items.iter().enumerate() {
        let conditions = item_conditions(item, departure);
        let failing = failing(&conditions);
        let qualifies = failing.is_empty();
        let wet = if qualifies { wine_tax(item)? } else { int(0) };
        rows.push(json!({
            "item": i + 1,
            "qualifies": qualifies,
            "failing_conditions": failing,
            "wine_tax_borne": wet.to_string(),
        }));
        if qualifies {
            qualifying.push((i + 1, item, wet));
        }
    }
    Ok((rows, qualifying))
}

/// A step adding `terms`, each a name and its value; `0` when there are none.
fn sum_step(
    id: &'static str,
    authority: &'static str,
    terms: &[(String, &Decimal)],
) -> Result<Step> {
    let mut value = int(0);
    for (_, v) in terms {
        value = value.add(v)?;
    }
    let names: Vec<&str> = terms.iter().map(|(n, _)| n.as_str()).collect();
    let formula = if names.is_empty() {
        "0".to_string()
    } else {
        names.join(" + ")
    };
    let operands: Vec<&Decimal> = terms.iter().map(|(_, v)| *v).collect();
    Ok(step(id, authority, &formula, &operands, value))
}

const PRICE_STEP: &str = "GST Regulations reg 168-5.04: the qualifying items' purchase prices";
const GST_STEP: &str = "GST Act s 168-5(1)(d): the GST payable on each qualifying taxable supply, nil unless every condition is met";
const WET_STEP: &str = "WET Act s 25-5(1)(d) and WET Regulations reg 25-5.03: 29% of half the GST-inclusive price of each qualifying wine, nil unless every condition is met";
const TOTAL_STEP: &str = "GST Act s 168-5(1) and WET Act s 25-5(1): the 2 refunds";

/// The 4 steps: qualifying price, the GST and wine tax refunds and their total.
fn steps(qualifying: &[Qualifying], eligible: bool) -> Result<Vec<Step>> {
    let prices: Vec<(String, &Decimal)> = qualifying
        .iter()
        .map(|(n, item, _)| (format!("price_{n}"), &item.price))
        .collect();
    let paid = if eligible { qualifying } else { &[] };
    let gst: Vec<(String, &Decimal)> = paid
        .iter()
        .map(|(n, item, _)| (format!("gst_{n}"), &item.gst))
        .collect();
    let wine: Vec<(String, &Decimal)> = paid
        .iter()
        .filter(|(_, item, _)| item.goods == "wine")
        .map(|(n, _, wet)| (format!("wine_tax_{n}"), wet))
        .collect();
    let gst_step = sum_step("gst_refund", GST_STEP, &gst)?;
    let wet_step = sum_step("wine_tax_refund", WET_STEP, &wine)?;
    let total = gst_step.value.add(&wet_step.value)?;
    let total_step = step(
        "total_refund",
        TOTAL_STEP,
        "gst_refund + wine_tax_refund",
        &[&gst_step.value, &wet_step.value],
        total,
    );
    Ok(vec![
        sum_step("qualifying_price", PRICE_STEP, &prices)?,
        gst_step,
        wet_step,
        total_step,
    ])
}

/// The 7 conditions of the claim as a whole.
fn claim_conditions(args: &TouristRefundArgs, threshold_met: bool) -> Result<Vec<Value>> {
    let by = choice(args.departing_by, "departing_by", &["aircraft", "ship"])?;
    let role = choice(
        args.departing_as,
        "departing_as",
        &["passenger", "crew", "in_charge"],
    )?;
    let needed = if by == "aircraft" { 30 } else { 60 };
    let minutes = minutes(args.minutes_before_departure)?;
    Ok(vec![
        condition(
            "acquired_by_traveller",
            ACQUIRED,
            flag(args.acquired_by_traveller, "acquired_by_traveller")?,
        ),
        condition(
            "supplier_registered",
            REGISTERED,
            flag(args.supplier_registered, "supplier_registered")?,
        ),
        condition("departing_as_passenger", DEPARTING_AS, role == "passenger"),
        condition("trs_port", PORT, flag(args.trs_port, "trs_port")?),
        condition(
            "accompanied_baggage",
            BAGGAGE,
            flag(args.accompanied_baggage, "accompanied_baggage")?,
        ),
        condition("presented_in_time", IN_TIME, minutes >= needed),
        condition("purchase_threshold", THRESHOLD, threshold_met),
    ])
}

const CASH: &str = "The refund is shown to the cent and not rounded for cash: regs 168-5.11 and 25-5.04 round a cash payment to the nearest 5 cents, reg 168-5.14 allows cash only at an airport facility for $200 or less including wine tax, and the ABF states that it does not pay TRS refunds in cash.";

/// GST Act s 168-5(1) and WET Act s 25-5(1): the conditions and the refund.
pub fn tourist_refund(args: &TouristRefundArgs) -> Result<Value> {
    precise(|| {
        let year = scope(args.scope_confirmed, args.year, KIND)?;
        let departure = departure(args.departure_date, &year)?;
        let items = items::items(args.items, departure)?;
        let (item_rows, qualifying) = assess(&items, departure)?;
        let mut price = int(0);
        for (_, item, _) in &qualifying {
            price = price.add(&item.price)?;
        }
        let rows = claim_conditions(args, !price.lt(&d("300"))?)?;
        let failing = failing(&rows);
        let eligible = failing.is_empty();
        let rates = vec![
            rate("purchase_threshold", "300"),
            rate("wine_tax_share_of_half_price", "0.29"),
        ];
        let mut out = derived_result(KIND, &year, steps(&qualifying, eligible)?, rates)?;
        out["eligible"] = json!(eligible);
        out["failing_conditions"] = Value::Array(failing);
        out["conditions"] = Value::Array(rows);
        out["items"] = Value::Array(item_rows);
        out["notes"] = json!([CASH]);
        Ok(out)
    })
}
