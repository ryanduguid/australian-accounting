"""Fuzz Payday Super Checker's CSV readers with fabricated exports."""

from __future__ import annotations

import sys
import tempfile
from pathlib import Path

# The fuzzing job installs only Atheris, so import the component from source.
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "packages/payday-super-checker"))

from paydaysuper import csv_io, importers  # noqa: E402

SCRATCH = Path(tempfile.mkdtemp(prefix="paydaysuper-fuzz-")) / "export.csv"
MAPPING, EXPLICIT = csv_io.load_mapping(None)


def test_one_input(data: bytes) -> None:
    SCRATCH.write_bytes(data)
    # A malformed export must be refused with ValueError (CsvError and the
    # other refusals subclass it), which the command line reports as
    # "error: <message>"; anything else escaping is a crash.
    for read in (importers.read_payroll, importers.read_super):
        try:
            read(SCRATCH)
        except ValueError:
            pass
    try:
        csv_io.parse_rows(SCRATCH, MAPPING, EXPLICIT)
    except ValueError:
        pass


if __name__ == "__main__":
    import atheris

    atheris.instrument_all()
    atheris.Setup(sys.argv, test_one_input)
    atheris.Fuzz()
