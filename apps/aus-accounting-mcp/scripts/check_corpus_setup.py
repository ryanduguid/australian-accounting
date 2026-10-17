"""Replay the real rulings builder into MCP retrieval using one fabricated page."""

from __future__ import annotations

import argparse
import asyncio
import importlib
import json
import os
import sys
from pathlib import Path

from aus_accounting_mcp.server import mcp

DOCID = "TXR/TR20991/NAT/ATO/00001"


async def check(builder_root: Path, destination: Path) -> None:
    builder_root = builder_root.resolve(strict=True)
    destination = destination.resolve()
    if destination.exists():
        raise ValueError("destination exists; use a new directory to preserve earlier evidence")
    if not (builder_root / "fadden/rulings.py").is_file():
        raise ValueError("builder must name an au-tax-legislation-corpus source checkout")
    sys.path.insert(0, str(builder_root))
    builder = importlib.import_module("fadden.rulings")
    module_path = builder.__file__
    if module_path is None or Path(module_path).resolve() != builder_root / "fadden/rulings.py":
        raise ValueError("a different corpus builder is already imported")
    raw = (
        '<html><head><meta name="ato.reference.id" content="' + DOCID + '" />'
        '<meta name="DC.Title" content="TR 2099/1 - Synthetic setup ruling" />'
        '<meta name="dc.Type.documentType" content="ATO Ruling" />'
        '<meta name="dc.Date.Issued" content="2099/01/01" /></head><body>'
        '<div id="LawBody"><p><a name="P1"></a>1. Synthetic setup evidence only.</p>'
        '<p>' + builder.NOTICE + '</p></div></body></html>'
    ).encode("utf-8")

    def fabricated_page(docid: str) -> tuple[bytes, str]:
        if docid != DOCID:
            raise ValueError("unexpected synthetic document")
        return raw, "https://example.invalid/synthetic-setup-ruling"

    run = destination / "run-synthetic"
    result = builder.run([DOCID], run, fetcher=fabricated_page, spacing=0, today="2099-01-01")
    if result != 0:
        raise ValueError("the builder excluded the synthetic page; inspect its manifest")
    os.environ["AUS_ACCOUNTING_RULINGS_ROOT"] = str(destination)
    found = await mcp.call_tool("search_ato_rulings", {"query": "Synthetic setup evidence"})
    if found.is_error or not isinstance(found.structured_content, dict):
        raise ValueError("MCP search did not return structured evidence")
    matches = found.structured_content["matches"]
    if len(matches) != 1 or matches[0]["docid"] != DOCID:
        raise ValueError("MCP did not find the builder's single synthetic paragraph")
    read = await mcp.call_tool("read_ato_ruling", {"row_ref": matches[0]["row_ref"]})
    if read.is_error or not isinstance(read.structured_content, dict):
        raise ValueError("MCP could not read its cited result")
    report = {
        "synthetic": True,
        "network_requests": 0,
        "scope": "Fabricated HTML through the actual builder and registered MCP retrieval tools.",
        "limits": (
            "No live capture, current-law verification, model behaviour "
            "or publication is established."
        ),
        "search": found.structured_content,
        "read": read.structured_content,
    }
    with (destination / "setup-report.json").open("x", encoding="utf-8") as stream:
        json.dump(report, stream, indent=2)
        stream.write("\n")
    print(f"Synthetic builder/MCP setup passed; evidence: {destination / 'setup-report.json'}")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--builder", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    asyncio.run(check(args.builder, args.out))


if __name__ == "__main__":
    main()
