//! The `wip-tally` command line. argparse formatting and option parsing are
//! reused from the Solomon's Sword and Exchequer ports; positional arguments
//! follow Python 3.15's `_parse_known_args` (the first `--` marks the rest as
//! positional and is removed from the positional span that contains it).
//!
//! Exit codes: 0 schedule produced with no review flags; 1 the run could not
//! produce a schedule; 2 schedule produced with a review flag, or a usage error.
use crate::argtheme::{self, Stream, Theme};
use crate::csv_io::{self, Failure};
use crate::engine::{self, Schedule};
use crate::py::{self, PyPath};
use crate::pytext;
use crate::report;
use crate::textwrap::wrap;
use std::collections::BTreeMap;

const PROG: &str = "wip-tally";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const DESCRIPTION: &str = "Build an AASB 15 construction WIP schedule from a contract CSV. Nothing leaves this machine. Review aid only, not a determination.";
const HELP_HELP: &str = "show this help message and exit";
const VERSION_HELP: &str = "show program's version number and exit";
const EXIT_OK: i32 = 0;
const EXIT_ERROR: i32 = 1;
const EXIT_REVIEW: i32 = 2;

/// An option taking one value.
struct Opt {
    flags: &'static [&'static str],
    dest: &'static str,
    required: bool,
    help: Option<&'static str>,
}

struct Command {
    name: &'static str,
    help: &'static str,
    positional: (&'static str, &'static str),
    options: &'static [Opt],
}

const fn opt(
    flags: &'static [&'static str],
    dest: &'static str,
    required: bool,
    help: Option<&'static str>,
) -> Opt {
    Opt {
        flags,
        dest,
        required,
        help,
    }
}

const AS_AT_HELP: &str = "reporting date YYYY-MM-DD (default: today)";
const MAPPING_HELP: &str = "JSON map of canonical fields to column headings";

static COMMANDS: [Command; 2] = [
    Command {
        name: "schedule",
        help: "compute the WIP schedule",
        positional: ("contracts", "contract CSV"),
        options: &[
            opt(&["-o", "--output"], "OUTPUT", false, None),
            opt(&["--as-at"], "AS_AT", false, Some(AS_AT_HELP)),
            opt(
                &["--mapping-file"],
                "MAPPING_FILE",
                false,
                Some(MAPPING_HELP),
            ),
        ],
    },
    Command {
        name: "review-pack",
        help: "write a practitioner sign-off pack",
        positional: ("schedule", "schedule CSV produced by `wip-tally schedule`"),
        options: &[
            opt(
                &["--source"],
                "SOURCE",
                true,
                Some("the contract CSV that produced it"),
            ),
            opt(&["-o", "--output"], "OUTPUT", false, None),
            opt(&["--as-at"], "AS_AT", false, Some(AS_AT_HELP)),
            opt(
                &["--mapping-file"],
                "MAPPING_FILE",
                false,
                Some(MAPPING_HELP),
            ),
        ],
    },
];

/// The flags of a command in the order argparse registers them.
fn flags(command: &Command) -> Vec<&'static str> {
    let mut flags = vec!["-h", "--help"];
    for option in command.options {
        flags.extend(option.flags.iter().copied());
    }
    flags
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

fn main_help(prog: &str, t: &Theme) -> String {
    let mut positional: Vec<Row> = vec![(2, coloured(&choices(), t.action, t), None)];
    positional.extend(
        COMMANDS
            .iter()
            .map(|c| (4, coloured(c.name, t.action, t), Some(c.help))),
    );
    let options: Vec<Row> = vec![
        (2, help_header(t), Some(HELP_HELP)),
        (2, coloured_flag("--version", t), Some(VERSION_HELP)),
    ];
    let all: Vec<Row> = positional.iter().chain(&options).cloned().collect();
    let description = wrap(DESCRIPTION, width().max(11)).join("\n");
    format!(
        "{}\n\n{description}\n\n{}\n{}\n{}\n{}",
        main_usage(prog, t),
        heading("positional arguments", t),
        action_rows(&positional, &all),
        heading("options", t),
        action_rows(&options, &all)
    )
}

