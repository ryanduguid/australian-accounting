"""A model recording must preserve argument types and its answer-key identity."""
import hashlib
import json
import xml.etree.ElementTree as ET
from copy import deepcopy

import pytest

from evaluation import tool_selection


def reference(case):
    pair = ET.parse(tool_selection.QUESTIONS).find(f"qa_pair[@id='{case}']")
    return {case: {"calls": json.loads(pair.findtext("calls")),
                   "answer": pair.findtext("answer")}}


@pytest.mark.parametrize("change", ["boolean", "number"])
def test_decoded_argument_types_cannot_match_by_python_coercion(change, capsys):
    if change == "boolean":
        root = ET.parse(tool_selection.QUESTIONS)
        pair = next(pair for pair in root.getroot()
                    if "scope_confirmed" in (pair.findtext("calls") or ""))
        case = pair.attrib["id"]
        run = reference(case)
        run[case]["calls"][0]["arguments"]["facts"]["scope_confirmed"] = 1
    else:
        case = "catalogue-pages"
        run = reference(case)
        run[case]["calls"][0]["arguments"]["limit"] = 2.0
    assert tool_selection._score(run) == 0
    report = capsys.readouterr().out
    assert "calls differ (arguments, order or count)" in report
    assert "0 of" in report.split("recorded calls and answers")[0].splitlines()[-1]


@pytest.mark.parametrize("text", [
    '{"unknown-rate": [], "unknown-rate": []}',
    '{"unknown-rate": {"calls": [{"name":"get_div7a_benchmark_rate",'
    '"arguments":{"year_of_income":"2025-26","year_of_income":"2027-28"}}],'
    '"answer":"UNKNOWN"}}',
    '{"unknown-rate": {"calls": [{"name":"x","arguments":{"x": NaN}}],'
    '"answer":"UNKNOWN"}}',
    '{"unknown-rate": {"calls": [{"name":"x","arguments":{"x": Infinity}}],'
    '"answer":"UNKNOWN"}}',
    '{"unknown-rate": {"calls": [{"name":"x","arguments":{"x": -Infinity}}],'
    '"answer":"UNKNOWN"}}',
    '{"unknown-rate": {"calls": [{"name":"x","arguments":{"x": 1e999}}],'
    '"answer":"UNKNOWN"}}',
])
def test_ambiguous_or_nonfinite_recordings_are_input_errors(text, tmp_path, capsys):
    record = tmp_path / "ambiguous.json"
    record.write_text(text, encoding="utf-8")
    assert tool_selection.main(["score", str(record)]) == 2
    assert "invalid recording:" in capsys.readouterr().err


def test_a_recording_can_bind_the_reference_snapshot(tmp_path, capsys):
    record = tmp_path / "run.json"
    record.write_text(json.dumps(reference("unknown-rate")), encoding="utf-8")
    digest = hashlib.sha256(tool_selection.QUESTIONS.read_bytes()).hexdigest()
    assert tool_selection.main(["score", str(record), "--reference-sha256", digest]) == 0
    report = capsys.readouterr().out
    assert f"reference_sha256 {digest}" in report
    assert "1 of 33 recorded calls and answers match the reference." in report
    assert tool_selection.main(["score", str(record), "--reference-sha256", "0" * 64]) == 2
    assert "reference digest differs" in capsys.readouterr().err


