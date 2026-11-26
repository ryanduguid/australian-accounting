# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: AGPL-3.0-or-later
# Additional terms under AGPL-3.0 section 7(b): see NOTICE.

"""The calculators wheel holds the release archive's executable byte for byte.

Standard library only:

    python -m unittest -v tests/test_calculators_wheel.py
"""

from __future__ import annotations

import base64
import hashlib
import importlib.util
import io
import tarfile
import tempfile
import unittest
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    "calculators_wheel", ROOT / ".github" / "ci" / "calculators_wheel.py")
assert SPEC is not None and SPEC.loader is not None
wheel = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(wheel)

EXECUTABLE = b"\x7fELF not really an executable \x00\xff"


def _archive(directory: Path, name: str, executable: str) -> Path:
    path = directory / name
    files = {executable: EXECUTABLE, "LICENSE": b"licence", "NOTICE": b"notice",
             "README.md": b"readme"}
    if name.endswith(".zip"):
        with zipfile.ZipFile(path, "w") as z:
            for member, data in files.items():
                z.writestr(member, data)
    else:
        with tarfile.open(path, "w:gz") as t:
            for member, data in files.items():
                info = tarfile.TarInfo(member)
                info.size = len(data)
                t.addfile(info, io.BytesIO(data))
    return path


class CalculatorsWheel(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.directory = Path(temporary.name)

    def _check(self, archive: str, executable: str, filename: str, tags: list[str],
               system: str) -> None:
        built = wheel.build(_archive(self.directory, archive, executable), self.directory)
        self.assertEqual(built.name, filename)
        info = "australian_tax_calculators-2.3.4.dist-info"
        script = f"australian_tax_calculators-2.3.4.data/scripts/{executable}"
        with zipfile.ZipFile(built) as z:
            self.assertEqual(z.namelist(), [
                script, f"{info}/METADATA", f"{info}/WHEEL", f"{info}/licenses/LICENSE",
                f"{info}/licenses/NOTICE", f"{info}/RECORD"])
            self.assertEqual(z.read(script), EXECUTABLE)
            self.assertEqual(z.getinfo(script).external_attr >> 16, 0o100755)
            self.assertEqual(z.read(f"{info}/licenses/NOTICE"), b"notice")
            record = z.read(f"{info}/RECORD").decode().splitlines()
            self.assertEqual(record[-1], f"{info}/RECORD,,")
            for line in record[:-1]:
                path, digest, size = line.split(",")
                data = z.read(path)
                expected = base64.urlsafe_b64encode(hashlib.sha256(data).digest()).rstrip(b"=")
                self.assertEqual((digest, int(size)), (f"sha256={expected.decode()}", len(data)))
            self.assertEqual(
                [line for line in z.read(f"{info}/WHEEL").decode().splitlines()
                 if line.startswith("Tag: ")],
                [f"Tag: py3-none-{tag}" for tag in tags])
            metadata = z.read(f"{info}/METADATA").decode()
        self.assertIn("Name: australian-tax-calculators\nVersion: 2.3.4\n", metadata)
        self.assertIn(f"Operating System :: {system}\n", metadata)
        self.assertIn("releases/tag/australian-tax-calculators/v2.3.4)", metadata)
        self.assertNotIn("{", metadata)
        # The same archive gives the same bytes.
        again = self.directory / "again"
        again.mkdir()
        self.assertEqual(wheel.build(self.directory / archive, again).read_bytes(),
                         built.read_bytes())

    def test_linux_archive(self) -> None:
        self._check(
            "australian-tax-calculators-2.3.4-x86_64-unknown-linux-musl.tar.gz",
            "australian-tax-calculators",
            "australian_tax_calculators-2.3.4-py3-none-manylinux_2_12_x86_64."
            "manylinux2010_x86_64.musllinux_1_1_x86_64.whl",
            ["manylinux_2_12_x86_64", "manylinux2010_x86_64", "musllinux_1_1_x86_64"],
            "POSIX :: Linux")

    def test_windows_archive(self) -> None:
        self._check(
            "australian-tax-calculators-2.3.4-x86_64-pc-windows-msvc.zip",
            "australian-tax-calculators.exe",
            "australian_tax_calculators-2.3.4-py3-none-win_amd64.whl",
            ["win_amd64"], "Microsoft :: Windows")

    def test_refuses_another_archive(self) -> None:
        for name in ("australian-tax-calculators-2.3.4.tar.gz",
                     "australian-tax-calculators-2.3.4-aarch64-apple-darwin.tar.gz",
                     "australian-tax-calculators-2.3-x86_64-pc-windows-msvc.zip"):
            with self.subTest(name=name), self.assertRaises(SystemExit):
                wheel.build(self.directory / name, self.directory)


if __name__ == "__main__":
    unittest.main()
