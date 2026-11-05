# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

"""CLI entrypoint for aus-accounting-mcp."""

from __future__ import annotations

import argparse

from aus_accounting_mcp import __version__, doctor
from aus_accounting_mcp.server import run_stdio

DESCRIPTION = (
    "Run the Aus Accounting MCP server over stdio. An MCP client such as Claude "
    "Desktop, Claude Code, Cursor or Codex starts this command itself; run by hand, "
    "it waits silently for a client."
)
EPILOG = (
    "Optional retrieval folders: set AUS_ACCOUNTING_LIBRARY_ROOT to a Markdown "
    "library, or AUS_ACCOUNTING_CORPUS_ROOT to a legislation corpus, in the client's "
    "server configuration, then run `aus-accounting-mcp doctor` with the same "
    "variables to check them. Setup guide: "
    "https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/"
    "docs/REFERENCE.md#client-setup"
)


def main(argv: list[str] | None = None) -> None:
    parser = argparse.ArgumentParser(
        prog="aus-accounting-mcp", description=DESCRIPTION, epilog=EPILOG
    )
    parser.add_argument("--version", action="version", version=f"%(prog)s {__version__}")
    commands = parser.add_subparsers(dest="command", title="commands")
    commands.add_parser("doctor", help=doctor.HELP, description=doctor.HELP)
    if parser.parse_args(argv).command == "doctor":
        raise SystemExit(doctor.run())
    run_stdio()


if __name__ == "__main__":
    main()
