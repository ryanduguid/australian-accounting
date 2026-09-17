"""The registry preflight: what it must refuse before mcp-publisher ever runs."""

from __future__ import annotations

import importlib.util
import json
import os
import re
import shutil
import subprocess
from pathlib import Path

import pytest

from repository_root import repository_root

APP = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    "registry_preflight", APP / "scripts" / "registry_preflight.py"
)
assert SPEC is not None and SPEC.loader is not None
preflight = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(preflight)

NAME = "aus-accounting-mcp"
VERSION = "9.9.9"
WHEEL = "aus_accounting_mcp-9.9.9-py3-none-any.whl"
SDIST = "aus_accounting_mcp-9.9.9.tar.gz"
PYPROJECT = (
    f'[build-system]\nrequires = ["x"]\n\n[project]\nname = "{NAME}"\nversion = "{VERSION}"\n'
)


def server(**overrides: object) -> dict[str, object]:
    document: dict[str, object] = {
        "name": preflight.SERVER_NAME,
        "version": VERSION,
        "packages": [
            {
                "registryType": "pypi",
                "identifier": NAME,
                "version": VERSION,
                "transport": {"type": "stdio"},
            }
        ],
    }
    document.update(overrides)
    return document


def package(**overrides: object) -> dict[str, object]:
    base: dict[str, object] = {
        "registryType": "pypi",
        "identifier": NAME,
        "version": VERSION,
        "transport": {"type": "stdio"},
    }
    base.update(overrides)
    return base


def bundles(
    publisher: dict[str, object], attestations: list[object] | None = None
) -> dict[str, object]:
    return {
        "attestation_bundles": [
            {"publisher": publisher, "attestations": [1] if attestations is None else attestations}
        ]
    }


def publisher(**overrides: object) -> dict[str, object]:
    base: dict[str, object] = {
        "kind": "GitHub",
        "repository": preflight.REPOSITORY,
        "workflow": preflight.RELEASE_WORKFLOW,
        "environment": preflight.PYPI_ENVIRONMENT,
    }
    base.update(overrides)
    return base


class FakePyPI:
    """Serves a fabricated release and provenance; records every URL asked for."""

    def __init__(self) -> None:
        self.release: dict[str, object] = {
            "info": {
                "name": NAME,
                "version": VERSION,
                "description": f"Docs\n\n<!-- mcp-name: {preflight.SERVER_NAME} -->\n",
            },
            "urls": [
                {"packagetype": "bdist_wheel", "filename": WHEEL},
                {"packagetype": "sdist", "filename": SDIST},
            ],
        }
        self.provenance: dict[str, object] = {
            "attestation_bundles": [{"publisher": publisher(), "attestations": [{"x": 1}]}]
        }
        self.missing: set[str] = set()
        self.urls: list[str] = []

    def __call__(self, url: str) -> object:
        self.urls.append(url)
        if url in self.missing:
            raise preflight.PreflightError(f"{url}: HTTP 404")
        if url.endswith("/provenance"):
            return self.provenance
        return self.release


def write_tree(tmp_path: Path, document: dict[str, object] | None = None) -> Path:
    (tmp_path / "pyproject.toml").write_text(PYPROJECT, encoding="utf-8")
    (tmp_path / "server.json").write_text(
        json.dumps(document if document is not None else server()), encoding="utf-8"
    )
    return tmp_path


def test_the_committed_server_json_binds_to_the_committed_pyproject() -> None:
    name, version = preflight.project_metadata((APP / "pyproject.toml").read_text(encoding="utf-8"))
    document = json.loads((APP / "server.json").read_text(encoding="utf-8"))
    assert preflight.registry_package(document, name, version) == (NAME, version)


