from __future__ import annotations

import asyncio
import importlib.metadata
import json
from pathlib import Path

from atobenchmark.dataset import load
from paydaysuper import LAW_CONTENT_DATE

from aus_accounting_mcp.server import mcp

CANONICAL_REPOSITORY = "https://github.com/ryanduguid/aus-accounting-mcp"


def _call(name: str, arguments: dict[str, str]) -> dict[str, object]:
    result = asyncio.run(mcp.call_tool(name, arguments))
    assert isinstance(result.structured_content, dict)
    return result.structured_content


def test_compatibility_record_matches_published_server_and_engine_owned_fields() -> None:
    root = Path(__file__).resolve().parents[1]
    record = json.loads((root / "compatibility.json").read_text(encoding="utf-8"))
    server_metadata = json.loads((root / "server.json").read_text(encoding="utf-8"))
    assert record["schema_version"] == 1
    assert server_metadata["version"] == server_metadata["packages"][0]["version"]
    assert record["server"] == {
        "distribution": "aus-accounting-mcp",
        "version": server_metadata["version"],
        "repository": CANONICAL_REPOSITORY,
        "pypi": "https://pypi.org/project/aus-accounting-mcp/0.1.5/",
        "registry_identity": "io.github.ryanduguid/aus-accounting",
        "registry": (
            "https://registry.modelcontextprotocol.io/v0.1/servers/"
            "io.github.ryanduguid%2Faus-accounting/versions/0.1.5"
        ),
        "release": f"{CANONICAL_REPOSITORY}/releases/tag/v0.1.5",
    }
    assert record["engines"] == [
        {
            "distribution": "ato-benchmark-compare",
            "version": "0.1.4",
            "repository": "https://github.com/ryanduguid/ato-benchmark-compare",
            "release": "https://github.com/ryanduguid/ato-benchmark-compare/releases/tag/v0.1.4",
        },
        {
            "distribution": "payday-super-checker",
            "version": "0.1.2",
            "repository": "https://github.com/ryanduguid/payday-super-checker",
            "release": "https://github.com/ryanduguid/payday-super-checker/releases/tag/v0.1.2",
        },
    ]
    distribution = record["server"]["distribution"]
    assert importlib.metadata.version(distribution) == "0.1.6"
    requirements = set(importlib.metadata.requires(record["server"]["distribution"]) or [])
    for engine in record["engines"]:
        assert importlib.metadata.version(engine["distribution"]) == engine["version"]
        assert f"{engine['distribution']}=={engine['version']}" in requirements
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
    assert benchmark["engine"] == "ato-benchmark-compare"
    assert benchmark["engine_version"] == record["engines"][0]["version"]
    assert benchmark["source"] == dict(load().source)
    assert payday["engine"] == "payday-super-checker"
    assert payday["engine_version"] == record["engines"][1]["version"]
    assert payday["law_content_date"] == LAW_CONTENT_DATE
    assert "law_content_date" not in json.dumps(record)
    assert '"source"' not in json.dumps(record)
