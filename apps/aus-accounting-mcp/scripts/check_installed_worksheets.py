# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

"""Verify installed facade/engine wheels, without editable workspace sources."""

from __future__ import annotations

import asyncio
import hashlib
import importlib.metadata
import json
from pathlib import Path

from austaxcalc import calculations

from aus_accounting_mcp import reviewed_rates, server

ENGINES = (
    "ato-benchmark-compare", "payday-super-checker",
    "div7a-loan-review", "australian-tax-calculators",
)
NEW_KINDS = {"study_loan_withholding", "medicare_levy", "low_income_tax_offset", "division_293"}
REVIEWED_RECORDS = Path(__file__).resolve().parents[3] / "packages" / "au-tax-rates-data" / "data"


def _digests(folder: Path) -> dict[str, str]:
    return {
        path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in folder.glob("*.json")
    }


def check_records(installed: Path, reviewed: Path) -> int:
    """How many records the wheel holds, refusing any difference from the reviewed dataset."""
    held, wanted = _digests(installed), _digests(reviewed)
    if not wanted or held != wanted:
        names = held.keys() | wanted.keys()
        differ = sorted(name for name in names if held.get(name) != wanted.get(name))
        raise ValueError(
            f"the wheel's rate records differ from {reviewed}: {len(wanted)} reviewed, "
            f"{len(held)} installed, differing {differ}"
        )
    return len(held)


async def check() -> None:
    for module in (server, calculations):
        if "site-packages" not in Path(module.__file__).resolve().parts:
            raise ValueError("run with non-editable installed wheels outside the source tree")
    requirements = set(importlib.metadata.requires("aus-accounting-mcp") or [])
    versions = {name: importlib.metadata.version(name) for name in ENGINES}
    for name, version in versions.items():
        if f"{name}=={version}" not in requirements:
            raise ValueError(f"installed {name} does not match the facade's exact pin")
    contents = list(await server.mcp.read_resource("aus-accounting://scope"))
    catalogue = json.loads(contents[0].content)["calculation_worksheets"]
    if len(catalogue) != 14 or not NEW_KINDS <= catalogue.keys():
        raise ValueError("candidate does not expose all fourteen worksheets")
    calls = 0
    for kind, entry in catalogue.items():
        for period in entry["supported_periods"]:
            facts = {**entry["example"]["facts"], **period["arguments"]}
            result = await server.mcp.call_tool("calculate_tax_worksheet", {"facts": facts})
            if result.is_error or not isinstance(result.structured_content, dict):
                raise ValueError(f"{kind} did not return structured calculation evidence")
            data = result.structured_content
            if data["calculation"] != kind or not data["warnings"]:
                raise ValueError(f"{kind} lost its calculation identity or warnings")
            if data["engine_version"] != versions["australian-tax-calculators"]:
                raise ValueError(f"{kind} reported another engine's version")
            calls += 1
    records = check_records(reviewed_rates.RECORDS, REVIEWED_RECORDS)
    rate = (await server.mcp.call_tool(
        "get_reviewed_rate", {"record_id": "concessional-cap-2026-27", "as_at": "2027-03-01"}
    )).structured_content
    if not rate or rate["status"] != "FOUND" or not rate["record"]["value"]:
        raise ValueError(f"the wheel returned no reviewed rate figure: {rate}")
    print(json.dumps({
        "facade": importlib.metadata.version("aus-accounting-mcp"),
        "engines": versions,
        "worksheet_kinds": len(catalogue),
        "supported_period_calls": calls,
        "reviewed_rate": rate["record"]["id"],
        "rate_records_equal_to_the_dataset": records,
        "scope": "Installed wheels; declared exact pins; synthetic examples, not tax validation.",
    }, indent=2))


if __name__ == "__main__":
    asyncio.run(check())
