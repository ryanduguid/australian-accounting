"""Preflight for the MCP Registry publication of aus-accounting-mcp.

The registry lists ``server.json`` and points at a PyPI package. Nothing on the
registry side checks that the package version named there is the one this
repository released, so this script does, with public reads only and no
credential: the exact version must exist on PyPI as one wheel and one source
distribution, its README must carry the registry's ``mcp-name`` ownership
marker, and PyPI's provenance record for both files must name this
repository's own release workflow and trusted-publisher environment. It also
binds ``server.json`` to ``pyproject.toml`` so the registry can never name a
version the source tree does not.

Run from ``apps/aus-accounting-mcp``. Exit 0 prints the checks it made; any
failure exits 1 and names the first mismatch.
"""

from __future__ import annotations

import hashlib
import json
import os
import re
import sys
import urllib.error
import urllib.request
from pathlib import Path
from typing import Callable

SERVER_NAME = "io.github.ryanduguid/aus-accounting"
REPOSITORY = "ryanduguid/australian-accounting"
RELEASE_WORKFLOW = "release-aus-accounting-mcp.yml"
PYPI_ENVIRONMENT = "pypi-aus-accounting-mcp"
PYPI = "https://pypi.org"
_PROJECT_FIELD = re.compile(r'^(name|version)\s*=\s*"([^"]+)"\s*$', re.M)


class PreflightError(Exception):
    """One named mismatch; the first one found stops the run."""


def project_metadata(pyproject: str) -> tuple[str, str]:
    """The distribution name and static version from the [project] table."""
    match = re.search(r"(?ms)^\[project\]\n(.*?)(?=^\[|\Z)", pyproject)
    if match is None:
        raise PreflightError("pyproject.toml has no [project] table")
    fields = dict(_PROJECT_FIELD.findall(match.group(1)))
    if "name" not in fields or "version" not in fields:
        raise PreflightError("pyproject.toml [project] needs a static name and version")
    return fields["name"], fields["version"]


def normalise(name: str) -> str:
    return re.sub(r"[-_.]+", "-", name).lower()


def registry_package(server: dict[str, object], name: str, version: str) -> tuple[str, str]:
    """Bind server.json to the source tree; return the PyPI identifier and version."""
    if server.get("name") != SERVER_NAME:
        raise PreflightError(f"server.json name {server.get('name')!r} != {SERVER_NAME!r}")
    if server.get("version") != version:
        raise PreflightError(
            f"server.json version {server.get('version')!r} != pyproject {version!r}"
        )
    packages = server.get("packages")
    if not isinstance(packages, list) or len(packages) != 1 or not isinstance(packages[0], dict):
        raise PreflightError("server.json must list exactly one package")
    package = packages[0]
    identifier = package.get("identifier")
    if package.get("registryType") != "pypi" or not isinstance(identifier, str):
        raise PreflightError("server.json package must be a PyPI package with an identifier")
    if normalise(identifier) != normalise(name):
        raise PreflightError(f"server.json package {identifier!r} != pyproject {name!r}")
    if package.get("version") != version:
        raise PreflightError(
            f"server.json package version {package.get('version')!r} != {version!r}"
        )
    return identifier, version


def _files(release: dict[str, object]) -> tuple[str, str]:
    urls = release.get("urls")
    if not isinstance(urls, list):
        raise PreflightError("PyPI release has no file list")
    wheels = [u for u in urls if isinstance(u, dict) and u.get("packagetype") == "bdist_wheel"]
    sdists = [u for u in urls if isinstance(u, dict) and u.get("packagetype") == "sdist"]
    if len(wheels) != 1 or len(sdists) != 1:
        raise PreflightError(
            f"PyPI release has {len(wheels)} wheel(s) and {len(sdists)} sdist(s), not 1 each"
        )
    wheel, sdist = wheels[0].get("filename"), sdists[0].get("filename")
    if not isinstance(wheel, str) or not isinstance(sdist, str):
        raise PreflightError("PyPI release files have no filenames")
    return wheel, sdist


