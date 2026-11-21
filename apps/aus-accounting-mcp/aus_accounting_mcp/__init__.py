# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: AGPL-3.0-or-later
# Additional terms under AGPL-3.0 section 7(b): see NOTICE.

"""aus-accounting-mcp: MCP facade over reviewed Australian accounting engines."""

from importlib.metadata import PackageNotFoundError, version

try:
    __version__ = version("aus-accounting-mcp")
except PackageNotFoundError:  # running from a source tree without installation
    __version__ = "0.0.0.dev0"

__author__ = "Ryan Duguid"

from .server import mcp, run_stdio

__all__ = ["mcp", "run_stdio"]
