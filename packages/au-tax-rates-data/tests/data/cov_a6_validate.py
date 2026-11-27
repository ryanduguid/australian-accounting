# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: AGPL-3.0-or-later
# Additional terms under AGPL-3.0 section 7(b): see NOTICE.

"""What `rates.py validate` (24fcf24, the differential reference) prints and
returns for a package holding one record, `r0.json`, with only an id: no
`source_url`, so its snapshot is `snapshots/.txt`, which does not exist.

Usage: python3.15 -I cov_a6_validate.py <path to rates.py at 24fcf24>
"""

import contextlib
import importlib.util
import io
import json
import sys
import tempfile
from pathlib import Path

spec = importlib.util.spec_from_file_location("rates", sys.argv[1])
rates = importlib.util.module_from_spec(spec)
spec.loader.exec_module(rates)

with tempfile.TemporaryDirectory() as folder:
    rates.ROOT = Path(folder)
    rates.DATA = rates.ROOT / "data"
    rates.SNAPSHOTS = rates.ROOT / "snapshots"
    rates.DATA.mkdir()
    (rates.DATA / "r0.json").write_text(json.dumps({"id": "r0"}), encoding="utf-8")
    out = io.StringIO()
    with contextlib.redirect_stdout(out):
        code = rates.cmd_validate(None)
    print(json.dumps({"python": sys.version.split()[0], "code": code, "stdout": out.getvalue()}))
