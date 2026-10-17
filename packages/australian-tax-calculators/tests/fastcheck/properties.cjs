'use strict';
// Input domains for tests/test_properties.py, keyed by test name. `c` is the
// context the Python decorator passes, such as the supported years.
module.exports = ({ fc, scaled }) => {
  const money = () => scaled(0, 10_000_000_000, 2); // 0.00 to 100,000,000.00
  const dollars = () => scaled(0, 2_000_000, 0); // whole dollars to 2,000,000
  const pay = () => scaled(0, 10_000_000, 2); // 0.00 to 100,000.00
  const age = () => fc.integer({ min: 0, max: 150 });
  const year = (c) => fc.constantFrom(...c.years);
  return {
    test_gst_and_the_exclusive_amount_add_to_the_inclusive_amount_to_the_cent: () =>
      fc.record({ amount: money(), inclusive: fc.boolean() }),
    test_resident_tax_never_falls_and_its_marginal_rate_never_falls: (c) =>
      fc.record({ year: year(c), low: dollars(), high: dollars() }),
    test_payg_withholding_never_falls_as_earnings_rise: (c) =>
      fc.record({
        year_scale: fc.constantFrom(...c.scales),
        period: fc.constantFrom('weekly', 'fortnightly', 'monthly'),
        low: pay(),
        high: pay(),
      }),
    test_study_loan_repayment_never_falls_and_never_exceeds_a_tenth: (c) =>
      fc.record({ year: year(c), low: dollars(), high: dollars() }),
    test_pension_minimum_never_falls_with_balance_or_age: (c) =>
      fc.record({
        year: year(c),
        low: money(),
        high: money(),
        younger: age(),
        older: age(),
        days: fc.integer({ min: 1, max: 365 }),
      }),
    test_capital_losses_are_all_accounted_for_and_more_losses_never_raise_the_gain: () =>
      fc.record({ other: money(), discount: money(), current: money(), prior: money(), extra: money() }),
    test_cap_room_and_excess_reconcile_to_the_cap_available: (c) =>
      fc.record({
        year: year(c),
        balance: money(),
        concessional: money(),
        unused: money(),
        non_concessional: money(),
        under_75: fc.boolean(),
      }),
  };
};
