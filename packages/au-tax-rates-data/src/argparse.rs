//! The subset of Python 3.15's argparse that `rates` needs: a raw description, required
//! subcommands with close-match suggestions, store options, abbreviations, help and usage text
//! with argparse's colour theme and terminal width, and its error messages and exit codes.
//! Formatting follows `HelpFormatter`; parsing follows `ArgumentParser._parse_known_args`.
use crate::argtheme::{self, Stream, Theme};
use crate::pytext;
use crate::textwrap::wrap;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    None,
    Str(String),
    List(Vec<String>),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Store,
    Help,
    Parsers,
}

/// Options take one argument (`Store`) or none (`Help`); the subcommand positional takes the
/// rest (`nargs=PARSER`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Nargs {
    One,
    Zero,
    Parser,
}

#[derive(Clone, Debug)]
pub struct Action {
    pub option_strings: Vec<String>,
    pub dest: String,
    pub kind: Kind,
    pub nargs: Nargs,
    pub default: Value,
    pub choices: Option<Vec<String>>,
    pub help: Option<String>,
}

pub struct Parser {
    pub prog: String,
    /// `description=text, formatter_class=RawDescriptionHelpFormatter`: printed as written.
    pub description: Option<String>,
    pub actions: Vec<Action>,
    /// (name, help, parser) for the subparsers action, in add order.
    pub subparsers: Vec<(String, Option<String>, Parser)>,
}

/// Builder-style helpers mirroring `add_argument`.
impl Action {
    pub fn option(flag: &str) -> Self {
        let dest = flag.trim_start_matches('-').replace('-', "_");
        Self {
            option_strings: vec![flag.into()],
            dest,
            kind: Kind::Store,
            nargs: Nargs::One,
            default: Value::None,
            choices: None,
            help: None,
        }
    }
    pub fn default(mut self, value: Value) -> Self {
        self.default = value;
        self
    }
    pub fn help(mut self, text: &str) -> Self {
        self.help = Some(text.into());
        self
    }
    fn help_action() -> Self {
        Self {
            option_strings: vec!["-h".into(), "--help".into()],
            dest: "help".into(),
            kind: Kind::Help,
            nargs: Nargs::Zero,
            default: Value::None,
            choices: None,
            help: Some("show this help message and exit".into()),
        }
    }
}

impl Parser {
    pub fn new(prog: &str) -> Self {
        Self {
            prog: prog.into(),
            description: None,
            actions: vec![Action::help_action()],
            subparsers: vec![],
        }
    }
    /// `description=text, formatter_class=RawDescriptionHelpFormatter`.
    pub fn raw_description(mut self, text: &str) -> Self {
        self.description = Some(text.into());
        self
    }
    pub fn arg(mut self, action: Action) -> Self {
        self.actions.push(action);
        self
    }
    /// `add_subparsers(dest=dest, required=True)`; sub-parsers are added with `sub`.
    pub fn subcommands(mut self, dest: &str) -> Self {
        self.actions.push(Action {
            option_strings: vec![],
            dest: dest.into(),
            kind: Kind::Parsers,
            nargs: Nargs::Parser,
            default: Value::None,
            choices: Some(vec![]),
            help: None,
        });
        self
    }
    /// `add_parser(name, help=...)`; the prog is "<this prog> <name>".
    pub fn sub(
        mut self,
        name: &str,
        help: Option<&str>,
        build: impl FnOnce(Parser) -> Parser,
    ) -> Self {
        let parser = build(Parser::new(&format!("{} {name}", self.prog)));
        if let Some(action) = self.actions.iter_mut().find(|a| a.kind == Kind::Parsers)
            && let Some(choices) = &mut action.choices
        {
            choices.push(name.into());
        }
        self.subparsers
            .push((name.into(), help.map(str::to_string), parser));
        self
    }
}

pub type Namespace = BTreeMap<String, Value>;

/// What argparse does instead of returning a namespace.
#[derive(Debug)]
pub enum Exit {
    /// Help printed to stdout, exit 0.
    Help(String),
    /// Usage and error printed to stderr, exit 2.
    Error(String),
}

// ---------------------------------------------------------------- formatting

