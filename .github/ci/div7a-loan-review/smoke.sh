#!/usr/bin/env bash
# Wheel smoke for div7a-loan-review. The benchmark rate table ships as package
# data, and the shipped sample registers must still behave as the documentation
# says: the compliant register passes, and the mixed register reports findings.
set -euo pipefail

# shellcheck source=.github/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/../lib.sh"

"$WHEEL_BIN/div7a-loan-review" rate --year 2026-27 --format json

"$WHEEL_BIN/div7a-loan-review" review \
  --input examples/sample_loans_myr_met.csv --year 2026-27

expect_ok_or_findings "$WHEEL_BIN/div7a-loan-review" review \
  --input examples/sample_loans_mixed.csv --year 2026-27
