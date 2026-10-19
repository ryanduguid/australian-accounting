//! The `div7a-loan-review` command line. argparse formatting and option
//! parsing are reused from The WIP Tally's port, extended with option choices,
//! a flag option, an epilog and subcommands without positionals.
//!
//! Exit codes: 0 where nothing is exposed and nothing is undecided, 2 where a
//! shortfall or an undecided row needs a human (or a usage error), and 1 where
//! the input or the rate table could not be read at all.
use std::collections::BTreeMap;

use serde_json::{Map, Value, json};

use crate::argtheme::{self, Stream, Theme};
use crate::decimal::Calc;
use crate::facts::optional_year_of_income;
use crate::gate::GateResult;
use crate::money::cents_str;
use crate::myr::MyrResult;
use crate::py::PyPath;
use crate::pytext;
use crate::rates::{RateResult, benchmark_rate, frozen_table, load_override};
use crate::raw::Raw;
use crate::register::{GATE_COLUMNS, Mode, ReviewReport, load_rows, review_register_file};
use crate::textwrap::wrap;
use crate::years::{YearOfIncome, parse_year};
use crate::{Failure, LAW_COMPILATION, LAW_CONTENT_DATE, VERSION};

const PROG: &str = "div7a-loan-review";
const DESCRIPTION: &str = "Experimental review of private-company loans against the ITAA \
1936 s 109N complying-loan criteria and the s 109E minimum yearly repayment. Not a Division \
7A determination.";
pub const BANNER: &str = "Experimental review aid. Not a Division 7A determination, not an \
ATO assessment, and not tax, legal or financial advice.";
const HELP_HELP: &str = "show this help message and exit";
const VERSION_HELP: &str = "show program's version number and exit";
const EXIT_OK: i32 = 0;
const EXIT_ERROR: i32 = 1;
const EXIT_ATTENTION: i32 = 2;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Takes {
    Value,
    Flag,
}

/// One option of a subcommand.
struct Opt {
    flag: &'static str,
    dest: &'static str,
    metavar: Option<&'static str>,
    takes: Takes,
    required: bool,
    choices: Option<&'static [&'static str]>,
    help: &'static str,
}

impl Opt {
    /// The metavar argparse prints: the choices, the metavar or the upper-cased dest.
    fn shown_metavar(&self) -> String {
        match (self.choices, self.metavar) {
            (Some(choices), _) => format!("{{{}}}", choices.join(",")),
            (None, Some(metavar)) => metavar.into(),
            (None, None) => self.dest.to_uppercase(),
        }
    }
}

struct Command {
    name: &'static str,
    help: &'static str,
    options: &'static [Opt],
}

const fn value(flag: &'static str, dest: &'static str, required: bool, help: &'static str) -> Opt {
    Opt {
        flag,
        dest,
        metavar: None,
        takes: Takes::Value,
        required,
        choices: None,
        help,
    }
}

const FORMAT: Opt = Opt {
    flag: "--format",
    dest: "format",
    metavar: None,
    takes: Takes::Value,
    required: false,
    choices: Some(&["text", "json"]),
    help: "output format",
};
const RATES_OVERRIDE: Opt = Opt {
    flag: "--rates-override",
    dest: "rates_override",
    metavar: Some("FILE"),
    takes: Takes::Value,
    required: false,
    choices: None,
    help: "reviewed benchmark-rate override (JSON carrying verified_until and a citation); \
           required for any year outside the frozen table",
};
const INPUT: Opt = value("--input", "input", true, "loan register CSV");

static COMMANDS: [Command; 4] = [
    Command {
        name: "rate",
        help: "the s 109N(2) benchmark rate for a year of income",
        options: &[
            value("--year", "year", true, "year of income, e.g. 2026-27"),
            FORMAT,
            RATES_OVERRIDE,
        ],
    },
    Command {
        name: "gate",
        help: "test a register against the s 109N(1) criteria",
        options: &[
            INPUT,
            value(
                "--year",
                "year",
                false,
                "year of income to read the benchmark floor for. Omit to use each row's \
                 year_loan_made. The annual-interest interpretation remains unresolved",
            ),
            FORMAT,
            RATES_OVERRIDE,
        ],
    },
    Command {
        name: "myr",
        help: "the s 109E minimum yearly repayment and any shortfall",
        options: &[
            INPUT,
            value(
                "--year",
                "year",
                true,
                "year of income being reviewed, e.g. 2026-27",
            ),
            FORMAT,
            RATES_OVERRIDE,
        ],
    },
    Command {
        name: "review",
        help: "the s 109N gate and the s 109E repayment together",
        options: &[
            INPUT,
            value("--year", "year", true, "year of income being reviewed"),
            Opt {
                flag: "--trace",
                dest: "trace",
                metavar: None,
                takes: Takes::Flag,
                required: false,
                choices: None,
                help: "print the statutory trace for every row",
            },
            FORMAT,
            RATES_OVERRIDE,
        ],
    },
];

