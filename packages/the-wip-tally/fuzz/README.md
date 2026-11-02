The `contracts` target reads fabricated bytes as a contracts CSV under the
default column mapping, then measures each contract it reads. A malformed
schedule must come back as a refusal, which the command line reports as
`error: <message>`; a panic fails the run. The seed corpus holds the
component's 2 example schedules, a header-only file, a row of accounting
amount formats and an empty file.

On Linux, from `packages/the-wip-tally`, with a nightly toolchain and
`cargo-fuzz` installed, run:

```sh
cargo +nightly fuzz run contracts fuzz/corpus -- -max_total_time=60 -max_len=4096 -timeout=2 -rss_limit_mb=1024
```

`.github/workflows/fuzzing.yml` runs the target weekly and
on demand, with read-only repository access and a 10-minute timeout; it is
not a merge gate. `tests/fuzz_corpus.rs` replays the corpus in
`cargo test`. A successful run is bounded evidence, not proof that every input
is handled.
