#!/usr/bin/env bash
# Wheel smoke for the-wip-tally. A schedule built from the shipped example must
# still reach the documented contract asset total.
set -euo pipefail

# shellcheck source=.github/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/../lib.sh"

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

expect_contains "221,000.00" "$("$WHEEL_BIN/wip-tally" schedule \
  examples/sample_contracts.csv --as-at 2026-08-31 -o "$work/wip-schedule.csv")"
