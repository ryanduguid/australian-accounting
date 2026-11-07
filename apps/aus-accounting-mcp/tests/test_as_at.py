# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

"""`as_at` on the legislation search and read tools, against fabricated titles.

The status rules follow docs/scope.md in PaulGoodman. Every row is invented.
"""

import asyncio
import json
import tracemalloc
from pathlib import Path

import pytest
from jsonschema import Draft202012Validator
from mcp.server.mcpserver.exceptions import ToolError

import synthetic_corpus
from aus_accounting_mcp import corpus
from aus_accounting_mcp.errors import InputError
from aus_accounting_mcp.server import mcp

SCHEMAS = {tool.name: tool.output_schema for tool in asyncio.run(mcp.list_tools())}
SEARCH = "search_tax_legislation"
READ = "read_tax_legislation_section"
GOLDEN = Path(__file__).with_name("legislation_without_as_at.jsonl")
CURRENT, REPLACED, LATER = "C9999A10001", "C9999A10002", "C9999A10003"
COVERS = (
    "captured means the build holds the compilation for that date; it is not a finding "
    "that the text was the law then."
)
# (tool, arguments) for every result shape the tools give without as_at.
UNDATED_CALLS = [
    (SEARCH, {"query": "synthetic levy rate"}),
    (SEARCH, {"query": "not applied here", "in_force_only": False}),
    (SEARCH, {"query": "what is the levy exemption for a small entity"}),
    (SEARCH, {"query": "synthetic levy", "act": "levy act", "limit": 1}),
    (SEARCH, {"query": "synthetic levy", "act": "levy act", "limit": 1, "offset": 1}),
    (SEARCH, {"query": "zebra"}),
    (READ, {"row_id": "C9999A00001:0002:5-10"}),
    (READ, {"row_id": "C9999A00001:0002:5-10", "neighbours": 1}),
    (READ, {"row_id": "C9999A00002:0001:2"}),
    (READ, {"row_id": "C9999A00002:0002:3", "start": 12000}),
]


def call(name, **arguments):
    result = asyncio.run(mcp.call_tool(name, arguments))
    payload = result.structured_content
    Draft202012Validator(SCHEMAS[name]).validate(payload)
    return payload


def undated_results():
    """Each undated result as the exact JSON text a client would read."""
    return [
        json.dumps(call(name, **arguments), ensure_ascii=False)
        for name, arguments in UNDATED_CALLS
    ]


TEXTS = (
    "The synthetic levy applies to a dated receipt.",
    "The synthetic levy return must be lodged.",
    "A dated receipt must be kept.",
)


def title(register_id, name, start, *, superseded=None, retrieved="2099-01-01", texts=TEXTS):
    """Rows carrying a window, as the corpus builder writes them."""
    return [
        {
            **synthetic_corpus.section(
                register_id, f"{ordinal:04}", str(ordinal), text, act=name,
                current=superseded is None,
            ),
            "compilation_date": start,
            "superseded_from": superseded,
            "retrieved": retrieved,
        }
        for ordinal, text in enumerate(texts, 1)
    ]


def write_title(root, rows):
    synthetic_corpus.write(root / "markdown" / rows[0]["register_id"] / "sections.jsonl", rows)


@pytest.fixture
def dated(tmp_path, monkeypatch):
    """A current title, one replaced on 2098-10-01 and one compiled on 2098-12-01."""
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))
    write_title(tmp_path, title(CURRENT, "Synthetic Current Act 2098", "2098-07-01"))
    write_title(tmp_path, title(REPLACED, "Synthetic Replaced Act 2098", "2098-07-01",
                                superseded="2098-10-01"))
    write_title(tmp_path, title(LATER, "Synthetic Later Act 2098", "2098-12-01"))
    return tmp_path


def by_title(result):
    return {refusal["register_id"]: refusal for refusal in result["refusals"]}


def rows_returned(result):
    return sorted({hit["register_id"] for hit in result["matches"] + result["partial_hits"]})


@pytest.mark.parametrize(
    "on, status",
    [
        ("2098-06-30", "before_compilation"),
        ("2098-07-01", "captured"),
        ("2099-01-01", "captured"),
        ("2099-01-02", "after_retrieval"),
    ],
)
def test_a_current_title_has_a_status_either_side_of_each_edge(dated, on, status):
    result = call(SEARCH, query="synthetic levy", as_at=on, act="current")

    assert result["as_at_summary"]["titles"][status] >= 1
    if status == "captured":
        assert rows_returned(result) == [CURRENT]
        assert not result["refusals"]
    else:
        assert rows_returned(result) == []
        assert by_title(result)[CURRENT]["as_at"]["status"] == status


