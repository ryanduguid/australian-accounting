// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Each method's arithmetic, as steps a reader can check alone.

use serde_json::Value;

use super::super::stated::{amount, known_keys};
use super::super::{Arg, Result, Step, d, money, rate, refuse, step};
use super::Method;
use super::facts::{
    Given, Period, Sample, method_started, restaurant_plan, sample, started, window,
};
use crate::decimal::Decimal;

mod norms;

type Rates = Vec<(String, String)>;
type Worked = (Vec<Step>, Rates);

const EVERY: &str = "F2017L01274 s 6(3) (and s 7(4)(a)): actual GST-free trading acquisitions; GST-free sales as their share of trading stock acquisitions times total trading sales";
const PURCHASE_SAMPLE: &str = "F2017L01274 s 6(4) (and s 7(3)(a)): the 4-week sample period percentage of GST-free trading acquisitions";
const SALES_SAMPLE: &str =
    "F2017L01274 s 7(3)(b): the 2-week sample period percentage of GST-free trading sales";
const MARKUP: &str = "F2017L01274 s 6(6)(b) (and s 7(7)): each product line resold GST-free, its GST-free acquisitions with its mark-up";
const SUPERMARKET: &str = "F2025L01055 s 7(1), input tax credits steps 1 to 7";
const RESTAURANT: &str = "F2026L01275 s 7: the relevant 4-week sample period's proportion of GST-free trading stock acquisitions, then one-eleventh of the rest";
const REST: &str = "The total less the GST-free part";

pub(super) fn work(
    m: Method,
    given: &Given,
    period: &Period,
    purchases: &Decimal,
) -> Result<Worked> {
    match m {
        Method::BusinessNorms => norms::business_norms(given, purchases),
        Method::StockEvery | Method::SnapEvery => every(m, given, period, purchases),
        Method::StockSamples | Method::SnapSamples => samples(m, given, period, purchases),
        Method::StockFive | Method::SnapFive => five(m, given, period, purchases),
        Method::Supermarket => supermarket(given, purchases),
        Method::Restaurant => restaurant(given, period, purchases),
    }
}

fn exceeds<T>(what: &str) -> Result<T> {
    refuse(
        "gst_food_retailer_sam.amount_exceeds_total",
        format!("{what}."),
    )
}

fn nil<T>(what: &str) -> Result<T> {
    refuse(
        "gst_food_retailer_sam.total_nil",
        format!("{what} must be more than nil for this method."),
    )
}

fn role(given: &Given, allowed: &[&str]) -> Result<&'static str> {
    let value = given.need("stock_role")?;
    match ["converter", "reseller"]
        .into_iter()
        .find(|r| Some(*r) == value.text())
    {
        Some(found) if allowed.contains(&found) => Ok(found),
        _ => refuse(
            "gst_food_retailer_sam.stock_role_invalid",
            format!(
                "stock_role must be {} for method {} (F2017L01274 s 6(1) and s 8: the stock purchases methods are for resellers only).",
                allowed.join(" or "),
                given.method
            ),
        ),
    }
}

fn sales(given: &Given) -> Result<Decimal> {
    money(given.need("total_sales")?)
}

fn rest(id: &'static str, total: &str, part: &str, a: &Decimal, b: &Decimal) -> Result<Step> {
    Ok(step(
        id,
        REST,
        &format!("{total} - {part}"),
        &[a, b],
        a.sub(b)?,
    ))
}

/// `base x percent / 100`.
fn percent(
    id: &'static str,
    auth: &'static str,
    names: (&str, &str),
    base: &Decimal,
    pct: &Decimal,
) -> Result<Step> {
    let value = base.mul(pct)?.div(&d("100"))?;
    Ok(step(
        id,
        auth,
        &format!("{} x {} / 100", names.0, names.1),
        &[base, pct],
        value,
    ))
}

/// `base x part / whole`.
fn share(
    id: &'static str,
    auth: &'static str,
    names: [&str; 3],
    values: [&Decimal; 3],
) -> Result<Step> {
    let value = values[0].mul(values[1])?.div(values[2])?;
    let formula = format!("{} x {} / {}", names[0], names[1], names[2]);
    Ok(step(id, auth, &formula, &values, value))
}

/// The sales pair then the purchases pair, from the GST-free steps given.
fn pairs(
    sales: Option<(&Decimal, Step)>,
    purchases: &Decimal,
    gst_free: Step,
) -> Result<Vec<Step>> {
    let mut steps = Vec::new();
    if let Some((total, free)) = sales {
        let other = rest(
            "other_sales",
            "total_sales",
            "gst_free_sales",
            total,
            &free.value,
        );
        let other = other?;
        steps.extend([free, other]);
    }
    let other = rest(
        "other_purchases",
        "trading_stock_purchases",
        "gst_free_purchases",
        purchases,
        &gst_free.value,
    );
    let other = other?;
    steps.extend([gst_free, other]);
    Ok(steps)
}

