//! argparse compatibility for the two original commands. Formatting and parsing
//! are adapted from The Exchequer Tally's port of the qualified
//! elizabeth-anne-alexander argparse helpers, extended with the tristate flag
//! pairs this command line uses (mutually exclusive `store_const` groups).
use crate::argtheme::{self, Stream, Theme};
use crate::decimal::Decimal;
use crate::engine::{self, Error, Fact, ForeignTrustReceipt, Section100AFacts};
use crate::pytext;
use crate::textwrap::wrap;
use std::collections::BTreeMap;

const NOT_ADVICE: &str =
    "Not advice. Review aid only; confirm against current law and the trust deed before acting.";
// Output carries the beneficiary name the operator supplied, so stdout is client data.
const DATA_BOUNDARY: &str =
    "Output contains client data. Write it only to the firm's approved secure location.";
const PROG: &str = "solomons-sword";
const DESCRIPTION: &str = "Solomon's Sword: Division 6 trust allocation with s 100A and s 99B checks. Review aid, not advice.";
const HELP_HELP: &str = "show this help message and exit";

/// One argument as added to a subparser: a value option, or a tristate fact's
/// mutually exclusive pair of `store_const` flags.
enum Arg {
    Value {
        flag: &'static str,
        metavar: &'static str,
        required: bool,
        help: &'static str,
    },
    Pair {
        dest: &'static str,
        yes: &'static str,
        no: &'static str,
        fact: &'static str,
    },
}

struct Command {
    name: &'static str,
    help: &'static str,
    args: &'static [Arg],
}

const fn value(
    flag: &'static str,
    metavar: &'static str,
    required: bool,
    help: &'static str,
) -> Arg {
    Arg::Value {
        flag,
        metavar,
        required,
        help,
    }
}

const fn pair(dest: &'static str, yes: &'static str, no: &'static str, fact: &'static str) -> Arg {
    Arg::Pair {
        dest,
        yes,
        no,
        fact,
    }
}

static COMMANDS: [Command; 2] = [
    Command {
        name: "s100a-check",
        help: "Evaluate Section 100A reimbursement agreement risk",
        args: &[
            value("--beneficiary", "BENEFICIARY", true, "Beneficiary name"),
            value("--amount", "AMOUNT", true, "Distribution amount ($)"),
            pair(
                "adult_child",
                "--adult-child",
                "--no-adult-child",
                "Beneficiary is an adult child",
            ),
            pair(
                "retained_by_parents",
                "--retained-by-parents",
                "--no-retained-by-parents",
                "Funds retained by parents without loan",
            ),
            pair(
                "pre_18_expenses",
                "--pre-18-expenses",
                "--no-pre-18-expenses",
                "Entitlement paid to a parent or caregiver, or set against a debit account, for expenses incurred before the beneficiary turned 18 (PCG 2022/2 para 34)",
            ),
            pair(
                "circular",
                "--circular",
                "--no-circular",
                "Trust income returned to the trust as a franked dividend (PCG 2022/2 para 36)",
            ),
            pair(
                "corporate_upe",
                "--corporate-upe",
                "--no-corporate-upe",
                "Corporate beneficiary holds an unpaid present entitlement",
            ),
            pair(
                "received_funds",
                "--received-funds",
                "--funds-not-received",
                "Beneficiary received and retained the funds",
            ),
            pair(
                "direct_benefit",
                "--direct-benefit",
                "--no-direct-benefit",
                "Funds applied directly for the beneficiary's benefit",
            ),
            pair(
                "commercial_loan",
                "--commercial-loan",
                "--no-commercial-loan",
                "Entitlement retained under a loan on PCG 2022/2 para 25(e) commercial terms",
            ),
            pair(
                "within_two_years",
                "--within-two-years",
                "--not-within-two-years",
                "Entitlement received within 2 years of present entitlement (para 22)",
            ),
            pair(
                "retention_conditions",
                "--retention-conditions",
                "--no-retention-conditions",
                "Rest of green zone scenario 3A (para 26) or 3B (para 28) met",
            ),
            pair(
                "para_32_exclusion",
                "--para-32-exclusion",
                "--no-para-32-exclusion",
                "A PCG 2022/2 paragraph 32 green zone exclusion applies",
            ),
        ],
    },
    Command {
        name: "s99b-check",
        help: "Evaluate Section 99B foreign trust distribution",
        args: &[
            value("--beneficiary", "BENEFICIARY", true, "Beneficiary name"),
            value("--gross", "GROSS", true, "Gross amount received AUD ($)"),
            value(
                "--corpus",
                "CORPUS",
                false,
                "Corpus / capital settlement ($)",
            ),
            value(
                "--corpus-attributable",
                "CORPUS_ATTRIBUTABLE",
                false,
                "Part of the corpus attributable to amounts that would have been assessable to a resident (s 99B(2)(a) proviso)",
            ),
            value(
                "--not-assessable-to-resident",
                "NOT_ASSESSABLE_TO_RESIDENT",
                false,
                "Amounts that would not have been assessable to a resident (s 99B(2)(b))",
            ),
            value(
                "--prior-taxed",
                "PRIOR_TAXED",
                false,
                "Amounts already assessed under s 97/98/99/99A (s 99B(2)(c))",
            ),
            pair(
                "resident_during_year",
                "--resident-during-year",
                "--not-resident-during-year",
                "Beneficiary was a resident at some time during the year of income",
            ),
        ],
    },
];