fn coloured_flag(flag: &str, t: &Theme) -> Part {
    let colour = if is_long(flag) {
        t.long_option
    } else {
        t.short_option
    };
    Part {
        plain: flag.to_string(),
        coloured: format!("{colour}{flag}{}", t.reset),
    }
}

fn command_usage(prog: &str, command: &Command, t: &Theme) -> String {
    let mut parts = vec![Part {
        plain: "[-h]".into(),
        coloured: format!("[{}-h{}]", t.summary_short_option, t.reset),
    }];
    for option in command.options {
        let flag = option.flags[0];
        let colour = if is_long(flag) {
            t.summary_long_option
        } else {
            t.summary_short_option
        };
        let plain = format!("{flag} {}", option.dest);
        let coloured = format!(
            "{colour}{flag} {}{}{}",
            t.summary_label, option.dest, t.reset
        );
        parts.push(if option.required {
            Part { plain, coloured }
        } else {
            Part {
                plain: format!("[{plain}]"),
                coloured: format!("[{coloured}]"),
            }
        });
    }
    let positional = coloured(command.positional.0, t.summary_action, t);
    usage(
        &format!("{prog} {}", command.name),
        &parts,
        &[positional],
        t,
    )
}

fn command_help(prog: &str, command: &Command, t: &Theme) -> String {
    let positional: Vec<Row> = vec![(
        2,
        coloured(command.positional.0, t.action, t),
        Some(command.positional.1),
    )];
    let mut options: Vec<Row> = vec![(2, help_header(t), Some(HELP_HELP))];
    for option in command.options {
        let names: Vec<Part> = option.flags.iter().map(|f| coloured_flag(f, t)).collect();
        let plain = format!(
            "{} {}",
            names
                .iter()
                .map(|p| p.plain.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            option.dest
        );
        let coloured = format!(
            "{} {}{}{}",
            names
                .iter()
                .map(|p| p.coloured.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            t.label,
            option.dest,
            t.reset
        );
        options.push((2, Part { plain, coloured }, option.help));
    }
    let all: Vec<Row> = positional.iter().chain(&options).cloned().collect();
    format!(
        "{}\n\n{}\n{}\n{}\n{}",
        command_usage(prog, command, t),
        heading("positional arguments", t),
        action_rows(&positional, &all),
        heading("options", t),
        action_rows(&options, &all)
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
fn invalid_choice(token: &str) -> String {
    let names: Vec<&'static str> = COMMANDS.iter().map(|c| c.name).collect();
    let quoted: Vec<String> = names.iter().map(|n| pytext::repr(n)).collect();
    let suggestion = close_match(token, &names)
        .map(|m| format!(", maybe you meant {}?", pytext::repr(m)))
        .unwrap_or_default();
    format!(
        "argument command: invalid choice: {}{suggestion} (choose from {})",
        pytext::repr_os(token),
        quoted.join(", ")
    )
}

/// A subcommand's parsed arguments: option values by destination, and the positional.
struct Parsed {
    command: &'static Command,
    values: BTreeMap<&'static str, String>,
    positional: String,
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
            // PARSER drops the marker and takes the rest.
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
        return Err(main_exit(prog, invalid_choice(&args[at])));
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
    /// A known option, with any explicit argument.
    Option(usize),
    Unknown,
    Arg,
    /// The first `--`.
    Marker,
}

/// A subparser's `_parse_known_args` with one positional and value options.
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
    let is_option = |p: Pattern| matches!(p, Pattern::Option(_) | Pattern::Unknown);
    let last_option = patterns.iter().rposition(|p| is_option(*p));
    let mut parsed = Parsed {
        command,
        values: BTreeMap::new(),
        positional: String::new(),
    };
    let mut have_positional = false;
    // consume_positionals: `-*A-*` from `start`, once.
    let mut consume_positional = |start: usize, parsed: &mut Parsed| -> usize {
        if have_positional {
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
        parsed.positional = value;
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
                    .find(|o| o.flags.contains(&flag))
                    .expect("known flag");
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
                            option.flags.join("/")
                        )));
                    }
                };
                parsed.values.insert(option.dest, value);
            }
            Pattern::Arg | Pattern::Marker => unreachable!("an option starts here"),
        }
    }
    let stop = consume_positional(start, &mut parsed);
    extras.extend(args[stop..].iter().cloned());

    let mut missing: Vec<&str> = Vec::new();
    if !have_positional {
        missing.push(command.positional.0);
    }
    for option in command.options {
        if option.required && !parsed.values.contains_key(option.dest) {
            missing.push(option.flags.join("/").leak());
        }
    }
    if !missing.is_empty() {
        return Err(sub_error(format!(
            "the following arguments are required: {}",
            missing.join(", ")
        )));
    }
    Ok(parsed)
}

