"""An ON_TIME verdict or candidate must say returned contributions were not assessed.

The pinned checker has no input for a contribution the fund returned, so a receipt
the fund later sent back still reads as received. All facts here are fabricated.
"""

from __future__ import annotations

import asyncio
import csv
import hashlib
import io
import json
from datetime import date
from pathlib import Path

import pytest
from jsonschema import Draft202012Validator, ValidationError
from paydaysuper import assess
from paydaysuper.calendar import load_calendar
from paydaysuper.evidence_pack import build_evidence_pack
from paydaysuper.rates import load_gic

from aus_accounting_mcp import resources
from aus_accounting_mcp.adapters import payday
from aus_accounting_mcp.server import mcp

ROOT = Path(__file__).resolve().parents[1]
SINGLE = "calc_payday_super_deadline"
GROUP = "review_payday_super_contributions"
PACK = "build_payday_super_evidence_pack"
AS_AT = "2026-09-30"
QUALIFICATION_KEY = "verdict_qualification"
BASE = dict(employee_id="SYN001", qe_day="2026-08-06", sg_amount="800.00")
FLAGS = dict(first_to_fund=False, out_of_cycle=False, db_interest=False)
# A full, timely receipt. The story the tools cannot hear: the fund returned it on 14 August.
ON_TIME = dict(received="2026-08-10", matched_amount="800.00")
NOT_ON_TIME = {
    "AT_RISK": dict(remitted="2026-08-14"),
    "LATE": dict(received="2026-09-20", matched_amount="800.00"),
    "UNPAID": dict(),
    # Past the calendar's coverage, with a partial receipt, so ON_TIME is not a candidate.
    "UNKNOWN": dict(qe_day="2031-07-01", received="2031-07-02", matched_amount="400.00"),
    "SKIPPED": dict(db_interest=True),
}
AS_ATS = {"UNKNOWN": "2032-01-01"}
# UNKNOWN rows listing ON_TIME in horizon_verdicts: facts, as_at, the horizon.
CANDIDATES = [
    (dict(received="2026-08-10"), AS_AT, ["UNPAID", "ON_TIME"]),
    (dict(qe_day="2031-07-01", received="2031-08-01", matched_amount="800.00"),
     "2032-01-01", ["LATE", "ON_TIME"]),
]
STANDARD_PACK_CAVEATS = 4
TOOLS = {tool.name: tool for tool in asyncio.run(mcp.list_tools())}


def _flat(text: str) -> str:
    return " ".join(text.split())


def _call(name: str, arguments: dict) -> dict:
    result = asyncio.run(mcp.call_tool(name, arguments))
    assert not result.is_error
    Draft202012Validator(TOOLS[name].output_schema).validate(result.structured_content)
    return result.structured_content


def _single(facts: dict, as_at: str = AS_AT) -> dict:
    return _call(SINGLE, {**BASE, **facts, "as_at": as_at})


def _rows(*facts: dict, as_at: str = AS_AT) -> dict:
    return {"contributions": [{**BASE, **FLAGS, **row} for row in facts], "as_at": as_at}


def _engine_files(*facts: dict, as_at: str = AS_AT) -> dict[str, str]:
    """The pack the checker renders alone, for comparison with the tool's."""
    arguments = _rows(*facts, as_at=as_at)
    lines = [
        payday._line(row=i, **payday.ContributionInput(**row).model_dump())
        for i, row in enumerate(arguments["contributions"], 1)
    ]
    gic = load_gic()
    results = assess(
        lines, load_calendar(), gic, date.fromisoformat(as_at),
        transition_allocation_confirmed=False,
    )
    return build_evidence_pack(
        results, as_at=date.fromisoformat(as_at), gic_provenance=gic.provenance(),
        rate_tables=(gic.source,) if gic.source else (),
    )


def _report_rows(report: str) -> list[dict[str, str]]:
    rows = csv.DictReader(io.StringIO(report.lstrip("﻿")))
    return [row for row in rows if row["row"] != "NOTE"]


def _assert_row_qualification(qualification: dict) -> None:
    assert qualification == {
        "code": "RETURNED_CONTRIBUTIONS_NOT_ASSESSED",
        "returned_contributions_assessed": False,
        "assumes_fund_kept_every_receipt": True,
    }


def _assert_pack_qualification(qualification: dict) -> None:
    _assert_row_qualification({k: v for k, v in qualification.items() if k != "statement"})
    _assert_limit_stated(qualification["statement"])


def _assert_limit_stated(text: str) -> None:
    assert "returned contributions were not assessed" in text
    assert "assumes the fund kept every receipt" in text
    assert "Check fund records before relying on it" in text


def _assert_caveat_present(caveats: list[str]) -> None:
    caveat = next(c for c in caveats if "returned contributions" in c)
    _assert_limit_stated(caveat)
    assert "aus-accounting-mcp" in caveat
    assert len(caveat) < 200


