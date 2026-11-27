//! When `rates` colours its help and errors: `_colorize.can_colorize`'s order of
//! PYTHON_COLORS, NO_COLOR, FORCE_COLOR and TERM, against CPython 3.15's answers.
mod cov_b_common;

use cov_b_common::{case, same_as_python};

#[test]
fn python_colors_and_force_color_turn_colour_on() {
    for name in ["python_colors_1", "force_color", "coloured_error"] {
        same_as_python(name);
        let case = case(name);
        assert!(
            format!("{}{}", case.stdout, case.stderr).contains("\x1b[1;34musage: "),
            "{name}"
        );
    }
}

#[test]
fn python_colors_0_no_color_and_a_dumb_terminal_turn_it_off() {
    for name in [
        "no_color_beats_force",
        "python_colors_0_beats_force",
        "dumb_terminal_beats_nothing",
    ] {
        same_as_python(name);
        assert!(!case(name).stdout.contains('\x1b'), "{name}");
    }
}
