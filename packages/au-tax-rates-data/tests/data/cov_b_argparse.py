"""Record what CPython 3.15's argparse prints for the `rates` command line.

Run from the package folder with CPython 3.15: `python -I tests/data/cov_b_argparse.py`.
It writes `tests/data/cov_b_argparse-python.json`, which `tests/cov_b_argparse.rs`,
`tests/cov_b_argtheme.rs` and `tests/cov_b_termsize.rs` compare the port against. Each case runs
in a child interpreter with only the named terminal and colour variables set.
"""

import json
import os
import pathlib
import re
import subprocess
import sys

HERE = pathlib.Path(__file__).resolve().parent
RATES = (HERE.parent.parent / "src" / "rates.rs").read_text(encoding="utf-8")
DOC = re.search(r'const DOC: &str = "(.*?)";\n', RATES, re.S).group(1)

DRIVER = r"""
import argparse, json, sys
p = argparse.ArgumentParser(prog="rates", description=sys.argv[1],
                            formatter_class=argparse.RawDescriptionHelpFormatter)
sub = p.add_subparsers(dest="command", required=True)
sub.add_parser("snapshot")
sub.add_parser("validate")
sub.add_parser("check").add_argument("--out", default="reports",
                                     help="folder for the Markdown report")
sub.add_parser("ato-tables")
print(json.dumps(vars(p.parse_args(sys.argv[2:])), sort_keys=True))
"""

TERMINAL = ("COLUMNS", "LINES", "PYTHON_COLORS", "NO_COLOR", "FORCE_COLOR", "TERM")
PLAIN = {"COLUMNS": "80", "LINES": "24", "PYTHON_COLORS": "0"}

CASES = [
    # Layout at narrow widths (argparse.rs).
    ("check_help_30", ["check", "--help"], {"COLUMNS": "30", "LINES": "10", "PYTHON_COLORS": "0"}),
    ("check_help_20", ["check", "--help"], {"COLUMNS": "20", "LINES": "10", "PYTHON_COLORS": "0"}),
    ("snapshot_help_12", ["snapshot", "-h"], {"COLUMNS": "12", "LINES": "10", "PYTHON_COLORS": "0"}),
    ("no_command_24", [], {"COLUMNS": "24", "LINES": "10", "PYTHON_COLORS": "0"}),
    ("no_command_16", [], {"COLUMNS": "16", "LINES": "10", "PYTHON_COLORS": "0"}),
    # Option and argument errors (argparse.rs).
    ("out_without_value", ["check", "--out"], PLAIN),
    ("bare_equals_is_ambiguous", ["check", "--=x"], PLAIN),
    ("long_help_with_value", ["--help=x"], PLAIN),
    ("short_help_with_value", ["-h=x"], PLAIN),
    ("short_help_with_dash", ["-h-x"], PLAIN),
    ("clustered_help_with_value", ["-hh=x"], PLAIN),
    ("lone_dash", ["-"], PLAIN),
    ("negative_integer", ["-5"], PLAIN),
    ("negative_decimal", ["-1.5"], PLAIN),
    ("dash_with_space", ["-x y"], PLAIN),
    ("unknown_option", ["--zzz"], PLAIN),
    ("stray_after_command", ["snapshot", "extra"], PLAIN),
    ("stray_before_option", ["check", "x", "--out", "y"], PLAIN),
    ("far_word", ["zzz"], PLAIN),
    ("near_word", ["validat"], PLAIN),
    ("very_long_word", ["validate" + "a" * 250], PLAIN),
    # Clustered and separated help flags (argparse.rs).
    ("clustered_help", ["-hh"], PLAIN),
    ("help_then_unknown", ["-hx"], PLAIN),
    ("clustered_help_then_unknown", ["-hhx"], PLAIN),
    ("help_after_dashes", ["--", "check", "-h"], PLAIN),
    # Namespaces (argparse.rs).
    ("out_after_equals", ["check", "--out=x"], PLAIN),
    ("abbreviated_out", ["check", "--o", "x"], PLAIN),
    ("abbreviated_out_equals", ["check", "--o=y"], PLAIN),
    ("command_after_dashes", ["--", "snapshot"], PLAIN),
    ("last_out_wins", ["check", "--out", "a", "--out", "b"], PLAIN),
    # Colour (argtheme.rs).
    ("python_colors_1", ["check", "-h"], {"COLUMNS": "80", "LINES": "24", "PYTHON_COLORS": "1"}),
    ("force_color", ["check", "-h"], {"COLUMNS": "80", "LINES": "24", "FORCE_COLOR": "1"}),
    ("no_color_beats_force", ["check", "-h"],
     {"COLUMNS": "80", "LINES": "24", "NO_COLOR": "1", "FORCE_COLOR": "1"}),
    ("python_colors_0_beats_force", ["check", "-h"],
     {"COLUMNS": "80", "LINES": "24", "PYTHON_COLORS": "0", "FORCE_COLOR": "1"}),
    ("dumb_terminal_beats_nothing", ["check", "-h"],
     {"COLUMNS": "80", "LINES": "24", "TERM": "dumb"}),
    ("coloured_error", ["check", "--out"], {"COLUMNS": "80", "LINES": "24", "PYTHON_COLORS": "1"}),
    # Terminal size (termsize.rs).
    ("columns_without_lines", ["check", "-h"], {"COLUMNS": "40", "PYTHON_COLORS": "0"}),
    ("columns_with_lines", ["check", "-h"], {"COLUMNS": "40", "LINES": "10", "PYTHON_COLORS": "0"}),
]


def run(args, env):
    child = {k: v for k, v in os.environ.items() if k not in TERMINAL}
    child.update(env)
    done = subprocess.run(
        [sys.executable, "-c", DRIVER, DOC, *args],
        env=child,
        capture_output=True,
        text=True,
        encoding="utf-8",
        stdin=subprocess.DEVNULL,
    )
    return {"args": args, "env": env, "code": done.returncode,
            "stdout": done.stdout, "stderr": done.stderr}


def main():
    if sys.version_info[:2] != (3, 15):
        sys.exit("record with CPython 3.15")
    cases = {name: run(args, env) for name, args, env in CASES}
    out = HERE / "cov_b_argparse-python.json"
    out.write_text(json.dumps(cases, indent=1, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")


main()
