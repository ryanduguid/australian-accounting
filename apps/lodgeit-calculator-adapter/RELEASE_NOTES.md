# Release notes

## Unreleased, 0.1.0

First cut. Not released, not published, not tagged.

- An optional adapter for the LodgeiT Labs calculator constellation, off by
  default, with an allowlist, bounded transport and no redirect following.
- Exact decimals in both directions, checked against the bytes on the wire.
- Reviewed contract snapshots for the calculators and for Fano, with drift
  reporting that never writes a snapshot.
- Digest-bound evidence records that separate deterministic calculation content
  from observation metadata.
- Comparison trials for Division 7A, one FBT benefit category, accounting
  depreciation and Fano, with evaluation outcomes that are deliberately not
  accounting-review states.
- A CLI.
- Ported from Python to Rust; the Python package is retired. The README
  lists the deliberate differences. The Python package's MCP surface was not
  ported.
- Relicensed from Apache-2.0 to FSL-1.1-ALv2 (Functional Source License, with
  Apache-2.0 as the future licence). Released versions keep their licence.

- A manual pass over the Division 7A and FBT car cases, recorded in
  `docs/differential-2026-11-06.md`. The Division 7A cases are corrected to
  s 109E(6): their remaining terms were one year short.

The live service was last read on 6 November 2026. The contract snapshot in
`contracts/` stays the reading of 18 September 2026.
