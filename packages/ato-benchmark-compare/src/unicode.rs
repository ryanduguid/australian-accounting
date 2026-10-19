//! `unicodedata.normalize("NFKD", text)` and `str.casefold()`, as Python 3.15
//! computes them, for matching business type names.

use crate::py;
use crate::pytext;
use crate::unitables::{CCC, NFKD};

fn ccc(cp: u32) -> u8 {
    CCC.binary_search_by_key(&cp, |&(c, _)| c)
        .map_or(0, |i| CCC[i].1)
}

/// The full compatibility decomposition of a text, canonically ordered.
pub fn nfkd(text: &str) -> String {
    let mut cps: Vec<u32> = Vec::new();
    for cp in pytext::code_points(text) {
        if (0xAC00..=0xD7A3).contains(&cp) {
            // Hangul syllables decompose algorithmically.
            let index = cp - 0xAC00;
            cps.push(0x1100 + index / 588);
            cps.push(0x1161 + (index % 588) / 28);
            if index % 28 != 0 {
                cps.push(0x11A7 + index % 28);
            }
        } else if let Ok(i) = NFKD.binary_search_by_key(&cp, |&(c, _)| c) {
            cps.extend_from_slice(NFKD[i].1);
        } else {
            cps.push(cp);
        }
    }
    // Canonical ordering: a stable sort of each run of non-starters.
    let mut start = 0;
    while start < cps.len() {
        if ccc(cps[start]) == 0 {
            start += 1;
            continue;
        }
        let mut end = start;
        while end < cps.len() && ccc(cps[end]) != 0 {
            end += 1;
        }
        cps[start..end].sort_by_key(|cp| ccc(*cp));
        start = end;
    }
    let mut out = String::with_capacity(cps.len());
    for cp in cps {
        pytext::push_code_point(&mut out, cp);
    }
    out
}

/// `str.casefold()`.
pub fn casefold(text: &str) -> String {
    py::casefold(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nfkd_matches_python() {
        assert_eq!(nfkd("\u{FB01}"), "fi");
        assert_eq!(nfkd("\u{E9}"), "e\u{301}");
        assert_eq!(nfkd("\u{AC01}"), "\u{1100}\u{1161}\u{11A8}");
        assert_eq!(nfkd("a\u{323}\u{302}"), "a\u{323}\u{302}");
        assert_eq!(nfkd("\u{1E09}\u{323}"), "c\u{327}\u{323}\u{301}");
        assert_eq!(casefold("Stra\u{DF}e"), "strasse");
    }
}
