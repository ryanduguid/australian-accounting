#!/usr/bin/env bash
# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

# Reconstruct the two bundle assets from the release the Python adapter pins.
# Run in both the pack and the independent verification jobs.
set -euo pipefail

app="$GITHUB_WORKSPACE/apps/aus-accounting-mcp"
adapter="$app/aus_accounting_mcp/adapters/tax.py"
version="$(sed -n 's/^CALCULATORS_VERSION = "\(.*\)"$/\1/p' "$adapter")"
policy="$(sed -n 's/.*release-binary\.yml@\([0-9a-f]\{40\}\)$/\1/p' \
    "$GITHUB_WORKSPACE/.github/workflows/release-australian-tax-calculators.yml")"
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ && "$policy" =~ ^[0-9a-f]{40}$ ]]

# Match the platform markers in the adapter's executable allowlist.
release_digest() {
    local platform="$1" digest
    digest="$(sed -n "/^RELEASED_DIGESTS = frozenset({$/,/^})$/s/^[[:space:]]*\"\([0-9a-f]\{64\}\)\",[[:space:]]*# $platform$/\1/p" "$adapter")"
    if [[ ! "$digest" =~ ^[0-9a-f]{64}$ ]]; then
        printf 'Expected one calculator digest for %s\n' "$platform" >&2
        return 1
    fi
    printf '%s\n' "$digest"
}
linux_digest="$(release_digest linux-musl)"
windows_digest="$(release_digest windows-msvc)"

name=australian-tax-calculators
tag="$name/v$version"
linux="$name-$version-x86_64-unknown-linux-musl.tar.gz"
windows="$name-$version-x86_64-pc-windows-msvc.zip"
commit="$(gh api "repos/$GITHUB_REPOSITORY/commits/$tag" --jq .sha)"
[[ "$commit" =~ ^[0-9a-f]{40}$ ]]

scratch="$(mktemp -d "$RUNNER_TEMP/bundle-calculators.XXXXXXXX")"
cd "$scratch"
gh release download "$tag" --repo "$GITHUB_REPOSITORY" \
    --pattern SHA256SUMS --pattern "$linux" --pattern "$windows"
pins=(--repo "$GITHUB_REPOSITORY" --source-ref "refs/tags/$tag" --source-digest "$commit"
      --signer-workflow ryanduguid/release-policy/.github/workflows/release-binary.yml
      --signer-digest "$policy")
gh attestation verify SHA256SUMS "${pins[@]}"
for asset in "$linux" "$windows"; do
    awk -v asset="$asset" '$2 == asset { print; found++ } END { if (found != 1) exit 1 }' \
        SHA256SUMS > checksum
    sha256sum --check --strict checksum
    gh attestation verify "$asset" "${pins[@]}"
done

# Extract only the named members, without taking any paths from the archives.
tar -xOzf "$linux" "$name" > "$name"
unzip -p "$windows" "$name.exe" > "$name.exe"
printf '%s  %s\n' "$linux_digest" "$name" "$windows_digest" "$name.exe" |
    sha256sum --check --strict
destination="$app/mcpb/bin"
mkdir "$destination"
install -m 0755 "$name" "$destination/$name"
install -m 0644 "$name.exe" "$destination/$name.exe"
