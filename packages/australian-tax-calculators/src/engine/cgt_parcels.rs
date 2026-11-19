// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `cgt_parcels`: the capital gains and losses from disposing of shares or
//! units held in parcels bought at different times (CGT event A1, ITAA 1997
//! s 104-10), and the year's net capital gain or loss from them and the
//! entity's other capital gains and losses (s 102-5(1)). Each parcel is a
//! separate CGT asset; a sale is matched to parcels first in, first out or as
//! the taxpayer selects, which TD 33 para 4 and the ATO's 'Keeping records of
//! shares and units' (Boris's example) accept. The citations are to ITAA 1997
//! compilation 264 (21 May 2026), the text that applies to 2025-26 and 2026-27
//! (Act No. 49 of 2026 Schedule 1 items 82 to 84 apply its changes from the
//! income year that includes 1 July 2027), read on 19 November 2026.

use serde_json::{Map, Value, json};

use super::business_capital::{year_first, year_text};
use super::{
    Arg, Result, Step, cents, derived_result, int, money, precise, rate, refuse, scope, step,
};
use crate::dates;
use crate::decimal::Decimal;

mod input;
mod steps;
use input::{objects, read_disposal, read_parcel};
use steps::amount_steps;

const KIND: &str = "cgt_parcels";

/// The most parcels or disposals one call takes.
const MOST: usize = 1000;

/// A parcel acquired at or before 11.45 am on 21 September 1999 can have its
/// cost base indexed (ss 114-1 and 114-10), and one acquired before
/// 20 September 1985 is pre-CGT (s 104-10(5)(a)); neither is worked here.
const INDEXATION_LAST_DAY: &str = "1999-09-21";

/// The entities and their discount percentage as (numerator, denominator):
/// s 115-100(a) and (b); a company makes no discount capital gain (s 115-10).
const ENTITIES: [(&str, (&str, &str)); 4] = [
    ("individual", ("1", "2")),
    ("trust", ("1", "2")),
    ("complying_superannuation_entity", ("1", "3")),
    ("company", ("0", "1")),
];

/// One parcel: the units acquired together on one day and the cost base
/// elements for them, split as the reduced cost base needs them.
struct Parcel {
    id: String,
    security: String,
    acquired: i64,
    units: Decimal,
    /// The first, second, fourth and fifth elements (s 110-25), which the
    /// reduced cost base shares (s 110-55(2)).
    base: Decimal,
    /// The third element, costs of owning (s 110-25(4)), which it does not.
    owning: Decimal,
}

/// A parcel the taxpayer selects for a disposal, and how many of its units.
struct Pick {
    parcel: String,
    units: Decimal,
}

struct Disposal {
    n: usize,
    security: String,
    date: i64,
    units: Decimal,
    proceeds: Decimal,
    selling: Decimal,
    picks: Option<Vec<Pick>>,
}

/// The units of one parcel matched to one disposal, with their shares of the
/// parcel's elements and the disposal's proceeds and selling costs.
struct Piece {
    disposal: usize,
    parcel: String,
    security: String,
    acquired: i64,
    date: i64,
    units: Decimal,
    proceeds: Decimal,
    cost_base: Decimal,
    reduced_cost_base: Decimal,
    gain: Decimal,
    loss: Decimal,
    discount: bool,
    counted: bool,
}

fn conflict<T>(message: String) -> Result<T> {
    refuse("cgt_parcels.fact_conflict", message)
}

/// What is left of a parcel or a disposal as its units are matched.
struct Left {
    units: Decimal,
    amounts: [Decimal; 2],
}

impl Left {
    /// The share of what is left that `units` of it take, in cents, or all of
    /// it for the last units, so the shares add up to the whole
    /// (ss 112-30(1) and (1A) and 116-40(1): the part reasonably attributable).
    fn take(&mut self, units: &Decimal) -> Result<[Decimal; 2]> {
        let all = units.equals(&self.units)?;
        let mut taken = [int(0), int(0)];
        for (share, left) in taken.iter_mut().zip(self.amounts.iter_mut()) {
            *share = if all {
                left.clone()
            } else {
                cents(left.mul(units)?.div(&self.units)?)?
            };
            *left = left.sub(share)?;
        }
        self.units = self.units.sub(units)?;
        Ok(taken)
    }
}

