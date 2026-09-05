# Releasing

The monorepo's [GitHub Releases](https://github.com/ryanduguid/australian-accounting/releases) page is the canonical release history from `payday-super-checker/v0.1.3` onward. Releases through v0.1.2 remain in the archived [source repository](https://github.com/ryanduguid/payday-super-checker/releases). A separate changelog is intentionally not maintained.

The root workflow `.github/workflows/release-payday-super-checker.yml` builds
and publishes this component. GitHub treats the package's nested workflow files
as source-repository history.

Before tagging:

1. Merge the release pull request after its required checks pass.
2. Check that GitHub release immutability remains enabled:

   ```bash
   gh api -H "X-GitHub-Api-Version: 2026-03-10" \
     repos/ryanduguid/australian-accounting/immutable-releases --jq .enabled
   ```

3. Confirm that PyPI lists the trusted publisher for repository
   `ryanduguid/australian-accounting`, workflow
   `release-payday-super-checker.yml`, and environment
   `pypi-payday-super-checker`.
4. Create an annotated tag on the current remote `main` commit and push that tag:

   ```bash
   git tag -a payday-super-checker/v0.1.3 \
     -m "payday-super-checker v0.1.3"
   git push origin refs/tags/payday-super-checker/v0.1.3
   ```

The workflow runs the locked tests, builds the wheel and source distribution,
records provenance and SBOM attestations, publishes an immutable GitHub release,
and uploads the same distribution files to PyPI through OIDC.

Verify the release before installing it:

```bash
tag=payday-super-checker/v0.1.3
repo=ryanduguid/australian-accounting
release_commit=$(git ls-remote "https://github.com/$repo.git" \
  "refs/tags/$tag^{}" | cut -f1)
test -n "$release_commit"
gh release download "$tag" -R "$repo" --dir payday-super-checker-v0.1.3
cd payday-super-checker-v0.1.3
sha256sum --check SHA256SUMS
gh release verify "$tag" -R "$repo"
gh release verify-asset "$tag" payday_super_checker-0.1.3-py3-none-any.whl \
  -R "$repo"
gh attestation verify payday_super_checker-0.1.3-py3-none-any.whl \
  -R "$repo" \
  --source-digest "$release_commit" \
  --source-ref "refs/tags/$tag" \
  --signer-workflow ryanduguid/release-policy/.github/workflows/release-python.yml \
  --signer-digest 787db4590e725cfd37104c8a9dd9e75f7fd4c018
```

Inspect a failed gate before changing remote state. Published tags stay fixed.
