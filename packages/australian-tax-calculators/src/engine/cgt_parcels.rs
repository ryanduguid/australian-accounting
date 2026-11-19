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

/// The stated facts of one parcel or disposal, each read once.
struct Fields<'a> {
    what: String,
    fields: &'a Map<String, Value>,
    taken: Vec<&'static str>,
}

impl<'a> Fields<'a> {
    fn invalid<T>(&self, key: &str, expected: &str) -> Result<T> {
        refuse(
            "cgt_parcels.fact_invalid",
            format!("{}: {key} must be {expected}.", self.what),
        )
    }

    fn get(&mut self, key: &'static str) -> Option<&'a Value> {
        self.taken.push(key);
        self.fields.get(key)
    }

    fn required(&mut self, key: &'static str) -> Result<&'a Value> {
        let fields = self.fields;
        match fields.get(key) {
            Some(value) => {
                self.taken.push(key);
                Ok(value)
            }
            None => refuse(
                "cgt_parcels.fact_missing",
                format!("{}: {key} is required.", self.what),
            ),
        }
    }

    fn text(&mut self, key: &'static str) -> Result<String> {
        match self.required(key)?.as_str() {
            Some(text) if !text.trim().is_empty() => Ok(text.to_string()),
            _ => self.invalid(key, "a non-empty string"),
        }
    }

    fn date(&mut self, key: &'static str) -> Result<i64> {
        match self.required(key)?.as_str().and_then(dates::parse) {
            Some(day) => Ok(day),
            None => self.invalid(key, "a YYYY-MM-DD date from 1900 to 2100"),
        }
    }

    fn decimal(value: Option<&Value>) -> Option<Decimal> {
        value?.as_str().and_then(|text| Decimal::parse(text).ok())
    }

    /// An amount in AUD; nil when an optional one is absent.
    fn amount(&mut self, key: &'static str, optional: bool) -> Result<Decimal> {
        let value = if optional {
            match self.get(key) {
                None => return Ok(int(0)),
                given => given,
            }
        } else {
            Some(self.required(key)?)
        };
        match Self::decimal(value).map(|v| money(&Arg::Decimal(v))) {
            Some(Ok(amount)) => Ok(amount),
            _ => self.invalid(
                key,
                "a finite non-negative decimal string, at most 2dp and 1e12",
            ),
        }
    }

    /// A number of shares or units: above nil, at most 6dp and 1e12.
    fn units(&mut self, key: &'static str) -> Result<Decimal> {
        let value = Self::decimal(Some(self.required(key)?));
        match value {
            Some(v) if v.is_finite() && v.exponent() >= -6 => {
                if v.gt(&int(0))? && !v.gt(&Decimal::parse("1e12")?)? {
                    return Ok(v);
                }
                self.invalid(key, "a decimal string above 0, at most 6dp and 1e12")
            }
            _ => self.invalid(key, "a decimal string above 0, at most 6dp and 1e12"),
        }
    }

    fn none_left(&self) -> Result<()> {
        match self
            .fields
            .keys()
            .find(|key| !self.taken.contains(&key.as_str()))
        {
            Some(key) => refuse(
                "input.field_unknown",
                format!(
                    "{}: {key} is not a fact it takes beside its other facts.",
                    self.what
                ),
            ),
            None => Ok(()),
        }
    }
}

fn list_invalid<T>(name: &str) -> Result<T> {
    refuse(
        "cgt_parcels.list_invalid",
        format!("{name} must be a list of 1 to {MOST} objects."),
    )
}

/// The objects of a non-empty list of at most [`MOST`].
fn objects<'a>(value: &'a Arg, name: &str) -> Result<Vec<&'a Map<String, Value>>> {
    let Arg::Json(Value::Array(list)) = value else {
        return list_invalid(name);
    };
    if list.is_empty() || list.len() > MOST {
        return list_invalid(name);
    }
    list.iter()
        .map(|v| match v.as_object() {
            Some(fields) => Ok(fields),
            None => list_invalid(name),
        })
        .collect()
}

fn read_parcel(n: usize, fields: &Map<String, Value>) -> Result<Parcel> {
    let mut f = Fields {
        what: format!("parcel {n}"),
        fields,
        taken: Vec::new(),
    };
    let id = f.text("parcel")?;
    let security = f.text("security")?;
    let acquired = f.date("acquired")?;
    let units = f.units("units")?;
    let mut base = f.amount("acquisition_cost", false)?;
    base = base.add(&f.amount("incidental_costs", false)?)?;
    let owning = f.amount("ownership_costs", true)?;
    base = base.add(&f.amount("capital_expenditure", true)?)?;
    base = base.add(&f.amount("title_costs", true)?)?;
    f.none_left()?;
    Ok(Parcel {
        id,
        security,
        acquired,
        units,
        base,
        owning,
    })
}

