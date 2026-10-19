"""Review states for coded bank lines. Every entity, line and date is fabricated."""

from __future__ import annotations

from datetime import date, datetime
from decimal import Decimal

import pytest
from gstreview import BankLine, Registration, __version__, review_coding
from gstreview.coding import DEFAULT_GST_CODES, NOT_DECIDED, STATES

REGISTERED = Registration(date(2025, 7, 1))
NEVER = Registration(None)


def line(line_id: str, amount: str, day: date = date(2026, 8, 10),
         entity: str = "Alder Pty Ltd", **coding: object) -> BankLine:
    return BankLine(id=line_id, entity=entity, date=day, amount=Decimal(amount), **coding)  # type: ignore[arg-type]


def states(result: dict) -> dict[str, str]:
    return {row["id"]: row["state"] for row in result["lines"]}


def test_coding_states_without_conflicts() -> None:
    result = review_coding(
        [
            line("c", "-110.00", account="Office", gst_code="GST", confirmed=True),
            line("s1", "-55.00", account="Office", gst_code="FRE"),
            line("s2", "-20.00", suggested_account="Bank fees"),
            line("s3", "-20.00", suggested_gst_code="N-T"),
            line("n", "-9.00"),
        ],
        {"Alder Pty Ltd": REGISTERED},
    )
    assert states(result) == {"c": "coded", "s1": "suggested", "s2": "suggested",
                              "s3": "suggested", "n": "nothing_suggested"}
    assert result["counts"] == {"unknown_gst_code": 0, "gst_differs_from_registration": 0,
                                "between_entities": 0, "coded": 1, "suggested": 3,
                                "nothing_suggested": 1}
    assert list(result["counts"]) == list(STATES)


def test_result_contract() -> None:
    result = review_coding([line("a", "10.00")], {"Alder Pty Ltd": Registration(
        date(2025, 7, 1), date(2026, 6, 30))})
    assert result["ok"] is True
    assert result["engine"] == "gst-review"
    assert result["engine_version"] == __version__
    assert result["facts_used"]["registrations"] == {
        "Alder Pty Ltd": {"registered_from": "2025-07-01", "registered_to": "2026-06-30"}}
    assert result["facts_used"]["match_window_days"] == 3
    assert result["facts_used"]["gst_codes"] == DEFAULT_GST_CODES
    assert result["not_decided"] == NOT_DECIDED
    assert result["sources"][0].startswith("https://www.legislation.gov.au/C2004A00446/")
    row = result["lines"][0]
    assert row == {"id": "a", "entity": "Alder Pty Ltd", "date": "2026-08-10",
                   "amount": "10.00", "state": "nothing_suggested",
                   "reason": "No account or GST code proposed.", "counterpart": None,
                   "sections": []}


@pytest.mark.parametrize(
    ("registration", "day", "expected"),
    [
        (Registration(date(2026, 8, 1)), date(2026, 7, 31), "gst_differs_from_registration"),
        (Registration(date(2026, 8, 1)), date(2026, 8, 1), "coded"),
        (Registration(date(2025, 7, 1), date(2026, 8, 31)), date(2026, 8, 31), "coded"),
        (Registration(date(2025, 7, 1), date(2026, 8, 31)), date(2026, 9, 1),
         "gst_differs_from_registration"),
        (NEVER, date(2026, 8, 10), "gst_differs_from_registration"),
    ],
)
def test_registration_date_edges(registration: Registration, day: date, expected: str) -> None:
    result = review_coding(
        [line("a", "220.00", day, account="Sales", gst_code="GST on Income", confirmed=True)],
        {"Alder Pty Ltd": registration},
    )
    assert states(result) == {"a": expected}


def test_unregistered_entity_conflict_cites_both_sections() -> None:
    row = review_coding([line("a", "-33.00", suggested_gst_code="CAP")],
                        {"Alder Pty Ltd": NEVER})["lines"][0]
    assert row["state"] == "gst_differs_from_registration"
    assert row["sections"] == ["GST Act s 9-5(d)", "GST Act s 11-5(d)"]
    assert "not registered for GST on 2026-08-10" in row["reason"]