def test_single_on_time_carries_the_qualification_and_keeps_verdict_and_figures() -> None:
    result = _single(ON_TIME)["result"]
    assert result["verdict"] == "ON_TIME"
    _assert_row_qualification(result[QUALIFICATION_KEY])
    _assert_caveat_present(result["caveats"])
    assert result["days_late"] is None
    for figure in ("base_shortfall", "final_shortfall", "notional_earnings",
                   "experimental_sgc_low", "experimental_sgc_high", "uplift"):
        assert result[figure] is None


def test_group_qualifies_each_on_time_row_and_only_those() -> None:
    late = {**NOT_ON_TIME["LATE"], "employee_id": "SYN002"}
    payload = _call(GROUP, _rows(ON_TIME, late))
    on_time, other = payload["results"]
    assert (on_time["verdict"], other["verdict"]) == ("ON_TIME", "LATE")
    _assert_row_qualification(on_time[QUALIFICATION_KEY])
    _assert_caveat_present(on_time["caveats"])
    assert QUALIFICATION_KEY not in other
    assert "returned contributions" not in json.dumps(other)


@pytest.mark.parametrize("detail", ["full", "compact"])
def test_pack_envelope_qualifies_an_all_on_time_exit_zero(detail: str) -> None:
    arguments = {**_rows(ON_TIME), "response_detail": detail}
    result = asyncio.run(mcp.call_tool(PACK, arguments))
    payload = result.structured_content
    assert payload["review_exit_code"] == 0
    _assert_pack_qualification(payload[QUALIFICATION_KEY])
    _assert_caveat_present(payload["caveats"])
    # A host may read the text alone, in either mode.
    assert "returned contributions were not assessed" in result.content[0].text


def test_pack_files_carry_the_limit_because_they_are_saved_without_the_envelope() -> None:
    files = _call(PACK, _rows(ON_TIME))["files"]
    engine = _engine_files(ON_TIME)

    report_row = _report_rows(files["report.csv"])[0]
    assert report_row["verdict"] == "ON_TIME"
    _assert_limit_stated(report_row["caveats"])

    for name in ("practitioner-review.md", "decision-log.md"):
        _assert_limit_stated(files[name])
    review = files["practitioner-review.md"]
    assert review.index("returned contributions") < review.index("NO EXCEPTION INDICATORS")

    queue = json.loads(files["exceptions.json"])
    _assert_pack_qualification(queue[QUALIFICATION_KEY])
    assert queue["exceptions"] == []

    # The hashes must still bind the report the pack actually returns.
    digest = hashlib.sha256(files["report.csv"].encode("utf-8")).hexdigest()
    assert queue["report_sha256"] == digest
    assert digest in review
    assert digest in files["decision-log.md"]
    assert json.loads(engine["exceptions.json"]).keys() < queue.keys()


def test_pack_changes_only_the_caveat_column_of_the_report() -> None:
    late = {**NOT_ON_TIME["LATE"], "employee_id": "SYN002"}
    files = _call(PACK, _rows(ON_TIME, late))["files"]
    engine_rows = _report_rows(_engine_files(ON_TIME, late)["report.csv"])
    rows = _report_rows(files["report.csv"])
    assert rows[1] == engine_rows[1]
    assert {**rows[0], "caveats": ""} == {**engine_rows[0], "caveats": ""}


def test_a_mixed_pack_keeps_its_exit_code_and_queue_and_is_still_qualified() -> None:
    late = {**NOT_ON_TIME["LATE"], "employee_id": "SYN002"}
    payload = _call(PACK, _rows(ON_TIME, late))
    assert payload["review_exit_code"] == 2
    _assert_pack_qualification(payload[QUALIFICATION_KEY])
    queue = json.loads(payload["files"]["exceptions.json"])
    assert [row["verdict"] for row in queue["exceptions"]] == ["LATE"]
    _assert_pack_qualification(queue[QUALIFICATION_KEY])


@pytest.mark.parametrize("verdict", NOT_ON_TIME)
def test_non_on_time_results_are_unchanged(verdict: str) -> None:
    facts = NOT_ON_TIME[verdict]
    as_at = AS_ATS.get(verdict, AS_AT)

    for payload, result in (
        (single := _single(facts, as_at), single["result"]),
        (group := _call(GROUP, _rows(facts, as_at=as_at)), group["results"][0]),
    ):
        assert result["verdict"] == verdict
        assert "ON_TIME" not in (result["horizon_verdicts"] or ())
        assert QUALIFICATION_KEY not in result
        assert "returned contributions" not in json.dumps(payload)

    pack = _call(PACK, _rows(facts, as_at=as_at))
    assert pack["review_exit_code"] == 2
    assert QUALIFICATION_KEY not in pack
    assert len(pack["caveats"]) == STANDARD_PACK_CAVEATS
    assert pack["files"] == _engine_files(facts, as_at=as_at)


