//! argparse compatibility for the two original commands. Formatting and parsing
//! are adapted from the already qualified elizabeth-anne-alexander Rust port.
const NOT_ADVICE: &str = "Not advice. Review aid only; confirm against current law before acting.";
use crate::argtheme::{self, Stream, Theme};
use crate::date::Date;
use crate::decimal::Decimal;
use crate::engine::{self, Bre, Statement};
use crate::pytext;
use crate::textwrap::wrap;
use std::collections::BTreeMap;
struct Opt {
    flag: &'static str,
    dest: &'static str,
    required: bool,
    help: Option<&'static str>,
}

struct Command {
    name: &'static str,
    help: &'static str,
    options: &'static [Opt],
}

const fn opt(
    flag: &'static str,
    dest: &'static str,
    required: bool,
    help: Option<&'static str>,
) -> Opt {
    Opt {
        flag,
        dest,
        required,
        help,
    }
}

static COMMANDS: [Command; 2] = [
    Command {
        name: "bre-test",
        help: "Test Base Rate Entity (BRE) eligibility under s 23AA ITRA 1986",
        options: &[
            opt(
                "--fy",
                "FY",
                true,
                Some("Financial Year ending (e.g. 2025)"),
            ),
            opt(
                "--turnover",
                "TURNOVER",
                true,
                Some("Aggregated turnover ($)"),
            ),
            opt(
                "--assessable",
                "ASSESSABLE",
                true,
                Some("Total assessable income ($)"),
            ),
            opt(
                "--passive",
                "PASSIVE",
                true,
                Some("Base Rate Entity Passive Income ($)"),
            ),
        ],
    },
    Command {
        name: "dist-statement",
        help: "Generate distribution statement details",
        options: &[
            opt("--entity", "ENTITY", true, Some("Company name")),
            opt("--acn", "ACN", true, Some("ACN or ABN")),
            opt("--recipient", "RECIPIENT", true, Some("Shareholder name")),
            opt(
                "--payment-date",
                "PAYMENT_DATE",
                true,
                Some("Actual payment date YYYY-MM-DD"),
            ),
            opt(
                "--amount",
                "AMOUNT",
                true,
                Some("Total dividend distribution ($)"),
            ),
            opt(
                "--franking-pct",
                "FRANKING_PCT",
                false,
                Some("Franking percentage (e.g. 100)"),
            ),
            opt(
                "--tax-rate",
                "TAX_RATE",
                true,
                Some(
                    "Corporate tax rate for the entity, e.g. 0.25 or 0.30. Required: it is not assumed",
                ),
            ),
        ],
    },
];
const DESCRIPTION: &str = "The Exchequer Tally: corporate tax rate and franking account engine for Australian companies. Outputs are review aids, not tax advice.";
const HELP_HELP: &str = "show this help message and exit";

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

