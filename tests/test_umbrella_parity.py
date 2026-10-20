"""Temporary path aliases must not hide real command-output differences."""
import os
import runpy
import unittest
from unittest.mock import patch

from test_boundaries import ROOT

scrub = runpy.run_path(str(ROOT / "apps/au-accounting/tests/parity.py"))["scrub"]


class UmbrellaParityTests(unittest.TestCase):
    def test_macos_alias_and_canonical_path_are_both_removed(self):
        folder = "/var/folders/test/scratch"
        canonical = "/private" + folder
        with patch.object(os.path, "realpath", return_value=canonical):
            self.assertEqual(
                scrub(f"{folder}/report.csv {canonical}/report.csv".encode(), folder),
                b"<TMP>/report.csv <TMP>/report.csv",
            )
            self.assertEqual(
                scrub(f"error: file not found: {canonical}/report.csv".encode(), folder),
                b"error: file not found: <TMP>/report.csv",
            )

    def test_windows_escaped_paths_and_other_errors_are_preserved(self):
        folder = r"C:\Temp\scratch"
        escaped = folder.replace("\\", "\\\\")
        with patch.object(os.path, "realpath", return_value=folder):
            self.assertEqual(
                scrub(f"{folder} {escaped} /private/unrelated wrong value".encode(), folder),
                b"<TMP> <TMP> /private/unrelated wrong value",
            )
