import subprocess
import sys
from importlib.metadata import version
from pathlib import Path

import edwinnixon


ROOT = Path(__file__).resolve().parents[1]


def test_published_distribution_uses_the_project_identity() -> None:
    pyproject = (ROOT / "pyproject.toml").read_text(encoding="utf-8")
    lockfile = (ROOT / "uv.lock").read_text(encoding="utf-8")
    readme = " ".join((ROOT / "README.md").read_text(encoding="utf-8").split())
    package_init = (ROOT / "edwinnixon" / "__init__.py").read_text(encoding="utf-8")
    cli = (ROOT / "edwinnixon" / "cli.py").read_text(encoding="utf-8")
    release_notes = (ROOT / "RELEASE_NOTES.md").read_text(encoding="utf-8")

    assert 'name = "the-exchequer-tally"' in pyproject
    assert 'the-exchequer-tally = "edwinnixon.cli:main"' in pyproject
    assert 'name = "the-exchequer-tally"' in lockfile
    assert 'name = "edwinnixon"' not in lockfile
    assert '_dist_version("the-exchequer-tally")' in package_init
    assert 'prog="the-exchequer-tally"' in cli
    assert version("the-exchequer-tally") == edwinnixon.__version__ == "0.1.3"
    assert not (ROOT / ".github" / "workflows" / "release.yml").exists()
    assert "**Package lifecycle:** published." in readme
    assert "pip install the-exchequer-tally" in readme
    assert "github.com/ryanduguid/australian-accounting" in readme
    assert "`the-exchequer-tally` distribution and command" in readme
    assert "`edwinnixon` import package" in readme
    assert "the-exchequer-tally bre-test" in readme
    assert "edwinnixon bre-test" not in readme
    assert release_notes.startswith("# v0.1.3\n")
    assert "first PyPI release" in release_notes
    assert "release-the-exchequer-tally.yml" in release_notes

    help_result = subprocess.run(
        [sys.executable, "-m", "edwinnixon.cli", "--help"],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    assert help_result.returncode == 0
    assert help_result.stdout.startswith("usage: the-exchequer-tally")