def test_a_complete_release_passes_and_reports_the_server_json_digest(tmp_path: Path) -> None:
    root = write_tree(tmp_path)
    pypi = FakePyPI()
    lines = preflight.run(root, pypi)
    assert lines[0].startswith(f"server.json names {preflight.SERVER_NAME} -> PyPI {NAME}")
    assert re.fullmatch(r"server-json-sha256=[0-9a-f]{64}", lines[-1])
    assert pypi.urls == [
        f"https://pypi.org/pypi/{NAME}/{VERSION}/json",
        f"https://pypi.org/integrity/{NAME}/{VERSION}/{WHEEL}/provenance",
        f"https://pypi.org/integrity/{NAME}/{VERSION}/{SDIST}/provenance",
    ]


def test_an_absent_package_version_fails_before_any_provenance_lookup(tmp_path: Path) -> None:
    root = write_tree(tmp_path)
    pypi = FakePyPI()
    pypi.missing.add(f"https://pypi.org/pypi/{NAME}/{VERSION}/json")
    with pytest.raises(preflight.PreflightError, match="HTTP 404"):
        preflight.run(root, pypi)
    assert len(pypi.urls) == 1


@pytest.mark.parametrize(
    "document, message",
    [
        (server(name="io.github.someone/else"), "server.json name"),
        (server(version="9.9.8"), "server.json version"),
        (server(packages=[]), "exactly one package"),
        (server(packages=[package(registryType="npm")]), "PyPI package"),
        (server(packages=[package(identifier="other-mcp")]), "server.json package 'other-mcp'"),
        (server(packages=[package(version="9.9.8")]), "package version"),
    ],
)
def test_identity_and_version_mismatches_fail(
    tmp_path: Path, document: dict[str, object], message: str
) -> None:
    root = write_tree(tmp_path, document)
    pypi = FakePyPI()
    with pytest.raises(preflight.PreflightError, match=message):
        preflight.run(root, pypi)
    assert pypi.urls == []


def test_pypi_metadata_that_disagrees_with_server_json_fails(tmp_path: Path) -> None:
    root = write_tree(tmp_path)
    for info, message in (
        ({"name": "other", "version": VERSION, "description": "x"}, "PyPI returned"),
        ({"name": NAME, "version": "9.9.8", "description": "x"}, "PyPI returned"),
        ({"name": NAME, "version": VERSION, "description": "no marker"}, "registry marker"),
    ):
        pypi = FakePyPI()
        pypi.release["info"] = info
        with pytest.raises(preflight.PreflightError, match=message):
            preflight.run(root, pypi)
    pypi = FakePyPI()
    pypi.release["urls"] = [{"packagetype": "bdist_wheel", "filename": WHEEL}]
    with pytest.raises(preflight.PreflightError, match="1 wheel\\(s\\) and 0 sdist\\(s\\)"):
        preflight.run(root, pypi)


def test_provenance_verification_failures_fail(tmp_path: Path) -> None:
    root = write_tree(tmp_path)
    cases: list[tuple[dict[str, object], str]] = [
        ({"attestation_bundles": []}, "no provenance record"),
        ({}, "no provenance record"),
        ({"attestation_bundles": [{"attestations": [1]}]}, "no publisher"),
        (bundles(publisher(repository="x/y")), "repository 'x/y'"),
        (bundles(publisher(workflow="other.yml")), "workflow 'other.yml'"),
        (bundles(publisher(environment="pypi")), "environment 'pypi'"),
        (bundles(publisher(kind="GitLab")), "kind 'GitLab'"),
        (bundles(publisher(), attestations=[]), "no attestation"),
    ]
    for provenance, message in cases:
        pypi = FakePyPI()
        pypi.provenance = provenance
        with pytest.raises(preflight.PreflightError, match=message):
            preflight.run(root, pypi)


def test_main_reports_failures_on_stderr_and_writes_the_digest_output(
    tmp_path: Path, capsys: pytest.CaptureFixture[str], monkeypatch: pytest.MonkeyPatch
) -> None:
    root = write_tree(tmp_path)
    output = tmp_path / "output.txt"
    monkeypatch.setenv("GITHUB_OUTPUT", str(output))
    assert preflight.main(root, FakePyPI()) == 0
    assert re.fullmatch(r"server-json-sha256=[0-9a-f]{64}\n", output.read_text(encoding="utf-8"))

    pypi = FakePyPI()
    pypi.missing.add(f"https://pypi.org/pypi/{NAME}/{VERSION}/json")
    assert preflight.main(root, pypi) == 1
    assert "FAIL" in capsys.readouterr().err

    (root / "server.json").write_text("{not json", encoding="utf-8")
    assert preflight.main(root, FakePyPI()) == 1


