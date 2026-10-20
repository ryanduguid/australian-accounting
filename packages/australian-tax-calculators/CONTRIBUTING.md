# Contributing

Keep statutory methods in this component. Use independently constructed synthetic
tests and official sources. Preserve scope refusals and source metadata.

Run from this directory:

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked --release
```

`tests/data/python-api.json` is the frozen Python 0.2.0 corpus. Do not edit it
to make a change pass: a deliberate change of result is a documented divergence
in `CHANGELOG.md`, with the corpus entries it changes. The native publisher is
pending; nothing releases this crate yet.
