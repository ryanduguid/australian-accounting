from __future__ import annotations

import asyncio
import importlib.metadata
import json
import tomllib
from pathlib import Path

from atobenchmark.dataset import load
from div7aloan import __version__ as DIV7A_VERSION
from paydaysuper import LAW_CONTENT_DATE

from aus_accounting_mcp.server import mcp

CANONICAL_REPOSITORY = "https://github.com/ryanduguid/australian-accounting"


def _call(name: str, arguments: dict[str, str]) -> dict[str, object]:
    result = asyncio.run(mcp.call_tool(name, arguments))
    assert isinstance(result.structured_content, dict)
    return result.structured_content


def test_compatibility_record_matches_published_server_and_engine_owned_fields() -> None:
    root = Path(__file__).resolve().parents[1]
    record = json.loads((root / "compatibility.json").read_text(encoding="utf-8"))
    server_metadata = json.loads((root / "server.json").read_text(encoding="utf-8"))
    assert record["schema_version"] == 1
    version = server_metadata["version"]
    assert version == server_metadata["packages"][0]["version"]
    assert record["server"] == {
        "distribution": "aus-accounting-mcp",
        "version": version,
        "repository": CANONICAL_REPOSITORY,
        "pypi": f"https://pypi.org/project/aus-accounting-mcp/{version}/",
        "registry_identity": "io.github.ryanduguid/aus-accounting",
        "registry": (
            "https://registry.modelcontextprotocol.io/v0.1/servers/"
            f"io.github.ryanduguid%2Faus-accounting/versions/{version}"
        ),
        "release": f"{CANONICAL_REPOSITORY}/releases/tag/aus-accounting-mcp/v{version}",
    }
    # Each engine's version is held to the exact pin in pyproject.toml below.
    assert [engine["distribution"] for engine in record["engines"]] == [
        "ato-benchmark-compare",
        "div7a-loan-review",
        "payday-super-checker",
        "australian-tax-calculators",
    ]
    for engine in record["engines"]:
        name, engine_version = engine["distribution"], engine["version"]
        assert engine == {
            "distribution": name,
            "version": engine_version,
            "repository": f"{CANONICAL_REPOSITORY}/tree/main/packages/{name}",
            "release": f"{CANONICAL_REPOSITORY}/releases/tag/{name}/v{engine_version}",
        }
    distribution = record["server"]["distribution"]
    project = tomllib.loads((root / "pyproject.toml").read_text(encoding="utf-8"))["project"]
    assert importlib.metadata.version(distribution) == project["version"]
    requirements = set(importlib.metadata.requires(record["server"]["distribution"]) or [])
    runtime_versions = {}
    for engine in record["engines"]:
        name = engine["distribution"]
        engine_version = importlib.metadata.version(name)
        runtime_versions[name] = engine_version
        # Workspace sources can be ahead of the published pins. Keep the
        # published record consistent and check runtime output separately below.
        published_pin = f"{name}=={engine['version']}"
        assert published_pin in project["dependencies"]
        assert published_pin in requirements
    benchmark = _call(
        "list_ato_benchmark_industries",
        {"search": "baker"},
    )
    payday = _call(
        "calc_payday_super_deadline",
        {
            "qe_day": "2026-08-06",
            "sg_amount": "800.00",
            "received": "2026-08-10",
            "as_at": "2026-08-21",
        },
    )
    div7a = _call("get_div7a_benchmark_rate", {"year_of_income": "2025-26"})
    assert benchmark["engine"] == "ato-benchmark-compare"
    assert benchmark["engine_version"] == runtime_versions["ato-benchmark-compare"]
    assert benchmark["source"] == dict(load().source)
    assert payday["engine"] == "payday-super-checker"
    assert payday["engine_version"] == runtime_versions["payday-super-checker"]
    assert payday["law_content_date"] == LAW_CONTENT_DATE
    assert div7a["engine"] == "div7a-loan-review"
    assert div7a["engine_version"] == DIV7A_VERSION == runtime_versions["div7a-loan-review"]
    assert "law_content_date" not in json.dumps(record)
    assert '"source"' not in json.dumps(record)
