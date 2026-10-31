//! The `payday-super-check` command line. argparse formatting and option
//! parsing are reused from the earlier engine ports, generalised to the three
//! flat parsers this command builds (check and evidence-pack, import,
//! review-pack). Positional arguments follow Python 3.15's
//! `_parse_known_args`.
//!
//! Exit codes: 0 nothing needs attention; 1 the run could not complete; 2 a
//! finding needs attention, or a usage error.

use std::collections::BTreeMap;

use crate::argtheme::{self, Stream, Theme};
use crate::assess::{self, ON_TIME, Options};
use crate::atomic;
use crate::calendar;
use crate::csv_io::{self, LATEST_SANE_YEAR};
use crate::evidence_pack::{self, PackFacts};
use crate::importers::{
    self, OUTCOME_MATCHED, OUTCOME_OVER, OUTCOME_OWES_NOTHING, OUTCOME_PARTIAL, OUTCOME_UNDATED,
    OUTCOME_UNMATCHED,
};
use crate::practitioner_pack;
use crate::profiles::Profile;
use crate::py::{self, Date, PyPath};
use crate::pyre::Regex;
use crate::pytext;
use crate::rates;
use crate::raw::Raw;
use crate::report::{self, NoteFacts, SummaryFacts};
use crate::textwrap::wrap;
use crate::{Failure, LAW_CONTENT_DATE, VERSION};

const PROG: &str = "payday-super-check";
const HELP_HELP: &str = "show this help message and exit";
const VERSION_HELP: &str = "show program's version number and exit";
const EXIT_OK: i32 = 0;
const EXIT_ERROR: i32 = 1;
const EXIT_LATE_FOUND: i32 = 2;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Help,
    Store,
    Append,
    Flag,
    Version,
}

struct Opt {
    flags: &'static [&'static str],
    dest: &'static str,
    metavar: &'static str,
    kind: Kind,
    required: bool,
    help: String,
}

const fn kind_opt(
    flags: &'static [&'static str],
    dest: &'static str,
    metavar: &'static str,
    kind: Kind,
) -> (&'static [&'static str], &'static str, &'static str, Kind) {
    (flags, dest, metavar, kind)
}

fn opt(
    spec: (&'static [&'static str], &'static str, &'static str, Kind),
    required: bool,
    help: &str,
) -> Opt {
    Opt {
        flags: spec.0,
        dest: spec.1,
        metavar: spec.2,
        kind: spec.3,
        required,
        help: help.to_string(),
    }
}

struct Parser {
    prog: String,
    description: &'static str,
    positional: Option<(&'static str, &'static str)>,
    /// Every option, `-h` first, in the order argparse registers them.
    options: Vec<Opt>,
}

const CHECK_DESCRIPTION: &str = "Experimental review of super contributions against payday-super deadlines \
     (SGAA 1992 s 18C, in force for paydays from 1 July 2026) and \
     experimental SG-charge exposure estimates on established late ones.";

fn help_opt() -> Opt {
    opt(
        kind_opt(&["-h", "--help"], "help", "", Kind::Help),
        false,
        HELP_HELP,
    )
}

fn check_parser(evidence_pack: bool) -> Parser {
    Parser {
        prog: if evidence_pack {
            format!("{PROG} evidence-pack")
        } else {
            PROG.to_string()
        },
        description: CHECK_DESCRIPTION,
        positional: Some(("csv_path", "contribution CSV to check")),
        options: vec![
            help_opt(),
            opt(
                kind_opt(&["-o", "--output"], "output", "OUTPUT", Kind::Store),
                false,
                if evidence_pack {
                    "new pack directory (default: evidence-pack)"
                } else {
                    "report CSV to write (default: report.csv)"
                },
            ),
            opt(
                kind_opt(&["--as-at"], "as_at", "AS_AT", Kind::Store),
                false,
                "date to measure unpaid notional earnings to (YYYY-MM-DD, default: today)",
            ),
            opt(
                kind_opt(
                    &["--assessment-date"],
                    "assessment_date",
                    "ASSESSMENT_DATE",
                    Kind::Store,
                ),
                false,
                "date the ATO assessed the SG charge for these paydays (YYYY-MM-DD). \
                 Only contributions received before it clear the shortfall under s 18D. \
                 Omit if no assessment has issued.",
            ),
            opt(
                kind_opt(
                    &["--mapping-file"],
                    "mapping_file",
                    "MAPPING_FILE",
                    Kind::Store,
                ),
                false,
                "JSON file mapping canonical field names to your CSV columns",
            ),
            opt(
                kind_opt(&["--map"], "map", "FIELD=COLUMN", Kind::Append),
                false,
                "override one column mapping; repeatable (e.g. --map qe_day=PayDate)",
            ),
            opt(
                kind_opt(&["--events"], "events", "EVENTS", Kind::Store),
                false,
                "CSV of dated, amount-specific fund receipts and returns for the rows in \
                 the input (see README). An event that matches no row is refused",
            ),
            opt(
                kind_opt(
                    &["--holidays-override"],
                    "holidays_override",
                    "HOLIDAYS_OVERRIDE",
                    Kind::Store,
                ),
                false,
                "JSON file adding or removing public holidays from the bundled calendar",
            ),
            opt(
                kind_opt(
                    &["--confirm-transition-allocation"],
                    "confirm_transition_allocation",
                    "",
                    Kind::Flag,
                ),
                false,
                "confirm you reconciled every contribution dated no later than 28 Jul \
                 2026 under LCR 2026/1: pre-1 Jul amounts are unused excess and 1-28 Jul \
                 amounts remain after any June-quarter employee shortfall",
            ),
            opt(
                kind_opt(&["--allow-stale-gic"], "allow_stale_gic", "", Kind::Flag),
                false,
                "estimate notional earnings past the last published GIC quarter at the \
                 last known rate, instead of refusing. The report says it did so",
            ),
            opt(
                kind_opt(
                    &["--confirm-remittance-only"],
                    "confirm_remittance_only",
                    "",
                    Kind::Flag,
                ),
                false,
                "confirm you accept a remittance-only review: no in-scope positive row \
                 has a fund-receipt date on or before the as-at date, so the file \
                 cannot produce ON_TIME",
            ),
            opt(
                kind_opt(&["--version"], "version", "", Kind::Version),
                false,
                VERSION_HELP,
            ),
        ],
    }
}