/// argparse's formatter width: `shutil.get_terminal_size().columns` minus 2.
fn width() -> usize {
    usize::try_from(crate::termsize::columns() - 2).unwrap_or(0)
}

/// A usage part or help header: its plain text, which sets every width, and the coloured text.
#[derive(Clone, Debug)]
struct Part {
    plain: String,
    coloured: String,
}

fn part(plain: String, coloured: String) -> Part {
    Part { plain, coloured }
}

fn is_long(option: &str) -> bool {
    pytext::py_len(option) > 2
}

fn metavar(action: &Action, default: &str) -> String {
    match &action.choices {
        Some(choices) => format!("{{{}}}", choices.join(",")),
        None => default.to_string(),
    }
}

fn format_args(action: &Action, default: &str) -> String {
    let m = metavar(action, default);
    match action.nargs {
        Nargs::One => m,
        Nargs::Parser => format!("{m} ..."),
        Nargs::Zero => String::new(),
    }
}

/// The usage parts; every option is optional, so each is bracketed.
fn usage_parts(p: &Parser, t: &Theme) -> (Vec<Part>, Vec<Part>) {
    let mut optionals = vec![];
    let mut positionals = vec![];
    for action in &p.actions {
        if action.option_strings.is_empty() {
            let text = format_args(action, &action.dest);
            positionals.push(part(
                text.clone(),
                format!("{}{text}{}", t.summary_action, t.reset),
            ));
            continue;
        }
        let option = &action.option_strings[0];
        let colour = if is_long(option) {
            t.summary_long_option
        } else {
            t.summary_short_option
        };
        let (plain, coloured) = if action.nargs == Nargs::Zero {
            (option.clone(), format!("{colour}{option}{}", t.reset))
        } else {
            let args = format_args(action, &action.dest.to_uppercase());
            (
                format!("{option} {args}"),
                format!("{colour}{option} {}{args}{}", t.summary_label, t.reset),
            )
        };
        optionals.push(part(format!("[{plain}]"), format!("[{coloured}]")));
    }
    (optionals, positionals)
}

/// `HelpFormatter._format_usage` without the trailing blank line. Every parser has its help
/// option, so the usage always has at least one part.
fn format_usage(p: &Parser, t: &Theme) -> String {
    let prefix = "usage: ";
    let (opt_parts, pos_parts) = usage_parts(p, t);
    let text_width = width();
    let plain: Vec<&str> = opt_parts
        .iter()
        .chain(&pos_parts)
        .map(|x| x.plain.as_str())
        .collect();
    let plain_usage = std::iter::once(p.prog.as_str())
        .chain(plain)
        .collect::<Vec<_>>()
        .join(" ");
    let rest = if prefix.len() + pytext::py_len(&plain_usage) <= text_width {
        let coloured: Vec<&str> = opt_parts
            .iter()
            .chain(&pos_parts)
            .map(|x| x.coloured.as_str())
            .collect();
        format!(" {}", coloured.join(" "))
    } else {
        wrapped_usage(&p.prog, &opt_parts, &pos_parts, text_width)
    };
    format!(
        "{}{prefix}{}{}{}{}{rest}",
        t.usage, t.reset, t.prog, p.prog, t.reset
    )
}

