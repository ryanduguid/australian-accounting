# v0.2.0

## Added

- FBT return dates: `due_dates("fbt_return", ...)` for the FBT year ending 31 March 2027
  (`financial_year` `2026-27`, the 2027 return). A return is due on 21 May 2027, or
  generally on 25 June 2027 when a tax agent lodges it electronically for a client on
  the agent's FBT client list by 21 May, the new `on_fbt_client_list_by_21_may` fact.
  A `bas_agent` row is `UNKNOWN`, because the ATO's FBT pages name only tax agents. Each
  rule cites the ATO page read on 31 October 2026
  ([#523](https://github.com/ryanduguid/australian-accounting/pull/523)).
- `to_ics(result, *, uid_scope="")` writes a `due_dates` result as an iCalendar file
  (RFC 5545) using only the standard library, with one all-day event per `DETERMINED` row
  and the `UNKNOWN` rows returned apart. `uid_scope` keeps 2 clients on the same terms
  from sharing event UIDs. Whether a re-imported event replaces the earlier one depends
  on the calendar program
  ([#515](https://github.com/ryanduguid/australian-accounting/pull/515)).
- Tests run real calculations under an audit hook that fails on a network attempt
  ([#522](https://github.com/ryanduguid/australian-accounting/pull/522)).

## Changed

- Licence: the Ryan Duguid Public Attribution Licence 1.0
  (`LicenseRef-Duguid-Public-Attribution-1.0`) replaces FSL-1.1-ALv2. It is the
  FSL-1.1-ALv2 text unchanged plus a Public Attribution Condition on the FSL grant: a
  copy, port or derivative made available outside your organisation must credit Ryan
  Duguid publicly, with a link to the repository. Internal use is exempt. 0.1.0 keeps
  FSL-1.1-ALv2. Source files carry copyright and SPDX headers
  ([#569](https://github.com/ryanduguid/australian-accounting/pull/569),
  [#564](https://github.com/ryanduguid/australian-accounting/pull/564)).
- `source_checked` on every row moves from 28 October to 31 October 2026
  ([#523](https://github.com/ryanduguid/australian-accounting/pull/523)).
- The README states its disclaimer once, at its end
  ([#557](https://github.com/ryanduguid/australian-accounting/pull/557)).
- The release workflow uses Release Policy at ca5492b
  ([#580](https://github.com/ryanduguid/australian-accounting/pull/580)).

## Known issues

- No ASIC dates: annual review fees and statements are not covered.
- `aus-accounting-mcp` 0.9.0 pins `lodgment-calendar` 0.1.0, so its
  `get_lodgment_due_dates` tool does not offer FBT returns yet.

# v0.1.0

This is the first PyPI release of `lodgment-calendar`, published through
`release-lodgment-calendar.yml`, the monorepo's namespaced, attested release workflow.

- `due_dates` returns lodgement, payment and concession dates for monthly and quarterly
  BAS and IAS periods in 2026-27 and for 2025-26 income tax returns of individuals,
  trusts, partnerships, companies (including head companies of consolidated groups) and
  SMSFs, lodged by the taxpayer or through a registered tax or BAS agent.
- Applies the 2026–27 registered agent and BAS agent lodgment program dates and
  concessions, the 2-week online concession for quarterly statements and the December
  BAS concession, each with its ATO source, provision and review date.
- Moves dates to the next business day using the ATO's 2026–27 public holiday tables.
  Callers can add holidays the tables do not list and state the date their list is
  complete to, which settles dates after 30 June 2027.
- Returns `UNKNOWN`, naming the missing condition, for any date that depends on a fact
  not supplied, a date the ATO pages leave open or a day outside the holiday tables.
- Relicensed from Apache-2.0 to FSL-1.1-ALv2 (Functional Source License, with
  Apache-2.0 as the future licence). Released versions keep their licence.
