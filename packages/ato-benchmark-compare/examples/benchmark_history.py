"""Compare periods using the existing benchmark engine and reviewed mappings."""
from __future__ import annotations

import argparse
import hashlib
import json
from decimal import Decimal
from pathlib import Path

from atobenchmark.dataset import load
from atobenchmark.mapping import MappingError, read_mapping, route
from atobenchmark.pnl import read
from atobenchmark.ratios import RatioError, compute
from atobenchmark.report import compare, to_evidenced_dict


def _compare(pnl_path, mapping, year, industry, flip_expense_signs=False):
    source = read(pnl_path, None)
    routed = route(source.rows, mapping, flip_expense_signs)
    figures = compute(routed.totals)
    data = load(year)
    result = compare(data, data.get(industry), figures, set(routed.supplied_buckets))
    return to_evidenced_dict(result, set(routed.supplied_buckets), unreviewed=routed.unreviewed)


def history(previous_pnl: Path, current_pnl: Path, previous_mapping: Path, current_mapping: Path,
            *, previous_period: str, current_period: str, industry: str,
            previous_year: str | None = None, current_year: str | None = None,
            previous_flip_expense_signs: bool = False,
            current_flip_expense_signs: bool = False) -> dict:
    if not previous_period.strip() or not current_period.strip() or previous_period == current_period:
        raise ValueError("Supply distinct, explicit period labels")
    paths = (previous_pnl, current_pnl, previous_mapping, current_mapping)
    hashes = [hashlib.sha256(path.read_bytes()).hexdigest() for path in paths]
    old_map, new_map = read_mapping(previous_mapping), read_mapping(current_mapping)
    previous = _compare(previous_pnl, old_map, previous_year, industry, previous_flip_expense_signs)
    current = _compare(current_pnl, new_map, current_year, industry, current_flip_expense_signs)
    mapping_changes = [{"account": (new_map.get(key) or old_map[key]).account,
                        "previous_bucket": old_map[key].bucket if key in old_map else None,
                        "current_bucket": new_map[key].bucket if key in new_map else None,
                        "previous_source": old_map[key].source if key in old_map else None,
                        "current_source": new_map[key].source if key in new_map else None}
                       for key in sorted(set(old_map) | set(new_map))
                       if key not in old_map or key not in new_map
                       or old_map[key].bucket != new_map[key].bucket
                       or old_map[key].source.strip().casefold() != new_map[key].source.strip().casefold()]
    try:
        constant = _compare(current_pnl, old_map, previous_year, industry, current_flip_expense_signs)
        limitation = None
    except (MappingError, RatioError) as exc:
        constant = None
        limitation = f"Current accounts cannot all use the previous mapping: {exc}"
    if previous["unreviewed_accounts"] or current["unreviewed_accounts"] or (constant and constant["unreviewed_accounts"]):
        constant = None
        limitation = "Suggested mappings remain; a performance/mapping split is withheld."
    old_ratios = {row["ratio"]: row for row in previous["ratios"]}
    same_ratios = {} if constant is None else {row["ratio"]: row for row in constant["ratios"]}
    bridge = []
    for row in current["ratios"]:
        key = row["ratio"]
        old, same = old_ratios.get(key, {}).get("value"), same_ratios.get(key, {}).get("value")
        now = row["value"]
        available = old is not None and same is not None and now is not None
        bridge.append({"ratio": key, "previous": old, "current": now,
                       "current_under_previous_mapping": same,
                       "movement_under_previous_mapping": str(Decimal(same) - Decimal(old)) if available else None,
                       "mapping_effect": str(Decimal(now) - Decimal(same)) if available else None,
                       "comparison": "DECOMPOSED" if available else "NOT_COMPARABLE"})
    basis_changes = [field for field in ("benchmark_year", "business_type", "turnover_basis", "turnover_band", "source")
                     if previous.get(field) != current.get(field)]
    if hashes != [hashlib.sha256(path.read_bytes()).hexdigest() for path in paths]:
        raise ValueError("An input changed during the comparison; rerun from stable files")
    return {"previous_period": previous_period, "current_period": current_period,
            "source_sha256": dict(zip(("previous_pnl", "current_pnl", "previous_mapping", "current_mapping"), hashes)),
            "previous": previous, "current": current, "mapping_changes": mapping_changes,
            "basis_changes": basis_changes, "ratio_bridge": bridge, "limitation": limitation,
            "scope": "Existing benchmark definitions and datasets only. Ratio differences are proportions, not percentage points. No cause inferred from ledger movement."}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("previous-pnl", "current-pnl", "previous-mapping", "current-mapping"):
        parser.add_argument("--" + name, type=Path, required=True)
    for name in ("previous-period", "current-period", "industry"):
        parser.add_argument("--" + name, required=True)
    parser.add_argument("--previous-year")
    parser.add_argument("--current-year")
    parser.add_argument("--previous-flip-expense-signs", action="store_true")
    parser.add_argument("--current-flip-expense-signs", action="store_true")
    args = parser.parse_args()
    print(json.dumps(history(**vars(args)), indent=2))
