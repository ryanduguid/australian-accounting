# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: AGPL-3.0-or-later
# Additional terms under AGPL-3.0 section 7(b): see NOTICE.

"""What CPython 3.15's `ntpath` answers for the cases in `src/cov_a7_pypath.rs`.

Run on Windows with CPython 3.15: `python -I tests/data/cov_a7_pypath.py`. The
`realpath` cases run in a new temporary folder, printed as `T` (its resolved
name). Creating symbolic links needs Developer Mode or an administrator.
"""

import ntpath
import os
import tempfile

print("splitroot", ntpath.splitroot("\\\\server"))
print("join", ntpath.join("C:\\x", "D:\\y"))
print("join", ntpath.join("\\\\server\\share", "x"))
for path in ["C:\\a\\b", "C:\\a\\", "C:\\", "a/b\\\\c", "b", "\\\\server\\share\\x"]:
    print("split", repr(path), ntpath.split(path))
for path in ["C:\\..\\a", "\\..\\a", "", "a\\.."]:
    print("normpath", repr(path), repr(ntpath.normpath(path)))

with tempfile.TemporaryDirectory() as tmp:
    top = ntpath.realpath(tmp)

    def show(path):
        print("realpath", repr(path.replace(tmp, "T")), repr(ntpath.realpath(path).replace(top, "T")))

    show(tmp + "\\nope\\deeper")
    os.symlink(tmp + "\\absent", tmp + "\\to-absent")
    show(tmp + "\\to-absent")
    os.mkdir(tmp + "\\d")
    os.symlink("missing", tmp + "\\d\\to-missing")
    show(tmp + "\\d\\to-missing")
    os.symlink(tmp + "\\loop-b", tmp + "\\loop-a")
    os.symlink(tmp + "\\loop-a", tmp + "\\loop-b")
    show(tmp + "\\loop-a")
    cwd = ntpath.realpath(os.getcwd())
    print("realpath 'nope-rel' is cwd\\nope-rel:", ntpath.realpath("nope-rel") == cwd + "\\nope-rel")