@pytest.mark.parametrize(
    "on, status",
    [
        ("2098-06-30", "before_compilation"),
        ("2098-07-01", "captured"),
        ("2098-09-30", "captured"),
        ("2098-10-01", "superseded"),
        ("2099-02-01", "superseded"),
    ],
)
def test_a_replaced_title_has_a_status_either_side_of_each_edge(dated, on, status):
    result = call(SEARCH, query="synthetic levy", as_at=on, act="replaced")

    if status == "captured":
        assert rows_returned(result) == [REPLACED]
    else:
        assert rows_returned(result) == []
        assert by_title(result)[REPLACED]["as_at"]["status"] == status


def test_a_date_before_every_compilation_returns_no_rows_and_says_so(dated):
    result = call(SEARCH, query="synthetic levy", as_at="2098-01-01")

    assert result["matches"] == result["partial_hits"] == []
    assert sorted(by_title(result)) == [CURRENT, REPLACED, LATER]
    assert {r["as_at"]["status"] for r in result["refusals"]} == {"before_compilation"}
    assert all(r["matching_rows"] == 2 for r in result["refusals"])
    summary = result["as_at_summary"]
    assert (summary["rows"], summary["withheld_rows"], summary["refused_titles"]) == (0, 6, 3)
    assert summary["titles"]["before_compilation"] == 3
    # A refusal is not the empty search's advice to reword the query.
    assert "No provision holds every word" not in result["notice"]
    assert "check matching_rows and partial_rows" in result["notice"]


def test_a_refusal_carries_the_status_window_and_next_step(dated):
    refusal = by_title(call(SEARCH, query="synthetic levy", as_at="2098-11-01"))[REPLACED]

    assert refusal == {
        "refused": True,
        "register_id": REPLACED,
        "act": "Synthetic Replaced Act 2098",
        "matching_rows": 2,
        "partial_rows": 0,
        "as_at": {
            "date": "2098-11-01",
            "status": "superseded",
            "compilation_date": "2098-07-01",
            "superseded_from": "2098-10-01",
            "retrieved": "2099-01-01",
            "covers": COVERS,
            "next_step": "a later version started; check the Register and application "
                         "provisions",
        },
    }


@pytest.mark.parametrize(
    "on, register_id, next_step",
    [
        ("2098-11-01", LATER, "read the Register's earlier compilations"),
        ("2099-02-01", CURRENT, "run check_current and read the Register"),
    ],
)
def test_each_refused_status_names_the_corpus_next_step(dated, on, register_id, next_step):
    refusal = by_title(call(SEARCH, query="synthetic levy", as_at=on))[register_id]

    assert refusal["as_at"]["next_step"] == next_step


def test_a_returned_row_reports_its_window_and_a_captured_row_no_next_step(dated):
    result = call(SEARCH, query="synthetic levy", as_at="2098-08-01")

    assert rows_returned(result) == [CURRENT, REPLACED]
    standing = result["matches"][0]["as_at"]
    assert standing["status"] == "captured"
    assert standing["next_step"] is None
    assert standing["covers"] == COVERS
    assert result["as_at_summary"]["rows"] == len(result["matches"])
    assert "not a finding that the text was the law" in result["notice"]
    assert "without a new compilation" in result["notice"]


def test_the_date_decides_for_a_replaced_title_not_in_force_only(dated):
    # in_force_only defaults to true, which would hide the one compilation held for this date.
    result = call(SEARCH, query="return lodged", as_at="2098-08-01", act="replaced")

    assert rows_returned(result) == [REPLACED]
    assert result["matches"][0]["version_is_current"] is False


def test_a_title_holding_only_partial_hits_is_refused_not_left_out(dated):
    # No row holds all 3 words, so the undated search answers with partial hits only.
    undated = call(SEARCH, query="levy return zebra")
    assert undated["partial_hits"] and not undated["matches"]

    result = call(SEARCH, query="levy return zebra", as_at="2098-01-01")

    assert result["partial_hits"] == []
    assert all(r["matching_rows"] == 0 and r["partial_rows"] == 1 for r in result["refusals"])
    assert result["as_at_summary"]["withheld_rows"] == 3


