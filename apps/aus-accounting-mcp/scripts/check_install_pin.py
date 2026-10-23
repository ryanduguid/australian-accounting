"""Check the README's selected release against its local namespaced tag."""

from __future__ import annotations

import re
import subprocess
import sys
import tomllib
from pathlib import Path

APP = Path(__file__).resolve().parents[1]
NAME = "aus-accounting-mcp"


def install_version(readme: str) -> str:
    section = readme.split("## Install\n", 1)[1].split("\n## ", 1)[0]
    block = re.search(r"```bash\n(.*?)\n```", section, re.DOTALL)
    match = (
        None
        if block is None
        else re.fullmatch(rf"uvx --from '{NAME}==([0-9]+\.[0-9]+\.[0-9]+)' {NAME}", block.group(1))
    )
    if match is None:
        raise ValueError("The first install command must select an exact release with uvx --from.")
    return match.group(1)


def check_install_pin(readme: str, repository: Path) -> str:
    version = install_version(readme)
    tag = f"{NAME}/v{version}"
    manifest = subprocess.run(  # nosec B607 - fixed git program, no shell
        ["git", "show", f"{tag}:apps/{NAME}/pyproject.toml"],
        cwd=repository,
        check=True,
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    project = tomllib.loads(manifest.stdout)["project"]
    if project["name"] != NAME or project["version"] != version:
        raise ValueError(f"The install pin does not match the manifest at {tag}.")
    return tag


def main() -> int:
    try:
        tag = check_install_pin((APP / "README.md").read_text(encoding="utf-8"), APP.parents[1])
    except subprocess.CalledProcessError:
        print(
            "Cannot read the selected release tag. Run git fetch --tags and retry.", file=sys.stderr
        )
        return 1
    except (IndexError, KeyError, ValueError) as error:
        print(f"Install pin check failed: {error}", file=sys.stderr)
        return 1
    print(f"Install pin matches {tag} and its tagged manifest (local check only).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
