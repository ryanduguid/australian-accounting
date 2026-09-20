import subprocess
import sys
from importlib.metadata import version
from pathlib import Path

import pytest
import wiptally

ROOT = Path(__file__).resolve().parents[1]


def test_published_distribution_uses_the_project_identity() -> None:
    pyproject = (ROOT / "pyproject.toml").read_text(encoding="utf-8")
    lockfile = (ROOT / "uv.lock").read_text(encoding="utf-8")
    readme = " ".join((ROOT / "README.md").read_text(encoding="utf-8").split())
    citation = (ROOT / "CITATION.cff").read_text(encoding="utf-8")
    llms = (ROOT / "llms.txt").read_text(encoding="utf-8")
    release_notes = (ROOT / "RELEASE_NOTES.md").read_text(encoding="utf-8")

    assert version("the-wip-tally") == wiptally.__version__ == "0.1.2"
    assert f'version = "{wiptally.__version__}"' in pyproject
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
    # The current version heads the notes; prior released sections remain.
    assert release_notes.startswith("# v0.1.2\n")
    assert "\n# v0.1.1\n" in release_notes
    assert "first PyPI release" in release_notes
    assert "date-released:" not in citation
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


@pytest.mark.parametrize("installed_version", ["0.0.1", None])
def test_checkout_version_ignores_unrelated_metadata(installed_version: str | None) -> None:
    code = f"""
from importlib.metadata import PackageNotFoundError
from unittest.mock import patch

with patch("importlib.metadata.version", return_value={installed_version!r},
           side_effect=PackageNotFoundError if {installed_version!r} is None else None):
    import wiptally
    print(wiptally.__version__)
"""
    result = subprocess.run(
        [sys.executable, "-c", code],
        cwd=Path(__file__).resolve().parents[1],
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 0, result.stderr
    assert result.stdout.strip() == wiptally.__version__
