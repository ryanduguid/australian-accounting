// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! A scrip for scrip roll-over in `cgt_cost_base_adjustments`: each original
//! parcel's cost base carried to its replacement shares or units (ITAA 1997
//! ss 124-780, 124-781 and 124-785), less the part attributable to any
//! ineligible proceeds, which makes a capital gain (s 124-790). Read on the
//! ATO's legal database on 26 November 2026; no change since 1 July 2018.

use super::events::{Event, Scrip};
use super::work::{Work, bases, max_nil};
use super::*;

const ORIGINAL: &str = "ITAA 1997 ss 124-780 (shares) or 124-781 (trust units) and 124-785(1): the capital gain on the original shares or units is disregarded; with ineligible proceeds, the part of the cost base reasonably attributable to them, in proportion to their share of the capital proceeds (s 124-790(2) example), makes a capital gain (s 124-790(1))";
const REPLACEMENT: &str = "ITAA 1997 s 124-785(2) to (4): the replacement shares or units take the original parcel's cost base and reduced cost base less the part attributable to ineligible proceeds; acquired at the exchange but, for the discount, when the original parcel was (s 115-30(1) item 2)";

impl Work {
    /// Each parcel of the security held by the exchange is replaced.
    pub(super) fn exchange(&mut self, event: &Event, terms: &Scrip) -> Result<()> {
        for i in self.affected(event, event.day)? {
            self.replace(event, i, terms)?;
        }
        Ok(())
    }

    fn replace(&mut self, event: &Event, i: usize, terms: &Scrip) -> Result<()> {
        let before = bases(&self.parcels[i])?;
        let p = &self.parcels[i];
        let units = terms.ratio.mul(&p.units)?;
        let value = cents(units.mul(&terms.value)?)?;
        let cash = cents(p.units.mul(&terms.cash)?)?;
        let proceeds = value.add(&cash)?;
        if !proceeds.gt(&before[0])? {
            return refuse(
                "cgt_cost_base.scrip_no_gain",
                format!(
                    "event {}: parcel {}'s capital proceeds, {proceeds}, are not above its cost base, {}, so apart from the roll-over it would make no capital gain and there is no roll-over (ITAA 1997 ss 124-780(3)(b) and 124-781(3)(b)); work the disposal in cgt-parcels.",
                    event.n, p.id, before[0]
                ),
            );
        }
        let part = |amount: &Decimal| -> Result<Decimal> {
            Ok(cents(amount.mul(&cash)?.div(&proceeds)?)?)
        };
        let (base_part, owning_part) = (part(&p.base)?, part(&p.owning)?);
        let gain = max_nil(cash.sub(&base_part.add(&owning_part)?)?)?;
        let replacement = Parcel {
            id: format!("{} from {}", terms.new_security, p.id),
            security: terms.new_security.clone(),
            acquired: p.acquired,
            units,
            base: p.base.sub(&base_part)?,
            owning: p.owning.sub(&owning_part)?,
        };
        let held = &mut self.parcels[i];
        held.units = int(0);
        held.base = int(0);
        held.owning = int(0);
        let amount = (!cash.is_zero()).then_some(cash);
        let recorded = self.row(
            event,
            i,
            "scrip_for_scrip",
            (amount, before, gain, ORIGINAL.into()),
        );
        recorded?;
        self.add_replacement(event, replacement)
    }

    fn add_replacement(&mut self, event: &Event, parcel: Parcel) -> Result<()> {
        if self.parcels.iter().any(|p| p.id == parcel.id) {
            return conflict(format!(
                "event {}: a parcel is already named {}.",
                event.n, parcel.id
            ));
        }
        self.parcels.push(parcel);
        let nil = [int(0), int(0)];
        let facts = (None, nil, int(0), REPLACEMENT.to_string());
        self.row(
            event,
            self.parcels.len() - 1,
            "scrip_for_scrip_replacement",
            facts,
        )
    }
}
