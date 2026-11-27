// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The business norms of F2017L01274 s 5, with the rural convenience store
//! and pharmacy rules.

use super::*;

const NORMS: &str = "F2017L01274 s 5(1) table: the business norms percentages for GST-free trading sales and GST-free trading acquisitions";
const RURAL: &str = "F2017L01274 s 5(2): fuel and Australia Post agency amounts deducted, then 22.5% (converter) or 30% (reseller) of sales and 30% of acquisitions";
const PHARMACY_SALES: &str = "F2017L01274 s 5(3)(a): PBS prescription sales, plus 98% of non-PBS prescription sales and 47.5% of over-the-counter sales";
const PHARMACY_PURCHASES: &str = "F2017L01274 s 5(3)(b), as the ATO reads it ('Pharmacies that also sell food'): 0% of non-PBS prescription and 2% of over-the-counter trading stock purchases";

const TYPES: [&str; 8] = [
    "cake_shop",
    "continental_delicatessen",
    "convenience_store",
    "fresh_fish_shop",
    "health_food_shop",
    "hot_bread_shop",
    "rural_convenience_store",
    "pharmacy",
];

/// (type, sales norm, purchases norm) of F2017L01274 s 5(1) and (2).
const NORM_TABLE: [(&str, &str, &str); 9] = [
    ("cake_shop", "2", "95"),
    ("continental_delicatessen", "85", "90"),
    ("convenience_store converter", "22.5", "30"),
    ("convenience_store reseller", "30", "30"),
    ("fresh_fish_shop", "35", "98"),
    ("health_food_shop", "35", "35"),
    ("hot_bread_shop", "50", "75"),
    ("rural_convenience_store converter", "22.5", "30"),
    ("rural_convenience_store reseller", "30", "30"),
];

pub(super) fn business_norms(given: &Given, purchases: &Decimal) -> Result<Worked> {
    let kind = given.need("retailer_type")?;
    let Some(kind) = TYPES.into_iter().find(|t| Some(*t) == kind.text()) else {
        return refuse(
            "gst_food_retailer_sam.retailer_type_invalid",
            format!("retailer_type must be one of {}.", TYPES.join(", ")),
        );
    };
    if kind == "pharmacy" {
        return pharmacy(given, purchases);
    }
    let key = if kind.contains("convenience") {
        format!("{kind} {}", role(given, &["converter", "reseller"])?)
    } else {
        kind.to_string()
    };
    let sales = sales(given)?;
    if kind == "rural_convenience_store" {
        return rural(given, &key, &sales, purchases);
    }
    given.done()?;
    let (_, s, p) = NORM_TABLE.iter().find(|row| row.0 == key).expect("a norm");
    let names = |n| ("total_sales", n);
    let free_sales = percent(
        "gst_free_sales",
        NORMS,
        names("sales_norm_percent"),
        &sales,
        &d(s),
    );
    let free_sales = free_sales?;
    let free = percent(
        "gst_free_purchases",
        NORMS,
        ("trading_stock_purchases", "purchases_norm_percent"),
        purchases,
        &d(p),
    );
    let free = free?;
    let rates = vec![
        rate("sales_norm_percent", s),
        rate("purchases_norm_percent", p),
    ];
    Ok((pairs(Some((&sales, free_sales)), purchases, free)?, rates))
}

/// `a x n / 100`.
fn of(a: &Decimal, n: &str) -> Result<Decimal> {
    Ok(a.mul(&d(n))?.div(&d("100"))?)
}

/// The fuel and Australia Post agency amounts, at most the totals.
fn fuel_and_post(
    given: &Given,
    sales: &Decimal,
    purchases: &Decimal,
) -> Result<(Decimal, Decimal)> {
    let fuel_sales = money(given.need("fuel_and_post_sales")?)?;
    let fuel_purchases = money(given.need("fuel_and_post_purchases")?)?;
    given.done()?;
    if fuel_sales.gt(sales)? || fuel_purchases.gt(purchases)? {
        return exceeds(
            "fuel_and_post_sales and fuel_and_post_purchases cannot exceed total_sales and trading_stock_purchases",
        );
    }
    Ok((fuel_sales, fuel_purchases))
}

