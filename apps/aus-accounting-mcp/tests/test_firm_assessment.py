"""The firm assessment page states the version and counts its evidence holds.

docs/firm-assessment.md is supplier information firms use in their AI
registers. It still said 32 evaluation questions after a 33rd joined
evaluation/questions.xml, so these tests hold every count the page states to
the files it describes.
"""

from __future__ import annotations

import json
import re
import xml.etree.ElementTree as ET
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PAGE = (ROOT / "docs" / "firm-assessment.md").read_text(encoding="utf-8")


def test_the_register_entry_names_the_current_version() -> None:
    project = (ROOT / "pyproject.toml").read_text(encoding="utf-8")
    version = re.search(r'^version = "([^"]+)"$', project, re.M)
    assert version is not None
    assert f"(`aus-accounting-mcp`) {version.group(1)}," in PAGE


def test_every_question_count_matches_the_evaluation_set() -> None:
    questions = ET.parse(ROOT / "evaluation" / "questions.xml").getroot().findall("qa_pair")
    stated = re.findall(r"(\d+) (?:fabricated|evaluation)\s+(?:evaluation\s+)?questions", PAGE)
    assert stated
    assert set(stated) == {str(len(questions))}


def test_the_conformance_count_matches_the_cases_file() -> None:
    cases = json.loads((ROOT / "conformance" / "cases.json").read_text(encoding="utf-8"))["cases"]
    stated = re.findall(r"(\d+) conformance cases", PAGE)
    assert stated
    assert set(stated) == {str(len(cases))}
