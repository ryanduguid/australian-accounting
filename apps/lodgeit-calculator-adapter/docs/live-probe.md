# Live probes

A live probe is manual, small and separate from every build. Nothing in this
repository schedules one, and no test in the default suite makes one.

The reason is not caution for its own sake. A build that calls a third-party
service fails when that service is slow, and a team that has seen a build fail
for reasons outside the change stops reading build failures. Upstream
availability must never gate a build here.

## Before you probe

- Use fabricated figures. Every fixture in this component is synthetic and the
  evidence records say so.
- Keep it small. A handful of requests answers the question; a sweep does not
  answer it better and costs the provider, who is running this at their own
  expense with a three-instance cap.
- Re-read `contracts/README.md`. You are about to compare against a reading
  from a particular day.

## The probe

```bash
cd apps/lodgeit-calculator-adapter
BASE=https://fbt-calculator-api-qkp3j5bjnq-ts.a.run.app

uv run --locked --extra dev lodgeit-adapter drift --enable-network --base-url "$BASE"
```

`drift` exits non-zero when the live catalogue differs from the snapshot. Read
the findings, decide what they mean, and edit the snapshot by hand if they are
real. Nothing updates it for you.

Then one calculation, with evidence:

```bash
uv run --locked --extra dev lodgeit-adapter invoke \
  --enable-network --base-url "$BASE" \
  --calculator urn:sbrm:calculator:div7a:at \
  --period urn:sbrm:period:div7a:fy2026 \
  --body fixtures/example-div7a-request.json \
  --evidence-out out/div7a-live.json

uv run --locked --extra dev lodgeit-adapter verify --evidence out/div7a-live.json
```

`out/` is not tracked. An evidence file from a live probe is a local record of
what the service said on a day, not a fixture: committing one would turn a
point-in-time observation into an expectation.

## Reading the result

An `UPSTREAM_UNAVAILABLE` means the service did not answer. It is not a
finding about either engine and it is not a number.

A `NUMERIC_DIFFERENCE` from a trial is worth reporting upstream, but only with
the request, the response, the figure you expected and where that expectation
comes from. The provider asks for exactly that, and says two independent
implementations disagreeing is how several defects were found, including one
where their engine was right.

Before reporting anything, check the case's own derivation. The expected figure
in a case file was derived from the statute; if it disagrees with both engines,
the case is what is wrong.

## What a probe on 18 September 2026 found

One drift check and twelve invocations across three calculators, all with
fabricated inputs. Recorded here because a probe that is never written down has
to be run again to be believed.

**Drift.** The live catalogue carries eighteen FBT calculators the snapshot does
not record. That is the snapshot being deliberately narrow, not the provider
changing: it records the four calculators that have been reviewed.

**The advisory is not shaped the way the standard shows.** `llms.txt` and
`publish.html` describe an `advisory` block with a `notes` array. The live
div7a route returns `advisory.disclaimer`, a single string, beside
`registered_agent_required`. The adapter refused the first response for that
reason, which is what it is for. The snapshot now records both keys under
`advisory_any_of`, and a response with neither is still a contract failure.

**Three other fields the standard does not mention.** A live 200 also carries
`amalgamated_base` echoed back, `period_uri`, and `rate_uris_consumed` beside
the manifest. The manifest's entries use `content_hash`, not the `sha256` key
the publishing standard's example shows.

**The arithmetic agreed.** Seven fabricated loans ran through the trial. Six
matched to the cent across three independent derivations: the figure worked out
by hand from s 109E(6), this repository's engine, and the provider's. The
seventh is the deliberate trap case, and it behaved as designed: the provider
derived a remaining term of 4 from the origination facts while the case supplied
3, and the trial reported a scope mismatch naming the convention difference
rather than an arithmetic error.

| Case | Outcome |
| --- | --- |
| D7A-1 met, first real year | MATCH |
| D7A-2 short repayment | MATCH |
| D7A-3 excess repayment | MATCH |
| D7A-4 term and rate trap | SCOPE_MISMATCH, as designed |
| D7A-5 earlier year's rate | MATCH |
| D7A-6 one-year remaining term | MATCH |
| D7A-7 twenty-five-year term | MATCH |
| D7A-8 nil remaining term | LOCAL_REFUSED, never sent |
| D7A-9 unsupported period | UNSUPPORTED_PERIOD, never sent |

A match says two implementations agreed on a number. It does not say either is
right about the law, and it approves nothing.

This is a point-in-time reading. Re-run it before quoting it.

### Accounting depreciation

Three windows on one fabricated asset: 120,000.00 of plant, five-year life,
prime cost, acquired 1 July 2024, `actual/actual`.

| Window | Outcome | Opening | Charge | Closing |
| --- | --- | ---: | ---: | ---: |
| 1 Jul 2024 to 30 Jun 2025 | MATCH | 120,000.00 | 24,000.00 | 96,000.00 |
| 1 Jan 2024 to 31 Dec 2024 | CONTRACT_FAILURE | 0.00 | 12,098.63 | 107,901.37 |
| 1 Jan 2025 to 30 Jun 2025 | MATCH | 107,901.37 | 11,901.37 | 96,000.00 |

The first is exactly cost over life for an anniversary year, which is what
`actual/actual` promises. The third telescopes with the second: its opening
balance is the second's closing balance, and 12,098.63 plus 11,901.37 is
24,000.00.

The second is the provider's own published pre-acquisition limitation,
reproduced. The cost enters during the window, no `cost_additions` field comes
back, and the three figures leave a 120,000.00 gap. The trial refused it and
named the limitation. Nothing was derived to close it.

### FBT, car statutory formula

A fabricated car: 40,000.00 base value, available all year, no accessories, no
employee contribution, Type 2.

| | Provider | Local worksheet |
| --- | ---: | ---: |
| Category taxable value | 8,000.00 | (input) |
| Grossed up | 15,094.40 | 15,094.40 |
| FBT | 7,094.37 | 7,094.37 |

Only the category taxable value crossed. Two independent engines agreed on the
gross-up and the FBT to the cent, which is worth having, and says nothing about
whether the benefit is reportable or how it should be classified. The type 2
classification was supplied as a reviewed input, not inherited from the
provider's default.

A live 200 from this route also carries `counts_towards_fbt_cap`,
`exempt_under`, `exemption_provenance`, `gross_taxable_value`,
`rfba_notional_grossed_up_t2`, `rfba_notional_taxable_value`, `s8a_inputs` and
`taxable_value_before_reductions`. The depreciation routes add `numeric_mode`.
All are recorded in the snapshot so they read as known rather than as drift.
None is consumed.

### Fano

Not probed. No classification request has been sent from this repository, so
the authentication discrepancy between `llms.txt` and the Fano integration kit
README is still unresolved. Its adapter and its offline tests are complete
against the schema snapshot, and live compatibility is unverified.
