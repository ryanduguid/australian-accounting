// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

pub(super) fn compare_integer(left: &str, right: &str) -> std::cmp::Ordering {
    let a = unsigned_digits(left);
    let b = unsigned_digits(right);
    let negative_a = left.starts_with("-") && !a.is_empty();
    let negative_b = right.starts_with("-") && !b.is_empty();
    match (negative_a, negative_b) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => ordered(&a, &b, negative_a),
    }
}

fn unsigned_digits(text: &str) -> String {
    let unsigned = text.strip_prefix("-").unwrap_or(text);
    unsigned.trim_start_matches("0").to_string()
}

fn ordered(left: &str, right: &str, negative: bool) -> std::cmp::Ordering {
    let order = if left.len() == right.len() {
        left.cmp(right)
    } else {
        left.len().cmp(&right.len())
    };
    if negative { order.reverse() } else { order }
}
