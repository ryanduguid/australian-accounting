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
    if can_colorize(stream) {
        &COLOURED
    } else {
        &PLAIN
    }
}

/// `_colorize.can_colorize(file=stream)` for a command started without `-I`.
fn can_colorize(stream: Stream) -> bool {
    can_colorize_with(std::env::var_os, || {
        console::supports_virtual_terminal() && console::isatty(stream)
    })
}

/// `can_colorize` with the environment read by `var` and the stream's
/// terminal checked by `terminal`.
fn can_colorize_with(
    var: impl Fn(&'static str) -> Option<std::ffi::OsString>,
    terminal: impl FnOnce() -> bool,
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
    terminal()
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

    /// `can_colorize_with` under the given variables, on a stream that is a
    /// colour terminal when `tty` is true.
    fn colours(pairs: &'static [(&'static str, &'static str)], tty: bool) -> bool {
        can_colorize_with(
            |name| {
                pairs
                    .iter()
                    .find(|(k, _)| *k == name)
                    .map(|(_, v)| v.into())
            },
            || tty,
        )
    }

    #[test]
    fn colour_follows_python_colorize() {
        // PYTHON_COLORS decides first, either way.
        assert!(colours(&[("PYTHON_COLORS", "1"), ("NO_COLOR", "1")], false));
        assert!(!colours(
            &[("PYTHON_COLORS", "0"), ("FORCE_COLOR", "1")],
            true
        ));
        // Then NO_COLOR, FORCE_COLOR and a dumb terminal; an empty value is unset.
        assert!(!colours(&[("NO_COLOR", "1"), ("FORCE_COLOR", "1")], true));
        assert!(colours(&[("NO_COLOR", ""), ("FORCE_COLOR", "1")], false));
        assert!(!colours(&[("TERM", "dumb")], true));
        // Otherwise only a colour terminal.
        assert!(colours(&[("TERM", "xterm")], true));
        assert!(!colours(&[], false));
    }
}
