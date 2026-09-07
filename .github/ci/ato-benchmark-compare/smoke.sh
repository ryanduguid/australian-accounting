#!/usr/bin/env bash
# Wheel smoke for ato-benchmark-compare.
#
# The benchmark data ships as package data. A wheel that cannot find it produces
# a CLI that starts and then has nothing to compare against, so this runs a
# command that reads the dataset and one that runs the whole comparison path.
set -euo pipefail

# shellcheck source=.github/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/../lib.sh"

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

expect_contains "31% to 38%" \
  "$("$WHEEL_BIN/ato-benchmark-compare" show "Bakeries and hot bread shops")"

printf 'account,amount\nSales,1000000\nPurchases,320000\n' > "$work/pnl.csv"
printf 'account,bucket\nSales,turnover\nPurchases,cost_of_sales\n' > "$work/map.csv"
expect_contains "32.00%" "$("$WHEEL_BIN/ato-benchmark-compare" compare \
  --profit-and-loss "$work/pnl.csv" --mapping "$work/map.csv" \
  --industry "Bakeries and hot bread shops")"

run_sdist_tests "$work"
