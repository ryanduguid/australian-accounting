import subprocess
import sys
from importlib.metadata import version
from pathlib import Path

import louisgoldberg


ROOT = Path(__file__).resolve().parents[1]


def test_unpublished_distribution_uses_the_project_identity() -> None:
    pyproject = (ROOT / "pyproject.toml").read_text(encoding="utf-8")
    lockfile = (ROOT / "uv.lock").read_text(encoding="utf-8")
    readme = " ".join((ROOT / "README.md").read_text(encoding="utf-8").split())
    package_init = (ROOT / "louisgoldberg" / "__init__.py").read_text(encoding="utf-8")
    cli = (ROOT / "louisgoldberg" / "cli.py").read_text(encoding="utf-8")
    release_notes = (ROOT / "RELEASE_NOTES.md").read_text(encoding="utf-8")

    assert 'name = "solomons-sword"' in pyproject
    assert 'solomons-sword = "louisgoldberg.cli:main"' in pyproject
    assert 'name = "solomons-sword"' in lockfile
    assert 'name = "louisgoldberg"' not in lockfile
    assert '_dist_version("solomons-sword")' in package_init
    assert 'prog="solomons-sword"' in cli
    assert version("solomons-sword") == louisgoldberg.__version__
    assert not (ROOT / ".github" / "workflows" / "release.yml").exists()
    assert "**Package lifecycle:** source-only." in readme
    assert "not published to PyPI" in readme
    assert "`solomons-sword` distribution and command" in readme
    assert "`louisgoldberg` import package" in readme
    assert "solomons-sword s100a-check" in readme
    assert "louisgoldberg s100a-check" not in readme
    assert "First published release" not in release_notes
    assert "No PyPI distribution has been published" in release_notes
    assert "fresh index-name availability check" in release_notes

    help_result = subprocess.run(
        [sys.executable, "-m", "louisgoldberg.cli", "--help"],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    assert help_result.returncode == 0
    assert help_result.stdout.startswith("usage: solomons-sword")
