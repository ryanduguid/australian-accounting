# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: AGPL-3.0-or-later
# Additional terms under AGPL-3.0 section 7(b): see NOTICE.

"""Locate the monorepo root for the tests that audit its workflows and policy."""

from __future__ import annotations

from pathlib import Path

import pytest


def repository_root() -> Path:
    # The package lives in apps/aus-accounting-mcp; the workflows these tests audit
    # live in the repository root .github directory above it.
    package_root = Path(__file__).resolve().parents[1]
    for candidate in (package_root, *package_root.parents):
        if (candidate / ".github" / "workflows").is_dir():
            return candidate
    pytest.skip("repository-root policy is not shipped in the Python source distribution")
