# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: AGPL-3.0-or-later
# Additional terms under AGPL-3.0 section 7(b): see NOTICE.

"""Wrap a released australian-tax-calculators archive's executable in a PyPI wheel.

usage: python -I .github/ci/calculators_wheel.py ARCHIVE OUT_DIR

ARCHIVE is a release's x86_64 Linux .tar.gz or Windows .zip, already checked
against its attested SHA256SUMS. The wheel holds the archive's executable byte
for byte as its one script, with its LICENSE and NOTICE: pip and uv copy a
script that does not start with #!python unchanged, so the installed command
has the released SHA-256 aus-accounting-mcp checks. Entries carry a fixed
timestamp, so the same archive gives the same wheel. Standard library only:
the publishing job runs no other code before the PyPI upload action.
"""

import base64
import hashlib
import re
import sys
import tarfile
import zipfile
from pathlib import Path

NAME = "australian-tax-calculators"
DIST = "australian_tax_calculators"
# The Linux executable is static (musl), so it needs no particular C library.
PLATFORMS = {
    "x86_64-unknown-linux-musl.tar.gz": (
        "manylinux_2_12_x86_64.manylinux2010_x86_64.musllinux_1_1_x86_64", ""),
    "x86_64-pc-windows-msvc.zip": ("win_amd64", ".exe"),
}
ARCHIVE = re.compile(rf"{NAME}-(\d+\.\d+\.\d+)-({'|'.join(map(re.escape, PLATFORMS))})")
DESCRIPTION = """\
# Australian Tax Calculators

Bounded Australian tax calculation worksheets with explicit scope and sources, as
one standalone Rust executable. Each worksheet is a subcommand that reads one JSON
object on stdin and writes one JSON result on stdout, with its amounts, scope
exclusions, official source URLs and engine version. It makes no network calls and
writes no files.

```sh
uvx australian-tax-calculators --version
```

This wheel holds, unchanged, the x86_64 {platform} executable attested in the GitHub
release [australian-tax-calculators/v{version}]({release}). Other platforms build it
from source; the [README]({source}) gives the worksheets, their inputs and the build.

Versions 0.2.0 and earlier were a Python package with an `austaxcalc` import API,
which 1.x replaces with this JSON interface.
"""
REPOSITORY = "https://github.com/ryanduguid/australian-accounting"


def metadata(version: str, platform: str) -> str:
    release = f"{REPOSITORY}/releases/tag/{NAME}/v{version}"
    source = f"{REPOSITORY}/tree/main/packages/{NAME}"
    return (
        "Metadata-Version: 2.4\n"
        f"Name: {NAME}\nVersion: {version}\n"
        "Summary: Bounded Australian tax calculation worksheets with explicit scope and sources\n"
        "Author: Ryan Duguid\nLicense-Expression: AGPL-3.0-or-later\n"
        "License-File: LICENSE\nLicense-File: NOTICE\n"
        "Classifier: Environment :: Console\n"
        "Classifier: Intended Audience :: Financial and Insurance Industry\n"
        f"Classifier: Operating System :: {platform}\n"
        "Classifier: Programming Language :: Rust\n"
        "Classifier: Topic :: Office/Business :: Financial :: Accounting\n"
        "Project-URL: Homepage, https://duguid.com.au/tools/australian-tax-ai-agents/\n"
        f"Project-URL: Source, {source}\nProject-URL: Release, {release}\n"
        "Description-Content-Type: text/markdown\n\n"
        + DESCRIPTION.format(platform=platform.split(" :: ")[-1], version=version,
                             release=release, source=source)
    )


def members(archive: Path, names: list[str]) -> list[bytes]:
    if archive.suffix == ".zip":
        with zipfile.ZipFile(archive) as z:
            return [z.read(n) for n in names]
    with tarfile.open(archive, "r:gz") as t:
        return [t.extractfile(n).read() for n in names]  # type: ignore[union-attr]


def build(archive: Path, out: Path) -> Path:
    match = ARCHIVE.fullmatch(archive.name)
    if not match:
        raise SystemExit(f"not a release platform archive: {archive.name}")
    version, kind = match.groups()
    tags, suffix = PLATFORMS[kind]
    platform = "Microsoft :: Windows" if suffix else "POSIX :: Linux"
    script = f"{DIST}-{version}.data/scripts/{NAME}{suffix}"
    info = f"{DIST}-{version}.dist-info"
    executable, licence, notice = members(archive, [NAME + suffix, "LICENSE", "NOTICE"])
    wheel_file = "Wheel-Version: 1.0\nGenerator: calculators_wheel.py\nRoot-Is-Purelib: false\n"
    wheel_file += "".join(f"Tag: py3-none-{tag}\n" for tag in tags.split("."))
    files = {
        script: executable,
        f"{info}/METADATA": metadata(version, platform).encode(),
        f"{info}/WHEEL": wheel_file.encode(),
        f"{info}/licenses/LICENSE": licence,
        f"{info}/licenses/NOTICE": notice,
    }
    record = "".join(
        f"{path},sha256={base64.urlsafe_b64encode(hashlib.sha256(data).digest()).rstrip(b'=').decode()},{len(data)}\n"
        for path, data in files.items()) + f"{info}/RECORD,,\n"
    files[f"{info}/RECORD"] = record.encode()
    wheel = out / f"{DIST}-{version}-py3-none-{tags}.whl"
    with zipfile.ZipFile(wheel, "x") as z:
        for path, data in files.items():
            entry = zipfile.ZipInfo(path, (1980, 1, 1, 0, 0, 0))
            entry.compress_type = zipfile.ZIP_DEFLATED
            entry.external_attr = (0o100755 if path == script else 0o100644) << 16
            z.writestr(entry, data)
    return wheel


if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit(__doc__)
    print(build(Path(sys.argv[1]), Path(sys.argv[2])))
