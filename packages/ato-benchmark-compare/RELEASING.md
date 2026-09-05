# Releasing

The monorepo's [GitHub Releases](https://github.com/ryanduguid/australian-accounting/releases) page is the canonical release history from v0.1.6 onward. Releases through v0.1.5 remain in the [source repository](https://github.com/ryanduguid/ato-benchmark-compare/releases). A separate changelog is intentionally not maintained.

Releases are built by GitHub Actions from an annotated tag on the exact `main` commit. Do not build or upload wheel and source-distribution assets by hand.

Before tagging:

1. Merge the release pull request and require every `main` check to pass.
2. Enable release immutability in the repository settings.
3. From an operator session authenticated with repository Administration read access, run:

    ```bash
    gh api -H "X-GitHub-Api-Version: 2026-03-10" repos/ryanduguid/australian-accounting/immutable-releases --jq .enabled
    ```

    Do not push the tag unless the output is exactly `true`. The Actions `GITHUB_TOKEN` cannot be granted repository Administration read access, so the tag workflow cannot perform this preflight itself.
4. Bump `__version__` in `atobenchmark/__init__.py`, the one place the version is written. `pyproject.toml` declares it dynamic and hatchling reads it from there, and `uv.lock` records the project as dynamic rather than pinning a number, so the only value to confirm against the `RELEASE_NOTES.md` heading is the module attribute. The release gate rejects the tag if the two disagree.
5. Create the namespaced annotated tag on current remote `main`, for example `git tag -a ato-benchmark-compare/v0.1.6 -m "ato-benchmark-compare v0.1.6"` (or `-s` when signing is configured), then push only that tag.

The workflow runs the locked tests, builds the wheel and source distribution once, generates an SPDX 2.3 SBOM for the wheel and `SHA256SUMS`, records GitHub provenance and an SBOM attestation, then publishes the completed draft. An existing release is never overwritten.

Verify the downloaded release with:

```bash
tag=ato-benchmark-compare/v0.1.6
repo=ryanduguid/australian-accounting
version="${tag#ato-benchmark-compare/v}"
wheel="ato_benchmark_compare-${version}-py3-none-any.whl"
release_commit="$(git ls-remote "https://github.com/$repo.git" "refs/tags/$tag^{}" | cut -f1)"
test -n "$release_commit"
release_dir="release-${tag//\//-}"
gh release download "$tag" -R "$repo" --dir "$release_dir"
cd "$release_dir"
sha256sum --check SHA256SUMS
gh attestation verify "$wheel" -R "$repo" \
  --source-digest "$release_commit" \
  --source-ref "refs/tags/$tag" \
  --signer-workflow ryanduguid/release-policy/.github/workflows/release-python.yml \
  --signer-digest 787db4590e725cfd37104c8a9dd9e75f7fd4c018
gh attestation verify "$wheel" -R "$repo" \
  --predicate-type https://spdx.dev/Document/v2.3 \
  --source-digest "$release_commit" \
  --source-ref "refs/tags/$tag" \
  --signer-workflow ryanduguid/release-policy/.github/workflows/release-python.yml \
  --signer-digest 787db4590e725cfd37104c8a9dd9e75f7fd4c018
gh release view "$tag" -R "$repo" --json isImmutable
gh release verify "$tag" -R "$repo"
gh release verify-asset "$tag" "$wheel" -R "$repo"
```

If any gate fails, inspect it before touching the tag or draft. Never move a published tag.
