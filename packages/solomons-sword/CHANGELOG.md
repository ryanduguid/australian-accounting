# Changes

## 1.0.0 (unreleased)

Standalone Rust binary, replacing Python package 0.3.2 and its import API.
The two original commands preserve flags, help, validation, output and exits.
The former API functions are JSON stdin subcommands, documented in README.md.
No allocation, rounding, zoning rule, statutory text or message is
intentionally changed.

### Documented divergences

- Python imports, dataclasses, keyword signatures, exception classes and package
  version lookup are removed. JSON returns decimal strings and `null` unknowns;
  a refused API operation exits 2 with the original `ValueError` text, or
  `decimal.<Signal>` for a trapped decimal signal. The native version is Cargo
  1.0.0.
- The JSON boundary requires amounts as strings, facts as JSON values and dates
  as `YYYY-MM-DD`, and refuses input over 8 MiB or a decimal field over 8,192
  bytes. JSON uses serde_json's nesting limit and refuses lone Unicode surrogate
  escapes. A `financial_year` outside the 64-bit range is refused as not an
  integer; Python's `date()` raised OverflowError beyond the C int range.
- Python formatting and encoding environment overrides (`PYTHONIOENCODING`,
  `PYTHONUTF8=0`, a console code page) are not supported. Output is UTF-8 with
  native text-mode newlines, and lone surrogates are escaped as before.
  argparse colour (`PYTHON_COLORS`, `NO_COLOR`, `FORCE_COLOR`) and width
  (`COLUMNS`) controls remain supported and are differential-tested.
- The Python/PyPI release workflow and the fast-check property bridge are
  removed. The footing property runs as a seeded Rust test over the same input
  domain. Native candidate builds are available in CI; a signed native
  publisher remains a separate release task.

There are no workbook or output-file formats in this engine. Command stdout and
stderr bytes, including CRLF on Windows, are checked by the differential and the
frozen subprocess corpus.

### Dependencies and reuse

Python 0.3.2 had no third-party runtime dependencies. Its Decimal dependency is
replaced by the BigUint-based Python Decimal compatibility code from The
Exchequer Tally's port (itself adapted from accounting-review-pipeline's
elizabeth-anne-alexander port, Apache-2.0), extended here with NaN, Infinity,
Overflow and DivisionByZero semantics and unary negation. Rust num-bigint,
num-integer and num-traits implement exact coefficient arithmetic; serde_json
provides the JSON boundary; libc reads Unix terminal state. The argparse, text
and terminal helpers come from the same port, extended with mutually exclusive
flag pairs. This component owns its copies and depends on no other engine.