def test_a_named_title_is_refused_even_when_no_row_matches(dated):
    result = call(SEARCH, query="zebra", as_at="2098-01-01", act="later")

    assert by_title(result)[LATER]["matching_rows"] == 0
    assert by_title(result)[LATER]["as_at"]["status"] == "before_compilation"


@pytest.mark.parametrize(
    "on, refused",
    [
        ("2098-01-01", {CURRENT: "before_compilation", REPLACED: "before_compilation",
                        LATER: "before_compilation"}),
        ("2098-08-01", {LATER: "before_compilation"}),
        ("2098-11-01", {REPLACED: "superseded", LATER: "before_compilation"}),
        ("2099-02-01", {CURRENT: "after_retrieval", REPLACED: "superseded",
                        LATER: "after_retrieval"}),
    ],
)
def test_an_empty_answer_names_every_title_in_scope_not_captured(dated, on, refused):
    result = call(SEARCH, query="zebra", as_at=on)

    assert rows_returned(result) == []
    assert {key: item["as_at"]["status"] for key, item in by_title(result).items()} == refused
    assert all(item["matching_rows"] == 0 for item in result["refusals"])
    assert result["as_at_summary"]["withheld_rows"] == 0
    assert result["as_at_summary"]["refused_titles"] == len(refused)
    assert "No provision holds every word" not in result["notice"]


def test_an_empty_answer_is_clean_only_when_each_title_in_scope_is_captured(dated):
    result = call(SEARCH, query="zebra", as_at="2098-08-01", act="current")

    assert rows_returned(result) == []
    assert result["refusals"] == []
    assert "No provision holds every word" in result["notice"]


def test_a_title_outside_the_answer_is_not_refused_when_rows_came_back(tmp_path, monkeypatch):
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))
    write_title(tmp_path, title(CURRENT, "Synthetic Current Act 2098", "2098-07-01"))
    write_title(tmp_path, title(LATER, "Synthetic Later Act 2098", "2098-12-01",
                                texts=("A wholly separate rule.",)))

    result = call(SEARCH, query="synthetic levy", as_at="2098-08-01")

    assert rows_returned(result) == [CURRENT]
    assert result["refusals"] == []
    assert result["as_at_summary"]["titles"]["before_compilation"] == 1


def test_a_partial_only_title_is_refused_beside_returned_rows(tmp_path, monkeypatch):
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))
    write_title(tmp_path, title(CURRENT, "Synthetic Current Act 2098", "2098-07-01",
                                texts=("The levy return zebra rule.",)))
    write_title(tmp_path, title(LATER, "Synthetic Later Act 2098", "2098-12-01"))

    result = call(SEARCH, query="levy return zebra", as_at="2098-08-01")

    assert rows_returned(result) == [CURRENT]
    refusal = by_title(result)[LATER]
    assert (refusal["matching_rows"], refusal["partial_rows"]) == (0, 1)


def test_a_title_act_names_is_refused_beside_returned_rows(tmp_path, monkeypatch):
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))
    write_title(tmp_path, title(CURRENT, "Synthetic Current Act 2098", "2098-07-01"))
    write_title(tmp_path, title(LATER, "Synthetic Later Act 2098", "2098-12-01",
                                texts=("A wholly separate rule.",)))

    result = call(SEARCH, query="synthetic levy", as_at="2098-08-01", act="synthetic")

    assert rows_returned(result) == [CURRENT]
    assert by_title(result)[LATER]["matching_rows"] == 0


def test_every_page_of_a_dated_search_keeps_its_refusals(dated):
    first = call(SEARCH, query="synthetic levy", as_at="2098-11-01", limit=1)
    second = call(SEARCH, query="synthetic levy", as_at="2098-11-01", limit=1, offset=1)

    assert first["next_offset"] == 1
    assert first["refusals"] == second["refusals"]
    assert first["matches"] != second["matches"]


@pytest.mark.parametrize("query", ["synthetic levy", "levy receipt zebra"])
def test_an_exhausted_page_keeps_the_same_refusals(dated, query):
    write_title(dated, title("C9999A10004", "Synthetic Unrelated Act 2098", "2098-12-01",
                             texts=("A wholly separate rule.",)))
    first = call(SEARCH, query=query, as_at="2098-11-01", limit=1)
    exhausted = call(SEARCH, query=query, as_at="2098-11-01", limit=1, offset=100)

    assert first["matches"] or first["partial_hits"]
    assert not exhausted["matches"] and not exhausted["partial_hits"]
    assert exhausted["refusals"] == first["refusals"]


