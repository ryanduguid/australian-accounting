from pathlib import Path

from PIL import Image

from scripts.render_demo_image import (
    BACKGROUND,
    BOTTOM_MARGIN,
    FONT_SIZE,
    FOREGROUND,
    HEIGHT,
    LEFT_MARGIN,
    LINE_HEIGHT,
    PROMPT,
    TOP_MARGIN,
    WIDTH,
    proof_lines,
    render_image,
)


def _decode(path: Path) -> tuple[str | None, tuple[int, int], int, bytes]:
    with Image.open(path) as image:
        return (
            image.format,
            image.size,
            getattr(image, "n_frames", 1),
            image.convert("RGB").tobytes(),
        )


def test_regenerated_static_proof_matches_committed_image(
    tmp_path: Path,
) -> None:
    root = Path(__file__).resolve().parents[1]
    generated = tmp_path / "quick-proof.webp"
    transcript = root / "docs" / "quick-proof.txt"

    render_image(transcript, generated)
    committed_path = root / "docs" / "quick-proof.webp"
    committed = _decode(committed_path)

    assert committed == _decode(generated)
    image_format, size, frame_count, pixels = committed
    assert image_format == "WEBP"
    assert size == (WIDTH, HEIGHT) == (1200, 720)
    assert frame_count == 1
    assert BACKGROUND == (4, 0, 31)
    assert FOREGROUND == (244, 239, 255)
    assert PROMPT == (192, 132, 252)
    assert LEFT_MARGIN == 40
    assert TOP_MARGIN == 32
    assert BOTTOM_MARGIN == 32
    assert LINE_HEIGHT == 40
    assert FONT_SIZE == 24
    assert pixels[0:3] == bytes(BACKGROUND)
    assert committed_path.stat().st_size < 100_000
    assert committed_path.read_bytes() == generated.read_bytes()


def test_static_proof_selects_both_checked_outcomes() -> None:
    root = Path(__file__).resolve().parents[1]
    lines = proof_lines(root / "docs" / "quick-proof.txt")

    assert lines[0] == "$ uv run --locked aus-accounting-mcp-demo"
    assert "generate_synthetic_sbr_fixture" in lines
    assert "  synthetic: true" in lines
    assert "  not_a_lodgment: true" in lines
    assert '  form_type: "BAS_AU_ACTIVITY_STATEMENT"' in lines
    assert '  summary.total_payable_to_ato: "42500.00"' in lines
    assert "refuse_div7a" in lines
    assert '  code: "ERR_POLICY_DIV7A_REFUSED"' in lines
    assert "  available: false" in lines
    assert "  reviewed_engine: false" in lines
