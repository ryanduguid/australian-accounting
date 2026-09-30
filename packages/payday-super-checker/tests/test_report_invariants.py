"""Report invariants also apply when Python assertions are disabled."""

import json
import subprocess
import sys
from datetime import date
from decimal import Decimal
from pathlib import Path

import pytest
from paydaysuper.assess import LATE, Result
from paydaysuper.deadlines import ContribLine, Deadline
from paydaysuper.rates import load_rates
from paydaysuper.report import console_summary, rounded_figures
from paydaysuper.sgc import uplift_scenarios


def probe(case: str) -> dict:
    result = Result(
        ContribLine("FABRICATED", date(2026, 7, 9), Decimal("100"), row=2),
        Deadline(date(2026, 7, 20), "usual period"),
        LATE,
    )
    result.final_shortfall = Decimal("100")
    result.nec = Decimal("1")
    result.uplift = uplift_scenarios(result.final_shortfall, result.nec)
    result.sgc_low = Decimal("101")
    result.sgc_high = Decimal("161.60")
    if case == "unavailable":
        result.nec = result.uplift = result.sgc_low = result.sgc_high = None
    elif case != "complete":
        setattr(result, case, None)
    try:
        if case == "final_shortfall":
            value = console_summary(
                [result], date(2026, 8, 10), "report.csv", "fixture", load_rates()
            )
        else:
            value = rounded_figures(result)
        return {"value": value}
    except Exception as exc:
        return {"error": type(exc).__name__, "message": str(exc)}


@pytest.mark.parametrize("optimised", [False, True], ids=["normal", "optimised"])
@pytest.mark.parametrize(
    "case", ["final_shortfall", "nec", "uplift", "sgc_low", "sgc_high", "complete", "unavailable"]
)
def test_report_invariants(case: str, optimised: bool):
    if optimised:
        completed = subprocess.run(
            [sys.executable, "-O", str(Path(__file__).resolve()), case],
            capture_output=True,
            text=True,
            check=True,
        )
        actual = json.loads(completed.stdout)
    else:
        actual = json.loads(json.dumps(probe(case), default=str))
    if case == "final_shortfall":
        expected = {
            "error": "AssertionError",
            "message": "exposed result has no final shortfall, so its exposure figures are incomplete",
        }
    elif case in {"nec", "uplift", "sgc_low", "sgc_high"}:
        expected = {
            "error": "AssertionError",
            "message": "exposed result has a partial SG charge estimate",
        }
    elif case == "complete":
        expected = {
            "value": {
                "shortfall": "100.00",
                "nec": "1.00",
                "up_low": "0.00",
                "up_high": "60.60",
                "low": "101.00",
                "high": "161.60",
            }
        }
    else:
        expected = {
            "value": {
                "shortfall": "100.00",
                "nec": None,
                "up_low": None,
                "up_high": None,
                "low": None,
                "high": None,
            }
        }
    assert actual == expected


if __name__ == "__main__":
    print(json.dumps(probe(sys.argv[1]), default=str))