fn import_parser() -> Parser {
    Parser {
        prog: format!("{PROG} import"),
        description: "Build the contributions CSV from a payroll export and a super \
             payments export after the statutory allocation is reconciled. \
             The supported export profiles do not establish a fund receipt date, \
             so fund_received_date is always left blank for you to fill in.",
        positional: None,
        options: vec![
            help_opt(),
            opt(
                kind_opt(&["--payroll"], "payroll", "PAYROLL", Kind::Store),
                true,
                "payroll activity export CSV",
            ),
            opt(
                kind_opt(&["--super"], "super_path", "SUPER_PATH", Kind::Store),
                true,
                "super payments export CSV",
            ),
            opt(
                kind_opt(&["-o", "--output"], "output", "OUTPUT", Kind::Store),
                false,
                "canonical CSV to write (default: contributions.csv)",
            ),
            opt(
                kind_opt(&["--vendor"], "vendor", "VENDOR", Kind::Store),
                false,
                "force a profile instead of detecting one (e.g. xero, myob-ar, employment-hero)",
            ),
            opt(
                kind_opt(
                    &["--confirm-statutory-allocation"],
                    "confirm_statutory_allocation",
                    "",
                    Kind::Flag,
                ),
                false,
                "confirm you reconciled LCR 2026/2 allocation using actual fund-receipt \
                 order, every relevant payday and contribution, and any SG-charge \
                 assessment; required when one employee has multiple positive paydays",
            ),
        ],
    }
}

fn review_pack_parser() -> Parser {
    Parser {
        prog: format!("{PROG} review-pack"),
        description: "Build a deterministic, privacy-conscious practitioner checklist from \
             a payday-super-checker report CSV. The source CSV remains the row-level \
             workpaper and every professional judgement remains human-only.",
        positional: Some(("report_csv", "report CSV produced by payday-super-check")),
        options: vec![
            help_opt(),
            opt(
                kind_opt(&["-o", "--output"], "output", "OUTPUT", Kind::Store),
                false,
                "Markdown review pack to write (default: practitioner-review.md)",
            ),
        ],
    }
}

