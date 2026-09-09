"""Run with an installed distribution's Python, using -I, outside the source tree."""

from decimal import Decimal
from pathlib import Path

from austaxcalc import calculations

assert "site-packages" in Path(calculations.__file__).parts
result = calculations.fbt(Decimal("16500"), Decimal("6000"), 2026, True)
assert result["amounts"]["fbt_estimate"] == "21452.68"
assert result["engine_version"] == "0.1.1"
assert result["sources"] and result["warnings"]
try:
    calculations.quarterly_sg(Decimal("1000"), Decimal("0"), "2026-27", 1, True)
except ValueError:
    pass
else:
    raise AssertionError("Unsupported post-June 2026 SG calculation was accepted")
print("Installed calculator distribution: numerical result, provenance and refusal pass.")
