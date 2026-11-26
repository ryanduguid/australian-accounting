// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! How each event of `cgt_cost_base_adjustments` changes the parcels it
//! touches, one row per event and parcel.

use super::demergers::split;
use super::events::{Change, Event};
use super::*;

/// Shares and units acquired before this day are pre-CGT.
const PRE_CGT_FROM: &str = "1985-09-20";

const G1: &str = "CGT event G1, ITAA 1997 s 104-135: a non-assessable part above the cost base is a capital gain and both bases become nil (s 104-135(3)); otherwise both are reduced by it (s 104-135(4))";
const E4: &str = "CGT event E4, ITAA 1997 s 104-70(4) to (6): the year's non-assessable part above the cost base is a capital gain and both bases become nil; otherwise the cost base is reduced by it and the reduced cost base also by the tax-free amounts s 104-71(3) leaves out";
const AMIT_EXCESS: &str = "ITAA 1997 s 104-107B(2): an excess AMIT cost base net amount reduces both bases, to nil where it is above the cost base, and the amount above is a capital gain (CGT event E10, s 104-107A(3))";
const AMIT_SHORTFALL: &str =
    "ITAA 1997 s 104-107B(3): a shortfall AMIT cost base net amount increases both bases";
pub(super) const NEW_INTEREST: &str = "ITAA 1997 s 125-80(2) (or s 125-85 without roll-over): the new shares' share of the post-CGT head shares' cost bases; acquired on the implementation date (s 109-5) but, for the discount, when the corresponding head shares were (s 115-30(1) item 2)";

/// One event's effect on one parcel.
pub(super) struct Row {
    pub(super) event: usize,
    pub(super) change: &'static str,
    pub(super) authority: String,
    pub(super) parcel: usize,
    pub(super) day: i64,
    pub(super) amount: Option<Decimal>,
    /// The cost base and reduced cost base before and after.
    pub(super) before: [Decimal; 2],
    pub(super) after: [Decimal; 2],
    pub(super) gain: Decimal,
    pub(super) discount: bool,
    pub(super) counted: bool,
}

pub(super) struct Work {
    pub(super) parcels: Vec<Parcel>,
    pub(super) rows: Vec<Row>,
    pub(super) company: bool,
    pub(super) first: i64,
    pub(super) last: i64,
    pub(super) unindexed: bool,
    /// Each parcel given a year's E4 or AMIT amount: its index, the income
    /// year's first day and whether the amount was an AMIT's.
    pub(super) trust_years: Vec<(usize, i64, bool)>,
}

/// A parcel's cost base and reduced cost base (s 110-55(2) leaves out costs
/// of owning).
pub(super) fn bases(p: &Parcel) -> Result<[Decimal; 2]> {
    Ok([p.base.add(&p.owning)?, p.base.clone()])
}

pub(super) fn max_nil(v: Decimal) -> Result<Decimal> {
    Ok(if v.gt(&int(0))? { v } else { int(0) })
}

impl Work {
    /// The parcels an event touches: those named, or every parcel of the
    /// security acquired by `cutoff`.
    pub(super) fn affected(&mut self, event: &Event, cutoff: i64) -> Result<Vec<usize>> {
        let what = format!("event {}", event.n);
        let mut found = Vec::new();
        for (i, p) in self.parcels.iter().enumerate() {
            let named = match &event.parcels {
                Some(names) => names.contains(&p.id),
                None => p.acquired <= cutoff,
            };
            if named && p.security == event.security && !p.units.is_zero() {
                found.push(i);
            }
        }
        let named = event.parcels.as_ref().map_or(0, Vec::len);
        if found.is_empty() || (named > 0 && found.len() != named) {
            return conflict(format!(
                "{what}: it must name parcels of {} held by then, or there must be some.",
                event.security
            ));
        }
        self.bought_in_year(event)?;
        for &i in &found {
            self.check_age(&what, i, event.day)?;
        }
        Ok(found)
    }

    /// A year's E4 or AMIT amount without `parcels` is for every parcel held
    /// by then, which holds only when none was acquired in that year: one
    /// acquired after the trustee's payment took no part (s 104-70(1)).
    fn bought_in_year(&self, event: &Event) -> Result<()> {
        let (None, Some(start)) = (&event.parcels, event.year) else {
            return Ok(());
        };
        let late = self
            .parcels
            .iter()
            .find(|p| p.security == event.security && (start..=event.day).contains(&p.acquired));
        match late {
            Some(p) => conflict(format!(
                "event {}: parcel {} was acquired in the income year, so the event must name the parcels the amount was paid for in parcels.",
                event.n, p.id
            )),
            None => Ok(()),
        }
    }

    fn check_age(&mut self, what: &str, i: usize, day: i64) -> Result<()> {
        let p = &self.parcels[i];
        if p.acquired > day {
            return conflict(format!("{what}: parcel {} was acquired after it.", p.id));
        }
        if p.acquired < dates::parse(PRE_CGT_FROM).expect("a date") {
            return refuse(
                "cgt_cost_base.pre_cgt",
                format!(
                    "{what}: parcel {} was acquired before 20 September 1985; a pre-CGT share or unit's gain is disregarded (ITAA 1997 ss 104-135(5), 104-70(7), 104-107A(4)) a demerger treats it under s 125-80(4) to (7), which this command does not work, and a scrip for scrip roll-over needs an interest acquired on or after that day (ss 124-780(3)(a) and 124-781(3)(a)).",
                    p.id
                ),
            );
        }
        if p.acquired <= dates::parse(INDEXATION_LAST_DAY).expect("a date") {
            if self.company {
                return refuse(
                    "cgt_cost_base.company_indexation",
                    format!(
                        "{what}: parcel {} was acquired at or before 21 September 1999, so a company's cost base is indexed (ITAA 1997 ss 110-36(1) and 114-1), which this command does not work.",
                        p.id
                    ),
                );
            }
            self.unindexed = true;
        }
        Ok(())
    }

