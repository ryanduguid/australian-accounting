"""Build the fabricated ATO rulings runs the rulings tests and evaluation replay use.

Every docid, title, paragraph and page hash here is invented, in the shape the corpus
builder's rulings stage writes. The real runs are the operator's own folder and never
enter this repository.
"""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any

NOTICE = (
    "You are free to copy, adapt, modify, transmit and distribute this material as you "
    "wish (but not in any way that suggests the ATO or the Commonwealth endorses you or "
    "any of your services or products)."
)
NOTICE_URL = "https://www.ato.gov.au/about-ato/using-our-website/copyright-notice"
RULING = "TXR/TR20991/NAT/ATO/00001"
GUIDELINE = "COG/PCG20991/NAT/ATO/00001"
STATEMENT = "LIT/ICD/X1/2099/00001"
EDITED_OLD = "EV/1099000000001"
EDITED_NEW = "EV/1099000000002"
LONG_TEXT_CHARS = 15000
OLDER_RUN = "run-2099-01-01"
NEWER_RUN = "run-2099-02-01"


def document(
    docid: str,
    family: str,
    title: str,
    paragraphs: list[tuple[str, str, str]],
    *,
    fetched_on: str,
    numbering: str = "document",
    licence_basis: str = "document-notice",
) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    """One manifest record and its rows, exactly as the rulings stage pairs them."""
    digest = hashlib.sha256(f"{docid}|{fetched_on}".encode()).hexdigest()
    url = f"https://www.ato.gov.au/law/view/document?DocID={docid}"
    record = {
        "docid": docid,
        "reference": docid,
        "family": family,
        "title": title,
        "document_type": family,
        "issued": "2099/01/01",
        "source_url": url,
        "fetched_on": fetched_on,
        "sha256": digest,
        "bytes": 1000,
        "raw_file": f"raw/{digest[:12]}.html",
        "licence_basis": licence_basis,
        "paragraphs": len(paragraphs),
    }
    rows = [
        {
            "docid": docid,
            "family": family,
            "title": title,
            "paragraph": label,
            "numbering": numbering,
            "heading": heading,
            "text": text,
            "source_url": url,
            "source_sha256": digest,
            "fetched_on": fetched_on,
            "licence_basis": licence_basis,
        }
        for label, heading, text in paragraphs
    ]
    return record, rows


def write_run(
    directory: Path,
    fetched_on: str,
    documents: list[tuple[dict[str, Any], list[dict[str, Any]]]],
    excluded: tuple[str, ...] = (),
) -> Path:
    directory.mkdir(parents=True)
    rows = [row for _, document_rows in documents for row in document_rows]
    (directory / "rulings.jsonl").write_text(
        "".join(json.dumps(row, ensure_ascii=False, sort_keys=True) + "\n" for row in rows),
        encoding="utf-8", newline="\n",
    )
    manifest = {
        "stage": "rulings",
        "fetched_on": fetched_on,
        "targets": [record["docid"] for record, _ in documents] + list(excluded),
        "documents": [record for record, _ in documents],
        "excluded": [
            {"docid": docid, "reason": "personal data patterns: email"} for docid in excluded
        ],
        "rows": len(rows),
        "reuse_notice": NOTICE,
        "reuse_notice_url": NOTICE_URL,
    }
    (directory / "manifest.json").write_text(
        json.dumps(manifest, indent=2, ensure_ascii=False, sort_keys=True) + "\n",
        encoding="utf-8", newline="\n",
    )
    (directory / "LICENCE-NOTICE.md").write_text("# Fabricated fixture\n", encoding="utf-8")
    return directory


def ruling(fetched_on: str, *, second: str) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    return document(
        RULING, "Taxation Ruling", "TR 2099/1 - Income tax: synthetic reimbursement agreements",
        [
            ("1", "What this Ruling is about",
             "This Ruling explains how the synthetic reimbursement rule applies to a "
             "fabricated trust."),
            ("2", "Ruling", second),
            ("3", "Ruling",
             "An ordinary family dealing is excluded from the rule about reimbursement."),
        ],
        fetched_on=fetched_on,
    )


def build(root: Path) -> Path:
    """Two runs: the newer re-fetches the ruling and excludes the first edited advice."""
    write_run(
        root / OLDER_RUN,
        "2099-01-01",
        [
            ruling("2099-01-01", second="OLDER COPY: a synthetic reimbursement agreement existed."),
            document(
                GUIDELINE, "Practical Compliance Guideline",
                "PCG 2099/1 - Synthetic reimbursement: ATO compliance approach",
                [
                    ("1", "Green zone", "A family distribution used for family purposes sits in "
                                        "the green zone for the synthetic reimbursement rule."),
                    ("2", "Red zone", "A circular flow of funds sits in the red zone."),
                ],
                fetched_on="2099-01-01",
            ),
            document(
                EDITED_OLD, "Edited version of private advice",
                "Edited version of private advice: synthetic withheld advice",
                [("1", "Question", "Is the synthetic withheld expense deductible?")],
                fetched_on="2099-01-01", numbering="sequential",
                licence_basis="site-notice-via-dc-rights",
            ),
        ],
    )
    write_run(
        root / NEWER_RUN,
        "2099-02-01",
        [
            ruling("2099-02-01", second="A synthetic reimbursement agreement exists where a "
                                        "benefit passes to another party."),
            document(
                STATEMENT, "Decision Impact Statement",
                "Decision impact statement: Synthetic Holdings v Commissioner",
                [
                    ("1", "Summary of decision", "The court held a passive synthetic "
                                                 "entitlement was not a loan."),
                    ("2", "ATO view", "L" * LONG_TEXT_CHARS),
                ],
                fetched_on="2099-02-01",
            ),
            document(
                EDITED_NEW, "Edited version of private advice",
                "Edited version of private advice: synthetic boat expenses",
                [
                    ("1", "Question", "Are the synthetic boat expenses deductible?"),
                    ("2", "Answer", "No. The synthetic boat is used privately."),
                ],
                fetched_on="2099-02-01", numbering="sequential",
                licence_basis="site-notice-via-dc-rights",
            ),
        ],
        excluded=(EDITED_OLD,),
    )
    return root
