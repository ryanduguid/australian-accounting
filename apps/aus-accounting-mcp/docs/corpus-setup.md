# Connect a corpus build to MCP retrieval

Aus Accounting MCP reads local corpus files. It does not fetch legislation or
rulings itself. Download the snapshot the
[corpus builder](https://github.com/ryanduguid/PaulGoodman) publishes, or build with it,
then point the server at the resulting files. Keep generated material outside every
source checkout. A successful search establishes retrieval, not current law or
the applicability of a passage.

## Download the published snapshot

This route needs no build. PaulGoodman's
[`corpus-2026-11-11` release](https://github.com/ryanduguid/PaulGoodman/releases/tag/corpus-2026-11-11)
holds the 9 principal Acts listed in the next section, each at its current compilation on
11 November 2026. Download `au-tax-legislation-corpus-dist-2026-11-11.tar.gz`
(6,227,238 bytes) and `SHA256SUMS` from it into a folder outside every source checkout,
then check and unpack the archive:

```bash
sha256sum -c SHA256SUMS
tar -xzf au-tax-legislation-corpus-dist-2026-11-11.tar.gz
```

In PowerShell, compare `(Get-FileHash au-tax-legislation-corpus-dist-2026-11-11.tar.gz).Hash`
with the digest in `SHA256SUMS`; `tar` is part of Windows 10 and later.

It unpacks to `dist/`, holding `sources.json`, `markdown/<register id>/sections.jsonl`
for each Act and `rates/rates.jsonl`: 9,439 section rows and 1,664 rate rows. Set
`AUS_ACCOUNTING_CORPUS_ROOT` to the unpacked `dist` folder as an absolute path, in the
`env` block of the server's entry in the MCP host's configuration, then restart the
server. Search 'general deductions' with `search_tax_legislation`, which returns
`C2004A05138:0035:8-1`, section 8-1 of the ITAA 1997, first, then read that `row_id`
with `read_tax_legislation_section`. Unlike the `quick` build below, the snapshot has a
rates index, so `search_tax_rates` works too, and `aus-accounting-mcp doctor` reports
the folder OK.

The snapshot is a finding aid derived from the Register's EPUB reading view, not the
authorised text. It holds no instruments, rulings or case law, so a 'no match' says
nothing about any other title. Its legislative content is licensed under
[CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) by the Federal Register of
Legislation; keep the attribution each row carries. The snapshot does not change: for
an Act's compilation after 11 November 2026, build it with `quick`.

## Build the principal Acts with one command

This is the shortest route to a working `search_tax_legislation` at each Act's current
compilation. It needs Rust,
Python 3.14 or later, `curl` and network access, and it fetches each Act from the Federal Register
of Legislation onto your machine. Choose an absent or empty folder outside every source
checkout:

```bash
git clone https://github.com/ryanduguid/PaulGoodman
cd PaulGoodman
cargo build --locked --release
python -m fadden quick --out ../tax-law
```

It builds and indexes 9 Acts: the ITAA 1997, ITAA 1936, GST Act, TAA 1953, FBTAA 1986,
*Superannuation Guarantee (Administration) Act 1992*, *Income Tax (Transitional
Provisions) Act 1997*, *Income Tax Rates Act 1986* and *Superannuation Industry
(Supervision) Act 1993*. It works in a sibling `<folder>.building` and moves the build
into your folder only once each Act holds rows and a query gets a hit. It then prints
that hit, the elapsed time and the setting for this server:

```text
AUS_ACCOUNTING_CORPUS_ROOT=<that folder as an absolute path>
```

Add that line to the `env` block of the server's entry in the MCP host's configuration,
then restart the server. Search 'general deductions' with `search_tax_legislation`, then
read a returned `row_id` with `read_tax_legislation_section`.

One run on 6 November 2026 took 116 seconds on a Windows 11 desktop PC with an Intel
Core i7-12700K, for 9 titles and 9,439 rows, and `search_tax_legislation` returned
section 8-1 of the ITAA 1997 first. Network speed and the Register's load will change
the figure, and one run is not a benchmark
([build evidence](https://github.com/ryanduguid/PaulGoodman/blob/main/docs/build-evidence.md#one-command-preset-build)).

The folder `quick` builds holds these 9 Acts only, so a 'no match' says nothing about any
other title. Unlike the snapshot, it has no rates index: `search_tax_rates` refuses it and, from 0.8.0,
`aus-accounting-mcp doctor` reports the folder as broken for that reason and exits 1. `define_tax_term` and
the 2 legislation tools work. The
[full build](https://github.com/ryanduguid/PaulGoodman/blob/main/BUILD.md)
covers the rest of the corpus and the rates index.

## First verify the connection with fabricated evidence

This route tests the rulings tools without building any legislation. Use Python 3.14 or later and uv. Clone both public repositories into sibling
directories. From `australian-accounting/apps/aus-accounting-mcp`, run:

```bash
uv run --locked --python 3.14 python scripts/check_corpus_setup.py --builder ../../../PaulGoodman --out ../../../../synthetic-corpus-setup
```

Choose an absent output directory. The script passes one fabricated HTML page
to the builder's existing rulings stage with a replacement fetch function. It
makes no network request. It then configures `AUS_ACCOUNTING_RULINGS_ROOT` for
this process, calls the registered `search_ato_rulings` and `read_ato_ruling`
tools, and saves `setup-report.json` beside the builder's immutable run files.
The fake title, future date and `example.invalid` address identify test material.
Its copied reuse-notice text exercises parsing only; no real ATO source or
permission is verified by this example. Package setup can download dependencies.

Inspect the report's source URL, paragraph reference, fetched date, source hash
and withdrawal status. `UNKNOWN` is not evidence that a ruling remains current.
Keep each run's `manifest.json`, `rulings.jsonl`, raw files and licence notice
together. The script refuses to overwrite an earlier destination.

## Use an authorised real build

For an expressly selected set of ATO documents, follow the builder's
[rulings procedure](https://github.com/ryanduguid/PaulGoodman/blob/main/docs/rulings.md).
It accepts a bounded target list and refuses unsupported document families or
missing reuse notices. This is a separate network operation, not part of the
fabricated example. Review exclusions before using its output.

Set `AUS_ACCOUNTING_RULINGS_ROOT` in the MCP host's server configuration to the
absolute run directory, or to a parent containing complete run directories.
Restart the server after changing its environment. Search a distinctive passage,
then read the returned `row_ref`. Compare the result with the retained source
and its current official version before relying on it.

For legislation, follow the builder's
[discover, resolve, download, extract and finalise stages](https://github.com/ryanduguid/PaulGoodman/blob/main/BUILD.md).
Point `AUS_ACCOUNTING_CORPUS_ROOT` at the finished corpus root containing
`sources.json` and `markdown/`, rather than at the source repository or one
individual JSONL file. Use `search_tax_legislation`, then
`read_tax_legislation_section` with its returned identifier.

`AUS_ACCOUNTING_LIBRARY_ROOT` is a third, separate option for an authorised
Markdown library. Do not convert builder JSONL into that format or rename one
root setting to another: the server already consumes the builder's formats.

The server sends excerpts back to its MCP host, whose model provider can receive
them. Never place client records or credentials in a corpus to test the setup.
