"""Compose review artefacts from assessed rows without recalculating the engine."""
from __future__ import annotations

import json
from dataclasses import asdict
from datetime import date
from decimal import Decimal
from pathlib import Path

from . import LAW_CONTENT_DATE
from .practitioner_pack import parse_report_snapshot, render_practitioner_pack
from .report import Result, render_csv


def _json_value(value: object) -> str:
    if isinstance(value, date):
        return value.isoformat()
    if isinstance(value, Decimal):
        return format(value, ".2f")
    raise TypeError(f"unsupported evidence value: {type(value).__name__}")


def build_evidence_pack(
    results: list[Result], *, as_at: date,
    assessment_date: date | None = None, gic_provenance: str = "",
    remittance_only_confirmed: bool = False,
) -> dict[str, str]:
    """Return four deterministic UTF-8 files with row references and no identifiers.

    Supply results from the engine, not caller-written caveats or provenance.
    The report contains no input path. Missing receipts remain missing facts.
    """
    report = "\ufeff" + render_csv(
        results, as_at, LAW_CONTENT_DATE, assessment_date,
        gic_provenance=gic_provenance, include_employee_ids=False,
    )
    snapshot = parse_report_snapshot(
        report.encode("utf-8"), Path("report.csv"), identifiers_omitted=True,
    )
    queue = {
        "schema_version": 1,
        "report_sha256": snapshot.source_sha256,
        "as_at": as_at.isoformat(),
        "assessment_date": assessment_date.isoformat() if assessment_date else None,
        "remittance_only_confirmed": remittance_only_confirmed,
        "exceptions": [asdict(row) for row in snapshot.rows if row.verdict != "ON_TIME"],
    }
    decision = "\n".join([
        "# Payday Super decision log", "",
        "Template only. No decisions or sign-off have been recorded.", "",
        f"Report SHA-256: `{snapshot.source_sha256}`",
        f"As at: {as_at.isoformat()}",
        f"Remittance-only confirmation supplied: {str(remittance_only_confirmed).lower()}", "",
        "## Evidence reviewed", "",
        "- [ ] Match the report SHA-256 to the file reviewed.",
        "- [ ] Reconcile source row references to the private contribution input.",
        "- [ ] Record evidence references for receipts, allocation and assessment facts.",
        "- [ ] Obtain missing fund receipt evidence; never infer receipt from remittance.", "",
        "Evidence references: ______________________________", "",
        "## Decisions", "",
        "Record a human decision or escalation for every row in exceptions.json.", "",
        "| Source row | Evidence reference | Decision and reasons | Reviewer | Date | Follow-up |",
        "| --- | --- | --- | --- | --- | --- |",
        "| | | | | | |", "",
        "## Practitioner sign-off", "",
        "- [ ] Every exception is resolved with evidence or remains explicitly escalated.",
        "- [ ] Review the work under the applicable APES 110 and TPB obligations.",
        "- [ ] An appropriately authorised human decides any advice, correction or lodgement.", "",
        "Reviewer: ______________________________", "",
        "Review date (Australia): ______________________________", "",
        "Conclusion and workpaper reference: ______________________________", "",
        "Private review aid, not advice, an ATO assessment or a compliance determination. "
        "This workflow does not lodge, pay, post or approve. Missing evidence and "
        "review flags remain unresolved until a human records the decision.", "",
    ])
    return {
        "report.csv": report,
        "practitioner-review.md": render_practitioner_pack(snapshot, decision_log=True),
        "exceptions.json": json.dumps(queue, default=_json_value, indent=2, ensure_ascii=True) + "\n",
        "decision-log.md": decision,
    }


def evidence_destination(path: str | Path) -> Path:
    """Refuse an existing file, directory or symlink, preserving previous decisions."""
    destination = Path(path)
    if destination.exists() or destination.is_symlink():
        raise ValueError("evidence-pack output already exists; choose a new directory")
    return destination


def write_evidence_pack(files: dict[str, str], path: str | Path) -> None:
    """Create a new directory exclusively; preserve partial files on failure.

    Consume the pack only after successful return. Directory-wide atomic
    publication with no replacement is not portable in the standard library.
    The caller supplies an access-controlled parent; OS permissions vary.
    """
    if set(files) != {"report.csv", "practitioner-review.md", "exceptions.json", "decision-log.md"}:
        raise ValueError("evidence pack must contain exactly the four fixed filenames")
    destination = evidence_destination(path)
    destination.mkdir(mode=0o700)
    # Never unlink on failure: a completed file may already have been edited.
    for name, text in files.items():
        with (destination / name).open("xb") as stream:
            stream.write(text.encode("utf-8"))