def verify_pypi(
    fetch_json: Callable[[str], object], identifier: str, version: str
) -> tuple[str, str]:
    """The exact version must be published with the ownership marker and both files."""
    release = fetch_json(f"{PYPI}/pypi/{identifier}/{version}/json")
    if not isinstance(release, dict):
        raise PreflightError(f"PyPI returned no release for {identifier} {version}")
    info = release.get("info")
    if not isinstance(info, dict):
        raise PreflightError("PyPI release has no info")
    if normalise(str(info.get("name"))) != normalise(identifier) or info.get("version") != version:
        raise PreflightError(
            f"PyPI returned {info.get('name')!r} {info.get('version')!r}, "
            f"not {identifier!r} {version!r}"
        )
    marker = f"mcp-name: {SERVER_NAME}"
    if marker not in str(info.get("description") or ""):
        raise PreflightError(f"the published README lacks the registry marker {marker!r}")
    return _files(release)


def verify_provenance(
    fetch_json: Callable[[str], object], identifier: str, version: str, filename: str
) -> None:
    """PyPI's provenance for the file must name this repository's release workflow."""
    record = fetch_json(f"{PYPI}/integrity/{identifier}/{version}/{filename}/provenance")
    bundles = record.get("attestation_bundles") if isinstance(record, dict) else None
    if not isinstance(bundles, list) or not bundles:
        raise PreflightError(f"{filename}: PyPI holds no provenance record")
    expected = {
        "kind": "GitHub",
        "repository": REPOSITORY,
        "workflow": RELEASE_WORKFLOW,
        "environment": PYPI_ENVIRONMENT,
    }
    for bundle in bundles:
        publisher = bundle.get("publisher") if isinstance(bundle, dict) else None
        if not isinstance(publisher, dict):
            raise PreflightError(f"{filename}: provenance bundle has no publisher")
        for field, value in expected.items():
            if publisher.get(field) != value:
                raise PreflightError(
                    f"{filename}: provenance publisher {field} "
                    f"{publisher.get(field)!r} != {value!r}"
                )
        if not bundle.get("attestations"):
            raise PreflightError(f"{filename}: provenance bundle carries no attestation")


def run(root: Path, fetch_json: Callable[[str], object]) -> list[str]:
    """Every check, in order; returns the lines a passing run prints."""
    name, version = project_metadata((root / "pyproject.toml").read_text(encoding="utf-8"))
    server_bytes = (root / "server.json").read_bytes()
    server = json.loads(server_bytes)
    if not isinstance(server, dict):
        raise PreflightError("server.json is not an object")
    identifier, version = registry_package(server, name, version)
    wheel, sdist = verify_pypi(fetch_json, identifier, version)
    for filename in (wheel, sdist):
        verify_provenance(fetch_json, identifier, version, filename)
    digest = hashlib.sha256(server_bytes).hexdigest()
    return [
        f"server.json names {SERVER_NAME} -> PyPI {identifier} {version}, matching pyproject.toml",
        f"PyPI holds {wheel} and {sdist} with the mcp-name marker",
        f"provenance for both files names {REPOSITORY} {RELEASE_WORKFLOW} ({PYPI_ENVIRONMENT})",
        f"server-json-sha256={digest}",
    ]


def _fetch_json(url: str) -> object:
    request = urllib.request.Request(url, headers={"Accept": "application/json"})
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            return json.load(response)
    except urllib.error.HTTPError as error:
        raise PreflightError(f"{url}: HTTP {error.code}") from error
    except (urllib.error.URLError, ValueError) as error:
        raise PreflightError(f"{url}: {error}") from error


def main(root: Path | None = None, fetch_json: Callable[[str], object] = _fetch_json) -> int:
    try:
        lines = run(root or Path.cwd(), fetch_json)
    except (PreflightError, OSError, json.JSONDecodeError) as error:
        print(f"FAIL {error}", file=sys.stderr)
        return 1
    for line in lines:
        print(line)
    output = os.environ.get("GITHUB_OUTPUT")
    if output:
        with open(output, "a", encoding="utf-8") as handle:
            handle.write(lines[-1] + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
