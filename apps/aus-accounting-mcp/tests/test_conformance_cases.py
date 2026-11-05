# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

"""Run the language-neutral conformance cases through the MCP tool-call path."""

import asyncio
import copy
import csv
import hashlib
import json
import re
from collections import Counter
from datetime import date, timedelta
from decimal import ROUND_HALF_UP, Decimal, localcontext
from math import ceil
from pathlib import Path

import pytest

from aus_accounting_mcp import server
from repository_root import repository_root

CASES_FILE = Path(__file__).resolve().parents[1] / "conformance" / "cases.json"
CONTRACT = json.loads(CASES_FILE.read_text(encoding="utf-8"))
CASES = CONTRACT["cases"]
_MISSING = object()
BASES = {"authority", "self-pinned"}
CASE_KEYS = {
    "id", "tool", "basis", "source", "worked", "arguments", "expect", "expect_error_contains",
}
PAYG_SAMPLE = "payg_withholding_sample_2026_27.csv"
SAMPLE_TOOL = "calculate_tax_worksheet"
KEPT_PINNED = "Kept self-pinned because"
CITATION = re.compile(r"\bATO\b|\bs \d+")
# Public holidays for the whole of a State or Territory inside the worked payday
# counts: the ATO's examples name this one, NT Picnic Day.
WORKED_HOLIDAYS = {date(2026, 8, 3)}
# ATO, Division 7A - benchmark interest rate: the rate for the income year ended
# 30 June 2027, restated here so a change to the cases file alone fails.
ATO_BENCHMARK_2026_27 = Decimal("0.0877")
# The ATO's own due dates on Payment deadlines for Payday Super, restated likewise.
ATO_DUE_DATES = {
    "payday_ato_example_1_first_payment": "2026-08-07",
    "payday_ato_example_1_second_payment": "2026-08-11",
    "payday_ato_example_1_second_payment_a_day_late": "2026-08-11",
    "payday_ato_example_2_first_contribution": "2026-09-04",
    "payday_ato_example_2_next_payment": "2026-09-15",
    "payday_ato_example_3_out_of_cycle": "2026-12-21",
    "payday_ato_example_4_out_of_cycle": "2026-09-14",
}


def _resolve(document: object, pointer: str) -> object:
    """Resolve an RFC 6901 JSON Pointer, returning _MISSING for an absent path."""
    node = document
    for token in pointer.split("/")[1:]:
        key = token.replace("~1", "/").replace("~0", "~")
        if isinstance(node, list) and key.isdigit() and int(key) < len(node):
            node = node[int(key)]
        elif isinstance(node, dict) and key in node:
            node = node[key]
        else:
            return _MISSING
    return node


def _call(case: dict) -> object:
    result = asyncio.run(server.mcp.call_tool(case["tool"], case["arguments"]))
    assert not result.is_error
    return result.structured_content


@pytest.mark.parametrize("case", CASES, ids=[case["id"] for case in CASES])
def test_conformance_case(case: dict) -> None:
    if "expect_error_contains" in case:
        with pytest.raises(Exception, match=case["expect_error_contains"]):
            _call(case)
        return
    payload = _call(case)
    for pointer, expected in case["expect"].items():
        actual = _resolve(payload, pointer)
        assert actual is not _MISSING, f"{case['id']}: {pointer} is absent"
        assert actual == expected, f"{case['id']}: {pointer} is {actual!r}"


def _problems(contract: dict, tools: set[str]) -> list[str]:
    """Every way the contract breaks its format, so a malformed case cannot run silently."""
    problems = []
    if contract.get("schema_version") != 2:
        problems.append("schema_version must be 2")
    if set(contract.get("basis", {})) != BASES:
        problems.append(f"basis must describe {sorted(BASES)}")
    ids = [case.get("id") for case in contract["cases"]]
    if len(ids) != len(set(ids)):
        problems.append("case ids must be unique")
    for case in contract["cases"]:
        name = case.get("id", "<no id>")
        if unknown := set(case) - CASE_KEYS:
            problems.append(f"{name}: unknown keys {sorted(unknown)}")
        if case.get("tool") not in tools:
            problems.append(f"{name}: tool is not registered")
        if case.get("basis") not in BASES:
            problems.append(f"{name}: basis must be one of {sorted(BASES)}")
        if not isinstance(case.get("source"), str) or not case["source"].strip():
            problems.append(f"{name}: source must name where the expected values come from")
        if ("expect" in case) == ("expect_error_contains" in case):
            problems.append(f"{name}: needs exactly one of expect and expect_error_contains")
        if "expect" in case and not case["expect"]:
            problems.append(f"{name}: expect must name at least one result")
        if not all(pointer.startswith("/") for pointer in case.get("expect", {})):
            problems.append(f"{name}: every expect key must be a JSON Pointer")
        if case.get("basis") == "self-pinned" and KEPT_PINNED not in str(case.get("source")):
            problems.append(f"{name}: a self-pinned case must say why it stays self-pinned")
    return problems