fn path_suffix(path: &PyPath) -> String {
    let name = path.name();
    match name.rfind('.') {
        Some(i) if !name[..i].trim_start_matches('.').is_empty() => name[i..].to_string(),
        _ => String::new(),
    }
}

fn require_suffix(path: &PyPath, suffix: &str) -> Result<(), Failure> {
    if py::casefold(&path_suffix(path)) != suffix {
        return Err(Failure::Message(format!(
            "{path} must have a {suffix} filename"
        )));
    }
    Ok(())
}

fn refuse_to_write_over_an_input(out: &PyPath, inputs: &[Option<&PyPath>]) -> Result<(), Failure> {
    let target = py::resolve_text(&out.fs());
    for candidate in inputs.iter().flatten() {
        if py::same_resolved(&target, &py::resolve_text(&candidate.fs())) {
            return Err(Failure::Message(format!(
                "refusing to write {out} because this run reads that same file"
            )));
        }
    }
    Ok(())
}

/// `_as_at`: the reporting date in YYYY-MM-DD form; today when not given.
fn as_at(raw: Option<&str>) -> Result<String, Failure> {
    match raw {
        None => Ok(py::Date::today().isoformat()),
        Some(raw) => py::Date::fromisoformat(raw)
            .map(|d| d.isoformat())
            .ok_or_else(|| {
                Failure::Message(format!(
                    "--as-at {} is not a date in YYYY-MM-DD form",
                    pytext::repr(raw)
                ))
            }),
    }
}

/// `_schedule_as_at`: the date the schedule's first row records, or None.
pub fn schedule_as_at(path: &PyPath) -> Option<String> {
    let bytes = std::fs::read(path.fs()).ok()?;
    let mut reader = crate::csvread::Reader::new(&bytes);
    let header = reader.next_row().ok()??;
    let row = loop {
        let row = reader.next_row().ok()??;
        if !row.is_empty() {
            break row;
        }
    };
    // dict(zip(fieldnames, row)): a repeated heading keeps its last column.
    let position = header.iter().rposition(|h| h == "as_at")?;
    let value = pytext::strip(row.get(position)?);
    if value.is_empty() {
        return None;
    }
    py::Date::fromisoformat(value).map(|d| d.isoformat())
}

fn read_source(path: &PyPath) -> Result<Vec<u8>, Failure> {
    std::fs::read(path.fs()).map_err(|e| {
        Failure::Message(format!(
            "cannot read {path}: {}",
            py::oserror_str(&e, &path.display(), py::OsStyle::Errno)
        ))
    })
}

fn optional_path(parsed: &Parsed, dest: &str) -> Option<PyPath> {
    parsed
        .values
        .get(dest)
        .filter(|v| !v.is_empty())
        .map(|v| PyPath::new(v))
}

fn output(parsed: &Parsed, default: &str) -> PyPath {
    PyPath::new(parsed.values.get("OUTPUT").map_or(default, String::as_str))
}

