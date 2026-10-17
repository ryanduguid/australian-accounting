'use strict';
// Input domains for the properties in tests/test_myr.py, keyed by test name.
module.exports = ({ fc, scaled }) => {
  const loan = (maxTerm) =>
    fc.record({
      principal: scaled(1, 10_000_000_000, 2), // 0.01 to 100,000,000.00
      rate: scaled(1, 2_500, 4), // 0.0001 to 0.2500
      term: fc.integer({ min: 1, max: maxTerm }),
    });
  return {
    test_formula_matches_exact_arithmetic_everywhere: () => loan(25),
    test_paying_the_figure_every_year_clears_the_loan_to_rounding: () => loan(25),
    // The property compares the term with the next one, which stays within 25.
    test_a_longer_term_never_raises_the_figure: () => loan(24),
    test_the_figure_covers_interest_and_a_straight_line_share: () => loan(25),
  };
};
