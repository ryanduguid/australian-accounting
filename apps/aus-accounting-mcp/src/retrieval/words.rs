// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! How a query's words are compared with casefolded text, as the Python server's
//! corpus.py reads them: whole words, a modal verb and the "not" after it as one
//! word, and a plain question's stop words dropped.
use super::POLICY;
use payday_super_checker::pyre::{is_digit, is_space, is_word};

fn listed(name: &str, word: &str) -> bool {
    POLICY["corpus"][name]
        .as_array()
        .unwrap()
        .iter()
        .any(|w| w == word)
}

pub(super) fn is_stop(word: &str) -> bool {
    listed("STOP_WORDS", word)
}

pub(super) fn is_modal(word: &str) -> bool {
    listed("MODALS", word)
}

/// The distinct words of terms: a modal and the "not" after it are one word, as
/// "cannot" is, which is kept as "can not".
pub(super) fn query_words(terms: &[String]) -> Vec<String> {
    let mut words: Vec<String> = Vec::new();
    for term in terms {
        match words.last_mut() {
            Some(last) if term == "not" && is_modal(last) => last.push_str(" not"),
            _ if term == "cannot" => words.push("can not".into()),
            _ => words.push(term.clone()),
        }
    }
    let mut distinct: Vec<String> = Vec::new();
    for word in words {
        if !distinct.contains(&word) {
            distinct.push(word);
        }
    }
    distinct
}

/// query_words of a query without its stop words, unless it holds nothing else.
pub(super) fn search_words(terms: &[String]) -> Vec<String> {
    let kept: Vec<String> = terms.iter().filter(|t| !is_stop(t)).cloned().collect();
    query_words(if kept.is_empty() { terms } else { &kept })
}

/// What a raw line holds if its row holds words or their opposites.
pub(super) fn parts(words: &[String]) -> Vec<String> {
    words
        .iter()
        .flat_map(|w| w.split(' '))
        .map(str::to_owned)
        .collect()
}

/// The other form of a modal query word: "must" for "must not", and back.
pub(super) fn opposite(term: &str) -> String {
    match term.strip_suffix(" not") {
        Some(modal) => modal.into(),
        None => format!("{term} not"),
    }
}

pub(super) fn has_digit(term: &str) -> bool {
    term.chars().any(|c| is_digit(c as u32))
}

fn word_char(text: &[char], at: usize) -> bool {
    text.get(at).is_some_and(|c| is_word(*c as u32))
}

/// Whether `word` sits at `at` with a word boundary before it.
fn starts(text: &[char], at: usize, word: &str) -> bool {
    (at == 0 || !word_char(text, at - 1))
        && word
            .chars()
            .enumerate()
            .all(|(i, c)| text.get(at + i) == Some(&c))
}

/// `\s+not\b` at `at`: the end of the match, if any.
fn then_not(text: &[char], at: usize) -> Option<usize> {
    let mut next = at;
    while text.get(next).is_some_and(|c| is_space(*c as u32)) {
        next += 1;
    }
    let end = next + 3;
    (next > at && starts_unbounded(text, next, "not") && !word_char(text, end)).then_some(end)
}

fn starts_unbounded(text: &[char], at: usize, word: &str) -> bool {
    word.chars()
        .enumerate()
        .all(|(i, c)| text.get(at + i) == Some(&c))
}

/// corpus.py's `_pattern(term).search(text)`: the whole-word test for one query word.
/// A modal alone refuses its negated form.
fn search(term: &str, text: &[char]) -> bool {
    (0..text.len()).any(|at| {
        if term == "can not" {
            (starts(text, at, "cannot") && !word_char(text, at + 6))
                || (starts(text, at, "can") && then_not(text, at + 3).is_some())
        } else if let Some(modal) = term.strip_suffix(" not") {
            starts(text, at, modal) && then_not(text, at + modal.chars().count()).is_some()
        } else {
            let end = at + term.chars().count();
            starts(text, at, term)
                && !word_char(text, end)
                && !(is_modal(term) && then_not(text, end).is_some())
        }
    })
}

/// corpus.py's `_has`: whether casefolded text holds term as `_pattern` reads it.
pub(super) fn has(term: &str, text: &str) -> bool {
    let first = term.split(' ').next().unwrap();
    text.contains(first) && search(term, &text.chars().collect::<Vec<_>>())
}

/// Whether casefolded text holds every word of words.
pub(super) fn holds_all(words: &[String], text: &str) -> bool {
    words.iter().all(|word| has(word, text))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(words: &[&str]) -> Vec<String> {
        words.iter().map(|w| w.to_string()).collect()
    }

    #[test]
    fn a_modal_and_its_negation_are_different_words() {
        assert_eq!(
            query_words(&strings(&["must", "not", "lodge", "cannot", "must"])),
            strings(&["must not", "lodge", "can not", "must"])
        );
        assert!(has("must", "you must lodge"));
        assert!(!has("must", "you must not lodge"));
        assert!(has("must", "you must, not lodge"));
        assert!(has("must not", "you must \t not lodge"));
        assert!(!has("must not", "you must nothing"));
        assert!(has("can not", "it cannot"));
        assert!(has("can not", "it can not"));
        assert!(!has("can", "it cannot"));
        assert!(!has("car", "scar limit"));
        assert!(has("40", "s 40-230"));
    }

    #[test]
    fn stop_words_leave_a_question_unless_nothing_else_is_left() {
        assert_eq!(
            search_words(&strings(&["what", "is", "the", "car", "limit"])),
            strings(&["car", "limit"])
        );
        assert_eq!(
            search_words(&strings(&["what", "is"])),
            strings(&["what", "is"])
        );
        assert_eq!(
            parts(&strings(&["must not", "lodge"])),
            strings(&["must", "not", "lodge"])
        );
        assert_eq!(opposite("must"), "must not");
        assert_eq!(opposite("must not"), "must");
    }
}
