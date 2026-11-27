// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `shutil.get_terminal_size().columns`: a positive $COLUMNS wins whether or
//! not $LINES is set. One test, as it changes this process's environment.
use lodgeit_calculator_adapter::termsize::columns;

#[test]
fn a_positive_columns_variable_sets_the_width_with_or_without_lines() {
    // SAFETY: this file's only test, so no other thread reads the environment.
    unsafe {
        std::env::set_var("COLUMNS", "100");
        std::env::set_var("LINES", "30");
    }
    assert_eq!(columns(), 100);
    unsafe { std::env::remove_var("LINES") };
    assert_eq!(columns(), 100);
    unsafe { std::env::set_var("COLUMNS", "0") };
    assert!(columns() > 0);
}