    /// Cuts the cost base and the reduced cost base of parcel `i`; the
    /// capital gain where the cost base cut is above the cost base.
    fn reduce(&mut self, i: usize, cuts: [&Decimal; 2]) -> Result<Decimal> {
        let [cost_base, reduced] = bases(&self.parcels[i])?;
        let p = &mut self.parcels[i];
        if cuts[0].gt(&cost_base)? {
            p.base = int(0);
            p.owning = int(0);
            return Ok(cuts[0].sub(&cost_base)?);
        }
        if cuts[1].gt(&reduced)? {
            return refuse(
                "cgt_cost_base.below_reduced_cost_base",
                format!(
                    "parcel {}: the reduced cost base, {reduced}, would be cut by {}, more than it is, which this command does not work.",
                    p.id, cuts[1]
                ),
            );
        }
        p.base = p.base.sub(cuts[1])?;
        p.owning = p.owning.sub(&cuts[0].sub(cuts[1])?)?;
        Ok(int(0))
    }

    pub(super) fn row(
        &mut self,
        event: &Event,
        i: usize,
        change: &'static str,
        facts: RowFacts,
    ) -> Result<()> {
        let p = &self.parcels[i];
        let (amount, before, gain, authority) = facts;
        self.rows.push(Row {
            event: event.n,
            change,
            authority,
            parcel: i,
            day: event.day,
            amount,
            before,
            after: bases(p)?,
            discount: !self.company && event.day >= discount_from(p.acquired),
            counted: (self.first..=self.last).contains(&event.day),
            gain,
        });
        Ok(())
    }
}

/// A row's amount, bases before, capital gain and authority.
pub(super) type RowFacts = (Option<Decimal>, [Decimal; 2], Decimal, String);

impl Work {
    pub(super) fn apply(&mut self, event: &Event) -> Result<()> {
        match &event.change {
            Change::ReturnOfCapital { per_unit } => {
                for i in self.affected(event, event.day)? {
                    let paid = cents(self.parcels[i].units.mul(per_unit)?)?;
                    let before = bases(&self.parcels[i])?;
                    let gain = self.reduce(i, [&paid, &paid])?;
                    let facts = (Some(paid), before, gain, G1.into());
                    self.row(event, i, "return_of_capital", facts)?;
                }
                Ok(())
            }
            Change::TaxDeferred { amount, tax_free } => {
                self.spread(event, [amount, tax_free], "tax_deferred", E4)
            }
            Change::Amit {
                amount,
                excess: true,
            } => self.spread(event, [amount, &int(0)], "amit_excess", AMIT_EXCESS),
            Change::Amit { amount, .. } => {
                for (i, share) in self.shares(event, amount)? {
                    let before = bases(&self.parcels[i])?;
                    self.parcels[i].base = self.parcels[i].base.add(&share)?;
                    let facts = (Some(share), before, int(0), AMIT_SHORTFALL.into());
                    self.row(event, i, "amit_shortfall", facts)?;
                }
                Ok(())
            }
            Change::Demerger {
                demerger,
                new_security,
                rollover,
            } => self.demerge(event, demerger, new_security, *rollover),
            Change::ScripForScrip(terms) => self.exchange(event, terms),
        }
    }

    /// Each affected parcel's share of a year's amount, by units.
    fn shares(&mut self, event: &Event, amount: &Decimal) -> Result<Vec<(usize, Decimal)>> {
        let found = self.affected(event, event.day)?;
        self.once_a_year(event, &found)?;
        let weights: Vec<&Decimal> = found.iter().map(|&i| &self.parcels[i].units).collect();
        Ok(found
            .iter()
            .copied()
            .zip(split(amount, &weights)?)
            .collect())
    }

    /// An AMIT unit has one AMIT cost base net amount a year (s 104-107C) and
    /// no CGT event E4 (s 104-70(1A)).
    fn once_a_year(&mut self, event: &Event, found: &[usize]) -> Result<()> {
        let amit = matches!(event.change, Change::Amit { .. });
        let year = event.year.expect("a year's amount");
        for &i in found {
            let clash = self
                .trust_years
                .iter()
                .any(|&(j, y, a)| j == i && y == year && (a || amit));
            if clash {
                return conflict(format!(
                    "event {}: parcel {} already has an amount for that income year, and an AMIT unit has one AMIT cost base net amount a year (ITAA 1997 s 104-107C) and no CGT event E4 (s 104-70(1A)).",
                    event.n, self.parcels[i].id
                ));
            }
        }
        self.trust_years
            .extend(found.iter().map(|&i| (i, year, amit)));
        Ok(())
    }

    /// A year's E4 or AMIT excess amount, with any tax-free amounts that cut
    /// only the reduced cost base, spread over the parcels by units.
    fn spread(
        &mut self,
        event: &Event,
        [amount, tax_free]: [&Decimal; 2],
        change: &'static str,
        authority: &str,
    ) -> Result<()> {
        let shares = self.shares(event, amount)?;
        let units: Vec<Decimal> = shares
            .iter()
            .map(|(i, _)| self.parcels[*i].units.clone())
            .collect();
        let free = split(tax_free, &units.iter().collect::<Vec<_>>())?;
        for ((i, share), free) in shares.into_iter().zip(free) {
            let before = bases(&self.parcels[i])?;
            let gain = self.reduce(i, [&share, &share.add(&free)?])?;
            self.row(
                event,
                i,
                change,
                (Some(share), before, gain, authority.into()),
            )?;
        }
        Ok(())
    }
}
