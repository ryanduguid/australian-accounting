# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: AGPL-3.0-or-later
# Additional terms under AGPL-3.0 section 7(b): see NOTICE.

"""What `rates.py snapshot` (24fcf24, the differential reference) prints,
returns and writes for the records in `cov_a6_rates.rs`, with the browser
replaced: one page fails with RuntimeError, one loads.

Usage: python3.15 -I cov_a6_snapshot.py <path to rates.py at 24fcf24>
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

PAGES = {
    "https://www.ato.gov.au/a-page/": RuntimeError("browser closed"),
    "https://www.ato.gov.au/b-page": "Effective date: 1 July 2026\nend\n",
}


def fetch_text(url):
    page = PAGES[url]
    if isinstance(page, Exception):
        raise page
    return page


with tempfile.TemporaryDirectory() as folder:
    folder = Path(folder)
    rates.DATA = folder / "data"
    rates.SNAPSHOTS = folder / "snapshots"
    rates.DATA.mkdir()
    for i, url in enumerate(sorted(PAGES, reverse=True)):
        record = {"id": f"r{i}", "source_url": url}
        (rates.DATA / f"record-{i}.json").write_text(json.dumps(record), encoding="utf-8")
    rates.fetch_text = fetch_text
    rates.time.sleep = lambda _seconds: None
    out = io.StringIO()
    with contextlib.redirect_stdout(out):
        code = rates.cmd_snapshot(None)
    files = {p.name: p.read_text(encoding="utf-8") for p in sorted(rates.SNAPSHOTS.iterdir())}
    print(json.dumps({"python": sys.version.split()[0], "code": code, "stdout": out.getvalue(), "files": files}))
