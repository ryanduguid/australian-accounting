The `csv_readers` target writes fabricated bytes to a CSV file and reads it
with the payroll and super-export importers, with vendor detection, and with
the canonical row parser under the default column mapping. A malformed export
must come back as a refusal, which the command line reports as
`error: <message>`; a panic fails the run. The seed corpus holds the
component's examples, its former importer fixtures truncated to 4 KiB, and an
empty file.

On Linux, from `packages/payday-super-checker`, with a nightly toolchain and
`cargo-fuzz` installed, run:

```sh
cargo +nightly fuzz run csv_readers fuzz/corpus -- -max_total_time=60 -max_len=4096 -timeout=2 -rss_limit_mb=1024
```

`.github/workflows/fuzzing.yml` runs the target after main pushes, weekly and
on demand, with read-only repository access and a ten-minute timeout; it is
not a merge gate. `tests/fuzz_corpus.rs` replays the corpus in
`cargo test`. A successful run is bounded evidence, not proof that every input
is handled.
