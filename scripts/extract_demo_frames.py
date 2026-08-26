"""Extract the first, middle and final frames from the proof GIF."""

from __future__ import annotations

import argparse
from pathlib import Path

from PIL import Image


def extract_frames(gif_path: Path, output_directory: Path) -> None:
    output_directory.mkdir(parents=True, exist_ok=True)

    with Image.open(gif_path) as image:
        targets = (
            (0, "first.png"),
            (image.n_frames // 2, "middle.png"),
            (image.n_frames - 1, "final.png"),
        )
        for index, filename in targets:
            image.seek(index)
            image.convert("RGB").save(output_directory / filename)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("gif", type=Path)
    parser.add_argument("output_directory", type=Path)
    args = parser.parse_args()
    extract_frames(args.gif, args.output_directory)


if __name__ == "__main__":
    main()
