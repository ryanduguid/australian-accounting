# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: FSL-1.1-ALv2

"""Both payday review tools report the cents the server's own evidence pack writes."""

from __future__ import annotations

import asyncio
import csv
import io
import re

from aus_accounting_mcp.server import calc_payday_super_deadline, mcp

# An unpaid row whose unrounded high estimate was 194.4942608030412703619145637.
# Rounding that sum gives 194.49, while report.csv rounds each component first
# and writes 194.50.
ROW = {"qe_day": "2026-09-10", "sg_amount": "120.00"}
GROUP_ROW = {**ROW, "employee_id": "SYN001", "first_to_fund": False,
             "out_of_cycle": False, "db_interest": False}
AS_AT = "2026-11-01"
CENTS = re.compile(r"\d+\.\d{2}")


def _call(tool: str) -> dict:
    arguments = {"contributions": [GROUP_ROW], "as_at": AS_AT}
    return asyncio.run(mcp.call_tool(tool, arguments)).structured_content


def test_both_review_tools_match_the_evidence_pack_to_the_cent() -> None:
    files = _call("build_payday_super_evidence_pack")["files"]
    report = next(csv.DictReader(io.StringIO(files["report.csv"].lstrip("﻿"))))
    assert (report["notional_earnings"], report["sgc_estimate_high"]) == ("1.56", "194.50")

    single = calc_payday_super_deadline(**ROW, as_at=AS_AT)["result"]
    group = _call("review_payday_super_contributions")["results"][0]
    for result in (single, group):
        assert result["notional_earnings"] == report["notional_earnings"]
        assert result["experimental_sgc_low"] == report["sgc_estimate_low"]
        assert result["experimental_sgc_high"] == report["sgc_estimate_high"]
        assert result["uplift"]["clean_history"]["vds_within_30d"] == report["uplift_best_case"]
        assert result["uplift"]["prior_history"]["no_vds"] == report["uplift_worst_case"]
        money = [result[key] for key in (
            "notional_earnings", "experimental_sgc_low", "experimental_sgc_high")]
        money += [value for scenario in result["uplift"].values() for value in scenario.values()]
        assert all(CENTS.fullmatch(value) for value in money), money


def test_sgc_estimates_say_what_they_exclude() -> None:
    # The CLI report says its estimates exclude choice loading, the maximum
    # contributions base and post-assessment penalties; an agent reading only the
    # tool schema must get the same limit.
    tools = {tool.name: tool.output_schema for tool in asyncio.run(mcp.list_tools())}
    exclusions = "exclude choice loading, the maximum contributions base and post-assessment"
    for schema in (tools["calc_payday_super_deadline"], tools["review_payday_super_contributions"]):
        text = str(schema)
        assert text.count(exclusions) >= 2, schema
