// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The steps of `cgt_parcels`: the year's gains and losses from the parcels
//! and the call's other figures, through ITAA 1997 s 102-5(1) steps 1 to 3 and
//! 5 (compilation 264, read on 19 November 2026).

use super::*;

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
pub(super) fn total(values: &[&Decimal]) -> Result<(String, Decimal)> {
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
pub(super) type Others = [Decimal; 4];

/// Each figure the steps show, named as the steps name them.
struct Figures {
    /// The parcels' discount gains, other gains and losses, with the sums
    /// written out.
    parcels: [(String, Decimal); 3],
    discount_gains: Decimal,
    other_gains: Decimal,
    losses: Decimal,
    /// The year's losses applied to the other and the discount gains.
    applied: [Decimal; 2],
    /// The earlier net capital losses applied to them.
    prior_applied: [Decimal; 2],
    discount: Decimal,
}

fn figures(pieces: &[Piece], others: &Others, fraction: [&Decimal; 2]) -> Result<Figures> {
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
    let parcels = [total(&gains(true))?, total(&gains(false))?, total(&losses)?];
    let [other_discount, other_gains, other_losses, prior] = others;
    let discount_gains = parcels[0].1.add(other_discount)?;
    let other_gains = parcels[1].1.add(other_gains)?;
    let losses = parcels[2].1.add(other_losses)?;
    let lo = min(&other_gains, &losses)?;
    let ld = min(&discount_gains, &losses.sub(&lo)?)?;
    let po = min(&other_gains.sub(&lo)?, prior)?;
    let pd = min(&discount_gains.sub(&ld)?, &prior.sub(&po)?)?;
    let [numerator, denominator] = fraction;
    let left = discount_gains.sub(&ld)?.sub(&pd)?;
    let discount = left.mul(numerator)?.div(denominator)?;
    Ok(Figures {
        parcels,
        discount_gains,
        other_gains,
        losses,
        applied: [lo, ld],
        prior_applied: [po, pd],
        discount,
    })
}

/// The gains and losses from the parcels and in all.
fn gain_steps(f: &Figures, others: &Others) -> Result<Vec<Step>> {
    let [(fd, pdg), (fo, pog), (fl, pl)] = &f.parcels;
    let [other_discount, other_gains, ..] = others;
    let (dg, og) = (&f.discount_gains, &f.other_gains);
    Ok(vec![
        step(
            "parcel_discount_gains",
            PARCEL_DISCOUNT_GAINS,
            fd,
            &[],
            pdg.clone(),
        ),
        step(
            "parcel_other_gains",
            PARCEL_OTHER_GAINS,
            fo,
            &[],
            pog.clone(),
        ),
        step("parcel_losses", PARCEL_LOSSES, fl, &[], pl.clone()),
        step(
            "discount_gains",
            PARCEL_DISCOUNT_GAINS,
            "parcel_discount_gains + other_discount_capital_gains",
            &[pdg, other_discount],
            dg.clone(),
        ),
        step(
            "other_gains",
            PARCEL_OTHER_GAINS,
            "parcel_other_gains + other_capital_gains",
            &[pog, other_gains],
            og.clone(),
        ),
        step(
            "total_capital_gains",
            "ITAA 1997 s 102-5(1) step 1: the capital gains made during the year, before losses and the discount (question 18 label H)",
            "discount_gains + other_gains",
            &[dg, og],
            dg.add(og)?,
        ),
    ])
}

/// The year's capital losses, and steps 1 and 2: those losses, then the
/// earlier net capital losses.
fn loss_steps(f: &Figures, others: &Others) -> Vec<Step> {
    let [_, _, other_losses, prior] = others;
    let (_, pl) = &f.parcels[2];
    let (dg, og, cl) = (&f.discount_gains, &f.other_gains, &f.losses);
    let ([lo, ld], [po, pd]) = (&f.applied, &f.prior_applied);
    vec![
        step(
            "capital_losses",
            PARCEL_LOSSES,
            "parcel_losses + other_capital_losses",
            &[pl, other_losses],
            f.losses.clone(),
        ),
        step(
            "losses_applied_to_other_gains",
            STEP_1,
            "min(other_gains, capital_losses)",
            &[og, cl],
            lo.clone(),
        ),
        step(
            "losses_applied_to_discount_gains",
            STEP_1,
            "min(discount_gains, capital_losses - losses_applied_to_other_gains)",
            &[dg, cl, lo],
            ld.clone(),
        ),
        step(
            "prior_losses_applied_to_other_gains",
            STEP_2,
            "min(other_gains - losses_applied_to_other_gains, prior_net_capital_losses)",
            &[og, lo, prior],
            po.clone(),
        ),
        step(
            "prior_losses_applied_to_discount_gains",
            STEP_2,
            "min(discount_gains - losses_applied_to_discount_gains, prior_net_capital_losses - prior_losses_applied_to_other_gains)",
            &[dg, ld, prior, po],
            pd.clone(),
        ),
    ]
}

/// Step 3's discount, step 5's net capital gain and what is carried forward.
fn net_steps(f: &Figures, prior: &Decimal, fraction: [&Decimal; 2]) -> Result<Vec<Step>> {
    let (dg, og, cl, discount) = (&f.discount_gains, &f.other_gains, &f.losses, &f.discount);
    let ([lo, ld], [po, pd]) = (&f.applied, &f.prior_applied);
    let [numerator, denominator] = fraction;
    let net = og
        .sub(lo)?
        .sub(po)?
        .add(dg)?
        .sub(ld)?
        .sub(pd)?
        .sub(discount)?;
    let net_loss = cl.sub(lo)?.sub(ld)?;
    let carried = net_loss.add(prior)?.sub(po)?.sub(pd)?;
    Ok(vec![
        step(
            "discount",
            STEP_3,
            "(discount_gains - losses_applied_to_discount_gains - prior_losses_applied_to_discount_gains) x discount_numerator / discount_denominator",
            &[dg, ld, pd, numerator, denominator],
            discount.clone(),
        ),
        step(
            "net_capital_gain",
            STEP_5,
            "other_gains - losses_applied_to_other_gains - prior_losses_applied_to_other_gains + discount_gains - losses_applied_to_discount_gains - prior_losses_applied_to_discount_gains - discount",
            &[og, lo, po, dg, ld, pd, discount],
            net,
        ),
        step(
            "net_capital_loss",
            NET_LOSS,
            "capital_losses - losses_applied_to_other_gains - losses_applied_to_discount_gains",
            &[cl, lo, ld],
            net_loss.clone(),
        ),
        step(
            "net_capital_losses_carried_forward",
            CARRIED,
            "net_capital_loss + prior_net_capital_losses - prior_losses_applied_to_other_gains - prior_losses_applied_to_discount_gains",
            &[&net_loss, prior, po, pd],
            carried,
        ),
    ])
}

pub(super) fn amount_steps(
    pieces: &[Piece],
    others: &Others,
    fraction: [&Decimal; 2],
) -> Result<Vec<Step>> {
    let f = figures(pieces, others, fraction)?;
    let mut steps = gain_steps(&f, others)?;
    steps.extend(loss_steps(&f, others));
    steps.extend(net_steps(&f, &others[3], fraction)?);
    Ok(steps)
}

fn min(a: &Decimal, b: &Decimal) -> Result<Decimal> {
    Ok(if b.lt(a)? { b.clone() } else { a.clone() })
}