/// The flags of a command in the order argparse registers them.
fn flags(command: &Command) -> Vec<&'static str> {
    let mut flags = vec!["-h", "--help"];
    flags.extend(command.options.iter().map(|o| o.flag));
    flags
}

/// argparse's formatter width: `shutil.get_terminal_size().columns` minus 2.
fn width() -> usize {
    usize::try_from(crate::termsize::columns() - 2).unwrap_or(0)
}

/// A usage part or help header: its plain text, which sets every width, and
/// the coloured text argparse prints.
#[derive(Clone)]
struct Part {
    plain: String,
    coloured: String,
}

/// `HelpFormatter._format_usage` for a parser whose usage parts are given.
fn usage(prog: &str, opt_parts: &[Part], pos_parts: &[Part], t: &Theme) -> String {
    let prefix = "usage: ";
    let text_width = width();
    let plain: Vec<&str> = opt_parts
        .iter()
        .chain(pos_parts)
        .map(|p| p.plain.as_str())
        .collect();
    let rest =
        if prefix.len() + pytext::py_len(&format!("{prog} {}", plain.join(" "))) <= text_width {
            let coloured: Vec<&str> = opt_parts
                .iter()
                .chain(pos_parts)
                .map(|p| p.coloured.as_str())
                .collect();
            format!(" {}", coloured.join(" "))
        } else {
            wrapped_usage(prog, opt_parts, pos_parts, text_width)
        };
    format!(
        "{}{prefix}{}{}{prog}{}{rest}",
        t.usage, t.reset, t.prog, t.reset
    )
}

/// The wrapped usage after the program name, as `_format_usage` lays it out.
fn wrapped_usage(prog: &str, opt_parts: &[Part], pos_parts: &[Part], text_width: usize) -> String {
    let prefix_len = "usage: ".len();
    let get_lines = |parts: &[Part], indent: &str, with_prefix: bool| {
        usage_lines(parts, indent, with_prefix.then_some(prefix_len), text_width)
    };
    if (prefix_len + pytext::py_len(prog)) as f64 <= 0.75 * text_width as f64 {
        let indent = " ".repeat(prefix_len + pytext::py_len(prog) + 1);
        let mut head = vec![Part {
            plain: prog.to_string(),
            coloured: prog.to_string(),
        }];
        let lines = if !opt_parts.is_empty() {
            head.extend(opt_parts.iter().cloned());
            let mut lines = get_lines(&head, &indent, true);
            lines.extend(get_lines(pos_parts, &indent, false));
            lines
        } else {
            head.extend(pos_parts.iter().cloned());
            get_lines(&head, &indent, true)
        };
        return lines.join("\n")[prog.len()..].to_string();
    }
    let indent = " ".repeat(prefix_len);
    let all: Vec<Part> = opt_parts.iter().chain(pos_parts).cloned().collect();
    let mut lines = get_lines(&all, &indent, false);
    if lines.len() > 1 {
        lines = get_lines(opt_parts, &indent, false);
        lines.extend(get_lines(pos_parts, &indent, false));
    }
    format!("\n{}", lines.join("\n"))
}

/// `_format_usage`'s `get_lines`: wrap parts under `indent`; with a prefix
/// length, the first line starts after the prefix instead of the indent.
fn usage_lines(
    parts: &[Part],
    indent: &str,
    prefix_len: Option<usize>,
    text_width: usize,
) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line: Vec<&str> = Vec::new();
    let indent_length = indent.len();
    let mut line_len = prefix_len.unwrap_or(indent_length) - 1;
    for part in parts {
        let part_len = pytext::py_len(&part.plain);
        if line_len + 1 + part_len > text_width && !line.is_empty() {
            lines.push(format!("{indent}{}", line.join(" ")));
            line.clear();
            line_len = indent_length - 1;
        }
        line.push(&part.coloured);
        line_len += part_len + 1;
    }
    if !line.is_empty() {
        lines.push(format!("{indent}{}", line.join(" ")));
    }
    if prefix_len.is_some() && !lines.is_empty() {
        lines[0] = lines[0][indent_length..].to_string();
    }
    lines
}

type Row<'a> = (usize, Part, Option<&'a str>);

