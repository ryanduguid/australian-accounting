//! The subset of Python's `re` the engine's fixed patterns use, with CPython's
//! Unicode semantics: `\d`, `\s` and `\w` are Unicode classes, `$` also matches
//! before a final newline, and IGNORECASE compares simple lowercase forms plus
//! `sre_compile`'s equivalence fixes (ı for i, ſ for s, and so on).
//!
//! Supported: literals and `\uXXXX`, `.`, classes with ranges and `\d\s\w`,
//! groups `(...)` and `(?:...)`, single-width lookbehind `(?<=...)`, `|`,
//! greedy, lazy and possessive `? * + {m} {m,n}`, `^`, `$` and `\b`. Patterns
//! are compiled once from string literals; a bad pattern is a programming error.

use crate::pytext;
use crate::unitables::{SIMPLE_LOWER, WORD};

/// `_sre.unicode_tolower`.
pub fn lower(cp: u32) -> u32 {
    SIMPLE_LOWER
        .binary_search_by_key(&cp, |&(from, _)| from)
        .map_or(cp, |i| SIMPLE_LOWER[i].1)
}

fn in_ranges(table: &[(u32, u32)], cp: u32) -> bool {
    table
        .binary_search_by(|&(low, high)| {
            if high < cp {
                std::cmp::Ordering::Less
            } else if low > cp {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        })
        .is_ok()
}

/// `\w` for a `str` pattern: alphanumeric or underscore.
pub fn is_word(cp: u32) -> bool {
    in_ranges(WORD, cp)
}

pub fn is_digit(cp: u32) -> bool {
    pytext::decimal_value(cp).is_some()
}

pub fn is_space(cp: u32) -> bool {
    pytext::is_space(cp)
}

/// `sre_compile._ignorecase_fixes`: characters equal under IGNORECASE that
/// simple lowercasing does not unite.
const EQUIVALENCES: &[&[u32]] = &[
    &[0x69, 0x131],
    &[0x73, 0x17F],
    &[0xB5, 0x3BC],
    &[0x345, 0x3B9, 0x1FBE],
    &[0x390, 0x1FD3],
    &[0x3B0, 0x1FE3],
    &[0x3B2, 0x3D0],
    &[0x3B5, 0x3F5],
    &[0x3B8, 0x3D1],
    &[0x3BA, 0x3F0],
    &[0x3C0, 0x3D6],
    &[0x3C1, 0x3F1],
    &[0x3C2, 0x3C3],
    &[0x3C6, 0x3D5],
    &[0x1E61, 0x1E9B],
    &[0xFB05, 0xFB06],
];

fn fixes(cp: u32) -> &'static [u32] {
    EQUIVALENCES
        .iter()
        .find(|group| group.contains(&cp))
        .copied()
        .unwrap_or(&[])
}

#[derive(Clone, Copy, Debug)]
enum Category {
    Digit,
    Space,
    Word,
}

impl Category {
    fn test(self, cp: u32) -> bool {
        match self {
            Category::Digit => is_digit(cp),
            Category::Space => is_space(cp),
            Category::Word => is_word(cp),
        }
    }
}

#[derive(Clone, Debug)]
enum Item {
    Range(u32, u32),
    Category(Category, bool),
}

#[derive(Clone, Debug)]
enum Node {
    Char(u32),
    Any,
    Class(Vec<Item>, bool),
    Seq(Vec<Node>),
    Alt(Vec<Node>),
    Repeat {
        node: Box<Node>,
        min: usize,
        max: Option<usize>,
        mode: Mode,
    },
    Start,
    End,
    Boundary(bool),
    Behind(Box<Node>),
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Mode {
    Greedy,
    Lazy,
    Possessive,
}

pub struct Regex {
    node: Node,
    ignore_case: bool,
}

struct Parser<'a> {
    chars: Vec<char>,
    at: usize,
    source: &'a str,
}

