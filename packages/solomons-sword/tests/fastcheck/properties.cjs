'use strict';
// Input domains for tests/test_properties.py, keyed by test name.
module.exports = ({ fc, scaled }) => ({
  test_every_allocated_column_foots_to_its_pool: () =>
    fc.record({
      weights: fc.array(fc.integer({ min: 1, max: 10_000 }), { minLength: 2, maxLength: 8 }),
      trust_income: scaled(1, 10_000_000_000, 2), // 0.01 to 100,000,000.00
      s95_net: scaled(1, 10_000_000_000, 2),
      credits: scaled(0, 10_000_000_000, 2), // 0.00 to 100,000,000.00
    }),
});
