"""Warn before payday-super-checker's GIC table or lodgment-calendar's holidays run out.

Past the GIC table's last quarter the engine and the MCP withhold notional
earnings and SG charge estimates (src/rates.rs, the stale-GIC refusal). Past the
holiday table's COVERS_TO, lodgment-calendar's roll() cannot show a weekday is a
business day, so due dates there stay UNKNOWN in the engine and the MCP.
The ATO publishes the next GIC quarter about four weeks ahead, and either table
still needs an update, a component release, an MCP pin bump and an MCP release.
The job runs weekly, so failing inside 28 days gives at least 21 days' notice:

    python .github/ci/check_rate_expiry.py [--today YYYY-MM-DD] [--table gic|holidays]

Standard library only, and no component is imported. The GIC table is read the
way load_gic() reads it, as the end of its last quarter; the holiday table's end
is its COVERS_TO assignment, read from the source.
"""

from __future__ import annotations

import argparse
import ast
import json
from datetime import date
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GIC_TABLE = ROOT / "packages" / "payday-super-checker" / "data" / "gic_rates.json"
HOLIDAY_TABLE = ROOT / "packages" / "lodgment-calendar" / "lodgmentcalendar" / "holidays.py"
WARN_DAYS = 28


def last_known(path: Path = GIC_TABLE) -> date:
    quarters = json.loads(path.read_text(encoding="utf-8"))["quarters"]
    return max(date.fromisoformat(quarter["to"]) for quarter in quarters)


def holidays_end(path: Path = HOLIDAY_TABLE) -> date:
    for node in ast.parse(path.read_text(encoding="utf-8")).body:
        if (isinstance(node, ast.Assign) and len(node.targets) == 1
                and isinstance(node.targets[0], ast.Name) and node.targets[0].id == "COVERS_TO"
                and isinstance(node.value, ast.Call) and isinstance(node.value.func, ast.Name)
                and node.value.func.id == "date" and not node.value.keywords):
            return date(*(ast.literal_eval(arg) for arg in node.value.args))
    raise ValueError(f"{path}: no 'COVERS_TO = date(year, month, day)' assignment")


# Name: (table, its end, what it is, how to extend it).
TABLES = {
    "gic": (GIC_TABLE, last_known, "The GIC table",
            "Add the next quarter from the ATO 'General interest charge rates' page to "
            "{where}, then release payday-super-checker and the MCP."),
    "holidays": (HOLIDAY_TABLE, holidays_end, "The lodgment calendar's holiday table",
                 "Add the next year's whole-of-state holidays from the ATO 'Lodgment and "
                 "payment dates on weekends or public holidays' page to {where} and move "
                 "COVERS_TO, then release lodgment-calendar and the MCP."),
}


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--today", type=date.fromisoformat, default=date.today())
    parser.add_argument("--table", choices=sorted(TABLES), help="check one table (default: all)")
    args = parser.parse_args(argv)
    status = 0
    for name in [args.table] if args.table else TABLES:
        path, read_end, label, fix = TABLES[name]
        end = read_end()
        left = (end - args.today).days
        where = path.relative_to(ROOT).as_posix()
        if left < WARN_DAYS:
            print(
                f"::error file={where}::{label} ends {end.isoformat()}, {left} day(s) after "
                f"{args.today.isoformat()}. {fix.format(where=where)}"
            )
            status = 1
        else:
            print(f"{label} in {where} ends {end.isoformat()}: {left} days left.")
    return status


if __name__ == "__main__":
    raise SystemExit(main())