/// Actual GST-free trading stock purchases, at most the total.
fn actual(given: &Given, purchases: &Decimal) -> Result<Step> {
    let free = money(given.need("gst_free_purchases")?)?;
    if free.gt(purchases)? {
        return exceeds("gst_free_purchases cannot exceed trading_stock_purchases");
    }
    Ok(step(
        "gst_free_purchases",
        EVERY,
        "gst_free_purchases",
        &[&free],
        free.clone(),
    ))
}

/// The 2017 sample window for the tax period's half, or the first 2 months
/// of trading.
fn half_window(given: &Given, period: &Period) -> Result<(i64, i64)> {
    let Some(july_half) = period.half() else {
        return refuse(
            "gst_food_retailer_sam.tax_period_spans_halves",
            "Each sample period percentage applies to tax periods in July to December or in January to June (F2017L01274 s 6(4) and s 7(3)); a tax period running across 31 December is not worked.",
        );
    };
    let started = started(given.may("trading_started"), period)?;
    window(period, july_half, started)
}

fn sales_from_sample(
    given: &Given,
    sales: &Decimal,
    win: (i64, i64),
    rates: &mut Rates,
) -> Result<Step> {
    let s = sample(given.need("sales_sample")?, "sales_sample", 14, win)?;
    rates.extend(sample_rates("sales_sample", &s));
    share(
        "gst_free_sales",
        SALES_SAMPLE,
        ["total_sales", "sales_sample_gst_free", "sales_sample_total"],
        [sales, &s.gst_free, &s.total],
    )
}

fn sample_rates(name: &str, s: &Sample) -> Rates {
    vec![
        rate(&format!("{name}_gst_free"), &s.gst_free),
        rate(&format!("{name}_total"), &s.total),
    ]
}

fn every(m: Method, given: &Given, period: &Period, purchases: &Decimal) -> Result<Worked> {
    let sales = sales(given)?;
    let free = actual(given, purchases)?;
    let mut rates = Rates::new();
    let free_sales = if m == Method::StockEvery {
        role(given, &["reseller"])?;
        given.done()?;
        if purchases.is_zero() {
            return nil("trading_stock_purchases");
        }
        share(
            "gst_free_sales",
            EVERY,
            [
                "total_sales",
                "gst_free_purchases",
                "trading_stock_purchases",
            ],
            [&sales, &free.value, purchases],
        )?
    } else {
        let win = half_window(given, period)?;
        let step = sales_from_sample(given, &sales, win, &mut rates)?;
        given.done()?;
        step
    };
    Ok((pairs(Some((&sales, free_sales)), purchases, free)?, rates))
}

fn sample_share(
    given: &Given,
    key: &'static str,
    purchases: &Decimal,
    win: (i64, i64),
    rates: &mut Rates,
) -> Result<(Step, Sample)> {
    let s = sample(given.need(key)?, key, 28, win)?;
    rates.extend(sample_rates(key, &s));
    let gst_free_key = format!("{key}_gst_free");
    let total_key = format!("{key}_total");
    let free = share(
        "gst_free_purchases",
        PURCHASE_SAMPLE,
        ["trading_stock_purchases", &gst_free_key, &total_key],
        [purchases, &s.gst_free, &s.total],
    );
    let free = free?;
    Ok((free, s))
}

fn purchases_from_sample(
    given: &Given,
    purchases: &Decimal,
    win: (i64, i64),
    rates: &mut Rates,
) -> Result<(Step, Sample)> {
    sample_share(given, "purchases_sample", purchases, win, rates)
}

fn samples(m: Method, given: &Given, period: &Period, purchases: &Decimal) -> Result<Worked> {
    let sales = sales(given)?;
    if m == Method::StockSamples {
        role(given, &["reseller"])?;
    }
    let win = half_window(given, period)?;
    let mut rates = Rates::new();
    let (free, s) = purchases_from_sample(given, purchases, win, &mut rates)?;
    let free_sales = if m == Method::StockSamples {
        share(
            "gst_free_sales",
            PURCHASE_SAMPLE,
            [
                "total_sales",
                "purchases_sample_gst_free",
                "purchases_sample_total",
            ],
            [&sales, &s.gst_free, &s.total],
        )?
    } else {
        sales_from_sample(given, &sales, win, &mut rates)?
    };
    given.done()?;
    Ok((pairs(Some((&sales, free_sales)), purchases, free)?, rates))
}

