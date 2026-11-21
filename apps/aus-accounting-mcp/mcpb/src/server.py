# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: AGPL-3.0-or-later
# Additional terms under AGPL-3.0 section 7(b): see NOTICE.

"""Select the bundled calculator and run the released server over stdio."""

import os
import platform
import sys
from pathlib import Path

from aus_accounting_mcp.cli import main


def configure_calculators() -> None:
    if platform.machine().lower() not in {"amd64", "x86_64"}:
        return
    names = {
        "linux": "australian-tax-calculators",
        "win32": "australian-tax-calculators.exe",
    }
    if name := names.get(sys.platform):
        binary = Path(__file__).resolve().parents[1] / "bin" / name
        if binary.is_file():
            os.environ.setdefault("AUS_TAX_CALCULATORS_BIN", str(binary))


if __name__ == "__main__":
    configure_calculators()
    main()
