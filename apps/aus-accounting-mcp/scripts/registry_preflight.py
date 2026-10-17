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

import argparse
import hashlib
import json
import os
import re
import sys
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Callable, cast

SERVER_NAME = "io.github.ryanduguid/aus-accounting"
REPOSITORY = "ryanduguid/australian-accounting"
RELEASE_WORKFLOW = "release-aus-accounting-mcp.yml"
PYPI_ENVIRONMENT = "pypi-aus-accounting-mcp"
PYPI = "https://pypi.org"
REGISTRY = "https://registry.modelcontextprotocol.io/v0.1/servers"
_PROJECT_FIELD = re.compile(r'^(name|version)\s*=\s*"([^"]+)"\s*$', re.M)


class PreflightError(Exception):
    """One named mismatch; the first one found stops the run."""


def project_metadata(pyproject: str) -> tuple[str, str]:
    """The distribution name and static version from the [project] table."""
    match = re.search(r"(?ms)^\[project\]\n(.*?)(?=(?:^\[)|\Z)", pyproject)
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


def verify_readback(
    fetch_json: Callable[[str], object], server: dict[str, object], name: str, version: str
) -> list[str]:
    """After publication, the active latest Registry record and release must agree."""
    query = urllib.parse.urlencode({"search": SERVER_NAME, "version": "latest"})
    response = fetch_json(f"{REGISTRY}?{query}")
    entries = response.get("servers") if isinstance(response, dict) else None
    if not isinstance(entries, list):
        raise PreflightError("Registry response has no server list")
    matches = [
        entry
        for entry in entries
        if isinstance(entry, dict)
        and isinstance(entry.get("server"), dict)
        and entry["server"].get("name") == SERVER_NAME
    ]
    if len(matches) != 1:
        raise PreflightError(f"Registry returned {len(matches)} exact-name records, not 1")
    entry = matches[0]
    registered = entry["server"]
    registry_package(registered, name, version)
    expected_repository = f"https://github.com/{REPOSITORY}"
    for document in (server, registered):
        repository = document.get("repository")
        if not isinstance(repository, dict) or repository.get("url") != expected_repository:
            raise PreflightError("Registry and server.json must name the expected repository")
        packages = cast(list[dict[str, object]], document["packages"])
        transport = packages[0].get("transport")
        if not isinstance(transport, dict) or transport.get("type") != "stdio":
            raise PreflightError("Registry and server.json must both use stdio transport")
    metadata = entry.get("_meta")
    official = (
        metadata.get("io.modelcontextprotocol.registry/official")
        if isinstance(metadata, dict)
        else None
    )
    if not isinstance(official, dict) or official.get("status") != "active":
        raise PreflightError("Registry record is not active")
    if official.get("isLatest") is not True:
        raise PreflightError("Registry record is not marked latest")
    tag = f"{name}/v{version}"
    release = fetch_json(
        f"https://api.github.com/repos/{REPOSITORY}/releases/tags/"
        f"{urllib.parse.quote(tag, safe='')}"
    )
    if (
        not isinstance(release, dict)
        or release.get("tag_name") != tag
        or release.get("draft") is not False
        or release.get("prerelease") is not False
        or not isinstance(release.get("published_at"), str)
        or not release["published_at"]
    ):
        raise PreflightError(f"GitHub has no published stable release for {tag}")
    return [
        f"Registry latest is active: {SERVER_NAME} {version}, matching package/repository/stdio",
        f"GitHub holds the published stable release {tag}",
    ]


def run(
    root: Path, fetch_json: Callable[[str], object], *, readback: bool = False
) -> list[str]:
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
    published = verify_readback(fetch_json, server, name, version) if readback else []
    digest = hashlib.sha256(server_bytes).hexdigest()
    return [
        f"server.json names {SERVER_NAME} -> PyPI {identifier} {version}, matching pyproject.toml",
        f"PyPI holds {wheel} and {sdist} with the mcp-name marker",
        f"provenance for both files names {REPOSITORY} {RELEASE_WORKFLOW} ({PYPI_ENVIRONMENT})",
        *published,
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


def main(
    root: Path | None = None,
    fetch_json: Callable[[str], object] = _fetch_json,
    *,
    readback: bool = False,
) -> int:
    try:
        lines = run(root or Path.cwd(), fetch_json, readback=readback)
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
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--readback",
        action="store_true",
        help="after publication, also verify the active latest Registry entry and GitHub release",
    )
    raise SystemExit(main(readback=parser.parse_args().readback))