/// Each product line resold GST-free: its GST-free purchases and mark-up.
fn lines(given: &Given, rates: &mut Rates) -> Result<(Step, Decimal)> {
    let Arg::Json(Value::Array(items)) = given.need("product_lines")? else {
        return refuse(
            "input.not_object",
            "product_lines must be a list of 1 to 50 product lines.",
        );
    };
    if items.is_empty() || items.len() > 50 {
        return refuse(
            "input.count_invalid",
            "product_lines must hold 1 to 50 product lines.",
        );
    }
    let (mut parts, mut operands, mut total, mut bought) = (Vec::new(), Vec::new(), d("0"), d("0"));
    for (i, item) in items.iter().enumerate() {
        let name = format!("product line {}", i + 1);
        let Value::Object(fields) = item else {
            return refuse("input.not_object", format!("{name} must be an object."));
        };
        known_keys(
            fields,
            &name,
            "a product line",
            &["gst_free_purchases", "markup_percent"],
        )?;
        let (cost, markup) = (
            amount(fields, &name, "gst_free_purchases")?,
            amount(fields, &name, "markup_percent")?,
        );
        let (p, m) = (
            format!("line_{}_gst_free_purchases", i + 1),
            format!("line_{}_markup_percent", i + 1),
        );
        parts.push(format!("{p} x (100 + {m}) / 100"));
        total = total.add(&cost.mul(&d("100").add(&markup)?)?.div(&d("100"))?)?;
        bought = bought.add(&cost)?;
        rates.extend([rate(&p, &cost), rate(&m, &markup)]);
        operands.extend([cost, markup]);
    }
    let refs: Vec<&Decimal> = operands.iter().collect();
    Ok((
        step("gst_free_sales", MARKUP, &parts.join(" + "), &refs, total),
        bought,
    ))
}

fn five(m: Method, given: &Given, period: &Period, purchases: &Decimal) -> Result<Worked> {
    let sales = sales(given)?;
    if m == Method::StockFive {
        role(given, &["reseller"])?;
    }
    let mut rates = Rates::new();
    let sampled = m == Method::SnapFive && given.may("purchases_sample").is_some();
    let free = if sampled {
        let win = half_window(given, period)?;
        purchases_from_sample(given, purchases, win, &mut rates)?.0
    } else {
        actual(given, purchases)?
    };
    let (free_sales, bought) = lines(given, &mut rates)?;
    given.done()?;
    if !sampled && bought.gt(&free.value)? {
        return exceeds("the product lines' gst_free_purchases cannot exceed gst_free_purchases");
    }
    Ok((pairs(Some((&sales, free_sales)), purchases, free)?, rates))
}

fn credits(auth: &'static str, other: &Decimal) -> Result<Step> {
    Ok(step(
        "trading_stock_input_tax_credits",
        auth,
        "other_purchases / 11",
        &[other],
        other.div(&d("11"))?,
    ))
}

fn supermarket(given: &Given, purchases: &Decimal) -> Result<Worked> {
    let sales = sales(given)?;
    let free = money(given.need("gst_free_sales")?)?;
    given.done()?;
    if sales.is_zero() {
        return nil("total_sales");
    }
    if free.gt(&sales)? {
        return exceeds("gst_free_sales cannot exceed total_sales");
    }
    let free_sales = step(
        "gst_free_sales",
        SUPERMARKET,
        "gst_free_sales",
        &[&free],
        free.clone(),
    );
    let free_purchases = share(
        "gst_free_purchases",
        SUPERMARKET,
        ["trading_stock_purchases", "gst_free_sales", "total_sales"],
        [purchases, &free, &sales],
    );
    let free_purchases = free_purchases?;
    let mut steps = pairs(Some((&sales, free_sales)), purchases, free_purchases)?;
    let other = steps.last().expect("other_purchases").value.clone();
    steps.push(credits(SUPERMARKET, &other)?);
    Ok((steps, Rates::new()))
}

fn restaurant(given: &Given, period: &Period, purchases: &Decimal) -> Result<Worked> {
    let started = started(given.may("trading_started"), period)?;
    let method_started = method_started(given.may("method_started"), period)?;
    let Some(july_half) = period.half() else {
        return refuse(
            "gst_food_retailer_sam.tax_period_spans_halves",
            "F2026L01275 s 7 gives a sample period for a tax period wholly in July to December or wholly in January to June.",
        );
    };
    let (is_first, win) = restaurant_plan(period, july_half, started, method_started)?;
    let mut rates = Rates::new();
    let sample_key = if is_first {
        "purchases_sample"
    } else {
        if given.may("purchases_sample").is_some() {
            return refuse(
                "gst_food_retailer_sam.first_period_sample_required",
                "F2026L01275 s 7 applies the GST-free trading stock proportion worked out for the first tax period in the 6-month half to every tax period in that half; pass first_tax_period_sample instead of choosing a separate purchases_sample for a later tax period.",
            );
        }
        "first_tax_period_sample"
    };
    let (free, _) = sample_share(given, sample_key, purchases, win, &mut rates)?;
    given.done()?;
    let free = Step {
        authority: RESTAURANT,
        ..free
    };
    let mut steps = pairs(None, purchases, free)?;
    let other = steps.last().expect("other_purchases").value.clone();
    steps.push(credits(RESTAURANT, &other)?);
    Ok((steps, rates))
}
