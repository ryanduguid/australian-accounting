# Benchmark comparisons across periods

The source example calls the existing parser, reviewed mapping, ratio and dataset functions. It retains each complete comparison and a separate mapping bridge.

```powershell
uv run --locked --extra dev python examples/benchmark_history.py --previous-pnl examples/bakery-pnl.csv --current-pnl examples/bakery-pnl.csv --previous-mapping examples/bakery-mapping.csv --current-mapping examples/bakery-mapping.csv --previous-period FY2025 --current-period FY2026 --industry "Bakeries and hot bread shops"
```

These supplied files are fabricated. Reusing them demonstrates zero business movement; it is not a claim about actual bakery performance. Supply distinct snapshots and reviewed mappings for a real comparison. Each source digest, declared period, benchmark year, turnover basis and dataset provenance remains visible. Select fixed dataset years with `--previous-year` and `--current-year` for reproducibility.

For supported ratios, the current ledger is also run through the previous mapping. The result separates movement under that mapping from the mapping effect. Added or removed accounts may make that counterfactual unavailable; suggested mappings also withhold it. A missing bucket stays unknown, so moving every wages account elsewhere cannot manufacture a nil labour ratio.

Ratio differences are proportions: multiply by 100 to express percentage points. Changes in dataset or turnover-band basis are listed independently. The program does not invent an industry range, infer a business cause or decide whether a return is correct. Operational measures such as utilisation need their own input evidence and definitions.
