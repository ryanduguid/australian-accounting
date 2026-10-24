# AU tax rates data

Key ATO rates and thresholds, one JSON file per figure in `data/`. Each record carries its source page, retrieval time and a verbatim quote from that page. `snapshots/` holds the visible text of each source page, so every quote can be checked offline.

## Record fields

| Field | Meaning |
| --- | --- |
| `id` | File name without `.json` |
| `label` | What the figure is |
| `value` | Number, ISO date, or a list of brackets for `tax_scale` |
| `unit` | `AUD`, `percent`, `cents_per_km`, `factor` (a plain multiplier such as an FBT gross-up rate), `date` or `tax_scale` |
| `period` | Any of `income_year` (like `2026-27`), `effective_from`, `effective_to`, `as_at` (ISO dates). `as_at` means the page gives no year, so the retrieval date is used |
| `source_url` | Public ato.gov.au page |
| `retrieved_at` | When the snapshot the quote was checked against was taken |
| `quote` | Verbatim page text containing the value |
| `period_quote` | Optional verbatim text naming the year, when the value quote does not |
| `pattern` | Optional regex with one group that re-reads the value from the page text |
| `notes` | Caveats, including anything not verified |

Income years must name consecutive calendar years, including century rollover such as `2099-00`. When both effective dates are present, the end cannot precede the start. Open-ended periods remain valid. The retrieval marker `as_at` does not set an effective-period boundary.

Numeric values must be finite and non-negative, with percentages between 0 and 100. Tax-scale bounds are whole dollar amounts, starting at zero and continuing without gaps or overlaps. Only the final bracket has no upper limit. A scale taxed from the first dollar, such as the foreign resident rates, is checked against the page wording "30c for each $1". Base tax and marginal rates follow the same numeric checks before the validator checks the arithmetic.

## How the data was built

