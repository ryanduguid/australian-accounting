# Changes

## 1.0.0 (unreleased)

Standalone Rust binary, replacing Python package 0.2.0 (the last Python release;
its engine source is unchanged since 0.1.8, the parity base) and its import API.
The two original commands preserve flags, help, validation, output and exits.
The former API operations are JSON stdin subcommands, documented in README.md.
No money calculation, rate table or statutory text is intentionally changed.

### Documented divergences

- Unhandled Decimal InvalidOperation in an original command prints
  `error: decimal.InvalidOperation` instead of a Python traceback. Exit 1 and
  empty stdout are preserved. This occurs, for example, when an extremely high
  rate makes a franking credit too large to quantize under 28-digit precision.
- Python imports, dataclasses, keyword signatures, exception classes and package
  version lookup are removed. JSON returns decimal strings and null unknowns;
  a refused API operation exits 2. The native version is Cargo 1.0.0.
- New JSON accounting inputs have explicit resource limits: 8 MiB total,
  8,192 bytes per decimal field and exponent/adjusted exponent within +/-10,000.
  `validate-value` mirrors the original CLI's raw-string validator and is exempt
  from the per-decimal limits. JSON uses serde_json's nesting limit and refuses
  lone Unicode surrogate escapes; Python object strings could contain them.
- Python formatting/encoding environment overrides are not supported. Output
  is UTF-8 with native text-mode newlines; argparse colour and width environment
  controls remain supported. Interactive terminal behaviour is locally inspected
  but not exhaustively differential-tested across terminal implementations.
- The Python/PyPI release workflow is removed. Native candidate builds are
  available in CI; a signed native publisher remains a separate release task.

There are no workbook or output-file formats in this engine. Existing command
stdout/stderr bytes, including CRLF on Windows, are checked by the differential
and frozen subprocess corpus, apart from the named traceback adaptation.

### Dependencies and reuse

Python 0.1.8 had no third-party runtime dependencies. Its Decimal dependency is
replaced by the existing BigUint-based Python Decimal compatibility code from
accounting-review-pipeline's elizabeth-anne-alexander port (Apache-2.0), extended
with ROUND_DOWN and fixed display formatting. Rust num-bigint, num-integer and
num-traits implement exact coefficient arithmetic; serde_json provides the new
JSON boundary; libc reads Unix terminal state. Python argparse/text/terminal
compatibility helpers are adapted from the same qualified port. This component
owns its copies and does not depend on another engine or a shared runtime.