def _business_days_after(start: date, count: int) -> date:
    """Count weekdays that are not listed holidays, as SGAA 1992 s 6(1) defines them."""
    day = start
    while count:
        day += timedelta(days=1)
        if day.weekday() < 5 and day not in WORKED_HOLIDAYS:
            count -= 1
    return day


def _worked_payday(case: dict) -> dict[str, object]:
    """The figures a payday case's facts settle: a verdict needs a whole receipt by as_at."""
    facts = case["arguments"]
    start = date.fromisoformat(facts.get("next_standard_qe_day", facts["qe_day"]))
    due = _business_days_after(start, 20 if facts.get("first_to_fund") else 7)
    as_at = date.fromisoformat(facts["as_at"])
    worked: dict[str, object] = {"/result/due": due.isoformat()}
    if "received" not in facts:
        if as_at > due:
            worked |= {"/result/verdict": "UNPAID", "/result/days_late": (as_at - due).days,
                       "/result/final_shortfall": facts["sg_amount"]}
        return worked
    received = date.fromisoformat(facts["received"])
    whole = Decimal(facts.get("matched_amount", "0")) >= Decimal(facts["sg_amount"])
    if received <= as_at and whole:
        if received <= due:
            worked["/result/verdict"] = "ON_TIME"
        else:
            worked |= {"/result/verdict": "LATE", "/result/days_late": (received - due).days}
    return worked


def _remaining_term(facts: dict) -> int:
    """s 109E(6): the term less the years from the year made to the year before this one."""
    years_between = int(facts["year_of_income"][:4]) - int(facts["year_loan_made"][:4]) - 1
    return ceil(Decimal(facts["maximum_term_years"]) - years_between)


def _term_problems(case: dict) -> list[str]:
    """A supplied remaining term the statute does not give for the case's own facts."""
    facts = case["arguments"]
    term = _remaining_term(facts)
    if str(term) == facts["remaining_term_years"]:
        return []
    return [f"{case['id']}: remaining_term_years is {facts['remaining_term_years']!r}, "
            f"s 109E(6) gives {term}"]


def _worked_repayment(case: dict) -> dict[str, object]:
    """s 109E(6), by the ATO's 5 steps, in exact decimals rounded to the cent."""
    facts = case["arguments"]
    balance = Decimal(facts["amalgamated_loan_unpaid_at_end_of_previous_year"])
    paid = Decimal(facts["payments_applied_during_the_year"])
    rate = ATO_BENCHMARK_2026_27
    with localcontext(prec=60):
        required = balance * rate / (1 - (1 / (1 + rate)) ** _remaining_term(facts))
    required = required.quantize(Decimal("0.01"), ROUND_HALF_UP)
    shortfall = max(required - paid, Decimal("0.00"))
    return {
        "/minimum_yearly_repayment/verdict": "MYR_SHORT" if shortfall else "MYR_MET",
        "/minimum_yearly_repayment/benchmark_rate": str(rate),
        "/minimum_yearly_repayment/myr_required": str(required),
        "/minimum_yearly_repayment/shortfall": str(shortfall),
    }


def _worked_rate(case: dict) -> dict[str, object]:
    assert case["arguments"]["year_of_income"] == "2026-27"
    return {"/verdict": "KNOWN", "/benchmark_rate": str(ATO_BENCHMARK_2026_27)}


WORKINGS = {
    "calc_payday_super_deadline": _worked_payday,
    "review_div7a_loan": _worked_repayment,
    "get_div7a_benchmark_rate": _worked_rate,
}


