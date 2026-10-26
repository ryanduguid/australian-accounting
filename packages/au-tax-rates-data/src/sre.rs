//! Python's `re` for the patterns the records carry.
//!
//! A record's `pattern` is Python regular-expression syntax read from data, so
//! it is parsed as CPython 3.15's `re/_parser.py` parses it: the same
//! `re.error` messages and positions, the same group count, and the same
//! `OverflowError` and `ValueError` for repeat counts. The tree is then
//! matched by a backtracking matcher that follows `_sre`: leftmost search,
//! greedy, lazy and possessive repeats with the zero-width-iteration guard,
//! atomic groups, fixed-width lookbehind, backreferences and conditionals,
//! and `re/_compiler.py`'s case-insensitive rules (`_sre.unicode_tolower`
//! and `re._casefix`). Patterns and text are code points, as Python's `str`
//! holds them, so a lone surrogate is matched like any other character.
//!
//! Not ported: `\N{name}` escapes, which need the Unicode name database (they
//! fail as a pattern error naming the escape), the `FutureWarning`s Python
//! prints for possible set operations, and Python's recursion limit, which
//! is stood in for by refusing groups nested more than [`MAX_NESTING`] deep.
//! The matcher recurses once per iteration of a repeat whose body is not a
//! single character, so the caller provides a deep stack.
use std::collections::HashMap;

use crate::py;
use crate::pycase;
use crate::pyvalue::repr_points;

pub const IGNORECASE: u32 = 2;
pub const MULTILINE: u32 = 8;
pub const DOTALL: u32 = 16;
pub const UNICODE: u32 = 32;
pub const VERBOSE: u32 = 64;
pub const ASCII: u32 = 256;
const LOCALE: u32 = 4;
const DEBUG: u32 = 128;
const TYPE_FLAGS: u32 = ASCII | LOCALE | UNICODE;

pub const MAXREPEAT: u64 = 4_294_967_295;
const MAXGROUPS: usize = 1_073_741_823;
const MAXWIDTH: u128 = 1 << 64;
const MAXCODE: u128 = (1 << 32) - 1;
/// Deepest group nesting accepted; CPython's parser recurses twice per level
/// and stops near 1,000 frames.
pub const MAX_NESTING: usize = 480;

fn pts(s: &str) -> Vec<u32> {
    s.chars().map(u32::from).collect()
}

