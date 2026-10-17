'use strict';
// Input domains for tests/test_properties.py, keyed by test name. `c` is the
// context the Python decorator passes, here the mapping buckets.
module.exports = ({ fc }) => {
  const lower = 'abcdefghijklmnopqrstuvwxyz';
  const letters = lower + lower.toUpperCase();
  const tail = "0123456789&'-";
  // One word matching [first][rest]{0,max}.
  const word = (first, rest, max) =>
    fc.tuple(fc.constantFrom(...first), fc.array(fc.constantFrom(...rest), { maxLength: max }))
      .map(([head, others]) => head + others.join(''));
  // Python's str.title() for ASCII: a letter after any non-letter is upper case.
  const title = (text) => text.replace(/(^|[^A-Za-z])([a-z])/g, (_, before, char) => before + char.toUpperCase());

  // A canonical account name and a variant differing only in case and whitespace.
  const equivalentAccounts = () =>
    fc.record({
      words: fc.array(word(letters, letters + tail, 15), { minLength: 1, maxLength: 4 }),
      casing: fc.constantFrom('lower', 'upper', 'title'),
      separator: fc.constantFrom(' ', '  ', '\t', '\n', '\r\n'),
      left: fc.constantFrom('', ' ', '\t', '\n'),
      right: fc.constantFrom('', ' ', '\t', '\r\n'),
    }).map(({ words, casing, separator, left, right }) => {
      const canonical = words.join(' ').toLowerCase();
      const cased = { lower: canonical, upper: canonical.toUpperCase(), title: title(canonical) }[casing];
      return { $tuple: [canonical, left + cased.split(' ').join(separator) + right] };
    });

  // A reviewed mapping row and the account identity it must keep.
  const rowSpec = (buckets) =>
    fc.record({
      words: fc.array(word(lower, lower + tail, 12), { minLength: 1, maxLength: 3 }),
      prefix: fc.constantFrom(['', ''], ['=', '='], ['@', '@'], ['+ref ', '+ref '], ['-ref ', '-ref '], ['\t=', '=']),
      bucket: fc.constantFrom(...buckets),
      source: fc.constantFrom('reviewed', 'suggested'),
      note: fc.constantFrom('', 'checked', 'invented fixture'),
      amount: fc.constantFrom('', '0', '1.25', '-50'),
    }).map(({ words, prefix: [prefix, normalised], bucket, source, note, amount }) => {
      const name = words.join(' ');
      return {
        $type: 'RowSpec',
        fields: {
          row: { $type: 'MappingRow', fields: { account: prefix + name, bucket, source, note, amount } },
          identity: normalised + name,
        },
      };
    });

  return {
    test_case_and_whitespace_variants_keep_one_logical_identity: () =>
      fc.record({ pair: equivalentAccounts() }),
    test_generated_reviewed_mappings_round_trip_without_identity_loss: (c) =>
      fc.record({
        specs: fc.uniqueArray(rowSpec(c.buckets), {
          minLength: 1,
          maxLength: 8,
          selector: (spec) => spec.fields.identity,
        }),
      }),
  };
};
