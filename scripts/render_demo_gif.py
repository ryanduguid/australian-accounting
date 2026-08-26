"""Render checked MCP proof text as deterministic terminal media."""

from __future__ import annotations

import argparse
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

WIDTH = 1200
HEIGHT = 720
BACKGROUND = (4, 0, 31)
FOREGROUND = (244, 239, 255)
PROMPT = (192, 132, 252)
LEFT_MARGIN = 40
TOP_MARGIN = 32
BOTTOM_MARGIN = 32
LINE_HEIGHT = 30
FONT_SIZE = 24
TOTAL_CENTISECONDS = 3_000
TOTAL_DURATION_MS = TOTAL_CENTISECONDS * 10


def _durations(frame_count: int) -> list[int]:
    if frame_count == 0:
        raise ValueError("quick-proof transcript must contain at least one line")

    centiseconds, remainder = divmod(TOTAL_CENTISECONDS, frame_count)
    return [(centiseconds + (1 if index < remainder else 0)) * 10 for index in range(frame_count)]


def render_gif(transcript_path: Path, output_path: Path) -> None:
    lines = transcript_path.read_text(encoding="utf-8").splitlines()
    durations = _durations(len(lines))
    font = ImageFont.load_default(size=FONT_SIZE)
    visible_lines = (HEIGHT - TOP_MARGIN - BOTTOM_MARGIN) // LINE_HEIGHT
    frames = []

    for revealed in range(1, len(lines) + 1):
        image = Image.new("RGB", (WIDTH, HEIGHT), BACKGROUND)
        draw = ImageDraw.Draw(image)
        start = max(0, revealed - visible_lines)

        for row, line in enumerate(lines[start:revealed]):
            colour = PROMPT if line.startswith("$ ") else FOREGROUND
            draw.text(
                (LEFT_MARGIN, TOP_MARGIN + row * LINE_HEIGHT),
                line,
                font=font,
                fill=colour,
            )
        frames.append(image)

    frames[0].save(
        output_path,
        format="GIF",
        save_all=True,
        append_images=frames[1:],
        duration=durations,
        loop=0,
        optimize=False,
        disposal=2,
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("transcript", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    render_gif(args.transcript, args.output)


if __name__ == "__main__":
    main()