/// The rows of one help section; argparse aligns help text across every section of a parser.
fn action_rows(rows: &[Row], all: &[Row]) -> String {
    let width = width();
    let max_help_position = 24.min(width.saturating_sub(20).max(4));
    let action_max = all
        .iter()
        .map(|(indent, header, _)| indent + pytext::py_len(&header.plain))
        .max()
        .unwrap_or(0);
    let help_position = (action_max + 2).min(max_help_position);
    let help_width = width.saturating_sub(help_position).max(11);
    let mut out = String::new();
    for (indent, header, help) in rows {
        let action_width = help_position.saturating_sub(indent + 2);
        let pad = " ".repeat(*indent);
        let Some(help) = help else {
            out.push_str(&format!("{pad}{}\n", header.coloured));
            continue;
        };
        let lines = wrap(help, help_width);
        if pytext::py_len(&header.plain) <= action_width {
            // argparse pads the plain header, then swaps the coloured one in.
            let padded = format!("{pad}{:<action_width$}  {}\n", header.plain, lines[0]);
            out.push_str(&padded.replacen(&header.plain, &header.coloured, 1));
        } else {
            out.push_str(&format!(
                "{pad}{}\n{}{}\n",
                header.coloured,
                " ".repeat(help_position),
                lines[0]
            ));
        }
        for line in &lines[1..] {
            out.push_str(&format!("{}{line}\n", " ".repeat(help_position)));
        }
    }
    out
}

fn coloured(text: &str, colour: &str, t: &Theme) -> Part {
    Part {
        plain: text.to_string(),
        coloured: format!("{colour}{text}{}", t.reset),
    }
}

/// `-h, --help` as `_format_action_invocation` colours it.
fn help_header(t: &Theme) -> Part {
    Part {
        plain: "-h, --help".into(),
        coloured: format!(
            "{}-h{}, {}--help{}",
            t.short_option, t.reset, t.long_option, t.reset
        ),
    }
}

/// The ratio difflib.SequenceMatcher(None, a, b).ratio() computes.
fn sequence_ratio(a: &[u32], b: &[u32]) -> f64 {
    let mut b2j: BTreeMap<u32, Vec<usize>> = BTreeMap::new();
    for (j, c) in b.iter().enumerate() {
        b2j.entry(*c).or_default().push(j);
    }
    if b.len() >= 200 {
        let ntest = b.len() / 100 + 1;
        b2j.retain(|_, indices| indices.len() <= ntest);
    }
    let mut matches = 0;
    let mut queue = vec![(0, a.len(), 0, b.len())];
    while let Some(range) = queue.pop() {
        let (alo, ahi, blo, bhi) = range;
        let (i, j, k) = longest_match(a, b, &b2j, range);
        if k > 0 {
            matches += k;
            if alo < i && blo < j {
                queue.push((alo, i, blo, j));
            }
            if i + k < ahi && j + k < bhi {
                queue.push((i + k, ahi, j + k, bhi));
            }
        }
    }
    let total = a.len() + b.len();
    if total == 0 {
        1.0
    } else {
        2.0 * matches as f64 / total as f64
    }
}

/// `SequenceMatcher.find_longest_match(alo, ahi, blo, bhi)`.
fn longest_match(
    a: &[u32],
    b: &[u32],
    b2j: &BTreeMap<u32, Vec<usize>>,
    (alo, ahi, blo, bhi): (usize, usize, usize, usize),
) -> (usize, usize, usize) {
    let (mut besti, mut bestj, mut bestsize) = (alo, blo, 0);
    let mut j2len: BTreeMap<usize, usize> = BTreeMap::new();
    for (i, ch) in a.iter().enumerate().take(ahi).skip(alo) {
        let mut new: BTreeMap<usize, usize> = BTreeMap::new();
        for &j in b2j.get(ch).map(Vec::as_slice).unwrap_or(&[]) {
            if j < blo {
                continue;
            }
            if j >= bhi {
                break;
            }
            let k = j
                .checked_sub(1)
                .and_then(|p| j2len.get(&p))
                .copied()
                .unwrap_or(0)
                + 1;
            new.insert(j, k);
            if k > bestsize {
                (besti, bestj, bestsize) = (i + 1 - k, j + 1 - k, k);
            }
        }
        j2len = new;
    }
    // Extend over popular characters the autojunk heuristic dropped from b2j.
    while besti > alo && bestj > blo && a[besti - 1] == b[bestj - 1] {
        (besti, bestj, bestsize) = (besti - 1, bestj - 1, bestsize + 1);
    }
    while besti + bestsize < ahi
        && bestj + bestsize < bhi
        && a[besti + bestsize] == b[bestj + bestsize]
    {
        bestsize += 1;
    }
    (besti, bestj, bestsize)
}

