# Changes

## 1.0.0 (unreleased)

Standalone Rust binary, replacing Python package 0.2.0 and its `austaxcalc`
import API. The fourteen worksheet functions, `worksheet_catalogue()` and
`periods()` are JSON stdin subcommands, documented in README.md. No worksheet
method, rounding, period, scope statement, source, warning or refusal message is
intentionally changed.

### Added

- `--refusal-json`, after a command, adds a stable refusal code on stdout as
  `{"ok":false,"code":...,"message":...}`. The `error:` line, the message and
  exit 2 stay as they were, and stdout stays empty on a refusal without the flag.
  `REFUSAL_CODES` in `src/texts.rs` is the catalogue.
- `main-residence` works out the taxable share of an individual's capital gain
  or loss on one dwelling under the main residence exemption (ITAA 1997
  Subdiv 118-B), for CGT events in 2024-25 to 2026-27, from caller-established
  day periods. It covers partial income use (s 118-190), the absence choice
  with its 6-year limit (s 118-145), changing homes (s 118-140) and the home
  first used to produce income rule (s 118-192). No Python release has it. The
  worksheet catalogue lists it, so `tests/parity.rs` also drops that entry
  before comparing, and `tests/main_residence.rs` covers it. Catalogue examples
  can now hold JSON values.
