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
- **Computational MCP**: this server's role. It applies defined tests to facts
  supplied by the operator; it is not a hosted source-document store.
- **Document-retrieval MCP**: a separate tool category for locating rulings,
  legislation and other source documents. Do not present this server as one.
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
  for tests and examples. It is never a lodgment or a real client payload.

## Control rules

Keep refusals and warnings visible in public interfaces. Preserve the
distinction between preparation support and professional advice, and require a
human decision for any consequential accounting action.