fn measure_all(
    contracts: &[engine::ContractInput],
) -> Result<Vec<engine::ContractPosition>, Failure> {
    contracts
        .iter()
        .map(|c| engine::measure(c).map_err(Failure::from))
        .collect()
}

fn cmd_schedule(parsed: &Parsed, out_text: &mut String) -> Result<i32, Failure> {
    let source = PyPath::new(&parsed.positional);
    let out = output(parsed, "wip-schedule.csv");
    require_suffix(&out, ".csv")?;
    let mapping_path = optional_path(parsed, "MAPPING_FILE");
    refuse_to_write_over_an_input(&out, &[Some(&source), mapping_path.as_ref()])?;
    let mapping = csv_io::load_mapping(mapping_path.as_ref())?;
    let bytes = read_source(&source)?;
    let contracts = csv_io::read_contracts(&source.display(), &bytes, &mapping)?;
    let positions = measure_all(&contracts)?;
    let as_at = as_at(parsed.values.get("AS_AT").map(String::as_str))?;
    let schedule = Schedule { as_at, positions };
    report::atomic_write(&out, &report::schedule_csv(&schedule))?;
    out_text.push_str(&report::render_console(&schedule)?);
    out_text.push_str(&format!("Wrote {out}\n"));
    Ok(if schedule.review_rows().is_empty() {
        EXIT_OK
    } else {
        EXIT_REVIEW
    })
}

fn cmd_review_pack(parsed: &Parsed, out_text: &mut String) -> Result<i32, Failure> {
    let schedule_path = PyPath::new(&parsed.positional);
    let out = output(parsed, "practitioner-review.md");
    require_suffix(&out, ".md")?;
    let source = optional_path(parsed, "SOURCE");
    let mapping_path = optional_path(parsed, "MAPPING_FILE");
    refuse_to_write_over_an_input(
        &out,
        &[Some(&schedule_path), source.as_ref(), mapping_path.as_ref()],
    )?;
    // Rebuilt from the source CSV so the pack cannot drift from a hand-edited schedule.
    let Some(source) = source else {
        return Err(Failure::Message(
            "review-pack needs --source (the contract CSV) so the pack can bind \
             itself to the facts, not only to a downstream schedule file"
                .into(),
        ));
    };
    let mapping = csv_io::load_mapping(mapping_path.as_ref())?;
    let source_bytes = report::read_bytes(&source)?;
    let contracts = csv_io::read_contracts(&source.display(), &source_bytes, &mapping)?;
    let positions = measure_all(&contracts)?;
    if !schedule_path.fs().exists() {
        return Err(Failure::Message(format!(
            "{schedule_path} does not exist. Run `wip-tally schedule` first so the \
             pack can hash the schedule bytes that were actually reviewed."
        )));
    }
    let as_at = match parsed.values.get("AS_AT") {
        Some(raw) => as_at(Some(raw))?,
        None => match schedule_as_at(&schedule_path) {
            Some(date) => date,
            None => as_at(None)?,
        },
    };
    let schedule = Schedule { as_at, positions };
    let schedule_bytes = report::read_bytes(&schedule_path)?;
    let text = report::build_review_pack(
        &schedule_path,
        &schedule_bytes,
        source.name(),
        &source_bytes,
        &schedule,
    )?;
    report::atomic_write(&out, &text)?;
    out_text.push_str(&format!("Wrote {out}\n"));
    Ok(if schedule.review_rows().is_empty() {
        EXIT_OK
    } else {
        EXIT_REVIEW
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
    let result = if parsed.command.name == "schedule" {
        cmd_schedule(&parsed, &mut out)
    } else {
        cmd_review_pack(&parsed, &mut out)
    };
    match result {
        Ok(code) => (code, out, String::new()),
        Err(Failure::Message(message)) => (EXIT_ERROR, out, format!("error: {message}\n")),
        // Python raised the decimal signal uncaught: a traceback and exit 1.
        Err(Failure::Decimal(signal)) => (
            EXIT_ERROR,
            out,
            format!("error: decimal.{}\n", signal.name()),
        ),
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