- `borrowing-expenses` works out one income year's deduction for one loan's
  borrowing expenses under ITAA 1997 s 25-25, for 2024-25 to 2026-27: the
  s 25-25(6) $100 rule, the spread by days over the period of the loan (the
  shortest of the contract's period, the period to repayment and 5 years), the
  write-off in the year of repayment and the income-producing share. It refuses
  a leisure facility unless s 26-50(3) applies at all times in the year, and
  stops before 2027-28, when s 26-155 (Treasury Laws Amendment (Tax Reform No.
  1) Act 2026, Schedule 2) quarantines residential rental deductions. No Python
  release has it. `tests/parity.rs` drops its catalogue entry before comparing,
  and `tests/borrowing_expenses.rs` holds it to the ATO's Peter and Fiona and
  Max examples and Library ¶4-160 (Ben).
- `division-296` estimates one individual's Division 296 tax on large super
  balances for 2026-27 (ITAA 1997 Subdivision 296-B; Superannuation (Building a
  Stronger and Fairer Super System) Imposition Act 2026 s 5): 15% of the
  earnings share above the $3 million threshold and a further 10% of the share
  above $10 million, each share a percentage rounded to 2 decimal places, from
  the established balance at 30 June 2027 and total superannuation earnings,
  which may be negative. The 3 exceptions give nil tax with the reason in
  `liability`, and every result carries a warning that it is an estimate.
  Defined benefit and other s 296-70 interests, departing temporary residents
  and years from 2027-28 are outside it. No Python release has it.
  `tests/parity.rs` drops its catalogue entry before comparing, and
  `tests/division_296.rs` holds it to the Explanatory Memorandum's examples 1.1
  to 1.3 and the threshold and rounding boundaries. The new refusal code
  `input.signed_amount_invalid` covers its earnings input, and
  `input.field_unknown` refuses a field it does not take, such as an opening
  balance, which the other worksheets ignore.
- `termination-payment` works out the tax-free part of a genuine redundancy or
  early retirement scheme payment ($13,598 plus $6,801 for each completed year
  of service, ITAA 1997 s 83-170) and the Schedule 11 withholding from one life
  benefit employment termination payment made in 2026-27: the excluded part
  within the $270,000 ETP cap, then the non-excluded part within the smaller of
  the ETP cap left and the $180,000 whole-of-income cap less other taxable
  payments (s 82-10(4) and (7)), at 17% from preservation age at 30 June 2027,
  32% below it and 47% above the cap, each part rounded to the dollar. Death
  benefit ETPs, ETPs with a tax-free component, earlier ETPs and the tax
  assessed on the ETP are outside it. No Python release has it.
  `tests/parity.rs` drops its catalogue entry before comparing,
  `tests/termination_payment.rs` holds it to the ATO's Schedule 11 examples 1
  to 4 and each cap, limit, preservation age and rounding boundary, and
  the repository's root `tests/test_dated_rates.py` keeps its 4 dollar figures
  equal to the rate records. The new refusal code `input.years_of_service_invalid` covers its
  years of service, and `input.field_unknown` refuses a field it does not take.
- `depreciation` takes an optional `opening_adjustable_value` for an income
  year after the asset's start time (ITAA 1997 ss 40-70(1), 40-72 and 40-75).
  Diminishing value then applies its rate to that value and prime cost to cost,
  and either decline is limited to it; it may not exceed cost. Without it the
  result is the start year's, as before. The worksheet catalogue lists it under
  a new `optional_inputs` key, and the scope, sources and source passage name
  the later year, which `tests/parity.rs` applies to the frozen results.
  `tests/depreciation.rs` holds it to the ATO's diminishing value and prime
  cost examples. The new refusal code `input.opening_value_above_cost` covers
  an opening value above cost.
- `tests/verdict_cases.rs` turns the edge cases in a published reviewer verdict,
  under CC BY 4.0, the default in the bounty's terms, into fabricated cases, or
  states why one does not apply.
- `fbt-car-operating-cost` takes a required `effective_life_years` and works out
  deemed depreciation at 200% over it (FBTAA s 11(1AA)). It had applied 25% to
  every owned car, which is right for an 8-year life and understates a taxi's
  4-year life or a rental car's 5. A life under 2 is refused, except 0 for a leased
  car or an owned car with no depreciated value left.
- `fbt-car-statutory` and `fbt-car-operating-cost` value one car fringe benefit
  under FBTAA s 9 or s 10 for the FBT years ended 31 March 2026 and 2027; the
  operating cost result takes the lower statutory formula value under s 10(5).
  No Python release has them. The worksheet catalogue lists both, so
  `tests/parity.rs` compares the 10 frozen `worksheet-catalogue` results without
  those 2 entries, and `tests/fbt_car.rs` covers them.
- Both car worksheets end their amounts with `reportable_taxable_value`, the
  taxable value before any exemption (an exempt electric car's notional value),
  and add a third warning: the employee's reportable fringe benefits amount is
  the total of all their reportable benefits at the type 2 gross-up rate when it
  exceeds $2,000, a pooled or shared car is not reportable, and the worksheet
  determines neither. Every other amount and warning is unchanged.

### Documented divergences

- `depreciation`'s closing adjustable value is the opening value less the
  decline in value as shown in cents. Python 0.2.0 rounded the 2 separately, so
  a decline ending in exactly half a cent showed a cent too much in both: $10,001
  over 8 years by prime cost gave $1,250.13 and $8,750.88, now $8,750.87.
  Carried forward year by year, the old figures wrote off more than cost. No
  frozen result changes.
- `depreciation` divides once, last, so an exact half cent is not lost: $1,000.50
  over 14 years by diminishing value at 21% taxable use is exactly $30.015,
  now $30.02; Python 0.2.0 gave $30.01. No frozen result changes. The result
  is exact whenever the effective life's significant digits and the taxable
  use's decimal places total 22 or fewer, within the 40-digit working
  precision, and `depreciation` refuses anything longer with
  `depreciation.precision_exceeded`. Python 0.2.0 accepted it, and an
  effective life of 37 significant digits could land a cent from the exact
  value. No frozen call has such an input.
- `depreciation` refuses a field it does not take with `input.field_unknown`,
  as `division-296` does; Python 0.2.0 ignored it. A misspelt
  `opening_adjustable_value` would otherwise give the start year's result for a
  later year. No frozen call passes such a field.
- Python imports, keyword signatures, the `metadata` dictionaries and package
  version lookup are removed. `engine_version` in every result is the Cargo
  version, 1.0.0; Python 0.2.0 reported `0.2.0`. That field is the only
  difference in the 3,247 frozen results that carry it.
- Amounts and ratios arrive as decimal strings. A JSON number or `null` reaches
  the worksheet as the int, float or `None` it would have been in Python and is
  refused with the original message; text that is not a decimal is refused as
  `decimal.InvalidOperation`, as `Decimal(text)` raised it. Flags, counts, ages,
  days, quarters and scales are JSON values checked by the original type rules,
  so `2.0` is not a scale and `true` is not an integer. An integer of up to 4,300
  digits, the limit Python's `json.loads` applied, is an `int`, as in Python, and a list or object period is quoted in the
  refusal as Python's `repr()` wrote it.
- The JSON boundary refuses input over 8 MiB, a decimal string over 8,192 bytes
  and a missing argument (`<name> is required`; Python raised `TypeError`). JSON
  uses serde_json's nesting limit and refuses lone Unicode surrogate escapes.
- A JSON object that repeats a key is refused as `input.invalid_json`, with a
  message naming the key, instead of keeping the last value and dropping the
  others. Python 0.2.0 took keyword arguments, which cannot repeat.
- `periods` with a worksheet name it does not know is refused as
  `unknown worksheet '<name>'`, a missing `kind` as `kind is required` and a
  non-string `kind` as `kind must be a string`; Python raised `KeyError` or
  `TypeError`.
- A field a worksheet does not take is ignored; Python raised `TypeError` for an
  unexpected keyword argument.
- Results are compact JSON on stdout with native text-mode newlines. A refusal
  is one `error:` line on stderr with exit 2.
- The Python/PyPI release workflow, installed-distribution smoke script and the
  fast-check property bridge are removed. The properties run as seeded Rust
  tests over the same input domains. Until the MCP server links this crate,
  the uv workspace installs the 0.2.0 Python release from PyPI for the MCP
  server and `lodgeit-calculator-adapter`. A signed native publisher remains a separate
  release task.
- The ATO moved and retitled its study loan page to "Study and training loan
  minimum repayment income and rates", which now says "minimum repayment
  income" where it said "repayment threshold". Read again on 20 October 2026,
  it prints the same figures, so no calculation changes. Every
  `study-loan-repayment` result cites the new address, with `source_checked`
  `2026-10-20`; the `worksheet-catalogue` entry also carries the new title and
  the page's 13 October 2026 update. `tests/parity.rs` applies these changes to
  the 227 `study-loan-repayment` and 10 `worksheet-catalogue` frozen results.
- The ATO moved its "Lump sum payment in arrears tax offsets" page to "Lump
  sum payment in arrears" under income you must declare; the old address
  redirected there on 26 October 2026. The fourth `medicare-levy` source and
  the s 9A refusal message cite the new address, with no calculation change.
  `tests/parity.rs` applies this to the 162 `medicare-levy` results, 56
  `medicare-levy` refusals and 10 `worksheet-catalogue` frozen results.
- `contribution-caps` reports as `carry_forward_applied` only the unused
  concessional cap that covers contributions over the year's cap, as ITAA 1997
  s 291-20(4) applies it; Python 0.2.0 reported all the eligible unused cap,
  even when contributions were under the cap. With a $30,000 cap, $15,000
  unused and $40,000 contributed, it is now $10,000, not $15,000. The cap
  available, room and excess are unchanged. `tests/parity.rs` applies this to
  the 233 frozen `contribution-caps` results; 18 change.
- `gst`, `capital-gains` and `depreciation` also accept 2026-27; Python 0.2.0
  took 2025-26 only. The law they apply is unchanged for that year: the GST rate
  is 10%, the decline in value formulas are those the ATO's prime cost and
  diminishing value page gives. CGT reforms take effect from 1 July 2027
  ([ATO](https://softwaredevelopers.ato.gov.au/reforming-negative-gearing-dsp-special-purpose-working-group),
  read on 3 November 2026); `tests/tables.rs` keeps 2027-28 outside
  `capital-gains`. `quarterly-sg`
  stays at 2025-26. `tests/parity.rs` adds the period to the 19 `periods` and
  10 `worksheet-catalogue` frozen results and to the 34 refusals that list the
  supported periods.
- `payg-withholding` also accepts 2024-25 and 2025-26, using the ATO's Schedule
  1 coefficients for payments from 1 July 2024 to 30 June 2026 (published 17
  June 2024); Python 0.2.0 took 2026-27 only. The earnings conversion and
  rounding rules are the same in both schedules. `tests/tables.rs` reproduces
  all 720 amounts in that schedule's sample data for each of the 2 years.
  The scope statement now covers payments from 1 July 2024 and says the income
  year selects the coefficients; the sources add the 2024-26 schedule, and the
  source check date is 4 November 2026, when both schedules' coefficient pages
  were read. `tests/parity.rs` applies this to the 789 frozen
  `payg-withholding` results, 7 `periods` results, 10 `worksheet-catalogue`
  results and the 8 refusals that list the supported periods. It skips the 1
  frozen call that expected 2025-26 to be refused; `tests/tables.rs` now
  checks that 2023-24 and 2027-28 are refused. The catalogue example stays at
  2026-27.
- `fbt` also accepts the FBT year ended 31 March 2027; Python 0.2.0 took 2026
  only. The ATO's FBT rates and thresholds page (updated 20 May 2026, read 4
  November 2026) gives the 47% rate and the 2.0802 and 1.8868 gross-up rates
  for years ending 31 March 2023 to 2027, so no figure changes. The Federal
  Register's 2026 Acts, read on 4 November 2026 to the latest made on 26
  October, change no FBT rule for that year: the Tax Reform No. 1 Act's FBT
  amendments apply from 1 April 2027, and no Act amends the electric car
  exemption in s 8A. The result's period now names its own year. The scope
  statement names both years and their rates and says the 2027 return items
  follow the FBT return 2026 instructions until the ATO publishes the 2027
  return; the sources add the rates page, and the source check date is 4
  November 2026. `tests/parity.rs` applies this to the 103 frozen `fbt`
  results, 6 `periods` results, 10 `worksheet-catalogue` results and the 56
  refusals that list the supported periods. It skips the 1 frozen call that
  expected 2027 to be refused; `tests/tables.rs` checks 2027 against the ATO's
  MantCo example and that 2028 is refused. The catalogue example stays at 2026.

### Verification

A differential harness exported base 83288b0, ran its pytest suite and captured
the calls each test made, capped at 60 per test, then compared Python 3.15.0
with the Rust binary on those calls, 2,008 generated cases, 1,215 cases at every
statutory threshold in the base tables and its neighbours (uncapped), and 23 of
the final review's counterexamples. All 4,459 results
match byte for byte, including key order, after the `engine_version` adaptation.
They are frozen in `tests/data/python-api.json` and replayed by
`tests/parity.rs`. `tests/tables.rs` and `tests/properties.rs` run the table
checks, every value in the ATO's Schedule 1 sample data, the seeded properties
and the whole-range sweeps in full. `tests/BEHAVIOURS.md` maps each of the 109
original tests to its Rust coverage.

### Dependencies and reuse

Python 0.2.0 had no third-party runtime dependencies. Its Decimal dependency is
replaced by the BigUint-based Python Decimal compatibility code from The WIP
Tally's port (adapted from The Exchequer Tally and accounting-review-pipeline's
elizabeth-anne-alexander port, Apache-2.0), extended here with a per-thread
precision for the worksheets' 40-digit `localcontext`, ROUND_DOWN, remainder and
integral rounding. Rust num-bigint, num-integer and num-traits implement exact
coefficient arithmetic; serde_json, with insertion order preserved, provides the
JSON boundary. Scope statements and sources in `src/texts.rs` are generated from
the base revision. This component owns its copies and depends on no other
engine.