def test_gst_free_and_excluded_codes_do_not_conflict_when_unregistered() -> None:
    result = review_coding(
        [line("a", "-10.00", account="Bank fees", gst_code="FRE", confirmed=True),
         line("b", "50.00", account="Interest", gst_code="BAS Excluded", confirmed=True)],
        {"Alder Pty Ltd": NEVER},
    )
    assert states(result) == {"a": "coded", "b": "coded"}


def test_unregistered_supplier_on_a_claimed_purchase() -> None:
    result = review_coding(
        [
            line("p", "-110.00", account="Repairs", gst_code="GST", confirmed=True,
                 supplier_registered=False),
            line("q", "-110.00", day=date(2026, 9, 30), account="Repairs", gst_code="GST",
                 confirmed=True, supplier_registered=True),
            line("r", "-110.00", day=date(2026, 10, 30), account="Repairs", gst_code="GST",
                 confirmed=True),
            line("s", "110.00", day=date(2026, 11, 30), account="Sales", gst_code="GST",
                 confirmed=True, supplier_registered=False),
        ],
        {"Alder Pty Ltd": REGISTERED},
    )
    assert states(result) == {"p": "gst_differs_from_registration", "q": "coded",
                              "r": "coded", "s": "coded"}
    assert result["lines"][0]["sections"] == ["GST Act s 9-5(d)", "GST Act s 11-5(b)"]


def test_unknown_code_is_reported_before_other_checks() -> None:
    result = review_coding([line("a", "-1.00", gst_code="ZZZ", confirmed=True, account="X")],
                           {"Alder Pty Ltd": NEVER})
    assert states(result) == {"a": "unknown_gst_code"}
    assert "'ZZZ'" in result["lines"][0]["reason"]


def test_caller_code_table_replaces_the_default() -> None:
    result = review_coding(
        [line("a", "-11.00", gst_code="G11", account="Supplies", confirmed=True),
         line("b", "-11.00", day=date(2026, 9, 1), gst_code="GST", account="X",
              confirmed=True)],
        {"Alder Pty Ltd": NEVER},
        gst_codes={"G11": "taxable"},
    )
    assert states(result) == {"a": "gst_differs_from_registration", "b": "unknown_gst_code"}


def test_between_entities_pairs_opposite_amounts_one_to_one() -> None:
    regs = {"Alder Pty Ltd": REGISTERED, "Birch Trust": NEVER}
    result = review_coding(
        [
            line("out1", "-5000.00", date(2026, 8, 10)),
            line("out2", "-5000.00", date(2026, 8, 11)),
            line("in1", "5000.00", date(2026, 8, 12), entity="Birch Trust"),
            line("same", "5000.00", date(2026, 8, 10)),
        ],
        regs,
    )
    assert states(result) == {"out1": "between_entities", "out2": "nothing_suggested",
                              "in1": "between_entities", "same": "nothing_suggested"}
    rows = {row["id"]: row for row in result["lines"]}
    assert rows["out1"]["counterpart"] == "in1"
    assert rows["in1"]["counterpart"] == "out1"
    assert "'in1'" in rows["out1"]["reason"]


def test_between_entities_prefers_the_nearest_date() -> None:
    result = review_coding(
        [
            line("in", "900.00", date(2026, 8, 10), entity="Birch Trust"),
            line("far", "-900.00", date(2026, 8, 7)),
            line("near", "-900.00", date(2026, 8, 9)),
        ],
        {"Alder Pty Ltd": REGISTERED, "Birch Trust": NEVER},
    )
    assert states(result) == {"in": "between_entities", "far": "nothing_suggested",
                              "near": "between_entities"}


