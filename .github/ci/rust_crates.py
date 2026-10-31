"""List the repository's Rust crates for the CI matrices.

usage: python .github/ci/rust_crates.py >> "$GITHUB_OUTPUT"

Every packages/*/Cargo.toml and apps/*/Cargo.toml is a crate CI checks; the
manifests are the only list, so adding a crate edits no workflow. Writes:

- ci-matrix: the ci-rust.yml include list (package, root, binary, smoke) of
  every crate except those whose own workflow calls ci-rust.yml
  ([package.metadata.ci] workflow), for ci.yml;
- parity: the artefact names, as a brace pattern, of au-accounting and the
  engines it links by path;
- parity-count: how many native commands that pattern unpacks.

CI smokes one command per crate: its one [[bin]] name, or the crate name from
src/main.rs. A crate with more commands names the one CI smokes in
[package.metadata.ci] binary; one with more and no choice, or with none, stops
the listing. Its smoke arguments are [package.metadata.ci] smoke, or --api-help.
"""

from __future__ import annotations

import json
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
UMBRELLA = "apps/au-accounting"


def crates(root: Path = ROOT, ci_yml: bool = False) -> list[dict[str, str]]:
    """Every crate, or with ci_yml only those no workflow of their own calls."""
    found = []
    manifests = [*root.glob("packages/*/Cargo.toml"), *root.glob("apps/*/Cargo.toml")]
    for manifest in sorted(manifests):
        where = manifest.relative_to(root).as_posix()
        data = tomllib.loads(manifest.read_text(encoding="utf-8"))
        package = data["package"]
        # ci-rust.yml uses one input as both the directory and the crate name.
        if package["name"] != manifest.parent.name:
            sys.exit(f"{where}: the crate must be named as its directory")
        ci = package.get("metadata", {}).get("ci", {})
        names = [b["name"] for b in data.get("bin", [])]
        if not names and (manifest.parent / "src/main.rs").is_file():
            names = [package["name"]]
        if "binary" not in ci and (manifest.parent / "src/bin").exists():
            sys.exit(f"{where}: ci-rust.yml smokes one command; this crate builds more")
        if "binary" in ci:
            if ci["binary"] not in names:
                sys.exit(f"{where}: [package.metadata.ci] binary is not one of its [[bin]] names")
            names = [ci["binary"]]
        if len(names) > 1:
            sys.exit(f"{where}: ci-rust.yml smokes one command; this crate builds more")
        if not names:
            sys.exit(f"{where}: ci-rust.yml smokes a command; this crate builds none")
        if ci_yml and "workflow" in ci:
            continue
        found.append({
            "package": package["name"],
            "root": manifest.parent.parent.name,
            "binary": names[0],
            "smoke": ci.get("smoke", "--api-help"),
        })
    if not found:
        sys.exit("No Rust crates found; run from a checkout with packages/ and apps/")
    return found


def linked(root: Path = ROOT) -> list[str]:
    data = tomllib.loads((root / UMBRELLA / "Cargo.toml").read_text(encoding="utf-8"))
    # An artefact is named after the crate, which a renamed dependency gives as package.
    return sorted(
        spec.get("package", name) for name, spec in data["dependencies"].items()
        if isinstance(spec, dict) and "path" in spec
    )


def main() -> None:
    parity = [*linked(), Path(UMBRELLA).name]
    print("ci-matrix=" + json.dumps({"include": crates(ci_yml=True)}, separators=(",", ":")))
    print("parity={" + ",".join(parity) + "}")
    print(f"parity-count={len(parity)}")


if __name__ == "__main__":
    main()
