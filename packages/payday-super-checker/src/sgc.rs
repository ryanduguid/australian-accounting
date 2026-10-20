//! SG-charge exposure estimates for late contributions.
//!
//! Components (SGAA s 16B(2)) modelled here:
//! - final SG shortfall (input: the unpaid individual SG amount)
//! - notional earnings component (s 19A): daily compounding at the GIC rate
//!   on the BASE shortfall, starting the day AFTER the last on-time day and
//!   running while the final shortfall exceeds nil
//! - administrative uplift (s 19B(1)): 60% of (shortfalls + notional
//!   earnings), reduced by reg 13C (clean history: -20 points) and reg 13D
//!   (voluntary disclosure within 30/60/120/120+ days: -40/-35/-30/-15 points)
//!
//! NOT modelled: choice loading (s 20A), the post-assessment late payment
//! penalty, GIC on an unpaid assessment. Estimates are indicative only.

use crate::Result;
use crate::csv_io::zero;
use crate::decimal::Decimal;
use crate::py::Date;
use crate::rates::GicTable;

/// reg 13D VDS-timing reduction points against the 60% start rate, with
/// (reg 13C) and without the extra 20-point clean-history reduction.
pub const UPLIFT_PCT: [(&str, [(&str, i64); 5]); 2] = [
    (
        "clean_history",
        [
            ("vds_within_30d", 0),
            ("vds_31_60d", 5),
            ("vds_61_120d", 10),
            ("vds_after_120d", 25),
            ("no_vds", 40),
        ],
    ),
    (
        "prior_history",
        [
            ("vds_within_30d", 20),
            ("vds_31_60d", 25),
            ("vds_61_120d", 30),
            ("vds_after_120d", 45),
            ("no_vds", 60),
        ],
    ),
];

/// Uplift by history, then by scenario.
pub type Uplift = Vec<(String, Vec<(String, Decimal)>)>;

/// NEC accrued over [due + 1 day, end] inclusive, compounding on the base
/// shortfall: each day's accrual is (shortfall + NEC so far) x daily GIC rate.
pub fn notional_earnings(
    shortfall: &Decimal,
    due: Date,
    end: Date,
    gic: &GicTable,
    allow_stale: bool,
) -> Result<Decimal> {
    if shortfall.lt(&zero())? {
        return Err(crate::Failure::Value("shortfall cannot be negative".into()));
    }
    let mut nec = zero();
    let mut d = due.add_days(1)?;
    while d <= end {
        let rate = gic.daily_rate(d, allow_stale)?;
        nec = nec.add(&shortfall.add(&nec)?.mul(&rate)?)?;
        d = d.add_days(1)?;
    }
    Ok(nec)
}

/// Administrative uplift under every reg 13C/13D scenario.
pub fn uplift_scenarios(final_shortfall: &Decimal, nec: &Decimal) -> Result<Uplift> {
    let base = final_shortfall.add(nec)?;
    let hundred = Decimal::from_int(100);
    let mut out = Vec::new();
    for (history, row) in UPLIFT_PCT {
        let mut scenarios = Vec::new();
        for (scenario, pct) in row {
            let value = base
                .mul(&Decimal::parse(&pct.to_string())?)?
                .div(&hundred)?;
            scenarios.push((scenario.to_string(), value));
        }
        out.push((history.to_string(), scenarios));
    }
    Ok(out)
}

pub fn scenario<'a>(uplift: &'a Uplift, history: &str, scenario: &str) -> &'a Decimal {
    let row = &uplift
        .iter()
        .find(|(h, _)| h == history)
        .expect("a history")
        .1;
    &row.iter()
        .find(|(s, _)| s == scenario)
        .expect("a scenario")
        .1
}

/// (low, high) total SGC estimate excluding choice loading.
pub fn exposure_range(final_shortfall: &Decimal, nec: &Decimal) -> Result<(Decimal, Decimal)> {
    let scenarios = uplift_scenarios(final_shortfall, nec)?;
    let low =
        final_shortfall
            .add(nec)?
            .add(scenario(&scenarios, "clean_history", "vds_within_30d"))?;
    let high = final_shortfall
        .add(nec)?
        .add(scenario(&scenarios, "prior_history", "no_vds"))?;
    Ok((low, high))
}
