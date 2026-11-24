// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `gst_food_retailer_sam`: for one tax period of a retailer who sells food
//! and has chosen a simplified accounting method under GST Act Division 123,
//! the GST-free share of trading sales and trading stock purchases the method
//! gives, and the input tax credits on trading stock where the method's
//! formula goes that far. The methods are those of the 3 determinations in
//! force for retailers who sell food: F2017L01274 (business norms, stock
//! purchases and snapshot methods), F2025L01055 (supermarkets and convenience
//! stores) and F2026L01275 (restaurants, cafes and caterers), each read on 24
//! November 2026 with the GST Act compilation 96 (C2026C00081).

mod facts;
mod methods;

use std::cell::RefCell;

use serde_json::Value;

use super::{Arg, Result, derived_result, money, precise, refuse, scope};
use facts::{Condition, Given, Want, eligible, period};

const KIND: &str = "gst_food_retailer_sam";

/// The facts as the command passes them; `given` holds each optional input
/// present, which the method then reads or refuses.
pub struct GstFoodRetailerSamArgs<'a> {
    pub method: &'a Arg,
    pub tax_period_start: &'a Arg,
    pub tax_period_end: &'a Arg,
    pub trading_stock_purchases: &'a Arg,
    pub eligibility: &'a Arg,
    pub given: &'a [(&'static str, Arg)],
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// The inputs a command may give beside the required ones.
pub const GST_FOOD_RETAILER_SAM_OPTIONAL: [&str; 16] = [
    "total_sales",
    "retailer_type",
    "stock_role",
    "fuel_and_post_sales",
    "fuel_and_post_purchases",
    "pbs_sales",
    "non_pbs_sales",
    "otc_sales",
    "non_pbs_purchases",
    "otc_purchases",
    "gst_free_purchases",
    "gst_free_sales",
    "purchases_sample",
    "sales_sample",
    "product_lines",
    "trading_started",
];

#[derive(Clone, Copy, PartialEq)]
enum Method {
    BusinessNorms,
    StockEvery,
    StockSamples,
    StockFive,
    SnapEvery,
    SnapSamples,
    SnapFive,
    Supermarket,
    Restaurant,
}

const METHODS: [(&str, Method); 9] = [
    ("business_norms", Method::BusinessNorms),
    ("stock_purchases_every_period", Method::StockEvery),
    ("stock_purchases_two_samples", Method::StockSamples),
    ("stock_purchases_five_percent", Method::StockFive),
    ("snapshot_every_period", Method::SnapEvery),
    ("snapshot_two_samples", Method::SnapSamples),
    ("snapshot_five_percent", Method::SnapFive),
    ("supermarket_sales_percentage", Method::Supermarket),
    ("restaurant_purchases_snapshot", Method::Restaurant),
];

const RETAIL_2017: [Condition; 3] = [
    (
        "same_premises_taxable_and_gst_free_food",
        Want::True,
        "F2017L01274 s 4(1)(a): sells taxable and GST-free food from the same premises",
    ),
    (
        "sam_turnover",
        Want::AtMostTwoMillion,
        "F2017L01274 s 4(1)(b) and s 8 'SAM turnover': $2 million or less",
    ),
    (
        "adequate_point_of_sale_equipment",
        Want::False,
        "F2017L01274 s 4(1)(c): does not have adequate point-of-sale equipment",
    ),
];

const FIVE_PERCENT: Condition = (
    "projected_gst_free_at_most_5_percent",
    Want::True,
    "F2017L01274 s 6(5) or s 7(5): the projected GST-free share is not more than 5% (exactly 5% qualifies)",
);

/// The business norms suit only a business that meets every part of its
/// retailer type's s 8 definition, which the operator establishes.
const DEFINITION: Condition = (
    "retailer_type_definition_met",
    Want::True,
    "F2017L01274 s 5 and s 8: the business meets every part of the definition of its retailer_type",
);

const SUPERMARKET: [Condition; 7] = [
    ("registered_throughout", Want::True, "F2025L01055 s 6(a)"),
    (
        "mainly_supermarket_or_convenience_range",
        Want::True,
        "F2025L01055 s 6(b)",
    ),
    (
        "converted_taxable_sales_under_5_percent",
        Want::True,
        "F2025L01055 s 6(c)",
    ),
    (
        "gst_turnover",
        Want::AtMostTwoMillion,
        "F2025L01055 s 6(d): GST turnover does not exceed $2 million",
    ),
    (
        "adequate_point_of_sale_equipment",
        Want::True,
        "F2025L01055 s 6(e)",
    ),
    ("mainly_sells_fuel", Want::False, "F2025L01055 s 6(f)"),
    (
        "section_7_3_goods_left_out",
        Want::True,
        "F2025L01055 s 7(3): trading_stock_purchases, gst_free_sales and total_sales leave out goods not held for sale or exchange, goods outside the range commonly sold at supermarkets or convenience stores, goods held in substantially greater quantity or variety and alcoholic beverages",
    ),
];

const RESTAURANT: [Condition; 3] = [
    ("registered_throughout", Want::True, "F2026L01275 s 6(a)"),
    (
        "restaurant_cafe_or_catering",
        Want::True,
        "F2026L01275 s 6(b)",
    ),
    (
        "gst_turnover",
        Want::AtMostTwoMillion,
        "F2026L01275 s 6(c) and GST Act s 123-7(2): GST turnover does not exceed $2 million",
    ),
];

impl Method {
    /// The day the method's determination commenced: the day after
    /// registration (27 September 2017, 8 September 2025, 23 September 2026).
    fn commences(self) -> &'static str {
        match self {
            Method::Supermarket => "2025-09-09",
            Method::Restaurant => "2026-09-24",
            _ => "2017-09-28",
        }
    }

    fn conditions(self) -> Vec<Condition> {
        match self {
            Method::Supermarket => SUPERMARKET.to_vec(),
            Method::Restaurant => RESTAURANT.to_vec(),
            Method::StockFive | Method::SnapFive => {
                let mut all = RETAIL_2017.to_vec();
                all.push(FIVE_PERCENT);
                all
            }
            Method::BusinessNorms => {
                let mut all = RETAIL_2017.to_vec();
                all.push(DEFINITION);
                all
            }
            _ => RETAIL_2017.to_vec(),
        }
    }
}

fn method(value: &Arg) -> Result<(&'static str, Method)> {
    match METHODS.iter().find(|(name, _)| Some(*name) == value.text()) {
        Some(found) => Ok(*found),
        None => refuse(
            "gst_food_retailer_sam.method_invalid",
            format!(
                "method must be one of {}; the determination for government entities selling food in prisons and detention institutions (F2025L00997) is not worked.",
                METHODS.map(|(name, _)| name).join(", ")
            ),
        ),
    }
}

pub fn gst_food_retailer_sam(args: &GstFoodRetailerSamArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let (name, chosen) = method(args.method)?;
    let period = period(
        args.tax_period_start,
        args.tax_period_end,
        &year,
        chosen.commences(),
    )?;
    eligible(args.eligibility, &chosen.conditions())?;
    let purchases = money(args.trading_stock_purchases)?;
    let given = Given {
        items: args.given,
        method: name,
        read: RefCell::new(Vec::new()),
    };
    let (steps, rates) = precise(|| methods::work(chosen, &given, &period, &purchases))?;
    derived_result(KIND, &year, steps, rates)
}