fn rural(given: &Given, key: &str, sales: &Decimal, purchases: &Decimal) -> Result<Worked> {
    let (fuel_sales, fuel_purchases) = fuel_and_post(given, sales, purchases)?;
    let (_, s, p) = NORM_TABLE.iter().find(|row| row.0 == key).expect("a norm");
    let free_sales = step(
        "gst_free_sales",
        RURAL,
        "(total_sales - fuel_and_post_sales) x sales_norm_percent / 100",
        &[sales, &fuel_sales, &d(s)],
        of(&sales.sub(&fuel_sales)?, s)?,
    );
    let free = step(
        "gst_free_purchases",
        RURAL,
        "(trading_stock_purchases - fuel_and_post_purchases) x purchases_norm_percent / 100",
        &[purchases, &fuel_purchases, &d(p)],
        of(&purchases.sub(&fuel_purchases)?, p)?,
    );
    let rates = vec![
        rate("sales_norm_percent", s),
        rate("purchases_norm_percent", p),
    ];
    Ok((pairs(Some((sales, free_sales)), purchases, free)?, rates))
}

/// A pharmacy's 3 sales and 2 purchase components, which must add up to the
/// totals: PBS, non-PBS and over-the-counter sales, then non-PBS and
/// over-the-counter purchases.
fn pharmacy_parts(given: &Given, sales: &Decimal, purchases: &Decimal) -> Result<[Decimal; 5]> {
    let mut parts = Vec::new();
    for key in [
        "pbs_sales",
        "non_pbs_sales",
        "otc_sales",
        "non_pbs_purchases",
        "otc_purchases",
    ] {
        parts.push(money(given.need(key)?)?);
    }
    given.done()?;
    let sold = parts[0].add(&parts[1])?.add(&parts[2])?;
    let bought = parts[3].add(&parts[4])?;
    if !sold.equals(sales)? || !bought.equals(purchases)? {
        return refuse(
            "gst_food_retailer_sam.components_unbalanced",
            "pbs_sales, non_pbs_sales and otc_sales must add up to total_sales, and non_pbs_purchases and otc_purchases to trading_stock_purchases: s 5(3)(b) gives no norm for any other purchase.",
        );
    }
    Ok(parts.try_into().expect("5 parts"))
}

fn pharmacy(given: &Given, purchases: &Decimal) -> Result<Worked> {
    let sales = sales(given)?;
    let [pbs, non_pbs, otc, non_pbs_p, otc_p] = pharmacy_parts(given, &sales, purchases)?;
    let value = pbs.add(&of(&non_pbs, "98")?)?.add(&of(&otc, "47.5")?)?;
    let free_sales = step(
        "gst_free_sales",
        PHARMACY_SALES,
        "pbs_sales + non_pbs_sales x non_pbs_sales_norm_percent / 100 + otc_sales x otc_sales_norm_percent / 100",
        &[&pbs, &non_pbs, &d("98"), &otc, &d("47.5")],
        value,
    );
    let free = step(
        "gst_free_purchases",
        PHARMACY_PURCHASES,
        "non_pbs_purchases x non_pbs_purchases_norm_percent / 100 + otc_purchases x otc_purchases_norm_percent / 100",
        &[&non_pbs_p, &d("0"), &otc_p, &d("2")],
        of(&otc_p, "2")?,
    );
    let rates = vec![
        rate("non_pbs_sales_norm_percent", "98"),
        rate("otc_sales_norm_percent", "47.5"),
        rate("non_pbs_purchases_norm_percent", "0"),
        rate("otc_purchases_norm_percent", "2"),
    ];
    Ok((pairs(Some((&sales, free_sales)), purchases, free)?, rates))
}
