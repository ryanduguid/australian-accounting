# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

"""Partial hits stay on by default only while the fixed relevance set passes its gate."""

from pathlib import Path

import pytest

from aus_accounting_mcp import corpus
from evaluation import search_relevance

RELEVANCE = search_relevance.load()
REFERENCE = " ".join(
    (Path(__file__).resolve().parents[1] / "docs" / "REFERENCE.md")
    .read_text(encoding="utf-8").split()
)


@pytest.mark.parametrize("name", ["queries", "held_out"])
def test_the_published_figures_are_what_the_set_measures(name):
    assert search_relevance.measure(RELEVANCE, RELEVANCE[name]) == RELEVANCE["published"][name]


@pytest.mark.parametrize("name", ["queries", "held_out"])
def test_partial_hits_pass_the_gate(name):
    # Failing this means the fallback misleads: turn it off, do not move the gate.
    figures = search_relevance.measure(RELEVANCE, RELEVANCE[name])

    assert search_relevance.passes(figures, RELEVANCE["gate"])


def test_the_one_word_floor_fails_the_gate_as_recorded(monkeypatch):
    # The record of why the floor exists, measured again rather than only stated.
    monkeypatch.setattr(corpus, "MIN_PARTIAL_WORDS", 1)
    monkeypatch.setattr(corpus, "STOP_WORDS", corpus.STOP_WORDS - {"s", "t"})

    figures = search_relevance.measure(RELEVANCE, RELEVANCE["queries"])

    recorded = RELEVANCE["one_word_floor"]
    assert {key: figures[key] for key in recorded} == recorded
    assert not search_relevance.passes(figures, RELEVANCE["gate"])


def test_the_command_exits_zero_when_both_lists_pass(capsys):
    assert search_relevance.main() == 0
    assert capsys.readouterr().out.count("gate: PASS") == 2


def test_the_command_exits_one_on_a_threshold_nothing_can_meet(capsys, monkeypatch):
    relevance = search_relevance.load()
    relevance["gate"]["first_hit_precision_at_least"] = 1.1
    monkeypatch.setattr(search_relevance, "load", lambda: relevance)

    assert search_relevance.main() == 1
    assert capsys.readouterr().out.count("gate: FAIL") == 2


@pytest.mark.parametrize("questions", [28, 18])
def test_either_list_failing_fails_the_command(capsys, monkeypatch, questions):
    monkeypatch.setattr(
        search_relevance, "passes", lambda figures, gate: figures["questions"] != questions
    )

    assert search_relevance.main() == 1
    assert capsys.readouterr().out.count("gate: FAIL") == 1


def test_a_question_with_no_answer_is_held_on_each_list():
    for name in ("queries", "held_out"):
        assert any(not query["relevant"] for query in RELEVANCE[name])
        assert any(query["relevant"] for query in RELEVANCE[name])


def test_the_reference_publishes_the_figures_and_the_limits():
    assert "dated 6 November 2026" in REFERENCE
    assert RELEVANCE["dated"] == "2026-11-06"
    rows = {
        "| One-word floor, not shipped, 28 questions | 20 | 0 | 1 | 17 | 0.478 | 0.271 | 0.625 |":
            RELEVANCE["one_word_floor"],
        "| Two-word floor, the same 28 | 20 | 0 | 1 | 9 | 0.778 | 0.625 | 0 |":
            RELEVANCE["published"]["queries"],
        "| Two-word floor, 18 held-out questions | 12 | 0 | 1 | 10 | 0.8 | 0.588 | 0 |":
            RELEVANCE["published"]["held_out"],
    }
    for row, figures in rows.items():
        assert row in REFERENCE
        cells = [cell.strip() for cell in row.strip("|").split("|")]
        assert cells[3:] == [
            f"{figures[key]:g}" for key in (
                "hits_stop_words_dropped", "hits_with_partial", "first_hit_precision",
                "all_hit_precision", "unanswerable_with_partial_hits",
            )
        ]
    assert "A partial hit proves only" in REFERENCE
    assert "does not prove that the provision answers the question" in REFERENCE
