#!/usr/bin/env bash
# Apply docs/DISCOVERY.md to GitHub About. Requires gh, authenticated to
# ryanduguid/au-tax-mcp-server with repo metadata write access.
set -euo pipefail

REPO="ryanduguid/au-tax-mcp-server"
DESCRIPTION="Local MCP server for ATO small-business benchmarks, Payday Super 2026 review, refused Division 7A, and synthetic SBR fixtures. Not advice."
HOMEPAGE="https://ryanduguid.github.io/tools/australian-tax-ai-agents/"
TOPICS=(
  accounting
  accounting-ai
  agent-skills
  ato
  ato-benchmarks
  australian-tax
  australian-taxation
  claude-code
  codex
  cursor
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
topic_flags=()
for topic in "${TOPICS[@]}"; do
  topic_flags+=(--add-topic "$topic")
done
gh repo edit "$REPO" "${topic_flags[@]}"
echo "Updated $REPO About. Pin this repository from github.com/ryanduguid (Customize your pins)."
