from __future__ import annotations

from pathlib import Path
import re

import pytest


ROOT = Path(__file__).resolve().parents[1]


def _repository_root() -> Path:
    # The package lives in apps/aus-accounting-mcp; the workflows these tests audit
    # live in the repository root .github directory above it.
    package_root = Path(__file__).resolve().parents[1]
    for candidate in (package_root, *package_root.parents):
        if (candidate / ".github" / "workflows").is_dir():
            return candidate
    pytest.skip("repository-root policy is not shipped in the Python source distribution")

EXPECTED_POLICY = """\
# Agent instructions

This package is an MCP facade. Keep statutory calculations in the delegated
`payday-super-checker`, `ato-benchmark-compare` and `div7a-loan-review` engines;
adapters may validate, translate and serialise, but must not reimplement their law
or datasets.

- Keep Division 7A limited to the delegated engine's reviewed s 109N and s 109E scope;
  refuse unsupported matters.
- Keep repository fixtures and demonstrations synthetic-only; never add client data or
  present a fixture as a lodgment.
- Route all MCP-boundary money parsing through `aus_accounting_mcp.money`; preserve
  finite decimal strings and the domain limits.
- Never invent current rates, thresholds, law dates, source dates or missing facts.
  Mutable facts and citations remain owned by the delegated engines and official sources.
- Preserve visible warnings, refusals, engine versions, no-advice language and the need
  for human review before consequential accounting action.
"""

MEDIA_COMMAND = (
    "uv run --locked --extra dev python scripts/render_demo_image.py "
    "docs/quick-proof.txt docs/quick-proof.webp"
)

SUPPLEMENTARY_COMMANDS = [
    "uv sync --locked --extra dev",
    "uv run --locked --extra dev python -m build",
    "uv run --locked aus-accounting-mcp-demo",
    MEDIA_COMMAND,
    (
        "uv run --locked --extra dev pytest -q tests/test_demo.py "
        "tests/test_demo_media.py tests/test_compatibility.py tests/test_engine_versions.py"
    ),
]


def _ci_run_commands() -> list[str]:
    workflow = (_repository_root() / ".github" / "workflows" / "ci.yml").read_text(
        encoding="utf-8"
    )
    commands = re.findall(r"^\s+(?:-\s+)?run:\s*(\S.*)$", workflow, flags=re.MULTILINE)
    return list(dict.fromkeys(commands))


def _section(document: str, heading: str) -> str:
    match = re.search(
        rf"^## {re.escape(heading)}\s*$\n(.*?)(?=^## |\Z)",
        document,
        flags=re.MULTILINE | re.DOTALL,
    )
    assert match is not None, f"missing ## {heading} section"
    return match.group(1)


def _fenced_commands(section: str) -> list[str]:
    blocks = re.findall(r"```(?:bash|powershell)\n(.*?)```", section, flags=re.DOTALL)
    return [line for block in blocks for line in block.splitlines() if line]


def _normalise(document: str) -> str:
    return re.sub(r"\s+", " ", document).strip()


def _without_fenced_commands(section: str) -> str:
    return re.sub(r"```(?:bash|powershell)\n.*?```", "", section, flags=re.DOTALL)


def test_agents_preserves_the_mcp_domain_boundaries() -> None:
    guidance = (ROOT / "AGENTS.md").read_text(encoding="utf-8")
    policy, separator, _remainder = guidance.partition("## CI gates")

    assert separator
    assert _normalise(policy) == _normalise(EXPECTED_POLICY)


def test_agents_tracks_exact_ci_commands_and_classifies_other_checks() -> None:
    guidance = (ROOT / "AGENTS.md").read_text(encoding="utf-8")

    ci = _section(guidance, "CI gates")
    assert _fenced_commands(ci) == _ci_run_commands()
    assert _normalise(_without_fenced_commands(ci)) == _normalise(
        "These are the current commands in `.github/workflows/ci.yml`:"
    )


def test_agents_pins_repository_backed_supplementary_commands_as_non_ci() -> None:
    guidance = (ROOT / "AGENTS.md").read_text(encoding="utf-8")
    supplementary = _section(guidance, "Supplementary local and release-readiness checks")
    pyproject = (ROOT / "pyproject.toml").read_text(encoding="utf-8")
    readme = (ROOT / "README.md").read_text(encoding="utf-8")

    assert (ROOT / "uv.lock").is_file()
    assert 'dev = [' in pyproject
    assert '"build>=1.2"' in pyproject
    assert '"pytest>=8.0.0"' in pyproject
    assert '"Pillow==12.3.0"' in pyproject
    assert 'aus-accounting-mcp-demo = "aus_accounting_mcp.demo:main"' in pyproject
    assert "uv run --locked aus-accounting-mcp-demo" in readme
    assert f"`{MEDIA_COMMAND}`" in readme
    for path in (
        "docs/quick-proof.webp",
        "tests/test_demo.py",
        "tests/test_demo_media.py",
        "tests/test_compatibility.py",
        "tests/test_engine_versions.py",
    ):
        assert (ROOT / path).is_file()

    assert _fenced_commands(supplementary) == SUPPLEMENTARY_COMMANDS
    assert _normalise(_without_fenced_commands(supplementary)) == _normalise(
        """\
        These checks are not CI gates. Use them when their affected artifact changes:

        Keep `docs/quick-proof.txt` as the accessible source of truth for
        `docs/quick-proof.webp`. Route publication through the existing release workflows;
        do not publish, tag, or change public metadata without explicit approval.
        """
    )


def test_runtime_entry_points_and_contribution_scope_are_explicit() -> None:
    assert (ROOT / "CLAUDE.md").read_text(encoding="utf-8") == "@AGENTS.md\n"

    contributing = re.sub(
        r"\s+",
        " ",
        (ROOT / "CONTRIBUTING.md").read_text(encoding="utf-8"),
    )
    required = (
        "adapter",
        "delegated engines",
        "compatibility.json",
        "server.json",
        "docs/quick-proof.txt",
        "docs/quick-proof.webp",
        "release workflow",
        "Publish to PyPI",
        "Publish to MCP Registry",
    )
    for text in required:
        assert text in contributing