/// The first day of the CGT event that leaves an asset acquired on `acquired`
/// held for at least 12 months before it, counting neither the day acquired
/// nor the day of the event (s 115-25(1); the ATO's 'CGT discount'). Where the
/// 12 months end in a February without the starting day, they end on its last
/// day, so the event can be on 1 March.
fn discount_from(acquired: i64) -> i64 {
    let start = acquired + 1;
    let end = dates::add_months(start, 12);
    if dates::to_civil(end).2 == dates::to_civil(start).2 {
        end
    } else {
        end + 1
    }
}

/// The context `match_disposals` works in.
struct Matching {
    company: bool,
    first: i64,
    last: i64,
}

impl Matching {
    fn piece(
        &self,
        parcel: &Parcel,
        disposal: &Disposal,
        units: Decimal,
        cost: [Decimal; 2],
        sale: [Decimal; 2],
    ) -> Result<Piece> {
        let [base, owning] = cost;
        let [proceeds, selling] = sale;
        let reduced_cost_base = base.add(&selling)?;
        let cost_base = reduced_cost_base.add(&owning)?;
        let gain = proceeds.sub(&cost_base)?;
        let loss = reduced_cost_base.sub(&proceeds)?;
        let zero = int(0);
        let counted = (self.first..=self.last).contains(&disposal.date);
        if counted && parcel.acquired <= dates::parse(INDEXATION_LAST_DAY).expect("a date") {
            return refuse(
                "cgt_parcels.indexation_or_pre_cgt",
                format!(
                    "parcel {} was acquired on {}, at or before 21 September 1999: its cost base can be indexed (ITAA 1997 ss 114-1 and 114-10), or it is a pre-CGT asset whose gain or loss is disregarded (s 104-10(5)(a)), and this worksheet does not work either.",
                    parcel.id,
                    dates::format(parcel.acquired)
                ),
            );
        }
        Ok(Piece {
            disposal: disposal.n,
            parcel: parcel.id.clone(),
            security: parcel.security.clone(),
            acquired: parcel.acquired,
            date: disposal.date,
            units,
            proceeds,
            gain: if gain.gt(&zero)? { gain } else { zero.clone() },
            loss: if loss.gt(&zero)? { loss } else { zero },
            cost_base,
            reduced_cost_base,
            discount: !self.company && disposal.date >= discount_from(parcel.acquired),
            counted,
        })
    }

    /// The parcels' units each disposal takes, in date order: the earliest
    /// acquired first, or those the taxpayer selected.
    fn match_disposals(
        &self,
        parcels: &[Parcel],
        disposals: &mut [Disposal],
    ) -> Result<(Vec<Piece>, Vec<Left>)> {
        let mut held: Vec<Left> = parcels
            .iter()
            .map(|p| Left {
                units: p.units.clone(),
                amounts: [p.base.clone(), p.owning.clone()],
            })
            .collect();
        let mut order: Vec<usize> = (0..parcels.len()).collect();
        order.sort_by_key(|&i| parcels[i].acquired);
        disposals.sort_by_key(|d| (d.date, d.n));
        let mut pieces = Vec::new();
        for disposal in disposals.iter() {
            let what = format!("disposal {}", disposal.n);
            let takes = match &disposal.picks {
                Some(picks) => selected(parcels, disposal, picks, &held, &what)?,
                None => first_in(parcels, disposal, &order, &held, &what)?,
            };
            let mut sale = Left {
                units: disposal.units.clone(),
                amounts: [disposal.proceeds.clone(), disposal.selling.clone()],
            };
            for (i, units) in takes {
                let cost = held[i].take(&units)?;
                let share = sale.take(&units)?;
                pieces.push(self.piece(&parcels[i], disposal, units, cost, share)?);
            }
        }
        Ok((pieces, held))
    }
}

/// The units of each parcel the taxpayer selected for a disposal.
fn selected(
    parcels: &[Parcel],
    disposal: &Disposal,
    picks: &[Pick],
    held: &[Left],
    what: &str,
) -> Result<Vec<(usize, Decimal)>> {
    let mut total = int(0);
    let mut takes = Vec::new();
    for pick in picks {
        let Some(i) = parcels.iter().position(|p| p.id == pick.parcel) else {
            return conflict(format!("{what}: no parcel is named {}.", pick.parcel));
        };
        let parcel = &parcels[i];
        if parcel.security != disposal.security {
            return conflict(format!(
                "{what}: parcel {} is of {}, not {}.",
                parcel.id, parcel.security, disposal.security
            ));
        }
        if parcel.acquired > disposal.date {
            return conflict(format!(
                "{what}: parcel {} was acquired after the disposal.",
                parcel.id
            ));
        }
        if pick.units.gt(&held[i].units)? {
            return refuse(
                "cgt_parcels.units_not_held",
                format!(
                    "{what}: parcel {} has {} units left, fewer than the {} selected.",
                    parcel.id, held[i].units, pick.units
                ),
            );
        }
        total = total.add(&pick.units)?;
        takes.push((i, pick.units.clone()));
    }
    if !total.equals(&disposal.units)? {
        return conflict(format!(
            "{what}: the selected parcels' units add up to {total}, not the {} disposed of.",
            disposal.units
        ));
    }
    Ok(takes)
}