def test_an_exhausted_empty_search_still_refuses_uncaptured_titles(dated):
    result = call(SEARCH, query="zebra", as_at="2098-08-01", offset=100)

    assert set(by_title(result)) == {LATER}


def test_a_long_refusal_list_is_cut_with_every_title_counted(tmp_path, monkeypatch):
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))
    for number in range(25):
        write_title(tmp_path, title(f"C9999B{number:05}", f"Synthetic Act {number}", "2098-07-01"))
    write_title(tmp_path, title("C9999Z00001", "Income Tax Assessment Act 1997", "2098-07-01"))

    result = call(SEARCH, query="synthetic levy", as_at="2098-01-01")

    assert len(result["refusals"]) == 20
    assert result["refusals"][0]["act"] == "Income Tax Assessment Act 1997"
    summary = result["as_at_summary"]
    assert (summary["refused_titles"], summary["withheld_rows"]) == (26, 52)
    assert "refusals lists 20 of 26" in result["notice"]
    # Every refused title is named once, in the list or under its status past the list.
    named = [item["register_id"] for item in result["refusals"]]
    named += summary["unlisted_titles"]["before_compilation"]
    assert len(named) == len(set(named)) == 26


def test_an_unlisted_refusal_can_be_read_using_its_register_id(tmp_path, monkeypatch):
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))
    for number in range(21):
        write_title(tmp_path, title(f"C9999B{number:05}", f"Synthetic Title {number}",
                                     "2098-07-01"))
    result = call(SEARCH, query="synthetic levy", as_at="2098-01-01")
    register_id = result["as_at_summary"]["unlisted_titles"]["before_compilation"][0]

    lookup = call(SEARCH, query="synthetic levy", as_at="2098-01-01", act=register_id)

    assert set(by_title(lookup)) == {register_id}
    refusal = by_title(lookup)[register_id]
    assert refusal["as_at"]["compilation_date"] == "2098-07-01"
    assert refusal["as_at"]["next_step"] == "read the Register's earlier compilations"


def test_withheld_counts_stream_without_retaining_every_result(tmp_path, monkeypatch):
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))
    write_title(tmp_path, title(CURRENT, "Synthetic Act 2098", "2098-07-01",
                                 texts=("The synthetic levy applies." for _ in range(10000))))
    tracemalloc.start()
    try:
        result = corpus.search_sections("synthetic levy", 5, 0, as_at="2098-01-01")
        _, peak = tracemalloc.get_traced_memory()
    finally:
        tracemalloc.stop()

    assert result["as_at_summary"]["withheld_rows"] == 10000
    assert peak < 1_000_000, f"counting 10000 withheld rows retained {peak} bytes"


def test_dated_search_does_not_retain_each_titles_first_text(tmp_path, monkeypatch):
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))
    for number in range(32):
        write_title(tmp_path, title(f"C9999B{number:05}", f"Synthetic Title {number}",
                                     "2098-07-01", texts=["synthetic levy " + "x" * 100000]))
    tracemalloc.start()
    try:
        result = corpus.search_sections("synthetic levy", 5, 0, as_at="2098-01-01")
        _, peak = tracemalloc.get_traced_memory()
    finally:
        tracemalloc.stop()

    assert result["as_at_summary"]["refused_titles"] == 32
    assert result["as_at_summary"]["withheld_rows"] == 32
    assert peak < 2_000_000, f"dated title metadata retained {peak} bytes"


@pytest.mark.parametrize("first_start", ["2098-12-01", "2098-07-01"])
@pytest.mark.parametrize("tool", [SEARCH, READ])
def test_different_windows_of_one_title_stop_search_and_read(
    tmp_path, monkeypatch, first_start, tool
):
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))
    rows = title(CURRENT, "Synthetic Act 2098", first_start)
    rows[1]["compilation_date"] = "2098-06-01"
    write_title(tmp_path, rows)
    arguments = {"query": "return lodged"} if tool == SEARCH else {"row_id": rows[1]["row_id"]}

    with pytest.raises(ToolError, match="disagree"):
        call(tool, as_at="2098-08-01", **arguments)


