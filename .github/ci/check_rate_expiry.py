"""Warn before payday-super-checker's GIC table runs out.

Past the table's last quarter the engine and the MCP withhold notional earnings
and SG charge estimates (paydaysuper/rates.py, StaleGicError). The ATO publishes
the next quarter about four weeks ahead, and the figure still needs a table
update, a payday-super-checker release, an MCP pin bump and an MCP release. The
job runs weekly, so failing inside 28 days gives at least 21 days' notice:

    python .github/ci/check_rate_expiry.py [--today YYYY-MM-DD]

Standard library only; the table is read the way load_gic() reads it, as the
end of its last quarter.
"""

from __future__ import annotations

import argparse
import json
from datetime import date
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GIC_TABLE = ROOT / "packages" / "payday-super-checker" / "paydaysuper" / "data" / "gic_rates.json"
WARN_DAYS = 28


def last_known(path: Path = GIC_TABLE) -> date:
    quarters = json.loads(path.read_text(encoding="utf-8"))["quarters"]
    return max(date.fromisoformat(quarter["to"]) for quarter in quarters)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--today", type=date.fromisoformat, default=date.today())
    args = parser.parse_args(argv)
    end = last_known()
    left = (end - args.today).days
    where = GIC_TABLE.relative_to(ROOT).as_posix()
    if left < WARN_DAYS:
        print(
            f"::error file={where}::The GIC table ends {end.isoformat()}, {left} day(s) after "
            f"{args.today.isoformat()}. Add the next quarter from the ATO 'General interest "
            f"charge rates' page to {where}, then release payday-super-checker and the MCP."
        )
        return 1
    print(f"The GIC table in {where} ends {end.isoformat()}: {left} days left.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
