# Domain context

## Purpose and boundaries

This repository provides a local Australian accounting MCP server. It is a
preparation aid for a qualified professional, not tax, legal, accounting,
financial, investment, BAS-agent, registered-tax-agent or assurance advice. It
does not lodge, approve, pay, or make compliance determinations.

## Glossary

- **MCP facade**: the local stdio server that exposes selected accounting tools
  through the Model Context Protocol.
- **Reviewed engine**: a separately published dependency that performs a
  supported calculation and whose version is reported with the result.
- **Computational MCP**: this server's primary role. It applies defined tests to
  facts supplied by the operator; it is not a hosted source-document store.
- **Operator-configured retrieval**: this server's secondary role. Search and read
  tools return cited excerpts from folders the operator configures and controls,
  and from nowhere else: no bundled corpus, no download, no hosted index and no
  record of a query. Retrieval never establishes calculation support, and a
  matching passage never extends a reviewed engine's scope.
- **Point-in-time copy**: what a retrieved provision or rate row is. It reflects
  the corpus build, not the register today, so it travels with its compilation
  number, compilation date and register page.
- **Document-retrieval MCP**: a hosted service that owns and serves its own
  corpus of rulings, legislation and other source documents. This server is not
  one. Do not present it as one, and do not present a retrieved passage as a
  determination, as advice or as a confirmation of current law.
- **Operator-supplied fact**: an input supplied to a tool invocation. Preserve
  its stated meaning; do not infer missing facts or silently replace it.
- **Structured result**: the machine-readable output of a tool. Reviewed-engine
  calculation results include calculations, citations, engine version and
  applicable warnings; refusals and synthetic fixtures use their own explicit
  schemas.
- **Refusal**: an explicit response that a requested calculation is not
  supported. Division 7A requests outside the delegated engine's reviewed
  s 109N and s 109E scope remain refused.
- **Synthetic SBR fixture**: deliberately artificial CTR or BAS-shaped data
  for tests and examples. It is never a lodgement or a real client payload.

## Control rules

Keep refusals and warnings visible in public interfaces. Preserve the
distinction between preparation support and professional advice, and require a
human decision for any consequential accounting action. Quote a retrieved
provision only with its citation and compilation date, keep the source licence
attribution with the text, and never present a stored copy as a confirmation of
current law.