def _authority_problems(case: dict) -> list[str]:
    """Every way a hand-worked authority case departs from its citation and working."""
    name = case["id"]
    problems = []
    if not CITATION.search(case["source"]):
        problems.append(f"{name}: source must cite the ATO or a section")
    if not str(case.get("worked", "")).strip():
        problems.append(f"{name}: an authority case needs its figure worked by hand")
    if case["tool"] not in WORKINGS:
        return [*problems, f"{name}: no independent working for {case['tool']}"]
    worked = WORKINGS[case["tool"]](case)
    if case["tool"] == "review_div7a_loan":
        problems += _term_problems(case)
    published = ATO_DUE_DATES.get(name)
    if published is not None and worked["/result/due"] != published:
        problems.append(f"{name}: /result/due is worked as {worked['/result/due']!r}, "
                        f"not the ATO's {published!r}")
    for pointer in sorted(worked.keys() - case["expect"].keys()):
        problems.append(f"{name}: {pointer} is worked but not expected")
    for pointer, expected in case["expect"].items():
        if pointer not in worked:
            problems.append(f"{name}: {pointer} has no independent working")
        elif worked[pointer] != expected:
            problems.append(f"{name}: {pointer} is {expected!r}, worked as {worked[pointer]!r}")
    return problems


TOOLS = {tool.name for tool in asyncio.run(server.mcp.list_tools())}


def test_cases_are_well_formed() -> None:
    assert _problems(CONTRACT, TOOLS) == []


@pytest.mark.parametrize(
    "change,problem",
    [
        (lambda case: case.pop("basis"), "basis must be one of"),
        (lambda case: case.update(basis="authoritative"), "basis must be one of"),
        (lambda case: case.pop("source"), "source must name"),
        (lambda case: case.update(source=" "), "source must name"),
        (lambda case: case.update(bases="authority"), "unknown keys"),
        (lambda case: case.update(source="No reason."), "must say why"),
    ],
    ids=["no-basis", "unknown-basis", "no-source", "blank-source", "misspelt-key", "no-reason"],
)
def test_a_case_without_a_valid_basis_or_source_is_rejected(change, problem) -> None:
    contract = copy.deepcopy(CONTRACT)
    self_pinned = next(case for case in contract["cases"] if case["basis"] == "self-pinned")
    change(self_pinned)

    assert any(problem in found for found in _problems(contract, TOOLS))


WORKED_CASES = [case for case in CASES
                if case["basis"] == "authority" and case["tool"] != SAMPLE_TOOL]


def _case(case_id: str) -> dict:
    return copy.deepcopy(next(case for case in CASES if case["id"] == case_id))


def test_fewer_cases_are_self_pinned_and_example_1_is_held() -> None:
    assert Counter(case["basis"] for case in CASES) == {"authority": 26, "self-pinned": 6}
    assert set(ATO_DUE_DATES) <= {case["id"] for case in WORKED_CASES}


@pytest.mark.parametrize("case", WORKED_CASES, ids=[case["id"] for case in WORKED_CASES])
def test_a_worked_authority_case_holds_without_the_server(case: dict) -> None:
    assert _authority_problems(case) == []


@pytest.mark.parametrize(
    "case_id,pointer,figure",
    [
        ("payday_ato_example_1_second_payment", "/result/due", "2026-08-10"),
        ("payday_ato_example_1_first_payment", "/result/due", "2026-08-06"),
        ("payday_ato_example_1_second_payment_a_day_late", "/result/days_late", 2),
        ("payday_nothing_received", "/result/final_shortfall", "119.99"),
        ("div7a_rate_known_year", "/benchmark_rate", "0.0837"),
        ("div7a_myr_seven_year_term", "/minimum_yearly_repayment/myr_required", "19715.98"),
        ("div7a_myr_seven_year_term", "/minimum_yearly_repayment/shortfall", "715.98"),
    ],
)
def test_a_changed_authority_figure_is_rejected(case_id, pointer, figure) -> None:
    case = _case(case_id)
    case["expect"][pointer] = figure

    assert any(pointer in found for found in _authority_problems(case))


@pytest.mark.parametrize(
    "case",
    [case for case in CASES if "remaining_term_years" in case.get("arguments", {})],
    ids=lambda case: case["id"],
)
def test_a_supplied_remaining_term_is_the_statutory_one(case: dict) -> None:
    assert _term_problems(case) == []


