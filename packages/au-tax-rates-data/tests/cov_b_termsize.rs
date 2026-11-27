//! `shutil.get_terminal_size().columns` as `rates` sizes its help: $COLUMNS wins whether or not
//! $LINES is set, against CPython 3.15's answers.
mod cov_b_common;

use au_tax_rates_data::termsize::py_int;
use cov_b_common::{case, same_as_python};

#[test]
fn columns_sets_the_width_with_or_without_lines() {
    same_as_python("columns_with_lines");
    same_as_python("columns_without_lines");
    // 40 columns wraps the help text that 80 columns keeps on one line.
    let help = case("columns_without_lines").stdout;
    assert!(help.contains("show this help message\n              and exit"));
    assert_eq!(help, case("columns_with_lines").stdout);
}

#[test]
fn an_oversized_column_count_saturates_rather_than_wrapping() {
    assert_eq!(py_int("99999999999999999999"), Some(i64::MAX));
    assert_eq!(py_int("-99999999999999999999"), Some(-i64::MAX));
}
