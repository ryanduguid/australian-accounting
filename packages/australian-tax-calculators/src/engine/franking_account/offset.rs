// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The year's totals and the franking deficit tax offset (s 205-70).

use super::*;

/// The year's totals the franking return and s 205-70 use.
pub(super) struct Totals {
    pub(super) credits: Decimal,
    pub(super) debits: Decimal,
    pub(super) closing: Decimal,
    pub(super) fdt: Decimal,
    pub(super) trigger: bool,
    pub(super) attributable_debits: Decimal,
    pub(super) label_k: Decimal,
}

pub(super) fn sum_of(lines: &[Line], keep: impl Fn(&Line) -> bool) -> Result<Decimal> {
    precise(|| {
        lines
            .iter()
            .filter(|l| keep(l))
            .try_fold(int(0), |t, l| Ok(t.add(&l.amount)?))
    })
}

pub(super) fn totals(opening: &Decimal, lines: &[Line]) -> Result<Totals> {
    let trigger = lines
        .iter()
        .any(|l| l.class == Class::Trigger && !l.amount.is_zero());
    let attributable_debits = sum_of(lines, |l| {
        l.class == Class::Trigger || (trigger && l.class == Class::Refund)
    })?;
    let label_k = sum_of(lines, |l| {
        l.class == Class::Other || (!trigger && l.class == Class::Refund)
    })?;
    let closing = balance(opening, lines)?;
    Ok(Totals {
        credits: sum_of(lines, |l| l.class == Class::Credit)?,
        debits: sum_of(lines, |l| l.class != Class::Credit)?,
        fdt: deficit(&closing)?,
        closing,
        trigger,
        attributable_debits,
        label_k,
    })
}

/// The current year's offset: the amount, the reduction, the franking
/// return's code and the authority.
pub(super) struct Offset {
    pub(super) amount: Decimal,
    pub(super) reduction: Decimal,
    pub(super) code: Option<&'static str>,
    pub(super) authority: &'static str,
}

pub(super) fn whole(t: &Totals, code: Option<&'static str>, authority: &'static str) -> Offset {
    Offset {
        amount: t.fdt.clone(),
        reduction: int(0),
        code,
        authority,
    }
}

/// The authorities the offset rests on.
const NO_TAX: &str = "ITAA 1997 s 205-70(1)(a): no franking deficit tax, so no offset for the year";
const NO_TRIGGER: &str = "ITAA 1997 s 205-70(2) step 1 and (8): no item 1, 3, 5 or 6 debit, so the offset is the whole franking deficit tax";
const WITHIN_TENTH: &str = "ITAA 1997 s 205-70(2) step 1: the part of the deficit attributable to item 1, 2, 3, 5 and 6 debits is not more than 10% of the year's credits, so the offset is the whole franking deficit tax";
const DETERMINED: &str = "ITAA 1997 s 205-70(6): the Commissioner determined that the excess was due to events outside the entity's control, so the offset is not reduced";
const FIRST_LIABILITY: &str = "ITAA 1997 s 205-70(5): a private company's first year of income tax liability, at least 90% of the deficit, so the offset is not reduced";
const REDUCED: &str = "ITAA 1997 s 205-70(2) step 1: the part of the deficit attributable to item 1, 2, 3, 5 and 6 debits exceeds 10% of the year's credits, so it is reduced by 30%";

/// s 205-70(2) step 1 by the ATO's method (section B, 'How to calculate the
/// reduced offset amount'): the franking deficit tax, less 30% of the part of
/// it attributable to item 1, 2, 3, 5 and 6 debits where that part exceeds
/// 10% of the year's credits; that part is the opening surplus and the
/// year's credits less those debits, when negative. Not reduced without an
/// item 1, 3, 5 or 6 debit (code F), for a private company meeting
/// s 205-70(5) (code P) or under a Commissioner's determination (s 205-70(6)).
pub(super) fn current_offset(
    opening: &Decimal,
    t: &Totals,
    facts: &Facts,
    tax: Option<&IncomeTax>,
    determination: bool,
) -> Result<Offset> {
    if t.fdt.is_zero() {
        return Ok(whole(t, None, NO_TAX));
    }
    if !t.trigger {
        return Ok(whole(t, Some("F"), NO_TRIGGER));
    }
    let part = attributable_part(opening, t)?;
    let tenth = precise(|| Ok(t.credits.mul(&d("0.10"))?))?;
    if !part.gt(&tenth)? {
        return Ok(whole(t, None, WITHIN_TENTH));
    }
    if determination {
        return Ok(whole(t, None, DETERMINED));
    }
    if facts.private && private_exception(t, tax)? {
        return Ok(whole(t, Some("P"), FIRST_LIABILITY));
    }
    let reduction = precise(|| Ok(part.mul(&d("0.30"))?))?;
    Ok(Offset {
        amount: precise(|| Ok(t.fdt.sub(&reduction)?))?,
        reduction,
        code: None,
        authority: REDUCED,
    })
}

/// The part of the deficit attributable to item 1, 2, 3, 5 and 6 debits by
/// the ATO's step 3: the opening surplus and the year's credits less those
/// debits, when negative.
pub(super) fn attributable_part(opening: &Decimal, t: &Totals) -> Result<Decimal> {
    let surplus = if opening.gt(&int(0))? {
        opening.clone()
    } else {
        int(0)
    };
    deficit(&precise(|| {
        Ok(surplus.add(&t.credits)?.sub(&t.attributable_debits)?)
    })?)
}

/// s 205-70(5)(b) to (d), for a private company: it would have had an income
/// tax liability without this offset, had none in an earlier year, and that
/// liability is at least 90% of the deficit.
pub(super) fn private_exception(t: &Totals, tax: Option<&IncomeTax>) -> Result<bool> {
    let Some(tax) = tax else {
        return refuse(
            "franking_account.income_tax_needed",
            "The offset reduction applies unless s 205-70(5) does, which turns on the private company's income tax: give income_tax with before_fdt_offset, offset_brought_forward and earlier_year_liability.",
        );
    };
    let floor = precise(|| Ok(t.fdt.mul(&d("0.90"))?))?;
    Ok(tax.earlier_liability == Some(false)
        && tax.before_offset.gt(&int(0))?
        && !tax.before_offset.lt(&floor)?)
}
