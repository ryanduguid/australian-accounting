# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: AGPL-3.0-or-later
# Additional terms under AGPL-3.0 section 7(b): see NOTICE.

"""What CPython 3.15 answers for the cases in `src/cov_a8_rates.rs`.

Run on Linux with CPython 3.15: `python -I tests/data/cov_a8_rates.py`.
"""

import tempfile
from pathlib import Path
from urllib.parse import urlparse

for url in ["http:a;b", "https://h/x;p/y;q", "https://h/x;p/"]:
    print("snapshot name", repr(url), repr(urlparse(url).path.rstrip("/").rsplit("/", 1)[-1] + ".txt"))
for name in ["r0.json", ".json", "a.b.json", "..json"]:
    print("stem", repr(name), repr(Path(name).stem))

with tempfile.TemporaryDirectory() as tmp:
    for name in ["b.json", "a.json", "notes.txt", ".h.json", "x.json.bak"]:
        (Path(tmp) / name).write_text("{}", encoding="utf-8")
    print("glob", [p.name for p in sorted(Path(tmp).glob("*.json"))])
    try:
        (Path(tmp) / "no-folder" / "x.txt").write_text("x", encoding="utf-8")
    except OSError as e:
        print("missing folder", type(e).__name__, str(e).replace(tmp, "T"))

try:
    Path("/r/a\ud800.txt").write_text("x", encoding="utf-8")
except UnicodeEncodeError as e:
    print("lone surrogate name", type(e).__name__, e)
