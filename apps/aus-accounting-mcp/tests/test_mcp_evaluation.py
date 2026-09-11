"""Verify the evaluation answer key through real MCP calls, without a model."""

import asyncio
import json
import sys
import tempfile
import xml.etree.ElementTree as ET
from copy import deepcopy
from pathlib import Path

import pytest
from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client

from aus_accounting_mcp.server import mcp
from evaluation import tool_selection

QUESTIONS = ET.parse(Path(__file__).resolve().parents[1] / "evaluation" / "questions.xml")
CASES = [
    (
        pair.attrib["id"],
        pair.findtext("answer"),
        sorted({tool.text for tool in pair.findall("tools/tool")}),
    )
    for pair in QUESTIONS.findall("qa_pair")
]


class _RecordingSession:
    """A client session that records which tool each question actually needed.

    The answer key says what a correct reply is. This records what reaching it
    costs in tool calls, so the tool selection published in questions.xml is held
    to the replay rather than being a second, unchecked description of it.
    """

    def __init__(self, session):
        self._session = session
        self.selected: list[str] = []
        self.calls: list[dict] = []

    async def call_tool(self, name, arguments):
        self.selected.append(name)
        self.calls.append({"name": name, "arguments": deepcopy(arguments)})
        return await self._session.call_tool(name, arguments)

    async def read_resource(self, uri):
        return await self._session.read_resource(uri)


async def _answer(session, case):
    if case.startswith("scope-"):
        response = await session.read_resource("aus-accounting://scope")
        scope = json.loads(response.contents[0].text)
        assert case.removeprefix("scope-") in scope["unsupported_calculations"]
        return "UNSUPPORTED"

    async def call(name, **arguments):
        result = await session.call_tool(name, arguments)
        assert not result.is_error, result.content
        assert result.structured_content is not None
        return result.structured_content

    def ratio(result, name):
        return next(row["status"] for row in result["ratios"] if row["ratio"] == name)

    if case in {"grouped-payday", "worksheet-gst", "library-reference", "payday-evidence-pack"}:
        reference = QUESTIONS.find(f"qa_pair[@id='{case}']/calls")
        results = [await call(c["name"], **c["arguments"])
                   for c in json.loads(reference.text)]
        if case == "grouped-payday":
            return results[0]["results"][1]["due"]
        if case == "worksheet-gst":
            return results[0]["amounts"]["gst"]
        if case == "payday-evidence-pack":
            return json.loads(results[0]["files"]["exceptions.json"])["exceptions"][0]["verdict"]
        return results[-1]["text"]

    if case == "catalogue-pages":
        first = await call("list_ato_benchmark_industries", search="shop", year="2023-24", limit=2)
        second = await call(
            "list_ato_benchmark_industries", search="shop", year=first["benchmark_year"],
            limit=2, offset=first["next_offset"],
        )
        return str(len({item["name"] for item in first["industries"] + second["industries"]}))

    if case in {"missing-income", "established-zero-income", "missing-rent"}:
        found = await call(
            "list_ato_benchmark_industries", search="baker", year="2023-24", limit=20
        )
        industry = next(
            item["name"] for item in found["industries"]
            if item["name"] == "Bakeries and hot bread shops"
        )
        figures = {
            "industry": industry, "year": "2023-24",
            "turnover": "850000.00", "cost_of_sales": "270000.00",
        }
        if case == "missing-rent":
            missing = await call("get_ato_benchmarks", **figures, other_income="0.00")
            supplied = await call(
                "get_ato_benchmarks", **figures, other_income="0.00", rent="40000.00"
            )
            assert ratio(supplied, "rent_to_turnover") != "not_supplied"
            return ratio(missing, "rent_to_turnover")
        missing = await call("get_ato_benchmarks", **figures)
        if case == "missing-income":
            return ratio(missing, "cost_of_sales_to_turnover")
        supplied = await call("get_ato_benchmarks", **figures, other_income="0.00")
        assert ratio(missing, "cost_of_sales_to_turnover") == "not_supplied"
        return ratio(supplied, "cost_of_sales_to_turnover")

    if case == "receipt-evidence":
        facts = {
            "qe_day": "2027-07-01", "sg_amount": "120.00",
            "remitted": "2027-07-02", "as_at": "2027-08-01",
        }
        missing = await call("calc_payday_super_deadline", **facts)
        supplied = await call("calc_payday_super_deadline", **facts, received="2027-07-02")
        assert supplied["result"]["verdict"] == "ON_TIME"
        return missing["result"]["verdict"]

    if case in {"unknown-rate", "audit-consistency"}:
        year = "2027-28" if case == "unknown-rate" else "2025-26"
        summary = await call("get_div7a_benchmark_rate", year_of_income=year)
        full = await call(
            "get_div7a_benchmark_rate", year_of_income=year, response_detail="full"
        )
        key = "verdict" if case == "unknown-rate" else "benchmark_rate"
        assert summary[key] == full[key]
        if case == "unknown-rate":
            assert summary["benchmark_rate"] is full["benchmark_rate"] is None
        else:
            assert full["statutory_trace"]
        return summary[key]

    if case in {"missing-loan-facts", "unsupported-scope"}:
        rate = await call("get_div7a_benchmark_rate", year_of_income="2025-26")
        assert rate["verdict"] == "KNOWN"
        if case == "missing-loan-facts":
            review = await call(
                "review_div7a_loan", year_of_income="2025-26",
                interest_rate_for_years_after_year_loan_made=rate["benchmark_rate"],
            )
            return review["gate"]["verdict"]
        refusal = await call(
            "refuse_div7a", borrower_name="Synthetic Borrower",
            lender_entity_name="Synthetic Lender", loan_principal="50000.00",
        )
        assert refusal["available"] is False
        return refusal["code"]

    assert case == "synthetic-boundary"
    fixtures = [
        await call("generate_synthetic_sbr_fixture", form_type=form,
                   entity_name="Synthetic Evaluation Pty Ltd", revenue_or_sales="110000.00")
        for form in ("BAS", "CTR")
    ]
    assert all(fixture["synthetic"] for fixture in fixtures)
    return str(all(fixture["not_a_lodgment"] for fixture in fixtures)).lower()


