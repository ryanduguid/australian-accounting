"""Score recorded tool calls, supplied arguments and final answers.

The pytest suite replays `questions.xml` through a real stdio session and checks
the answers reproduce. That measures the server, not the model: the replay is
told which tool to call. This measures the other half.

Run it in 3 steps, none of which contacts a network or a model:

    python evaluation/tool_selection.py context     # what a model is given
    python evaluation/tool_selection.py questions   # questions without answers
    python evaluation/tool_selection.py score runs/claude.json

`context` prints the server instructions from initialisation, every tool's name,
title, description and complete input and output schemas, and the scope resource
preloaded for this evaluation. A host may expose resources differently; record
that difference when comparing model trials. The schemas are
printed whole rather than summarised, because most of what decides both the
selection and the arguments lives inside them. `questions` prints the questions
with the answer and tool elements stripped, so nothing in the prompt hints at
the selection.

Put those in front of the client under test and record each question's ordered
calls (name and arguments) and final answer:

    {"question-id": {"calls": [{"name": "tool", "arguments": {}}], "answer": "text"}}

`score` compares these with the reference calls and answers in `questions.xml`.
The stdio replay verifies the reference. Comparison is exact, including decimal
strings, omitted fields, call order and repetitions; only surrounding answer
whitespace is ignored. Equivalent alternative workflows may differ from this
reference, so a mismatch needs human review and is not proof of a bad answer.
Legacy lists of tool names remain accepted, with arguments and answers explicitly
marked NOT EVALUATED. No recording is executed by the scorer.

This is a supplementary check and deliberately not a CI gate. Steps one and 2
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
from typing import Any

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
            scope = await session.read_resource("aus-accounting://scope")

    lines = ["# Server instructions", "", initialized.instructions or "", "", "# Tools", ""]
    for tool in tools:
        # The whole schema, not a summary of it. A host hands the model the
        # tool definition as it stands, and most of what decides both the
        # selection and the arguments lives inside the schemas: which inputs
        # are required, that amounts are decimal strings within a documented
        # ceiling, that an omitted bucket is not an established zero, how the
        # pages of a listing continue. A run scored against a trimmed
        # definition would measure a server that is not this one.
        lines.append(f"## {tool.name}" + (f" ({tool.title})" if tool.title else ""))
        lines.append("")
        lines.append((tool.description or "").strip())
        lines.append("")
        lines.append("Input schema:")
        lines.append("")
        lines.append(json.dumps(tool.input_schema, indent=2, sort_keys=True))
        lines.append("")
        lines.append("Output schema:")
        lines.append("")
        lines.append(json.dumps(tool.output_schema, indent=2, sort_keys=True))
        lines.append("")
    lines.extend([
        "# Preloaded scope resource", "", "aus-accounting://scope", "",
        scope.contents[0].text, "",
    ])
    return "\n".join(lines)


def _validate_recording(recorded: Any) -> None:
    if not isinstance(recorded, dict):
        raise ValueError("recording must be an object keyed by question id")
    for case, entry in recorded.items():
        if isinstance(entry, list) and all(isinstance(name, str) for name in entry):
            continue
        if (
            not isinstance(entry, dict) or set(entry) != {"calls", "answer"}
            or not isinstance(entry["calls"], list) or not isinstance(entry["answer"], str)
        ):
            raise ValueError(
                f"{case}: supply calls and a string answer, or a legacy tool-name list"
            )
        for call in entry["calls"]:
            if (
                not isinstance(call, dict) or set(call) != {"name", "arguments"}
                or not isinstance(call["name"], str) or not isinstance(call["arguments"], dict)
            ):
                raise ValueError(
                    f"{case}: each call requires a string name and an arguments object"
                )


def _score(recorded: dict[str, Any]) -> int:
    _validate_recording(recorded)
    published = _published()
    references = {
        pair.attrib["id"]: {
            "calls": json.loads(pair.findtext("calls") or "[]"),
            "answer": pair.findtext("answer") or "",
        }
        for pair in ET.parse(QUESTIONS).getroot().findall("qa_pair")
    }
    unknown = sorted(set(recorded) - set(published))
    if unknown:
        print(f"not questions in this evaluation: {', '.join(unknown)}", file=sys.stderr)
        return 2

    exact = 0
    verified = 0
    width = max(len(case) for case in published)
    for case, expected in published.items():
        if case not in recorded:
            print(f"{case.ljust(width)}  NOT ANSWERED")
            continue
        entry = recorded[case]
        detailed = isinstance(entry, dict)
        selected = sorted(set(call["name"] for call in entry["calls"]) if detailed else set(entry))
        missed = [name for name in expected if name not in selected]
        extra = [name for name in selected if name not in expected]
        if not missed and not extra:
            exact += 1
        detail = []
        if missed:
            detail.append(f"missed {', '.join(missed)}")
        if extra:
            detail.append(f"also called {', '.join(extra)}")
        if detailed:
            if entry["calls"] != references[case]["calls"]:
                detail.append("calls differ (arguments, order or count)")
            if entry["answer"].strip() != references[case]["answer"]:
                detail.append("answer differs")
            if not detail:
                verified += 1
        else:
            detail.append("arguments and answer NOT EVALUATED")
        print(f"{case.ljust(width)}  {'; '.join(detail) if detail else 'calls and answer match'}")

    print(f"\n{exact} of {len(published)} questions selected exactly the published tools.")
    print(f"{verified} of {len(published)} recorded calls and answers match the reference.")
    # A reported score is the whole output. Exiting non-zero on an imperfect run
    # would turn a measurement into a gate, which is what this deliberately is not.
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = parser.add_subparsers(dest="command", required=True)
    sub.add_parser("context", help="print the instructions and tools a model is given")
    sub.add_parser("questions", help="print the questions with answers and tools stripped")
    scorer = sub.add_parser("score", help="score recorded calls and answers")
    scorer.add_argument(
        "recorded", type=Path, help="JSON keyed by question id, with calls and answer"
    )
    args = parser.parse_args(argv)

    if args.command == "context":
        print(asyncio.run(_describe()))
        return 0
    if args.command == "questions":
        for case, question in _questions().items():
            print(f"{case}: {question}")
        return 0
    try:
        return _score(json.loads(args.recorded.read_text(encoding="utf-8")))
    except (ValueError, OSError) as exc:
        print(f"invalid recording: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