1. Firecrawl search found each ATO page (24 September 2026).
2. Firecrawl JSON extraction pulled each value and a supporting quote.
3. `python rates.py snapshot` saved each page's visible text through the local Camofox browser.
4. Every value, quote and pattern was checked against those snapshots. Where Firecrawl's extraction and the page text disagreed, the page text won and the record's `notes` say so.
5. On 15 October 2026, 30 more figures were added. The figure list in [au-accounting](https://github.com/darrentmorgan/au-accounting) (MIT) suggested which figures to look for; no values were taken from it. Each page was saved with `fetch_text`, the same browser call `snapshot` uses, and each value and quote was read from that snapshot. Pages already in `snapshots/` were saved again, and their records' `retrieved_at` updated.

Snapshots store the page's `innerText` with non-breaking and other Unicode spaces turned into plain spaces. Quote checks collapse all whitespace, so table cells compare as plain text.

## Commands

The validator is a Rust command, `rates`. Build it with `cargo build --locked --release`, then run `target/release/rates` (or `cargo run --release --`) with a subcommand:

```
cargo test --locked           # schema, units, tax-scale arithmetic and brackets, quotes and patterns against snapshots
rates validate                # the same record checks, printed per record
rates snapshot                # refresh snapshots (needs the local nodriver browser in C:\Tools\nodriver-browser)
rates check [--out DIR]       # weekly check, writes au-tax-rates-check-YYYY-MM-DD.md
rates ato-tables              # compare figures with the tables the ATO's own calculators read
```

`rates` reads `data/` and `snapshots/` from the package folder it was built from, as `rates.py` read them from beside itself.

`check` re-fetches every source page and reports a finding for a changed value, a page that is missing or unreadable, or a quote or pattern that no longer matches. An invalid pattern or a captured value that cannot be parsed is also a finding; the report retains it and continues checking the remaining records. A record whose `effective_to` has passed or falls within 14 days is a finding unless a record with the same label starts the next day. The report also lists pages whose body changed since the committed snapshot, which is where a new year's figure usually first appears. Where present, the ATO site menu and QC reference mark the comparison boundaries. Header or footer text outside those boundaries is ignored; navigation and contents lists inside them can still produce wording alerts. It exits 1 when there are findings.

These are review prompts: a successful check does not establish that a value and its period come from the same table. Check the year, row and column against the source before relying on a figure. Links in `notes` provide supplementary context; `check` fetches only each record's `source_url`.

`ato-tables` downloads `TC2TAXRTE` and `TC9GENTAC`, the rate tables the ATO's online calculators read from `https://onlineservices.ato.gov.au/cdn/static-data/codes-tables/`. It compares each record those tables also hold, using the row in force on the first day of the record's period: the resident tax scales, the Medicare levy rate and low-income thresholds, the FBT rate, the general SG rate and the Division 7A benchmark rate. The other records have no counterpart there and are listed by id. A different figure, a missing or doubled row, or an unreadable table is a finding, and the command exits 1 when there are findings. The tables are undocumented and can change shape without notice, so check a finding against the record's source page before changing a figure.

After a real change, update the record, run `rates snapshot`, then `rates validate` and the tests, and commit.

GitHub Actions runs `cargo fmt --check`, clippy, the tests and `rates validate` on Ubuntu, Windows and macOS for pull requests and pushes to `main` (the `au-tax-rates-data` job in `.github/workflows/ci.yml`). These checks use committed snapshots and do not fetch live ATO pages or tables, or refresh the data.

## The Rust port

`rates` replaced `rates.py` at australian-accounting 24fcf24. It keeps the 4 subcommands, their options, output, exit codes and written files, including what Python did with malformed records: the same problems where `rates.py` reported one, and the same exception, by class and message, where it stopped. `src/rates_tests.rs` holds each of the 4 Python test files' tests; `tests/BEHAVIOURS.md` maps them. A differential harness ran both on 40,000 generated cases on Windows and 20,000 on Linux, comparing exit codes, stdout, stderr and every file left behind.

Where it differs:

- Help names the command `rates`, and the description lists `rates <command>` rather than `python rates.py <command>`.
- A run Python would stop with an uncaught exception prints only the traceback's header and its last line, the exception, and exits 1.
- A record `pattern` using a `\N{name}` escape is reported as `value pattern could not be read: undefined character name ...`; the port carries no Unicode name database.
- With `source_url`s of mixed types, both refuse to sort them with a `TypeError`, but the 2 types it names can differ. Python's own choice varies between runs with string hash randomisation.
- Python stops decoding deeply nested JSON where its C stack runs low. The port reproduces the depths measured for CPython 3.15 on Windows. On Linux, Python's limit moves with the size of the environment and the stack's address layout (64,414 to 65,216 levels measured with the default 8 MiB stack), and the port stops at a fixed 65,220, so a record nested about that deep can become a `RecursionError` in one and not the other, and the stack figure in the message (`used 8156 kB`) can differ. The depth can also differ on macOS or with another stack size.
- `ato-tables` connects with rustls and the operating system's certificate store. Network, TLS and HTTP failures still print `cannot read the ATO tables: ...` and exit 1, but their wording can differ from Python's `ssl` and `http.client`, as can the wording of a table body that fails to decode as UTF-16 or UTF-32.
- Command-line arguments that are not valid Unicode are read with replacement characters.
- A figure whose fixed-point form would run past 100,000,000 digits stops with `MemoryError` instead of being built. A tax-scale bound of 5,000 or more digits shown by `ato-tables` stops with the same integer digit-limit `ValueError` as Python, without first building the integer.
- A regular expression in a record's `pattern` that Python would warn about with `FutureWarning` is used without the warning.

The instant asset write-off record retains the source table's $20,000 limit from 1 July 2023. A separate [ATO legislation update](https://www.ato.gov.au/about-ato/new-legislation/in-detail/businesses/20000-dollars-instant-asset-write-off), checked on 28 September 2026, confirms that the limit is permanent from 1 July 2026 for eligible small businesses. For the full-cost deduction, each eligible depreciating asset must cost less than $20,000. Separate rules cover later additions to an asset's cost; other conditions and exclusions apply.

## Known gaps

- The car cost limit record uses an ATO newsroom item; next year's figure may be published at a different URL.
- Medicare levy low-income thresholds for 2026-27 were not on the ATO page when it was checked on 15 October 2026.
- The ATO foreign resident rates page lists tables up to 2025-26 only, so there is no 2026-27 foreign resident record.
- Figures quoted in words or millions (`$2 million`, `$50m`, `2.1` in a `($m)` column) have no `pattern`. For those records `check` reports a change only when the quote no longer matches.
