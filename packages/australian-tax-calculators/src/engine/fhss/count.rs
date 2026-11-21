// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `fhss` counting: which contributions are concessional, the order they are
//! counted in (TAA 1953 Sch 1 s 138-30(2) and (3)) and the limits
//! (s 138-35(1)).

use std::collections::BTreeMap;

use super::input::{Piece, fy_label};
use crate::decimal::Decimal;
use crate::engine::{Result, d, refuse};

/// For each year with a deduction, the personal contributions left
/// non-concessional: the year's personal contributions less the deduction.
fn undeducted(
    pieces: &[Piece],
    deductions: &BTreeMap<i64, Decimal>,
) -> Result<BTreeMap<i64, Decimal>> {
    let mut by_year: BTreeMap<i64, Decimal> = BTreeMap::new();
    for p in pieces.iter().filter(|p| p.kind == "personal") {
        let total = by_year.entry(p.fy).or_insert_with(|| Decimal::from_int(0));
        *total = total.add(&p.amount)?;
    }
    let mut left = BTreeMap::new();
    for (fy, deducted) in deductions {
        let made = by_year
            .get(fy)
            .cloned()
            .unwrap_or_else(|| Decimal::from_int(0));
        if deducted.gt(&made)? {
            return refuse(
                "fhss.deductions_invalid",
                format!(
                    "The deduction for {} is more than the personal contributions given for that year.",
                    fy_label(*fy)
                ),
            );
        }
        left.insert(*fy, made.sub(deducted)?);
    }
    Ok(left)
}

/// Each year's deducted personal contributions are the latest of its
/// personal contributions, so the non-concessional are made first
/// (s 138-30(3)(b) and its example); a contribution the deduction reaches
/// part of is split, the non-concessional part first. `pieces` are in date
/// order.
pub(super) fn split_personal(
    pieces: Vec<Piece>,
    deductions: &BTreeMap<i64, Decimal>,
) -> Result<Vec<Piece>> {
    let mut left = undeducted(&pieces, deductions)?;
    let mut out = Vec::with_capacity(pieces.len());
    for mut p in pieces {
        if p.kind != "personal" {
            out.push(p);
            continue;
        }
        let Some(room) = left.get_mut(&p.fy) else {
            out.push(p);
            continue;
        };
        if p.amount.le(room)? {
            *room = room.sub(&p.amount)?;
            out.push(p);
        } else if room.is_zero() {
            p.concessional = true;
            out.push(p);
        } else {
            let deducted = p.amount.sub(room)?;
            let free = std::mem::replace(room, Decimal::from_int(0));
            out.push(Piece {
                amount: free,
                ..p.clone()
            });
            p.amount = deducted;
            p.concessional = true;
            out.push(p);
        }
    }
    Ok(out)
}

/// s 138-30(2) and (3): earliest first; a non-concessional contribution made
/// at the same time as a concessional one first; and in a year where you
/// personally made both kinds, the year's non-concessional contributions
/// first.
pub(super) fn order(pieces: &mut [Piece], deductions: &BTreeMap<i64, Decimal>) {
    let both: Vec<i64> = deductions
        .keys()
        .copied()
        .filter(|fy| {
            pieces
                .iter()
                .any(|p| p.fy == *fy && p.kind == "personal" && !p.concessional)
                && pieces
                    .iter()
                    .any(|p| p.fy == *fy && p.kind == "personal" && p.concessional)
        })
        .collect();
    pieces.sort_by_key(|p| {
        let year_first = both.contains(&p.fy) && !p.concessional;
        (p.fy, !year_first, p.day, p.concessional, p.n)
    });
}

/// s 138-35(1): at most $15,000 of a year's contributions and $50,000 in all
/// ($30,000 for a determination requested before 1 July 2022: GN 2024/1
/// section 3), tested on the contributions in full.
pub(super) fn count(pieces: &mut [Piece], total_limit: &Decimal) -> Result<()> {
    let year_limit = d("15000");
    let mut total = Decimal::from_int(0);
    let mut years: BTreeMap<i64, Decimal> = BTreeMap::new();
    for p in pieces.iter_mut() {
        let used = years.entry(p.fy).or_insert_with(|| Decimal::from_int(0));
        let year_room = year_limit.sub(used)?;
        let total_room = total_limit.sub(&total)?;
        let room = if year_room.lt(&total_room)? {
            year_room
        } else {
            total_room
        };
        p.counted = if p.amount.lt(&room)? {
            p.amount.clone()
        } else {
            room
        };
        *used = used.add(&p.counted)?;
        total = total.add(&p.counted)?;
    }
    Ok(())
}
