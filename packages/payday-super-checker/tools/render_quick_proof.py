"""Render a compact README proof from the fabricated CLI example."""

from __future__ import annotations

import argparse
import html
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SAMPLE = "examples/sample_payrun_no_transition.csv"
SVG = ROOT / "assets" / "quick-proof.svg"
TRANSCRIPT = ROOT / "assets" / "quick-proof.md"
COMMAND = f"payday-super-check {SAMPLE} --as-at 2026-09-10"


def run_example() -> str:
    with tempfile.TemporaryDirectory(prefix="payday-super-proof-") as directory:
        report = Path(directory) / "report.csv"
        result = subprocess.run(
            [
                "cargo",
                "run",
                "--quiet",
                "--locked",
                "--release",
                "--",
                SAMPLE,
                "--as-at",
                "2026-09-10",
                "-o",
                str(report),
            ],
            cwd=ROOT,
            text=True,
            capture_output=True,
            check=False,
        )
        if result.returncode != 2:
            raise RuntimeError(
                f"expected review-required exit code 2, got {result.returncode}: "
                f"{result.stdout}{result.stderr}"
            )
        output = (result.stdout + result.stderr).replace(str(report), "report.csv")
        output = output.replace("\r\n", "\n")

    required = (
        "ON_TIME: 3  AT_RISK: 1  LATE: 1  UNPAID: 1  UNKNOWN: 0  SKIPPED: 1",
        "shortfall $780.00",
        "experimental estimated SG charge $788.76 - $1262.02",
        "Educational tool, not advice",
    )
    missing = [text for text in required if text not in output]
    if missing:
        raise RuntimeError(f"the example output changed: missing {missing}")
    return output


def render_transcript(output: str) -> str:
    return f"""# Fabricated Payday Super review

Command:

```bash
{COMMAND}
```

Exit code: 2, review required.

```text
{output.rstrip()}
```

The input is fabricated. No employer or employee data are used.
"""


def render_svg() -> str:
    command = html.escape(COMMAND)
    return f"""<svg xmlns="http://www.w3.org/2000/svg" width="1000" height="450" viewBox="0 0 1000 450" role="img" aria-labelledby="title desc">
  <title id="title">Fabricated Payday Super review</title>
  <desc id="desc">A fabricated seven-line review with three on-time lines and three lines requiring attention.</desc>
  <rect width="1000" height="450" rx="20" fill="#000000"/>
  <rect x="34" y="34" width="932" height="382" rx="14" fill="#0A0A0A" stroke="#E5E4E2"/>
  <text x="68" y="84" fill="#FFFFF0" font-family="ui-monospace, SFMono-Regular, Consolas, monospace" font-size="28" font-weight="700">payday-super-checker</text>
  <text x="68" y="116" fill="#C8C8C8" font-family="ui-monospace, SFMono-Regular, Consolas, monospace" font-size="15">FABRICATED REVIEW PROOF  |  AS AT 2026-09-10</text>
  <line x1="68" y1="142" x2="932" y2="142" stroke="#606060"/>
  <text x="68" y="181" fill="#E5E4E2" font-family="ui-monospace, SFMono-Regular, Consolas, monospace" font-size="15">{command}</text>
  <text x="68" y="232" fill="#FFFFF0" font-family="ui-monospace, SFMono-Regular, Consolas, monospace" font-size="18">ON_TIME  3     AT_RISK  1     LATE  1</text>
  <text x="68" y="270" fill="#FFFFF0" font-family="ui-monospace, SFMono-Regular, Consolas, monospace" font-size="18">UNPAID  1      UNKNOWN  0     SKIPPED  1</text>
  <text x="68" y="320" fill="#E5E4E2" font-family="ui-monospace, SFMono-Regular, Consolas, monospace" font-size="17">shortfall</text>
  <text x="240" y="320" fill="#FFFFF0" font-family="ui-monospace, SFMono-Regular, Consolas, monospace" font-size="17">$780.00</text>
  <text x="420" y="320" fill="#E5E4E2" font-family="ui-monospace, SFMono-Regular, Consolas, monospace" font-size="17">experimental range</text>
  <text x="672" y="320" fill="#FFFFF0" font-family="ui-monospace, SFMono-Regular, Consolas, monospace" font-size="17">$788.76 - $1,262.02</text>
  <rect x="704" y="354" width="206" height="40" rx="20" fill="#990024" stroke="#E5E4E2"/>
  <text x="807" y="380" text-anchor="middle" fill="#FFFFF0" font-family="ui-monospace, SFMono-Regular, Consolas, monospace" font-size="15" font-weight="700">REVIEW REQUIRED</text>
  <text x="68" y="380" fill="#C8C8C8" font-family="ui-monospace, SFMono-Regular, Consolas, monospace" font-size="15">exit 2  |  full transcript linked</text>
</svg>
"""


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args(argv)
    output = run_example()
    expected = {
        SVG: render_svg(),
        TRANSCRIPT: render_transcript(output),
    }

    if args.check:
        stale = [path for path, text in expected.items() if not path.is_file() or path.read_text(encoding="utf-8") != text]
        if stale:
            for path in stale:
                print(f"stale: {path.relative_to(ROOT)}")
            return 1
        print("quick proof is current")
        return 0

    for path, text in expected.items():
        path.write_text(text, encoding="utf-8", newline="\n")
        print(f"wrote {path.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
