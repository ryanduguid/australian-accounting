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
