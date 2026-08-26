from pathlib import Path

from PIL import Image, ImageSequence

from scripts.extract_demo_frames import extract_frames
from scripts.render_demo_gif import (
    BACKGROUND,
    BOTTOM_MARGIN,
    FONT_SIZE,
    FOREGROUND,
    HEIGHT,
    LEFT_MARGIN,
    LINE_HEIGHT,
    PROMPT,
    TOTAL_DURATION_MS,
    TOP_MARGIN,
    WIDTH,
    render_gif,
)


def _decode(path: Path) -> tuple[tuple[int, int], list[int], list[bytes]]:
    with Image.open(path) as image:
        size = image.size
        durations: list[int] = []
        pixels: list[bytes] = []
        for frame in ImageSequence.Iterator(image):
            durations.append(frame.info["duration"])
            pixels.append(frame.convert("RGB").tobytes())
    return size, durations, pixels


def test_regenerated_gif_matches_committed_decoded_frames(
    tmp_path: Path,
) -> None:
    root = Path(__file__).resolve().parents[1]
    generated = tmp_path / "quick-proof.gif"
    transcript = root / "docs" / "quick-proof.txt"

    render_gif(transcript, generated)
    committed = _decode(root / "docs" / "quick-proof.gif")

    assert committed == _decode(generated)
    size, durations, pixels = committed
    assert size == (WIDTH, HEIGHT) == (1200, 720)
    assert BACKGROUND == (4, 0, 31)
    assert FOREGROUND == (244, 239, 255)
    assert PROMPT == (192, 132, 252)
    assert LEFT_MARGIN == 40
    assert TOP_MARGIN == 32
    assert BOTTOM_MARGIN == 32
    assert LINE_HEIGHT == 30
    assert FONT_SIZE == 24
    assert len(durations) == len(transcript.read_text(encoding="utf-8").splitlines())
    assert sum(durations) == TOTAL_DURATION_MS == 30_000
    assert all(duration > 0 and duration % 10 == 0 for duration in durations)
    assert pixels[0][0:3] == bytes(BACKGROUND)
    with Image.open(root / "docs" / "quick-proof.gif") as image:
        assert image.info["loop"] == 0


def test_extracted_key_frames_match_the_gif(tmp_path: Path) -> None:
    root = Path(__file__).resolve().parents[1]
    gif = root / "docs" / "quick-proof.gif"
    output_directory = tmp_path / "frames"

    extract_frames(gif, output_directory)

    with Image.open(gif) as image:
        targets = (
            (0, "first.png"),
            (image.n_frames // 2, "middle.png"),
            (image.n_frames - 1, "final.png"),
        )
        for index, filename in targets:
            image.seek(index)
            expected = image.convert("RGB").tobytes()
            with Image.open(output_directory / filename) as extracted:
                assert extracted.size == image.size
                assert extracted.convert("RGB").tobytes() == expected
