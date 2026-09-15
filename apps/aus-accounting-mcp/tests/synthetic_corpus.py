"""Build the fabricated legislation corpus the retrieval tests and evaluation replay use.

Every title, provision, rate and register link here is invented. The real corpus is the
operator's own folder and never enters this repository.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

ATTRIBUTION = "Based on content from a synthetic register at 2099-01-01."
LEVY_ACT = "Synthetic Levy Act 2099"
CHARGE_ACT = "Synthetic Charge Act 2099"
LONG_TEXT_CHARS = 20000


def section(
    register_id: str,
    ordinal: str,
    label: str,
    text: str,
    *,
    act: str,
    current: bool = True,
    container: str = "Part 1",
) -> dict[str, Any]:
    return {
        "register_id": register_id,
        "act": act,
        "collection": "Act",
        "compilation_number": "12",
        "compilation_date": "2098-07-01",
        "version_is_current": current,
        "row_id": f"{register_id}:{ordinal}:{label}",
        "section": label,
        "heading": f"{label} {text.split('.')[0]}",
        "container": container,
        "kind": "section",
        "granularity": "section",
        "text": text,
        "source_url": f"https://example.invalid/{register_id}/text",
        "register_page": f"https://example.invalid/{register_id}/latest",
        "licence": "CC BY 4.0",
        "attribution": ATTRIBUTION,
    }


def rate(
    rate_id: str,
    topic: str,
    content: str,
    *,
    amounts: list[str],
    years: list[str],
    section_label: str = "5-10",
) -> dict[str, Any]:
    return {
        "register_id": "C9999A00001",
        "act": LEVY_ACT,
        "collection": "Act",
        "compilation_number": "12",
        "compilation_date": "2098-07-01",
        "section": section_label,
        "heading": f"{section_label} Synthetic rate",
        "register_page": "https://example.invalid/C9999A00001/latest",
        "topic": topic,
        "kind": "rate",
        "amounts": amounts,
        "years": years,
        "content": content,
        "rate_id": rate_id,
        "content_ascii": content,
        "heading_ascii": f"{section_label} Synthetic rate",
    }


def write(path: Path, rows: list[dict[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        "".join(json.dumps(row, ensure_ascii=False) + "\n" for row in rows), encoding="utf-8"
    )


def build(root: Path) -> Path:
    """Write the fabricated corpus under root and return it."""
    write(
        root / "markdown" / "C9999A00001" / "sections.jsonl",
        [
            section("C9999A00001", "0001", "1",
                    "This Act may be cited as the Levy Act.", act=LEVY_ACT),
            section("C9999A00001", "0002", "5-10",
                    "The synthetic levy rate is 7% of the assessable amount.", act=LEVY_ACT),
            section("C9999A00001", "0003", "5-15",
                    "A synthetic levy exemption applies to a small entity.", act=LEVY_ACT),
        ],
    )
    write(
        root / "markdown" / "C9999A00002" / "sections.jsonl",
        [
            section("C9999A00002", "0001", "2",
                    "The synthetic levy rate is not applied here.",
                    act=CHARGE_ACT, current=False),
            section("C9999A00002", "0002", "3", "x" * LONG_TEXT_CHARS, act=CHARGE_ACT),
        ],
    )
    write(
        root / "rates" / "rates.jsonl",
        [
            rate("R00001", "levies", "The rate is 7% of the assessable amount.",
                 amounts=["7%"], years=["2098-99"]),
            rate("R00002", "superannuation", "The cap is $30,000 for the year.",
                 amounts=["$30,000"], years=["2098-99"], section_label="9-20"),
        ],
    )
    (root / "sources.json").write_text(
        json.dumps({
            "corpus": "Synthetic statutes",
            "source": "Synthetic register",
            "retrieved": "2099-01-01",
            "licence": "CC BY 4.0",
            "licence_url": "https://creativecommons.org/licenses/by/4.0/",
            "attribution_markdown": ATTRIBUTION,
        }),
        encoding="utf-8",
    )
    return root