impl Parser<'_> {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.at).copied()
    }

    fn next(&mut self) -> char {
        let c = self.chars[self.at];
        self.at += 1;
        c
    }

    fn eat(&mut self, c: char) -> bool {
        if self.peek() == Some(c) {
            self.at += 1;
            true
        } else {
            false
        }
    }

    fn fail(&self) -> ! {
        panic!("unsupported pattern {:?} at {}", self.source, self.at)
    }

    fn alternation(&mut self) -> Node {
        let mut branches = vec![self.sequence()];
        while self.eat('|') {
            branches.push(self.sequence());
        }
        if branches.len() == 1 {
            branches.pop().unwrap()
        } else {
            Node::Alt(branches)
        }
    }

    fn sequence(&mut self) -> Node {
        let mut items = Vec::new();
        while let Some(c) = self.peek() {
            if c == '|' || c == ')' {
                break;
            }
            let atom = self.atom();
            items.push(self.quantified(atom));
        }
        Node::Seq(items)
    }

    fn number(&mut self) -> usize {
        let start = self.at;
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.at += 1;
        }
        self.chars[start..self.at]
            .iter()
            .collect::<String>()
            .parse()
            .unwrap_or_else(|_| self.fail())
    }

    fn quantified(&mut self, atom: Node) -> Node {
        let (min, max) = match self.peek() {
            Some('?') => (0, Some(1)),
            Some('*') => (0, None),
            Some('+') => (1, None),
            Some('{') => {
                self.at += 1;
                let min = self.number();
                let max = if self.eat(',') {
                    Some(self.number())
                } else {
                    Some(min)
                };
                if !self.eat('}') {
                    self.fail();
                }
                self.at -= 1;
                (min, max)
            }
            _ => return atom,
        };
        self.at += 1;
        let mode = if self.eat('?') {
            Mode::Lazy
        } else if self.eat('+') {
            Mode::Possessive
        } else {
            Mode::Greedy
        };
        Node::Repeat {
            node: Box::new(atom),
            min,
            max,
            mode,
        }
    }

    fn escape(&mut self) -> Result<u32, Item> {
        match self.next() {
            'd' => Err(Item::Category(Category::Digit, false)),
            's' => Err(Item::Category(Category::Space, false)),
            'w' => Err(Item::Category(Category::Word, false)),
            'D' => Err(Item::Category(Category::Digit, true)),
            'S' => Err(Item::Category(Category::Space, true)),
            'W' => Err(Item::Category(Category::Word, true)),
            'n' => Ok(0x0A),
            't' => Ok(0x09),
            'r' => Ok(0x0D),
            'u' => {
                let hex: String = (0..4).map(|_| self.next()).collect();
                Ok(u32::from_str_radix(&hex, 16).unwrap_or_else(|_| self.fail()))
            }
            c if !c.is_ascii_alphanumeric() => Ok(c as u32),
            _ => self.fail(),
        }
    }

    fn atom(&mut self) -> Node {
        match self.next() {
            '(' => {
                if self.eat('?') {
                    if self.eat(':') {
                        let inner = self.alternation();
                        if !self.eat(')') {
                            self.fail();
                        }
                        return inner;
                    }
                    if self.eat('<') && self.eat('=') {
                        let inner = self.alternation();
                        if !self.eat(')') {
                            self.fail();
                        }
                        return Node::Behind(Box::new(inner));
                    }
                    self.fail();
                }
                let inner = self.alternation();
                if !self.eat(')') {
                    self.fail();
                }
                inner
            }
            '[' => self.class(),
            '.' => Node::Any,
            '^' => Node::Start,
            '$' => Node::End,
            '\\' => {
                if self.eat('b') {
                    return Node::Boundary(true);
                }
                if self.eat('B') {
                    return Node::Boundary(false);
                }
                match self.escape() {
                    Ok(cp) => Node::Char(cp),
                    Err(item) => Node::Class(vec![item], false),
                }
            }
            c => Node::Char(c as u32),
        }
    }

    fn class(&mut self) -> Node {
        let negated = self.eat('^');
        let mut items = Vec::new();
        let mut first = true;
        loop {
            let c = self.next();
            if c == ']' && !first {
                break;
            }
            first = false;
            let low = if c == '\\' {
                match self.escape() {
                    Ok(cp) => cp,
                    Err(item) => {
                        items.push(item);
                        continue;
                    }
                }
            } else {
                c as u32
            };
            if self.peek() == Some('-') && self.chars.get(self.at + 1).is_some_and(|c| *c != ']') {
                self.at += 1;
                let c = self.next();
                let high = if c == '\\' {
                    self.escape().unwrap_or_else(|_| self.fail())
                } else {
                    c as u32
                };
                items.push(Item::Range(low, high));
            } else {
                items.push(Item::Range(low, low));
            }
        }
        Node::Class(items, negated)
    }
}

impl Regex {
    pub fn new(pattern: &str) -> Regex {
        Regex::with_case(pattern, false)
    }

    pub fn ignore_case(pattern: &str) -> Regex {
        Regex::with_case(pattern, true)
    }

    fn with_case(pattern: &str, ignore_case: bool) -> Regex {
        let mut parser = Parser {
            chars: pattern.chars().collect(),
            at: 0,
            source: pattern,
        };
        let node = parser.alternation();
        if parser.at != parser.chars.len() {
            parser.fail();
        }
        Regex { node, ignore_case }
    }

    fn char_eq(&self, cp: u32, pattern: u32) -> bool {
        if cp == pattern {
            return true;
        }
        if !self.ignore_case {
            return false;
        }
        let want = lower(pattern);
        let got = lower(cp);
        got == want || fixes(want).contains(&got)
    }