/// Why `re.compile` raised.
#[derive(Clone, Debug, PartialEq)]
pub enum CompileError {
    /// `re.PatternError`, as `str()` renders it.
    Pattern(Vec<u32>),
    /// Another exception: its class name and message.
    Other(&'static str, String),
}

impl CompileError {
    /// The exception's `str()`.
    pub fn text(&self) -> Vec<u32> {
        match self {
            CompileError::Pattern(text) => text.clone(),
            CompileError::Other(_, message) => pts(message),
        }
    }
}

fn pattern_error(msg: Vec<u32>, pattern: &[u32], pos: Option<usize>) -> CompileError {
    let mut text = msg;
    if let Some(pos) = pos {
        text.extend(pts(&format!(" at position {pos}")));
        let nl = u32::from('\n');
        if pattern.contains(&nl) {
            let before = &pattern[..pos.min(pattern.len())];
            let lineno = before.iter().filter(|&&c| c == nl).count() + 1;
            let colno = match before.iter().rposition(|&c| c == nl) {
                Some(i) => pos - i,
                None => pos + 1,
            };
            text.extend(pts(&format!(" (line {lineno}, column {colno})")));
        }
    }
    CompileError::Pattern(text)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cat {
    Digit,
    NotDigit,
    Space,
    NotSpace,
    Word,
    NotWord,
}

impl Cat {
    fn negated(self) -> Cat {
        match self {
            Cat::Digit => Cat::NotDigit,
            Cat::NotDigit => Cat::Digit,
            Cat::Space => Cat::NotSpace,
            Cat::NotSpace => Cat::Space,
            Cat::Word => Cat::NotWord,
            Cat::NotWord => Cat::Word,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Item {
    Negate,
    Literal(u32),
    Range(u32, u32),
    Category(Cat),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum At {
    Beginning,
    BeginningString,
    End,
    EndString,
    Boundary,
    NonBoundary,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rep {
    Max,
    Min,
    Possessive,
}

type Sub = Vec<Node>;

#[derive(Clone, Debug)]
enum Node {
    Literal(u32),
    NotLiteral(u32),
    Any,
    In(Vec<Item>),
    Branch(Vec<Sub>),
    Subpattern(Option<usize>, u32, u32, Sub),
    Repeat(Rep, u64, u64, Sub),
    At(At),
    GroupRef(usize),
    GroupRefExists(usize, Sub, Option<Sub>),
    /// `(?=...)` and `(?<=...)` (`negative` false) or their negations; `dir` is -1 behind.
    Assert(bool, i8, Sub),
    Atomic(Sub),
    Failure,
}

/// `==` as the parser's common-prefix check compares two items: a tuple that
/// holds a subpattern equals only itself.
fn node_eq(a: &Node, b: &Node) -> bool {
    match (a, b) {
        (Node::Literal(x), Node::Literal(y)) | (Node::NotLiteral(x), Node::NotLiteral(y)) => x == y,
        (Node::Any, Node::Any) | (Node::Failure, Node::Failure) => true,
        (Node::In(x), Node::In(y)) => x == y,
        (Node::At(x), Node::At(y)) => x == y,
        (Node::GroupRef(x), Node::GroupRef(y)) => x == y,
        _ => false,
    }
}

/// One tokenizer step: a character, or a backslash and the character after it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Tok {
    c: u32,
    escaped: Option<u32>,
}

impl Tok {
    fn is(&self, c: char) -> bool {
        self.escaped.is_none() && self.c == u32::from(c)
    }

    fn len(&self) -> usize {
        1 + usize::from(self.escaped.is_some())
    }

    fn points(&self) -> Vec<u32> {
        match self.escaped {
            Some(e) => vec![self.c, e],
            None => vec![self.c],
        }
    }

    /// `this in chars` for a one-character token.
    fn among(&self, chars: &str) -> bool {
        self.escaped.is_none() && chars.chars().any(|c| u32::from(c) == self.c)
    }
}

fn is_digit(t: Option<Tok>) -> bool {
    t.is_some_and(|t| t.among("0123456789"))
}

fn is_octal(t: Option<Tok>) -> bool {
    t.is_some_and(|t| t.among("01234567"))
}

fn is_hex(t: Option<Tok>) -> bool {
    t.is_some_and(|t| t.among("0123456789abcdefABCDEF"))
}

struct Tokenizer<'p> {
    string: &'p [u32],
    index: usize,
    next: Option<Tok>,
}

type PResult<T> = Result<T, CompileError>;

impl<'p> Tokenizer<'p> {
    fn new(string: &'p [u32]) -> PResult<Self> {
        let mut t = Tokenizer {
            string,
            index: 0,
            next: None,
        };
        t.advance()?;
        Ok(t)
    }

    fn advance(&mut self) -> PResult<()> {
        let index = self.index;
        let Some(&c) = self.string.get(index) else {
            self.next = None;
            return Ok(());
        };
        if c == u32::from('\\') {
            let Some(&e) = self.string.get(index + 1) else {
                return Err(pattern_error(
                    pts("bad escape (end of pattern)"),
                    self.string,
                    Some(self.string.len() - 1),
                ));
            };
            self.index = index + 2;
            self.next = Some(Tok {
                c,
                escaped: Some(e),
            });
        } else {
            self.index = index + 1;
            self.next = Some(Tok { c, escaped: None });
        }
        Ok(())
    }

    fn matches(&mut self, c: char) -> PResult<bool> {
        if self.next.is_some_and(|t| t.is(c)) {
            self.advance()?;
            return Ok(true);
        }
        Ok(false)
    }

    fn get(&mut self) -> PResult<Option<Tok>> {
        let this = self.next;
        self.advance()?;
        Ok(this)
    }

    fn getwhile(&mut self, n: usize, accept: fn(Option<Tok>) -> bool) -> PResult<Vec<u32>> {
        let mut result = Vec::new();
        for _ in 0..n {
            let c = self.next;
            if !accept(c) {
                break;
            }
            result.extend(c.map(|t| t.points()).unwrap_or_default());
            self.advance()?;
        }
        Ok(result)
    }

    fn getuntil(&mut self, terminator: char, name: &str) -> PResult<Vec<u32>> {
        let mut result: Vec<u32> = Vec::new();
        loop {
            let c = self.next;
            self.advance()?;
            let Some(c) = c else {
                if result.is_empty() {
                    return Err(self.error(pts(&format!("missing {name}")), 0));
                }
                return Err(self.error(
                    pts(&format!("missing {terminator}, unterminated name")),
                    result.len(),
                ));
            };
            if c.is(terminator) {
                if result.is_empty() {
                    return Err(self.error(pts(&format!("missing {name}")), 1));
                }
                break;
            }
            result.extend(c.points());
        }
        Ok(result)
    }

    fn tell(&self) -> usize {
        self.index - self.next.map_or(0, |t| t.len())
    }

    fn seek(&mut self, index: usize) -> PResult<()> {
        self.index = index;
        self.advance()
    }

    fn error(&self, msg: Vec<u32>, offset: usize) -> CompileError {
        pattern_error(msg, self.string, Some(self.tell() - offset))
    }

    fn checkgroupname(&self, name: &[u32], offset: usize) -> PResult<()> {
        if !is_identifier(name) {
            let mut msg = pts("bad character in group name ");
            msg.extend(pts(&repr_points(name)));
            return Err(self.error(msg, name.len() + offset));
        }
        Ok(())
    }
}

fn in_ranges(cp: u32, table: &[(u32, u32)]) -> bool {
    table
        .binary_search_by(|&(lo, hi)| {
            if hi < cp {
                std::cmp::Ordering::Less
            } else if lo > cp {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        })
        .is_ok()
}

/// `str.isidentifier`.
pub fn is_identifier(name: &[u32]) -> bool {
    let Some((&first, rest)) = name.split_first() else {
        return false;
    };
    (first == u32::from('_') || in_ranges(first, pycase::XID_START))
        && rest.iter().all(|&c| in_ranges(c, pycase::XID_CONTINUE))
}

fn mapped(cp: u32, table: &[(u32, u32)]) -> u32 {
    match table.binary_search_by_key(&cp, |&(k, _)| k) {
        Ok(i) => table[i].1,
        Err(_) => cp,
    }
}

/// `_sre.unicode_tolower`.
pub fn lower(cp: u32) -> u32 {
    mapped(cp, pycase::SRE_LOWER)
}

/// `sre_upper_unicode`.
pub fn upper(cp: u32) -> u32 {
    mapped(cp, pycase::SRE_UPPER)
}

fn iscased(cp: u32) -> bool {
    lower(cp) != cp || upper(cp) != cp
}

fn ascii_lower(cp: u32) -> u32 {
    if (u32::from('A')..=u32::from('Z')).contains(&cp) {
        cp + 32
    } else {
        cp
    }
}

fn ascii_iscased(cp: u32) -> bool {
    cp < 128 && char::from_u32(cp).is_some_and(|c| c.is_ascii_alphabetic())
}

fn extra_cases(cp: u32) -> Option<&'static [u32]> {
    pycase::EXTRA_CASES
        .binary_search_by_key(&cp, |&(k, _)| k)
        .ok()
        .map(|i| pycase::EXTRA_CASES[i].1)
}

/// `str.lower()`, with the final-sigma rule.
pub fn str_lower(points: &[u32]) -> Vec<u32> {
    let mut out = Vec::with_capacity(points.len());
    for (i, &cp) in points.iter().enumerate() {
        if cp == 0x3A3 {
            out.push(if final_sigma(points, i) { 0x3C2 } else { 0x3C3 });
            continue;
        }
        match pycase::LOWER_MULTI.binary_search_by_key(&cp, |&(k, _)| k) {
            Ok(j) => out.extend_from_slice(pycase::LOWER_MULTI[j].1),
            Err(_) => out.push(lower(cp)),
        }
    }
    out
}

fn final_sigma(points: &[u32], i: usize) -> bool {
    let ignorable = |c: u32| in_ranges(c, pycase::CASE_IGNORABLE);
    let cased = |c: u32| in_ranges(c, pycase::CASED);
    let before = points[..i].iter().rev().find(|&&c| !ignorable(c));
    if !before.is_some_and(|&c| cased(c)) {
        return false;
    }
    !points[i + 1..]
        .iter()
        .find(|&&c| !ignorable(c))
        .is_some_and(|&c| cased(c))
}

struct State {
    flags: u32,
    groupdict: HashMap<Vec<u32>, usize>,
    groupwidths: Vec<Option<(u128, u128)>>,
    lookbehindgroups: Option<usize>,
    grouprefpos: Vec<(usize, usize)>,
    depth: usize,
}

impl State {
    fn groups(&self) -> usize {
        self.groupwidths.len()
    }

    fn opengroup(&mut self, name: Option<&[u32]>) -> Result<usize, Vec<u32>> {
        let gid = self.groups();
        self.groupwidths.push(None);
        if self.groups() > MAXGROUPS {
            return Err(pts("too many groups"));
        }
        if let Some(name) = name {
            if let Some(&ogid) = self.groupdict.get(name) {
                return Err(pts(&format!(
                    "redefinition of group name {} as group {gid}; was group {ogid}",
                    repr_points(name)
                )));
            }
            self.groupdict.insert(name.to_vec(), gid);
        }
        Ok(gid)
    }

    fn checkgroup(&self, gid: usize) -> bool {
        gid < self.groups() && self.groupwidths[gid].is_some()
    }

    fn checklookbehindgroup(&self, gid: usize, source: &Tokenizer) -> PResult<()> {
        if let Some(lookbehind) = self.lookbehindgroups {
            if !self.checkgroup(gid) {
                return Err(source.error(pts("cannot refer to an open group"), 0));
            }
            if gid >= lookbehind {
                return Err(source.error(
                    pts("cannot refer to group defined in the same lookbehind subpattern"),
                    0,
                ));
            }
        }
        Ok(())
    }
}

fn getwidth(sub: &[Node], widths: &[Option<(u128, u128)>]) -> (u128, u128) {
    let mut lo: u128 = 0;
    let mut hi: u128 = 0;
    for node in sub {
        match node {
            Node::Branch(items) => {
                let mut i = MAXWIDTH;
                let mut j = 0;
                for item in items {
                    let (l, h) = getwidth(item, widths);
                    i = i.min(l);
                    j = j.max(h);
                }
                lo += i;
                hi += j;
            }
            Node::Atomic(p) | Node::Subpattern(_, _, _, p) => {
                let (i, j) = getwidth(p, widths);
                lo += i;
                hi += j;
            }
            Node::Repeat(_, min, max, p) => {
                let (i, j) = getwidth(p, widths);
                lo += i * u128::from(*min);
                if *max == MAXREPEAT && j > 0 {
                    hi = MAXWIDTH;
                } else {
                    hi += j * u128::from(*max);
                }
            }
            Node::Any | Node::In(_) | Node::Literal(_) | Node::NotLiteral(_) => {
                lo += 1;
                hi += 1;
            }
            Node::GroupRef(g) => {
                let (i, j) = widths[*g].unwrap_or((0, 0));
                lo += i;
                hi += j;
            }
            Node::GroupRefExists(_, yes, no) => {
                let (mut i, mut j) = getwidth(yes, widths);
                match no {
                    Some(no) => {
                        let (l, h) = getwidth(no, widths);
                        i = i.min(l);
                        j = j.max(h);
                    }
                    None => i = 0,
                }
                lo += i;
                hi += j;
            }
            Node::At(_) | Node::Assert(..) | Node::Failure => {}
        }
        // Python's integers do not overflow; the sums stay far below u128's range.
        lo = lo.min(MAXWIDTH * 4);
        hi = hi.min(MAXWIDTH * 4);
    }
    (lo.min(MAXWIDTH), hi.min(MAXWIDTH))
}

fn escape_lookup(t: Tok) -> Option<u32> {
    let e = t.escaped?;
    Some(match char::from_u32(e)? {
        'a' => 7,
        'b' => 8,
        'f' => 12,
        'n' => 10,
        'r' => 13,
        't' => 9,
        'v' => 11,
        '\\' => 92,
        _ => return None,
    })
}

enum Esc {
    Node(Node),
    In(Cat),
}

fn category_lookup(t: Tok) -> Option<Esc> {
    let e = t.escaped?;
    Some(match char::from_u32(e)? {
        'A' => Esc::Node(Node::At(At::BeginningString)),
        'b' => Esc::Node(Node::At(At::Boundary)),
        'B' => Esc::Node(Node::At(At::NonBoundary)),
        'd' => Esc::In(Cat::Digit),
        'D' => Esc::In(Cat::NotDigit),
        's' => Esc::In(Cat::Space),
        'S' => Esc::In(Cat::NotSpace),
        'w' => Esc::In(Cat::Word),
        'W' => Esc::In(Cat::NotWord),
        'z' | 'Z' => Esc::Node(Node::At(At::EndString)),
        _ => return None,
    })
}

fn hex_value(digits: &[u32]) -> u32 {
    digits.iter().fold(0, |v, &d| {
        v * 16 + char::from_u32(d).and_then(|c| c.to_digit(16)).unwrap_or(0)
    })
}

fn octal_value(digits: &[u32]) -> u32 {
    digits.iter().fold(0, |v, &d| v * 8 + (d - u32::from('0')))
}

fn bad_escape(source: &Tokenizer, escape: &[u32]) -> CompileError {
    let mut msg = pts("bad escape ");
    msg.extend_from_slice(escape);
    source.error(msg, escape.len())
}

/// `\x`, `\u`, `\U` and `\N` as both escape readers handle them; None for other escapes.
fn unicode_escape(source: &mut Tokenizer, escape: &mut Vec<u32>) -> PResult<Option<u32>> {
    let incomplete = |source: &Tokenizer, escape: &[u32]| {
        let mut msg = pts("incomplete escape ");
        msg.extend_from_slice(escape);
        source.error(msg, escape.len())
    };
    let c = escape[1];
    let digits = match char::from_u32(c) {
        Some('x') => 2,
        Some('u') => 4,
        Some('U') => 8,
        Some('N') => {
            if !source.matches('{')? {
                return Err(source.error(pts("missing {"), 0));
            }
            let charname = source.getuntil('}', "character name")?;
            // shortcut: no Unicode name database, so every name is refused as undefined.
            // Add a name table if a pattern needs a named escape.
            let mut msg = pts("undefined character name ");
            msg.extend(pts(&repr_points(&charname)));
            return Err(source.error(msg, charname.len() + 4));
        }
        _ => return Ok(None),
    };
    escape.extend(source.getwhile(digits, is_hex)?);
    if escape.len() != digits + 2 {
        return Err(incomplete(source, escape));
    }
    let value = hex_value(&escape[2..]);
    if value > 0x10FFFF {
        return Err(bad_escape(source, escape));
    }
    Ok(Some(value))
}

/// `_class_escape`: an escape inside a character set.
fn class_escape(source: &mut Tokenizer, this: Tok) -> PResult<Esc> {
    if let Some(code) = escape_lookup(this) {
        return Ok(Esc::Node(Node::Literal(code)));
    }
    if let Some(Esc::In(cat)) = category_lookup(this) {
        return Ok(Esc::In(cat));
    }
    let mut escape = this.points();
    if let Some(value) = unicode_escape(source, &mut escape)? {
        return Ok(Esc::Node(Node::Literal(value)));
    }
    let c = escape[1];
    if (u32::from('0')..=u32::from('7')).contains(&c) {
        escape.extend(source.getwhile(2, is_octal)?);
        let value = octal_value(&escape[1..]);
        if value > 0o377 {
            let mut msg = pts("octal escape value ");
            msg.extend_from_slice(&escape);
            msg.extend(pts(" outside of range 0-0o377"));
            return Err(source.error(msg, escape.len()));
        }
        return Ok(Esc::Node(Node::Literal(value)));
    }
    if c == u32::from('8') || c == u32::from('9') {
        return Err(bad_escape(source, &escape));
    }
    if escape.len() == 2 {
        if char::from_u32(c).is_some_and(|c| c.is_ascii_alphabetic()) {
            return Err(bad_escape(source, &escape));
        }
        return Ok(Esc::Node(Node::Literal(c)));
    }
    Err(bad_escape(source, &escape))
}

/// `_escape`: an escape outside a character set.
fn escape(source: &mut Tokenizer, this: Tok, state: &State) -> PResult<Esc> {
    if let Some(code) = category_lookup(this) {
        return Ok(code);
    }
    if let Some(code) = escape_lookup(this) {
        return Ok(Esc::Node(Node::Literal(code)));
    }
    let mut escape = this.points();
    if let Some(value) = unicode_escape(source, &mut escape)? {
        return Ok(Esc::Node(Node::Literal(value)));
    }
    let c = escape[1];
    if c == u32::from('0') {
        escape.extend(source.getwhile(2, is_octal)?);
        return Ok(Esc::Node(Node::Literal(octal_value(&escape[1..]))));
    }
    if (u32::from('1')..=u32::from('9')).contains(&c) {
        if is_digit(source.next) {
            escape.extend(source.get()?.map(|t| t.points()).unwrap_or_default());
            let oct = |d: u32| (u32::from('0')..=u32::from('7')).contains(&d);
            if oct(escape[1]) && oct(escape[2]) && is_octal(source.next) {
                escape.extend(source.get()?.map(|t| t.points()).unwrap_or_default());
                let value = octal_value(&escape[1..]);
                if value > 0o377 {
                    let mut msg = pts("octal escape value ");
                    msg.extend_from_slice(&escape);
                    msg.extend(pts(" outside of range 0-0o377"));
                    return Err(source.error(msg, escape.len()));
                }
                return Ok(Esc::Node(Node::Literal(value)));
            }
        }
        let group = escape[1..]
            .iter()
            .fold(0usize, |v, &d| v * 10 + (d - u32::from('0')) as usize);
        if group < state.groups() {
            if !state.checkgroup(group) {
                return Err(source.error(pts("cannot refer to an open group"), escape.len()));
            }
            state.checklookbehindgroup(group, source)?;
            return Ok(Esc::Node(Node::GroupRef(group)));
        }
        return Err(source.error(
            pts(&format!("invalid group reference {group}")),
            escape.len() - 1,
        ));
    }
    if escape.len() == 2 {
        if char::from_u32(c).is_some_and(|c| c.is_ascii_alphabetic()) {
            return Err(bad_escape(source, &escape));
        }
        return Ok(Esc::Node(Node::Literal(c)));
    }
    Err(bad_escape(source, &escape))
}

fn esc_node(code: Esc) -> Node {
    match code {
        Esc::Node(node) => node,
        Esc::In(cat) => Node::In(vec![Item::Category(cat)]),
    }
}

fn uniq(items: Vec<Item>) -> Vec<Item> {
    let mut out: Vec<Item> = Vec::with_capacity(items.len());
    for item in items {
        if !out.contains(&item) {
            out.push(item);
        }
    }
    out
}

/// `int(digits)` for an ASCII digit run, with `int`'s 4,300-digit limit.
fn int_of(digits: &[u32]) -> PResult<u128> {
    if digits.len() > py::INT_MAX_STR_DIGITS {
        return Err(CompileError::Other(
            "ValueError",
            py::int_digits_error(digits.len()),
        ));
    }
    let mut value: u128 = 0;
    for &d in digits {
        value = value
            .saturating_mul(10)
            .saturating_add(u128::from(d - u32::from('0')));
    }
    Ok(value)
}

fn parse_sub(
    source: &mut Tokenizer,
    state: &mut State,
    verbose: bool,
    nested: usize,
) -> PResult<Sub> {
    state.depth += 1;
    if state.depth > 2 * MAX_NESTING {
        return Err(CompileError::Other(
            "RecursionError",
            "maximum recursion depth exceeded".into(),
        ));
    }
    let result = parse_sub_inner(source, state, verbose, nested);
    state.depth -= 1;
    result
}

fn parse_sub_inner(
    source: &mut Tokenizer,
    state: &mut State,
    mut verbose: bool,
    nested: usize,
) -> PResult<Sub> {
    let mut items: Vec<Sub> = Vec::new();
    loop {
        let first = nested == 0 && items.is_empty();
        items.push(parse(source, state, verbose, nested + 1, first)?);
        if !source.matches('|')? {
            break;
        }
        if nested == 0 {
            verbose = state.flags & VERBOSE != 0;
        }
    }
    if items.len() == 1 {
        return Ok(items.pop().unwrap_or_default());
    }
    let mut subpattern: Sub = Vec::new();
    // Move a prefix every alternative shares out of the branch.
    while let Some(prefix) = items[0].first().cloned() {
        if items[1..]
            .iter()
            .any(|item| item.first().is_none_or(|first| !node_eq(first, &prefix)))
        {
            break;
        }
        for item in &mut items {
            item.remove(0);
        }
        subpattern.push(prefix);
    }
    // A branch of single characters is a character set.
    let mut set = Vec::new();
    let mut all = true;
    for item in &items {
        if item.len() != 1 {
            all = false;
            break;
        }
        match &item[0] {
            Node::Literal(c) => set.push(Item::Literal(*c)),
            Node::In(av) if av[0] != Item::Negate => set.extend(av.iter().copied()),
            _ => {
                all = false;
                break;
            }
        }
    }
    if all {
        subpattern.push(Node::In(uniq(set)));
        return Ok(subpattern);
    }
    subpattern.push(Node::Branch(items));
    Ok(subpattern)
}

fn parse(
    source: &mut Tokenizer,
    state: &mut State,
    verbose: bool,
    nested: usize,
    first: bool,
) -> PResult<Sub> {
    state.depth += 1;
    if state.depth > 2 * MAX_NESTING {
        return Err(CompileError::Other(
            "RecursionError",
            "maximum recursion depth exceeded".into(),
        ));
    }
    let result = parse_inner(source, state, verbose, nested, first);
    state.depth -= 1;
    result
}

#[allow(clippy::too_many_lines)]
fn parse_inner(
    source: &mut Tokenizer,
    state: &mut State,
    mut verbose: bool,
    nested: usize,
    first: bool,
) -> PResult<Sub> {
    let mut subpattern: Sub = Vec::new();
    while let Some(this) = source.next {
        if this.is('|') || this.is(')') {
            break;
        }
        source.get()?;
        if verbose {
            if this.among(" \t\n\r\x0b\x0c") {
                continue;
            }
            if this.is('#') {
                loop {
                    match source.get()? {
                        None => break,
                        Some(t) if t.is('\n') => break,
                        _ => {}
                    }
                }
                continue;
            }
        }
        if this.escaped.is_some() {
            let code = escape(source, this, state)?;
            subpattern.push(esc_node(code));
        } else if !this.among(".\\[{()*+?^$|") {
            subpattern.push(Node::Literal(this.c));
        } else if this.is('[') {
            let here = source.tell() - 1;
            let mut set: Vec<Item> = Vec::new();
            let negate = source.matches('^')?;
            loop {
                let Some(this) = source.get()? else {
                    return Err(
                        source.error(pts("unterminated character set"), source.tell() - here)
                    );
                };
                if this.is(']') && !set.is_empty() {
                    break;
                }
                let code1 = if this.escaped.is_some() {
                    class_escape(source, this)?
                } else {
                    Esc::Node(Node::Literal(this.c))
                };
                if source.matches('-')? {
                    let Some(that) = source.get()? else {
                        return Err(
                            source.error(pts("unterminated character set"), source.tell() - here)
                        );
                    };
                    if that.is(']') {
                        set.push(esc_item(&code1));
                        set.push(Item::Literal(u32::from('-')));
                        break;
                    }
                    let code2 = if that.escaped.is_some() {
                        class_escape(source, that)?
                    } else {
                        Esc::Node(Node::Literal(that.c))
                    };
                    let bad = || {
                        let mut msg = pts("bad character range ");
                        msg.extend(this.points());
                        msg.push(u32::from('-'));
                        msg.extend(that.points());
                        source.error(msg, this.len() + 1 + that.len())
                    };
                    let (Esc::Node(Node::Literal(lo)), Esc::Node(Node::Literal(hi))) =
                        (&code1, &code2)
                    else {
                        return Err(bad());
                    };
                    if hi < lo {
                        return Err(bad());
                    }
                    set.push(Item::Range(*lo, *hi));
                } else {
                    set.push(esc_item(&code1));
                }
            }
            let set = uniq(set);
            if let [Item::Literal(c)] = set[..] {
                subpattern.push(if negate {
                    Node::NotLiteral(c)
                } else {
                    Node::Literal(c)
                });
            } else {
                let mut set = set;
                if negate {
                    set.insert(0, Item::Negate);
                }
                subpattern.push(Node::In(set));
            }
        } else if this.among("*+?{") {
            let here = source.tell();
            let (min, max): (u64, u64);
            if this.is('?') {
                (min, max) = (0, 1);
            } else if this.is('*') {
                (min, max) = (0, MAXREPEAT);
            } else if this.is('+') {
                (min, max) = (1, MAXREPEAT);
            } else {
                if source.next.is_some_and(|t| t.is('}')) {
                    subpattern.push(Node::Literal(this.c));
                    continue;
                }
                let mut lo = Vec::new();
                let mut hi = Vec::new();
                while is_digit(source.next) {
                    lo.extend(source.get()?.map(|t| t.points()).unwrap_or_default());
                }
                if source.matches(',')? {
                    while is_digit(source.next) {
                        hi.extend(source.get()?.map(|t| t.points()).unwrap_or_default());
                    }
                } else {
                    hi.clone_from(&lo);
                }
                if !source.matches('}')? {
                    subpattern.push(Node::Literal(this.c));
                    source.seek(here)?;
                    continue;
                }
                let mut lo_value = 0;
                let mut hi_value = MAXREPEAT;
                if !lo.is_empty() {
                    let v = int_of(&lo)?;
                    if v >= u128::from(MAXREPEAT) {
                        return Err(CompileError::Other(
                            "OverflowError",
                            "the repetition number is too large".into(),
                        ));
                    }
                    lo_value = v as u64;
                }
                if !hi.is_empty() {
                    let v = int_of(&hi)?;
                    if v >= u128::from(MAXREPEAT) {
                        return Err(CompileError::Other(
                            "OverflowError",
                            "the repetition number is too large".into(),
                        ));
                    }
                    hi_value = v as u64;
                    if hi_value < lo_value {
                        return Err(source.error(
                            pts("min repeat greater than max repeat"),
                            source.tell() - here,
                        ));
                    }
                }
                (min, max) = (lo_value, hi_value);
            }
            let offset = source.tell() - here + this.len();
            let Some(last) = subpattern.last() else {
                return Err(source.error(pts("nothing to repeat"), offset));
            };
            if matches!(last, Node::At(_)) {
                return Err(source.error(pts("nothing to repeat"), offset));
            }
            if matches!(last, Node::Repeat(..)) {
                return Err(source.error(pts("multiple repeat"), offset));
            }
            let last = subpattern.pop().unwrap_or(Node::Failure);
            let item = match last {
                Node::Subpattern(None, 0, 0, p) => p,
                other => vec![other],
            };
            let rep = if source.matches('?')? {
                Rep::Min
            } else if source.matches('+')? {
                Rep::Possessive
            } else {
                Rep::Max
            };
            subpattern.push(Node::Repeat(rep, min, max, item));
        } else if this.is('.') {
            subpattern.push(Node::Any);
        } else if this.is('(') {
            let start = source.tell() - 1;
            let mut capture = true;
            let mut atomic = false;
            let mut name: Option<Vec<u32>> = None;
            let mut add_flags = 0;
            let mut del_flags = 0;
            if source.matches('?')? {
                let Some(char) = source.get()? else {
                    return Err(source.error(pts("unexpected end of pattern"), 0));
                };
                if char.is('P') {
                    if source.matches('<')? {
                        let n = source.getuntil('>', "group name")?;
                        source.checkgroupname(&n, 1)?;
                        name = Some(n);
                    } else if source.matches('=')? {
                        let n = source.getuntil(')', "group name")?;
                        source.checkgroupname(&n, 1)?;
                        let Some(&gid) = state.groupdict.get(&n) else {
                            let mut msg = pts("unknown group name ");
                            msg.extend(pts(&repr_points(&n)));
                            return Err(source.error(msg, n.len() + 1));
                        };
                        if !state.checkgroup(gid) {
                            return Err(
                                source.error(pts("cannot refer to an open group"), n.len() + 1)
                            );
                        }
                        state.checklookbehindgroup(gid, source)?;
                        subpattern.push(Node::GroupRef(gid));
                        continue;
                    } else {
                        let Some(char) = source.get()? else {
                            return Err(source.error(pts("unexpected end of pattern"), 0));
                        };
                        let mut msg = pts("unknown extension ?P");
                        msg.extend(char.points());
                        return Err(source.error(msg, char.len() + 2));
                    }
                } else if char.is(':') {
                    capture = false;
                } else if char.is('#') {
                    loop {
                        if source.next.is_none() {
                            return Err(source.error(
                                pts("missing ), unterminated comment"),
                                source.tell() - start,
                            ));
                        }
                        if source.get()?.is_some_and(|t| t.is(')')) {
                            break;
                        }
                    }
                    continue;
                } else if char.among("=!<") {
                    let mut dir = 1;
                    let mut char = char;
                    let mut lookbehindgroups = None;
                    if char.is('<') {
                        let Some(next) = source.get()? else {
                            return Err(source.error(pts("unexpected end of pattern"), 0));
                        };
                        char = next;
                        if !char.among("=!") {
                            let mut msg = pts("unknown extension ?<");
                            msg.extend(char.points());
                            return Err(source.error(msg, char.len() + 2));
                        }
                        dir = -1;
                        lookbehindgroups = state.lookbehindgroups;
                        if lookbehindgroups.is_none() {
                            state.lookbehindgroups = Some(state.groups());
                        }
                    }
                    let p = parse_sub(source, state, verbose, nested + 1)?;
                    if dir < 0 && lookbehindgroups.is_none() {
                        state.lookbehindgroups = None;
                    }
                    if !source.matches(')')? {
                        return Err(source.error(
                            pts("missing ), unterminated subpattern"),
                            source.tell() - start,
                        ));
                    }
                    if char.is('=') {
                        subpattern.push(Node::Assert(false, dir, p));
                    } else if !p.is_empty() {
                        subpattern.push(Node::Assert(true, dir, p));
                    } else {
                        subpattern.push(Node::Failure);
                    }
                    continue;
                } else if char.is('(') {
                    let condname = source.getuntil(')', "group name")?;
                    let ascii_decimal = condname
                        .iter()
                        .all(|&c| (u32::from('0')..=u32::from('9')).contains(&c));
                    let condgroup;
                    if !ascii_decimal {
                        source.checkgroupname(&condname, 1)?;
                        let Some(&gid) = state.groupdict.get(&condname) else {
                            let mut msg = pts("unknown group name ");
                            msg.extend(pts(&repr_points(&condname)));
                            return Err(source.error(msg, condname.len() + 1));
                        };
                        condgroup = gid;
                    } else {
                        let value = int_of(&condname)?;
                        if value == 0 {
                            return Err(source.error(pts("bad group number"), condname.len() + 1));
                        }
                        if value >= MAXGROUPS as u128 {
                            let digits: String = condname
                                .iter()
                                .filter_map(|&c| char::from_u32(c))
                                .collect::<String>()
                                .trim_start_matches('0')
                                .to_string();
                            return Err(source.error(
                                pts(&format!("invalid group reference {digits}")),
                                condname.len() + 1,
                            ));
                        }
                        condgroup = value as usize;
                        if !state.grouprefpos.iter().any(|&(g, _)| g == condgroup) {
                            state
                                .grouprefpos
                                .push((condgroup, source.tell() - condname.len() - 1));
                        }
                    }
                    state.checklookbehindgroup(condgroup, source)?;
                    let item_yes = parse(source, state, verbose, nested + 1, false)?;
                    let item_no = if source.matches('|')? {
                        let no = parse(source, state, verbose, nested + 1, false)?;
                        if source.next.is_some_and(|t| t.is('|')) {
                            return Err(source
                                .error(pts("conditional backref with more than two branches"), 0));
                        }
                        Some(no)
                    } else {
                        None
                    };
                    if !source.matches(')')? {
                        return Err(source.error(
                            pts("missing ), unterminated subpattern"),
                            source.tell() - start,
                        ));
                    }
                    subpattern.push(Node::GroupRefExists(condgroup, item_yes, item_no));
                    continue;
                } else if char.is('>') {
                    capture = false;
                    atomic = true;
                } else if char.among("iLmsxau-") {
                    match parse_flags(source, state, char)? {
                        None => {
                            if !first || !subpattern.is_empty() {
                                return Err(source.error(
                                    pts("global flags not at the start of the expression"),
                                    source.tell() - start,
                                ));
                            }
                            verbose = state.flags & VERBOSE != 0;
                            continue;
                        }
                        Some((add, del)) => {
                            add_flags = add;
                            del_flags = del;
                            capture = false;
                        }
                    }
                } else {
                    let mut msg = pts("unknown extension ?");
                    msg.extend(char.points());
                    return Err(source.error(msg, char.len() + 1));
                }
            }
            let group = if capture {
                match state.opengroup(name.as_deref()) {
                    Ok(gid) => Some(gid),
                    Err(msg) => {
                        let offset = name.as_ref().map_or(0, Vec::len) + 1;
                        return Err(source.error(msg, offset));
                    }
                }
            } else {
                None
            };
            let sub_verbose = (verbose || add_flags & VERBOSE != 0) && del_flags & VERBOSE == 0;
            let p = parse_sub(source, state, sub_verbose, nested + 1)?;
            if !source.matches(')')? {
                return Err(source.error(
                    pts("missing ), unterminated subpattern"),
                    source.tell() - start,
                ));
            }
            if let Some(gid) = group {
                state.groupwidths[gid] = Some(getwidth(&p, &state.groupwidths));
            }
            if atomic {
                subpattern.push(Node::Atomic(p));
            } else {
                subpattern.push(Node::Subpattern(group, add_flags, del_flags, p));
            }
        } else if this.is('^') {
            subpattern.push(Node::At(At::Beginning));
        } else if this.is('$') {
            subpattern.push(Node::At(At::End));
        }
    }
    // Unpack non-capturing groups without flags.
    let mut out = Vec::with_capacity(subpattern.len());
    for node in subpattern {
        match node {
            Node::Subpattern(None, 0, 0, p) => out.extend(p),
            other => out.push(other),
        }
    }
    Ok(out)
}

fn esc_item(code: &Esc) -> Item {
    match code {
        Esc::In(cat) => Item::Category(*cat),
        Esc::Node(Node::Literal(c)) => Item::Literal(*c),
        // class_escape returns only literals and categories.
        Esc::Node(_) => Item::Negate,
    }
}

fn flag_bit(c: u32) -> Option<u32> {
    Some(match char::from_u32(c)? {
        'i' => IGNORECASE,
        'L' => LOCALE,
        'm' => MULTILINE,
        's' => DOTALL,
        'x' => VERBOSE,
        'a' => ASCII,
        'u' => UNICODE,
        _ => return None,
    })
}

fn is_alpha_tok(t: Tok) -> bool {
    t.escaped.is_none() && py::is_alpha(t.c)
}

fn parse_flags(
    source: &mut Tokenizer,
    state: &mut State,
    char: Tok,
) -> PResult<Option<(u32, u32)>> {
    let mut add_flags = 0;
    let mut del_flags = 0;
    let mut char = char;
    if !char.is('-') {
        loop {
            let flag = flag_bit(char.c).unwrap_or(0);
            if char.is('L') {
                return Err(source.error(
                    pts("bad inline flags: cannot use 'L' flag with a str pattern"),
                    0,
                ));
            }
            add_flags |= flag;
            if flag & TYPE_FLAGS != 0 && add_flags & TYPE_FLAGS != flag {
                return Err(source.error(
                    pts("bad inline flags: flags 'a', 'u' and 'L' are incompatible"),
                    0,
                ));
            }
            let Some(next) = source.get()? else {
                return Err(source.error(pts("missing -, : or )"), 0));
            };
            char = next;
            if char.among(")-:") {
                break;
            }
            if char.escaped.is_some() || flag_bit(char.c).is_none() {
                let msg = if is_alpha_tok(char) {
                    "unknown flag"
                } else {
                    "missing -, : or )"
                };
                return Err(source.error(pts(msg), char.len()));
            }
        }
    }
    if char.is(')') {
        state.flags |= add_flags;
        return Ok(None);
    }
    if add_flags & DEBUG != 0 {
        return Err(source.error(pts("bad inline flags: cannot turn on global flag"), 1));
    }
    if char.is('-') {
        let Some(next) = source.get()? else {
            return Err(source.error(pts("missing flag"), 0));
        };
        char = next;
        if char.escaped.is_some() || flag_bit(char.c).is_none() {
            let msg = if is_alpha_tok(char) {
                "unknown flag"
            } else {
                "missing flag"
            };
            return Err(source.error(pts(msg), char.len()));
        }
        loop {
            let flag = flag_bit(char.c).unwrap_or(0);
            if flag & TYPE_FLAGS != 0 {
                return Err(source.error(
                    pts("bad inline flags: cannot turn off flags 'a', 'u' and 'L'"),
                    0,
                ));
            }
            del_flags |= flag;
            let Some(next) = source.get()? else {
                return Err(source.error(pts("missing :"), 0));
            };
            char = next;
            if char.is(':') {
                break;
            }
            if char.escaped.is_some() || flag_bit(char.c).is_none() {
                let msg = if is_alpha_tok(char) {
                    "unknown flag"
                } else {
                    "missing :"
                };
                return Err(source.error(pts(msg), char.len()));
            }
        }
    }
    if del_flags & DEBUG != 0 {
        return Err(source.error(pts("bad inline flags: cannot turn off global flag"), 1));
    }
    if add_flags & del_flags != 0 {
        return Err(source.error(pts("bad inline flags: flag turned on and off"), 1));
    }
    Ok(Some((add_flags, del_flags)))
}

// ---------------------------------------------------------------------------
// The compiled form: each character test and anchor resolved for its flags.

#[derive(Clone, Copy, Debug)]
enum Probe {
    Raw,
    Lower,
    AsciiLower,
}

impl Probe {
    fn apply(self, cp: u32) -> u32 {
        match self {
            Probe::Raw => cp,
            Probe::Lower => lower(cp),
            Probe::AsciiLower => ascii_lower(cp),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Tail {
    Literal(u32),
    Range(u32, u32),
    RangeUniIgnore(u32, u32),
    Category(Cat),
}

#[derive(Clone, Debug)]
struct Set {
    negate: bool,
    probe: Probe,
    ascii: bool,
    /// Sorted members below 0x10000.
    chars: Vec<u32>,
    tail: Vec<Tail>,
}

#[derive(Clone, Debug)]
enum Test {
    /// `ch == c` (or `!=` when negated).
    Exact(u32, bool),
    /// `probe(ch) == lo` (or `!=`).
    Probed(Probe, u32, bool),
    Set(Box<Set>),
    Any,
    AnyAll,
    Fail,
}

#[derive(Clone, Copy, Debug)]
enum Anchor {
    Beginning,
    BeginningLine,
    End,
    EndLine,
    EndString,
    Boundary(bool),
    NonBoundary(bool),
}

#[derive(Clone, Debug)]
enum M {
    Char(Test),
    Branch(Vec<Vec<M>>),
    Group(usize, Vec<M>),
    Repeat(Rep, u64, u64, Vec<M>),
    At(Anchor),
    Ref(usize, Probe),
    RefExists(usize, Vec<M>, Vec<M>),
    Assert {
        negative: bool,
        back: Option<usize>,
        body: Vec<M>,
    },
    Atomic(Vec<M>),
}

fn category(cat: Cat, ascii: bool, cp: u32) -> bool {
    let c = char::from_u32(cp);
    let digit = || {
        if ascii {
            (u32::from('0')..=u32::from('9')).contains(&cp)
        } else {
            c.is_some_and(py::is_decimal)
        }
    };
    let space = || {
        if ascii {
            matches!(cp, 9..=13 | 32)
        } else {
            c.is_some_and(py::is_space)
        }
    };
    match cat {
        Cat::Digit => digit(),
        Cat::NotDigit => !digit(),
        Cat::Space => space(),
        Cat::NotSpace => !space(),
        Cat::Word => is_word(cp, ascii),
        Cat::NotWord => !is_word(cp, ascii),
    }
}

fn is_word(cp: u32, ascii: bool) -> bool {
    if ascii {
        cp < 128 && char::from_u32(cp).is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
    } else {
        char::from_u32(cp).is_some_and(py::is_word)
    }
}

impl Set {
    fn contains(&self, ch: u32) -> bool {
        let probe = self.probe.apply(ch);
        let hit = (probe < 0x10000 && self.chars.binary_search(&probe).is_ok())
            || self.tail.iter().any(|t| match *t {
                Tail::Literal(c) => probe == c,
                Tail::Range(lo, hi) => (lo..=hi).contains(&probe),
                Tail::RangeUniIgnore(lo, hi) => {
                    (lo..=hi).contains(&probe) || (lo..=hi).contains(&upper(probe))
                }
                Tail::Category(cat) => category(cat, self.ascii, probe),
            });
        hit != self.negate
    }
}

impl Test {
    fn matches(&self, ch: u32) -> bool {
        match self {
            Test::Exact(c, negated) => (ch == *c) != *negated,
            Test::Probed(probe, lo, negated) => (probe.apply(ch) == *lo) != *negated,
            Test::Set(set) => set.contains(ch),
            Test::Any => ch != u32::from('\n'),
            Test::AnyAll => true,
            Test::Fail => false,
        }
    }
}

/// `_optimize_charset` and the `IN` opcode choice.
fn compile_set(items: &[Item], flags: u32) -> Test {
    let ignore = flags & IGNORECASE != 0;
    let unicode = flags & UNICODE != 0;
    let ascii = !unicode;
    let fixup = |c: u32| if unicode { lower(c) } else { ascii_lower(c) };
    let cased = |c: u32| {
        if unicode {
            iscased(c)
        } else {
            ascii_iscased(c)
        }
    };
    let mut negate = false;
    let mut chars: Vec<u32> = Vec::new();
    let mut tail: Vec<Tail> = Vec::new();
    let mut hascased = false;
    let add = |chars: &mut Vec<u32>, c: u32| -> bool {
        if c >= 0x10000 {
            return false;
        }
        chars.push(c);
        if ignore
            && unicode
            && let Some(fixes) = extra_cases(c)
        {
            chars.extend_from_slice(fixes);
        }
        true
    };
    for item in items {
        match *item {
            Item::Literal(c) => {
                if ignore {
                    let lo = fixup(c);
                    if !add(&mut chars, lo) {
                        tail.push(Tail::Literal(lo));
                    }
                    if !hascased && cased(lo) {
                        hascased = true;
                    }
                } else if c < 0x10000 {
                    chars.push(c);
                } else {
                    tail.push(Tail::Literal(c));
                }
            }
            Item::Range(lo, hi) => {
                if ignore {
                    let mut beyond = false;
                    for c in lo..=hi {
                        if !add(&mut chars, fixup(c)) {
                            beyond = true;
                            break;
                        }
                    }
                    if beyond {
                        tail.push(if unicode {
                            Tail::RangeUniIgnore(lo, hi)
                        } else {
                            Tail::Range(lo, hi)
                        });
                        hascased = true;
                    } else if !hascased {
                        hascased = (lo..=hi).any(cased);
                    }
                } else {
                    let top = hi.min(0xFFFF);
                    if lo <= top {
                        chars.extend(lo..=top);
                    }
                    if hi >= 0x10000 {
                        tail.push(Tail::Range(lo, hi));
                    }
                }
            }
            Item::Negate => negate = true,
            Item::Category(cat) => {
                if tail
                    .iter()
                    .any(|t| matches!(t, Tail::Category(other) if *other == cat.negated()))
                {
                    // [\s\S] matches anything; negated, nothing.
                    return if negate { Test::Fail } else { Test::AnyAll };
                }
                tail.push(Tail::Category(cat));
            }
        }
    }
    chars.sort_unstable();
    chars.dedup();
    let probe = if !ignore || !hascased {
        Probe::Raw
    } else if unicode {
        Probe::Lower
    } else {
        Probe::AsciiLower
    };
    Test::Set(Box::new(Set {
        negate,
        probe,
        ascii,
        chars,
        tail,
    }))
}

fn compile_literal(c: u32, negated: bool, flags: u32) -> Test {
    if flags & IGNORECASE == 0 {
        return Test::Exact(c, negated);
    }
    if flags & UNICODE == 0 {
        if !ascii_iscased(c) {
            return Test::Exact(c, negated);
        }
        return Test::Probed(Probe::AsciiLower, ascii_lower(c), negated);
    }
    if !iscased(c) {
        return Test::Exact(c, negated);
    }
    let lo = lower(c);
    match extra_cases(lo) {
        None => Test::Probed(Probe::Lower, lo, negated),
        Some(fixes) => {
            let mut chars = vec![lo];
            chars.extend_from_slice(fixes);
            chars.sort_unstable();
            Test::Set(Box::new(Set {
                negate: negated,
                probe: Probe::Lower,
                ascii: false,
                chars,
                tail: Vec::new(),
            }))
        }
    }
}

fn combine_flags(flags: u32, add: u32, del: u32) -> u32 {
    let flags = if add & TYPE_FLAGS != 0 {
        flags & !TYPE_FLAGS
    } else {
        flags
    };
    (flags | add) & !del
}

fn compile_sub(
    sub: &[Node],
    flags: u32,
    widths: &[Option<(u128, u128)>],
    out: &mut Vec<M>,
) -> PResult<()> {
    for node in sub {
        match node {
            Node::Literal(c) => out.push(M::Char(compile_literal(*c, false, flags))),
            Node::NotLiteral(c) => out.push(M::Char(compile_literal(*c, true, flags))),
            Node::In(items) => out.push(M::Char(compile_set(items, flags))),
            Node::Any => out.push(M::Char(if flags & DOTALL != 0 {
                Test::AnyAll
            } else {
                Test::Any
            })),
            Node::Repeat(rep, min, max, body) => {
                let mut compiled = Vec::new();
                compile_sub(body, flags, widths, &mut compiled)?;
                out.push(M::Repeat(*rep, *min, *max, compiled));
            }
            Node::Subpattern(group, add, del, body) => {
                let inner = combine_flags(flags, *add, *del);
                match group {
                    Some(g) => {
                        let mut compiled = Vec::new();
                        compile_sub(body, inner, widths, &mut compiled)?;
                        out.push(M::Group(*g, compiled));
                    }
                    None => compile_sub(body, inner, widths, out)?,
                }
            }
            Node::Atomic(body) => {
                let mut compiled = Vec::new();
                compile_sub(body, flags, widths, &mut compiled)?;
                out.push(M::Atomic(compiled));
            }
            Node::Failure => out.push(M::Char(Test::Fail)),
            Node::Assert(negative, dir, body) => {
                let back = if *dir >= 0 {
                    None
                } else {
                    let (lo, hi) = getwidth(body, widths);
                    if lo > MAXCODE {
                        return Err(CompileError::Pattern(pts("looks too much behind")));
                    }
                    if lo != hi {
                        return Err(CompileError::Pattern(pts(
                            "look-behind requires fixed-width pattern",
                        )));
                    }
                    Some(lo as usize)
                };
                let mut compiled = Vec::new();
                compile_sub(body, flags, widths, &mut compiled)?;
                out.push(M::Assert {
                    negative: *negative,
                    back,
                    body: compiled,
                });
            }
            Node::At(at) => {
                let multiline = flags & MULTILINE != 0;
                let unicode = flags & UNICODE != 0;
                out.push(M::At(match at {
                    At::Beginning if multiline => Anchor::BeginningLine,
                    At::Beginning | At::BeginningString => Anchor::Beginning,
                    At::End if multiline => Anchor::EndLine,
                    At::End => Anchor::End,
                    At::EndString => Anchor::EndString,
                    At::Boundary => Anchor::Boundary(!unicode),
                    At::NonBoundary => Anchor::NonBoundary(!unicode),
                }));
            }
            Node::Branch(items) => {
                let mut alternatives = Vec::with_capacity(items.len());
                for item in items {
                    let mut compiled = Vec::new();
                    compile_sub(item, flags, widths, &mut compiled)?;
                    alternatives.push(compiled);
                }
                out.push(M::Branch(alternatives));
            }
            Node::GroupRef(g) => {
                let probe = if flags & IGNORECASE == 0 {
                    Probe::Raw
                } else if flags & UNICODE == 0 {
                    Probe::AsciiLower
                } else {
                    Probe::Lower
                };
                out.push(M::Ref(*g, probe));
            }
            Node::GroupRefExists(g, yes, no) => {
                let mut y = Vec::new();
                compile_sub(yes, flags, widths, &mut y)?;
                let mut n = Vec::new();
                if let Some(no) = no {
                    compile_sub(no, flags, widths, &mut n)?;
                }
                out.push(M::RefExists(*g, y, n));
            }
        }
    }
    Ok(())
}

/// A compiled pattern: `re.compile(pattern, flags)`.
#[derive(Clone, Debug)]
pub struct Pattern {
    prog: Vec<M>,
    /// `pattern.groups`.
    pub groups: usize,
}

/// `re.compile(pattern, flags)` for a `str` pattern.
pub fn compile(pattern: &[u32], flags: u32) -> Result<Pattern, CompileError> {
    let mut source = Tokenizer::new(pattern)?;
    let mut state = State {
        flags,
        groupdict: HashMap::new(),
        groupwidths: vec![None],
        lookbehindgroups: None,
        grouprefpos: Vec::new(),
        depth: 0,
    };
    let p = parse_sub(&mut source, &mut state, flags & VERBOSE != 0, 0)?;
    let mut flags = state.flags;
    if flags & ASCII == 0 {
        flags |= UNICODE;
    } else if flags & UNICODE != 0 {
        return Err(CompileError::Other(
            "ValueError",
            "ASCII and UNICODE flags are incompatible".into(),
        ));
    }
    if source.next.is_some() {
        return Err(source.error(pts("unbalanced parenthesis"), 0));
    }
    for &(g, pos) in &state.grouprefpos {
        if g >= state.groups() {
            return Err(pattern_error(
                pts(&format!("invalid group reference {g}")),
                pattern,
                Some(pos),
            ));
        }
    }
    let mut prog = Vec::new();
    compile_sub(&p, flags, &state.groupwidths, &mut prog)?;
    Ok(Pattern {
        prog,
        groups: state.groups() - 1,
    })
}

/// `re.compile` of a pattern written in this program.
pub fn compile_str(pattern: &str, flags: u32) -> Pattern {
    match compile(&pts(pattern), flags) {
        Ok(p) => p,
        Err(e) => panic!("pattern {pattern:?}: {e:?}"),
    }
}

// ---------------------------------------------------------------------------
// Matching.

type Caps = Vec<Option<(usize, usize)>>;

/// A successful match: group 0 and the numbered groups.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Match {
    spans: Caps,
}

impl Match {
    /// `match.span(i)`, or None where group `i` did not take part.
    pub fn span(&self, i: usize) -> Option<(usize, usize)> {
        self.spans.get(i).copied().flatten()
    }

    pub fn start(&self) -> usize {
        self.span(0).map_or(0, |s| s.0)
    }

    pub fn end(&self) -> usize {
        self.span(0).map_or(0, |s| s.1)
    }
}

struct Ctx<'t> {
    text: &'t [u32],
    caps: Caps,
}

type K<'k> = &'k mut dyn FnMut(usize, &mut Ctx) -> bool;

impl Pattern {
    fn run(&self, text: &[u32], start: usize, full: bool) -> Option<Match> {
        let mut ctx = Ctx {
            text,
            caps: vec![None; self.groups + 1],
        };
        let mut end = None;
        let ok = seq(&self.prog, 0, start, &mut ctx, &mut |p, ctx| {
            if full && p != ctx.text.len() {
                return false;
            }
            end = Some(p);
            true
        });
        if !ok {
            return None;
        }
        let mut spans = ctx.caps;
        spans[0] = Some((start, end.unwrap_or(start)));
        Some(Match { spans })
    }

    /// `pattern.search(text)`.
    pub fn search(&self, text: &[u32]) -> Option<Match> {
        (0..=text.len()).find_map(|start| self.run(text, start, false))
    }

    /// `pattern.match(text)`.
    pub fn match_start(&self, text: &[u32]) -> Option<Match> {
        self.run(text, 0, false)
    }

    /// `pattern.fullmatch(text)`.
    pub fn fullmatch(&self, text: &[u32]) -> Option<Match> {
        self.run(text, 0, true)
    }
}

fn seq(nodes: &[M], i: usize, pos: usize, ctx: &mut Ctx, k: K) -> bool {
    let Some(node) = nodes.get(i) else {
        return k(pos, ctx);
    };
    if let M::Char(test) = node {
        // A run of single characters needs no continuation frame each.
        if pos < ctx.text.len() && test.matches(ctx.text[pos]) {
            return seq(nodes, i + 1, pos + 1, ctx, k);
        }
        return false;
    }
    one(node, pos, ctx, &mut |p, ctx| seq(nodes, i + 1, p, ctx, k))
}

fn at(anchor: Anchor, pos: usize, text: &[u32]) -> bool {
    let nl = u32::from('\n');
    let end = text.len();
    let boundary = |ascii: bool| {
        let before = pos > 0 && is_word(text[pos - 1], ascii);
        let after = pos < end && is_word(text[pos], ascii);
        before != after
    };
    match anchor {
        Anchor::Beginning => pos == 0,
        Anchor::BeginningLine => pos == 0 || text[pos - 1] == nl,
        Anchor::End => pos == end || (pos + 1 == end && text[pos] == nl),
        Anchor::EndLine => pos == end || text[pos] == nl,
        Anchor::EndString => pos == end,
        Anchor::Boundary(ascii) => end > 0 && boundary(ascii),
        // CPython 3.14 and later: \B matches an empty string.
        Anchor::NonBoundary(ascii) => end == 0 || !boundary(ascii),
    }
}

/// The first way `body` matches at `pos`, as `_sre` runs an assertion, an
/// atomic group or a possessive iteration: its end, with its groups kept.
fn first_end(body: &[M], pos: usize, ctx: &mut Ctx) -> Option<usize> {
    let mut end = None;
    seq(body, 0, pos, ctx, &mut |p, _| {
        end = Some(p);
        true
    })
    .then_some(end)
    .flatten()
}

fn single(body: &[M]) -> Option<&Test> {
    match body {
        [M::Char(test)] => Some(test),
        _ => None,
    }
}

/// One node at `pos`, then `k`. A failed attempt leaves the groups as it found them.
fn one(node: &M, pos: usize, ctx: &mut Ctx, k: K) -> bool {
    match node {
        M::Char(test) => pos < ctx.text.len() && test.matches(ctx.text[pos]) && k(pos + 1, ctx),
        M::Branch(alternatives) => alternatives
            .iter()
            .any(|alternative| seq(alternative, 0, pos, ctx, &mut *k)),
        M::Group(g, body) => {
            let g = *g;
            seq(body, 0, pos, ctx, &mut |end, ctx| {
                let old = ctx.caps[g];
                ctx.caps[g] = Some((pos, end));
                if k(end, ctx) {
                    return true;
                }
                ctx.caps[g] = old;
                false
            })
        }
        M::Repeat(rep, min, max, body) => match single(body) {
            Some(test) => repeat_one(*rep, *min, *max, test, pos, ctx, k),
            None => match rep {
                Rep::Max => repeat_max(body, *min, *max, 0, None, pos, ctx, k),
                Rep::Min => repeat_min(body, *min, *max, 0, None, pos, ctx, k),
                Rep::Possessive => repeat_possessive(body, *min, *max, pos, ctx, k),
            },
        },
        M::At(anchor) => at(*anchor, pos, ctx.text) && k(pos, ctx),
        M::Ref(g, probe) => {
            let Some((s, e)) = ctx.caps[*g] else {
                return false;
            };
            let len = e - s;
            if pos + len > ctx.text.len() {
                return false;
            }
            let same =
                (0..len).all(|i| probe.apply(ctx.text[pos + i]) == probe.apply(ctx.text[s + i]));
            same && k(pos + len, ctx)
        }
        M::RefExists(g, yes, no) => {
            let branch = if ctx.caps[*g].is_some() { yes } else { no };
            seq(branch, 0, pos, ctx, k)
        }
        M::Assert {
            negative,
            back,
            body,
        } => {
            let from = match back {
                Some(width) if pos < *width => return *negative && k(pos, ctx),
                Some(width) => pos - width,
                None => pos,
            };
            let saved = ctx.caps.clone();
            let found = first_end(body, from, ctx).is_some();
            if *negative {
                ctx.caps = saved;
                return !found && k(pos, ctx);
            }
            if found && k(pos, ctx) {
                return true;
            }
            ctx.caps = saved;
            false
        }
        M::Atomic(body) => {
            let saved = ctx.caps.clone();
            if let Some(end) = first_end(body, pos, ctx)
                && k(end, ctx)
            {
                return true;
            }
            ctx.caps = saved;
            false
        }
    }
}

fn repeat_one(rep: Rep, min: u64, max: u64, test: &Test, pos: usize, ctx: &mut Ctx, k: K) -> bool {
    let limit = if max == MAXREPEAT {
        usize::MAX
    } else {
        max as usize
    };
    let min = min as usize;
    let text = ctx.text;
    let available = text.len() - pos;
    match rep {
        Rep::Max | Rep::Possessive => {
            let mut n = 0;
            while n < limit && n < available && test.matches(text[pos + n]) {
                n += 1;
            }
            if n < min {
                return false;
            }
            if rep == Rep::Possessive {
                return k(pos + n, ctx);
            }
            (min..=n).rev().any(|c| k(pos + c, ctx))
        }
        Rep::Min => {
            if min > available || !(0..min).all(|i| test.matches(text[pos + i])) {
                return false;
            }
            let mut c = min;
            loop {
                if k(pos + c, ctx) {
                    return true;
                }
                if c < limit && c < available && test.matches(text[pos + c]) {
                    c += 1;
                } else {
                    return false;
                }
            }
        }
    }
}

fn below_max(count: u64, max: u64) -> bool {
    max == MAXREPEAT || count < max
}

/// `REPEAT` and `MAX_UNTIL`: another iteration is tried only after one that
/// consumed text, once the minimum is met.
#[allow(clippy::too_many_arguments)]
fn repeat_max(
    body: &[M],
    min: u64,
    max: u64,
    count: u64,
    last: Option<usize>,
    pos: usize,
    ctx: &mut Ctx,
    k: K,
) -> bool {
    if count < min {
        return seq(body, 0, pos, ctx, &mut |p, ctx| {
            repeat_max(body, min, max, count + 1, last, p, ctx, &mut *k)
        });
    }
    if below_max(count, max)
        && last != Some(pos)
        && seq(body, 0, pos, ctx, &mut |p, ctx| {
            repeat_max(body, min, max, count + 1, Some(pos), p, ctx, &mut *k)
        })
    {
        return true;
    }
    k(pos, ctx)
}

/// `REPEAT` and `MIN_UNTIL`.
#[allow(clippy::too_many_arguments)]
fn repeat_min(
    body: &[M],
    min: u64,
    max: u64,
    count: u64,
    last: Option<usize>,
    pos: usize,
    ctx: &mut Ctx,
    k: K,
) -> bool {
    if count < min {
        return seq(body, 0, pos, ctx, &mut |p, ctx| {
            repeat_min(body, min, max, count + 1, last, p, ctx, &mut *k)
        });
    }
    if k(pos, ctx) {
        return true;
    }
    if !below_max(count, max) || last == Some(pos) {
        return false;
    }
    seq(body, 0, pos, ctx, &mut |p, ctx| {
        repeat_min(body, min, max, count + 1, Some(pos), p, ctx, &mut *k)
    })
}

/// `POSSESSIVE_REPEAT`: each iteration is its body's first match.
fn repeat_possessive(body: &[M], min: u64, max: u64, pos: usize, ctx: &mut Ctx, k: K) -> bool {
    let saved = ctx.caps.clone();
    let mut p = pos;
    let mut count = 0;
    while count < min {
        let Some(end) = first_end(body, p, ctx) else {
            ctx.caps = saved;
            return false;
        };
        p = end;
        count += 1;
    }
    let mut last = None;
    while below_max(count, max) && last != Some(p) {
        last = Some(p);
        let before = ctx.caps.clone();
        match first_end(body, p, ctx) {
            Some(end) => {
                p = end;
                count += 1;
            }
            None => {
                ctx.caps = before;
                break;
            }
        }
    }
    if k(p, ctx) {
        return true;
    }
    ctx.caps = saved;
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn err(pattern: &str) -> String {
        match compile(&pts(pattern), 0) {
            Ok(p) => format!("groups {}", p.groups),
            Err(CompileError::Pattern(text)) => {
                text.iter().filter_map(|&c| char::from_u32(c)).collect()
            }
            Err(CompileError::Other(class, message)) => format!("{class}: {message}"),
        }
    }

    fn search(pattern: &str, text: &str) -> Option<Vec<Option<(usize, usize)>>> {
        let p = compile_str(pattern, 0);
        p.search(&pts(text))
            .map(|m| (0..=p.groups).map(|i| m.span(i)).collect())
    }

    #[test]
    fn parser_errors_match_cpython() {
        // Each line is CPython 3.15's `str(re.error)` or group count.
        let cases = [
            ("(", "missing ), unterminated subpattern at position 0"),
            ("a\\", "bad escape (end of pattern) at position 1"),
            ("[a", "unterminated character set at position 0"),
            ("a**", "multiple repeat at position 2"),
            ("*a", "nothing to repeat at position 0"),
            ("(?<n>a)", "unknown extension ?<n at position 1"),
            ("(?P<1>a)", "bad character in group name '1' at position 4"),
            ("(?P=x)", "unknown group name 'x' at position 4"),
            ("\\8", "invalid group reference 8 at position 1"),
            ("[z-a]", "bad character range z-a at position 1"),
            ("a{3,2}", "min repeat greater than max repeat at position 2"),
            (
                "(?i)a(?m)b",
                "global flags not at the start of the expression at position 5",
            ),
            (
                "x|(?i)y",
                "global flags not at the start of the expression at position 2",
            ),
            ("(?x) a b # c\n d", "groups 0"),
            ("(?q)", "unknown extension ?q at position 1"),
            ("(?-q:a)", "unknown flag at position 3"),
            (
                "(?i-i:a)",
                "bad inline flags: flag turned on and off at position 5",
            ),
            ("\\x4", "incomplete escape \\x4 at position 0"),
            ("\\u12", "incomplete escape \\u12 at position 0"),
            ("[\\w-z]", "bad character range \\w-z at position 1"),
            (
                "(?(1)a|b|c)",
                "conditional backref with more than two branches at position 8",
            ),
            ("(?(0)a)", "bad group number at position 3"),
            ("(?(x)a)", "unknown group name 'x' at position 3"),
            ("(a)(?(1)b)\\2", "invalid group reference 2 at position 11"),
            ("\\N", "missing { at position 2"),
            ("(?#abc", "missing ), unterminated comment at position 0"),
            ("a)", "unbalanced parenthesis at position 1"),
            ("\\9", "invalid group reference 9 at position 1"),
            (
                "a\nb(",
                "missing ), unterminated subpattern at position 3 (line 2, column 2)",
            ),
            (
                "(?P<a>x)(?P<a>y)",
                "redefinition of group name 'a' as group 2; was group 1 at position 12",
            ),
            ("(?<=\\1)(a)", "invalid group reference 1 at position 5"),
            ("(a)(?<=\\1)", "groups 1"),
            ("((?<=a)|b)*", "groups 1"),
            ("(?a:\\w)", "groups 0"),
            (
                "(?au)",
                "bad inline flags: flags 'a', 'u' and 'L' are incompatible at position 4",
            ),
            (
                "(?L)",
                "bad inline flags: cannot use 'L' flag with a str pattern at position 3",
            ),
            ("\\0777", "groups 0"),
            (
                "[\\777]",
                "octal escape value \\777 outside of range 0-0o377 at position 1",
            ),
            (
                "\\777",
                "octal escape value \\777 outside of range 0-0o377 at position 0",
            ),
            (
                "a{99999999999}",
                "OverflowError: the repetition number is too large",
            ),
            ("(?<=a+)b", "look-behind requires fixed-width pattern"),
            ("(?<=aaaaa)b", "groups 0"),
        ];
        for (pattern, expected) in cases {
            assert_eq!(err(pattern), expected, "{pattern:?}");
        }
    }

    #[test]
    fn matching_follows_sre() {
        assert_eq!(
            search(r"(?:(a)|b)*", "ab"),
            Some(vec![Some((0, 2)), Some((0, 1))])
        );
        assert_eq!(
            search(r"(a*)*", "b"),
            Some(vec![Some((0, 0)), Some((0, 0))])
        );
        assert_eq!(
            search(r"(a|)+b", "aab"),
            Some(vec![Some((0, 3)), Some((2, 2))])
        );
        assert!(search(r"(?i)ſ", "S").is_some());
        assert!(search(r"(?i)[ſx]", "S").is_some());
        assert!(search("(?i)\u{212a}", "k").is_some());
        assert!(search(r"(?i)[a-z]", "\u{212a}").is_some());
        assert!(search("(?i)[\u{10400}-\u{10410}]", "\u{10428}").is_some());
        assert!(search(r"(?i)İ", "i").is_some());
        assert!(search(r"(?i)i", "İ").is_some());
        assert!(search(r"\B", "").is_some());
        assert!(search(r"\b", "").is_none());
        assert_eq!(search(r"a$", "a\n"), Some(vec![Some((0, 1))]));
        assert_eq!(
            search(r"(\d+)\s?%", "rate 8.77%"),
            Some(vec![Some((7, 10)), Some((7, 9))])
        );
        assert_eq!(search(r"x(?=y)", "xxy"), Some(vec![Some((1, 2))]));
        assert_eq!(search(r"(?<=\$)\d+", "a $12"), Some(vec![Some((3, 5))]));
        assert_eq!(
            search(r"(a)(?(1)b|c)", "ab"),
            Some(vec![Some((0, 2)), Some((0, 1))])
        );
        assert_eq!(search(r"(?>a+)a", "aaa"), None);
        assert_eq!(search(r"a++a", "aaa"), None);
        assert_eq!(search(r"(?:ab)*?c", "ababc"), Some(vec![Some((0, 5))]));
    }

    #[test]
    fn str_lower_handles_final_sigma_and_dotted_i() {
        let lower = |s: &str| -> String {
            str_lower(&pts(s))
                .into_iter()
                .filter_map(char::from_u32)
                .collect()
        };
        assert_eq!(lower("ΟΔΟΣ"), "οδος");
        assert_eq!(lower("Σ"), "σ");
        assert_eq!(lower("İ"), "i\u{307}");
        assert_eq!(lower("AUGUST"), "august");
    }
}