async def _evaluate(case, library_root):
    parameters = StdioServerParameters(
        command=sys.executable, args=["-m", "aus_accounting_mcp.cli"],
        env={"AUS_ACCOUNTING_LIBRARY_ROOT": library_root},
    )
    async with stdio_client(parameters) as (reader, writer):
        async with ClientSession(reader, writer) as session:
            await session.initialize()
            recorder = _RecordingSession(session)
            return await _answer(recorder, case), recorder.selected, recorder.calls


@pytest.mark.parametrize(
    "case,expected,tools", CASES, ids=[case for case, _, _ in CASES]
)
def test_evaluation_answer_is_reproducible(case, expected, tools):
    with tempfile.TemporaryDirectory() as library_root:
        (Path(library_root) / "example.md").write_text(
            "# Synthetic\nsynthetic credit example\n", encoding="utf-8"
        )
        answer, selected, calls = asyncio.run(_evaluate(case, library_root))

    assert answer == expected
    # questions.xml publishes the tools a correct answer needs, and the
    # tool-selection harness scores a model against that list. A list nobody
    # checks is a list that drifts, so the replay has to have called exactly it.
    assert sorted(set(selected)) == tools
    reference = QUESTIONS.find(f"qa_pair[@id='{case}']/calls")
    assert calls == json.loads(reference.text)


def test_every_published_tool_selection_names_a_registered_tool():
    registered = {tool.name for tool in asyncio.run(mcp.list_tools())}
    published = {tool for _, _, tools in CASES for tool in tools}

    assert published <= registered, published - registered
    # Every tool the server offers is exercised by at least one question, so a
    # new tool cannot ship with no question covering it.
    assert registered == published


def test_the_selection_harness_publishes_questions_without_their_answers(capsys):
    # The harness exists to measure whether a model picks the right tool, so the
    # prompt it prints must not carry the tool names or the answers. A leak here
    # would quietly turn the measurement into a lookup.
    assert tool_selection.main(["questions"]) == 0
    printed = capsys.readouterr().out.splitlines()

    # Exact, rather than a substring hunt for each answer: several answers are
    # short enough to occur inside a question by coincidence ("4" sits inside
    # "2023-24"), and a test that reads those as leaks fails for the wrong
    # reason. Each line has to be the question and nothing else.
    questions = tool_selection._questions()
    assert printed == [f"{case}: {text}" for case, text in questions.items()]
    assert set(questions) == {case for case, _, _ in CASES}

    body = "\n".join(printed)
    for case, _, tools in CASES:
        for name in tools:
            assert name not in body, f"{case} leaked its tool selection"


def test_the_selection_harness_scores_a_recorded_run(tmp_path, capsys):
    published = tool_selection._published()
    assert published == {case: tools for case, _, tools in CASES}

    perfect = tmp_path / "perfect.json"
    perfect.write_text(json.dumps(published), encoding="utf-8")
    assert tool_selection.main(["score", str(perfect)]) == 0
    assert f"{len(CASES)} of {len(CASES)} questions selected exactly" in capsys.readouterr().out

    wrong = tmp_path / "wrong.json"
    wrong.write_text(
        json.dumps(
            {
                "catalogue-pages": ["get_ato_benchmarks"],
                "unsupported-scope": [*published["unsupported-scope"], "review_div7a_loan"],
            }
        ),
        encoding="utf-8",
    )
    assert tool_selection.main(["score", str(wrong)]) == 0
    report = capsys.readouterr().out
    assert "missed list_ato_benchmark_industries" in report
    assert "also called review_div7a_loan" in report
    assert "NOT ANSWERED" in report
    assert f"0 of {len(CASES)} questions selected exactly" in report


def test_the_selection_harness_rejects_a_question_it_does_not_know(tmp_path):
    recorded = tmp_path / "unknown.json"
    recorded.write_text(json.dumps({"no-such-question": ["get_ato_benchmarks"]}), encoding="utf-8")

    assert tool_selection.main(["score", str(recorded)]) == 2


