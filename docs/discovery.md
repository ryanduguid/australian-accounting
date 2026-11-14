# Documentation discovery

The root `context7.json` selects the component documentation and gives coding
agents the same calculation and privacy boundaries as the repository. It uses
the configuration described in the [Context7 library-owner documentation](https://context7.com/docs/library-owners).

This file prepares documentation for discovery. It does not submit the
repository, claim a listing, refresh an index or establish that any component
has been published. Those are separate external actions.

Use each component's README for its version, installation, supported inputs
and JSON or MCP interface. The native engine source and a released Python/MCP
adapter may expose different capabilities. Preserve sources, warnings and
refusals when presenting an answer, including through another adapter.