fn read_picks(f: &mut Fields) -> Result<Vec<Pick>> {
    let what = f.what.clone();
    let bad = || {
        refuse(
            "cgt_parcels.fact_invalid",
            format!(
                "{what}: parcels must be a non-empty list of objects, each with parcel and units."
            ),
        )
    };
    let Value::Array(list) = f.required("parcels")? else {
        return bad();
    };
    if list.is_empty() {
        return bad();
    }
    let mut picks = Vec::new();
    for (i, value) in list.iter().enumerate() {
        let Some(fields) = value.as_object() else {
            return bad();
        };
        let mut pick = Fields {
            what: format!("{what}, selected parcel {}", i + 1),
            fields,
            taken: Vec::new(),
        };
        let parcel = pick.text("parcel")?;
        let units = pick.units("units")?;
        pick.none_left()?;
        if picks.iter().any(|p: &Pick| p.parcel == parcel) {
            return conflict(format!("{what}: parcel {parcel} is selected twice."));
        }
        picks.push(Pick { parcel, units });
    }
    Ok(picks)
}

fn read_disposal(n: usize, fields: &Map<String, Value>, selected: bool) -> Result<Disposal> {
    let mut f = Fields {
        what: format!("disposal {n}"),
        fields,
        taken: Vec::new(),
    };
    let security = f.text("security")?;
    let date = f.date("date")?;
    let units = f.units("units")?;
    let proceeds = f.amount("capital_proceeds", false)?;
    let selling = f.amount("incidental_costs", false)?;
    let picks = if selected {
        Some(read_picks(&mut f)?)
    } else {
        None
    };
    f.none_left()?;
    Ok(Disposal {
        n,
        security,
        date,
        units,
        proceeds,
        selling,
        picks,
    })
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

const PARCEL_DISCOUNT_GAINS: &str = "ITAA 1997 s 104-10(4), ss 115-5 to 115-25: the capital gains from disposals in the year of parcels acquired at least 12 months before, the day acquired and the day of the event not counted, by an entity that can make a discount capital gain";
const PARCEL_OTHER_GAINS: &str = "ITAA 1997 s 104-10(4) and s 115-25(1): the capital gains from disposals in the year that are not discount capital gains";
const PARCEL_LOSSES: &str = "ITAA 1997 s 104-10(4) and s 110-55(2): the capital losses from disposals in the year, where the capital proceeds are less than the reduced cost base, which leaves out costs of owning";
const STEP_1: &str = "ITAA 1997 s 102-5(1) step 1 and its note 1: the year's capital losses reduce the capital gains in the order chosen, here the gains that are not discount capital gains first, which leaves the smallest net capital gain";
const STEP_2: &str = "ITAA 1997 s 102-5(1) step 2 and its note 2, and s 102-15: net capital losses of earlier years reduce what step 1 leaves, here the gains that are not discount capital gains first";
const STEP_3: &str = "ITAA 1997 s 102-5(1) step 3 and s 115-100: each discount capital gain left after step 2 is reduced by the discount percentage, 50% for an individual or a trust and 33 1/3% for a complying superannuation entity";
const STEP_5: &str = "ITAA 1997 s 102-5(1) step 5: the capital gains left after step 3 (step 4's small business concessions are not applied here)";
const NET_LOSS: &str =
    "ITAA 1997 s 102-10(1): the year's capital losses less its capital gains, where more than nil";
const CARRIED: &str = "ITAA 1997 ss 102-10(1) and 102-15: the year's net capital loss and the earlier years' net capital losses not applied, carried forward";

/// A sum over the pieces, written out term by term.
fn total(values: &[&Decimal]) -> Result<(String, Decimal)> {
    let mut sum = int(0);
    for v in values {
        sum = sum.add(v)?;
    }
    let terms: Vec<String> = values.iter().map(|v| v.to_string()).collect();
    let formula = if terms.is_empty() {
        "0".to_string()
    } else {
        terms.join(" + ")
    };
    Ok((formula, sum))
}

/// The other gains and losses the call states: (discount gains, other gains,
/// capital losses, earlier net capital losses).
type Others = [Decimal; 4];

fn amount_steps(pieces: &[Piece], others: &Others, fraction: [&Decimal; 2]) -> Result<Vec<Step>> {
    let counted: Vec<&Piece> = pieces.iter().filter(|p| p.counted).collect();
    let gains = |discount: bool| -> Vec<&Decimal> {
        counted
            .iter()
            .filter(|p| p.discount == discount && !p.gain.is_zero())
            .map(|p| &p.gain)
            .collect()
    };
    let losses: Vec<&Decimal> = counted
        .iter()
        .filter(|p| !p.loss.is_zero())
        .map(|p| &p.loss)
        .collect();
    let [other_discount, other_gains_in, other_losses, prior] = others;
    let (fd, pdg) = total(&gains(true))?;
    let (fo, pog) = total(&gains(false))?;
    let (fl, pl) = total(&losses)?;
    let dg = pdg.add(other_discount)?;
    let og = pog.add(other_gains_in)?;
    let h = dg.add(&og)?;
    let cl = pl.add(other_losses)?;
    let lo = min(&og, &cl)?;
    let ld = min(&dg, &cl.sub(&lo)?)?;
    let po = min(&og.sub(&lo)?, prior)?;
    let pd = min(&dg.sub(&ld)?, &prior.sub(&po)?)?;
    let [numerator, denominator] = fraction;
    let left = dg.sub(&ld)?.sub(&pd)?;
    let discount = left.mul(numerator)?.div(denominator)?;
    let net = og
        .sub(&lo)?
        .sub(&po)?
        .add(&dg)?
        .sub(&ld)?
        .sub(&pd)?
        .sub(&discount)?;
    let net_loss = cl.sub(&lo)?.sub(&ld)?;
    let carried = net_loss.add(prior)?.sub(&po)?.sub(&pd)?;
    Ok(vec![
        step(
            "parcel_discount_gains",
            PARCEL_DISCOUNT_GAINS,
            &fd,
            &[],
            pdg.clone(),
        ),
        step(
            "parcel_other_gains",
            PARCEL_OTHER_GAINS,
            &fo,
            &[],
            pog.clone(),
        ),
        step("parcel_losses", PARCEL_LOSSES, &fl, &[], pl.clone()),
        step(
            "discount_gains",
            PARCEL_DISCOUNT_GAINS,
            "parcel_discount_gains + other_discount_capital_gains",
            &[&pdg, other_discount],
            dg.clone(),
        ),
        step(
            "other_gains",
            PARCEL_OTHER_GAINS,
            "parcel_other_gains + other_capital_gains",
            &[&pog, other_gains_in],
            og.clone(),
        ),
        step(
            "total_capital_gains",
            "ITAA 1997 s 102-5(1) step 1: the capital gains made during the year, before losses and the discount (question 18 label H)",
            "discount_gains + other_gains",
            &[&dg, &og],
            h,
        ),
        step(
            "capital_losses",
            PARCEL_LOSSES,
            "parcel_losses + other_capital_losses",
            &[&pl, other_losses],
            cl.clone(),
        ),
        step(
            "losses_applied_to_other_gains",
            STEP_1,
            "min(other_gains, capital_losses)",
            &[&og, &cl],
            lo.clone(),
        ),
        step(
            "losses_applied_to_discount_gains",
            STEP_1,
            "min(discount_gains, capital_losses - losses_applied_to_other_gains)",
            &[&dg, &cl, &lo],
            ld.clone(),
        ),
        step(
            "prior_losses_applied_to_other_gains",
            STEP_2,
            "min(other_gains - losses_applied_to_other_gains, prior_net_capital_losses)",
            &[&og, &lo, prior],
            po.clone(),
        ),
        step(
            "prior_losses_applied_to_discount_gains",
            STEP_2,
            "min(discount_gains - losses_applied_to_discount_gains, prior_net_capital_losses - prior_losses_applied_to_other_gains)",
            &[&dg, &ld, prior, &po],
            pd.clone(),
        ),
        step(
            "discount",
            STEP_3,
            "(discount_gains - losses_applied_to_discount_gains - prior_losses_applied_to_discount_gains) x discount_numerator / discount_denominator",
            &[&dg, &ld, &pd, numerator, denominator],
            discount.clone(),
        ),
        step(
            "net_capital_gain",
            STEP_5,
            "other_gains - losses_applied_to_other_gains - prior_losses_applied_to_other_gains + discount_gains - losses_applied_to_discount_gains - prior_losses_applied_to_discount_gains - discount",
            &[&og, &lo, &po, &dg, &ld, &pd, &discount],
            net,
        ),
        step(
            "net_capital_loss",
            NET_LOSS,
            "capital_losses - losses_applied_to_other_gains - losses_applied_to_discount_gains",
            &[&cl, &lo, &ld],
            net_loss.clone(),
        ),
        step(
            "net_capital_losses_carried_forward",
            CARRIED,
            "net_capital_loss + prior_net_capital_losses - prior_losses_applied_to_other_gains - prior_losses_applied_to_discount_gains",
            &[&net_loss, prior, &po, &pd],
            carried,
        ),
    ])
}

fn min(a: &Decimal, b: &Decimal) -> Result<Decimal> {
    Ok(if b.lt(a)? { b.clone() } else { a.clone() })
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

pub fn cgt_parcels(args: &CgtParcelsArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let start = year_first(&year).expect("a supported income year");
    let first = dates::parse(&format!("{start}-07-01")).expect("a date");
    let last = dates::add_months(first, 12) - 1;
    let Some((_, (numerator, denominator))) = ENTITIES
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
    let others = [
        money(args.other_discount_capital_gains)?,
        money(args.other_capital_gains)?,
        money(args.other_capital_losses)?,
        money(args.prior_net_capital_losses)?,
    ];
    let company = args.entity.text() == Some("company");
    if company && !others[0].is_zero() {
        return conflict(
            "other_discount_capital_gains must be 0.00 for a company, which makes no discount capital gain (ITAA 1997 s 115-10).".into(),
        );
    }
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
    let mut disposals = objects(args.disposals, "disposals")?
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
