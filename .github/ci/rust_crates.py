"""List the repository's Rust crates for the CI matrices.

usage: python .github/ci/rust_crates.py >> "$GITHUB_OUTPUT"

Every packages/*/Cargo.toml and apps/*/Cargo.toml is a crate CI checks; the
manifests are the only list, so adding a crate edits no workflow. Writes:

- matrix: the ci-rust.yml include list (package, root, binary, smoke);
- parity: the artefact names, as a brace pattern, of au-accounting and the
  engines it links by path;
- parity-count: how many native commands that pattern unpacks.

A crate's command is its one [[bin]] name, or the crate name without one. Its
smoke arguments are [package.metadata.ci] smoke, or --api-help.
"""

from __future__ import annotations

import json
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
UMBRELLA = "apps/au-accounting"


def crates(root: Path = ROOT) -> list[dict[str, str]]:
    found = []
    manifests = [*root.glob("packages/*/Cargo.toml"), *root.glob("apps/*/Cargo.toml")]
    for manifest in sorted(manifests):
        where = manifest.relative_to(root).as_posix()
        data = tomllib.loads(manifest.read_text(encoding="utf-8"))
        package = data["package"]
        # ci-rust.yml uses one input as both the directory and the crate name.
        if package["name"] != manifest.parent.name:
            sys.exit(f"{where}: the crate must be named as its directory")
        bins = data.get("bin", [])
        if len(bins) > 1:
            sys.exit(f"{where}: ci-rust.yml smokes one command; this crate has {len(bins)}")
        found.append({
            "package": package["name"],
            "root": manifest.parent.parent.name,
            "binary": bins[0]["name"] if bins else package["name"],
            "smoke": package.get("metadata", {}).get("ci", {}).get("smoke", "--api-help"),
        })
    if not found:
        sys.exit("No Rust crates found; run from a checkout with packages/ and apps/")
    return found


def linked(root: Path = ROOT) -> list[str]:
    data = tomllib.loads((root / UMBRELLA / "Cargo.toml").read_text(encoding="utf-8"))
    return sorted(
        name for name, spec in data["dependencies"].items()
        if isinstance(spec, dict) and "path" in spec
    )


def main() -> None:
    parity = [*linked(), Path(UMBRELLA).name]
    print("matrix=" + json.dumps({"include": crates()}, separators=(",", ":")))
    print("parity={" + ",".join(parity) + "}")
    print(f"parity-count={len(parity)}")


if __name__ == "__main__":
    main()
