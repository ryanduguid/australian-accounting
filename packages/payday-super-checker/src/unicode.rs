//! `unicodedata.normalize("NFKD", text)` and `str.casefold()`, as Python 3.15
//! computes them, for matching business type names.

use crate::compose::COMPOSE;
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

fn compose_pair(a: u32, b: u32) -> Option<u32> {
    // Hangul syllables compose algorithmically.
    if (0x1100..=0x1112).contains(&a) && (0x1161..=0x1175).contains(&b) {
        return Some(0xAC00 + ((a - 0x1100) * 21 + (b - 0x1161)) * 28);
    }
    if (0xAC00..=0xD7A3).contains(&a)
        && (a - 0xAC00).is_multiple_of(28)
        && (0x11A8..=0x11C2).contains(&b)
    {
        return Some(a + (b - 0x11A7));
    }
    COMPOSE
        .binary_search_by(|&(x, y, _)| (x, y).cmp(&(a, b)))
        .ok()
        .map(|i| COMPOSE[i].2)
}

/// `unicodedata.normalize("NFKC", text)`: the compatibility decomposition,
/// then canonical composition.
pub fn nfkc(text: &str) -> String {
    let decomposed: Vec<u32> = pytext::code_points(&nfkd(text)).collect();
    let mut out: Vec<u32> = Vec::with_capacity(decomposed.len());
    let mut starter: Option<usize> = None;
    let mut last: Option<u8> = None;
    for cp in decomposed {
        let class = ccc(cp);
        if let Some(at) = starter {
            let blocked = last.is_some_and(|l| l == 0 || l >= class);
            if !blocked && let Some(composite) = compose_pair(out[at], cp) {
                out[at] = composite;
                continue;
            }
        }
        if class == 0 {
            starter = Some(out.len());
            last = None;
        } else {
            last = Some(class);
        }
        out.push(cp);
    }
    let mut text = String::with_capacity(out.len());
    for cp in out {
        pytext::push_code_point(&mut text, cp);
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nfkc_matches_python() {
        assert_eq!(nfkc("e\u{301}"), "\u{E9}");
        assert_eq!(nfkc("\u{FB01}"), "fi");
        assert_eq!(nfkc("\u{1100}\u{1161}\u{11A8}"), "\u{AC01}");
        assert_eq!(nfkc("\u{FF21}\u{3000}"), "A ");
        assert_eq!(nfkc("a\u{323}\u{302}"), "\u{1EAD}");
    }

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
