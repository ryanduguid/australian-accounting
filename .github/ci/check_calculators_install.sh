#!/usr/bin/env bash
# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: AGPL-3.0-or-later
# Additional terms under AGPL-3.0 section 7(b): see NOTICE.

# Runs the calculators README's Linux install command as a reader would, in a
# fresh home with no GitHub sign-in: it must install an executable that
# answers --version with the tag's version. Then runs it twice more with a
# curl that alters one download, the archive or its SHA256SUMS digest, as a
# tampered mirror would: each must refuse and install nothing. The MCP README
# must give the same command.
#
# usage: bash .github/ci/check_calculators_install.sh
set -euo pipefail

readme=packages/australian-tax-calculators/README.md
# The reviewed installer commit (release-policy v0.5.0); a README that names
# another fails here, before anything runs. Change it with the README pins.
pin=d7039efe6d8c441f1f213c674902909e80535599
# The whole line, flags included, so nothing else can ride along into bash -c.
pattern="^curl --proto '=https' --tlsv1\.2 -fsSL https://raw\.githubusercontent\.com/ryanduguid/release-policy/$pin/scripts/install\.sh \| sh -s -- ryanduguid/australian-accounting australian-tax-calculators/v[0-9]+\.[0-9]+\.[0-9]+ --modify-path\$"
command=$(grep -E "$pattern" "$readme") \
  || { echo "$readme gives no install command pinned to release-policy $pin" >&2; exit 1; }
[ "$(printf '%s\n' "$command" | wc -l)" -eq 1 ] || { echo "$readme gives more than one install command" >&2; exit 1; }
grep -qxF -- "$command" apps/aus-accounting-mcp/README.md \
  || { echo "apps/aus-accounting-mcp/README.md does not give $readme's install command" >&2; exit 1; }
version=$(printf '%s\n' "$command" | sed -E 's|.* australian-tax-calculators/v([0-9.]+) .*|\1|')

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
real_curl=$(command -v curl)

# Runs the command in a new home; the remaining arguments go before it in env.
run() {
  home=$work/home-$1
  shift
  mkdir -p "$home" "$home/gh"
  env -u GH_TOKEN -u GITHUB_TOKEN -u XDG_DATA_HOME HOME="$home" SHELL=/bin/bash \
    GH_CONFIG_DIR="$home/gh" "$@" bash -c "$command"
}

run clean
installed=$work/home-clean/.local/share/australian-tax-calculators/australian-tax-calculators
[ "$("$installed" --version)" = "australian-tax-calculators $version" ]
grep -q 'added by release-policy install.sh' "$work/home-clean/.bashrc"

for altered in archive sums; do
  shim=$work/shim-$altered
  mkdir -p "$shim"
  # The real curl, then one change to the file after -o.
  cat > "$shim/curl" <<EOF
#!/bin/sh
"$real_curl" "\$@" || exit \$?
previous=
for argument in "\$@"; do
  if [ "\$previous" = -o ]; then
    case "$altered:\$argument" in
      archive:*.tar.gz) printf X | dd of="\$argument" bs=1 seek=100 conv=notrunc 2>/dev/null ;;
      sums:*/SHA256SUMS)
        awk '/linux-musl[.]tar[.]gz\$/ { c = substr(\$0, 1, 1); \$0 = (c == "0" ? "1" : "0") substr(\$0, 2) } { print }' \
          "\$argument" > "\$argument.new" && mv "\$argument.new" "\$argument" ;;
    esac
  fi
  previous=\$argument
done
EOF
  chmod +x "$shim/curl"
  if run "$altered" PATH="$shim:$PATH" 2> "$work/$altered.err"; then
    echo "installed with an altered $altered" >&2
    exit 1
  fi
  cat "$work/$altered.err"
  grep -q 'failed provenance verification' "$work/$altered.err"
  [ ! -e "$work/home-$altered/.local/share/australian-tax-calculators" ]
  [ ! -e "$work/home-$altered/.bashrc" ]
done
echo "The install command installs $version and refuses an altered archive or SHA256SUMS."