fn is_long(flag: &str) -> bool {
    flag.len() > 2
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

/// `_format_usage`'s `get_lines`.
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

type Row = (usize, Part, Option<String>);

/// The rows of one help section; argparse aligns help across every section.
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

fn heading(text: &str, t: &Theme) -> String {
    format!("{}{text}:{}", t.heading, t.reset)
}

fn flag_colour<'a>(flag: &str, long: &'a str, short: &'a str) -> &'a str {
    if is_long(flag) { long } else { short }
}

fn usage_text(parser: &Parser, t: &Theme) -> String {
    let mut parts = Vec::new();
    for option in &parser.options {
        let flag = option.flags[0];
        let colour = flag_colour(flag, t.summary_long_option, t.summary_short_option);
        let (plain, coloured) = if option.metavar.is_empty() {
            (flag.to_string(), format!("{colour}{flag}{}", t.reset))
        } else {
            (
                format!("{flag} {}", option.metavar),
                format!(
                    "{colour}{flag} {}{}{}",
                    t.summary_label, option.metavar, t.reset
                ),
            )
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
    let positional: Vec<Part> = parser
        .positional
        .iter()
        .map(|(name, _)| coloured(name, t.summary_action, t))
        .collect();
    usage(&parser.prog, &parts, &positional, t)
}

fn help_text(parser: &Parser, t: &Theme) -> String {
    let positional: Vec<Row> = parser
        .positional
        .iter()
        .map(|(name, help)| (2, coloured(name, t.action, t), Some(help.to_string())))
        .collect();
    let mut options: Vec<Row> = Vec::new();
    for option in &parser.options {
        let names: Vec<Part> = option
            .flags
            .iter()
            .map(|f| coloured(f, flag_colour(f, t.long_option, t.short_option), t))
            .collect();
        let plain_names = names
            .iter()
            .map(|p| p.plain.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let coloured_names = names
            .iter()
            .map(|p| p.coloured.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let part = if option.metavar.is_empty() {
            Part {
                plain: plain_names,
                coloured: coloured_names,
            }
        } else {
            Part {
                plain: format!("{plain_names} {}", option.metavar),
                coloured: format!("{coloured_names} {}{}{}", t.label, option.metavar, t.reset),
            }
        };
        options.push((2, part, Some(option.help.clone())));
    }
    let all: Vec<Row> = positional.iter().chain(&options).cloned().collect();
    let description = wrap(parser.description, width().max(11)).join("\n");
    let mut out = format!("{}\n\n{description}\n\n", usage_text(parser, t));
    if !positional.is_empty() {
        out.push_str(&format!(
            "{}\n{}\n",
            heading("positional arguments", t),
            action_rows(&positional, &all)
        ));
    }
    out.push_str(&format!(
        "{}\n{}",
        heading("options", t),
        action_rows(&options, &all)
    ));
    out
}

/// What argparse does instead of returning a namespace.
enum Exit {
    Help(String),
    Error(String),
}

/// How argparse's `_parse_optional` classifies one token.
enum Class<'a> {
    Flag(&'a str, Option<(String, bool)>),
    UnknownOption,
    Ambiguous(String),
    Positional,
}

fn classify<'a>(token: &str, flags: &[&'a str]) -> Class<'a> {
    if token.is_empty() || !token.starts_with('-') {
        return Class::Positional;
    }
    if let Some(exact) = flags.iter().find(|f| **f == token) {
        return Class::Flag(exact, None);
    }
    if pytext::py_len(token) == 1 {
        return Class::Positional;
    }
    if let Some((name, value)) = token.split_once('=')
        && let Some(exact) = flags.iter().find(|f| **f == name)
    {
        return Class::Flag(exact, Some((value.to_string(), true)));
    }
    let mut found = option_prefix_matches(token, flags);
    match found.len() {
        0 => {}
        1 => {
            let (flag, explicit) = found.remove(0);
            return Class::Flag(flag, explicit);
        }
        _ => {
            let names: Vec<&str> = found.iter().map(|(f, _)| *f).collect();
            return Class::Ambiguous(format!(
                "ambiguous option: {token} could match {}",
                names.join(", ")
            ));
        }
    }
    if looks_like_negative_number(&token[1..]) || token.contains(' ') {
        return Class::Positional;
    }
    Class::UnknownOption
}

/// argparse's `_get_option_tuples`.
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
    let mut chars = body.chars();
    match chars.next() {
        Some('.') => chars.next().is_some_and(digit),
        Some(c) => digit(c),
        None => false,
    }
}

fn action_name(option: &Opt) -> String {
    option.flags.join("/")
}

/// argparse's refusal of an explicit argument to a zero-argument option.
/// `-hx` is read as further short flags instead, and help runs first.
fn ignored_argument(
    option: &Opt,
    flag: &str,
    explicit: Option<(String, bool)>,
) -> Result<(), String> {
    match explicit {
        None => Ok(()),
        Some((rest, false)) if flag == "-h" && !rest.is_empty() && !rest.starts_with('-') => Ok(()),
        Some((rest, _)) => Err(format!(
            "argument {}: ignored explicit argument {}",
            action_name(option),
            pytext::repr_os(&rest)
        )),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pattern {
    Option(usize),
    Ambiguous(usize),
    Unknown,
    Arg,
    Marker,
}

#[derive(Default)]
struct Parsed {
    values: BTreeMap<&'static str, String>,
    lists: BTreeMap<&'static str, Vec<String>>,
    flags: Vec<&'static str>,
    positional: Option<String>,
}

impl Parsed {
    fn get(&self, dest: &str) -> Option<&str> {
        self.values.get(dest).map(String::as_str)
    }

    fn flag(&self, dest: &str) -> bool {
        self.flags.contains(&dest)
    }
}

/// `parser.parse_args(args)`: `_parse_known_args`, then the extras check.
fn parse(parser: &Parser, args: &[String]) -> Result<Parsed, Exit> {
    let error = |message: String| Exit::Error(message);
    let flags: Vec<&'static str> = parser
        .options
        .iter()
        .flat_map(|o| o.flags.iter().copied())
        .collect();
    let mut patterns = Vec::with_capacity(args.len());
    let mut explicit: Vec<Option<(String, bool)>> = vec![None; args.len()];
    let mut chosen: Vec<&'static str> = vec![""; args.len()];
    let mut refusals: Vec<String> = vec![String::new(); args.len()];
    let mut marked = false;
    for (i, token) in args.iter().enumerate() {
        if marked {
            patterns.push(Pattern::Arg);
        } else if token == "--" {
            marked = true;
            patterns.push(Pattern::Marker);
        } else {
            patterns.push(match classify(token, &flags) {
                Class::Flag(flag, value) => {
                    chosen[i] = flag;
                    explicit[i] = value;
                    Pattern::Option(i)
                }
                Class::UnknownOption => Pattern::Unknown,
                Class::Ambiguous(message) => {
                    refusals[i] = message;
                    Pattern::Ambiguous(i)
                }
                Class::Positional => Pattern::Arg,
            });
        }
    }
    let is_option = |p: Pattern| {
        matches!(
            p,
            Pattern::Option(_) | Pattern::Ambiguous(_) | Pattern::Unknown
        )
    };
    let last_option = patterns.iter().rposition(|p| is_option(*p));
    let mut parsed = Parsed::default();
    let mut extras: Vec<String> = Vec::new();
    let wants_positional = parser.positional.is_some();
    let mut have_positional = false;
    let mut consume_positional = |start: usize, parsed: &mut Parsed| -> usize {
        if !wants_positional || have_positional {
            return start;
        }
        let mut i = start;
        if patterns.get(i) == Some(&Pattern::Marker) {
            i += 1;
        }
        if patterns.get(i) != Some(&Pattern::Arg) {
            return start;
        }
        let value = args[i].clone();
        i += 1;
        if patterns.get(i) == Some(&Pattern::Marker) {
            i += 1;
        }
        parsed.positional = Some(value);
        have_positional = true;
        i
    };
    let mut start = 0;
    while last_option.is_some_and(|last| start <= last) {
        let next_option = (start..patterns.len())
            .find(|&i| is_option(patterns[i]))
            .unwrap_or(patterns.len());
        if start != next_option {
            let end = consume_positional(start, &mut parsed);
            if end > start {
                start = end;
                continue;
            }
            extras.extend(args[start..next_option].iter().cloned());
            start = next_option;
        }
        match patterns[start] {
            Pattern::Unknown => {
                extras.push(args[start].clone());
                start += 1;
            }
            Pattern::Ambiguous(i) => return Err(error(refusals[i].clone())),
            Pattern::Option(i) => {
                let flag = chosen[i];
                let option = parser
                    .options
                    .iter()
                    .find(|o| o.flags.contains(&flag))
                    .expect("a known flag");
                match option.kind {
                    Kind::Help => {
                        ignored_argument(option, flag, explicit[i].clone()).map_err(error)?;
                        return Err(Exit::Help(help_text(
                            parser,
                            argtheme::theme(Stream::Stdout),
                        )));
                    }
                    Kind::Version => {
                        ignored_argument(option, flag, explicit[i].clone()).map_err(error)?;
                        // The version action formats through the help
                        // formatter, so a narrow terminal wraps it.
                        let text =
                            wrap(&format!("{} {VERSION}", parser.prog), width().max(11)).join("\n");
                        return Err(Exit::Help(format!("{text}\n")));
                    }
                    Kind::Flag => {
                        ignored_argument(option, flag, explicit[i].clone()).map_err(error)?;
                        if !parsed.flags.contains(&option.dest) {
                            parsed.flags.push(option.dest);
                        }
                        start += 1;
                    }
                    Kind::Store | Kind::Append => {
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
                                return Err(error(format!(
                                    "argument {}: expected one argument",
                                    action_name(option)
                                )));
                            }
                        };
                        if option.kind == Kind::Append {
                            parsed.lists.entry(option.dest).or_default().push(value);
                        } else {
                            parsed.values.insert(option.dest, value);
                        }
                    }
                }
            }
            Pattern::Arg | Pattern::Marker => unreachable!("an option starts here"),
        }
    }
    let stop = consume_positional(start, &mut parsed);
    extras.extend(args[stop..].iter().cloned());

    let mut missing: Vec<String> = Vec::new();
    for option in &parser.options {
        if option.required && !parsed.values.contains_key(option.dest) {
            missing.push(action_name(option));
        }
    }
    if let Some((name, _)) = parser.positional
        && !have_positional
    {
        missing.push(name.to_string());
    }
    if !missing.is_empty() {
        // argparse lists required actions in registration order: the
        // positional of these parsers is registered before every option.
        if let Some((name, _)) = parser.positional
            && missing.last().is_some_and(|m| m == name)
        {
            missing.rotate_right(1);
        }
        return Err(error(format!(
            "the following arguments are required: {}",
            missing.join(", ")
        )));
    }
    if !extras.is_empty() {
        return Err(error(format!(
            "unrecognized arguments: {}",
            extras.join(" ")
        )));
    }
    Ok(parsed)
}

/// Run a parser: the parsed arguments, or the output argparse produced.
fn run_parser(parser: &Parser, args: &[String]) -> Result<Parsed, (i32, String, String)> {
    match parse(parser, args) {
        Ok(parsed) => Ok(parsed),
        Err(Exit::Help(text)) => Err((0, text, String::new())),
        Err(Exit::Error(message)) => {
            let t = argtheme::theme(Stream::Stderr);
            Err((
                2,
                String::new(),
                format!(
                    "{}\n{}: {}error:{} {}{message}{}\n",
                    usage_text(parser, t),
                    parser.prog,
                    t.error,
                    t.reset,
                    t.message,
                    t.reset
                ),
            ))
        }
    }
}

/// Output gathered for the two streams, in the order Python printed it.
#[derive(Default)]
struct Streams {
    out: String,
    err: String,
}

impl Streams {
    fn out(&mut self, line: &str) {
        self.out.push_str(line);
        self.out.push('\n');
    }

    fn err(&mut self, line: &str) {
        self.err.push_str(line);
        self.err.push('\n');
    }

    fn done(self, code: i32) -> (i32, String, String) {
        (code, self.out, self.err)
    }
}

/// A failure Python left uncaught: its traceback's last line, exit 1.
fn uncaught(mut s: Streams, failure: &Failure) -> (i32, String, String) {
    s.err(&format!("error: {}", failure.traceback_line()));
    s.done(EXIT_ERROR)
}

fn parse_cli_date(value: Option<&str>, flag: &str) -> Result<Option<Date>, Failure> {
    let Some(value) = value else { return Ok(None) };
    let parsed = csv_io::parse_date_text(value)
        .map_err(|e| Failure::Value(format!("{flag} {}", e.message())))?;
    let Some(parsed) = parsed else {
        return Err(Failure::Value(format!(
            "{flag} expects a date such as 2026-08-10 or 10/08/2026, got {}",
            pytext::repr_os(value)
        )));
    };
    if parsed.year > LATEST_SANE_YEAR {
        return Err(Failure::Value(format!(
            "{flag} value {} is not a real date",
            pytext::repr_os(value)
        )));
    }
    Ok(Some(parsed))
}

fn resolved(path: &str) -> String {
    py::resolve_text(&py::fs_path(path))
}

/// The check and evidence-pack commands.
fn check_main(args: &[String], evidence_pack: bool) -> (i32, String, String) {
    let parser = check_parser(evidence_pack);
    let parsed = match run_parser(&parser, args) {
        Ok(parsed) => parsed,
        Err(done) => return done,
    };
    let csv_path = parsed.positional.clone().expect("required");
    let output = parsed
        .get("output")
        .unwrap_or(if evidence_pack {
            "evidence-pack"
        } else {
            "report.csv"
        })
        .to_string();
    let mut s = Streams::default();
    let prepared = (|| -> Result<_, Failure> {
        let as_at = match parse_cli_date(parsed.get("as_at"), "--as-at")? {
            Some(d) => d,
            None => {
                let today = Date::today();
                s.err(&format!(
                    "note: --as-at not supplied; assuming {} from this \
                     machine's clock, which may not be today's date in Australia. Pass \
                     --as-at explicitly in scheduled runs.",
                    today.isoformat()
                ));
                today
            }
        };
        let assessment_date = parse_cli_date(parsed.get("assessment_date"), "--assessment-date")?;
        let out = resolved(&output);
        for (value, label) in [
            (Some(csv_path.as_str()), "the input file"),
            (parsed.get("mapping_file"), "the --mapping-file input"),
            (parsed.get("events"), "the --events input"),
            (
                parsed.get("holidays_override"),
                "the --holidays-override input",
            ),
        ] {
            if let Some(value) = value
                && py::same_resolved(&resolved(value), &out)
            {
                return Err(Failure::Value(format!(
                    "the report would overwrite {label} {value}. Choose a \
                     different path with -o."
                )));
            }
        }
        if evidence_pack {
            evidence_pack::evidence_destination(&output)?;
        } else {
            atomic::csv_destination(&output)?;
        }
        let overrides = parsed.lists.get("map").cloned().unwrap_or_default();
        let mapping = csv_io::load_mapping(parsed.get("mapping_file"), &overrides)?;
        let mut lines = csv_io::parse_rows(&csv_path, &mapping)?;
        if let Some(events) = parsed.get("events") {
            csv_io::attach_events(events, &mut lines)?;
        }
        let cal = calendar::load_calendar(parsed.get("holidays_override"))?;
        let gic = rates::load_gic()?;
        let rates = if evidence_pack {
            Raw::Dict(Vec::new())
        } else {
            rates::load_rates()?
        };
        let results = assess::assess(
            lines,
            &cal,
            &gic,
            as_at,
            Options {
                assessment_date,
                transition_allocation_confirmed: parsed.flag("confirm_transition_allocation"),
                allow_stale_gic: parsed.flag("allow_stale_gic"),
            },
        )?;
        Ok((as_at, assessment_date, gic, rates, results))
    })();
    let (as_at, assessment_date, gic, rates, results) = match prepared {
        Ok(prepared) => prepared,
        Err(Failure::Overflow) => {
            s.err(
                "error: a date in this file is too far in the future to work with. \
                 Check for placeholder dates such as 9999-12-31.",
            );
            return s.done(EXIT_ERROR);
        }
        Err(f) if f.is_value_or_arithmetic() => {
            s.err(&format!("error: {}", f.message()));
            return s.done(EXIT_ERROR);
        }
        Err(Failure::Os(e)) if e.not_found() => {
            s.err(&format!(
                "error: file not found: {}",
                e.filename.clone().unwrap_or_default()
            ));
            return s.done(EXIT_ERROR);
        }
        Err(Failure::Os(e)) => {
            let target = e.filename.clone().unwrap_or_else(|| csv_path.clone());
            s.err(&format!("error: cannot read {target}: {}", strerror(&e)));
            return s.done(EXIT_ERROR);
        }
        Err(f) => return uncaught(s, &f),
    };
    let remittance_only = parsed.flag("confirm_remittance_only");

    if evidence_pack {
        let tables: Vec<rates::TableSource> = gic.source.iter().cloned().collect();
        let built = evidence_pack::build_evidence_pack(
            &results,
            &PackFacts {
                as_at,
                assessment_date,
                gic_provenance: &gic.provenance(),
                remittance_only_confirmed: remittance_only,
                rate_tables: &tables,
            },
        )
        .and_then(|files| evidence_pack::write_evidence_pack(&files, &output));
        match built {
            Ok(()) => {}
            Err(
                f @ (Failure::Os(_)
                | Failure::Value(_)
                | Failure::StaleGic(_)
                | Failure::Signal(_)
                | Failure::Overflow),
            ) => {
                s.err(&format!("error: {}", f.message()));
                return s.done(EXIT_ERROR);
            }
            Err(f) => return uncaught(s, &f),
        }
        s.out(&format!("wrote evidence pack to {output}"));
        let exceptions = results.iter().any(|r| r.verdict != ON_TIME);
        return s.done(if exceptions { EXIT_LATE_FOUND } else { EXIT_OK });
    }

    let source = resolved(&csv_path);
    let written = report::render_csv(
        &results,
        &NoteFacts {
            as_at,
            law_date: LAW_CONTENT_DATE,
            assessment_date,
            source: Some(&source),
            gic_provenance: &gic.provenance(),
        },
        true,
    )
    .and_then(|text| {
        atomic::write(
            &output,
            format!("\u{feff}{text}").as_bytes(),
            atomic::csv_destination,
        )
    });
    match written {
        Ok(()) => {}
        Err(Failure::Os(e)) => {
            s.err(&format!("error: cannot write {output}: {}", strerror(&e)));
            return s.done(EXIT_ERROR);
        }
        Err(f) if f.is_value_or_arithmetic() => {
            s.err(&format!("error: {}", f.message()));
            return s.done(EXIT_ERROR);
        }
        Err(f) => return uncaught(s, &f),
    }
    let shown = PyPath::new(&output).display();
    let summary = report::console_summary(
        &results,
        &SummaryFacts {
            as_at,
            csv_path: &shown,
            law_date: LAW_CONTENT_DATE,
            rates: &rates,
            assessment_date,
            remittance_only_confirmed: remittance_only,
        },
    );
    match summary {
        Ok(text) => s.out(&text),
        Err(f) if f.is_value_or_arithmetic() => {
            s.err(&format!(
                "error: {}. The report was still written to {output}; \
                 only this console summary failed.",
                f.message()
            ));
            return s.done(EXIT_ERROR);
        }
        Err(f) => return uncaught(s, &f),
    }
    match report::needs_attention(&results, remittance_only) {
        Ok(true) => s.done(EXIT_LATE_FOUND),
        Ok(false) => s.done(EXIT_OK),
        Err(f) => uncaught(s, &f),
    }
}

/// `exc.strerror or exc`.
fn strerror(e: &crate::OsError) -> String {
    if e.strerror.is_empty() {
        e.display()
    } else {
        e.strerror.clone()
    }
}

/// How many warning lines the console shows in full before a count.
const MAX_WARNINGS_SHOWN: usize = 20;

fn profile_line(label: &str, profile: &Profile) -> String {
    let marker = if profile.verified {
        ""
    } else {
        " (unverified against a real export)"
    };
    format!(
        "{label} profile: {} -- {}{marker}",
        profile.key, profile.name
    )
}

fn import_main(args: &[String]) -> (i32, String, String) {
    let parser = import_parser();
    let parsed = match run_parser(&parser, args) {
        Ok(parsed) => parsed,
        Err(done) => return done,
    };
    let payroll = parsed.get("payroll").expect("required").to_string();
    let super_path = parsed.get("super_path").expect("required").to_string();
    let output = parsed
        .get("output")
        .unwrap_or("contributions.csv")
        .to_string();
    let mut s = Streams::default();
    let report = match importers::import_files(
        &payroll,
        &super_path,
        &output,
        parsed.get("vendor"),
        parsed.flag("confirm_statutory_allocation"),
    ) {
        Ok(report) => report,
        Err(f) if f.is_value_or_arithmetic() => {
            s.err(&format!("error: {}", f.message()));
            return s.done(EXIT_ERROR);
        }
        Err(Failure::Os(e)) if e.not_found() => {
            s.err(&format!(
                "error: file not found: {}",
                e.filename.clone().unwrap_or_default()
            ));
            return s.done(EXIT_ERROR);
        }
        Err(Failure::Os(e)) => {
            let target = e
                .filename
                .clone()
                .unwrap_or_else(|| format!("{payroll} or {super_path}"));
            let writing = match &e.filename {
                None => false,
                Some(name) => match atomic::csv_destination(&output) {
                    Ok(destination) => *name == destination.display(),
                    // Python raised this inside the handler, uncaught.
                    Err(f) => return uncaught(s, &Failure::Uncaught(f.traceback_line())),
                },
            };
            let verb = if writing {
                "cannot write"
            } else {
                "cannot read"
            };
            s.err(&format!("error: {verb} {target}: {}", strerror(&e)));
            return s.done(EXIT_ERROR);
        }
        Err(f) => return uncaught(s, &f),
    };

    let mut lines = vec![
        profile_line("payroll", &report.payroll_profile),
        profile_line("super", &report.super_profile),
        format!("employee matching: by {}", report.key_mode),
        String::new(),
        "No supported payroll or clearing-house export carries a fund receipt date, \
         so fund_received_date is written blank on every row. The checker's \
         deadline tests receipt by the fund, not remittance -- fill that \
         column in from your fund or clearing house before relying on any \
         verdict it produces."
            .into(),
        "A dated part payment writes remitted_date and remitted_amount; \
         sg_amount stays the amount owed. matched_amount separately records \
         the total associated with the payday, even when the vendor supplied \
         no payment date, and caps any fund receipt later added to the row. \
         Where a match contains dated and undated super rows, remitted_date is \
         conservatively the latest known date for the dated subtotal: the \
         checker credits none of the subtotal before that date and only \
         remitted_amount afterwards. An entirely undated match leaves both \
         remittance fields blank but keeps matched_amount. The warning lines \
         below still name every \
         partial: <received> of <owed> matched and every \"... has no \
         payment date on record\" figure."
            .into(),
    ];
    if !report.warnings.is_empty() {
        let row_level = Regex::new(r"^(row|super row) \d+: ");
        let uncappable = Regex::new(
            r"^row \d+: (partial|over): |carry no payment date|matched has no payment date on record",
        );
        lines.push(String::new());
        lines.push(format!("warnings ({}):", report.warnings.len()));
        let (row_lines, structural): (Vec<&String>, Vec<&String>) = report
            .warnings
            .iter()
            .partition(|w| row_level.matches_start(w));
        lines.extend(structural.iter().map(|w| format!("  - {w}")));
        let (figures, other): (Vec<&String>, Vec<&String>) =
            row_lines.into_iter().partition(|w| uncappable.search(w));
        lines.extend(figures.iter().map(|w| format!("  - {w}")));
        lines.extend(
            other
                .iter()
                .take(MAX_WARNINGS_SHOWN)
                .map(|w| format!("  - {w}")),
        );
        if other.len() > MAX_WARNINGS_SHOWN {
            lines.push(format!(
                "  ... and {} more (none of them a partial, \
                 over-payment or missing-remittance-date figure -- those are \
                 always shown in full above)",
                other.len() - MAX_WARNINGS_SHOWN
            ));
        }
    }
    if !report.orphan_reasons.is_empty() {
        lines.push(String::new());
        lines.push(format!(
            "super payments that were not applied to any payday ({}):",
            report.orphan_reasons.len()
        ));
        let mut counts = report.orphan_counts();
        counts.sort_by(|a, b| a.0.cmp(&b.0));
        for (code, count) in counts {
            lines.push(format!("  {count}  {code}"));
        }
    }
    lines.extend([
        String::new(),
        format!(
            "paydays: matched {}, no remittance date {}, partial {}, over {}, \
             owes nothing {}, no payment found {}",
            report.count(OUTCOME_MATCHED),
            report.count(OUTCOME_UNDATED),
            report.count(OUTCOME_PARTIAL),
            report.count(OUTCOME_OVER),
            report.count(OUTCOME_OWES_NOTHING),
            report.count(OUTCOME_UNMATCHED)
        ),
        format!("wrote {output}"),
    ]);
    s.out(&lines.join("\n"));
    s.done(if report.clean() {
        EXIT_OK
    } else {
        EXIT_LATE_FOUND
    })
}

fn review_pack_main(args: &[String]) -> (i32, String, String) {
    let parser = review_pack_parser();
    let parsed = match run_parser(&parser, args) {
        Ok(parsed) => parsed,
        Err(done) => return done,
    };
    let report_csv = parsed.positional.clone().expect("required");
    let requested = parsed
        .get("output")
        .unwrap_or("practitioner-review.md")
        .to_string();
    let mut output = PyPath::new(&requested);
    let mut s = Streams::default();
    let result = (|| -> Result<practitioner_pack::ReportSnapshot, Failure> {
        if same_local_path(&report_csv, &requested) {
            return Err(Failure::Value(format!(
                "the review pack would overwrite the input report {report_csv}. \
                 Choose a different path with -o."
            )));
        }
        output = atomic::markdown_destination(&requested)?;
        let snapshot = practitioner_pack::load_report_snapshot(&report_csv)?;
        practitioner_pack::write_practitioner_pack(&snapshot, &output.display())?;
        Ok(snapshot)
    })();
    let snapshot = match result {
        Ok(snapshot) => snapshot,
        Err(f @ (Failure::Value(_) | Failure::StaleGic(_))) => {
            s.err(&format!("error: {}", f.message()));
            return s.done(EXIT_ERROR);
        }
        Err(Failure::Os(e)) if e.not_found() => {
            s.err(&format!(
                "error: file not found: {}",
                e.filename.clone().unwrap_or_default()
            ));
            return s.done(EXIT_ERROR);
        }
        Err(Failure::Os(e)) => {
            let target = e.filename.clone().unwrap_or_else(|| report_csv.clone());
            let verb = if e.filename.as_deref() == Some(output.display().as_str()) {
                "cannot write"
            } else {
                "cannot read"
            };
            s.err(&format!("error: {verb} {target}: {}", strerror(&e)));
            return s.done(EXIT_ERROR);
        }
        Err(f) => return uncaught(s, &f),
    };
    s.out(&format!("wrote {output}"));
    s.done(if snapshot.needs_attention() {
        EXIT_LATE_FOUND
    } else {
        EXIT_OK
    })
}

/// `normcase(realpath(left)) == normcase(realpath(right))`.
fn same_local_path(left: &str, right: &str) -> bool {
    py::same_resolved(&resolved(left), &resolved(right))
}

/// (exit code, stdout, stderr), before platform text-mode newline conversion.
pub fn run(args: &[String]) -> (i32, String, String) {
    match args.first().map(String::as_str) {
        Some("import") => import_main(&args[1..]),
        Some("review-pack") => review_pack_main(&args[1..]),
        Some("evidence-pack") => check_main(&args[1..], true),
        _ => check_main(args, false),
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
    // No command line reaches the WebAssembly build.
    #[cfg(not(any(unix, windows)))]
    for ch in text.to_string_lossy().chars() {
        pytext::push_code_point(&mut out, ch as u32);
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

use crate::{api, derivation};
use serde_json::Value;

/// One JSON object from stdin or, for `verify FILE`, a file, bounded at
/// 8 MiB before the complete input is allocated.
fn read_json(path: Option<&str>) -> Result<Value, String> {
    use std::io::Read;
    let mut bytes = Vec::new();
    let limit = (8 << 20) + 1;
    match path {
        None | Some("-") => std::io::stdin().take(limit).read_to_end(&mut bytes),
        Some(path) => std::fs::File::open(path)
            .and_then(|f| f.take(limit).read_to_end(&mut bytes))
            .map_err(|e| std::io::Error::new(e.kind(), format!("{path}: {e}"))),
    }
    .map_err(|e| e.to_string())?;
    if bytes.len() > 8 << 20 {
        return Err("JSON input exceeds 8 MiB".into());
    }
    serde_json::from_slice(&bytes).map_err(|e| format!("invalid JSON: {e}"))
}

/// One JSON stdin command: (exit code, stdout, stderr).
fn run_api(command: &str) -> (i32, String, String) {
    match read_json(None).and_then(|v| api::evaluate(command, &v)) {
        Ok(v) => (0, format!("{v}\n"), String::new()),
        Err(m) => (2, String::new(), format!("error: {m}\n")),
    }
}

/// `derive` writes the derivation; `verify` exits 1 when a step differs.
fn run_derivation(command: &str, path: Option<&str>) -> (i32, String, String) {
    let result = read_json(path).and_then(|v| {
        if command == "derive" {
            derivation::derive(&v)
        } else {
            derivation::verify(&v)
        }
    });
    match result {
        Ok(v) => (
            i32::from(v["verified"] == false),
            format!("{}\n", serde_json::to_string_pretty(&v).expect("JSON")),
            String::new(),
        ),
        Err(m) => (2, String::new(), format!("error: {m}\n")),
    }
}

/// Shared command routing for the standalone binary and umbrella CLI.
pub fn dispatch(args: &[String]) -> (i32, String, String) {
    match args {
        [command] if api::COMMANDS.contains(&command.as_str()) => run_api(command),
        [command] if command == "derive" || command == "verify" => run_derivation(command, None),
        [command, path] if command == "verify" => run_derivation(command, Some(path)),
        [flag] if flag == "--api-help" => (
            0,
            format!(
                "JSON stdin commands: {}\nSee README.md for their schemas.\n",
                api::COMMANDS.join(", ")
            ),
            String::new(),
        ),
        _ => run(args),
    }
}
