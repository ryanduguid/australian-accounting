#!/usr/bin/env bash
# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: AGPL-3.0-or-later
# Additional terms under AGPL-3.0 section 7(b): see NOTICE.

# Apply docs/DISCOVERY.md to GitHub About. Requires gh, authenticated to
# ryanduguid/australian-accounting with repo metadata write access.
set -euo pipefail

REPO="ryanduguid/australian-accounting"
DESCRIPTION="Local MCP for ATO benchmarks, Payday Super timing, limited Division 7A loan reviews, 148 bounded tax and super worksheets and cited local Markdown retrieval. Synthetic SBR fixtures. Not advice."
HOMEPAGE="https://duguid.com.au/tools/australian-tax-ai-agents/"
TOPICS=(
  accounting
  accounting-ai
  agent-skills
  ato
  ato-benchmarks
  australian-accounting
  australian-tax
  australian-taxation
  claude-code
  codex
  cursor
  division-7a
  mcp
  mcp-server
  model-context-protocol
  payday-super
  python
  tax-prep
)

if ! command -v gh >/dev/null 2>&1; then
  echo "gh is required" >&2
  exit 1
fi

gh repo edit "$REPO" --description "$DESCRIPTION" --homepage "$HOMEPAGE"
topic_fields=()
for topic in "${TOPICS[@]}"; do
  topic_fields+=(-f "names[]=$topic")
done
# Replace the complete topic set so removed discovery terms do not linger.
gh api --method PUT "repos/$REPO/topics" "${topic_fields[@]}"
echo "Updated $REPO About."
