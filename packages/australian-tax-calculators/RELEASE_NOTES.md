# v0.1.2

- Bind the release workflow to the checked current Release Policy main commit.
  Both historical pins are rejected by GitHub Actions despite remaining readable
  through the commit API. Versions 0.1.0 and 0.1.1 produced no release artifacts.
  Calculation behaviour is unchanged.

# v0.1.1

- Use the Release Policy commit already verified by the MCP release workflow.
  GitHub could not resolve the earlier policy reference, so 0.1.0 never built or
  published. Calculation behaviour is unchanged.

# v0.1.0

- Add six bounded calculation worksheets with official sources, explicit periods
  and scope confirmation.
- Refuse unsupported periods and malformed amounts. Keep reference retrieval and
  MCP transport outside the calculation engine.