def test_publication_is_gated_by_the_preflight_the_main_ref_and_an_environment() -> None:
    workflow = (repository_root() / ".github" / "workflows" / "publish-mcp.yml").read_text(
        encoding="utf-8"
    )
    preflight_job = workflow[workflow.index("\n  preflight:") : workflow.index("\n  publish:")]
    publish_job = workflow[workflow.index("\n  publish:") :]

    assert 'test "$GITHUB_REF" = "refs/heads/main"' in preflight_job
    assert "python scripts/registry_preflight.py" in preflight_job
    assert "id-token" not in preflight_job
    assert "mcp-publisher" not in preflight_job

    assert "needs: preflight" in publish_job
    assert "if: github.ref == 'refs/heads/main'" in publish_job
    assert "environment:\n      name: mcp-registry" in publish_job
    assert "EXPECTED_SHA256: ${{ needs.preflight.outputs.server-json-sha256 }}" in publish_job
    assert publish_job.index("sha256sum --check") < publish_job.index("Install mcp-publisher")
    assert workflow.count("id-token: write") == 1


def posix_bash() -> str | None:
    """A real POSIX bash, never the WSL launcher.

    On Windows, PATH resolves `bash` to C:\\Windows\\System32\\bash.exe, which
    starts WSL. On a runner with no distribution installed that exits 1 with a
    UTF-16LE error before the script under test runs, so prefer Git Bash and
    refuse the System32 stub.
    """
    if os.name == "nt":
        for variable in ("ProgramFiles", "ProgramW6432", "ProgramFiles(x86)"):
            root = os.environ.get(variable)
            if not root:
                continue
            candidate = Path(root) / "Git" / "bin" / "bash.exe"
            if candidate.is_file():
                return str(candidate)
    found = shutil.which("bash")
    if found and os.name == "nt" and Path(found).parent.name.lower() == "system32":
        return None
    return found


@pytest.mark.parametrize(
    "ref, expected",
    [("refs/heads/main", 0), ("refs/heads/feature", 1), ("refs/tags/aus-accounting-mcp/v0.2.2", 1)],
)
def test_the_ref_guard_admits_only_main(ref: str, expected: int, tmp_path: Path) -> None:
    workflow = (repository_root() / ".github" / "workflows" / "publish-mcp.yml").read_text(
        encoding="utf-8"
    )
    lines = workflow.splitlines()
    marker = 'test "$GITHUB_REF" = "refs/heads/main"'
    first = next(i for i, line in enumerate(lines) if marker in line)
    guard = chr(10).join(line.strip() for line in lines[first : first + 2])
    bash = posix_bash()
    if bash is None:
        pytest.skip("no POSIX bash available; System32 bash.exe only launches WSL")
    preflight_job = workflow[workflow.index("\n  preflight:") : workflow.index("\n  publish:")]
    defaults, steps = preflight_job.split("    steps:", 1)
    guard_step = steps.split("      - uses:", 1)[0]
    directory = re.search(r"^        working-directory: (.+)$", guard_step, re.M)
    if directory is None:
        directory = re.search(r"^        working-directory: (.+)$", defaults, re.M)
    # The guard runs before checkout, so only the empty workspace exists.
    working_directory = tmp_path / directory.group(1) if directory else tmp_path
    completed = subprocess.run(
        [bash, "-euo", "pipefail", "-c", guard],
        cwd=working_directory,
        env={**os.environ, "GITHUB_REF": ref},
        capture_output=True,
        text=True,
        check=False,
    )
    assert completed.returncode == expected, completed.stderr