@pytest.mark.parametrize("facts,as_at,horizon", CANDIDATES)
def test_an_on_time_candidate_is_qualified_and_stays_unknown(facts, as_at, horizon) -> None:
    for result in (
        _single(facts, as_at)["result"],
        _call(GROUP, _rows(facts, as_at=as_at))["results"][0],
    ):
        assert result["verdict"] == "UNKNOWN"
        assert result["horizon_verdicts"] == horizon
        _assert_row_qualification(result[QUALIFICATION_KEY])
        _assert_caveat_present(result["caveats"])
        for figure in ("days_late", "base_shortfall", "final_shortfall", "notional_earnings",
                       "experimental_sgc_low", "experimental_sgc_high", "uplift"):
            assert result[figure] is None


@pytest.mark.parametrize("facts,as_at,horizon", CANDIDATES)
def test_a_pack_with_an_on_time_candidate_is_qualified_and_keeps_exit_and_figures(
    facts, as_at, horizon,
) -> None:
    payload = _call(PACK, _rows(facts, as_at=as_at))
    assert payload["review_exit_code"] == 2
    _assert_pack_qualification(payload[QUALIFICATION_KEY])
    _assert_limit_stated(next(c for c in payload["caveats"] if "returned contributions" in c))

    files = payload["files"]
    engine = _engine_files(facts, as_at=as_at)
    row, engine_row = _report_rows(files["report.csv"])[0], _report_rows(engine["report.csv"])[0]
    assert row["verdict"] == "UNKNOWN"
    assert row["unassessable_between"] == " or ".join(horizon)
    _assert_limit_stated(row["caveats"])
    assert {**row, "caveats": ""} == {**engine_row, "caveats": ""}

    queue = json.loads(files["exceptions.json"])
    _assert_pack_qualification(queue[QUALIFICATION_KEY])
    queued, engine_queued = queue["exceptions"], json.loads(engine["exceptions.json"])["exceptions"]
    assert [{**q, "caveats": ""} for q in queued] == [{**q, "caveats": ""} for q in engine_queued]
    for name in ("practitioner-review.md", "decision-log.md"):
        _assert_limit_stated(files[name])


def test_published_schema_refuses_a_bare_on_time_candidate() -> None:
    payload = _single(*CANDIDATES[0][:2])
    validator = Draft202012Validator(TOOLS[SINGLE].output_schema)
    validator.validate(payload)
    del payload["result"][QUALIFICATION_KEY]
    with pytest.raises(ValidationError):
        validator.validate(payload)


def test_published_schema_refuses_a_bare_on_time_row() -> None:
    payload = _single(ON_TIME)
    validator = Draft202012Validator(TOOLS[SINGLE].output_schema)
    validator.validate(payload)
    del payload["result"][QUALIFICATION_KEY]
    with pytest.raises(ValidationError):
        validator.validate(payload)

    group = _call(GROUP, _rows(ON_TIME))
    del group["results"][0][QUALIFICATION_KEY]
    with pytest.raises(ValidationError):
        Draft202012Validator(TOOLS[GROUP].output_schema).validate(group)


def test_published_schema_refuses_an_unqualified_all_on_time_pack() -> None:
    payload = _call(PACK, _rows(ON_TIME))
    del payload[QUALIFICATION_KEY]
    with pytest.raises(ValidationError):
        Draft202012Validator(TOOLS[PACK].output_schema).validate(payload)


def test_published_schema_accepts_an_unknown_row_with_no_on_time_candidate() -> None:
    payload = _single(NOT_ON_TIME["UNKNOWN"], AS_ATS["UNKNOWN"])
    assert QUALIFICATION_KEY not in payload["result"]
    Draft202012Validator(TOOLS[SINGLE].output_schema).validate(payload)


def test_published_schema_refuses_a_qualification_claiming_returns_were_assessed() -> None:
    payload = _single(ON_TIME)
    payload["result"][QUALIFICATION_KEY]["returned_contributions_assessed"] = True
    with pytest.raises(ValidationError):
        Draft202012Validator(TOOLS[SINGLE].output_schema).validate(payload)


@pytest.mark.parametrize("name", [SINGLE, GROUP, PACK])
def test_no_input_claims_to_assess_a_return(name: str) -> None:
    schema = TOOLS[name].input_schema
    rows = schema["$defs"]["ContributionInput"] if name != SINGLE else schema
    assert not [field for field in rows["properties"] if "return" in field]


@pytest.mark.parametrize("name", [SINGLE, GROUP, PACK])
def test_tool_descriptions_state_the_limit(name: str) -> None:
    description = _flat(TOOLS[name].description)
    assert QUALIFICATION_KEY in description
    assert "assumes the fund kept every receipt" in description


def test_the_limit_is_published_everywhere_a_reader_looks() -> None:
    assert "assumes the fund kept every receipt" in _flat(mcp.instructions)
    assert "assumes the fund kept every receipt" in resources.scope()["payday_limitations"]
    for name in ("README.md", "docs/REFERENCE.md", "DISCLAIMER.md", "docs/firm-assessment.md"):
        text = _flat((ROOT / name).read_text(encoding="utf-8"))
        assert "assumes the fund kept every receipt" in text, name
    assert "s 18C(2) item 4" in resources.scope()["payday_limitations"]