    fn class_test(&self, items: &[Item], negated: bool, cp: u32) -> bool {
        let probe = if self.ignore_case { lower(cp) } else { cp };
        let hit = items.iter().any(|item| match item {
            Item::Category(category, inverted) => category.test(probe) != *inverted,
            Item::Range(low, high) => {
                if !self.ignore_case {
                    return (*low..=*high).contains(&cp);
                }
                // The compiled set holds the lowercase and fixes of each member.
                (*low..=*high).any(|member| {
                    let l = lower(member);
                    l == probe || fixes(l).contains(&probe)
                })
            }
        });
        hit != negated
    }

    fn word_at(text: &[u32], at: usize) -> bool {
        text.get(at).is_some_and(|cp| is_word(*cp))
    }

    /// Match `node` at `at`, calling `k` with each end position in
    /// backtracking order until it accepts.
    fn m(&self, node: &Node, text: &[u32], at: usize, k: &mut dyn FnMut(usize) -> bool) -> bool {
        match node {
            Node::Char(p) => at < text.len() && self.char_eq(text[at], *p) && k(at + 1),
            Node::Any => at < text.len() && text[at] != 0x0A && k(at + 1),
            Node::Class(items, negated) => {
                at < text.len() && self.class_test(items, *negated, text[at]) && k(at + 1)
            }
            Node::Seq(items) => self.seq(items, text, at, k),
            Node::Alt(branches) => branches.iter().any(|b| self.m(b, text, at, k)),
            Node::Start => at == 0 && k(at),
            Node::End => (at == text.len() || (at + 1 == text.len() && text[at] == 0x0A)) && k(at),
            Node::Boundary(expected) => {
                let boundary = (at > 0 && Self::word_at(text, at - 1)) != Self::word_at(text, at);
                boundary == *expected && k(at)
            }
            Node::Behind(inner) => {
                at > 0 && self.m(inner, text, at - 1, &mut |end| end == at) && k(at)
            }
            Node::Repeat {
                node,
                min,
                max,
                mode,
            } => self.repeat(node, *min, *max, *mode, text, at, 0, k),
        }
    }