def test_a_term_changed_with_its_figure_is_rejected() -> None:
    case = _case("div7a_myr_one_year_term")
    case["arguments"]["maximum_term_years"] = "7"
    case["expect"]["/minimum_yearly_repayment/myr_required"] = "108770.00"

    assert any("s 109E(6) gives 7" in found for found in _authority_problems(case))
    case["arguments"]["remaining_term_years"] = "7"
    assert any("myr_required" in found for found in _authority_problems(case))


@pytest.mark.parametrize(
    "case_id,dropped",
    [
        ("div7a_rate_known_year", "/benchmark_rate"),
        ("div7a_myr_seven_year_term", "/minimum_yearly_repayment/shortfall"),
        ("payday_ato_example_1_second_payment", "/result/verdict"),
        ("payday_ato_example_1_second_payment_a_day_late", "/result/days_late"),
        ("payday_nothing_received", "/result/final_shortfall"),
    ],
)
def test_a_worked_figure_must_be_expected(case_id, dropped) -> None:
    case = _case(case_id)
    del case["expect"][dropped]

    assert any(f"{dropped} is worked but not expected" in found
               for found in _authority_problems(case))


def test_an_empty_expect_is_rejected() -> None:
    case = _case("div7a_rate_known_year")
    case["expect"] = {}

    assert len(_authority_problems(case)) == 2
    assert _problems({**CONTRACT, "cases": [case]}, TOOLS) == [
        "div7a_rate_known_year: expect must name at least one result"
    ]


@pytest.mark.parametrize(
    "change",
    [
        {"matched_amount": "0.01"},
        {"matched_amount": "119.99"},
        {"received": "2026-08-11", "as_at": "2026-08-10"},
    ],
    ids=["a_cent", "a_cent_short", "after_as_at"],
)
def test_a_receipt_that_does_not_settle_the_line_has_no_verdict(change) -> None:
    case = _case("payday_ato_example_1_second_payment")
    case["arguments"] |= change

    assert "/result/verdict has no independent working" in "\n".join(_authority_problems(case))


def test_an_authority_case_needs_a_citation_and_a_working() -> None:
    case = _case("payday_nothing_received")
    case["source"] = "A book."
    del case["worked"]
    case["expect"]["/result/caveats"] = []

    found = "\n".join(_authority_problems(case))
    assert "cite" in found and "worked by hand" in found and "no independent working" in found
    case["tool"] = "get_gst_rate"
    assert "no independent working for" in "\n".join(_authority_problems(case))


def test_authority_cases_reproduce_the_ato_sample() -> None:
    """An authority case's expected value is the ATO sample's, not this server's.

    The sample is the ATO's Schedule 1 withholding amounts data published on 17 June
    2026, held by australian-tax-calculators as its test oracle. Each authority case
    must name it and expect exactly the amount it gives for the case's period,
    earnings and scale, so a case cannot drift to whatever the engine returns.
    """
    sample = (repository_root() / "packages" / "australian-tax-calculators" / "tests"
              / "data" / PAYG_SAMPLE)
    digest = hashlib.sha256(sample.read_bytes()).hexdigest()
    with sample.open(encoding="utf-8", newline="") as handle:
        rows = {(row["period"], row["earnings"]): row for row in csv.DictReader(handle)}
    authority = [case for case in CASES
                 if case["basis"] == "authority" and case["tool"] == SAMPLE_TOOL]
    covered = set()
    for case in authority:
        facts = case["arguments"]["facts"]
        assert (facts["kind"], facts["year"]) == ("payg_withholding", "2026-27")
        assert PAYG_SAMPLE in case["source"] and "17 June 2026" in case["source"]
        # The held sample must still be the bytes the case was written from.
        assert f"(SHA-256 {digest})" in case["source"], case["id"]
        row = rows[(facts["pay_period"], facts["earnings"])]
        expected = f"{Decimal(row[f'scale{facts['scale']}']):.2f}"
        assert case["expect"] == {"/amounts/withholding": expected}, case["id"]
        covered.add((facts["pay_period"], facts["scale"]))
    # One case for each pay period and scale the sample publishes.
    assert covered == {(period, scale) for period in ("weekly", "fortnightly", "monthly")
                       for scale in (1, 2, 3, 5, 6)}


def test_the_resolver_reports_absent_paths() -> None:
    document = {"a": [{"b~/c": 1}]}
    assert _resolve(document, "/a/0/b~0~1c") == 1
    assert _resolve(document, "/a/1") is _MISSING
    assert _resolve(document, "/x") is _MISSING
