# Security policy

## Supported versions

Security fixes are applied to the latest version of each component on the default
branch.

## Reporting a vulnerability

Use this repository's private vulnerability-reporting feature. Do not open a public
issue for a suspected vulnerability. Include a clear description, reproduction steps
with fabricated data, the likely impact and any suggested mitigation. A valid report is
acknowledged within seven days, and the fix and disclosure timeline is agreed with the
reporter.

## Boundaries

- No client data, credentials, tokens, workpapers or generated client reports are
  committed anywhere in this repository.
- Each component's own `SECURITY.md` states that tool's path, input and network
  boundaries. Consolidation does not change them.
- Publishing uses trusted publishing through per-component workflows and environments.
  No publishing secret is stored in the repository.
- Only root workflows are active. Workflow files inside imported component directories
  are historical records and are never run.
