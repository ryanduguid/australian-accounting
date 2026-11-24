# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: AGPL-3.0-or-later
# Additional terms under AGPL-3.0 section 7(b): see NOTICE.

"""List the repository's Rust crates for the CI matrices.

usage: python .github/ci/rust_crates.py >> "$GITHUB_OUTPUT"

Every packages/*/Cargo.toml and apps/*/Cargo.toml is a crate CI checks; the
manifests are the only list, so adding a crate edits no workflow. Writes:

- ci-matrix: the ci-rust.yml include list (package, root, binary, smoke and
  candidate archive inputs) of
  every crate except those whose own workflow calls ci-rust.yml
  ([package.metadata.ci] workflow), for ci.yml;
- parity: the artefact names, as a brace pattern, of au-accounting and the
  engines it links by path;
- parity-count: how many native commands that pattern unpacks.

CI smokes one command per crate: its one [[bin]] name, or the crate name from
src/main.rs. A crate with more commands names the one CI smokes in
[package.metadata.ci] binary; one with more and no choice, or with none, stops
the listing. Its smoke arguments are [package.metadata.ci] smoke, or --api-help;
[package.metadata.ci] candidate-test names an ignored test run on its archive.
The optional extra-binaries, extra-examples and candidate-files lists name what
the archive includes alongside the primary command.
"""

from __future__ import annotations

import json
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
UMBRELLA = "apps/au-accounting"
DEP_SECTIONS = ("dependencies", "dev-dependencies", "build-dependencies")


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
            "candidate-test": ci.get("candidate-test", ""),
            "extra-binaries": " ".join(ci.get("extra-binaries", [])),
            "extra-examples": " ".join(ci.get("extra-examples", [])),
            "candidate-files": " ".join(ci.get("candidate-files", [])),
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


def _manifest_path_deps(manifest: Path) -> list[Path]:
    data = tomllib.loads(manifest.read_text(encoding="utf-8"))
    tables: list[object] = [data.get(section) for section in DEP_SECTIONS]
    targets = data.get("target", {})
    if isinstance(targets, dict):
        for target in targets.values():
            if isinstance(target, dict):
                tables.extend(target.get(section) for section in DEP_SECTIONS)
    paths: list[Path] = []
    for table in tables:
        if not isinstance(table, dict):
            continue
        for spec in table.values():
            if isinstance(spec, dict) and isinstance(spec.get("path"), str):
                paths.append((manifest.parent / spec["path"]).resolve())
    return paths


def crate_dependencies(root: Path = ROOT, ci_yml: bool = True) -> dict[str, set[str]] | None:
    """Map each crate to the local crates it links by path or candidate-files.

    Returns None if any path dependency or candidate file escapes the listed crates,
    so callers fail closed and run every crate.
    """
    listed = crates(root, ci_yml=ci_yml)
    dirs = {
        crate["package"]: (root / crate["root"] / crate["package"]).resolve()
        for crate in listed
    }
    prefixes = {
        f"{crate['root']}/{crate['package']}/": crate["package"]
        for crate in listed
    }
    deps: dict[str, set[str]] = {name: set() for name in dirs}
    for crate in listed:
        name = crate["package"]
        own_dir = dirs[name]
        for manifest in sorted(own_dir.rglob("Cargo.toml")):
            if "target" in manifest.relative_to(own_dir).parts:
                continue
            for target in _manifest_path_deps(manifest):
                if target.is_relative_to(own_dir):
                    continue
                owner = next((pkg for pkg, d in dirs.items() if target.is_relative_to(d)), None)
                if owner is None:
                    return None
                deps[name].add(owner)
        for candidate in crate["candidate-files"].split():
            owner = next((pkg for prefix, pkg in prefixes.items() if candidate.startswith(prefix)), None)
            if owner is None:
                return None
            if owner != name:
                deps[name].add(owner)
    return deps


def main() -> None:
    parity = [*linked(), Path(UMBRELLA).name]
    print("ci-matrix=" + json.dumps({"include": crates(ci_yml=True)}, separators=(",", ":")))
    print("parity={" + ",".join(parity) + "}")
    print(f"parity-count={len(parity)}")


if __name__ == "__main__":
    main()
