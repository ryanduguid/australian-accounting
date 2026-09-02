import subprocess
import sys
from importlib.metadata import version
from pathlib import Path

import wiptally


ROOT = Path(__file__).resolve().parents[1]


def test_published_distribution_uses_the_project_identity() -> None:
    pyproject = (ROOT / "pyproject.toml").read_text(encoding="utf-8")
    lockfile = (ROOT / "uv.lock").read_text(encoding="utf-8")
    readme = " ".join((ROOT / "README.md").read_text(encoding="utf-8").split())
    citation = (ROOT / "CITATION.cff").read_text(encoding="utf-8")
    llms = (ROOT / "llms.txt").read_text(encoding="utf-8")
    release_notes = (ROOT / "RELEASE_NOTES.md").read_text(encoding="utf-8")

    assert version("the-wip-tally") == wiptally.__version__ == "0.1.0"
    assert 'name = "the-wip-tally"' in pyproject
    assert 'wip-tally = "wiptally.cli:main"' in pyproject
    assert 'name = "the-wip-tally"' in lockfile
    assert "/RELEASE_NOTES.md" in pyproject
    assert "/uv.lock" in pyproject
    assert "**Package lifecycle:** published." in readme
    assert "pip install the-wip-tally" in readme
    assert "github.com/ryanduguid/australian-accounting" in readme
    assert "raw.githubusercontent.com/ryanduguid/australian-accounting/main" in readme
    assert (
        "https://raw.githubusercontent.com/ryanduguid/australian-accounting/main/"
        "packages/the-wip-tally/examples/mapping.example.json"
    ) in readme
    assert "release-the-wip-tally.yml" in release_notes
    assert release_notes.startswith("# v0.1.0\n")
    assert "first PyPI release" in release_notes
    assert "date-released: 2026-09-02" in citation
    assert "australian-accounting/tree/main/packages/the-wip-tally" in citation
    assert "australian-accounting/tree/main/packages/the-wip-tally" in llms
    assert "github.com/ryanduguid/TheWIPTally" not in "\n".join(
        (pyproject, readme, citation, llms)
    )

    help_result = subprocess.run(
        [sys.executable, "-m", "wiptally", "--help"],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    assert help_result.returncode == 0
    assert help_result.stdout.startswith("usage: wip-tally")
