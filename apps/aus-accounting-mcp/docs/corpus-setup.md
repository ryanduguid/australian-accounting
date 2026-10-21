# Connect a corpus build to MCP retrieval

Aus Accounting MCP reads local corpus files. It does not fetch legislation or
rulings itself. Use the existing
[corpus builder](https://github.com/ryanduguid/au-tax-legislation-corpus), then
point the server at the resulting files. Keep generated material outside every
source checkout. A successful search establishes retrieval, not current law or
the applicability of a passage.

## Build the principal Acts with one command

This is the shortest route to a working `search_tax_legislation`. It needs Rust,
Python 3.14 or later, `curl` and network access, and it fetches each Act from the Federal Register
of Legislation onto your machine. Choose an absent or empty folder outside every source
checkout:

```bash
git clone https://github.com/ryanduguid/au-tax-legislation-corpus
cd au-tax-legislation-corpus
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
([build evidence](https://github.com/ryanduguid/au-tax-legislation-corpus/blob/main/docs/build-evidence.md#one-command-preset-build)).

The folder holds these 9 Acts only, so a 'no match' says nothing about any other
title. It has no rates index: `search_tax_rates` refuses it and, from the release after 0.7.1,
`aus-accounting-mcp doctor` reports the folder as broken for that reason and exits 1. `define_tax_term` and
the 2 legislation tools work. The
[full build](https://github.com/ryanduguid/au-tax-legislation-corpus/blob/main/BUILD.md)
covers the rest of the corpus and the rates index.

## Verify the native candidate with fabricated evidence

For the unpublished Rust candidates, select the corpus builder's native checkout
explicitly and build its scripted HTTP test variant outside the source tree:

```bash
cargo build --locked --manifest-path /absolute/path/to/au-tax-legislation-corpus/Cargo.toml --features fake-http --bin fadden --target-dir /absolute/path/to/disposable-builder-target
```

From this MCP component directory, run the Rust integration check:

```bash
FADDEN_SETUP_BINARY=/absolute/path/to/disposable-builder-target/debug/fadden cargo test --locked --test native_installation native_builder_fabricated_page_reaches_registered_mcp_citation_tools -- --ignored --nocapture
```

On Windows, the binary is `debug/fadden.exe`; set `$env:FADDEN_SETUP_BINARY` before
running the same Cargo command. Package setup can download dependencies. The
test itself uses scripted responses and loopback proxies. It first requires a
scripted 404 refusal, then sends the fabricated HTML through the actual native
builder. An ordinary builder without `fake-http` fails the probe. Do not use the
scripted test build for real capture.

The copied native MCP server reads the resulting immutable run through registered
search and citation tools with an empty working directory and PATH. Assertions
cover the full paragraph, original HTML digest, source URL, fetched date and
`UNKNOWN` withdrawal status. The URL has the ATO shape required by the builder;
the bytes, future date and title are fabricated and establish no ATO capture,
reuse permission, current law or advice. The independent builder is required, so
this test is explicitly ignored in ordinary runs. No changes to that checkout
are needed.

By default, the test cleans its temporary run files and report. To retain evidence
for inspection, also set `FADDEN_SETUP_OUT` to an absolute, absent directory whose
parent exists. Existing files, directories and symbolic links are refused. The
selected directory is retained even if a later check fails. On success it holds
`setup-report.json` with the complete search and read results beside
`run-synthetic/`, the builder's immutable run. Keep the manifest, JSONL, raw files
and licence notice together. The report marks the evidence as synthetic and
records its limits.

## Verify a compatible Python builder with fabricated evidence

This route tests the rulings tools without building any legislation. The retained Python check requires a compatible builder checkout that still
contains `fadden/rulings.py`. Use Python 3.14 or later and uv. Place the selected
checkouts in sibling directories. From `australian-accounting/apps/aus-accounting-mcp`, run:

```bash
uv run --locked --python 3.14 python scripts/check_corpus_setup.py --builder ../../../au-tax-legislation-corpus --out ../../../../synthetic-corpus-setup
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
[rulings procedure](https://github.com/ryanduguid/au-tax-legislation-corpus/blob/main/docs/rulings.md).
It accepts a bounded target list and refuses unsupported document families or
missing reuse notices. This is a separate network operation, not part of the
fabricated example. Review exclusions before using its output.

Set `AUS_ACCOUNTING_RULINGS_ROOT` in the MCP host's server configuration to the
absolute run directory, or to a parent containing complete run directories.
Restart the server after changing its environment. Search a distinctive passage,
then read the returned `row_ref`. Compare the result with the retained source
and its current official version before relying on it.

For legislation, follow the builder's
[discover, resolve, download, extract and finalise stages](https://github.com/ryanduguid/au-tax-legislation-corpus/blob/main/BUILD.md).
Point `AUS_ACCOUNTING_CORPUS_ROOT` at the finished corpus root containing
`sources.json` and `markdown/`, rather than at the source repository or one
individual JSONL file. Use `search_tax_legislation`, then
`read_tax_legislation_section` with its returned identifier.

`AUS_ACCOUNTING_LIBRARY_ROOT` is a third, separate option for an authorised
Markdown library. Do not convert builder JSONL into that format or rename one
root setting to another: the server already consumes the builder's formats.

The server sends excerpts back to its MCP host, whose model provider can receive
them. Never place client records or credentials in a corpus to test the setup.
