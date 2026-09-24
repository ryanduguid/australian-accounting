"""The OpenAccountants handoff document states what the engine produces for its case."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOCUMENT = ROOT / "docs" / "openaccountants-handoff.md"
CASE = "examples/quarterly_remittance_2026_27.csv"


def test_document_matches_the_engine_run(tmp_path):
    result = subprocess.run(
        [sys.executable, "-c", "import sys; from paydaysuper.cli import main; sys.exit(main())",
         CASE, "--as-at", "2026-10-28", "-o", str(tmp_path / "report.csv")],
        cwd=ROOT, text=True, capture_output=True, check=False,
    )
    assert result.returncode == 2, result.stdout + result.stderr
    document = " ".join(DOCUMENT.read_text(encoding="utf-8").split())
    for claim in ("ON_TIME: 0  AT_RISK: 0  LATE: 7  UNPAID: 0  UNKNOWN: 0  SKIPPED: 0",
                  "experimental estimated SG charge $62.49 - $99.98"):
        assert claim in result.stdout
        assert " ".join(claim.split()) in document
    assert "LATE, 101 days late" in result.stdout
    assert "LATE, 15 days late" in result.stdout
    assert CASE in DOCUMENT.read_text(encoding="utf-8")
