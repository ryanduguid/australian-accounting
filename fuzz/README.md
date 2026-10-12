The Atheris target writes fabricated bytes to a CSV file and reads it with
Payday Super Checker's payroll and super-export importers, with vendor
detection, and with the canonical row parser under the default column mapping.
A malformed export must be refused with `ValueError` (`CsvError` and the other
refusals subclass it), which the command line reports as `error: <message>`.
Anything else that escapes fails the run. The seed corpus holds the
component's examples, its importer fixtures truncated to 4 KiB, and an empty
file.

On Linux with Python 3.14, from the repository root, run:

```sh
python -m pip install --require-hashes --only-binary :all: -r fuzz/requirements-fuzz.txt
python -m fuzz.payday_super_import fuzz/corpus -max_total_time=60 -max_len=4096 -timeout=2 -rss_limit_mb=1024
```

The component has no runtime dependencies, so the harness imports it from
`packages/payday-super-checker`. `.github/workflows/fuzzing.yml` runs the
target after main pushes, weekly and on demand, with read-only repository
access and a five-minute timeout; it is not a merge gate.
`tests/test_fuzz_harness.py` replays the corpus in the root suite. Atheris
does not support Windows, so coverage-guided fuzzing runs on Linux. A
successful run is bounded evidence, not proof that every input is handled.

After changing the pinned version, regenerate the hashes with:

```sh
uv pip compile fuzz/requirements-fuzz.in --python-version 3.14 --python-platform x86_64-unknown-linux-gnu --generate-hashes --output-file fuzz/requirements-fuzz.txt
```
