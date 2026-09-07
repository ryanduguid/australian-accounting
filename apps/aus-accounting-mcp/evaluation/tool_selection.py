"""Score whether a model picks the right tool, not just whether it gets the answer.

The pytest suite replays `questions.xml` through a real stdio session and checks
the answers reproduce. That measures the server, not the model: the replay is
told which tool to call. This measures the other half.

Run it in three steps, none of which contacts a network or a model:

    python evaluation/tool_selection.py context     # what a model is given
    python evaluation/tool_selection.py questions   # the ten questions, no answers
    python evaluation/tool_selection.py score runs/claude.json

`context` prints exactly what an MCP host puts in front of a model before it
chooses: the server instructions from initialization, and every tool's name,
title, description and inputs. `questions` prints the questions with the answer
and tool elements stripped, so nothing in the prompt hints at the selection.

Put those in front of the client under test, record which tools it called, and
write a JSON object mapping each question id to the tool names it selected, in
any order:

    {"catalogue-pages": ["list_ato_benchmark_industries"], ...}

`score` compares that against the selection published in `questions.xml`, which
the test suite holds to what the replay actually calls. It reports, per question,
the tools that were missed and the ones called that the answer does not need.

This is a supplementary check and deliberately not a CI gate. Steps one and two
are deterministic; the step in the middle is a model, and a gate whose result
depends on one would fail for reasons that are not this repository's.
"""

from __future__ import annotations

import argparse
import asyncio
import json
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client

QUESTIONS = Path(__file__).resolve().parent / "questions.xml"


def _published() -> dict[str, list[str]]:
    """The tool selection each question needs, from the checked answer key."""
    root = ET.parse(QUESTIONS).getroot()
    return {
        pair.attrib["id"]: sorted({tool.text or "" for tool in pair.findall("tools/tool")})
        for pair in root.findall("qa_pair")
    }


def _questions() -> dict[str, str]:
    root = ET.parse(QUESTIONS).getroot()
    return {
        pair.attrib["id"]: " ".join((pair.findtext("question") or "").split())
        for pair in root.findall("qa_pair")
    }


async def _describe() -> str:
    parameters = StdioServerParameters(
        command=sys.executable, args=["-m", "aus_accounting_mcp.cli"]
    )
    async with stdio_client(parameters) as (reader, writer):
        async with ClientSession(reader, writer) as session:
            initialized = await session.initialize()
            tools = (await session.list_tools()).tools

    lines = ["# Server instructions", "", initialized.instructions or "", "", "# Tools", ""]
    for tool in tools:
        required = tool.input_schema.get("required") or []
        optional = [
            name for name in tool.input_schema.get("properties", {}) if name not in required
        ]
        lines.append(f"## {tool.name}" + (f" ({tool.title})" if tool.title else ""))
        lines.append("")
        lines.append((tool.description or "").strip())
        lines.append("")
        lines.append(f"required: {', '.join(required) or 'none'}")
        lines.append(f"optional: {', '.join(optional) or 'none'}")
        lines.append("")
    return "\n".join(lines)


def _score(recorded: dict[str, list[str]]) -> int:
    published = _published()
    unknown = sorted(set(recorded) - set(published))
    if unknown:
        print(f"not questions in this evaluation: {', '.join(unknown)}", file=sys.stderr)
        return 2

    exact = 0
    width = max(len(case) for case in published)
    for case, expected in published.items():
        if case not in recorded:
            print(f"{case.ljust(width)}  NOT ANSWERED")
            continue
        selected = sorted(set(recorded[case]))
        missed = [name for name in expected if name not in selected]
        extra = [name for name in selected if name not in expected]
        if not missed and not extra:
            exact += 1
            print(f"{case.ljust(width)}  exact")
            continue
        detail = []
        if missed:
            detail.append(f"missed {', '.join(missed)}")
        if extra:
            detail.append(f"also called {', '.join(extra)}")
        print(f"{case.ljust(width)}  {'; '.join(detail)}")

    print(f"\n{exact} of {len(published)} questions selected exactly the published tools.")
    # A reported score is the whole output. Exiting non-zero on an imperfect run
    # would turn a measurement into a gate, which is what this deliberately is not.
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = parser.add_subparsers(dest="command", required=True)
    sub.add_parser("context", help="print the instructions and tools a model is given")
    sub.add_parser("questions", help="print the questions with answers and tools stripped")
    scorer = sub.add_parser("score", help="score recorded tool selections")
    scorer.add_argument("recorded", type=Path, help="JSON of {question id: [tool names]}")
    args = parser.parse_args(argv)

    if args.command == "context":
        print(asyncio.run(_describe()))
        return 0
    if args.command == "questions":
        for case, question in _questions().items():
            print(f"{case}: {question}")
        return 0
    return _score(json.loads(args.recorded.read_text(encoding="utf-8")))


if __name__ == "__main__":
    raise SystemExit(main())
