# Connect a corpus build to MCP retrieval

Aus Accounting MCP reads local corpus files. It does not fetch legislation or
rulings itself. Use the existing
[corpus builder](https://github.com/ryanduguid/au-tax-legislation-corpus), then
point the server at the resulting files. Keep generated material outside every
source checkout. A successful search establishes retrieval, not current law or
the applicability of a passage.

## First verify the connection with fabricated evidence

Use Python 3.14 or later and uv. Clone both public repositories into sibling
directories. From `australian-accounting/apps/aus-accounting-mcp`, run:

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
