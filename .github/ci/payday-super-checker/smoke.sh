#!/usr/bin/env bash
# Wheel smoke for payday-super-checker.
#
# The law files ship as package data; a wheel that cannot find them produces a
# CLI that will not start. The import path reads data/profiles/*.json, package
# data the check path never touches, so a wheel that ships the law files and
# drops the profiles passes the first command and still cannot import a payroll
# export. Both commands of the documented 2-command flow run, the second
# against the first's output.
set -euo pipefail

# shellcheck source=.github/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/../lib.sh"

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

expect_ok_or_findings "$WHEEL_BIN/payday-super-check" \
  examples/sample_payrun.csv -o "$work/report.csv" --as-at 2026-08-10 \
  --confirm-transition-allocation

expect_ok_or_findings "$WHEEL_BIN/payday-super-check" import \
  --payroll tests/fixtures/importers/myob_payroll.csv \
  --super tests/fixtures/importers/myob_super.csv \
  -o "$work/contributions.csv"

expect_ok_or_findings "$WHEEL_BIN/payday-super-check" \
  "$work/contributions.csv" -o "$work/report_from_import.csv" \
  --as-at 2026-08-10 --confirm-transition-allocation

run_sdist_tests "$work"
