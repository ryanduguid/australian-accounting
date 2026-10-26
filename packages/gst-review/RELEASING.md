# Releasing

Native version 1.0.0 is unreleased. `Cargo.toml` owns its version. Keep the citation,
release-note heading and Cargo locks aligned. CI builds and checks Windows, macOS
and Linux archives; it does not publish them. Native publication waits for the
repository's reviewed Rust release workflow and explicit release authorisation.
Do not send the Rust crate through the retired Python publisher.

The published Python 0.1.0 remains available for the MCP façade's exact pin.
Removing its source from this checkout does not withdraw or replace that release.
Historical tags and attestations remain unchanged.

Before a future native release, recheck the current GST Act and ATO BAS sources
recorded in `src/constants.json`, then run the checks documented in the README,
both fabricated examples and the root boundary and umbrella parity checks.
A port parity result does not establish that the law or guidance remains current.