@pytest.mark.parametrize("change", [
    "duplicate-id", "missing-calls", "missing-answer", "tool-set", "missing-question",
    "blank-question", "duplicate-question", "duplicate-calls", "duplicate-answer",
    "conflicting-answer", "duplicate-json", "nonfinite-json",
])
def test_invalid_reference_keys_cannot_produce_scores(change, tmp_path, monkeypatch, capsys):
    tree = ET.parse(tool_selection.QUESTIONS)
    root = tree.getroot()
    pair = root[0]
    if change == "duplicate-id":
        root.append(deepcopy(pair))
    elif change == "missing-calls":
        pair.remove(pair.find("calls"))
    elif change == "missing-answer":
        pair.remove(pair.find("answer"))
    elif change == "tool-set":
        pair.find("tools/tool").text = "get_div7a_benchmark_rate"
    elif change == "missing-question":
        pair.remove(pair.find("question"))
    elif change == "blank-question":
        pair.find("question").text = " \n "
    elif change.startswith("duplicate-") and change.removeprefix("duplicate-") in {
        "question", "calls", "answer",
    }:
        pair.append(deepcopy(pair.find(change.removeprefix("duplicate-"))))
    elif change == "conflicting-answer":
        ET.SubElement(pair, "answer").text = "Different answer"
    elif change == "duplicate-json":
        pair.find("calls").text = '[{"name":"x","arguments":{"x":1,"x":2}}]'
    else:
        pair.find("calls").text = '[{"name":"x","arguments":{"x":NaN}}]'
    path = tmp_path / "questions.xml"
    tree.write(path, encoding="utf-8")
    monkeypatch.setattr(tool_selection, "QUESTIONS", path)
    record = tmp_path / "run.json"
    record.write_text("{}", encoding="utf-8")
    assert tool_selection.main(["score", str(record)]) == 2
    output = capsys.readouterr()
    assert "invalid recording:" in output.err
    assert "recorded calls and answers" not in output.out


@pytest.mark.parametrize(("expected", "actual", "matches"), [
    ("1.0000000000000001", "1.0", False),
    ("1.0000000000000001", "1.0000000000000001", True),
    ("1.20", "1.2", True),
])
def test_finite_float_values_are_not_rounded_before_comparison(
    expected, actual, matches, tmp_path, monkeypatch, capsys,
):
    path = tmp_path / "questions.xml"
    root = ET.Element("questions")
    pair = ET.SubElement(root, "qa_pair", id="decimal")
    ET.SubElement(pair, "question").text = "Synthetic precision check"
    ET.SubElement(ET.SubElement(pair, "tools"), "tool").text = "x"
    ET.SubElement(pair, "calls").text = '[{"name":"x","arguments":{"value":'+expected+'}}]'
    ET.SubElement(pair, "answer").text = "OK"
    ET.ElementTree(root).write(path, encoding="utf-8")
    monkeypatch.setattr(tool_selection, "QUESTIONS", path)
    record = tmp_path / "run.json"
    record.write_text(
        '{"decimal":{"calls":[{"name":"x","arguments":{"value":'+actual+'}}],'
        '"answer":"OK"}}', encoding="utf-8",
    )
    assert tool_selection.main(["score", str(record)]) == 0
    assert f"{int(matches)} of 1 recorded calls and answers match" in capsys.readouterr().out


@pytest.mark.parametrize(("reference_answer", "recorded_answer", "matches"), [
    (" UNKNOWN ", "UNKNOWN", True),
    ("UNKNOWN", " UNKNOWN ", True),
    (" UNKNOWN ", " UNKNOWN ", True),
    ("UN KNOWN", "UNKNOWN", False),
])
def test_answer_whitespace_is_symmetric_but_internal_text_is_exact(
    reference_answer, recorded_answer, matches, tmp_path, monkeypatch, capsys,
):
    path = tmp_path / "questions.xml"
    root = ET.Element("questions")
    pair = ET.SubElement(root, "qa_pair", id="scope")
    ET.SubElement(pair, "question").text = "Synthetic no-tool question"
    ET.SubElement(pair, "calls").text = "[]"
    ET.SubElement(pair, "answer").text = reference_answer
    ET.ElementTree(root).write(path, encoding="utf-8")
    monkeypatch.setattr(tool_selection, "QUESTIONS", path)
    record = tmp_path / "run.json"
    record.write_text(json.dumps({"scope": {"calls": [], "answer": recorded_answer}}),
                      encoding="utf-8")
    assert tool_selection.main(["score", str(record)]) == 0
    assert f"{int(matches)} of 1 recorded calls and answers match" in capsys.readouterr().out
