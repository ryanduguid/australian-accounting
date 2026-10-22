"""Keep install examples consistent without tying a release to development metadata."""

from __future__ import annotations

import importlib.util
import subprocess
from pathlib import Path

import pytest

APP = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    "check_install_pin", APP / "scripts/check_install_pin.py"
)
assert SPEC is not None and SPEC.loader is not None
checker = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(checker)


def test_readme_first_install_selects_an_exact_release() -> None:
    checker.install_version((APP / "README.md").read_text(encoding="utf-8"))


@pytest.mark.parametrize(
    "command",
    [
        "uvx aus-accounting-mcp",
        "uvx aus-accounting-mcp==1.2.3",
        "uvx --from 'aus-accounting-mcp>=1.2.3' aus-accounting-mcp",
    ],
)
def test_rejects_an_unpinned_first_install(command: str) -> None:
    with pytest.raises(ValueError, match="exact release"):
        checker.install_version(f"## Install\n```bash\n{command}\n```\n")


@pytest.mark.parametrize("tagged_version", ["1.2.3", "1.2.4"])
def test_uses_the_selected_tag_not_the_development_manifest(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path, tagged_version: str
) -> None:
    (tmp_path / "pyproject.toml").write_text('[project]\nversion = "2.0.0"\n', encoding="utf-8")
    readme = "## Install\n```bash\nuvx --from 'aus-accounting-mcp==1.2.3' aus-accounting-mcp\n```\n"

    def git_show(command: list[str], **kwargs: object) -> subprocess.CompletedProcess[str]:
        assert command == [
            "git",
            "show",
            "aus-accounting-mcp/v1.2.3:apps/aus-accounting-mcp/pyproject.toml",
        ]
        assert kwargs["cwd"] == tmp_path
        return subprocess.CompletedProcess(
            command, 0, f'[project]\nname = "aus-accounting-mcp"\nversion = "{tagged_version}"\n'
        )

    monkeypatch.setattr(checker.subprocess, "run", git_show)
    if tagged_version == "1.2.3":
        assert checker.check_install_pin(readme, tmp_path) == "aus-accounting-mcp/v1.2.3"
    else:
        with pytest.raises(ValueError, match="does not match"):
            checker.check_install_pin(readme, tmp_path)
