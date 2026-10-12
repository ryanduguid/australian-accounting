import pytest
from paydaysuper.atomic_io import atomic_text_output


def test_atomic_text_output_keeps_the_previous_file_when_writing_fails(tmp_path):
    """A partially written payroll report must never replace the last complete one."""
    output = tmp_path / "report.csv"
    output.write_text("previous complete report\n", encoding="utf-8")

    with pytest.raises(RuntimeError, match="simulated failure"):
        with atomic_text_output(output, encoding="utf-8") as stream:
            stream.write("partial replacement\n")
            raise RuntimeError("simulated failure")

    assert output.read_text(encoding="utf-8") == "previous complete report\n"
    assert not list(tmp_path.glob(".payday-super-checker-*.tmp"))


def test_generated_output_requires_an_explicit_csv_filename(tmp_path):
    with pytest.raises(ValueError, match=r"must use a \.csv filename"):
        with atomic_text_output(tmp_path / "existing-report.txt", encoding="utf-8"):
            pytest.fail("validation must happen before opening a staging file")

    assert not list(tmp_path.glob(".payday-super-checker-*.tmp"))


def test_a_stream_that_cannot_be_opened_closes_its_descriptor_and_names_the_output(
    tmp_path, monkeypatch
):
    import errno
    import os

    from paydaysuper import atomic_io

    closed = []
    real_close = os.close

    def fail_to_open(fd, *args, **kwargs):
        raise OSError(errno.ENOSPC, "No space left on device")

    def record_close(fd):
        closed.append(fd)
        real_close(fd)

    monkeypatch.setattr(atomic_io.os, "fdopen", fail_to_open)
    monkeypatch.setattr(atomic_io.os, "close", record_close)
    output = tmp_path / "report.csv"

    with pytest.raises(OSError) as exc_info:
        with atomic_text_output(output, encoding="utf-8"):
            pytest.fail("the stream must not open")

    assert exc_info.value.filename == str(output)  # nosec B101: pytest outcome assertion
    assert exc_info.value.strerror == "No space left on device"  # nosec B101: pytest outcome assertion
    assert len(closed) == 1  # nosec B101: pytest outcome assertion
    assert not list(tmp_path.glob(".payday-super-checker-*.tmp"))  # nosec B101: pytest outcome assertion


def test_a_failed_cleanup_does_not_replace_the_write_failure(tmp_path, monkeypatch):
    import errno
    import os

    from paydaysuper import atomic_io

    def fail_replace(source, destination):
        raise OSError(errno.EACCES, "Access is denied")

    def fail_unlink(path):
        raise OSError(errno.EACCES, "Access is denied")

    monkeypatch.setattr(atomic_io.os, "replace", fail_replace)
    monkeypatch.setattr(atomic_io.os, "unlink", fail_unlink)
    output = tmp_path / "report.csv"
    output.write_text("previous complete report\n", encoding="utf-8")

    with pytest.raises(PermissionError) as exc_info:
        with atomic_text_output(output, encoding="utf-8") as stream:
            stream.write("replacement\n")

    monkeypatch.undo()
    assert exc_info.value.filename == str(output)  # nosec B101: pytest outcome assertion
    assert output.read_text(encoding="utf-8") == "previous complete report\n"  # nosec B101: pytest outcome assertion
    for staged in tmp_path.glob(".payday-super-checker-*.tmp"):
        os.unlink(staged)
