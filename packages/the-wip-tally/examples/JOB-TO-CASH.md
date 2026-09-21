# From WIP to project cash

Generate the supplied contract evidence in the `the-wip-tally` component of `australian-accounting`:

```powershell
uv run --locked --extra dev python examples/job_to_cash.py --output path/to/new-job-pack
```

Then, from the `au-fpa-pack` repository:

```powershell
uv run --locked --extra dev python examples/job-to-cash/project_cash.py --pack path/to/new-job-pack
```

The file hand-off is `job-to-cash.json` plus `cash-assumptions.csv`. The consumer verifies the cash file's SHA-256 and uses the existing `cash13_forecast` kernel. It does not import or reproduce the WIP engine.

The supplied $1 million contract has estimated total cost of $800,000, so forecast gross profit is $200,000. Cost-based progress produces $500,000 revenue to date against $450,000 certified billings. The engine reports a $50,000 contract asset. These amounts are distinct from cash.

Cash received is $300,000 and cash paid is $440,000. Cumulative project cash is therefore -$140,000. Certified gross billings of $495,000 split into $300,000 already received, $145,500 ordinary outstanding collection and $49,500 retention. The $75,000 uncertified claim is visible in the WIP input but contributes no forecast cash.

The supplied forecast places $145,500 collection inside the next 13 weeks and $440,000 remaining cost payments in December. Its cumulative project cash reaches -$434,500. The $49,500 retention is explicitly outside that horizon. This is a deliberately incomplete receipt plan: future unbilled amounts have no collection assumptions. The output is a funding view of the supplied items, not a full contract valuation or an entity bank forecast.

Run the producer again in a new folder with `--extra-cost-to-complete 100000`. Gross profit falls to $100,000 and future cost cash increases by $110,000 under the fabricated 1.10 cost multiplier. The $380,000 outstanding commitment is already inside cost to complete and is never added again. Tax and retention assumptions are supplied facts for this case, not inferred legal conclusions.
