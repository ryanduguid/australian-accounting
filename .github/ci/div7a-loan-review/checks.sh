#!/usr/bin/env bash
# Source guards for div7a-loan-review, run from the package directory.
#
# The benchmark rate table is a reviewed local dataset. A module that reaches
# the network for a rate would make the review aid depend on whatever a host
# answered on the day, so no production module may open one.
set -euo pipefail

if grep -rniE 'requests\.|urllib|https?://|httpx|aiohttp|urlopen|socket\.' div7aloan/*.py; then
  echo "::error::a div7aloan module reaches the network; the rate table stays local" >&2
  exit 1
fi