/// `difflib.get_close_matches(word, possibilities, n=1)`.
fn close_match(word: &str, possibilities: &[&'static str]) -> Option<&'static str> {
    // Python compares code points, so a lone surrogate counts once.
    let b: Vec<u32> = pytext::code_points(word).collect();
    possibilities
        .iter()
        .map(|p| {
            (
                sequence_ratio(&p.chars().map(u32::from).collect::<Vec<_>>(), &b),
                *p,
            )
        })
        .filter(|(score, _)| *score >= 0.6)
        .max_by(|x, y| x.0.partial_cmp(&y.0).unwrap().then_with(|| x.1.cmp(y.1)))
        .map(|(_, p)| p)
}

/// What argparse does instead of returning a namespace.
enum Exit {
    Help(String, i32),
    Error {
        usage: String,
        prog: String,
        message: String,
    },
}

/// How argparse's `_parse_optional` classifies one token against a parser's flags.
enum Class<'a> {
    /// A flag, with an explicit argument (`--flag=value`, or `-hvalue` for a short
    /// flag) and whether `=` introduced it.
    Flag(&'a str, Option<(String, bool)>),
    UnknownOption,
    Positional,
}

fn classify<'a>(token: &str, flags: &[&'a str]) -> Result<Class<'a>, String> {
    if token.is_empty() || !token.starts_with('-') {
        return Ok(Class::Positional);
    }
    if let Some(exact) = flags.iter().find(|f| **f == token) {
        return Ok(Class::Flag(exact, None));
    }
    if pytext::py_len(token) == 1 {
        return Ok(Class::Positional);
    }
    if let Some((name, value)) = token.split_once('=')
        && let Some(exact) = flags.iter().find(|f| **f == name)
    {
        return Ok(Class::Flag(exact, Some((value.to_string(), true))));
    }
    let mut found = option_prefix_matches(token, flags);
    match found.len() {
        0 => {}
        1 => {
            let (flag, explicit) = found.remove(0);
            return Ok(Class::Flag(flag, explicit));
        }
        _ => {
            let names: Vec<&str> = found.iter().map(|(f, _)| *f).collect();
            return Err(format!(
                "ambiguous option: {token} could match {}",
                names.join(", ")
            ));
        }
    }
    if looks_like_negative_number(&token[1..]) || token.contains(' ') {
        return Ok(Class::Positional);
    }
    Ok(Class::UnknownOption)
}

/// argparse's `_get_option_tuples`: the flags a token abbreviates, with any explicit value.
fn option_prefix_matches<'a>(
    token: &str,
    flags: &[&'a str],
) -> Vec<(&'a str, Option<(String, bool)>)> {
    if token.starts_with("--") {
        let (prefix, explicit) = match token.split_once('=') {
            Some((name, value)) => (name, Some((value.to_string(), true))),
            None => (token, None),
        };
        return flags
            .iter()
            .filter(|f| f.starts_with(prefix))
            .map(|f| (*f, explicit.clone()))
            .collect();
    }
    let short: String = token.chars().take(2).collect();
    let rest: String = token.chars().skip(2).collect();
    flags
        .iter()
        .filter_map(|f| {
            if *f == short {
                Some((*f, Some((rest.clone(), false))))
            } else {
                f.starts_with(token).then_some((*f, None))
            }
        })
        .collect()
}

/// argparse's `_negative_number_matcher` on the text after the leading '-'.
fn looks_like_negative_number(body: &str) -> bool {
    let digit = |c: char| pytext::decimal_value(c as u32).is_some();
    (!body.is_empty() && body.chars().all(digit))
        || body.split_once('.').is_some_and(|(int, frac)| {
            int.chars().all(digit) && !frac.is_empty() && frac.chars().all(digit)
        })
}

/// argparse's error for an explicit argument given to a zero-argument flag.
/// `-hx` is read as further short flags instead, and help runs first.
fn ignored_argument(flag: &str, explicit: Option<(String, bool)>) -> Result<(), String> {
    match explicit {
        None => Ok(()),
        Some((rest, false)) if flag == "-h" && !rest.is_empty() && !rest.starts_with('-') => Ok(()),
        Some((rest, _)) => {
            let name = if flag == "-h" || flag == "--help" {
                "-h/--help"
            } else {
                flag
            };
            Err(format!(
                "argument {name}: ignored explicit argument {}",
                pytext::repr_os(&rest)
            ))
        }
    }
}