def test_the_selection_harness_context_is_the_whole_tool_definition():
    # A host hands a model the tool definition as it stands. Most of what
    # decides both the selection and the arguments lives inside the schemas, so
    # printing a summary of them would score a model against a server that is
    # not this one.
    context = asyncio.run(tool_selection._describe())
    tools = asyncio.run(mcp.list_tools())

    assert "# Server instructions" in context
    assert "# Preloaded scope resource" in context
    assert "aus-accounting://scope" in context
    assert "unsupported_calculations" in context
    for tool in tools:
        assert tool.name in context
        assert (tool.description or "").strip().splitlines()[0] in context
        # The schemas verbatim. Asserting the serialised block covers every
        # argument's type, description, constraint and default at once, and a
        # summary of any of them would fail here.
        assert json.dumps(tool.input_schema, indent=2, sort_keys=True) in context
        assert json.dumps(tool.output_schema, indent=2, sort_keys=True) in context
        for argument in tool.input_schema["properties"]:
            assert argument in context

    # Two distinctions a name-only summary drops, and both change the answer:
    # an omitted bucket is not an established zero, and amounts have a ceiling.
    assert "only for an established zero" in context
    assert "1000000000000.00" in context


UNKNOWN_RATE_RUN = {
    "unknown-rate": {
        "calls": [
            {"name": "get_div7a_benchmark_rate", "arguments": {"year_of_income": "2027-28"}},
            {"name": "get_div7a_benchmark_rate", "arguments": {
                "year_of_income": "2027-28", "response_detail": "full",
            }},
        ],
        "answer": "UNKNOWN",
    },
}


def test_recorded_calls_and_answer_are_scored_together(tmp_path, capsys):
    recorded = tmp_path / "run.json"
    recorded.write_text(json.dumps(UNKNOWN_RATE_RUN), encoding="utf-8")
    assert tool_selection.main(["score", str(recorded)]) == 0
    report = capsys.readouterr().out
    assert f"1 of {len(CASES)} recorded calls and answers match the reference" in report


@pytest.mark.parametrize("change", ["answer", "argument", "repetition", "order"])
def test_correct_tool_names_cannot_hide_wrong_calls_or_answer(change, tmp_path, capsys):
    run = deepcopy(UNKNOWN_RATE_RUN)
    record = run["unknown-rate"]
    if change == "answer":
        record["answer"] = "0.0837"
    elif change == "argument":
        record["calls"][0]["arguments"]["year_of_income"] = "2025-26"
    elif change == "repetition":
        record["calls"].append(deepcopy(record["calls"][0]))
    else:
        record["calls"].reverse()
    recorded = tmp_path / "run.json"
    recorded.write_text(json.dumps(run), encoding="utf-8")
    assert tool_selection.main(["score", str(recorded)]) == 0
    report = capsys.readouterr().out
    assert f"0 of {len(CASES)} recorded calls and answers match the reference" in report
    difference = "answer differs" if change == "answer" else "calls differ"
    assert difference in report


def test_invented_zero_fails_even_when_the_answer_and_tools_are_right(tmp_path, capsys):
    run = {"missing-income": {
        "calls": [
            {"name": "list_ato_benchmark_industries", "arguments": {
                "search": "baker", "year": "2023-24", "limit": 20,
            }},
            {"name": "get_ato_benchmarks", "arguments": {
                "industry": "Bakeries and hot bread shops", "year": "2023-24",
                "turnover": "850000.00", "cost_of_sales": "270000.00", "other_income": "0.00",
            }},
        ],
        "answer": "not_supplied",
    }}
    recorded = tmp_path / "run.json"
    recorded.write_text(json.dumps(run), encoding="utf-8")
    assert tool_selection.main(["score", str(recorded)]) == 0
    report = capsys.readouterr().out
    assert f"0 of {len(CASES)} recorded calls and answers match the reference" in report
    assert "calls differ" in report


def test_legacy_tool_names_do_not_claim_argument_or_answer_verification(tmp_path, capsys):
    recorded = tmp_path / "legacy.json"
    recorded.write_text(json.dumps({"unknown-rate": ["get_div7a_benchmark_rate"]}))
    assert tool_selection.main(["score", str(recorded)]) == 0
    assert "arguments and answer NOT EVALUATED" in capsys.readouterr().out


@pytest.mark.parametrize("bad", [
    [], {"unknown-rate": None}, {"unknown-rate": {"calls": [], "answer": 0}},
    {"unknown-rate": {"calls": [{"name": "get_div7a_benchmark_rate"}], "answer": "UNKNOWN"}},
    {"unknown-rate": [123]},
])
def test_malformed_recordings_return_an_input_error(bad, tmp_path, capsys):
    recorded = tmp_path / "bad.json"
    recorded.write_text(json.dumps(bad), encoding="utf-8")
    assert tool_selection.main(["score", str(recorded)]) == 2
    assert capsys.readouterr().err
