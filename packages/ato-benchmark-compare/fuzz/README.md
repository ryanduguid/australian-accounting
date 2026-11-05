The `csv_readers` target writes fabricated bytes to a CSV file and reads it
as a profit and loss export, in the report style and the neutral
`account,amount` style, and as an account mapping. Both readers use this
engine's copy of payday's CSV reader, which the root
`tests/test_shared_blocks.py` pins to payday's fuzzed original; this target
covers the code around it, which turns rows into accounts and amounts. A
malformed file must come back as a refusal, which the command line reports as
`error: <message>`; a panic fails the run. The seed corpus holds the
component's 2 example files, a fabricated neutral export and an empty file.

On Linux, from `packages/ato-benchmark-compare`, with a nightly toolchain and
`cargo-fuzz` installed, run:

```sh
cargo +nightly fuzz run csv_readers fuzz/corpus -- -max_total_time=60 -max_len=4096 -timeout=2 -rss_limit_mb=1024
```

`.github/workflows/fuzzing.yml` runs the target weekly and
on demand, with read-only repository access and a 10-minute timeout; it is
not a merge gate. `tests/fuzz_corpus.rs` replays the corpus in
`cargo test`. A successful run is bounded evidence, not proof that every input
is handled.
