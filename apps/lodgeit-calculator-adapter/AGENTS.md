# Agent instructions

This component is the only place in this repository that makes a network call,
and it does so only when switched on. Read [README.md](README.md) first.

## Rules that are not negotiable here

- **No engine depends on this crate.** Engines depend on nothing in this
  repository, and `tests/test_boundaries.py` at the root proves it. The
  dependency runs the other way: the trials link the engines by path.
- **No fallback.** There is no code path from a failed local calculation to a
  remote one, and none may be added. A local engine that cannot answer says so.
- **Nothing sends on construction, in offline discovery or during error
  recovery.** `tests/no_network_by_default.rs` holds this. If a
  change makes one of those tests inconvenient, the change is wrong.
- **A refusal is never a number.** A timeout, a malformed body, a missing
  advisory, an unknown schema and a provider 400 each keep their own status.
  Do not add a default, a fallback value or a retry that turns one into a
  figure.
- **Contract snapshots are reviewed by people.** `compare` reports drift and
  nothing writes a snapshot. Do not add a refresh command, an auto-accept flag
  or a test that regenerates a snapshot to make itself pass.
- **Money is exact.** Requests serialise exact decimals to their own digits;
  responses keep every number as its digits. A float in either direction is a
  defect. Check the wire bytes, not a round trip.
- **Fixtures are fabricated.** No client data, no real loan, no real payroll.
  Every case file carries `"synthetic": true` and a derivation.
- **The provider's labels stay the provider's.** `is_complying`,
  `deemed_dividend`, `accepted_fact` and `draft_fact` are recorded as its
  output. None of them may be mapped into a local verdict, an approval field or
  an accounting-review state.

## Adding a calculator

1. Read the live schema and the provider's scope notes.
2. Add it to `contracts/lodgeit-calculators.json` with its route, its supported
   periods, its required and optional decimal fields, and the scope notes that
   matter. Update `read_at` and `read_by` honestly: an automated retrieval is
   not a professional review.
   `request_number_fields` is the list the command line uses to decide which fields of a body read from a file become JSON numbers. It is
   the only thing that decides that: a field the list does not name stays the
   string it was written as, because a reference and an ABN are numeric-looking
   strings whose leading zeros are part of their identity. Paths are dotted and
   `[]` walks a list, as in `repayments[].amount`. Naming a field that is not a
   number makes the command refuse the body rather than send a guess.
3. Add offline tests driven by a stub before anything is sent live.
4. If it has a local counterpart, align the conventions in writing before
   comparing a single figure.

## Checks

Run these from this directory:

```bash
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo clippy --locked --all-targets --features differential -- -D warnings
cargo test --locked
```

This crate has its own `Cargo.lock`; commit it with any dependency change. The
trials link `packages/div7a-loan-review` and `packages/australian-tax-calculators`
by path, so a change to either engine can change a trial's figures.

The port was differentially tested against the retired Python package. The
README's 'The Rust port' section says what matched and lists the deliberate
differences; keep it current when behaviour changes.

Do not release, tag or publish. This component has no release workflow, which
is deliberate: it is development source.
