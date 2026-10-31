# Primary-source review, 20 October 2026

Every rule in `lodgment-calendar` 0.1.0 comes from an ato.gov.au page read live on 20
October 2026. No secondary commentary, search snippet or earlier knowledge was used to
settle a rule. Where the pages conflict or leave a date open, the calendar returns
`UNKNOWN` and this note records why. The package is unreleased, and this review does not
authorise a release.

## Review position

The calendar models the ATO's 2026–27 lodgment program: activity statements for periods
in 2026-27, income tax returns for the 2025-26 income year (the 2026 return) and, from the
section on FBT returns below, the 2027 FBT return. It
returns a date as `DETERMINED` only when the caller has supplied every fact on that
date's decision path, the ATO pages give one date for those facts and the holiday table
covers every weekday the next-business-day roll passes. Everything else is `UNKNOWN`
with the missing fact or open question named.

The ATO says on its About the lodgment program page that its list of program due dates
'is a guide only'. Online services for agents, the practitioner lodgment service and the
statement itself remain the authority for a client's date, and every result says so.

## Holiday calendar

**Source:** [Lodgment and payment dates on weekends or public holidays](https://www.ato.gov.au/tax-and-super-professionals/for-tax-professionals/prepare-and-lodge/registered-agent-lodgment-program/lodgment-and-payment-dates-on-weekends-or-public-holidays),
last updated 1 July 2026: 'Approach to weekends and public holidays', Table 1 (2026) and
Table 2 (2027).

The page defines a business day as a day other than a Saturday, a Sunday or a public
holiday, and a public holiday as 'a day that is a public holiday for the whole of any
state or territory in Australia'. A due date on such a day moves to the next business day
for taxpayers in every state. The BAS due dates page and the SMSF annual return
instructions give the same next-business-day rule.

The calendar copies the 20 rows of Tables 1 and 2 as published, including the part-day
Christmas Eve and New Year's Eve holidays in some states, WA King's Birthday and
Melbourne Cup Day. It reads no other holiday source. The tables run from 3 August 2026 to
14 June 2027, so coverage is set to the program year, 1 July 2026 to 30 June 2027. A
weekday outside that span returns `UNKNOWN`, keeping the nominal date; a missing holiday
can only move a date later. A caller can add holidays the tables omit and state the date
the combined list is complete to; the calendar takes that statement as given and marks
each added holiday as caller-supplied. Without that date, coverage stays at 30 June 2027.

The `payday-super-checker` calendar was not reused. It applies the Superannuation
Guarantee (Administration) Act 1992 definition, which excludes holidays that are not
observed throughout a state, so it omits WA King's Birthday and Melbourne Cup Day. The
lodgment program table lists both. Engines in this repository also may not import one
another.

Tests reproduce every row of the table's 'First business day' column except one. The
row for Friday 25 September 2026 gives Monday 28 September as the first business day,
but the next row lists 28 September as King's Birthday (WA), so under the page's own
definition the answer would be 29 September. A roll that starts on 25 September 2026
returns `UNKNOWN` naming that conflict. No modelled due date falls on that day.

## Activity statements

| Rule | Date | Source and provision |
| --- | --- | --- |
| `AS-M-21` | Monthly BAS and IAS: lodge and pay by the 21st of the following month | [Due dates for lodging and paying your BAS](https://www.ato.gov.au/businesses-and-organisations/preparing-lodging-and-paying/business-activity-statements-bas/due-dates-for-lodging-and-paying-your-bas) (17 September 2026), Monthly reporting; IAS from [Activity statement generate dates](https://www.ato.gov.au/businesses-and-organisations/preparing-lodging-and-paying/business-activity-statements-bas/instalment-notices-for-gst-and-payg-instalments/activity-statement-generate-dates) (16 June 2026), 2026–27 monthly table |
| `AS-Q-28` | Quarterly: 28 October, 28 February, 28 April, 28 July | BAS page, table 'Due date for each quarter'; generate dates page, 2026–27 quarterly table |
| `AS-Q-ONLINE` | Self-lodger that receives and lodges the statement online, quarters 1, 3 and 4: 2 weeks after the original date (11 November 2026, 12 May 2027, 11 August 2027) | [Two week lodgment concession – terms and conditions](https://www.ato.gov.au/online-services/businesses-and-organisations-online-services/lodgments-in-online-services-for-business/two-week-lodgment-concession-terms-and-conditions) (18 August 2025), Eligibility for lodgment concession |
| `AS-Q-AGENT` | Registered agent, lodged electronically, each statement generating after the previous one was lodged electronically and no monthly PAYG obligation on the statement: 25 November 2026, 26 May 2027; quarter 2 none | [BAS agent lodgment program 2026–27](https://www.ato.gov.au/tax-and-super-professionals/for-tax-professionals/prepare-and-lodge/bas-agent-lodgment-program-2026-27) (1 July 2026), Table 2; the same dates in [Activity statements](https://www.ato.gov.au/tax-and-super-professionals/for-tax-professionals/prepare-and-lodge/registered-agent-lodgment-program/due-dates-by-obligation-type/activity-statements) (1 July 2026) |
| `AS-Q-AGENT-PAPER` | Agent lodging on paper: original date | Activity statements page, Quarterly activity statements |
| `AS-Q-AGENT-MONTHLY-PAYG` | Quarterly statement that also carries a monthly PAYG obligation: original date | Activity statements page, Eligible activity statements and Monthly activity statements |
| `AS-M-DEC-AGENT` | December monthly BAS (GST reported monthly), turnover up to $10 million, lodged electronically by an agent after the previous statement was lodged electronically, not deferred GST: 21 February 2027 | BAS agent program, Table 1; Activity statements page, Monthly activity statements |
| `AS-Q-HEAD` | Head company's quarterly PAYG instalment statement: the 21st after the quarter | Activity statements page, 'Quarterly PAYG instalment activity statement – head companies of consolidated groups' |

The program dates apply to both `tax_agent` and `bas_agent`, because both pages speak of
statements lodged electronically through Online services for agents or the practitioner
lodgment service by a registered agent. The input contract fixes 2 roles the pages
distinguish: a monthly `bas` reports GST monthly (other monthly activity statements are
`ias`), and for a consolidated head company a quarterly `ias` is its PAYG instalment
activity statement.

## Income tax returns

Agent dates come from the [registered agent lodgment program 2026–27](https://www.ato.gov.au/tax-and-super-professionals/for-tax-professionals/prepare-and-lodge/registered-agent-lodgment-program/about-the-lodgment-program)
pages, all last updated 1 July 2026. Self-lodger dates come from the pages named in the
table.

| Entity | Self-lodged | Through a tax agent |
| --- | --- | --- |
| Individual | 31 October ([Preparing your tax return](https://www.ato.gov.au/individuals-and-families/your-tax-return/before-you-prepare-your-tax-return/preparing-your-tax-return), 27 April 2026, 'Due dates for your tax return') | Overdue returns not all lodged by 31 October 2026: 31 October 2026 ([Taxpayers with overdue tax returns](https://www.ato.gov.au/tax-and-super-professionals/for-tax-professionals/prepare-and-lodge/registered-agent-lodgment-program/taxpayers-with-overdue-tax-returns)); otherwise latest liability $20,000 or more 31 March 2027; otherwise 15 May 2027, with the 5 June concession ([Individuals and trusts](https://www.ato.gov.au/tax-and-super-professionals/for-tax-professionals/prepare-and-lodge/registered-agent-lodgment-program/due-dates-for-tax-returns-by-client-type/individuals-and-trusts), Lodgment due dates table) |
| Trust | 31 October ([Income tax return](https://www.ato.gov.au/businesses-and-organisations/preparing-lodging-and-paying/reports-and-returns/income-tax-return), 27 May 2026, 'When to lodge') | As for individuals, and large or medium trusts 31 January 2027 (taxable in latest year lodged) or 28 February 2027 (non-taxable or new registrant) |
| Partnership | 31 October (Income tax return page) | Overdue returns 31 October 2026 ([October 2026](https://www.ato.gov.au/tax-and-super-professionals/for-tax-professionals/prepare-and-lodge/registered-agent-lodgment-program/due-dates-by-month/october-2026)); otherwise 15 May 2027 ([May 2027](https://www.ato.gov.au/tax-and-super-professionals/for-tax-professionals/prepare-and-lodge/registered-agent-lodgment-program/due-dates-by-month/may-2027)) |
| Company | 31 October with overdue returns (Income tax return page); otherwise `UNKNOWN`, see below | [Companies and super funds](https://www.ato.gov.au/tax-and-super-professionals/for-tax-professionals/prepare-and-lodge/registered-agent-lodgment-program/due-dates-for-tax-returns-by-client-type/companies-and-super-funds) 30 June balancers table and [Large and medium taxpayers](https://www.ato.gov.au/tax-and-super-professionals/for-tax-professionals/prepare-and-lodge/registered-agent-lodgment-program/due-dates-for-tax-returns-by-client-type/large-and-medium-taxpayers): 31 October 2026, 31 January 2027, 28 February 2027, 31 March 2027 or 15 May 2027 |
| Head company | Not published | [Consolidated groups](https://www.ato.gov.au/tax-and-super-professionals/for-tax-professionals/prepare-and-lodge/registered-agent-lodgment-program/due-dates-for-tax-returns-by-client-type/consolidated-groups), checked against [February 2027](https://www.ato.gov.au/tax-and-super-professionals/for-tax-professionals/prepare-and-lodge/registered-agent-lodgment-program/due-dates-by-month/february-2027) and the May 2027 page |
| SMSF | 31 October 2026 (new registrant or overdue), 31 January 2027 (taxable large or medium in 2025–26) or 28 February 2027 ([SMSF annual return 2026 instructions](https://www.ato.gov.au/print/section/e18658ad-de6e-49f3-b050-095fdf695f83), 30 May 2026, table 'Dates and types of lodgment for self-preparers') | [Super lodgment](https://www.ato.gov.au/tax-and-super-professionals/for-tax-professionals/prepare-and-lodge/registered-agent-lodgment-program/due-dates-by-obligation-type/super-lodgment) key dates table and Newly registered SMSFs |

For agent clients, returns outstanding at 30 June 2026 lead to 31 October 2026 only if
they are not all lodged by then: the overdue returns page says the normal program then
applies. Whether they were is a separate fact, `overdue_returns_lodged_by_31_october`.

A separate payment row appears only where payment is not due with the lodgement. It
gives 1 December 2026 for companies and super funds with overdue returns (from the
overdue returns page, which names the 2025–26 liability) or a taxable
large or medium 31 January date, and for self-preparing SMSFs lodging on 31 October or
31 January. Individuals and trusts get an `UNKNOWN` payment row, because the notice of
assessment sets their date. Other company and super fund dates cover lodgement and
payment together, as the tables state; partnerships pay no income tax.

## Questions left UNKNOWN

1. **Quarter 4, 2026–27 agent date.** Both agent pages give 25 August 2027 but mark it 'to
   be confirmed when the BAS agent lodgment program 2027–28 is developed'.
2. **First statement, or one after a paper lodgement.** The agent pages say such a client
   'may need to lodge by an earlier due date' without naming it. The original date is kept
   as the nominal date.
3. **Rolling the 5 June 2027 concession.** 5 June 2027 is a Saturday. The individuals and
   trusts page calls it 'a concessional lodgment due date', whilst the companies table and
   June 2027 page say it 'is not a lodgment due date'. Whether the weekend rule extends it
   is not stated.
4. **Head company with only one size condition.** The consolidated groups page gives 31
   January 2027 for a taxable head company with a large or medium member and a large or
   medium group, without saying whether both are needed. The calendar decides January
   only when both hold, and February for a non-taxable head company with a large or medium
   member, as the February 2027 page says. Other combinations are `UNKNOWN`.
5. **New registrant head company without a large or medium member.** The companies table
   lists 'Head companies of consolidated groups that are new registrants' at 28 February
   2027; the consolidated groups page sends 'all other head companies' to 31 March or 15
   May 2027.
6. **A head company's quarterly BAS.** The 21st applies to the head company's PAYG
   instalment statement; the facts do not show whether a given BAS is that statement.
7. **First consolidated return.** Due on the latest date for the group's notification of
   formation, which the calendar does not hold.
8. **Client added after 31 October 2026.** The October page makes 31 October the last
   date to add a client so the 2026 return is covered; no page states the date for a
   client outside the program.
9. **Self-lodged company dates.** The self-lodger page says 'Generally, the lodgment and
   payment date for small companies is 28 February' without defining a small company, so
   that date is kept as the nominal date of an `UNKNOWN` row. It publishes no date for a
   head company. For a company with overdue returns it gives no payment date, and the 1
   December 2026 date on the overdue returns page is addressed to registered agents.
10. **Individual and trust payments.** These follow the notice of assessment. For 15 May
    returns the staggered dates of 21 March, 21 April and 5 June 2027 depend on the
    lodgement date and are the latest dates if processing takes 2 weeks. A self-lodger's 21
    November payment date applies only if the assessment issues by 31 October.
11. **Dates after 30 June 2027.** Outside the holiday tables, as described above, unless
    the caller supplies a complete calendar. This affects quarter 4 and June 2027
    statements.
12. **Facts the caller cannot yet know.** Whether overdue returns will all be lodged by 31
    October 2026, whether a statement carries a monthly PAYG obligation and whether each
    statement generates after an electronic lodgement are inputs; until supplied, the
    affected dates are `UNKNOWN`.

The 5 June 2027 concession is listed for partnerships on the individuals and trusts
page. The June 2027 page does not name them but does not exclude them, so partnerships
get the concession row, which stays `UNKNOWN` for the weekend question in item 3.

## Readings for review

These rules are modelled as `DETERMINED` on a reading that a reviewer should confirm.

- **The $2 million test.** The companies table says 'total income in the 2024–25 year';
  the March 2027 and Super lodgment pages say 'latest year lodged'. These can differ when
  a return was not necessary, so the input is true or false only when both readings give
  the same answer, and null otherwise.
- **New registrants.** A new registrant has no earlier return, so the $20,000 liability
  and $2 million income tests are not asked of one, and a new SMSF takes 28 February 2027
  without a size test, as the Super lodgment page says.
- **Which year is taxable.** Large or medium companies, trusts and agent-lodged funds
  use the latest year lodged (the 2025 return). Head companies use the 2026 return, as
  the consolidated groups page says. Self-preparing SMSFs use 2025–26, the year the
  return covers, as the instructions say.
- **Overdue returns first.** For agent clients, 31 October 2026 for overdue returns comes
  before every other program date, except a head company's first consolidated return,
  which stays `UNKNOWN` because its own date depends on the notification of formation.
- **Self-preparing SMSF earliest date.** The instructions say the earliest applicable
  date applies, so a new registrant or overdue fund is due 31 October 2026 whatever its
  other facts.
- **Two-week concession.** Applied as 14 days after the original date, before any roll.
  The original dates for quarters 1, 3 and 4 are all weekdays.
- **Monthly payment.** The BAS page says the 21st is the date 'to lodge and pay'. The
  PAYG instalments page (last updated 9 March 2022) says deferred BAS payers pay monthly
  instalments on the 28th; that arrangement is listed in `not_decided` and not modelled.

Dates the ATO sets for one client (earlier-lodgement notices, prosecution letters,
deferrals and an advised first-year date for a new SMSF) are not inputs and are listed
in `not_decided`.

## Independent review

A Alcuin final review (GPT-6 with Pro thinking, verified in the model and effort
pickers) on 20 October 2026 read the engine, the MCP tool and the ATO page text above.
Every finding was checked against the pages before it was acted on:

- **Accepted and fixed:** overdue returns lodged by 31 October restoring the program
  date; the generation-order and monthly PAYG conditions on agent program dates; online
  receipt for the 2-week concession; separate member and group size tests for head
  companies; new-registrant and non-taxable short cuts that made settled dates
  `UNKNOWN`; the notice-of-assessment and self-lodger payment wording; the partnership
  concession; the payment source for overdue companies and funds; the 'small companies'
  and $2 million readings; the 25 September 2026 conflict; and keeping a partnership's
  'no payment' when its lodgement date is unknown.
- **Accepted as contract wording:** the GST-monthly meaning of a monthly BAS and the
  PAYG instalment meaning of a head company's quarterly IAS.
- **Not done:** working out a payment date from rules that agree whilst the lodgement
  date stays unknown (for example 1 December 2026 for a taxable large or medium company
  whose overdue status is unknown). The payment stays `unknown`, which is conservative.

A second Alcuin final review (GPT-6 with Pro thinking, verified the same way) on the
same day read the later changes: caller-supplied holidays, the invariant test and the
split of the engine into modules. It found no wrong due date or status:

- **Accepted and fixed:** the fact check again refuses an object that only claims to
  be a Boolean; holiday names must be printable, which also refuses DEL and Unicode
  control and format characters; the invariant test now checks that the roll passes
  over exactly the non-business days before the due date, and that the caller calendar
  settles dates after the ATO table; new tests cover a complete span that must reach
  the rolled date, a roll into the 25 September 2026 conflict and the tool's 50-holiday
  cap.
- **Not done:** tracing every decision path the invariant sample reaches. The whole
  suite keeps 100% branch coverage, which is not the same as covering every path.

## Cross-checks

- The ATO media release [The final countdown](https://www.ato.gov.au/media-centre/the-final-countdown-ato-reminds-taxpayers-to-lodge-now),
  published 19 October 2026, gives self-lodgers until Monday 2 November 2026. The
  calendar rolls 31 October 2026 to the same date.
- The generate dates page's legislative due dates match every monthly and quarterly
  nominal date for 2026-27.

## Reuse of au-fpa-pack

`pyfpa/au/gst.py` in au-fpa-pack (commit `a3c1df9`) holds `_month_due_date` (the 21st of
the following month), `_quarter_due_date` (a table of the 28th, with 28 February for
quarter 2) and `_next_business_day`, which moves weekends only. Line 44 records agent
lodgement program extensions as 'not modelled; flag only'. The monthly and quarterly
rules here agree with it. Its roll was not reused because it ignores public holidays,
and its due-date table is loaded from that package's rates data. This engine depends on
no other distribution.

## Pre-release source check, 28 October 2026

Every page in `SOURCES`, including the holiday page, was read live again on 28 October
2026 with one plain request each. All 21 returned HTTP 200, and each page's 'Last
updated' date matches the date recorded in its provision: the ATO records no update to
any of them since this review. The page text from 20 October was not kept, so the pages
were not compared word for word. Instead:

- Tables 1 and 2 of the holiday page list the same 20 dates as `HOLIDAYS`; the names
  differ only in the capital A of 'Additional' in the joined 28 December 2026 entry. The
  table's 'First business day' column agrees with the next-business-day roll for every
  date except 25 September 2026, the conflict this review already leaves `UNKNOWN`.
- The engine was run over every request shape with 403 fact sets. It returned 86
  distinct combinations of rule, cited page and nominal date. For 68 the date is
  written on the cited page; the other 18 follow from a rule that page states (the
  21st day of the following month, or 2 weeks after the original quarterly date). This
  ties each date to its cited page as the page reads now; it does not show that every
  condition around it is unchanged.

No rule, test or reading changed; `SOURCE_CHECKED` is now `2026-10-28`, the date of this
re-check. The quarter 4, 2026–27 agent date is still 'to be confirmed', and the holiday
table still ends on 14 June 2027.

## FBT returns, 31 October 2026

Added after 0.1.0. The Library holds FBT worked examples that mention the return's due date
without stating it, so it settled nothing; these rules come from the ATO pages below, read
live on 31 October 2026.

| Rule | Date | Source and provision |
| --- | --- | --- |
| `FBT-21-MAY` | Not lodged by a tax agent: lodge and pay by 21 May 2027 | [Lodging your FBT return and paying](https://www.ato.gov.au/businesses-and-organisations/hiring-and-paying-your-workers/fringe-benefits-tax/fbt-registration-lodgment-payment-and-reporting/lodging-your-fbt-return-and-paying) (10 September 2026), Due date to lodge your return and pay |
| `FBT-AGENT-ELECTRONIC` | Tax agent lodges electronically, client on its FBT client list by 21 May: 25 June 2027 | [FBT return](https://www.ato.gov.au/tax-and-super-professionals/for-tax-professionals/prepare-and-lodge/registered-agent-lodgment-program/due-dates-by-obligation-type/fbt-return) (1 July 2026); the same date in the June 2027 page |
| `FBT-AGENT-PAPER` | Tax agent lodges by paper: 21 May 2027 | FBT return page; the same date in the May 2027 page |
| `FBT-AGENT-NOT-LISTED` | Client not on the list by 21 May: 21 May 2027 | FBT return page, with the Lodging your FBT return page's 'you must be an FBT client of the tax agent by 21 May' |

Both dates are Fridays and neither is in the holiday table, so neither rolls. Both fall
inside `COVERS_TO`, so this adds no table end date: `.github/ci/check_rate_expiry.py`
already warns before that span ends.

Readings for review:

- **The return's year.** The agent pages say the '2027 FBT return'; the 2026 return
  instructions use the 'FBT year ending 31 March 2026' for the 2026 return. By that pattern
  the 2027 return is for the FBT year ending 31 March 2027, recorded as `2026-27`.
- **'Generally'.** The Lodging your FBT return page says the agent date is 'generally' 25
  June and does not say when it is not. The summary keeps the word, and a date the ATO sets
  for one client is in `not_decided`.
- **Electronic lodgement.** The agent page says 'through the Practitioner lodgment service';
  the other says 'electronically'. Both are read through the existing `lodges_electronically`
  fact.
- **Not on the list.** The pages say a client must be listed by 21 May for the June date.
  That the general 21 May date then applies is read from the 'unless' in the Lodging your FBT
  return page.
- **Entity type.** The pages give one date for every employer, so the entity type does not
  change the rule.

Left `UNKNOWN`: a BAS agent. The FBT pages name only tax agents and the
[BAS agent lodgment program](https://www.ato.gov.au/tax-and-super-professionals/for-tax-professionals/prepare-and-lodge/bas-agent-lodgment-program-2026-27)
page lists no FBT date, so whether a BAS agent can lodge the return is not settled here.

Every page in `SOURCES` was also read live on 31 October 2026 with one plain request each.
All returned HTTP 200, and each page's 'Last updated' date matches the date recorded in its
provision. The pages were not compared word for word, and the engine was not rerun over the
fact sets of the 28 October check. `SOURCE_CHECKED` is now `2026-10-31`.

The MCP tool does not offer `fbt_return`: it pins the published 0.1.0, which has no such
obligation, until a release and a pin bump.

## Before release

- Read every page above live again and update `SOURCE_CHECKED`.
- Confirm the quarter 4, 2026–27 agent date when the 2027–28 program is published, and
  extend the holiday table when the ATO publishes later holidays.
- Confirm or change each reading in the section above.