/// The wrapped usage after the program name, as `_format_usage` lays it out; `opt_parts`
/// always holds the help option.
fn wrapped_usage(prog: &str, opt_parts: &[Part], pos_parts: &[Part], text_width: usize) -> String {
    let prefix_len = "usage: ".len();
    let get_lines = |parts: &[Part], indent: &str, with_prefix: bool| {
        usage_lines(parts, indent, with_prefix.then_some(prefix_len), text_width)
    };
    if (prefix_len + pytext::py_len(prog)) as f64 <= 0.75 * text_width as f64 {
        let indent = " ".repeat(prefix_len + pytext::py_len(prog) + 1);
        let head: Vec<Part> = std::iter::once(part(prog.to_string(), prog.to_string()))
            .chain(opt_parts.iter().cloned())
            .collect();
        let mut lines = get_lines(&head, &indent, true);
        lines.extend(get_lines(pos_parts, &indent, false));
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
    for p in parts {
        let part_len = pytext::py_len(&p.plain);
        if line_len + 1 + part_len > text_width && !line.is_empty() {
            lines.push(format!("{indent}{}", line.join(" ")));
            line.clear();
            line_len = indent_length - 1;
        }
        line.push(&p.coloured);
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

fn invocation(action: &Action, t: &Theme) -> Part {
    if action.option_strings.is_empty() {
        let text = metavar(action, &action.dest);
        return part(text.clone(), format!("{}{text}{}", t.action, t.reset));
    }
    let colour = |s: &String| {
        if is_long(s) {
            format!("{}{s}{}", t.long_option, t.reset)
        } else {
            format!("{}{s}{}", t.short_option, t.reset)
        }
    };
    let plain_options = action.option_strings.join(", ");
    let coloured_options = action
        .option_strings
        .iter()
        .map(colour)
        .collect::<Vec<_>>()
        .join(", ");
    if action.nargs == Nargs::Zero {
        return part(plain_options, coloured_options);
    }
    let args = format_args(action, &action.dest.to_uppercase());
    part(
        format!("{plain_options} {args}"),
        format!("{coloured_options} {}{args}{}", t.label, t.reset),
    )
}

/// One help entry: (indent, header, help).
type Row = (usize, Part, Option<String>);

fn rows_for(p: &Parser, positional: bool, t: &Theme) -> Vec<Row> {
    let mut rows = vec![];
    for action in p
        .actions
        .iter()
        .filter(|a| a.option_strings.is_empty() == positional)
    {
        rows.push((2, invocation(action, t), action.help.clone()));
        if action.kind == Kind::Parsers {
            for (name, help, _) in &p.subparsers {
                if let Some(help) = help {
                    rows.push((
                        4,
                        part(name.clone(), format!("{}{name}{}", t.action, t.reset)),
                        Some(help.clone()),
                    ));
                }
            }
        }
    }
    rows
}

fn normalise(text: &str) -> String {
    text.split(|c: char| c.is_ascii_whitespace())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// The help rows. A help text that is empty or only whitespace shows the header alone
/// (argparse prints a blank line after a whitespace-only one; no help text here is either).
fn format_rows(rows: &[Row], action_max: usize) -> String {
    let width = width();
    let max_help_position = 24.min(width.saturating_sub(20).max(4));
    let help_position = (action_max + 2).min(max_help_position);
    let help_width = width.saturating_sub(help_position).max(11);
    let mut out = String::new();
    for (indent, header, help) in rows {
        let action_width = help_position.saturating_sub(indent + 2);
        let pad = " ".repeat(*indent);
        let Some(text) = help.as_deref().filter(|h| !pytext::strip(h).is_empty()) else {
            out.push_str(&format!("{pad}{}\n", header.coloured));
            continue;
        };
        let indent_first = if pytext::py_len(&header.plain) <= action_width {
            let padded = format!("{pad}{:<action_width$}  ", header.plain);
            out.push_str(&padded.replacen(&header.plain, &header.coloured, 1));
            0
        } else {
            out.push_str(&format!("{pad}{}\n", header.coloured));
            help_position
        };
        let lines = wrap(&normalise(text), help_width);
        out.push_str(&format!("{}{}\n", " ".repeat(indent_first), lines[0]));
        for line in &lines[1..] {
            out.push_str(&format!("{}{line}\n", " ".repeat(help_position)));
        }
    }
    out
}

pub fn format_help(p: &Parser, t: &Theme) -> String {
    let mut text = format!("{}\n\n", format_usage(p, t));
    if let Some(description) = &p.description {
        text.push_str(&format!("{description}\n\n"));
    }
    let positional = rows_for(p, true, t);
    let optional = rows_for(p, false, t);
    let action_max = positional
        .iter()
        .chain(&optional)
        .map(|(indent, header, _)| indent + pytext::py_len(&header.plain))
        .max()
        .unwrap_or(0);
    for (heading, rows) in [
        ("positional arguments", &positional),
        ("options", &optional),
    ] {
        if rows.is_empty() {
            continue;
        }
        text.push_str(&format!(
            "\n{}{heading}:{}\n{}\n",
            t.heading,
            t.reset,
            format_rows(rows, action_max)
        ));
    }
    while text.contains("\n\n\n") {
        text = text.replace("\n\n\n", "\n\n");
    }
    format!("{}\n", text.trim_matches('\n'))
}

fn error_text(p: &Parser, message: &str) -> String {
    let usage = format!("{}\n", format_usage(p, argtheme::theme(Stream::Stderr)));
    let t = argtheme::theme(Stream::Stderr);
    format!(
        "{usage}{}: {}error:{} {}{message}{}\n",
        p.prog, t.error, t.reset, t.message, t.reset
    )
}

// ---------------------------------------------------------------- parsing

/// An `ArgumentError`: the argument name (if any) and the message.
struct ArgError(Option<String>, String);

impl ArgError {
    fn render(&self) -> String {
        match &self.0 {
            Some(name) => format!("argument {name}: {}", self.1),
            None => self.1.clone(),
        }
    }
}

fn action_name(action: &Action) -> String {
    if !action.option_strings.is_empty() {
        action.option_strings.join("/")
    } else {
        action.dest.clone()
    }
}

/// Failure modes inside one parser: its own argument error, or an exit already decided by a
/// sub-parser (which reports with its own prog and usage).
enum Fail {
    Arg(ArgError),
    Exit(Exit),
}

impl From<ArgError> for Fail {
    fn from(e: ArgError) -> Self {
        Fail::Arg(e)
    }
}

/// One `_parse_optional` interpretation: the action index (None for an unknown option), the
/// option string, the separator and any explicit argument.
type OptionTuple = (Option<usize>, String, Option<String>, Option<String>);

fn looks_like_negative_number(arg: &str) -> bool {
    let Some(body) = arg.strip_prefix('-') else {
        return false;
    };
    let digit = |c: char| pytext::decimal_value(c as u32).is_some();
    (!body.is_empty() && body.chars().all(digit))
        || body.split_once('.').is_some_and(|(int, frac)| {
            int.chars().all(digit) && !frac.is_empty() && frac.chars().all(digit)
        })
}

impl Parser {
    fn option_map(&self) -> Vec<(String, usize)> {
        let mut map = vec![];
        for (i, action) in self.actions.iter().enumerate() {
            for s in &action.option_strings {
                map.push((s.clone(), i));
            }
        }
        map
    }

    fn parse_optional(&self, arg: &str) -> Option<Vec<OptionTuple>> {
        if arg.is_empty() || !arg.starts_with('-') {
            return None;
        }
        let map = self.option_map();
        if let Some((s, i)) = map.iter().find(|(s, _)| s == arg) {
            return Some(vec![(Some(*i), s.clone(), None, None)]);
        }
        if pytext::py_len(arg) == 1 {
            return None;
        }
        if let Some((name, value)) = arg.split_once('=')
            && let Some((s, i)) = map.iter().find(|(s, _)| s == name)
        {
            return Some(vec![(
                Some(*i),
                s.clone(),
                Some("=".into()),
                Some(value.into()),
            )]);
        }
        let tuples = self.option_tuples(arg, &map);
        if !tuples.is_empty() {
            return Some(tuples);
        }
        if looks_like_negative_number(arg) {
            return None;
        }
        if arg.contains(' ') {
            return None;
        }
        Some(vec![(None, arg.to_string(), None, None)])
    }

    fn option_tuples(&self, arg: &str, map: &[(String, usize)]) -> Vec<OptionTuple> {
        let mut result = vec![];
        let second = arg.chars().nth(1);
        let (prefix, sep, explicit) = match arg.split_once('=') {
            Some((p, v)) => (p, Some("=".to_string()), Some(v.to_string())),
            None => (arg, None, None),
        };
        if second == Some('-') {
            for (s, i) in map {
                if s.starts_with(prefix) {
                    result.push((Some(*i), s.clone(), sep.clone(), explicit.clone()));
                }
            }
        } else {
            let short: String = arg.chars().take(2).collect();
            let short_explicit: String = arg.chars().skip(2).collect();
            for (s, i) in map {
                if *s == short {
                    result.push((
                        Some(*i),
                        s.clone(),
                        Some(String::new()),
                        Some(short_explicit.clone()),
                    ));
                } else if s.starts_with(prefix) {
                    result.push((Some(*i), s.clone(), sep.clone(), explicit.clone()));
                }
            }
        }
        result
    }

    /// `_get_nargs_pattern`: options take one argument or none, and the only positional is
    /// the subcommand.
    fn nargs_pattern(action: &Action) -> &'static str {
        match action.nargs {
            Nargs::One => "([A])",
            Nargs::Zero => "()",
            Nargs::Parser => "(-*A[-AO]*)",
        }
    }

    /// The arguments an option consumes; only a store option can find none.
    fn match_argument(action: &Action, pattern: &str) -> Result<usize, ArgError> {
        let re = crate::sre::compile_str(Self::nargs_pattern(action), 0);
        match re.match_start(&points(pattern)) {
            Some(c) => Ok(span_len(&c, 1)),
            None => Err(ArgError(
                Some(action_name(action)),
                "expected one argument".into(),
            )),
        }
    }

    /// The arguments each leading positional consumes. The subcommand's pattern needs an
    /// argument, so no count is ever 0.
    fn match_partial(actions: &[&Action], pattern: &str) -> Vec<usize> {
        for i in (1..=actions.len()).rev() {
            let joined: String = actions[..i]
                .iter()
                .map(|a| Self::nargs_pattern(a))
                .collect();
            let re = crate::sre::compile_str(&joined, 0);
            if let Some(c) = re.match_start(&points(pattern)) {
                return (1..=re.groups).map(|g| span_len(&c, g)).collect();
            }
        }
        vec![]
    }

    /// The subcommand name must be one of the choices.
    fn check(action: &Action, value: &str) -> Result<(), ArgError> {
        let choices = action.choices.as_deref().unwrap_or_default();
        if choices.iter().any(|c| c == value) {
            return Ok(());
        }
        let quoted: Vec<String> = choices.iter().map(|c| pytext::repr(c)).collect();
        let names: Vec<&str> = choices.iter().map(String::as_str).collect();
        let suggestion = close_match(value, &names)
            .map(|m| format!(", maybe you meant {}?", pytext::repr(m)))
            .unwrap_or_default();
        Err(ArgError(
            Some(action_name(action)),
            format!(
                "invalid choice: {}{suggestion} (choose from {})",
                pytext::repr_os(value),
                quoted.join(", ")
            ),
        ))
    }

    fn take(
        &self,
        index: usize,
        args: &[String],
        ns: &mut Namespace,
        extras: &mut Vec<String>,
        seen: &mut Vec<usize>,
    ) -> Result<(), Fail> {
        seen.push(index);
        let action = &self.actions[index];
        match action.kind {
            Kind::Help => {
                return Err(Fail::Exit(Exit::Help(format_help(
                    self,
                    argtheme::theme(Stream::Stdout),
                ))));
            }
            // match_argument gave a store option exactly one argument.
            Kind::Store => {
                ns.insert(action.dest.clone(), Value::Str(args[0].clone()));
            }
            Kind::Parsers => {
                Self::check(action, &args[0])?;
                ns.insert(action.dest.clone(), Value::Str(args[0].clone()));
                let (_, _, sub) = self
                    .subparsers
                    .iter()
                    .find(|(name, _, _)| *name == args[0])
                    .expect("checked choice");
                let (sub_ns, sub_extras) = sub.parse_known(&args[1..]).map_err(Fail::Exit)?;
                ns.extend(sub_ns);
                extras.extend(sub_extras);
            }
        }
        Ok(())
    }

    /// `parse_known_args`: errors are reported by this parser.
    pub fn parse_known(&self, arg_strings: &[String]) -> Result<(Namespace, Vec<String>), Exit> {
        self.parse_inner(arg_strings).map_err(|f| match f {
            Fail::Arg(e) => Exit::Error(error_text(self, &e.render())),
            Fail::Exit(e) => e,
        })
    }

    fn parse_inner(&self, arg_strings: &[String]) -> Result<(Namespace, Vec<String>), Fail> {
        let mut ns = Namespace::new();
        for action in &self.actions {
            if action.kind != Kind::Help {
                ns.insert(action.dest.clone(), action.default.clone());
            }
        }
        let mut option_indices: BTreeMap<usize, Vec<OptionTuple>> = BTreeMap::new();
        let mut pattern = String::new();
        let mut after_dashes = false;
        for (i, arg) in arg_strings.iter().enumerate() {
            if after_dashes {
                pattern.push('A');
            } else if arg == "--" {
                pattern.push('-');
                after_dashes = true;
            } else {
                match self.parse_optional(arg) {
                    None => pattern.push('A'),
                    Some(tuples) => {
                        option_indices.insert(i, tuples);
                        pattern.push('O');
                    }
                }
            }
        }
        let mut extras: Vec<String> = vec![];
        let mut seen: Vec<usize> = vec![];
        let mut positionals: Vec<usize> = self
            .actions
            .iter()
            .enumerate()
            .filter(|(_, a)| a.option_strings.is_empty())
            .map(|(i, _)| i)
            .collect();

        // The only positional is the subcommand, which drops a leading "--".
        let consume_positionals = |start: usize,
                                   ns: &mut Namespace,
                                   extras: &mut Vec<String>,
                                   seen: &mut Vec<usize>,
                                   positionals: &mut Vec<usize>|
         -> Result<usize, Fail> {
            let actions: Vec<&Action> = positionals.iter().map(|&i| &self.actions[i]).collect();
            let counts = Self::match_partial(&actions, &pattern[start..]);
            let mut index = start;
            for (k, &count) in counts.iter().enumerate() {
                let action_index = positionals[k];
                let mut args: Vec<String> = arg_strings[index..index + count].to_vec();
                if pattern.as_bytes().get(index) == Some(&b'-') {
                    args.remove(0);
                }
                index += count;
                self.take(action_index, &args, ns, extras, seen)?;
            }
            positionals.drain(..counts.len());
            Ok(index)
        };

        let max_option = option_indices.keys().max().map_or(-1, |&m| m as i64);
        let mut start = 0usize;
        while (start as i64) <= max_option {
            let mut next = start;
            while (next as i64) <= max_option && !option_indices.contains_key(&next) {
                next += 1;
            }
            if start != next {
                let end =
                    consume_positionals(start, &mut ns, &mut extras, &mut seen, &mut positionals)?;
                if end > start {
                    start = end;
                    continue;
                }
                start = end;
            }
            if !option_indices.contains_key(&start) {
                extras.extend(arg_strings[start..next].iter().cloned());
                start = next;
            }
            start = self.consume_optional(
                start,
                arg_strings,
                &pattern,
                &option_indices,
                &mut ns,
                &mut extras,
                &mut seen,
            )?;
        }
        let stop = consume_positionals(start, &mut ns, &mut extras, &mut seen, &mut positionals)?;
        extras.extend(arg_strings[stop..].iter().cloned());
        // The subcommand is the only required argument.
        let missing: Vec<String> = self
            .actions
            .iter()
            .enumerate()
            .filter(|(i, a)| a.kind == Kind::Parsers && !seen.contains(i))
            .map(|(_, a)| action_name(a))
            .collect();
        if !missing.is_empty() {
            return Err(Fail::Arg(ArgError(
                None,
                format!(
                    "the following arguments are required: {}",
                    missing.join(", ")
                ),
            )));
        }
        Ok((ns, extras))
    }

    #[allow(clippy::too_many_arguments)]
    fn consume_optional(
        &self,
        start: usize,
        arg_strings: &[String],
        pattern: &str,
        option_indices: &BTreeMap<usize, Vec<OptionTuple>>,
        ns: &mut Namespace,
        extras: &mut Vec<String>,
        seen: &mut Vec<usize>,
    ) -> Result<usize, Fail> {
        let tuples = &option_indices[&start];
        if tuples.len() > 1 {
            let names: Vec<&str> = tuples.iter().map(|t| t.1.as_str()).collect();
            return Err(Fail::Arg(ArgError(
                None,
                format!(
                    "ambiguous option: {} could match {}",
                    arg_strings[start],
                    names.join(", ")
                ),
            )));
        }
        let (mut action, mut option_string, mut sep, mut explicit) = tuples[0].clone();
        let mut action_tuples: Vec<(usize, Vec<String>, String)> = vec![];
        let stop;
        loop {
            let Some(index) = action else {
                extras.push(arg_strings[start].clone());
                return Ok(start + 1);
            };
            let act = &self.actions[index];
            if let Some(arg) = explicit.clone() {
                let count = Self::match_argument(act, "A")?;
                let second = option_string.chars().nth(1);
                if count == 0 && second != Some('-') && !arg.is_empty() {
                    if sep.as_deref().is_some_and(|s| !s.is_empty()) || arg.starts_with('-') {
                        return Err(Fail::Arg(ArgError(
                            Some(action_name(act)),
                            format!("ignored explicit argument {}", pytext::repr_os(&arg)),
                        )));
                    }
                    action_tuples.push((index, vec![], option_string.clone()));
                    let first: String = arg.chars().take(1).collect();
                    option_string = format!("-{first}");
                    let rest: String = arg.chars().skip(1).collect();
                    match self.option_map().iter().find(|(s, _)| *s == option_string) {
                        Some((_, i)) => {
                            action = Some(*i);
                            if rest.is_empty() {
                                sep = None;
                                explicit = None;
                            } else if let Some(r) = rest.strip_prefix('=') {
                                sep = Some("=".into());
                                explicit = Some(r.into());
                            } else {
                                sep = Some(String::new());
                                explicit = Some(rest);
                            }
                        }
                        None => {
                            extras.push(format!("-{arg}"));
                            stop = start + 1;
                            break;
                        }
                    }
                } else if count == 1 {
                    stop = start + 1;
                    action_tuples.push((index, vec![arg], option_string.clone()));
                    break;
                } else {
                    return Err(Fail::Arg(ArgError(
                        Some(action_name(act)),
                        format!("ignored explicit argument {}", pytext::repr_os(&arg)),
                    )));
                }
            } else {
                let begin = start + 1;
                let count = Self::match_argument(act, &pattern[begin..])?;
                stop = begin + count;
                action_tuples.push((
                    index,
                    arg_strings[begin..stop].to_vec(),
                    option_string.clone(),
                ));
                break;
            }
        }
        for (index, args, _) in action_tuples {
            self.take(index, &args, ns, extras, seen)?;
        }
        Ok(stop)
    }

    /// `parse_args`: leftover arguments are an error for this parser.
    pub fn parse(&self, args: &[String]) -> Result<Namespace, Exit> {
        let (ns, extras) = self.parse_known(args)?;
        if !extras.is_empty() {
            return Err(Exit::Error(error_text(
                self,
                &format!("unrecognized arguments: {}", extras.join(" ")),
            )));
        }
        Ok(ns)
    }
}

// ---------------------------------------------------------------- difflib

/// The ratio difflib.SequenceMatcher(None, a, b).ratio() computes for a choice `a`, which is
/// never empty. difflib's autojunk prunes popular characters only from a `b` of 200 or more
/// characters, and the extension loops after its longest match only grow a match across junk
/// or pruned characters; neither is kept. A word that long cannot reach `close_match`'s 0.6
/// cutoff against a choice under 86 characters, and every choice is a subcommand name.
fn sequence_ratio(a: &[u32], b: &[u32]) -> f64 {
    let mut b2j: BTreeMap<u32, Vec<usize>> = BTreeMap::new();
    for (j, c) in b.iter().enumerate() {
        b2j.entry(*c).or_default().push(j);
    }
    let mut matches = 0;
    let mut queue = vec![(0, a.len(), 0, b.len())];
    while let Some(range) = queue.pop() {
        let (alo, ahi, blo, bhi) = range;
        let (i, j, k) = longest_match(a, &b2j, range);
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
    2.0 * matches as f64 / (a.len() + b.len()) as f64
}

fn longest_match(
    a: &[u32],
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
    (besti, bestj, bestsize)
}

/// `difflib.get_close_matches(word, possibilities, n=1)`.
fn close_match<'a>(word: &str, possibilities: &[&'a str]) -> Option<&'a str> {
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

fn points(text: &str) -> Vec<u32> {
    text.chars().map(u32::from).collect()
}

fn span_len(found: &crate::sre::Match, group: usize) -> usize {
    found.span(group).map_or(0, |(start, end)| end - start)
}
