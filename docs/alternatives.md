# Alternatives compared

The dated comparisons behind the [README's Alternatives](../README.md#alternatives) list. Each
was read on 13 November 2026 unless it gives another date.

- [william-laverty/ato-mcp](https://github.com/william-laverty/ato-mcp): a hosted MCP connector,
  for Claude.ai, ChatGPT and other clients after sign-in, over more than 34,500 ATO documents
  (guidance, legislation and public rulings), with point-in-time legislation queries. Its latest release
  is v2.3.0 of 22 October 2026. Its [pricing](https://ato-mcp.com.au/pricing) gives 25 lookups a
  month free and unlimited lookups for $300 a year, and its
  [privacy page](https://ato-mcp.com.au/privacy), last updated 11 November 2026, names Voyage AI in
  the United States as turning search queries into vectors and ranking the matching passages. The
  repository is AGPL-3.0, and its README calls the hosted platform and corpus proprietary.
- [Nagaris](https://nagaris.com/llms.txt): a hosted practice platform for Australian accounting
  firms, with proposals, AML/CTF checks, e-signing, job tracking and an AI assistant on one client
  file. It publishes browser calculators, including [Division 7A](https://nagaris.com/division-7a-calculator),
  which with repayments entered also shows interest at the benchmark rate and the closing balance
  (this repository's Division 7A review computes neither), and
  [Payday Super due dates](https://nagaris.com/payday-super-due-date-calculator), whose page also
  states 11 August 2026 for a 30 July 2026 payday. Its
  [AI Accounting Index](https://nagaris.com/ai-accounting-index), which tested third-party AI models
  with each question answered once, reports that none of the 14 setups that answered that question
  got the date right, and says Ask Nagaris adds lookup tables for business days.
- [LodgeiT Labs](https://lodgeit.org/calculators.html): a public HTTP gateway for deterministic
  calculators with no key or signup, which its page describes as 'Not stable yet'. The gateway also
  speaks MCP, and its page says to add it to Claude as a custom connector with no sign-in. Its
  [listing](https://fbt-calculator-api-qkp3j5bjnq-ts.a.run.app/v1/calculators) showed 23 calculators,
  with Division 7A periods `fy2025` and `fy2026` and FBT period `fy2026`. Its page also lists a
  Coal LSL calculator that external contributor Ryan Duguid built and hosts, and this repository has
  an optional adapter for the gateway. A
  [differential](../apps/lodgeit-calculator-adapter/docs/differential-2026-11-06.md) of 6 November 2026
  ran 12 fabricated Division 7A and FBT car formula cases against the gateway and records each
  outcome, with differences traced to statute. Its `hp:schedule` calculator, as its OpenAPI document
  read on 15 November 2026 describes it, amortises a hire purchase agreement at a stated nominal
  rate, reports any residual and totals interest by financial year; `hire-purchase-schedule` instead
  works out the implicit rate from the payments, as ITAA 1997 s 240-60 does, and closes at nil, but
  leaves the caller to split a month that spans 30 June. On 16 November 2026 the listing held 19 FBT
  calculators, each for `fy2026` only. This repository's engine values car, loan, meal entertainment
  (50/50 split and 12 week register), car parking (commercial parking station, market value, average
  cost, spaces and 12 week register), debt waiver, expense payment (external, in-house property and
  in-house residual), property, residual, housing, living-away-from-home allowance, board and
  tax-exempt body entertainment benefits for the FBT years ending 31 March 2026 and 2027, so each of
  the listing's 19 FBT calculators has a counterpart here. Its spaces calculator takes one space at
  a time and leaves the s 39FB reduction to the caller; `fbt-car-parking` takes every space an
  election covers and applies s 39FB.
- [aftertaxau/au-compliance-kit](https://github.com/aftertaxau/au-compliance-kit), read on
  14 November 2026: an MIT-licensed JavaScript library with no dependencies, v0.4.0 of
  13 November 2026, with a local MCP server and a hosted one at `https://aftertaxau.com/mcp` that
  needs no key. Both it and this repository cover Schedule 1 withholding with scale 4, quarterly
  pays, tax offsets and the Medicare levy adjustment, and both test all 432 rows of each year's ATO
  sample data. This repository takes quarterly pays under Schedules 1 and 8 only: its
  Schedule 5, 9 and 15 commands refuse them. The library also takes quarterly pays under
  Schedule 9, and covers overtime, which this repository does not. Both add the Schedule 8
  study loan component to Schedule 9 pays, as rechecked in v0.4.0's source on 15 November 2026;
  for 2025-26 the library applies the rates from 24 September 2025 when no pay date is given,
  where this repository requires the date.
  Both compose a year's tax and levies, work taxable income out from salary and deductions with
  the $1,000 standard deduction for work-related expenses from 2026-27, and compare the result
  with tax withheld, as rechecked in the library's source on 16 November 2026. Both work that
  tax withheld out from one employer's regular pays, with Schedule 5 Method A for a bonus, as
  rechecked at the library's commit e94cef24 on 16 November 2026: the library from an annual salary
  spread over 52 weekly, 26 fortnightly or 12 monthly pays, and this repository from one pay's
  earnings and the number of pays in the year, so it also takes 53 weekly or 27 fortnightly pays,
  quarterly pays under Schedules 1 and 8, and, under Schedule 1 without a study loan or a bonus,
  the tax offsets and Medicare levy adjustment a payee declares. The library
  also applies Method A to a senior's or pensioner's bonus, which this repository refuses. The library's
  `annual_tax_estimate` approximates repayment income and income for surcharge purposes by taxable
  income. Both compose a resident's 2027-28 tax with the working Australians tax offset, as
  rechecked at the library's commit e94cef24 on 16 November 2026: the lesser of $250 and the tax on
  net labour income, when that exceeds $18,200, which the library's notes call approximate below $250, pending
  a legislative instrument, though ITAA 1997 s 61-160 as Act No. 49 of 2026 enacts it leaves
  nothing to one. This repository applies the offset before the seniors and low income tax offsets,
  as s 63-10(1) orders, where the library applies it after them; the total is the same, but the
  library can show a smaller offset. This repository also works out 2027-28's seniors and pensioners
  tax offset thresholds from the regulations' indexation, where the library applies 2026-27's, and
  compares the total with a given `tax_withheld`, where the library gives no refund for 2027-28.
  The library estimates 2027-28 with a study loan or without hospital cover on figures the ATO had
  not published, which this repository refuses: it takes 2027-28 with `tax_withheld`, hospital cover all
  year and no study loan. This repository's `personal-tax-estimate` takes those incomes as established,
  names the worksheet behind each line
  and keeps its sources and refusals, shows Division 293 tax, and refuses part-year residents and
  working holiday makers; both compose a full-year foreign resident's tax without the Medicare
  levy. Both apply full and half Medicare levy exemptions and the Medicare levy
  surcharge; the library takes a whole-year exemption and cover status only and applies the
  surcharge rate to all income for surcharge purposes, where this repository apportions both by
  days and applies the rate to taxable income and reportable fringe benefits only, as the ATO's
  examples do. The library also covers a sole parent's surcharge, which this repository refuses.
  Both work back from take-home pay to gross: v0.4.0's `salary_for_take_home`, read on
  15 November 2026, gives the lowest whole-dollar annual salary whose take-home reaches a target,
  with any study loan component, and `payg-gross-up` gives the gross of one pay, with or without
  the component, whose withholding leaves the net exactly, listing every gross where more than one
  does.
  This repository covers 2024-25
  withholding, Schedule 9's Medicare levy exemption scales, Schedule 5 Method B, Schedule 15 for
  working holiday makers and Schedule 11 for termination payments, which the library does not. For
  2026-27 both apply the 2025-26 Medicare levy low-income thresholds: the library notes that
  2026-27's are not yet legislated, and this repository applies them as Act No. 58 of 2026 enacts
  them for 2025-26 and later years.
