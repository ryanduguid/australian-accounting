"""CLI entrypoint for aus-accounting-mcp."""

from __future__ import annotations

import argparse

from aus_accounting_mcp import __version__
from aus_accounting_mcp.server import run_stdio

DESCRIPTION = (
    "Run the Aus Accounting MCP server over stdio. An MCP client such as Claude "
    "Desktop, Claude Code, Cursor or Codex starts this command itself; run by hand, "
    "it waits silently for a client."
)
EPILOG = (
    "Optional retrieval folders: set AUS_ACCOUNTING_LIBRARY_ROOT to a Markdown "
    "library, or AUS_ACCOUNTING_CORPUS_ROOT to a legislation corpus, in the client's "
    "server configuration. Setup guide: "
    "https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/"
    "docs/REFERENCE.md#client-setup"
)


def main(argv: list[str] | None = None) -> None:
    parser = argparse.ArgumentParser(
        prog="aus-accounting-mcp", description=DESCRIPTION, epilog=EPILOG
    )
    parser.add_argument("--version", action="version", version=f"%(prog)s {__version__}")
    parser.parse_args(argv)
    run_stdio()


if __name__ == "__main__":
    main()