/// The flags of a command in the order argparse registers them.
fn flags(command: &Command) -> Vec<&'static str> {
    let mut flags = vec!["-h", "--help"];
    for arg in command.args {
        match arg {
            Arg::Value { flag, .. } => flags.push(flag),
            Arg::Pair { yes, no, .. } => flags.extend([*yes, *no]),
        }
    }
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

fn main_usage(prog: &str, t: &Theme) -> String {
    let help = Part {
        plain: "[-h]".into(),
        coloured: format!("[{}-h{}]", t.summary_short_option, t.reset),
    };
    // The subparsers action is one usage part: "{choices} ...".
    usage(
        prog,
        &[help],
        &[coloured(&format!("{} ...", choices()), t.summary_action, t)],
        t,
    )
}

fn main_help(prog: &str, t: &Theme) -> String {
    let mut positional: Vec<Row> = vec![(
        2,
        coloured(&choices(), t.action, t),
        Some("Available subcommands"),
    )];
    positional.extend(
        COMMANDS
            .iter()
            .map(|c| (4, coloured(c.name, t.action, t), Some(c.help))),
    );
    let options: Vec<Row> = vec![(2, help_header(t), Some(HELP_HELP))];
    let all: Vec<Row> = positional.iter().chain(&options).cloned().collect();
    let (positional, options) = (action_rows(&positional, &all), action_rows(&options, &all));
    let description = wrap(DESCRIPTION, width().max(11)).join("\n");
    let heading = |text: &str| format!("{}{text}:{}", t.heading, t.reset);
    format!(
        "{}\n\n{description}\n\n{}\n{positional}\n{}\n{options}",
        main_usage(prog, t),
        heading("positional arguments"),
        heading("options")
    )
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
/// The main parser up to the subcommand: its help flag, unknown options kept as
/// extras, and the command choice. Returns the command and the index after it.
fn parse_command(
    prog: &str,
    args: &[String],
    extras: &mut Vec<String>,
) -> Result<(&'static Command, usize), Exit> {
    let mut index = 0;
    let mut options_ended = false;
    loop {
        let Some(token) = args.get(index) else {
            return Err(if extras.is_empty() {
                Exit::Help(main_help(prog, argtheme::theme(Stream::Stdout)), 1)
            } else {
                main_exit(
                    prog,
                    format!("unrecognized arguments: {}", extras.join(" ")),
                )
            });
        };
        index += 1;
        if !options_ended {
            if token == "--" {
                options_ended = true;
                continue;
            }
            match classify(token, &["-h", "--help"]).map_err(|m| main_exit(prog, m))? {
                Class::Flag(flag, explicit) => {
                    ignored_argument(flag, explicit).map_err(|m| main_exit(prog, m))?;
                    return Err(Exit::Help(
                        main_help(prog, argtheme::theme(Stream::Stdout)),
                        0,
                    ));
                }
                Class::UnknownOption => {
                    extras.push(token.clone());
                    continue;
                }
                Class::Positional => {}
            }
        }
        return match COMMANDS.iter().find(|c| c.name == token) {
            Some(command) => Ok((command, index)),
            None => Err(main_exit(prog, invalid_choice(token))),
        };
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
fn command_usage(prog: &str, command: &Command, t: &Theme) -> String {
    let mut parts = vec![Part {
        plain: "[-h]".into(),
        coloured: format!("[{}-h{}]", t.summary_short_option, t.reset),
    }];
    for arg in command.args {
        match arg {
            Arg::Value {
                flag,
                metavar,
                required,
                ..
            } => {
                let plain = format!("{flag} {metavar}");
                let colour = format!(
                    "{}{flag} {}{metavar}{}",
                    t.summary_long_option, t.summary_label, t.reset
                );
                parts.push(if *required {
                    Part {
                        plain,
                        coloured: colour,
                    }
                } else {
                    Part {
                        plain: format!("[{plain}]"),
                        coloured: format!("[{colour}]"),
                    }
                });
            }
            // A mutually exclusive group: "[--yes |" and "--no]" wrap separately.
            Arg::Pair { yes, no, .. } => {
                let colour = |flag: &str| format!("{}{flag}{}", t.summary_long_option, t.reset);
                parts.push(Part {
                    plain: format!("[{yes} |"),
                    coloured: format!("[{} |", colour(yes)),
                });
                parts.push(Part {
                    plain: format!("{no}]"),
                    coloured: format!("{}]", colour(no)),
                });
            }
        }
    }
    usage(&format!("{prog} {}", command.name), &parts, &[], t)
}

fn command_help(prog: &str, command: &Command, t: &Theme) -> String {
    let mut rows: Vec<(usize, Part, String)> = vec![(2, help_header(t), HELP_HELP.into())];
    let flag_row = |flag: &str, help: String| {
        (
            2,
            Part {
                plain: flag.to_string(),
                coloured: format!("{}{flag}{}", t.long_option, t.reset),
            },
            help,
        )
    };
    for arg in command.args {
        match arg {
            Arg::Value {
                flag,
                metavar,
                help,
                ..
            } => rows.push((
                2,
                Part {
                    plain: format!("{flag} {metavar}"),
                    coloured: format!(
                        "{}{flag}{} {}{metavar}{}",
                        t.long_option, t.reset, t.label, t.reset
                    ),
                },
                help.to_string(),
            )),
            Arg::Pair { yes, no, fact, .. } => {
                rows.push(flag_row(yes, format!("{fact}: yes")));
                rows.push(flag_row(no, format!("{fact}: no")));
            }
        }
    }
    let rows: Vec<Row> = rows
        .iter()
        .map(|(indent, part, help)| (*indent, part.clone(), Some(help.as_str())))
        .collect();
    format!(
        "{}\n\n{}options:{}\n{}",
        command_usage(prog, command, t),
        t.heading,
        t.reset,
        action_rows(&rows, &rows)
    )
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

/// What a subcommand's arguments parse to: option values by flag, and each
/// stated fact by its destination.
struct Parsed {
    command: &'static Command,
    values: BTreeMap<&'static str, String>,
    facts: BTreeMap<&'static str, bool>,
}

fn parse(prog: &str, args: &[String]) -> Result<Parsed, Exit> {
    let mut extras = Vec::new();
    let (command, index) = parse_command(prog, args, &mut extras)?;
    let parsed = parse_options(prog, command, &args[index..], &mut extras)?;
    let missing: Vec<&str> = command
        .args
        .iter()
        .filter_map(|arg| match arg {
            Arg::Value {
                flag,
                required: true,
                ..
            } if !parsed.values.contains_key(flag) => Some(*flag),
            _ => None,
        })
        .collect();
    if !missing.is_empty() {
        return Err(sub_exit(
            prog,
            command,
            format!(
                "the following arguments are required: {}",
                missing.join(", ")
            ),
        ));
    }
    if !extras.is_empty() {
        return Err(main_exit(
            prog,
            format!("unrecognized arguments: {}", extras.join(" ")),
        ));
    }
    Ok(parsed)
}

/// The subcommand's options after its name, consumed in order as argparse does:
/// help, type and conflict errors are raised where they occur.
fn parse_options(
    prog: &str,
    command: &'static Command,
    args: &[String],
    extras: &mut Vec<String>,
) -> Result<Parsed, Exit> {
    let sub_error = |message: String| sub_exit(prog, command, message);
    let flags = flags(command);
    let mut parsed = Parsed {
        command,
        values: BTreeMap::new(),
        facts: BTreeMap::new(),
    };
    let mut seen: Vec<&'static str> = Vec::new();
    let mut index = 0;
    while let Some(token) = args.get(index) {
        index += 1;
        if token == "--" {
            // Everything from here is positional, and this parser takes none.
            extras.extend(args[index - 1..].iter().cloned());
            break;
        }
        let (flag, explicit) = match classify(token, &flags).map_err(sub_error)? {
            Class::Flag(flag, explicit) => (flag, explicit),
            Class::UnknownOption | Class::Positional => {
                extras.push(token.clone());
                continue;
            }
        };
        if flag == "-h" || flag == "--help" {
            ignored_argument(flag, explicit).map_err(sub_error)?;
            return Err(Exit::Help(
                command_help(prog, command, argtheme::theme(Stream::Stdout)),
                0,
            ));
        }
        let pair = command.args.iter().find_map(|arg| match arg {
            Arg::Pair { dest, yes, no, .. } if flag == *yes => Some((*dest, true, *no)),
            Arg::Pair { dest, yes, no, .. } if flag == *no => Some((*dest, false, *yes)),
            _ => None,
        });
        if let Some((dest, stated, partner)) = pair {
            ignored_argument(flag, explicit).map_err(sub_error)?;
            seen.push(flag);
            if seen.contains(&partner) {
                return Err(sub_error(format!(
                    "argument {flag}: not allowed with argument {partner}"
                )));
            }
            parsed.facts.insert(dest, stated);
            continue;
        }
        let value = match explicit {
            Some((value, _)) => value,
            None => match args.get(index) {
                Some(next)
                    if next != "--" && matches!(classify(next, &flags), Ok(Class::Positional)) =>
                {
                    index += 1;
                    next.clone()
                }
                _ => return Err(sub_error(format!("argument {flag}: expected one argument"))),
            },
        };
        if flag != "--beneficiary" {
            decimal_type(&value).map_err(|m| sub_error(format!("argument {flag}: {m}")))?;
        }
        parsed.values.insert(flag, value);
    }
    Ok(parsed)
}

/// `decimal_type`: the fail-closed argparse type for Decimal money.
pub fn decimal_type(value: &str) -> Result<Decimal, String> {
    let parsed = Decimal::parse(value)
        .map_err(|_| format!("not a decimal amount: {}", pytext::repr(value)))?;
    if !parsed.is_finite() {
        return Err(format!(
            "not a finite decimal amount: {}",
            pytext::repr(value)
        ));
    }
    Ok(parsed)
}

fn fact(parsed: &Parsed, dest: &str) -> Fact {
    parsed
        .facts
        .get(dest)
        .map_or(Fact::Unknown, |stated| Fact::Known(*stated))
}

fn dispatch(parsed: &Parsed) -> Result<String, Error> {
    let text = |flag: &str| parsed.values.get(flag).map(String::as_str);
    let amount = |flag: &str, default: &str| {
        decimal_type(text(flag).unwrap_or(default)).expect("validated decimal")
    };
    let beneficiary = text("--beneficiary").expect("required argument validated");
    let border = "=".repeat(60);
    let mut out = String::new();
    let mut line = |text: &str| {
        out.push_str(text);
        out.push('\n');
    };
    if parsed.command.name == "s100a-check" {
        let facts = Section100AFacts {
            beneficiary_is_adult_child: fact(parsed, "adult_child"),
            funds_retained_by_parents_without_loan: fact(parsed, "retained_by_parents"),
            circular_flow_of_funds: fact(parsed, "circular"),
            corporate_beneficiary_unpaid_present_entitlement: fact(parsed, "corporate_upe"),
            beneficiary_actually_received_funds: fact(parsed, "received_funds"),
            funds_used_for_beneficiary_direct_benefit: fact(parsed, "direct_benefit"),
            commercial_loan_agreement_in_place: fact(parsed, "commercial_loan"),
            entitlement_applied_to_pre_18_expenses: fact(parsed, "pre_18_expenses"),
            received_within_two_years: fact(parsed, "within_two_years"),
            retention_scenario_conditions_met: fact(parsed, "retention_conditions"),
            paragraph_32_exclusion_present: fact(parsed, "para_32_exclusion"),
        };
        let res = engine::evaluate_section100a_risk(beneficiary, &amount("--amount", ""), &facts)?;
        // After the engine, so a zero or negative amount keeps its own error.
        engine::validate_printable_amount("distribution amount", &res.distribution_amount)?;
        line(&border);
        line(&format!(
            "Section 100A Risk Evaluation — {}",
            res.beneficiary_name
        ));
        line(&border);
        line(&format!(
            "Distribution Amount:     ${}",
            res.distribution_amount.fixed(2, true)
        ));
        line(&format!(
            "Risk Zone:               {}",
            res.risk_zone.value()
        ));
        let ofd = match res.is_ordinary_family_dealing {
            None => "Not determined",
            Some(true) => "True",
            Some(false) => "False",
        };
        line(&format!("Ordinary Family Dealing: {ofd}"));
        if !res.unestablished_facts.is_empty() {
            line(&format!(
                "Facts Not Established:   {}",
                res.unestablished_facts.join(", ")
            ));
        }
        if !res.risk_factors_identified.is_empty() {
            line(&format!(
                "Risk Factors:            {}",
                res.risk_factors_identified.join(", ")
            ));
        }
        line(&format!(
            "Consequence:             {}",
            res.tax_consequence_summary
        ));
    } else {
        let receipt = ForeignTrustReceipt {
            beneficiary_name: beneficiary.to_string(),
            gross_amount_received_aud: amount("--gross", ""),
            beneficiary_was_resident_during_year: fact(parsed, "resident_during_year"),
            corpus_amount_aud: amount("--corpus", "0.00"),
            corpus_attributable_to_notional_assessable_income_aud: amount(
                "--corpus-attributable",
                "0.00",
            ),
            not_assessable_to_resident_aud: amount("--not-assessable-to-resident", "0.00"),
            already_assessed_under_div6_aud: amount("--prior-taxed", "0.00"),
        };
        let s99b = engine::evaluate_section99b_liability(&receipt)?;
        line(&border);
        line(&format!(
            "Section 99B Assessment — {}",
            s99b.beneficiary_name
        ));
        line(&border);
        line(&format!(
            "Gross Receipt:           ${}",
            s99b.gross_receipt.fixed(2, true)
        ));
        line(&format!(
            "Corpus Exemption:        ${}",
            s99b.corpus_exemption.fixed(2, true)
        ));
        line(&format!(
            "Assessable under s99B:   ${}",
            s99b.assessable_income_under_s99b.fixed(2, true)
        ));
        line(&format!(
            "Basis:                   {}",
            s99b.statutory_basis
        ));
        for caveat in &s99b.caveats {
            line(&format!("Caveat:                  {caveat}"));
        }
    }
    line(NOT_ADVICE);
    line(DATA_BOUNDARY);
    line(&border);
    Ok(out)
}

/// (exit code, stdout, stderr), before platform text-mode newline conversion.
pub fn run(args: &[String]) -> (i32, String, String) {
    match parse(PROG, args) {
        Err(Exit::Help(text, code)) => (code, text, String::new()),
        Err(Exit::Error {
            usage,
            prog,
            message,
        }) => {
            let t = argtheme::theme(Stream::Stderr);
            (
                2,
                String::new(),
                // Python 3.15 colours "error:" and the message, not the program name.
                format!(
                    "{usage}\n{prog}: {}error:{} {}{message}{}\n",
                    t.error, t.reset, t.message, t.reset
                ),
            )
        }
        Ok(parsed) => match dispatch(&parsed) {
            Ok(text) => (0, text, String::new()),
            Err(Error::Value(message)) => (2, String::new(), format!("error: {message}\n")),
            // Individually accepted finite amounts can still overflow when combined.
            Err(Error::Decimal(_)) => (
                2,
                String::new(),
                "error: the supplied amounts produce a value outside the decimal arithmetic \
                 range this command supports\n"
                    .into(),
            ),
        },
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
/// surrogate from the command line is written as its `\udcXX` escape.
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
