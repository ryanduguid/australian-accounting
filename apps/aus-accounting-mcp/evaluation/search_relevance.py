# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

"""Measure corpus search on a fixed relevance set, hits and precision together.

    python evaluation/search_relevance.py

`search_relevance.json` holds a fabricated corpus, dated plain questions and which rows
answer each one; some questions have no answer in the corpus. A hit count alone rewards
a search for returning more, so this reports how many answers are found and how much of
what comes back is relevant, with and without partial hits. It uses no network or model.
"""

from __future__ import annotations

import json
import os
import sys
import tempfile
from pathlib import Path
from typing import Any

from aus_accounting_mcp import corpus

SET = Path(__file__).with_name("search_relevance.json")
REGISTER = "C9999A00099"
# A real principal Act's name on invented rows: every row of a title holds its name, and a
# name such as this shares words with plain questions.
ACT = "Income Tax Assessment Act 1997"


def load() -> dict[str, Any]:
    return json.loads(SET.read_text(encoding="utf-8"))


def _write(root: Path, rows: list[list[str]]) -> None:
    index = root / "markdown" / REGISTER / "sections.jsonl"
    index.parent.mkdir(parents=True)
    index.write_text(
        "".join(
            json.dumps({
                "row_id": f"{REGISTER}:{label}:{label}", "register_id": REGISTER,
                "act": ACT, "section": label, "heading": heading,
                "text": text, "version_is_current": True,
            }) + "\n"
            for label, heading, text in rows
        ),
        encoding="utf-8",
    )


def _share(part: int, whole: int) -> float:
    return round(part / whole, 3) if whole else 0.0


def _as_written(question: str, rows: list[list[str]], limit: int) -> list[str]:
    """The rows the search returned before stop words were dropped: every word, as typed."""
    terms = corpus._terms(question, "query")
    held = [
        label for label, heading, text in rows
        if corpus._holds(
            {"act": ACT, "heading": heading, "text": text}, ("act", "heading", "text"), terms
        )
    ]
    return held[:limit]


def measure(relevance: dict[str, Any], queries: list[dict[str, Any]]) -> dict[str, Any]:
    """Every figure the gate and the documentation use, for one list of questions."""
    rows, limit = relevance["rows"], relevance["limit"]
    answerable = [query for query in queries if query["relevant"]]
    unanswerable = [query for query in queries if not query["relevant"]]
    found_as_written = found_matches = found_with_partial = 0
    match_total = match_right = partial_total = partial_right = 0
    first_total = first_right = 0
    fired_on_unanswerable = 0
    with tempfile.TemporaryDirectory() as folder:
        _write(Path(folder), rows)
        previous = os.environ.get("AUS_ACCOUNTING_CORPUS_ROOT")
        os.environ["AUS_ACCOUNTING_CORPUS_ROOT"] = folder
        try:
            for query in queries:
                result = corpus.search_sections(query["question"], limit, 0)
                relevant = set(query["relevant"])
                matches = [hit["section"] for hit in result["matches"]]
                partial = [hit["section"] for hit in result["partial_hits"]]
                written = _as_written(query["question"], rows, limit)
                found_as_written += bool(relevant & set(written))
                found_matches += bool(relevant & set(matches))
                found_with_partial += bool(relevant & set(matches + partial))
                match_total += len(matches)
                match_right += len(relevant & set(matches))
                partial_total += len(partial)
                partial_right += len(relevant & set(partial))
                if partial and not matches:
                    first_total += 1
                    first_right += partial[0] in relevant
                fired_on_unanswerable += bool(partial) and not relevant
        finally:
            if previous is None:
                del os.environ["AUS_ACCOUNTING_CORPUS_ROOT"]
            else:
                os.environ["AUS_ACCOUNTING_CORPUS_ROOT"] = previous
    return {
        "questions": len(queries),
        "answerable": len(answerable),
        "unanswerable": len(unanswerable),
        "hits_as_written": found_as_written,
        "hits_stop_words_dropped": found_matches,
        "hits_with_partial": found_with_partial,
        "matches": match_total,
        "match_precision": _share(match_right, match_total),
        "partial_hits": partial_total,
        "all_hit_precision": _share(partial_right, partial_total),
        "first_hit_questions": first_total,
        "first_hit_precision": _share(first_right, first_total),
        "unanswerable_with_partial_hits": _share(fired_on_unanswerable, len(unanswerable)),
    }


def passes(figures: dict[str, Any], gate: dict[str, float]) -> bool:
    """Whether partial hits are precise enough to stay on by default."""
    return (
        figures["first_hit_precision"] >= gate["first_hit_precision_at_least"]
        and figures["all_hit_precision"] >= gate["all_hit_precision_at_least"]
        and figures["unanswerable_with_partial_hits"]
        <= gate["unanswerable_with_partial_hits_at_most"]
    )


def main() -> int:
    relevance = load()
    failed = 0
    for name in ("queries", "held_out"):
        figures = measure(relevance, relevance[name])
        passed = passes(figures, relevance["gate"])
        failed += not passed
        print(name, json.dumps(figures, indent=2))
        print(name, "gate:", "PASS" if passed else "FAIL")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