def test_blank_lines_do_not_hide_a_dated_title(dated):
    path = dated / "markdown" / CURRENT / "sections.jsonl"
    path.write_text("\n" + path.read_text(encoding="utf-8"), encoding="utf-8")

    result = call(SEARCH, query="synthetic levy", as_at="2098-08-01", act="current")

    assert rows_returned(result) == [CURRENT]


def test_a_title_changed_after_validation_cannot_return_uncaptured_text(dated, monkeypatch):
    validate = corpus._first_row

    def then_change(path, on):
        result = validate(path, on)
        if path.parent.name == CURRENT:
            write_title(dated, title(CURRENT, "Synthetic Current Act 2098", "2098-12-01"))
        return result

    monkeypatch.setattr(corpus, "_first_row", then_change)
    with pytest.raises(ToolError, match="disagree"):
        call(SEARCH, query="synthetic levy", as_at="2098-08-01")


def test_a_read_failure_while_counting_refusals_stops_the_search(dated, monkeypatch):
    def unreadable(*args):
        raise OSError("synthetic storage failure")

    monkeypatch.setattr(corpus, "_rows", unreadable)
    with pytest.raises(ToolError, match="Cannot read a UTF-8 JSONL index"):
        call(SEARCH, query="synthetic levy", as_at="2098-01-01")


def test_a_short_refusal_list_leaves_no_unlisted_titles(dated):
    result = call(SEARCH, query="synthetic levy", as_at="2098-01-01")

    assert result["as_at_summary"]["unlisted_titles"] == {}


def test_the_byte_bound_covers_both_halves_of_a_dated_search(dated, monkeypatch):
    size = sum(path.stat().st_size for path in dated.glob("markdown/*/sections.jsonl"))
    monkeypatch.setattr(corpus, "MAX_CORPUS_BYTES", size - 1)

    with pytest.raises(ToolError, match="Corpus exceeds"):
        asyncio.run(mcp.call_tool(SEARCH, {"query": "levy", "as_at": "2098-11-01"}))


def test_a_read_held_for_the_date_returns_text_status_and_window(dated):
    result = call(READ, row_id=f"{REPLACED}:0002:2", as_at="2098-08-01", neighbours=1)

    assert "return must be lodged" in result["section"]["text"]
    assert result["section"]["as_at"]["status"] == "captured"
    assert result["section"]["as_at"]["superseded_from"] == "2098-10-01"
    assert [near["as_at"]["status"] for near in result["before"] + result["after"]] == [
        "captured", "captured",
    ]
    assert "refusals" not in result
    assert "not a finding that the text was the law" in result["notice"]


@pytest.mark.parametrize(
    "on, status",
    [
        ("2098-06-30", "before_compilation"),
        ("2098-10-01", "superseded"),
    ],
)
def test_a_read_refused_for_the_date_keeps_the_citation_and_withholds_the_text(dated, on, status):
    result = call(READ, row_id=f"{REPLACED}:0002:2", as_at=on, neighbours=1)

    section = result["section"]
    assert section["text"] == ""
    assert section["total_chars"] == len("The synthetic levy return must be lodged.")
    assert section["row_id"] == f"{REPLACED}:0002:2"
    assert section["compilation_date"] == "2098-07-01"
    assert section["as_at"]["status"] == status
    assert section["caveats"][0].startswith("Text withheld")
    assert result["before"] == result["after"] == []
    assert result["next_start"] is None
    assert result["refusals"][0]["refused"] is True
    assert result["refusals"][0]["matching_rows"] == 3
    assert "check matching_rows and partial_rows" in result["notice"]


def test_a_read_after_retrieval_is_refused_too(dated):
    result = call(READ, row_id=f"{CURRENT}:0001:1", as_at="2099-01-02")

    assert result["section"]["as_at"]["status"] == "after_retrieval"
    assert result["section"]["text"] == ""


@pytest.mark.parametrize(
    "bad",
    [
        "20980801",  # Python's date parser accepts this; it is not YYYY-MM-DD.
        "2098-W31-1",
        "2098-02-30",
        "2098-8-1",
        "2098-08-01T00:00",
        " 2098-08-01",
        "",
        "yesterday",
    ],
)
@pytest.mark.parametrize(
    "tool, arguments",
    [(SEARCH, {"query": "synthetic levy"}), (READ, {"row_id": f"{CURRENT}:0001:1"})],
)
def test_a_malformed_date_is_an_input_error_never_coerced(dated, tool, arguments, bad):
    with pytest.raises(ToolError):
        asyncio.run(mcp.call_tool(tool, {**arguments, "as_at": bad}))