/// The units the earliest acquired parcels of the security still held give a
/// disposal, the earlier listed first on the same day.
fn first_in(
    parcels: &[Parcel],
    disposal: &Disposal,
    order: &[usize],
    held: &[Left],
    what: &str,
) -> Result<Vec<(usize, Decimal)>> {
    let mut wanted = disposal.units.clone();
    let mut takes = Vec::new();
    for &i in order {
        let parcel = &parcels[i];
        if wanted.is_zero() || parcel.acquired > disposal.date {
            break;
        }
        if parcel.security != disposal.security || held[i].units.is_zero() {
            continue;
        }
        let units = if held[i].units.lt(&wanted)? {
            held[i].units.clone()
        } else {
            wanted.clone()
        };
        wanted = wanted.sub(&units)?;
        takes.push((i, units));
    }
    if !wanted.is_zero() {
        return refuse(
            "cgt_parcels.units_not_held",
            format!(
                "{what}: {} units of {} are disposed of, {wanted} more than the parcels acquired by then still hold.",
                disposal.units, disposal.security
            ),
        );
    }
    Ok(takes)
}
/// The income year a day falls in, such as `2025-26`.
fn income_year(day: i64) -> String {
    let (y, m, _) = dates::to_civil(day);
    year_text(if m < 7 { y - 1 } else { y })
}

fn piece_row(p: &Piece) -> Result<Value> {
    let discount = if p.discount {
        "discount capital gain: acquired at least 12 months before the event, not counting either day (ITAA 1997 s 115-25(1))"
    } else {
        "not a discount capital gain: a company's (s 115-10), or acquired less than 12 months before the event (s 115-25(1))"
    };
    Ok(json!({
        "disposal": p.disposal,
        "parcel": p.parcel,
        "security": p.security,
        "acquired": dates::format(p.acquired),
        "date": dates::format(p.date),
        "income_year": income_year(p.date),
        "counted": p.counted,
        "units": p.units.fixed_format(),
        "capital_proceeds": cents(p.proceeds.clone())?.to_string(),
        "cost_base": cents(p.cost_base.clone())?.to_string(),
        "reduced_cost_base": cents(p.reduced_cost_base.clone())?.to_string(),
        "capital_gain": cents(p.gain.clone())?.to_string(),
        "capital_loss": cents(p.loss.clone())?.to_string(),
        "discount": if p.gain.is_zero() { Value::Null } else { json!(discount) },
    }))
}

fn holding_rows(parcels: &[Parcel], held: &[Left]) -> Result<Vec<Value>> {
    let mut rows = Vec::new();
    for (parcel, left) in parcels.iter().zip(held) {
        if left.units.is_zero() {
            continue;
        }
        let [base, owning] = &left.amounts;
        rows.push(json!({
            "parcel": parcel.id,
            "security": parcel.security,
            "acquired": dates::format(parcel.acquired),
            "units": left.units.fixed_format(),
            "cost_base": cents(base.add(owning)?)?.to_string(),
            "reduced_cost_base": cents(base.clone())?.to_string(),
        }));
    }
    Ok(rows)
}

/// The facts as the command passes them; see the README for each.
pub struct CgtParcelsArgs<'a> {
    pub entity: &'a Arg,
    pub identification: &'a Arg,
    pub parcels: &'a Arg,
    pub disposals: &'a Arg,
    pub other_discount_capital_gains: &'a Arg,
    pub other_capital_gains: &'a Arg,
    pub other_capital_losses: &'a Arg,
    pub prior_net_capital_losses: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}
