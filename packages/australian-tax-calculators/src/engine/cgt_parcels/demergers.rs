// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The demergers `cgt_cost_base_adjustments` carries, each from its class
//! ruling read on 22 November 2026, and the cost base split of ITAA 1997
//! ss 125-80(2) and (3) and 125-85.

use super::events::Event;
use super::work::{NEW_INTEREST, Work, bases, max_nil};
use super::*;

/// A demerger with a published class ruling: one new share for each head
/// share, the Commissioner's apportionment of the post-CGT head shares' cost
/// bases, and the capital reduction per head share (CGT event G1).
pub(super) struct Demerger {
    pub(super) key: &'static str,
    /// The implementation date, when G1 happened and the new shares were
    /// acquired (s 109-5).
    pub(super) implemented: &'static str,
    /// The record date: the parcels acquired after it took no part.
    pub(super) record: &'static str,
    pub(super) head_share: &'static str,
    pub(super) new_share: &'static str,
    pub(super) capital_reduction: &'static str,
    pub(super) ruling: &'static str,
}

pub(super) const DEMERGERS: [Demerger; 2] = [
    Demerger {
        key: "wesfarmers_coles_2018",
        implemented: "2018-11-28",
        record: "2018-11-22",
        head_share: "0.7109",
        new_share: "0.2891",
        capital_reduction: "5.68",
        ruling: "CR 2018/59 paras 10, 20 to 22, 37 and 46; the ATO's 'Wesfarmers Limited – demerger of Coles Group Limited (2018)'",
    },
    Demerger {
        key: "woolworths_endeavour_2021",
        implemented: "2021-07-01",
        record: "2021-06-25",
        head_share: "0.8581",
        new_share: "0.1419",
        capital_reduction: "0.71",
        ruling: "CR 2021/50 paras 1, 5, 10, 16 and 47",
    },
];

/// `total` split by `weights`, each part to the cent and the last taking
/// what is left, so the parts add up to `total`.
pub(super) fn split(total: &Decimal, weights: &[&Decimal]) -> Result<Vec<Decimal>> {
    let mut whole = int(0);
    for w in weights {
        whole = whole.add(w)?;
    }
    let mut left = total.clone();
    let mut parts = Vec::with_capacity(weights.len());
    for (i, w) in weights.iter().enumerate() {
        let part = if i + 1 == weights.len() {
            left.clone()
        } else {
            cents(total.mul(w)?.div(&whole)?)?
        };
        left = left.sub(&part)?;
        parts.push(part);
    }
    Ok(parts)
}

/// The head shares' and the new shares' parts of `sum`, each rounded to the
/// cent on its own as the ATO's examples do, so they can differ from `sum` by
/// a cent.
pub(super) fn sides(sum: &Decimal, head: &str, new: &str) -> Result<[Decimal; 2]> {
    Ok([
        cents(sum.mul(&Decimal::parse(head)?)?)?,
        cents(sum.mul(&Decimal::parse(new)?)?)?,
    ])
}

impl Work {
    /// The head shares held at the record date keep the head share part of
    /// their cost bases' sum, and each gives one new share with the new share
    /// part; without roll-over, G1 makes a gain where the capital reduction is
    /// above a parcel's cost base (s 104-135(3)), which roll-over disregards
    /// (s 125-80(1)).
    pub(super) fn demerge(
        &mut self,
        event: &Event,
        d: &Demerger,
        new_security: &str,
        rollover: bool,
    ) -> Result<()> {
        let found = self.affected(event, dates::parse(d.record).expect("a date"))?;
        let before: Vec<[Decimal; 2]> = found
            .iter()
            .map(|&i| bases(&self.parcels[i]))
            .collect::<Result<_>>()?;
        let mut sum = int(0);
        for b in &before {
            sum = sum.add(&b[0])?;
        }
        let [head, new] = sides(&sum, d.head_share, d.new_share)?;
        let units: Vec<Decimal> = found
            .iter()
            .map(|&i| self.parcels[i].units.clone())
            .collect();
        let weights: Vec<&Decimal> = units.iter().collect();
        let parts = split(&head, &weights)?
            .into_iter()
            .zip(split(&new, &weights)?);
        let per_share = Decimal::parse(d.capital_reduction)?;
        let authority = format!(
            "ITAA 1997 ss 125-80(2) and (3) with roll-over, s 125-85 without, and CGT event G1 (s 104-135(3)), disregarded with roll-over (s 125-80(1)); {}",
            d.ruling
        );
        for ((k, &i), (head_part, new_part)) in found.iter().enumerate().zip(parts) {
            let paid = cents(units[k].mul(&per_share)?)?;
            let gain = if rollover {
                int(0)
            } else {
                max_nil(paid.sub(&before[k][0])?)?
            };
            self.parcels[i].base = head_part;
            self.parcels[i].owning = int(0);
            let facts = (Some(paid), before[k].clone(), gain, authority.clone());
            self.row(event, i, "demerger", facts)?;
            self.new_parcel(event, i, new_security, new_part)?;
        }
        Ok(())
    }

    /// The new shares parcel `i` gives, named after it.
    fn new_parcel(&mut self, event: &Event, i: usize, security: &str, base: Decimal) -> Result<()> {
        let from = &self.parcels[i];
        let id = format!("{security} from {}", from.id);
        if self.parcels.iter().any(|p| p.id == id) {
            return conflict(format!(
                "event {}: a parcel is already named {id}.",
                event.n
            ));
        }
        let parcel = Parcel {
            id,
            security: security.to_string(),
            acquired: from.acquired,
            units: from.units.clone(),
            base,
            owning: int(0),
        };
        self.parcels.push(parcel);
        let nil = [int(0), int(0)];
        let facts = (None, nil, int(0), NEW_INTEREST.to_string());
        self.row(
            event,
            self.parcels.len() - 1,
            "demerger_new_interest",
            facts,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(text: &str) -> Decimal {
        Decimal::parse(text).unwrap()
    }

    /// ITAA 1997 s 125-80(3) example: Peter's 400 A shares at $4.60 and 5%
    /// to his 24 B shares.
    #[test]
    fn peter_s_shares_split_as_the_act_s_example_does() {
        let [a, b] = sides(&d("1840"), "0.95", "0.05").unwrap();
        assert_eq!(
            (a.to_string(), b.to_string()),
            ("1748.00".into(), "92.00".into())
        );
    }

    /// The Guide to capital gains tax 2025, example 30: Anita's $2,500 at
    /// 94.937% and 5.063% rounds to $2,373.43 and $126.58, a cent over.
    #[test]
    fn each_side_rounds_on_its_own_as_anita_s_example_does() {
        let [a, b] = sides(&d("2500"), "0.94937", "0.05063").unwrap();
        assert_eq!(
            (a.to_string(), b.to_string()),
            ("2373.43".into(), "126.58".into())
        );
    }

    #[test]
    fn a_split_adds_up_with_the_last_part_taking_the_rest() {
        let parts = split(&d("16350.70"), &[&d("2000"), &d("1000")]).unwrap();
        let shown: Vec<String> = parts.iter().map(|p| p.to_string()).collect();
        assert_eq!(shown, ["10900.47", "5450.23"]);
    }
}
