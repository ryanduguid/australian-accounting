# v0.9.0

## Fixed

- `review_div7a_loan` refuses a remaining term longer than the Act allows and gives no
  figure. Before, such a loan got a minimum yearly repayment worked from the longer
  term, which is lower than the Act requires. It now refuses a remaining term above the
  s 109N(3) maximum (`REFUSED_REMAINING_TERM_ABOVE_MAXIMUM_TERM`), above the loan's own
  term (`REFUSED_REMAINING_TERM_ABOVE_LOAN_TERM`), or above that term less the years
  elapsed since the loan year under s 109E(6)
  (`REFUSED_REMAINING_TERM_ABOVE_TERM_LESS_YEARS_ELAPSED`). A 7-year loan made in
  2024-25 accepts 7 for 2025-26 and 6 for 2026-27, and refuses 7 for 2026-27. The pinned
  `div7a-loan-review` 0.3.2 has none of these refusals, so this server applies them
  after the engine's answer, with the engine's codes and wording. A call that returned
  a figure in 0.8.0 can now be refused
  ([#626](https://github.com/ryanduguid/australian-accounting/pull/626),
  [#627](https://github.com/ryanduguid/australian-accounting/pull/627)).
- With no legislation corpus configured, the corpus tools' refusal names PaulGoodman's
  published `corpus-2026-11-11` snapshot first, which needs no build, and building one
  second ([#609](https://github.com/ryanduguid/australian-accounting/pull/609)).
- The README is the PyPI description, and its 10 relative links resolved against
  pypi.org. They are now absolute GitHub links
  ([#609](https://github.com/ryanduguid/australian-accounting/pull/609)).
- The component lock that builds this release takes `opentelemetry-api` 1.45.2, patched
  for GHSA-mf5v-w3w3-f6cg. The wheel's dependency ranges are unchanged, so an install
  resolves its own versions
  ([#646](https://github.com/ryanduguid/australian-accounting/pull/646)).

## Changed

- `search_tax_legislation` and `read_tax_legislation_section` with `as_at` answer each
  title from the compilation whose window, from its compilation date to the date it was
  superseded, covers the date. Earlier compilations are read from an optional
  `compilations/<name>/markdown/` folder beside `markdown/`; calls without `as_at`,
  `define_tax_term` and `search_tax_rates` still read only `markdown/`. Overlapping
  windows are an input error. When no window covers the date the title is refused as in
  0.8.0, and a title held in more than one compilation also lists its windows in the new
  `held_compilations`. An undated read of a title that `compilations/` also holds carries
  a caveat. The published snapshot holds one compilation per title, so with it results
  are unchanged apart from the refusal notice and the tool descriptions
  ([#644](https://github.com/ryanduguid/australian-accounting/pull/644)).
- The `review_div7a_loan` descriptions of `year_loan_made`, `maximum_term_years` and
  `remaining_term_years` give the s 109E(6) limit and, for a loan extended under
  s 109E(3A), the s 109E(3B) loan year and the term as the ATO works it: the agreed term
  counted from when the loan was first made, at most 25 years, less the time it had
  already run. A loan made on 1 July 2022 and extended on 1 July 2024 is entered with 23,
  not 25 ([#636](https://github.com/ryanduguid/australian-accounting/pull/636)).
- Aligns citation, compatibility, bundle and MCP Registry metadata with 0.9.0.

## Added

- `get_reviewed_rate` holds 74 reviewed records, adding the Division 296 tax rate of
  15% on taxable superannuation earnings and the further 10% on very large balances,
  each from 1 July 2026 ([#608](https://github.com/ryanduguid/australian-accounting/pull/608)).
  This server has no Division 296 worksheet.
- An MCP Bundle for Claude Desktop, `aus-accounting-mcp-bundle/v0.9.0`, published as its
  own release once 0.9.0 is on PyPI. It pins `aus-accounting-mcp==0.9.0`.

## Not in this release

- FBT return dates. `get_lodgment_due_dates` still lists only BAS, IAS and income tax
  return dates: this server pins `lodgment-calendar` 0.1.0, and the tool gains FBT
  returns only with a later pin move to 0.2.0 or later.
- Worksheets and years added to the Rust calculators since 0.2.0, including 2026-27 for
  the Medicare levy, the Division 296 estimate and Schedules 5 and 15 withholding. They
  run through the `au-accounting` command; this server pins the Python
  `australian-tax-calculators` 0.2.0.
- Retains `payday-super-checker` 0.2.3, `ato-benchmark-compare` 0.2.2,
  `div7a-loan-review` 0.3.2, `australian-tax-calculators` 0.2.0, `gst-review` 0.1.0 and
  `lodgment-calendar` 0.1.0 unchanged.

## Known issues

- `calculate_tax_worksheet` refuses the Medicare levy for 2026-27, because the pinned
  calculators 0.2.0 hold no 2026-27 thresholds.
- `payg_withholding` applies no Medicare levy variation for scales 2 and 6, has no
  scale 4 and takes only weekly, fortnightly and monthly pays.
- `search_ato_rulings` refuses a rulings run whose documents are labelled as Law
  Companion Rulings, a family it does not list. Runs built so far label them as
  Practical Compliance Guidelines.
- Dated reads from `compilations/` have been tested only on fabricated corpora: no
  published snapshot holds earlier compilations yet, `doctor` does not inspect
  `compilations/` and the cost of a dated search over many compilations is unmeasured.
- `reconcile_bas` lists differences from lodged figures but does not say whether a
  correction can be made on a later BAS within the time and value limits of
  LI 2023/32.
- The bundle release carries a SHA-256 checksum but no build provenance attestation.
- `docs/firm-assessment.md` gives no supplier details under APP 1.7 to 1.9.

# v0.8.0

- Licence: the Ryan Duguid Public Attribution Licence 1.0
  (`LicenseRef-Duguid-Public-Attribution-1.0`) replaces FSL-1.1-ALv2. It is the
  FSL-1.1-ALv2 text unchanged plus a Public Attribution Condition on the FSL grant: a
  copy, port or derivative made available outside your organisation must credit Ryan
  Duguid publicly, with a link to the repository. Internal use is exempt, and the
  condition does not apply to a version's Apache-2.0 licence. 0.7.0 and 0.7.1 keep
  FSL-1.1-ALv2. Source files now carry copyright and SPDX headers
  ([#569](https://github.com/ryanduguid/australian-accounting/pull/569),
  [#564](https://github.com/ryanduguid/australian-accounting/pull/564)).
- Breaking: `search_tax_rates` leaves out rows from a compilation the corpus marks as
  superseded, as `search_tax_legislation` and `define_tax_term` already did. Pass
  `in_force_only: false` to get 0.7.1's rows back; each superseded row then carries a
  caveat. A row whose currency the corpus did not record is kept either way. Each row
  gains `version_is_current`: true, false or null
  ([#528](https://github.com/ryanduguid/australian-accounting/pull/528)).
- Breaking: all 4 searches (`search_tax_legislation`, `search_tax_rates`,
  `search_accounting_library` and `search_ato_rulings`) match whole words only, so a word
  inside a longer word no longer matches: 'deduct' does not find 'deductible'. A modal
  verb and the 'not' after it count as one word, so 'must lodge' never fully matches
  text holding only 'must not lodge', and the reverse holds too. In the legislation and
  rate searches such a row can still appear in `partial_hits`, because a modal's
  opposite form counts towards their 2-word threshold; the library and rulings searches
  have no partial list, so there it is simply not a match
  ([#585](https://github.com/ryanduguid/australian-accounting/pull/585)).
- `search_tax_legislation` and `search_tax_rates` take a plain question. Stop words
  such as 'what' and 'the' are dropped and listed in the new `dropped_words`; words of
  legal force such as 'must', 'may' and 'not' are kept. Room left on a page goes to the
  new `partial_hits`: rows holding at least 2 of the words, each labelled
  `match: "partial"` with `matched_words` and `missing_words`. A partial hit shows
  shared words, not an answer
  ([#585](https://github.com/ryanduguid/australian-accounting/pull/585)).
- Adds `get_reviewed_rate`, which returns one reviewed ATO rate or threshold for a date
  with its ATO page, capture date and a verbatim quote holding the figure, from 72
  bundled records covering 2025-26 and 2026-27. Any other date, a date outside the
  record's period or an unknown id is `REFUSED` with no figure and a `refusal` code:
  `UNKNOWN_ID`, `OUTSIDE_COVERED_YEARS`, `OUTSIDE_RECORD_PERIOD` or
  `NO_RECORDED_PERIOD`. `get_data_freshness` reports the records as the new
  `reviewed_ato_rates` dataset, and its `tool` argument accepts `get_reviewed_rate`
  ([#585](https://github.com/ryanduguid/australian-accounting/pull/585)).
- Adds `validate_abn` and `validate_acn`, which test a check digit and return
  `CHECK_DIGIT_VALID`, `CHECK_DIGIT_INVALID` or `MALFORMED`. They contact no register,
  and a valid check digit proves neither ABR nor GST registration
  ([#523](https://github.com/ryanduguid/australian-accounting/pull/523)).
- Adds `aus-accounting-mcp doctor`, which prints the version, the periods each bundled
  dataset records and each configured retrieval folder with its row count, and exits 1
  if a configured folder is broken. A legislation corpus with no rates index counts as
  broken, because `search_tax_rates` fails without one
  ([#585](https://github.com/ryanduguid/australian-accounting/pull/585)).
- `search_tax_legislation` and `read_tax_legislation_section` take `as_at`, a date the
  user names, and answer from each title's compilation window: `captured`,
  `before_compilation`, `superseded` or `after_retrieval`. Text comes back only for
  `captured`; every other title is listed in the new `refusals` with its status and
  `next_step`, a search adds `as_at_summary`, and each row carries `as_at`. `captured`
  is not a finding that the text was the law on that date. With `as_at`,
  `in_force_only` is ignored and `act` also takes a Register ID. Calls without `as_at`
  are unaffected by it
  ([#592](https://github.com/ryanduguid/australian-accounting/pull/592),
  [#594](https://github.com/ryanduguid/australian-accounting/pull/594)).
- `calculate_tax_worksheet` takes `file_note: true` and then also returns `file_note`, a
  Markdown note restating the result for a workpaper. Without it the result is
  unchanged, and a refused worksheet returns only its refusal
  ([#568](https://github.com/ryanduguid/australian-accounting/pull/568)).
- `search_accounting_library` ranks matches: the query phrase in the heading in force
  at a line first, then every word in that heading, then the phrase in the line, then
  the words anywhere in it; ties keep
  folder and file order. The order is presentation, not authority. It now reads the
  whole library on every search; 3.0 to 3.8 seconds were measured at the 1,000-file,
  64 MB limit ([#536](https://github.com/ryanduguid/australian-accounting/pull/536)).
- `search_ato_rulings` ranks a paragraph whose captured page held a withdrawal notice
  after those without, within the same tier; none is left out
  ([#528](https://github.com/ryanduguid/australian-accounting/pull/528)).
- `get_ato_benchmarks`: when no business type matches, the error names
  `list_ato_benchmark_industries` and the dataset year instead of the engine's
  command-line command, which an MCP client cannot run
  ([#536](https://github.com/ryanduguid/australian-accounting/pull/536)).
- Payday Super: `calc_payday_super_deadline`, and each contribution row of
  `review_payday_super_contributions` and `build_payday_super_evidence_pack`, take
  `returned_by_fund`. True refuses the review with an input error naming the row,
  because the pinned checker cannot assess a return and would read the receipt as kept
  ([#582](https://github.com/ryanduguid/australian-accounting/pull/582)).
- Payday Super: every `ON_TIME` verdict, and every row listing `ON_TIME` in
  `horizon_verdicts`, carries `verdict_qualification`
  (`code: RETURNED_CONTRIBUTIONS_NOT_ASSESSED`, `returned_contributions_assessed: false`,
  `assumes_fund_kept_every_receipt: true`) and a caveat saying returned contributions
  were not assessed. An evidence pack with an `ON_TIME` verdict or candidate, including
  every exit code 0, carries it with a `statement`; its `exceptions.json` gains the
  field, `practitioner-review.md` and `decision-log.md` open with the notice and
  `report.csv` rows carry the caveat. No verdict, exit code or figure changes
  ([#524](https://github.com/ryanduguid/australian-accounting/pull/524)).
- Payday Super: `experimental_sgc_low` and `experimental_sgc_high`, the scope resource
  and the disclaimer say the estimates exclude choice loading, the maximum
  contributions base and post-assessment penalties. No figure changes
  ([#529](https://github.com/ryanduguid/australian-accounting/pull/529)). The
  `first_to_fund` description covers the first eligible contribution after employment
  starts or recommences, including a worker rehired under a new arrangement, as well as
  a fund switch ([#600](https://github.com/ryanduguid/australian-accounting/pull/600)).
- The server instructions tell the model, when its own working or an operator's figure
  differs from a tool figure, to recheck the inputs, state both figures and flag the
  difference, never substituting either silently
  ([#559](https://github.com/ryanduguid/australian-accounting/pull/559)). They also
  cover partial hits, `as_at`, `returned_by_fund`, `get_reviewed_rate` and the
  identifier tools, and link DrDebits at its renamed repository
  ([#598](https://github.com/ryanduguid/australian-accounting/pull/598)). The scope
  resource lists the 3 new tools and adds the returned-contribution limit to
  `payday_limitations`; the disclaimer resource adds the returned-contribution and
  SG-charge sentences.
- What a strict consumer will notice: `tools/list` holds 25 tools, up from 22. New
  fields always present: `partial_hits` and `dropped_words` on legislation and rate
  searches, and `version_is_current` on rate rows. New fields present only in some
  results: `verdict_qualification`, which the output schema requires on an `ON_TIME`
  row, on a row listing `ON_TIME` in `horizon_verdicts` and on a pack with
  `review_exit_code` 0; `file_note`; and, with `as_at`, `refusals`, `as_at_summary` and
  `as_at`. New optional inputs: `returned_by_fund`, `file_note`, `as_at` on the 2
  legislation tools and `in_force_only` on `search_tax_rates`. A Payday Super row or
  evidence pack that states `ON_TIME` carries one more caveat, such a pack's files
  change as above, and `get_data_freshness` reports one more dataset.
- Not in this release: FBT return dates. `get_lodgment_due_dates` still lists only BAS,
  IAS and income tax return dates. FBT return dates are in the `lodgment-calendar`
  source ([#523](https://github.com/ryanduguid/australian-accounting/pull/523)) but not
  in the pinned `lodgment-calendar` 0.1.0 from PyPI, which has no FBT module; they reach
  this server only after a `lodgment-calendar` release and a pin move.
- Not in this release: assessment of returned contributions. The pinned
  `payday-super-checker` 0.2.3 has no input for a contribution the fund returned, so
  this release refuses rows flagged `returned_by_fund` and qualifies every `ON_TIME`
  instead. The checker's `returned_by_fund` column and `--events` file exist only in its
  unreleased 1.0.0 source. Check fund records for returns before relying on `ON_TIME`.
- Adds an MCP Bundle for Claude Desktop, published as its own release
  `aus-accounting-mcp-bundle/v0.8.0` once 0.8.0 is on PyPI. It pins
  `aus-accounting-mcp==0.8.0`, which uv installs on first launch, and offers the 3
  research folders as optional settings. Its workflow checks the bundle against this
  release's source and starts it before publishing; no Claude Desktop install has
  been tested.
- Bundles the 72 reviewed ATO rate records as package data
  (`aus_accounting_mcp/rate_records/`).
- Retains `payday-super-checker` 0.2.3, `ato-benchmark-compare` 0.2.2,
  `div7a-loan-review` 0.3.2, `australian-tax-calculators` 0.2.0, `gst-review` 0.1.0 and
  `lodgment-calendar` 0.1.0 unchanged.
- Aligns citation, compatibility and MCP Registry metadata with 0.8.0.

# v0.7.1

- Pins `payday-super-checker` 0.2.3 (was 0.2.2). The checker's only change is to its
  Employment Hero export profile: a Beam status of Reconciled is no longer read as sent
  to the fund, so such a row gets no remitted date. This server's tools take structured
  contributions and import no vendor export, so no tool output changes except
  `engine_version`, now 0.2.3 for the payday tools. Anyone who ran the checker's own
  import on an Employment Hero export with 0.1.3 to 0.2.2 should re-run it; see the checker's
  [v0.2.3 release](https://github.com/ryanduguid/australian-accounting/releases/tag/payday-super-checker/v0.2.3).
- No other check, comparison or review changed. Retains `ato-benchmark-compare` 0.2.2,
  `div7a-loan-review` 0.3.2, `australian-tax-calculators` 0.2.0, `gst-review` 0.1.0 and
  `lodgment-calendar` 0.1.0 unchanged.
- Aligns citation, compatibility and MCP Registry metadata with 0.7.1.

# v0.7.0

- Adds `get_lodgment_due_dates`, which lists ATO 2026-27 lodgment program due dates
  for monthly and quarterly BAS and IAS and for 2025-26 income tax returns, by entity
  type and by self-lodgement or registered tax or BAS agent. Each row carries its rule,
  ATO source and review date, and a date that depends on a fact not supplied is
  `UNKNOWN` with that fact named. `extra_holidays` and `holidays_complete_until` let the
  operator settle dates after the ATO holiday table ends on 30 June 2027. It delegates to the new `lodgment-calendar` 0.1.0
  engine and lodges nothing.
- Adds `get_data_freshness`, which reports each bundled dataset's recorded effective
  periods, engine and server versions, source URLs and retrieval and review dates, with
  `UNKNOWN` where an engine records none. With `as_at` each dataset reports
  `INSIDE_RECORDED_PERIOD`, `OUTSIDE_RECORDED_PERIOD` or `UNKNOWN`, and `tool` limits the
  report to the datasets one tool reads. `currency` is always `NOT_ESTABLISHED`: the
  tool reads only what the installed engines record, looks nothing up and predicts no
  next change, and a cached `uvx` release can be older than the latest.
- Lists `lodgment-calendar` in the component-versions resource, the disclaimer resource
  and the scope resource.
- Retains `payday-super-checker` 0.2.2, `ato-benchmark-compare` 0.2.2,
  `div7a-loan-review` 0.3.2, `australian-tax-calculators` 0.2.0 and `gst-review` 0.1.0
  unchanged.
- Aligns citation, compatibility and MCP Registry metadata with 0.7.0.
- Relicensed from Apache-2.0 to FSL-1.1-ALv2 (Functional Source License, with
  Apache-2.0 as the future licence). Released versions keep their licence.

# v0.6.0

- Adds `review_gst_coding`, which gives each coded bank line one review state
  against the supplied GST registration dates, and `reconcile_bas`, which compares
  lodged G1, G2, G3, G10, G11, 1A and 1B figures with ledger lines per period on
  the cash or accruals basis. Both delegate to the new `gst-review` 0.1.0 engine;
  neither classifies a supply or prepares, revises or lodges a BAS.
- Lists `gst-review` in the component-versions resource, the disclaimer resource
  and the scope resource.
- Pins the Apache-2.0 releases of three engines: `payday-super-checker` 0.2.2
  (was 0.1.9), `ato-benchmark-compare` 0.2.2 (was 0.1.11) and
  `div7a-loan-review` 0.3.2 (was 0.2.0). Each now ships a `NOTICE` file and
  requires Python 3.14 or later, as this package has since 0.4.0. Their earlier
  releases stay under MIT.
- No check, comparison or review changed between the old and new pins. The
  releases in between (`payday-super-checker` 0.1.10 and `ato-benchmark-compare`
  0.1.12) raised the Python floor and, for the benchmark engine, changed the
  build backend. Tool results now report `engine_version` 0.2.2, 0.2.2 and
  0.3.2.
- Aligns citation, compatibility and MCP Registry metadata with 0.6.0.
- Retains `australian-tax-calculators` 0.2.0 unchanged.

# v0.5.0

- Pins the published `australian-tax-calculators` 0.2.0, bringing the catalogue
  to fourteen worksheets with Schedule 8 study-loan withholding, the bounded
  annual Medicare levy, low income tax offset and Division 293. Calculation
  logic, warnings and scope refusals remain owned by the engine.
- Keeps annual study-loan repayment separate from payroll withholding. The
  Medicare worksheet does not apply payroll variations or section 9A, and
  still refuses 2026-27 until its thresholds are published.
- Checks every advertised worksheet and supported period from installed PyPI
  pins before publication, in addition to the existing demonstration check.
- Adds a synthetic corpus-builder/search/read walkthrough with retained
  provenance and no source-network requests.
- Aligns citation, compatibility and MCP Registry metadata with 0.5.0.
- Retains `payday-super-checker` 0.1.9, `ato-benchmark-compare` 0.1.11 and
  `div7a-loan-review` 0.2.0 unchanged.

# v0.4.0

- Breaking: requires Python 3.14 or later. 0.3.0 remains the last release for Python 3.11 to 3.13 ([#355](https://github.com/ryanduguid/australian-accounting/pull/355)).
- Requires `mcp` 2.3.0 or later ([#370](https://github.com/ryanduguid/australian-accounting/pull/370)).
- Licence: Apache-2.0 replaces MIT. Anyone who redistributes this package or a
  derivative must keep the copyright notices in `NOTICE`. Versions already released stay
  under MIT.
- Breaking: money arguments accept plain decimal notation only: ASCII digits, an
  optional leading minus and at most 2 decimal places, with surrounding whitespace
  ignored. Exponents such as `1e3`, a leading plus sign, underscores, non-ASCII digits,
  `.5` and `12.` are now refused with the existing "is not a decimal amount" error;
  `1e30` no longer reaches the magnitude check. An optional amount must be omitted or
  `null`; a blank string is refused instead of being read as omitted. The AUD
  1000000000000.00 and 2 decimal place limits are unchanged. Every money argument's
  schema, including the contribution rows and worksheet facts, now publishes the pattern
  `^\s*-?[0-9]+(?:\.[0-9]{1,2})?\s*$`, so a client that validates schemas refuses these
  forms before calling.
- The pins of `div7a-loan-review` 0.2.0, `payday-super-checker` 0.1.9, `ato-benchmark-compare` 0.1.11 and
  `australian-tax-calculators` 0.1.8 are unchanged.
- Align the release, citation, compatibility and MCP Registry metadata to 0.4.0.

# v0.3.0

- Breaking: requires Python 3.11 or later. CPython 3.10 reached end of life on 1 October 2026; 0.2.12 remains the last release that installs on Python 3.10 ([#351](https://github.com/ryanduguid/australian-accounting/pull/351)).
- Pin `div7a-loan-review` 0.2.0; `payday-super-checker` 0.1.9, `ato-benchmark-compare` 0.1.11 and
  `australian-tax-calculators` 0.1.8 are unchanged.
- Through the new pin: `review_div7a_loan` with an astronomically large `remaining_term_years`, such as
  `1e999999999`, returns its result instead of hanging, because the engine no longer converts the term to an
  integer before the s 109E(6) calculation ([#344](https://github.com/ryanduguid/australian-accounting/pull/344)). Results report `engine_version` 0.2.0.
- The supplementary tool-selection scorer rejects duplicate JSON keys and non-finite numbers and compares
  decoded types exactly ([#352](https://github.com/ryanduguid/australian-accounting/pull/352)); the server and its tools are unchanged.
- Align the release, citation, compatibility and MCP Registry metadata to 0.3.0.
- Security: ships PyJWT 2.15.1, which fixes CVE-2026-101918 (GHSA-42vr-xj54-vc7v, medium), published on
  30 September 2026. 0.2.12 and earlier lock PyJWT 2.14.0.

# v0.2.12

- Pin `payday-super-checker` 0.1.9 and `div7a-loan-review` 0.1.6;
  `ato-benchmark-compare` 0.1.11 and `australian-tax-calculators` 0.1.8 are
  unchanged.
- Through the new pins: the payday checker's nine control arguments and the
  Division 7A engine's mode flags accept only literal booleans, and the
  Division 7A engine refuses conflicting modes. The tools already took strict
  booleans and set no Division 7A mode flag, so tool behaviour is unchanged;
  results report `engine_version` 0.1.9 (payday) and 0.1.6 (Division 7A). The
  payday checker's report invariant checks also hold under `python -O`.
- Raise the declared `mcp` requirement to `>=2.2.0,<3` (#331). The lock
  records `mcp` and `mcp-types` 2.2.0, `pyjwt` 2.14.0 and `urllib3` 2.8.0.
- Align the release, citation, compatibility and MCP Registry metadata to
  0.2.12.

# v0.2.11

- Pin `ato-benchmark-compare` 0.1.11 and `australian-tax-calculators` 0.1.8;
  `payday-super-checker` 0.1.8 and `div7a-loan-review` 0.1.5 are unchanged.
- Through the new pins: `get_ato_benchmarks` compares labour, rent and motor
  vehicle expenses, and cost of sales where it is not the key range, against
  the ranges the ATO publishes on each industry's page. Each ratio row's
  `benchmark_source` and the result's `industry_page_source` say where a range
  came from; these ranges never set the key ratio. `calculate_tax_worksheet`
  adds the `study_loan_repayment` worksheet for 2025-26 and 2026-27.
- Add `search_ato_rulings` and `read_ato_ruling`, which serve ATO ruling
  paragraphs from a configured local folder of rulings runs, bringing the
  server to 18 tools.
- Declare `benchmark_source` and `industry_page_source` in the
  `get_ato_benchmarks` output schema.
- Share one page builder between the legislation corpus and accounting
  library tools.
- Lock the build backend in the dev extra.
- Align the release, citation, compatibility and MCP Registry metadata to 0.2.11.

# v0.2.10

- Pin `payday-super-checker` 0.1.8, `ato-benchmark-compare` 0.1.10,
  `div7a-loan-review` 0.1.5 and `australian-tax-calculators` 0.1.7, the engine
  versions the workspace already tested. 0.2.9's own pins could not reproduce
  its quick proof: the published `div7a-loan-review` 0.1.4 carries an older
  benchmark table digest.
- `calc_payday_super_deadline` and `review_payday_super_contributions` report
  notional earnings, both SG-charge estimates and every uplift scenario to the
  cent, built the way the checker's report builds them, so they match the
  evidence pack's `report.csv`. They carried up to 27 decimal places, and
  rounding their sum could land a cent away from the report.
- Through the new pins: Division 7A results record each benchmark rate's RBA
  workbook cell and accept amounts written with trailing zeros past the cent;
  the `fbt` worksheet adds FBT return items 14A to 16; a report-style P&L with
  amounts in more than one column is refused when a column left of the fullest
  also holds amounts; and Payday Super evidence packs write caveat and note
  cells that start like a formula as text.
- The release workflow installs the built wheel outside the workspace, so its
  pins resolve from PyPI, and stops before publishing unless its demo matches
  `docs/quick-proof.txt`.
- Align the release, citation, compatibility and MCP Registry metadata to 0.2.10.

# v0.2.9

- Pin `australian-tax-calculators` 0.1.6 and add its `contribution_caps` and
  `pension_minimum` worksheets to `calculate_tax_worksheet`, for 2024-25 to
  2026-27: cap room, excess and bring-forward for one person's classified
  contributions, and the SISR Schedule 7 minimum for one account-based
  pension. Retain the `ato-benchmark-compare` 0.1.9, `payday-super-checker`
  0.1.7 and `div7a-loan-review` 0.1.4 pins.
- Count nine worksheets in the server instructions, scope resource, README,
  reference and registry descriptions, and narrow the unsupported SMSF and
  contribution-cap topics to what the worksheets leave out.
- Align the release, citation, compatibility and MCP Registry metadata to 0.2.9.

# v0.2.8

- `search_tax_legislation` and `search_tax_rates` return the best match first
  instead of in corpus file order: the query as a phrase in a heading, then every
  word in a heading, then the phrase in the text, then the words anywhere, with the
  principal tax Acts first within each tier. On a 946-title corpus "small business
  entity" previously led with Excise Act 1901 s 4 and "general deductions" did not
  reach ITAA 1997 s 8-1 in the first five results. Ranking reads the whole corpus
  on each search: about 0.5 seconds on that corpus, up to about 1.6 seconds for
  a word nearly every provision holds.
- `define_tax_term` puts the principal tax Acts first within exact and partial
  matches, and its 20-entry partial cap keeps the best-ranked entries rather than
  the first ones the scan meets.
- `read_tax_legislation_section` takes `start` and returns `start` and
  `next_start`, so a provision longer than 12000 characters can be read in parts.
  165 of 21,916 rows in that corpus were cut off with no way to read further,
  including ITAA 1997 s 995-1.
- A search with no match adds a notice suggesting statutory wording.
- A missing or invalid retrieval folder error tells the caller to ask the user
  rather than retry, and the scope resource reports whether each retrieval folder
  is configured, without its path.
- `aus-accounting-mcp --version` and `--help` print and exit instead of starting
  the server; an unknown argument is refused.
- Two evaluation cases cover ranking and reading a long provision in parts.
- Retain the `ato-benchmark-compare` 0.1.9, `payday-super-checker` 0.1.7,
  `div7a-loan-review` 0.1.4 and `australian-tax-calculators` 0.1.5 pins.
- Align the release, citation, compatibility and MCP Registry metadata to 0.2.8.

# v0.2.7

- Pin `australian-tax-calculators` 0.1.5 and add its `payg_withholding`
  worksheet to `calculate_tax_worksheet`: one regular weekly, fortnightly or
  monthly pay from 1 July 2026 on Schedule 1 scale 1, 2, 3, 5 or 6. Retain the
  `ato-benchmark-compare` 0.1.9, `payday-super-checker` 0.1.7 and
  `div7a-loan-review` 0.1.4 pins.
- Resolve each worksheet's engine function when it is called, so a kind the
  installed engine lacks is refused with a message and the other kinds keep
  working.
- Put the missing space back in eight input descriptions for
  `get_ato_benchmarks`, `calc_payday_super_deadline` and `review_div7a_loan`,
  which reached clients as run-together words such as "ratiodenominator".
- State in the scope resource, README and disclaimer that the author is not a
  registered tax or BAS agent, and limit support to software issues reproduced
  with fabricated data.
- Align the release, citation, compatibility and MCP Registry metadata to 0.2.7.

# v0.2.6

- Pin `ato-benchmark-compare` 0.1.9 and `australian-tax-calculators` 0.1.4.
  The benchmark release refuses a neutral CSV whose header names `account`,
  `amount` or `section` more than once instead of reading the first such column
  silently. The calculator release computes every worksheet in its own decimal
  context, so a caller's precision no longer changes a result. Retain the
  `payday-super-checker` 0.1.7 and `div7a-loan-review` 0.1.4 pins.
- Serve the richer worksheet catalogue from the pinned published engine. An
  installation on an older calculator keeps the legacy discovery entries.
- Align the release, citation, compatibility and MCP Registry metadata to 0.2.6.

# v0.2.5

- Pin `payday-super-checker` 0.1.7. A timely receipt date without either amount field remains `UNKNOWN`; a late receipt without an amount remains `LATE` without the unsupported shortfall reduction. Retain the other 3 published engine pins.
- Update the tool descriptions, server instructions and package description for the receipt-amount rule. Require its regression tests in standalone installations as well as workspace tests.
- Include the merged evidence semantics and explicit scope qualifications. The optional richer worksheet catalogue still depends on an unreleased worksheet engine; the pinned published engine retains its existing scope data.
- Include the synthetic group-review example, which indexes separate engine evidence without combining conclusions or transferring outputs between engines.
- Align the release, citation, compatibility and MCP Registry metadata to 0.2.5.

# v0.2.4

- A corpus line that spells a solidus as the JSON escape `\/` is parsed rather than prefiltered, so a `row_id` such as `5/10` that search returned can be read back without asking for neighbours.
- `define_tax_term` keeps the lettered conditions of a definition, such as `(a)` and `(b)` paragraphs, with the definition they complete; only a numbered subsection label such as `(2)` closes the dictionary.
- Pin `payday-super-checker` 0.1.6 and `div7a-loan-review` 0.1.4; `ato-benchmark-compare` 0.1.8 and `australian-tax-calculators` 0.1.3 are retained.
- Division 7A 0.1.4 names the primary source behind every benchmark rate, so a rate result and the checked demo transcript carry `primary_url`, `retrieved_on` and `snapshot_sha256` beside `verify_at`. No rate value changed.
- Checker 0.1.6 withholds a row's notional earnings, administrative uplift and SG-charge exposure when its period runs past the last GIC quarter on record, and says so in a caveat that names the day, the quarter and the file to update. The server has no equivalent of the checker's `--allow-stale-gic`, so a result it returns never carries an extrapolated rate. The checker also names the financial year a `rates.json` lookup is missing for, and caveats a fund-receipt row that carries no matched or remitted amount.
- Trim about 4.5 KB (10 percent) from the published tool descriptions and input schemas: the money rule ("AUD decimal string such as "1000.00": finite, at most 2 decimal places, at most 1000000000000.00; omit or null means not supplied, "0.00" only for an established zero") is now stated once in each tool's description instead of once per money field, and the worksheet scope rule once instead of six times. The server instructions carry one retrieval policy paragraph instead of two. No input, bound or behaviour changed.
- `refuse_div7a` publishes no inputs. The five legacy fields it accepted and ignored are removed, so a schema can no longer invite a borrower, lender or principal to reach a refusal. A call that still passes them is rejected as unknown arguments.
- Three adversarial evaluation cases: a corpus provision carrying an instruction, an undefined term, and a stored rate row that must be reported at its compilation date rather than as the current figure.

- Add `define_tax_term`, which finds an expression's statutory definitions in the configured corpus's dictionary, definitions and interpretation sections, exact matches first, each cited to the dictionary section that holds it. No match is reported as no match, never as an ordinary meaning.
- `read_tax_legislation_section` accepts `neighbours` (0 to 5) and returns the provisions either side of the cited one as `before` and `after`, each cited and truncated like a search match.

# v0.2.3

- Resolve the standalone lock against published `payday-super-checker` 0.1.5 and `ato-benchmark-compare` 0.1.8; `div7a-loan-review` 0.1.3 and `australian-tax-calculators` 0.1.3.
- Checker 0.1.5 carries the join's structural matching warnings into the canonical CSV and the evidence-pack report, so a degraded match can no longer read as an unqualified verdict from the files alone.
- Benchmark compare 0.1.8 withholds a bucket no account was mapped to (`not_supplied`) across its command-line output instead of presenting a computed nil; the MCP tool results already used the evidenced payload and are unchanged.
- Refresh the published package description, which still described the 0.2.2 evidence pack as unreleased.

# v0.2.2

- Publish the website URL and duguid.com.au icons in `server.json` and the initialize result so registries and clients show them.
- Reject library excerpts with a start line below one.
- Request an explicit as-at date and disclose all 6 engine-owned worksheet scopes.
- Resolve the standalone lock against published `ato-benchmark-compare` 0.1.7, `payday-super-checker` 0.1.4 and `div7a-loan-review` 0.1.2; retain australian-tax-calculators 0.1.3.
- Regenerate the static WebP proof from the candidate demo transcript; retain its source and SHA-256 in `docs/REFERENCE.md`.
