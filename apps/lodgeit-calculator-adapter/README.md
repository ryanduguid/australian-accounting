# LodgeiT calculator adapter

Distribution `lodgeit-calculator-adapter`, import package `lodgeitadapter`,
command `lodgeit-adapter`.

**Optional, off by default, and never on a fallback path.** This component
calls the [LodgeiT Labs calculator constellation](https://lodgeit.org/calculators.html)
over the network, and it does so only when someone switches it on. Nothing else
in this repository imports it. The Aus Accounting MCP stays offline, every
engine stays offline, and a local calculation that fails does not reach out to
anything.

Unreleased development source. Not on PyPI, not published anywhere.

## Three switches, not one

A request is sent only when all three are true:

1. remote access is enabled (`enabled=True`, `LODGEIT_ADAPTER_ENABLED=1`, or
   `--enable-network` for one run);
2. a base URL is configured;
3. that base URL's host is on the allowlist.

Through the optional MCP surface, `invoke_calculator` needs a fourth: a
`network_acknowledged=true` argument on the call itself. The CLI asks for
`--enable-network` every time it invokes a calculator, and this keeps the MCP
tool on the same footing rather than letting one launch decision cover a whole
session of calls.

Importing the package, constructing a config, running discovery with remote
access off, and recovering from an error all send nothing.
`tests/test_no_network_by_default.py` proves each of those with sockets
sabotaged, including one that imports every module in a fresh interpreter.

## Use it

```bash
uv run --locked --extra dev lodgeit-adapter contract
```

That reads the reviewed contract snapshot and touches no network. To call the
service:

```bash
uv run --locked --extra dev lodgeit-adapter drift \
  --enable-network --base-url https://fbt-calculator-api-qkp3j5bjnq-ts.a.run.app
```

`drift` fetches live discovery and reports how it differs from the reviewed
snapshot. It never edits the snapshot: a contract change is for a person to
read and accept, and there is no code path that accepts one.

```bash
uv run --locked --extra dev lodgeit-adapter invoke \
  --enable-network --base-url https://fbt-calculator-api-qkp3j5bjnq-ts.a.run.app \
  --calculator urn:sbrm:calculator:div7a:at \
  --period urn:sbrm:period:div7a:fy2026 \
  --body fixtures/example-div7a-request.json \
  --evidence-out out/div7a-evidence.json
```

Every request you send should carry fabricated figures. The evidence file
records `synthetic_input`, and the flag travels with the file.

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
declare. They are serialised from `Decimal` straight to their own digits, so
`Decimal("55000.10")` reaches the wire as `55000.10` and not as
`55000.099999999998`. Responses are read with `json.loads(parse_float=str)`, so
the provider's digits survive whether it sent a string or a number.
`tests/test_wire_precision.py` asserts against the bytes the server received,
not against a value that has been through a float.

## What is in here

| Path | What |
| --- | --- |
| `lodgeitadapter/config.py` | The three switches, the allowlist, the route list |
| `lodgeitadapter/transport.py` | Bounded HTTP, no redirects, a byte ceiling, retries only where they are safe |
| `lodgeitadapter/contract.py` | Reviewed snapshots and drift reporting |
| `lodgeitadapter/client.py` | Discovery, invocation and the outcome table above |
| `lodgeitadapter/decimals.py` | Exact decimals in both directions |
| `lodgeitadapter/evidence.py` | Digest-bound evidence files |
| `lodgeitadapter/cli.py` | The command line |
| `lodgeitadapter/mcp.py` | The optional MCP surface, a separate switch again |
| `lodgeitadapter/trials/` | Comparison trials against the local engines |
| `contracts/` | The reviewed snapshots, with provenance |
| `fixtures/` | Fabricated cases with their derivations |

## The trials

`lodgeitadapter/trials/` compares this repository's engines with the provider's
overlapping scope. Neither engine is the oracle: every case carries its own
expected figure, derived from the statute or the standard and written out in
the case file, and a disagreement is traced back to that.

The evaluation outcomes are `MATCH`, `NUMERIC_DIFFERENCE`, `SCOPE_MISMATCH`,
`UNSUPPORTED_PERIOD`, `CONTRACT_FAILURE`, `UPSTREAM_UNAVAILABLE`,
`LOCAL_REFUSED` and `NOT_RUN`. **They are not accounting-review states.** A
`MATCH` says two implementations agreed on a number. It does not say a loan
complies, a benefit is reportable, or that a reviewer should sign anything.

- **Division 7A** (`trials/div7a.py`): six conventions are aligned before any
  figure is compared, and the provider's `is_complying` label is recorded and
  never mapped onto the local engine's verdict.
- **FBT** (`trials/fbt.py`): one benefit category, the car statutory formula.
  Only the category taxable value crosses into the local aggregate worksheet.
  The single door that carries it refuses a grossed-up figure by its name and
  by its value, comparing the candidate against the figures the provider
  reported as already grossed up, because a caller that reads the wrong field
  keeps the right name. Both derived figures are then compared, the grossed-up
  amount and the FBT payable; a response carrying neither is a scope mismatch,
  not a match. Type 1 or type 2 is a required reviewed input, never the
  provider's engine-side default.
- **Accounting depreciation** (`trials/depreciation.py`): the provider's own
  movement has to close, or it is refused. It is deliberately not compared with
  the local Division 40 worksheet, which answers a different question.
- **Fano** (`trials/fano.py`): experimental, opt-in, model in the path. It
  produces suggestions and nothing else. `accepted_fact` is the classifier's
  verdict on its own prediction, not an approval, and a LodgeiT chart code needs
  a reviewed crosswalk before it means anything elsewhere.

## Checks

```bash
uv run --locked --extra dev pytest -q --cov=lodgeitadapter --cov-branch --cov-report=term-missing
uv run --locked --extra dev ruff check lodgeitadapter tests
uv run --locked --extra dev mypy lodgeitadapter
uv run --locked --extra dev --with "pip-audit==2.10.1" pip-audit --local --strict
```

The whole suite runs offline. The only socket it opens is to a `http.server` on
loopback that the test starts itself.

A live probe is separate, manual and small: see
[docs/live-probe.md](docs/live-probe.md). Nothing schedules one, and upstream
availability never gates a build.

One was run on 18 September 2026: a drift check and eleven invocations across
Division 7A, accounting depreciation and one FBT category, all with fabricated
inputs. The arithmetic agreed everywhere it was meant to, the provider's own
published pre-acquisition limitation was reproduced and refused, and the probe
found that the live advisory block is shaped differently from the published
standard. What it found, and the caveat that it is a point-in-time reading, are
in that document. Fano was not probed and its live compatibility is unverified.

## Boundary

This adapter calls a third-party service that says of itself: "Response shapes
are not yet versioned and may change; nothing here is stable for production
integration." Figures it returns are calculated on facts you supplied. They are
not advice, not a lodgement and not a review sign-off. This repository has no
agreement, listing, partnership or endorsement from LodgeiT Labs, and the
contract snapshots here are one reading of a public surface on one day.