fn choices() -> String {
    format!(
        "{{{}}}",
        COMMANDS
            .iter()
            .map(|c| c.name)
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn main_usage(prog: &str, t: &Theme) -> String {
    let parts = [
        Part {
            plain: "[-h]".into(),
            coloured: format!("[{}-h{}]", t.summary_short_option, t.reset),
        },
        Part {
            plain: "[--version]".into(),
            coloured: format!("[{}--version{}]", t.summary_long_option, t.reset),
        },
    ];
    usage(
        prog,
        &parts,
        &[coloured(&format!("{} ...", choices()), t.summary_action, t)],
        t,
    )
}

fn heading(text: &str, t: &Theme) -> String {
    format!("{}{text}:{}", t.heading, t.reset)
}

fn paragraph(text: &str) -> String {
    wrap(text, width().max(11)).join("\n")
}

fn main_help(prog: &str, t: &Theme) -> String {
    let mut positional: Vec<Row> = vec![(2, coloured(&choices(), t.action, t), None)];
    positional.extend(
        COMMANDS
            .iter()
            .map(|c| (4, coloured(c.name, t.action, t), Some(c.help))),
    );
    let options: Vec<Row> = vec![
        (2, help_header(t), Some(HELP_HELP)),
        (
            2,
            coloured("--version", t.long_option, t),
            Some(VERSION_HELP),
        ),
    ];
    let all: Vec<Row> = positional.iter().chain(&options).cloned().collect();
    format!(
        "{}\n\n{}\n\n{}\n{}\n{}\n{}\n{}\n",
        main_usage(prog, t),
        paragraph(DESCRIPTION),
        heading("positional arguments", t),
        action_rows(&positional, &all),
        heading("options", t),
        action_rows(&options, &all),
        paragraph(BANNER)
    )
}

fn command_usage(prog: &str, command: &Command, t: &Theme) -> String {
    let mut parts = vec![Part {
        plain: "[-h]".into(),
        coloured: format!("[{}-h{}]", t.summary_short_option, t.reset),
    }];
    for option in command.options {
        let (plain, coloured) = match option.takes {
            Takes::Flag => (
                option.flag.to_string(),
                format!("{}{}{}", t.summary_long_option, option.flag, t.reset),
            ),
            Takes::Value => {
                let metavar = option.shown_metavar();
                (
                    format!("{} {metavar}", option.flag),
                    format!(
                        "{}{} {}{metavar}{}",
                        t.summary_long_option, option.flag, t.summary_label, t.reset
                    ),
                )
            }
        };
        parts.push(if option.required {
            Part { plain, coloured }
        } else {
            Part {
                plain: format!("[{plain}]"),
                coloured: format!("[{coloured}]"),
            }
        });
    }
    usage(&format!("{prog} {}", command.name), &parts, &[], t)
}

fn command_help(prog: &str, command: &Command, t: &Theme) -> String {
    let mut options: Vec<Row> = vec![(2, help_header(t), Some(HELP_HELP))];
    for option in command.options {
        let header = match option.takes {
            Takes::Flag => coloured(option.flag, t.long_option, t),
            Takes::Value => {
                let metavar = option.shown_metavar();
                Part {
                    plain: format!("{} {metavar}", option.flag),
                    coloured: format!(
                        "{}{}{} {}{metavar}{}",
                        t.long_option, option.flag, t.reset, t.label, t.reset
                    ),
                }
            }
        };
        options.push((2, header, Some(option.help)));
    }
    format!(
        "{}\n\n{}\n{}",
        command_usage(prog, command, t),
        heading("options", t),
        action_rows(&options, &options)
    )
}

fn main_exit(prog: &str, message: String) -> Exit {
    Exit::Error {
        usage: main_usage(prog, argtheme::theme(Stream::Stderr)),
        prog: prog.to_string(),
        message,
    }
}

fn sub_exit(prog: &str, command: &Command, message: String) -> Exit {
    Exit::Error {
        usage: command_usage(prog, command, argtheme::theme(Stream::Stderr)),
        prog: format!("{prog} {}", command.name),
        message,
    }
}

/// argparse's invalid-choice message, with Python 3.15's close-match suggestion.
fn invalid_choice(argument: &str, token: &str, names: &[&'static str]) -> String {
    let quoted: Vec<String> = names.iter().map(|n| pytext::repr(n)).collect();
    let suggestion = close_match(token, names)
        .map(|m| format!(", maybe you meant {}?", pytext::repr(m)))
        .unwrap_or_default();
    format!(
        "argument {argument}: invalid choice: {}{suggestion} (choose from {})",
        pytext::repr_os(token),
        quoted.join(", ")
    )
}

/// A subcommand's parsed arguments by destination; a flag holds "true".
struct Parsed {
    command: &'static Command,
    values: BTreeMap<&'static str, String>,
}

impl Parsed {
    fn get(&self, dest: &str) -> Option<&str> {
        self.values.get(dest).map(String::as_str)
    }
}

/// The main parser: help, version, unknown options kept as extras, then the
/// subparsers action, which takes the first positional and everything after it.
fn parse(prog: &str, args: &[String]) -> Result<Parsed, Exit> {
    let main_flags = ["-h", "--help", "--version"];
    let mut extras: Vec<String> = Vec::new();
    let mut index = 0;
    let mut command_at = None;
    while let Some(token) = args.get(index) {
        if token == "--" {
            if index + 1 < args.len() {
                command_at = Some(index + 1);
            }
            break;
        }
        match classify(token, &main_flags).map_err(|m| main_exit(prog, m))? {
            Class::Flag(flag, explicit) => {
                ignored_argument(flag, explicit).map_err(|m| main_exit(prog, m))?;
                if flag == "--version" {
                    return Err(Exit::Help(format!("{prog} {VERSION}\n"), 0));
                }
                return Err(Exit::Help(
                    main_help(prog, argtheme::theme(Stream::Stdout)),
                    0,
                ));
            }
            Class::UnknownOption => extras.push(token.clone()),
            Class::Positional => {
                command_at = Some(index);
                break;
            }
        }
        index += 1;
    }
    let Some(at) = command_at else {
        return Err(main_exit(
            prog,
            "the following arguments are required: command".into(),
        ));
    };
    let Some(command) = COMMANDS.iter().find(|c| c.name == args[at]) else {
        let names: Vec<&'static str> = COMMANDS.iter().map(|c| c.name).collect();
        return Err(main_exit(
            prog,
            invalid_choice("command", &args[at], &names),
        ));
    };
    let parsed = parse_command(prog, command, &args[at + 1..], &mut extras)?;
    if !extras.is_empty() {
        return Err(main_exit(
            prog,
            format!("unrecognized arguments: {}", extras.join(" ")),
        ));
    }
    Ok(parsed)
}

/// One token's place in argparse's pattern string.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Pattern {
    Option(usize),
    Unknown,
    Arg,
    Marker,
}

/// A subparser's `_parse_known_args` with options only: every positional
/// token is an extra.
fn parse_command(
    prog: &str,
    command: &'static Command,
    args: &[String],
    extras: &mut Vec<String>,
) -> Result<Parsed, Exit> {
    let sub_error = |message: String| sub_exit(prog, command, message);
    let flags = flags(command);
    let mut patterns = Vec::with_capacity(args.len());
    let mut explicit: Vec<Option<(String, bool)>> = vec![None; args.len()];
    let mut chosen: Vec<&'static str> = vec![""; args.len()];
    let mut marked = false;
    for (i, token) in args.iter().enumerate() {
        if marked {
            patterns.push(Pattern::Arg);
        } else if token == "--" {
            marked = true;
            patterns.push(Pattern::Marker);
        } else {
            patterns.push(match classify(token, &flags).map_err(sub_error)? {
                Class::Flag(flag, value) => {
                    chosen[i] = flag;
                    explicit[i] = value;
                    Pattern::Option(i)
                }
                Class::UnknownOption => Pattern::Unknown,
                Class::Positional => Pattern::Arg,
            });
        }
    }
    let mut parsed = Parsed {
        command,
        values: BTreeMap::new(),
    };
    let mut start = 0;
    while start < args.len() {
        match patterns[start] {
            // No positional consumes these, so they are extras, the first `--` included.
            Pattern::Arg | Pattern::Marker | Pattern::Unknown => {
                extras.push(args[start].clone());
                start += 1;
            }
            Pattern::Option(i) => {
                let flag = chosen[i];
                if flag == "-h" || flag == "--help" {
                    ignored_argument(flag, explicit[i].clone()).map_err(sub_error)?;
                    return Err(Exit::Help(
                        command_help(prog, command, argtheme::theme(Stream::Stdout)),
                        0,
                    ));
                }
                let option = command
                    .options
                    .iter()
                    .find(|o| o.flag == flag)
                    .expect("known flag");
                if option.takes == Takes::Flag {
                    ignored_argument(flag, explicit[i].clone()).map_err(sub_error)?;
                    parsed.values.insert(option.dest, "true".into());
                    start += 1;
                    continue;
                }
                let value = match explicit[i].clone() {
                    Some((value, _)) => {
                        start += 1;
                        value
                    }
                    None if patterns.get(start + 1) == Some(&Pattern::Arg) => {
                        start += 2;
                        args[start - 1].clone()
                    }
                    None => {
                        return Err(sub_error(format!(
                            "argument {}: expected one argument",
                            option.flag
                        )));
                    }
                };
                if let Some(names) = option.choices
                    && !names.contains(&value.as_str())
                {
                    return Err(sub_error(invalid_choice(option.flag, &value, names)));
                }
                parsed.values.insert(option.dest, value);
            }
        }
    }
    let missing: Vec<&str> = command
        .options
        .iter()
        .filter(|o| o.required && !parsed.values.contains_key(o.dest))
        .map(|o| o.flag)
        .collect();
    if !missing.is_empty() {
        return Err(sub_error(format!(
            "the following arguments are required: {}",
            missing.join(", ")
        )));
    }
    Ok(parsed)
}

fn emit_json(document: Value, out: &mut String) {
    out.push_str(&serde_json::to_string_pretty(&document).expect("JSON values serialise"));
    out.push('\n');
}

fn envelope(command: &str, payload: Value) -> Value {
    let mut out = Map::new();
    out.insert("tool".into(), json!(PROG));
    out.insert("version".into(), json!(VERSION));
    out.insert("command".into(), json!(command));
    out.insert("law_content_date".into(), json!(LAW_CONTENT_DATE));
    out.insert("law_compilation".into(), json!(LAW_COMPILATION));
    out.insert("disclaimer".into(), json!(BANNER));
    if let Value::Object(payload) = payload {
        out.extend(payload);
    }
    Value::Object(out)
}

fn print_lines(out: &mut String, prefix: &str, items: &[String]) {
    for item in items {
        out.push_str(&format!("  {prefix} {item}\n"));
    }
}

fn print_rate(out: &mut String, result: &RateResult) {
    if result.known {
        let entry = result.entry.as_ref().expect("a known rate has its entry");
        out.push_str(&format!(
            "{} benchmark interest rate: {}\n",
            result.year_of_income,
            result.rate_text().unwrap_or_default()
        ));
        out.push_str(&format!(
            "  source: RBA table {} series {}, {} figure ({})\n",
            entry.rba_table, entry.rba_series, entry.rba_month, entry.origin
        ));
        out.push_str(&format!("  entry reviewed: {}\n", entry.seen));
    } else {
        out.push_str(&format!(
            "{} benchmark interest rate: UNKNOWN\n",
            result.year_of_income
        ));
        out.push_str(&format!(
            "  {}\n",
            result.reason.clone().unwrap_or_default()
        ));
    }
    out.push_str(&format!(
        "  table reviewed to {} on {}\n",
        result.table_reviewed_until, result.table_reviewed_on
    ));
}

fn print_gate(out: &mut String, gate: &GateResult) {
    out.push_str(&format!(
        "  s 109N gate: {}  (benchmark year {})\n",
        gate.verdict, gate.benchmark_year_used
    ));
    for limb in &gate.limbs {
        out.push_str(&format!(
            "    [{:<7}] {}: {}\n",
            limb.state.name(),
            limb.cite,
            limb.finding
        ));
    }
    print_lines(out, "!", &gate.caveats);
}

fn print_myr(out: &mut String, myr: &MyrResult) -> Calc<()> {
    out.push_str(&format!("  s 109E MYR: {}\n", myr.verdict));
    if let (Some(required), Some(applied), Some(shortfall)) =
        (&myr.myr_required, &myr.payments_applied, &myr.shortfall)
    {
        out.push_str(&format!("    required : {}\n", cents_str(required)?));
        out.push_str(&format!("    applied  : {}\n", cents_str(applied)?));
        out.push_str(&format!("    shortfall: {}\n", cents_str(shortfall)?));
        if let Some(exposure) = &myr.experimental_deemed_dividend_exposure {
            out.push_str(&format!(
                "    experimental deemed-dividend exposure: {} (review aid, not an assessment)\n",
                cents_str(exposure)?
            ));
        }
    }
    print_lines(out, "!", &myr.reasons);
    Ok(())
}

fn print_report(out: &mut String, report: &ReviewReport, show_trace: bool) -> Calc<()> {
    out.push_str(&format!(
        "Loan register review for {}\n",
        report.year_of_income
    ));
    out.push_str(&format!("rows reviewed: {}\n", report.rows_reviewed));
    let counts: Vec<String> = report
        .summary
        .iter()
        .map(|(key, value)| format!("{key} {value}"))
        .collect();
    out.push_str(&format!("summary: {}\n", counts.join(", ")));
    out.push_str(
        "  counts are per question, not per row: a reviewed row answers both the s 109N \
         terms question and the s 109E repayment question\n",
    );
    out.push_str(&format!(
        "experimental total exposure: {} (review aid, not an ATO assessment)\n",
        cents_str(&report.total_exposure()?)?
    ));
    out.push('\n');
    for line in &report.lines {
        let mut label = format!("[{}] {}", line.row_number, line.loan_id);
        if !line.borrower_reference.is_empty() {
            label.push_str(&format!(" ({})", line.borrower_reference));
        }
        out.push_str(&label);
        out.push('\n');
        if line.is_skipped() {
            out.push_str(&format!("  SKIPPED: {}\n\n", line.skipped_reason));
            continue;
        }
        if let Some(gate) = &line.gate {
            print_gate(out, gate);
        }
        if let Some(myr) = &line.myr {
            print_myr(out, myr)?;
        }
        if show_trace {
            if let Some(gate) = &line.gate {
                print_lines(out, "|", &gate.statutory_trace);
            }
            if let Some(myr) = &line.myr {
                print_lines(out, "|", &myr.statutory_trace);
            }
        }
        out.push('\n');
    }
    Ok(())
}

fn year_arg(text: &str) -> Result<YearOfIncome, Failure> {
    parse_year(&Raw::text(text), "year of income").map_err(Failure::Value)
}

fn run_rate(parsed: &Parsed, out: &mut String) -> Result<i32, Failure> {
    let table = frozen_table();
    let over = parsed
        .get("rates_override")
        .filter(|p| !p.is_empty())
        .map(|p| load_override(&PyPath::new(p)))
        .transpose()?;
    let year = year_arg(parsed.get("year").unwrap_or_default())?;
    let result = benchmark_rate(year, Some(&table), over.as_ref());
    if parsed.get("format") == Some("json") {
        emit_json(envelope("rate", result.to_json()), out);
    } else {
        out.push_str(&format!("{BANNER}\n\n"));
        print_rate(out, &result);
    }
    Ok(if result.known {
        EXIT_OK
    } else {
        EXIT_ATTENTION
    })
}

/// A gate run without --year still needs a year to label the report with.
fn label_year(input: &str) -> Result<YearOfIncome, Failure> {
    for row in load_rows(&PyPath::new(input), &GATE_COLUMNS)? {
        if let Some(year) = optional_year_of_income(row.get("year_loan_made"), "year_loan_made")
            .map_err(Failure::Value)?
        {
            return Ok(year);
        }
    }
    Ok(frozen_table().reviewed_until)
}

fn run_register(parsed: &Parsed, out: &mut String) -> Result<i32, Failure> {
    let table = frozen_table();
    let over = parsed
        .get("rates_override")
        .filter(|p| !p.is_empty())
        .map(|p| load_override(&PyPath::new(p)))
        .transpose()?;
    let name = parsed.command.name;
    let input = parsed.get("input").unwrap_or_default();
    let report = if name == "gate" {
        let nominated = match parsed.get("year").filter(|y| !y.is_empty()) {
            Some(year) => Some(year_arg(year)?),
            None => None,
        };
        let year = match nominated {
            Some(year) => year,
            None => label_year(input)?,
        };
        review_register_file(
            input,
            year,
            Mode {
                gate_only: true,
                myr_only: false,
            },
            nominated,
            Some(&table),
            over.as_ref(),
        )?
    } else {
        let year = year_arg(parsed.get("year").unwrap_or_default())?;
        review_register_file(
            input,
            year,
            Mode {
                gate_only: false,
                myr_only: name == "myr",
            },
            None,
            Some(&table),
            over.as_ref(),
        )?
    };
    if parsed.get("format") == Some("json") {
        emit_json(envelope(name, report.to_json()?), out);
    } else {
        out.push_str(&format!("{BANNER}\n\n"));
        print_report(out, &report, parsed.get("trace").is_some())?;
    }
    Ok(if report.needs_attention() {
        EXIT_ATTENTION
    } else {
        EXIT_OK
    })
}

/// (exit code, stdout, stderr), before platform text-mode newline conversion.
pub fn run(args: &[String]) -> (i32, String, String) {
    let parsed = match parse(PROG, args) {
        Err(Exit::Help(text, code)) => return (code, text, String::new()),
        Err(Exit::Error {
            usage,
            prog,
            message,
        }) => {
            let t = argtheme::theme(Stream::Stderr);
            return (
                2,
                String::new(),
                format!(
                    "{usage}\n{prog}: {}error:{} {}{message}{}\n",
                    t.error, t.reset, t.message, t.reset
                ),
            );
        }
        Ok(parsed) => parsed,
    };
    let mut out = String::new();
    let result = if parsed.command.name == "rate" {
        run_rate(&parsed, &mut out)
    } else {
        run_register(&parsed, &mut out)
    };
    match result {
        Ok(code) => (code, out, String::new()),
        // Python printed `error: <message>` for its own refusals, and a
        // traceback for anything else; this prints the traceback's last line.
        Err(failure) => (EXIT_ERROR, out, format!("error: {failure}\n")),
    }
}

pub fn decode_os(text: &std::ffi::OsStr) -> String {
    let mut out = String::new();
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        for chunk in text.as_bytes().utf8_chunks() {
            for ch in chunk.valid().chars() {
                pytext::push_code_point(&mut out, ch as u32);
            }
            for &byte in chunk.invalid() {
                pytext::push_code_point(&mut out, 0xDC00 + u32::from(byte));
            }
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        for unit in char::decode_utf16(text.encode_wide()) {
            pytext::push_code_point(
                &mut out,
                unit.map_or_else(|e| u32::from(e.unpaired_surrogate()), u32::from),
            );
        }
    }
    out
}

/// Encoded output: both streams use `errors="backslashreplace"`, so a lone
/// surrogate from the command line is written as its escape.
pub fn output_bytes(text: &str) -> Vec<u8> {
    let mut out = Vec::new();
    for cp in pytext::code_points(text) {
        match char::from_u32(cp) {
            None => out.extend_from_slice(format!("\\u{cp:04x}").as_bytes()),
            Some('\n') if cfg!(windows) => out.extend_from_slice(b"\r\n"),
            Some(ch) => out.extend_from_slice(ch.encode_utf8(&mut [0; 4]).as_bytes()),
        }
    }
    out
}