fn command_usage(prog: &str, command: &Command, t: &Theme) -> String {
    let mut parts = vec![Part {
        plain: "[-h]".into(),
        coloured: format!("[{}-h{}]", t.summary_short_option, t.reset),
    }];
    for option in command.options {
        let plain = format!("{} {}", option.flag, option.dest);
        let colour = format!(
            "{}{} {}{}{}",
            t.summary_long_option, option.flag, t.summary_label, option.dest, t.reset
        );
        parts.push(if option.required {
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
    usage(&format!("{prog} {}", command.name), &parts, &[], t)
}

fn command_help(prog: &str, command: &Command, t: &Theme) -> String {
    let mut rows = vec![(2, help_header(t), Some(HELP_HELP))];
    rows.extend(command.options.iter().map(|o| {
        let plain = format!("{} {}", o.flag, o.dest);
        let colour = format!(
            "{}{}{} {}{}{}",
            t.long_option, o.flag, t.reset, t.label, o.dest, t.reset
        );
        (
            2,
            Part {
                plain,
                coloured: colour,
            },
            o.help,
        )
    }));
    format!(
        "{}\n\n{}options:{}\n{}",
        command_usage(prog, command, t),
        t.heading,
        t.reset,
        action_rows(&rows, &rows)
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

/// argparse's `_negative_number_matcher` (`-\.?\d`, matched as a prefix) on the text after the
/// leading '-': "-1x", "-.5x" and "-1\n" all count, as in Python 3.15.
fn looks_like_negative_number(body: &str) -> bool {
    let digit = |c: char| pytext::decimal_value(c as u32).is_some();
    let mut chars = body.chars();
    match chars.next() {
        Some('.') => chars.next().is_some_and(digit),
        Some(c) => digit(c),
        None => false,
    }
}

/// argparse's handling of an explicit argument given to the zero-argument help flag.
fn help_or_error(flag: &str, explicit: Option<(String, bool)>) -> Result<(), String> {
    match explicit {
        None => Ok(()),
        // "-hx": the rest is read as further short flags, and help runs first.
        Some((rest, false)) if flag == "-h" && !rest.is_empty() && !rest.starts_with('-') => Ok(()),
        Some((rest, _)) => Err(format!(
            "argument -h/--help: ignored explicit argument {}",
            pytext::repr_os(&rest)
        )),
    }
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

fn parse(
    prog: &str,
    args: &[String],
) -> Result<(&'static Command, BTreeMap<&'static str, String>), Exit> {
    let mut extras = Vec::new();
    let (command, index) = parse_command(prog, args, &mut extras)?;
    let values = parse_options(prog, command, &args[index..], &mut extras)?;
    let missing: Vec<_> = command
        .options
        .iter()
        .filter(|o| o.required && !values.contains_key(o.flag))
        .map(|o| o.flag)
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
    Ok((command, values))
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
                    help_or_error(flag, explicit).map_err(|m| main_exit(prog, m))?;
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

/// The subcommand's options after its name.
fn parse_options(
    prog: &str,
    command: &Command,
    args: &[String],
    extras: &mut Vec<String>,
) -> Result<BTreeMap<&'static str, String>, Exit> {
    let sub_error = |message: String| sub_exit(prog, command, message);
    let mut flags: Vec<&'static str> = vec!["-h", "--help"];
    flags.extend(command.options.iter().map(|o| o.flag));
    let mut values: BTreeMap<&'static str, String> = BTreeMap::new();
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
            help_or_error(flag, explicit).map_err(sub_error)?;
            return Err(Exit::Help(
                command_help(prog, command, argtheme::theme(Stream::Stdout)),
                0,
            ));
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
        validate_option(flag, &value).map_err(|m| sub_error(format!("argument {flag}: {m}")))?;
        values.insert(flag, value);
    }
    Ok(values)
}

fn finite(value: &str) -> Result<Decimal, String> {
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
pub fn money(value: &str) -> Result<Decimal, String> {
    let p = finite(value)?;
    if p < engine::d("0") || p > engine::d("1000000000000.00") {
        return Err(format!(
            "money must be between 0 and 1000000000000.00 AUD: {}",
            pytext::repr(value)
        ));
    }
    if p.exponent() < -2 {
        return Err(format!(
            "money cannot have more than 2 decimal places: {}",
            pytext::repr(value)
        ));
    }
    Ok(p)
}
pub fn percentage(value: &str) -> Result<Decimal, String> {
    let p = finite(value)?;
    if p < engine::d("0") || p > engine::d("100") {
        return Err(format!(
            "percentage must be between 0 and 100: {}",
            pytext::repr(value)
        ));
    }
    Ok(p)
}
pub fn tax_rate(value: &str) -> Result<Decimal, String> {
    let p = finite(value)?;
    if p <= engine::d("0") || p >= engine::d("1") {
        return Err(format!(
            "rate must be between 0 and 1: {}",
            pytext::repr(value)
        ));
    }
    Ok(p)
}
fn validate_option(flag: &str, value: &str) -> Result<(), String> {
    match flag {
        "--fy" => {
            integer(value)?;
        }
        "--turnover" | "--assessable" | "--passive" | "--amount" => {
            money(value)?;
        }
        "--franking-pct" => {
            percentage(value)?;
        }
        "--tax-rate" => {
            tax_rate(value)?;
        }
        "--payment-date" => {
            Date::parse(value)?;
        }
        _ => {}
    }
    Ok(())
}
fn integer(value: &str) -> Result<String, String> {
    let invalid = || format!("invalid int value: {}", pytext::repr(value));
    crate::termsize::py_int(value).ok_or_else(invalid)?;
    let mut ascii = String::new();
    for ch in pytext::strip(value).chars() {
        if ch == '+' || ch == '-' {
            ascii.push(ch);
        } else if ch != '_' {
            ascii.push(
                char::from_digit(pytext::decimal_value(ch as u32).ok_or_else(invalid)?, 10)
                    .unwrap(),
            );
        }
    }
    if ascii.trim_start_matches(['+', '-']).len() > 4300 {
        return Err(invalid());
    }
    let n = ascii.parse::<num_bigint::BigInt>().map_err(|_| invalid())?;
    Ok(n.to_string())
}
fn dispatch(command: &Command, values: &BTreeMap<&str, String>) -> Result<String, String> {
    let get = |flag| {
        values
            .get(flag)
            .expect("required argument validated")
            .as_str()
    };
    let dec = |flag| Decimal::parse(get(flag)).expect("validated decimal");
    let border = "=".repeat(60);
    if command.name == "bre-test" {
        let fy_text = integer(get("--fy"))?;
        let fy = crate::termsize::py_int(get("--fy")).expect("validated integer");
        let test = Bre {
            fy,
            turnover: dec("--turnover"),
            assessable: dec("--assessable"),
            passive: dec("--passive"),
        };
        let res = test
            .corporate_tax()
            .map_err(|m| m.replace(&format!("FY{fy}"), &format!("FY{fy_text}")))?;
        let passive = match test.passive_pct()? {
            None => "n/a (no assessable income)".into(),
            Some(p) => format!("{}%", p.fixed(2, false)),
        };
        let boolean = |v| if v { "True" } else { "False" };
        let brepi = match test.brepi() {
            None => "None",
            Some(v) => boolean(v),
        };
        Ok(format!(
            "{border}\nBase Rate Entity (BRE) Evaluation - FY{fy_text}\n{border}\nAggregated Turnover:     ${} (< ${}M: {})\nPassive Income Ratio:    {passive} (<= 80%: {brepi})\nBase Rate Entity:        {}\nApplicable Tax Rate:     {}%\nStatutory Basis:         {}\n{NOT_ADVICE}\n{border}\n",
            test.turnover.fixed(2, true),
            engine::threshold(fy)?
                .div(&engine::d("1000000"))
                .fixed(0, false),
            boolean(test.turnover_eligible()?),
            boolean(res.bre),
            res.rate.mul(&engine::d("100")).fixed(1, false),
            res.basis
        ))
    } else {
        let pct = values
            .get("--franking-pct")
            .map(|s| Decimal::parse(s).unwrap())
            .unwrap_or_else(|| engine::d("100.00"));
        let s = Statement::new(
            get("--entity").into(),
            get("--acn").into(),
            get("--recipient").into(),
            Date::parse(get("--payment-date"))?,
            dec("--amount"),
            pct,
            Some(dec("--tax-rate")),
        )?;
        Ok(format!(
            "{border}\nAustralian Dividend Distribution Statement - {}\n{border}\nACN/ABN:                 {}\nRecipient:               {}\nPayment Date:            {}\nFranked Dividend Amount: ${}\nUnfranked Amount:        ${}\nFranking Credit:         ${}\nFranking Percentage:     {}%\nGross Assessable:        ${}\n{NOT_ADVICE}\n{border}\n",
            s.entity,
            s.acn,
            s.recipient,
            s.date,
            s.franked.fixed(2, true),
            s.unfranked.fixed(2, true),
            s.credit.fixed(2, true),
            s.pct.fixed(2, false),
            s.gross().fixed(2, true)
        ))
    }
}
/// (exit code, stdout, stderr), before platform text-mode newline conversion.
pub fn run(args: &[String]) -> (i32, String, String) {
    const PROG: &str = "the-exchequer-tally";
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
                format!(
                    // Python 3.15 leaves the program name plain and colours "error:" with its colon.
                    "{usage}\n{prog}: {}error:{} {}{message}{}\n",
                    t.error, t.reset, t.message, t.reset
                ),
            )
        }
        Ok((command, values)) => match dispatch(command, &values) {
            Ok(text) => (0, text, String::new()),
            Err(m) => (
                if m.starts_with("decimal.") { 1 } else { 2 },
                String::new(),
                format!("error: {m}\n"),
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
pub fn output_bytes(text: &str, stderr: bool) -> Vec<u8> {
    let mut out = Vec::new();
    for cp in pytext::code_points(text) {
        match char::from_u32(cp) {
            None if (0xDC80..=0xDCFF).contains(&cp) && !stderr => out.push((cp - 0xDC00) as u8),
            None => out.extend_from_slice(format!("\\u{cp:04x}").as_bytes()),
            Some('\n') if cfg!(windows) => out.extend_from_slice(b"\r\n"),
            Some(ch) => out.extend_from_slice(ch.encode_utf8(&mut [0; 4]).as_bytes()),
        }
    }
    out
}
