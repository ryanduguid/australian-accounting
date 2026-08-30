"""Render the checked MCP proof transcript as deterministic static media."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

from PIL import Image, ImageDraw, ImageFont

WIDTH = 1200
HEIGHT = 720
BACKGROUND = (4, 0, 31)
FOREGROUND = (244, 239, 255)
PROMPT = (192, 132, 252)
LEFT_MARGIN = 40
TOP_MARGIN = 32
BOTTOM_MARGIN = 32
LINE_HEIGHT = 40
FONT_SIZE = 24


def _json_value(value: Any) -> str:
    return json.dumps(value, ensure_ascii=False)


def proof_lines(transcript_path: Path) -> list[str]:
    lines = transcript_path.read_text(encoding="utf-8").splitlines()
    if len(lines) < 2 or not lines[0].startswith("$ "):
        raise ValueError("quick-proof transcript must start with its command")

    payload = json.loads("\n".join(lines[1:]))
    calls = {call["tool"]: call for call in payload["calls"]}
    fixture = calls["generate_synthetic_sbr_fixture"]["result"]
    refusal = calls["refuse_div7a"]["result"]

    return [
        lines[0],
        "",
        "generate_synthetic_sbr_fixture",
        f"  synthetic: {_json_value(fixture['synthetic'])}",
        f"  not_a_lodgment: {_json_value(fixture['not_a_lodgment'])}",
        f"  form_type: {_json_value(fixture['form_type'])}",
        "  summary.total_payable_to_ato: "
        f"{_json_value(fixture['summary']['total_payable_to_ato'])}",
        "",
        "refuse_div7a",
        f"  code: {_json_value(refusal['code'])}",
        f"  available: {_json_value(refusal['available'])}",
        f"  reviewed_engine: {_json_value(refusal['reviewed_engine'])}",
    ]


def render_image(transcript_path: Path, output_path: Path) -> None:
    lines = proof_lines(transcript_path)
    if TOP_MARGIN + len(lines) * LINE_HEIGHT > HEIGHT - BOTTOM_MARGIN:
        raise ValueError("quick-proof summary does not fit the image")

    image = Image.new("RGB", (WIDTH, HEIGHT), BACKGROUND)
    draw = ImageDraw.Draw(image)
    font = ImageFont.load_default(size=FONT_SIZE)

    for row, line in enumerate(lines):
        colour = (
            PROMPT if line.startswith("$ ") or (line and not line.startswith(" ")) else FOREGROUND
        )
        draw.text(
            (LEFT_MARGIN, TOP_MARGIN + row * LINE_HEIGHT),
            line,
            font=font,
            fill=colour,
        )

    image.save(output_path, format="WEBP", lossless=True, method=6, exact=True)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("transcript", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    render_image(args.transcript, args.output)


if __name__ == "__main__":
    main()
