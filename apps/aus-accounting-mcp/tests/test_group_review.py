"""Keep independent engine outcomes intact in a review index."""
import hashlib
import json
import runpy
from pathlib import Path

import pytest

BUILD = runpy.run_path(str(Path(__file__).parents[1] / "examples/group_review.py"))["build_index"]


@pytest.fixture
def manifest(tmp_path):
    reports = []
    for index, (engine, text) in enumerate([
        ("company", "Supplied-input rate result. Professional review required.\n"),
        ("trust", "FACTS_NOT_ESTABLISHED: missing deed evidence.\n"),
        ("loan", "REVIEW: unsupported matter. No approval.\n"),
    ]):
        path = tmp_path / f"report-{index}.txt"
        path.write_text(text, encoding="utf-8")
        reports.append({"id": str(index), "entity": f"Synthetic Entity {index}",
                        "period": "FY2026", "engine": engine, "engine_version": "fixture",
                        "scope": "Fabricated test output", "path": path.name,
                        "sha256": hashlib.sha256(path.read_bytes()).hexdigest()})
    path = tmp_path / "manifest.json"
    path.write_text(json.dumps({"schema_version": "group-review-input.v1", "reports": reports}))
    return path


def test_preserves_all_outputs_without_group_verdict(manifest):
    result = BUILD(manifest)
    assert len(result["reports"]) == 3
    assert "FACTS_NOT_ESTABLISHED" in result["reports"][1]["source_output"]
    assert "REVIEW" in result["reports"][2]["source_output"]
    assert "status" not in result and "approved" not in result
    for report in result["reports"]:
        assert report["source_output"] == (manifest.parent / report["path"]).read_bytes().decode()


@pytest.mark.parametrize("change", ["tamper", "missing", "duplicate", "escape", "empty"])
def test_incomplete_or_mismatched_evidence_fails(manifest, change):
    data = json.loads(manifest.read_text())
    if change == "tamper":
        (manifest.parent / "report-0.txt").write_text("Replaced report")
    elif change == "missing":
        data["reports"][0]["path"] = "missing.txt"
    elif change == "duplicate":
        data["reports"].append(data["reports"][0])
    elif change == "escape":
        data["reports"][0]["path"] = "../outside.txt"
    else:
        data["reports"] = []
    manifest.write_text(json.dumps(data))
    with pytest.raises((ValueError, OSError)):
        BUILD(manifest)