@pytest.mark.parametrize(("gap", "window", "expected"),
                         [(3, 3, "between_entities"), (4, 3, "nothing_suggested"),
                          (0, 0, "between_entities"), (1, 0, "nothing_suggested")])
def test_match_window_boundary(gap: int, window: int, expected: str) -> None:
    result = review_coding(
        [line("a", "-70.00", date(2026, 8, 10)),
         line("b", "70.00", date(2026, 8, 10 + gap), entity="Birch Trust")],
        {"Alder Pty Ltd": REGISTERED, "Birch Trust": REGISTERED},
        match_window_days=window,
    )
    assert states(result)["a"] == expected


def test_zero_amounts_are_never_paired() -> None:
    result = review_coding(
        [line("a", "0.00"), line("b", "0.00", entity="Birch Trust")],
        {"Alder Pty Ltd": REGISTERED, "Birch Trust": REGISTERED},
    )
    assert states(result) == {"a": "nothing_suggested", "b": "nothing_suggested"}


def test_registration_conflict_outranks_between_entities() -> None:
    result = review_coding(
        [line("a", "-70.00", gst_code="GST", account="X", confirmed=True),
         line("b", "70.00", entity="Birch Trust")],
        {"Alder Pty Ltd": NEVER, "Birch Trust": REGISTERED},
    )
    assert states(result) == {"a": "gst_differs_from_registration", "b": "between_entities"}


@pytest.mark.parametrize(
    ("lines", "registrations", "kwargs", "message"),
    [
        ([], {}, {"match_window_days": -1}, "match_window_days"),
        ([], {}, {"match_window_days": 32}, "match_window_days"),
        ([], {}, {"match_window_days": True}, "match_window_days"),
        ([], {}, {"gst_codes": {"X": "zero_rated"}}, "Unknown GST treatments"),
        ([], {"A": Registration(datetime(2026, 1, 1))}, {}, "must be dates"),  # type: ignore[arg-type]
        ([], {"A": Registration(date(2026, 1, 1), "2026-02-01")}, {}, "must be dates"),  # type: ignore[arg-type]
        ([], {"A": Registration(None, date(2026, 1, 1))}, {}, "ends before"),
        ([], {"A": Registration(date(2026, 2, 1), date(2026, 1, 1))}, {}, "ends before"),
        ([line("", "1.00")], {"Alder Pty Ltd": NEVER}, {}, "unique"),
        ([line("a", "1.00"), line("a", "2.00")], {"Alder Pty Ltd": NEVER}, {}, "unique"),
        ([line("a", "1.00")], {}, {}, "No registration facts"),
        ([BankLine("a", "Alder Pty Ltd", "2026-08-10", Decimal(1))],  # type: ignore[arg-type]
         {"Alder Pty Ltd": NEVER}, {}, "needs a date"),
        ([BankLine("a", "Alder Pty Ltd", date(2026, 8, 10), 1)],  # type: ignore[arg-type]
         {"Alder Pty Ltd": NEVER}, {}, "finite Decimal"),
        ([line("a", "NaN")], {"Alder Pty Ltd": NEVER}, {}, "finite Decimal"),
        ([line("a", "1.001")], {"Alder Pty Ltd": NEVER}, {}, "finite Decimal"),
        ([line("a", "-1000000000000.01")], {"Alder Pty Ltd": NEVER}, {}, "finite Decimal"),
        ([line("a", "1.00", confirmed=1)], {"Alder Pty Ltd": NEVER}, {}, "booleans"),
        ([line("a", "1.00", supplier_registered=1)], {"Alder Pty Ltd": NEVER}, {}, "booleans"),
        ([line("a", "1.00", confirmed=True, account="X")], {"Alder Pty Ltd": NEVER}, {},
         "confirmed without"),
    ],
)
def test_invalid_input_is_refused(lines: list[BankLine], registrations: dict,
                                  kwargs: dict, message: str) -> None:
    with pytest.raises(ValueError, match=message):
        review_coding(lines, registrations, **kwargs)
