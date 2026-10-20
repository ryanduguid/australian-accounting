# Changes

## 1.0.0 (unreleased)

Standalone Rust binary, replacing Python package 0.2.0 and its `wiptally` import
API. The `schedule` and `review-pack` commands preserve their flags, help, exit
codes, stdout and stderr, and the schedule CSV and review pack bytes. The former
public API (`measure`, `parse_money`) becomes JSON stdin subcommands, documented
in README.md. `examples/job_to_cash.rs` replaces `examples/job_to_cash.py` and
writes the same files. No AASB 15 rule, rounding step, flag or message is
intentionally changed.

### Documented divergences

- Python imports, dataclasses, keyword signatures, exception classes and package
  version lookup are removed. `--version` prints `wip-tally 1.0.0`.
- The JSON boundary takes amounts as decimal strings. A JSON number is refused
  as `<field> must be a string`, where the Python API raised `TypeError` for a
  float ratio. Text that is not a decimal is refused as
  `decimal.InvalidOperation`, as `Decimal(text)` raised it. JSON input is
  limited to 8 MiB and each decimal field to 8,192 bytes; JSON uses serde_json's nesting limit and refuses lone surrogate escapes.
  Other fields follow Python: `parse-money`'s `where` defaults to `amount` when
  omitted and prints as `None` when `null`, a `null` `raw` is `no amount given`,
  and `line_number` is any JSON integer, printed exactly. An optional amount
  that is omitted is `None`; the Python dataclass required it explicitly.
- Where Python raised an uncaught decimal signal (an amount beyond 28
  significant digits reaching a quantisation), the command prints
  `error: decimal.<Signal>` instead of a traceback. Exit 1 and the files already
  written are preserved. The job-to-cash example likewise prints its refusals
  (an existing output folder, assumptions that do not reconcile) as one
  `error:` line with exit 1, and does not reproduce the Python script's argparse
  help and usage text.
- Python formatting and encoding environment overrides (`PYTHONIOENCODING`,
  `PYTHONUTF8=0`, a console code page) are not supported. Output is UTF-8 with
  native text-mode newlines, and lone surrogates are escaped as before. argparse
  colour (`PYTHON_COLORS`, `NO_COLOR`, `FORCE_COLOR`) and width (`COLUMNS`)
  controls remain supported and are differential-tested. A command-line path
  that is not valid Unicode is not supported.
- On Unix, a stdout or stderr the binary cannot write (a closed pipe, a full
  disk) is reported as Python's shutdown reported it: `Exception ignored while
  flushing sys.stdout:` and the OSError on stderr, with exit 120. On Windows the
  binary exits 1 without that message.
- File errors reproduce Python's `[Errno N] ...` and `[WinError N] ...` text for
  the failures the tests and differential exercise (missing files, unreadable
  inputs, an existing folder). Rarer failures, such as an atomic replacement
  failing (Python's message named its random temporary file) or a parent folder
  that is a file several levels up, may be worded differently.
- The Python/PyPI release workflow is removed. Native candidate builds are
  available in CI; a signed native publisher remains a separate release task.

### Consumers

accounting-review-pipeline's monthly-close utility workflows and grant replay
run this component as a uv Python project (`uv run --project
packages/the-wip-tally ... python examples/job_to_cash.py`) from a pinned
australian-accounting revision. They are unaffected until that pin moves past
this change; then they must run `cargo run --locked --example job_to_cash --`
with the same arguments and stop recording a Python runtime for this project.

### Dependencies and reuse

Python 0.2.0 had no third-party runtime dependencies. Rust uses num-bigint,
num-integer and num-traits for exact Decimal coefficients, serde_json for the
JSON boundary, sha2 for the review pack's SHA-256, chrono for today's local date
and libc for Unix terminal state. The Decimal, argparse, text and terminal
helpers come from this repository's Exchequer Tally and Solomon's Sword ports;
the Python string, path, OSError, date, UTF-8 and JSON helpers come from
llm-tax-guardrails. The `csv` reader, including the text layer's 8,192-byte
chunked decoding that sets a UTF-8 error's reported byte position, is a port of
CPython's `_csv.c` and `TextIOWrapper`. This component owns its copies and
depends on no other engine.
