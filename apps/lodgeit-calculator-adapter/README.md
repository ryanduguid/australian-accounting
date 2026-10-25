# LodgeiT calculator adapter

Crate `lodgeit-calculator-adapter`, command `lodgeit-adapter`. A Rust port of
the Python `lodgeitadapter` package, which this repository no longer carries.

**Optional, off by default, and never on a fallback path.** This component
calls the [LodgeiT Labs calculator constellation](https://lodgeit.org/calculators.html)
over the network, and it does so only when someone switches it on. Nothing else
in this repository depends on it. The Aus Accounting MCP stays offline, every
engine stays offline, and a local calculation that fails does not reach out to
anything.

Unreleased development source. Not on crates.io, not published anywhere.

This application also has an [opt-in Core client catalogue](docs/core-security.md)
through a fixed read-only listing route. The [Core Clients API preview note](docs/core-clients-api-preview.md)
records the broader public documentation inspected on 9 October 2026 and the
boundaries for any additional client or workspace operations.

The [public transform interface note](docs/public-transform-surfaces.md) records
separate document, journal and asset migration interfaces discovered through
LodgeiT's subdomains, including their observed availability and input patterns.

## Three switches, not one

A request is sent only when all 3 are true:

1. remote access is enabled (`enabled: true` in `AdapterConfig`,
   `LODGEIT_ADAPTER_ENABLED=1`, or `--enable-network` for one run);
2. a base URL is configured;
3. that base URL's host is on the allowlist.

The CLI asks for `--enable-network` every time it invokes a calculator.

Constructing a config, running discovery with remote access off, and recovering
from an error all send nothing. `tests/no_network_by_default.rs` proves each of
those with a dialler that fails the test on any connection attempt.

The Python package also had an MCP surface, `invoke_calculator` with a per-call
`network_acknowledged` switch. It was never released and nothing used it, so
the Rust port omits it. The offline Aus Accounting MCP is a separate application
and is unaffected.

## Build

Install Rust with rustup, then run from this directory:

```bash
cargo build --locked --release
```

The toolchain is pinned in `rust-toolchain.toml`. The executable is
`target/release/lodgeit-adapter` (`.exe` on Windows). It reads the reviewed
snapshots from this directory's `contracts/`, or from a `contracts/` directory
beside the executable when this source tree is not where it was built. No
Python runtime is needed.

## Use it

```bash
cargo run --locked --release -- contract
```

That reads the reviewed contract snapshot and touches no network. To call the
service:

```bash
cargo run --locked --release -- drift \
  --enable-network --base-url https://fbt-calculator-api-qkp3j5bjnq-ts.a.run.app
```

`drift` fetches live discovery and reports how it differs from the reviewed
snapshot. It never edits the snapshot: a contract change is for a person to
read and accept, and there is no code path that accepts one.

```bash
cargo run --locked --release -- invoke \
  --enable-network --base-url https://fbt-calculator-api-qkp3j5bjnq-ts.a.run.app \
  --calculator urn:sbrm:calculator:div7a:at \
  --period urn:sbrm:period:div7a:fy2026 \
  --body fixtures/example-div7a-request.json \
  --evidence-out out/div7a-evidence.json
```

Every request you send should carry fabricated figures. The evidence file
records `synthetic_input`, and the flag travels with the file.

The evidence writer takes `synthetic` as a `bool`. Verification checks the
types of `synthetic_input` and `validation.accepted` when present, and reports
a `COMPUTED` record with `validation.accepted=false` as inconsistent. Older
records that omit these fields keep their existing verification behaviour.

A body file is JSON, and the money in it is written as a decimal string. The
snapshot's `request_number_fields` names which fields become JSON numbers on
the wire, and nothing else is converted: a reference of `0012` and an ABN stay
the strings they were written as, because their digits are an identity rather
than a quantity. A field the snapshot names that is not a number makes the
command refuse the body instead of sending a guess.

`--evidence-out` writes a record the monthly-close control plane can read. Its
`--label` is a slug, lower-case letters, digits and single hyphens, at most 120
characters, because that is what the consumer keys a source digest on.

## Outcomes, not results

Every call returns an `Outcome` with a `status`. Only `COMPUTED` carries
figures.

| Status | Means |
| --- | --- |
| `COMPUTED` | A 200 that matched the reviewed contract |
| `UPSTREAM_REFUSED` | The provider returned 400 with its own `refusal_class` |
| `UPSTREAM_REJECTED` | The provider returned 422: our request was malformed |
| `UPSTREAM_NOT_FOUND` | Unknown calculator or period URN |
| `UPSTREAM_THROTTLED` | 429 |
| `UPSTREAM_UNAVAILABLE` | 5xx, a timeout, a connection failure, an oversized body |
| `CONTRACT_FAILURE` | Answered, but not in a shape this adapter accepts |
| `REFUSED_TO_SEND` | Not enabled, or the target was not allowed |

A timeout is not a zero. A missing advisory is not a result. A 400 is the
provider working correctly.

## Money

Requests carry JSON numbers, because that is what the provider's schemas
declare. They are serialised from exact decimals (`src/decimal.rs`, a port of
Python's `decimal`) straight to their own digits, so `55000.10` reaches the
wire as `55000.10` and not as `55000.099999999998`. Responses are decoded with
every number kept as its digits, so the provider's figures survive whether it
sent a string or a number. `tests/wire_precision.rs` asserts against the bytes
the server received, not against a value that has been through a float.

## What is in here

| Path | What |
| --- | --- |
| `src/config.rs` | The three switches, the allowlist, the route list |
| `src/transport.rs`, `src/dial.rs`, `src/https.rs` | Bounded HTTP over rustls, no redirects, a byte ceiling, retries only where they are safe |
| `src/contract.rs` | Reviewed snapshots and drift reporting |
| `src/client.rs` | Discovery, invocation and the outcome table above |
| `src/decimal.rs`, `src/wire.rs` | Exact decimals in both directions |
| `src/evidence.rs` | Digest-bound evidence files |
| `src/cli.rs` | The command line |
| `src/core.rs` | The Core client catalogue preview |
| `src/trials/` | Comparison trials against the local engines |
| `src/py*.rs` and the rest | The Python behaviour the port reproduces: JSON, text, paths, URLs, `argparse` |
| `src/bin/vectors.rs` | The differential harness's runner, built only with the `differential` feature |
| `contracts/` | The reviewed snapshots, with provenance |
| `fixtures/` | Fabricated cases with their derivations |

## The trials

`src/trials/` compares this repository's engines with the provider's
overlapping scope. It links the in-tree Rust engines by path:
`packages/div7a-loan-review` and `packages/australian-tax-calculators`.
Neither engine is the oracle: every case carries its own expected figure,
derived from the statute or the standard and written out in the case file,
and a disagreement is traced back to that.

The evaluation outcomes are `MATCH`, `NUMERIC_DIFFERENCE`, `SCOPE_MISMATCH`,
`UNSUPPORTED_PERIOD`, `CONTRACT_FAILURE`, `UPSTREAM_UNAVAILABLE`,
`LOCAL_REFUSED` and `NOT_RUN`. **They are not accounting-review states.** A
`MATCH` says two implementations agreed on a number. It does not say a loan
complies, a benefit is reportable, or that a reviewer should sign anything.

- **Division 7A** (`src/trials/div7a.rs`): six conventions are aligned before
  any figure is compared, and the provider's `is_complying` label is recorded
  and never mapped onto the local engine's verdict.
- **FBT** (`src/trials/fbt.rs`): one benefit category, the car statutory
  formula. Only the category taxable value crosses into the local aggregate
  worksheet. The single door that carries it refuses a grossed-up figure by its
  name and by its value, comparing the candidate against the figures the
  provider reported as already grossed up, because a caller that reads the
  wrong field keeps the right name. Both derived figures are then compared, the
  grossed-up amount and the FBT payable; a response carrying neither is a scope
  mismatch, not a match. Type 1 or type 2 is a required reviewed input, never
  the provider's engine-side default.
- **Accounting depreciation** (`src/trials/depreciation.rs`): the provider's
  own movement has to close, or it is refused. It is deliberately not compared
  with the local Division 40 worksheet, which answers a different question.
- **Fano** (`src/trials/fano.rs`): experimental, opt-in, model in the path. It
  produces suggestions and nothing else. `accepted_fact` is the classifier's
  verdict on its own prediction, not an approval, and a LodgeiT chart code needs
  a reviewed crosswalk before it means anything elsewhere.

A figure the provider repeats under one name is read as its last value, and a
repeated crosswalk code takes its last entry, as Python's `dict` reads them.

## Checks

```bash
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo clippy --locked --all-targets --features differential -- -D warnings
cargo test --locked
```

The whole suite runs offline. The only sockets it opens are to loopback servers
the tests start themselves. One test, which checks real certificate handling
against the internet, is ignored by default; run it with
`cargo test --locked -- --ignored`.

`tests/BEHAVIOURS.md` maps each of the Python package's 305 tests to its Rust
test, or records why it has none.

A live probe is separate, manual and small: see
[docs/live-probe.md](docs/live-probe.md). Nothing schedules one, and upstream
availability never gates a build.

One was run on 18 September 2026, with the Python package: a drift check and
eleven invocations across Division 7A, accounting depreciation and one FBT
category, all with fabricated inputs. The arithmetic agreed everywhere it was
meant to, the provider's own published pre-acquisition limitation was
reproduced and refused, and the probe found that the live advisory block is
shaped differently from the published standard. What it found, and the caveat
that it is a point-in-time reading, are in that document. Fano was not probed
and its live compatibility is unverified. The Rust port has not been run
against the live service.

## Core client catalogue preview

The separate `core` module reads one page of non-archived client identities
through a fixed Core API route. It requires explicit enablement and a
host-managed bearer token on each call. It requests no profile expansion,
returns client summaries and makes no automatic pagination or retry. It is
available only through the library, `lodgeit_calculator_adapter::core`.

Read [Core security and usage](docs/core-security.md) before using it. Returned
UUIDs, custom codes, private flags and continuations are sensitive. The public
preview reference was captured on 9 October 2026; offline fabricated tests pass,
and live tenant compatibility remains unverified.

## The Rust port

The port reproduces the Python package's output, exit codes, evidence bytes
and wire bytes. Differential harnesses ran the Python package and this crate
side by side on Windows and Linux over generated and directed cases, from the
command line and transport to the Core client and the trials, with no
unexplained difference. These differences are deliberate:

- **Typed interfaces.** The library takes Rust types, so Python's runtime type
  checks have no counterpart, and a Rust `String` cannot hold a lone surrogate.
  A Division 7A case file's fields are typed when it is read: a list amount is
  refused with `TypeError`, where Python's `Decimal` reads a list as a decimal
  tuple, and repayment amounts are converted when the file is read, where
  Python converts them at first use. A Fano line's code and confidence cannot
  be `None`.
- **Non-Unicode input is refused.** A command-line argument that is not valid
  Unicode exits with status 2, and a `LODGEIT_ADAPTER_*` variable that is not
  valid Unicode is a configuration error. Python carries such bytes as
  surrogates.
- **Output.** Standard output is always UTF-8; `PYTHONIOENCODING` is not
  honoured. A failed write to standard output is reported at exit (status
  120, with Python's flush message), where Python can raise during the write
  once its buffer fills. An uncaught exception prints the traceback's header
  and last line, without frames.
- **TLS and hosts.** HTTPS uses rustls with the platform's certificate
  verifier. TLS and operating system error wording differs from OpenSSL's and
  Python's; the outcomes map them to the same status and do not show the
  wording. A non-ASCII allowlisted host, which Python would encode with IDNA,
  is refused. A socket close cannot fail.
- **Limits.** Nesting limits for JSON and `repr` reproduce Python's
  `RecursionError` points from measured stack costs. On Linux, where the stack
  starts at a different place in each process, a point can move by up to about
  7.6 KB of stack. Text is held as 32-bit code points, so the size at which
  memory runs out differs. A backoff sleep longer than about 292 years sleeps
  on Linux, where CPython refuses it. A lone surrogate in a route reaches the
  request URL as U+FFFD.
- **Core.** Python clears the token, request body and response from its frame
  locals so a traceback cannot show them. A Rust `CoreError` holds only the
  category and status, and the token stays the caller's borrowed string.
  `list_clients_via` takes any dialler, which is how the tests stub the socket.
- **Engines.** The trials link the in-tree Rust engines, which match
  `div7a-loan-review` 0.3.2 and `australian-tax-calculators` 0.1.9. The FBT
  trial's local figures therefore also carry `return_item_14a`, `14b`, `15`
  and `16`, which the Python package's pinned 0.1.3 did not return. They are
  never compared, and every evaluation is unchanged.
- **MCP.** The MCP surface is omitted, as described above.

## Boundary

This adapter calls a third-party service that says of itself: "Response shapes
are not yet versioned and may change; nothing here is stable for production
integration." Figures it returns are calculated on facts you supplied. They are
not advice, not a lodgement and not a review sign-off. The contract snapshots record a reading of the public service on one day.
