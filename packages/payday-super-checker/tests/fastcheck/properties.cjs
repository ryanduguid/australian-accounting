'use strict';
// Input domains for tests/test_properties.py, keyed by test name.
module.exports = ({ fc, scaled, day }) => {
  const money = () => scaled(0, 100_000_000_000, 2); // 0.00 to 1,000,000,000.00
  return {
    test_business_day_addition_counts_only_days_after_the_start: () =>
      fc.record({ start: day('2026-07-01', '2027-07-01'), count: fc.integer({ min: 0, max: 40 }) }),
    test_exposure_scenarios_conserve_the_shortfall_plus_nec_base: () =>
      fc.record({ final_shortfall: money(), nec: money() }),
  };
};
