// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use super::rates::{SUBITEMS, rates};
use crate::dates;

#[test]
fn the_bundled_rates_are_whole_and_contiguous() {
    let r = rates();
    assert_eq!(r.periods.len(), 10);
    assert_eq!(dates::format(r.periods[0].from), "2022-07-01");
    assert_eq!(dates::format(r.periods[9].to), "2027-01-31");
    assert!(r.periods.iter().all(|p| p.rates.len() == SUBITEMS.len()));
}

/// Excise Tariff Act 1921 s 6L: the draught subitems stay at their 31 July
/// 2025 rates through the 4 frozen indexation days held here.
#[test]
fn the_draught_rates_are_frozen_from_august_2025() {
    let r = rates();
    let frozen = &r.periods[6..];
    for i in [1, 3, 5] {
        assert!(
            frozen
                .iter()
                .all(|p| p.rates[i].equals(&frozen[0].rates[i]).unwrap()),
            "{}",
            SUBITEMS[i]
        );
    }
}