    fn seq(
        &self,
        items: &[Node],
        text: &[u32],
        at: usize,
        k: &mut dyn FnMut(usize) -> bool,
    ) -> bool {
        match items.split_first() {
            None => k(at),
            Some((first, rest)) => {
                self.m(first, text, at, &mut |next| self.seq(rest, text, next, k))
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn repeat(
        &self,
        node: &Node,
        min: usize,
        max: Option<usize>,
        mode: Mode,
        text: &[u32],
        at: usize,
        count: usize,
        k: &mut dyn FnMut(usize) -> bool,
    ) -> bool {
        if count == 0
            && let Some(step) = self.single(node)
        {
            // One character per iteration: count the run, then try ends
            // without recursing once per character.
            let mut run = 0;
            while max.is_none_or(|m| run < m) && at + run < text.len() && step(text[at + run]) {
                run += 1;
            }
            if run < min {
                return false;
            }
            return match mode {
                Mode::Possessive => k(at + run),
                Mode::Greedy => (min..=run).rev().any(|n| k(at + n)),
                Mode::Lazy => (min..=run).any(|n| k(at + n)),
            };
        }
        if mode == Mode::Possessive {
            // Atomic: take each iteration's first match, never give one back.
            let mut pos = at;
            let mut n = 0;
            while max.is_none_or(|m| n < m) {
                let mut found = None;
                self.m(node, text, pos, &mut |end| {
                    found = Some(end);
                    true
                });
                match found {
                    Some(end) if end != pos => {
                        pos = end;
                        n += 1;
                    }
                    _ => break,
                }
            }
            return n >= min && k(pos);
        }
        if count == 0 && Self::deterministic(node) {
            // Each iteration has one possible end, so collect the ends in a
            // loop, then offer them in backtracking order: a long run such as
            // "1,000,000,..." no longer recurses once per iteration.
            let mut ends = vec![at];
            while max.is_none_or(|m| ends.len() <= m) {
                let pos = *ends.last().unwrap();
                let mut found = None;
                self.m(node, text, pos, &mut |end| {
                    found = Some(end);
                    true
                });
                match found {
                    Some(end) if end != pos => ends.push(end),
                    _ => break,
                }
            }
            if ends.len() <= min {
                return false;
            }
            let mut tries = ends[min..].iter();
            return match mode {
                Mode::Greedy => tries.rev().any(|&end| k(end)),
                _ => tries.any(|&end| k(end)),
            };
        }
        let can_more = max.is_none_or(|m| count < m);
        let more = |k: &mut dyn FnMut(usize) -> bool| {
            can_more
                && self.m(node, text, at, &mut |end| {
                    // An empty iteration cannot make progress; stop it looping.
                    end != at && self.repeat(node, min, max, mode, text, end, count + 1, k)
                })
        };
        if count < min {
            return more(k);
        }
        match mode {
            Mode::Greedy => more(k) || k(at),
            _ => k(at) || more(k),
        }
    }

    /// Whether `node` can end in at most one place from any start.
    fn deterministic(node: &Node) -> bool {
        match node {
            Node::Char(_)
            | Node::Any
            | Node::Class(..)
            | Node::Start
            | Node::End
            | Node::Boundary(_)
            | Node::Behind(_) => true,
            Node::Seq(items) => items.iter().all(Self::deterministic),
            Node::Alt(_) => false,
            Node::Repeat {
                node,
                min,
                max,
                mode,
            } => *mode == Mode::Possessive || (*max == Some(*min) && Self::deterministic(node)),
        }
    }

    /// A node that always consumes exactly one character, as a predicate.
    fn single<'s>(&'s self, node: &'s Node) -> Option<Box<dyn Fn(u32) -> bool + 's>> {
        match node {
            Node::Char(p) => Some(Box::new(move |cp| self.char_eq(cp, *p))),
            Node::Any => Some(Box::new(|cp| cp != 0x0A)),
            Node::Class(items, negated) => {
                Some(Box::new(move |cp| self.class_test(items, *negated, cp)))
            }
            _ => None,
        }
    }

    fn code_points(text: &str) -> Vec<u32> {
        pytext::code_points(text).collect()
    }

    /// The end of a match starting exactly at `at`, if any.
    fn match_from(&self, text: &[u32], at: usize) -> Option<usize> {
        let mut end = None;
        self.m(&self.node, text, at, &mut |e| {
            end = Some(e);
            true
        });
        end
    }

    /// `pattern.match(text) is not None`.
    pub fn matches_start(&self, text: &str) -> bool {
        self.match_from(&Self::code_points(text), 0).is_some()
    }

    /// `pattern.fullmatch(text) is not None`.
    pub fn fullmatch(&self, text: &str) -> bool {
        let cps = Self::code_points(text);
        self.m(&self.node, &cps, 0, &mut |end| end == cps.len())
    }

    /// `pattern.search(text) is not None`.
    pub fn search(&self, text: &str) -> bool {
        let cps = Self::code_points(text);
        (0..=cps.len()).any(|at| self.match_from(&cps, at).is_some())
    }

    /// `pattern.sub(replacement, text)` for a literal replacement.
    pub fn sub(&self, replacement: &str, text: &str) -> String {
        let cps = Self::code_points(text);
        let mut out = String::new();
        let mut at = 0;
        let mut copied = 0;
        while at <= cps.len() {
            match self.match_from(&cps, at) {
                Some(end) => {
                    for &cp in &cps[copied..at] {
                        pytext::push_code_point(&mut out, cp);
                    }
                    out.push_str(replacement);
                    copied = end;
                    // An empty match advances one position past itself.
                    at = if end == at {
                        if at < cps.len() {
                            pytext::push_code_point(&mut out, cps[at]);
                        }
                        copied = at + 1;
                        at + 1
                    } else {
                        end
                    };
                }
                None => at += 1,
            }
        }
        for &cp in &cps[copied.min(cps.len())..] {
            pytext::push_code_point(&mut out, cp);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unicode_classes_and_anchors_follow_python() {
        let number = Regex::new(r"^[\d,]*+(?:\.\d++|(?<=\d))$");
        assert!(number.matches_start("00123"));
        assert!(number.matches_start("\u{663}\u{664}"));
        assert!(number.matches_start("1,234.5\n"));
        assert!(!number.matches_start("A1"));
        assert!(!number.matches_start(""));
        assert!(!number.matches_start(",,"));
        assert!(number.matches_start(".5"));
        let rent = Regex::ignore_case(r"\brent\b");
        assert!(rent.search("Office RENT"));
        assert!(!rent.search("Parental leave"));
        assert!(!rent.search("rental"));
        let sales = Regex::ignore_case(r"\bsales\b");
        assert!(sales.search("\u{17F}ales"), "long s folds to s");
        assert!(Regex::ignore_case("kitchen").search("\u{212A}itchen"));
        assert!(Regex::ignore_case("income").search("\u{130}ncome"));
        assert!(Regex::ignore_case("income").search("\u{131}ncome"));
    }

    #[test]
    fn sub_replaces_leftmost_non_overlapping_matches() {
        let spaces = Regex::new(r"\s+");
        assert_eq!(spaces.sub(" ", "a \t b\u{3000}c"), "a b c");
        let cleaned = Regex::ignore_case(r"[\s$,()]|CR$|DR$");
        assert_eq!(cleaned.sub("", "$ (1,234.50) cr"), "1234.50");
    }
}
