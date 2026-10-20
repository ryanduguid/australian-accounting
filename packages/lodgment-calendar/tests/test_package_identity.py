from importlib.metadata import version
from pathlib import Path

import lodgmentcalendar

ROOT = Path(__file__).resolve().parents[1]


def test_unreleased_distribution_uses_the_project_identity() -> None:
    pyproject = (ROOT / "pyproject.toml").read_text(encoding="utf-8")
    manifest = (ROOT / "MANIFEST.in").read_text(encoding="utf-8")
    lockfile = (ROOT / "uv.lock").read_text(encoding="utf-8")
    citation = (ROOT / "CITATION.cff").read_text(encoding="utf-8")
    llms = (ROOT / "llms.txt").read_text(encoding="utf-8")
    release_notes = (ROOT / "RELEASE_NOTES.md").read_text(encoding="utf-8")
    readme = " ".join((ROOT / "README.md").read_text(encoding="utf-8").split())

    assert version("lodgment-calendar") == lodgmentcalendar.__version__ == "0.1.0"
    assert f'version = "{lodgmentcalendar.__version__}"' in pyproject
    assert f"version: {lodgmentcalendar.__version__}\n" in citation
    assert 'name = "lodgment-calendar"' in pyproject
    assert "[project.scripts]" not in pyproject
    assert 'name = "lodgment-calendar"' in lockfile
    for shipped in ("CITATION.cff", "DISCLAIMER.md", "RELEASE_NOTES.md", "RELEASING.md",
                    "llms.txt", "uv.lock", "docs"):
        assert shipped in manifest
    assert release_notes.startswith("# v0.1.0\n\nNot yet released.\n")
    assert "**Package lifecycle:** not yet released." in readme
    for text in (pyproject, citation, llms):
        assert "australian-accounting/tree/main/packages/lodgment-calendar" in text