/// The entity's discount percentage as (numerator, denominator), whether it
/// is a company, and whether the taxpayer selects the parcels.
fn entity_and_identification(
    args: &CgtParcelsArgs,
) -> Result<((&'static str, &'static str), bool, bool)> {
    let Some((_, fraction)) = ENTITIES
        .iter()
        .find(|(name, _)| Some(*name) == args.entity.text())
    else {
        return refuse(
            "cgt_parcels.entity_invalid",
            "entity must be individual, trust, complying_superannuation_entity or company.",
        );
    };
    let selected = match args.identification.text() {
        Some("first_in_first_out") => false,
        Some("selected") => true,
        _ => {
            return refuse(
                "cgt_parcels.identification_invalid",
                "identification must be first_in_first_out or selected.",
            );
        }
    };
    Ok((*fraction, args.entity.text() == Some("company"), selected))
}

/// The parcels, each named once and acquired by the year's end.
fn parcels_of(args: &CgtParcelsArgs, year: &str, last: i64) -> Result<Vec<Parcel>> {
    let parcels = objects(args.parcels, "parcels")?
        .into_iter()
        .enumerate()
        .map(|(i, fields)| read_parcel(i + 1, fields))
        .collect::<Result<Vec<_>>>()?;
    for (i, parcel) in parcels.iter().enumerate() {
        if parcels[..i].iter().any(|p| p.id == parcel.id) {
            return conflict(format!(
                "parcel {}: the name {} is used twice.",
                i + 1,
                parcel.id
            ));
        }
        if parcel.acquired > last {
            return conflict(format!(
                "parcel {}: it was acquired after the {year} income year ended.",
                i + 1
            ));
        }
    }
    Ok(parcels)
}

/// The disposals, each contracted by the year's end.
fn disposals_of(
    args: &CgtParcelsArgs,
    selected: bool,
    year: &str,
    last: i64,
) -> Result<Vec<Disposal>> {
    let disposals = objects(args.disposals, "disposals")?
        .into_iter()
        .enumerate()
        .map(|(i, fields)| read_disposal(i + 1, fields, selected))
        .collect::<Result<Vec<_>>>()?;
    if let Some(late) = disposals.iter().find(|d| d.date > last) {
        return conflict(format!(
            "disposal {}: its contract is after the {year} income year ended (s 104-10(3)).",
            late.n
        ));
    }
    Ok(disposals)
}

pub fn cgt_parcels(args: &CgtParcelsArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let start = year_first(&year).expect("a supported income year");
    let first = dates::parse(&format!("{start}-07-01")).expect("a date");
    let last = dates::add_months(first, 12) - 1;
    let ((numerator, denominator), company, selected) = entity_and_identification(args)?;
    let others = [
        money(args.other_discount_capital_gains)?,
        money(args.other_capital_gains)?,
        money(args.other_capital_losses)?,
        money(args.prior_net_capital_losses)?,
    ];
    if company && !others[0].is_zero() {
        return conflict(
            "other_discount_capital_gains must be 0.00 for a company, which makes no discount capital gain (ITAA 1997 s 115-10).".into(),
        );
    }
    let parcels = parcels_of(args, &year, last)?;
    let mut disposals = disposals_of(args, selected, &year, last)?;
    let matching = Matching {
        company,
        first,
        last,
    };
    let (numerator, denominator) = (Decimal::parse(numerator)?, Decimal::parse(denominator)?);
    let (steps, pieces, held) = precise(|| {
        let (pieces, held) = matching.match_disposals(&parcels, &mut disposals)?;
        let steps = amount_steps(&pieces, &others, [&numerator, &denominator])?;
        Ok((steps, pieces, held))
    })?;
    let rates = vec![
        rate("discount_numerator", &numerator),
        rate("discount_denominator", &denominator),
    ];
    let mut out = derived_result(KIND, &year, steps, rates)?;
    out["matches"] = Value::Array(pieces.iter().map(piece_row).collect::<Result<_>>()?);
    out["holdings"] = Value::Array(holding_rows(&parcels, &held)?);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn from(acquired: &str) -> String {
        dates::format(discount_from(dates::parse(acquired).unwrap()))
    }

    /// The 12 months start the day after acquisition; from 29 February 2024
    /// they end on 28 February 2025, so the event can be on 1 March.
    #[test]
    fn the_12_months_end_on_the_last_day_of_a_short_february() {
        assert_eq!(from("2024-02-28"), "2025-03-01");
        assert_eq!(from("2024-02-29"), "2025-03-01");
        assert_eq!(from("2025-02-02"), "2026-02-03");
        assert_eq!(from("2025-12-31"), "2027-01-01");
    }
}
