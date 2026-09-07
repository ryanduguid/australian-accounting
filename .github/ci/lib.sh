# Helpers shared by the per-package CI scripts. Source it, do not run it.
#
# ci-package.yml runs a package's smoke.sh from the package directory with
# WHEEL_BIN pointing at the clean environment the built wheel was installed
# into, so a smoke check exercises the wheel rather than the source tree.

# Fail with the whole output when a command did not print what it must print.
# Substring matching in the shell, so a checked command is never killed by a
# reader that stops early.
expect_contains() {
  local needle=$1 actual=$2
  if [[ "$actual" != *"$needle"* ]]; then
    printf 'expected to find %s in:\n%s\n' "$needle" "$actual" >&2
    return 1
  fi
}

# The engines exit 2 to report findings a reviewer must read, which is a pass
# for a smoke check: the command ran and reached a verdict.
expect_ok_or_findings() {
  local status=0
  "$@" || status=$?
  if [ "$status" -ne 0 ] && [ "$status" -ne 2 ]; then
    printf '%s exited %d, expected 0 or 2\n' "$1" "$status" >&2
    return 1
  fi
}

# The tests the source distribution ships must pass from the unpacked sdist, in
# an environment built from that sdist alone.
run_sdist_tests() {
  local work=$1 sdist="$1/sdist"
  mkdir -p "$sdist"
  tar -xzf dist/*.tar.gz -C "$sdist" --strip-components=1
  python3 -m venv "$work/sdist-venv"
  "$work/sdist-venv/bin/pip" install --quiet --upgrade pip
  "$work/sdist-venv/bin/pip" install --quiet -e "$sdist[dev]"
  (cd "$sdist" && "$work/sdist-venv/bin/pytest" -q)
}
