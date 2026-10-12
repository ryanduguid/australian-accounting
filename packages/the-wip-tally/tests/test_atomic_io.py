from __future__ import annotations

import os
from pathlib import Path

import pytest
from wiptally.atomic_io import atomic_write_text


def _staged_files(path: Path) -> list[Path]:
    return list(path.parent.glob(f".{path.name}.*.tmp"))


def test_a_failed_replacement_keeps_the_reviewed_output(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    output = tmp_path / "practitioner-review.md"
    output.write_text("signed-off pack\n", encoding="utf-8")

    def fail_replace(_source: Path, _destination: Path) -> None:
        raise PermissionError("destination is locked")

    monkeypatch.setattr(os, "replace", fail_replace)

    with pytest.raises(PermissionError, match="locked"):
        atomic_write_text(output, "new pack\n", newline="\n")

    assert output.read_text(encoding="utf-8") == "signed-off pack\n"  # nosec B101: pytest outcome assertion
    assert _staged_files(output) == []  # nosec B101: pytest outcome assertion


def test_a_staged_file_already_gone_does_not_mask_the_failure(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    output = tmp_path / "practitioner-review.md"

    def vanish_then_fail(source: Path, _destination: Path) -> None:
        Path(source).unlink()
        raise PermissionError("destination is locked")

    monkeypatch.setattr(os, "replace", vanish_then_fail)

    with pytest.raises(PermissionError, match="locked"):
        atomic_write_text(output, "new pack\n", newline="\n")

    assert not output.exists()  # nosec B101: pytest outcome assertion
    assert _staged_files(output) == []  # nosec B101: pytest outcome assertion
