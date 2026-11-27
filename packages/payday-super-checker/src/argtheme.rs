//! argparse's colour theme (Python 3.14+) and `_colorize.can_colorize`, which
//! decides per stream whether help, usage and errors are coloured.

/// `_colorize.Argparse`: the escape codes argparse writes, or all empty.
pub struct Theme {
    pub usage: &'static str,
    pub prog: &'static str,
    pub heading: &'static str,
    pub summary_long_option: &'static str,
    pub summary_short_option: &'static str,
    pub summary_label: &'static str,
    pub summary_action: &'static str,
    pub long_option: &'static str,
    pub short_option: &'static str,
    pub label: &'static str,
    pub action: &'static str,
    pub reset: &'static str,
    pub error: &'static str,
    pub message: &'static str,
}

pub const COLOURED: Theme = Theme {
    usage: "\x1b[1;34m",
    prog: "\x1b[1;35m",
    heading: "\x1b[1;34m",
    summary_long_option: "\x1b[36m",
    summary_short_option: "\x1b[32m",
    summary_label: "\x1b[33m",
    summary_action: "\x1b[32m",
    long_option: "\x1b[1;36m",
    short_option: "\x1b[1;32m",
    label: "\x1b[1;33m",
    action: "\x1b[1;32m",
    reset: "\x1b[0m",
    error: "\x1b[1;35m",
    message: "\x1b[35m",
};

pub const PLAIN: Theme = Theme {
    usage: "",
    prog: "",
    heading: "",
    summary_long_option: "",
    summary_short_option: "",
    summary_label: "",
    summary_action: "",
    long_option: "",
    short_option: "",
    label: "",
    action: "",
    reset: "",
    error: "",
    message: "",
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Stream {
    Stdout,
    Stderr,
}

/// The theme argparse uses when it writes to `stream`.
pub fn theme(stream: Stream) -> &'static Theme {
    let tty = || console::supports_virtual_terminal() && console::isatty(stream);
    if can_colorize(|name| std::env::var_os(name), tty) {
        &COLOURED
    } else {
        &PLAIN
    }
}

/// `_colorize.can_colorize(file=stream)` for a command started without `-I`,
/// given the environment and whether the stream is a colour terminal.
fn can_colorize(
    var: impl Fn(&str) -> Option<std::ffi::OsString>,
    tty: impl FnOnce() -> bool,
) -> bool {
    match var("PYTHON_COLORS").as_deref().and_then(|v| v.to_str()) {
        Some("0") => return false,
        Some("1") => return true,
        _ => {}
    }
    if var("NO_COLOR").is_some_and(|v| !v.is_empty()) {
        return false;
    }
    if var("FORCE_COLOR").is_some_and(|v| !v.is_empty()) {
        return true;
    }
    if var("TERM").is_some_and(|v| v == "dumb") {
        return false;
    }
    tty()
}

#[cfg(unix)]
mod console {
    use super::Stream;

    pub fn supports_virtual_terminal() -> bool {
        true
    }

    /// `os.isatty(fd)`.
    pub fn isatty(stream: Stream) -> bool {
        let fd = if stream == Stream::Stdout { 1 } else { 2 };
        // SAFETY: isatty only inspects the descriptor number.
        // nosemgrep: rust.lang.security.unsafe-usage.unsafe-usage
        unsafe { libc::isatty(fd) == 1 }
    }
}

#[cfg(windows)]
mod console {
    use super::Stream;

    type Handle = *mut core::ffi::c_void;
    const STD_OUTPUT_HANDLE: u32 = -11i32 as u32;
    const STD_ERROR_HANDLE: u32 = -12i32 as u32;
    const FILE_TYPE_CHAR: u32 = 2;
    const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 4;

    unsafe extern "system" {
        fn GetStdHandle(which: u32) -> Handle;
        fn GetFileType(file: Handle) -> u32;
        fn GetConsoleMode(console: Handle, mode: *mut u32) -> i32;
    }

    /// `nt._supports_virtual_terminal()`: the console on stderr processes escape codes.
    pub fn supports_virtual_terminal() -> bool {
        let mut mode = 0u32;
        // SAFETY: GetStdHandle takes a constant; GetConsoleMode writes one u32.
        // nosemgrep: rust.lang.security.unsafe-usage.unsafe-usage
        let ok = unsafe { GetConsoleMode(GetStdHandle(STD_ERROR_HANDLE), &mut mode) } != 0;
        ok && mode & ENABLE_VIRTUAL_TERMINAL_PROCESSING != 0
    }

    /// `os.isatty(fd)`, which the C runtime answers from the handle's file type.
    pub fn isatty(stream: Stream) -> bool {
        let which = if stream == Stream::Stdout {
            STD_OUTPUT_HANDLE
        } else {
            STD_ERROR_HANDLE
        };
        // SAFETY: GetStdHandle takes a constant and GetFileType only inspects the handle.
        // nosemgrep: rust.lang.security.unsafe-usage.unsafe-usage
        unsafe { GetFileType(GetStdHandle(which)) == FILE_TYPE_CHAR }
    }
}

#[cfg(not(any(unix, windows)))]
mod console {
    use super::Stream;

    pub fn supports_virtual_terminal() -> bool {
        false
    }

    pub fn isatty(_stream: Stream) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<OsString> {
        let pairs: Vec<(String, OsString)> =
            pairs.iter().map(|(k, v)| (k.to_string(), v.into())).collect();
        move |name| pairs.iter().find(|(k, _)| k == name).map(|(_, v)| v.clone())
    }

    #[test]
    fn colour_follows_python_colorize_precedence() {
        let tty = || true;
        assert!(!can_colorize(env(&[("PYTHON_COLORS", "0"), ("FORCE_COLOR", "1")]), tty));
        assert!(can_colorize(env(&[("PYTHON_COLORS", "1"), ("NO_COLOR", "1")]), || false));
        assert!(!can_colorize(env(&[("NO_COLOR", "1"), ("FORCE_COLOR", "1")]), tty));
        assert!(can_colorize(env(&[("NO_COLOR", ""), ("FORCE_COLOR", "1")]), || false));
        assert!(!can_colorize(env(&[("TERM", "dumb")]), tty));
        assert!(can_colorize(env(&[("PYTHON_COLORS", "2")]), tty));
        assert!(!can_colorize(env(&[]), || false));
    }

    #[cfg(unix)]
    #[test]
    fn the_unix_console_check_agrees_with_std() {
        use std::io::IsTerminal;
        assert!(console::supports_virtual_terminal());
        assert_eq!(console::isatty(Stream::Stdout), std::io::stdout().is_terminal());
        assert_eq!(console::isatty(Stream::Stderr), std::io::stderr().is_terminal());
    }
}