@pytest.mark.parametrize("bad", ["20980801", "2098-W31-1", "2098-02-30", ""])
def test_the_reader_refuses_a_date_the_schema_would_stop(dated, bad):
    # Python's date parser reads the first 2 as dates, and neither is YYYY-MM-DD.
    with pytest.raises(InputError, match="as_at must be a YYYY-MM-DD date"):
        corpus.search_sections("levy", 5, 0, as_at=bad)
    with pytest.raises(InputError, match="as_at must be a YYYY-MM-DD date"):
        corpus.read_section(f"{CURRENT}:0001:1", as_at=bad)


def test_a_date_that_is_not_text_is_an_input_error(dated):
    with pytest.raises(ToolError):
        asyncio.run(mcp.call_tool(SEARCH, {"query": "levy", "as_at": 20980801}))


@pytest.mark.parametrize(
    "change, named",
    [
        ({"retrieved": None}, "retrieved is not a"),
        ({"retrieved": ""}, "retrieved is not a"),
        ({"retrieved": "2098-06-30"}, "compilation_date is after retrieved"),
        ({"compilation_date": "July 2098"}, "compilation_date is not a"),
        ({"version_is_current": None}, "version_is_current must be"),
        ({"version_is_current": "yes"}, "version_is_current must be"),
        ({"version_is_current": True, "superseded_from": "2098-10-01"},
         "superseded_from must be null"),
        ({"version_is_current": True, "superseded_from": "not a date"},
         "superseded_from must be null"),
        ({"version_is_current": False, "superseded_from": None}, "superseded_from is not a"),
        ({"version_is_current": False, "superseded_from": "2098-07-01"}, "superseded_from must"),
        ({"version_is_current": False, "superseded_from": "2099-01-02"}, "superseded_from must"),
    ],
)
@pytest.mark.parametrize("tool", [SEARCH, READ])
def test_a_row_whose_window_cannot_be_read_stops_a_dated_call(tool, tmp_path, monkeypatch,
                                                              change, named):
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))
    rows = title(CURRENT, "Synthetic Current Act 2098", "2098-07-01")
    write_title(tmp_path, [{**row, **change} for row in rows])
    arguments = (
        {"query": "synthetic levy"} if tool == SEARCH else {"row_id": f"{CURRENT}:0001:1"}
    )

    with pytest.raises(ToolError, match=named):
        asyncio.run(mcp.call_tool(tool, {**arguments, "as_at": "2098-08-01"}))
    call(tool, **arguments)  # The same corpus still answers without a date.


def test_an_older_build_without_dates_is_refused_and_says_to_rebuild(tmp_path, monkeypatch):
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))
    synthetic_corpus.build(tmp_path)

    with pytest.raises(ToolError, match="rebuilds it"):
        asyncio.run(mcp.call_tool(SEARCH, {"query": "levy", "as_at": "2098-08-01"}))


def test_rows_of_one_title_that_disagree_are_refused_not_returned(tmp_path, monkeypatch):
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))
    rows = title(CURRENT, "Synthetic Current Act 2098", "2098-07-01")
    rows[1] = {**rows[1], "compilation_date": "2098-12-01"}
    write_title(tmp_path, rows)

    with pytest.raises(ToolError, match="disagree"):
        asyncio.run(mcp.call_tool(SEARCH, {"query": "synthetic levy", "as_at": "2098-08-01"}))
    with pytest.raises(ToolError, match="disagree"):
        asyncio.run(mcp.call_tool(READ, {"row_id": f"{CURRENT}:0001:1", "as_at": "2098-08-01",
                                         "neighbours": 1}))


def test_a_title_index_that_cannot_be_read_stops_a_dated_search(tmp_path, monkeypatch):
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))
    index = tmp_path / "markdown" / CURRENT / "sections.jsonl"
    index.parent.mkdir(parents=True)
    index.write_bytes(b"\xff\n")

    with pytest.raises(ToolError, match="Cannot read a UTF-8"):
        asyncio.run(mcp.call_tool(SEARCH, {"query": "levy", "as_at": "2098-08-01"}))


def test_results_without_as_at_keep_every_byte(tmp_path, monkeypatch):
    # legislation_without_as_at.jsonl was written by the server before as_at existed.
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))
    synthetic_corpus.build(tmp_path)

    assert undated_results() == GOLDEN.read_text(encoding="utf-8").splitlines()
