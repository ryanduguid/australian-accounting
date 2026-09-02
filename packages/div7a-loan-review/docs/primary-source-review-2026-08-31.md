# Primary-source implementation review, 31 August 2026

This review records the source trail behind the shipped rules. Every rule in
this engine was settled from the compiled Act on the Federal Register of
Legislation. Secondary commentary, ATO calculator behaviour and existing
Division 7A implementations were not used to settle any rule.

## Review position

The engine reviews two questions and refuses the rest: whether a loan meets
the s 109N(1) criteria on facts the operator asserts, and what the s 109E
minimum yearly repayment is for a later year of income on an amalgamated loan
the operator has already identified.

The engine and its monetary output remain **experimental**. A shortfall figure
is a review aid, not an assessment and not a s 109E(1) determination. This
review does not authorise a release, and it does not establish that any
particular loan is or is not a dividend.

The fail-closed controls this review put in place are:

- a year of income outside the reviewed rate table is `UNKNOWN`, never
  extrapolated from an adjacent year, and never fetched at run time;
- a rate override is refused unless it carries both `verified_until` and a
  non-empty citation, and refused again if it lists a year past its own
  `verified_until`;
- an unestablished s 109N(1) limb gives `UNKNOWN`, never `NOT_COMPLYING` and
  never `COMPLYING`;
- a minimum yearly repayment is refused, not estimated, where the gate did not
  return `COMPLYING`, where the year requested is the year the loan was made,
  or where the remaining term has run out;
- a missing payment figure is `UNKNOWN` rather than nil, because nil would
  report the whole repayment as exposure; and
- a loan made in or before the 1997-98 year of income is `SKIPPED`, including
  the year that straddles 4 December 1997.

## The compilation that was read

The controlling text is the
[Income Tax Assessment Act 1936](https://www.legislation.gov.au/C1936A00027/latest/text),
Federal Register identifier **C1936A00027**, compilation in force **1 July
2026**. The compiled Act is published in seven volumes; Part III Division 7A
(ss 109B to 109ZE) sits in volume 2.

Read in full from that text on 31 August 2026:

- **s 109D**: loans treated as dividends, the definition of *loan*, the year
  a loan is made, payments converted to loans, loans made before 4 December
  1997, and the lodgment day;
- **s 109E**: the amalgamated-loan dividend, the definition of an
  amalgamated loan, the minimum yearly repayment and its formula;
- **s 109N**: the complying-loan criteria, the benchmark interest rate, the
  maximum term and the refinancing reductions;
- **s 109P**: amalgamated loans not treated as dividends in the year they are
  made;
- **s 109R**: payments not taken into account; and
- **s 109ZD**: defined terms, to confirm where each defined expression is
  anchored.

Sections 109Q, 109Y, 109RB and 109RD were read for scope only, to establish
what the engine must *not* claim. They are not implemented.

### Sources deliberately not used

AustLII (`classic.austlii.edu.au`) returns `Access denied for AI crawlers` to
automated retrieval. That was respected rather than circumvented. The ATO
legal database returned HTTP 403 to the same. Neither was needed: the
Federal Register text is the authoritative source and settled every rule
below.

No existing Division 7A calculator was consulted, ported or used as a
reference implementation.

## s 109E(6): recovering the formula

The formula is not text in the compilation. It is a bitmap,
`OEBPS/document_2/image.017.png`, so a text extraction of the Act silently
drops it and leaves only the surrounding `where:` definitions. The image was
retrieved and read directly. It reads:

```
    Amount of the loan not repaid by         Current year's benchmark
    the end of the previous year of    x         interest rate
                 income
  ---------------------------------------------------------------------
                                                        Remaining term
                  /                       1                 \
          1  -   |  ---------------------------------------  |
                  \  1 + Current year's benchmark interest   /
                                     rate
```

The exponent applies to the whole bracketed fraction, not to its denominator
alone. This is the ordinary annuity form, `(P x r) / (1 - (1 + r)^-n)`.

**Runtime result:** implemented in `div7aloan/myr.py` exactly as written,
using `decimal.Decimal` only. Intermediate arithmetic runs at 60 significant
digits in a local decimal context so a result never depends on the ambient
context a caller has set, and is quantised once at the end. The
implementation was cross-checked against exact rational arithmetic
(`fractions.Fraction`) over the fixture set and over boundary inputs including
a one-year remaining term and a 25-year term; the two agree to the cent.

At `n = 1` the formula reduces to `P x (1 + r)`, which is asserted directly as
a closed-form check on the shape.

## s 109E(6): the remaining term

The subsection defines *remaining term* as the difference between:

> (a) the number of years in the longest term of any of the constituent loans
> that the amalgamated loan takes account of; and
> (b) the number of years between the end of the private company's year of
> income in which the loan was made and the end of the private company's year
> of income before the year of income for which the minimum yearly repayment
> is being worked out;

> rounded up to the next higher whole number if the difference is not already
> a whole number.

Working (a) requires the constituent loans, which means forming the
amalgamated loan under s 109E(3). v1 does not do that.

**Runtime result:** the operator supplies the difference. The engine applies
the statutory rounding rule with `ROUND_CEILING` and records the rounding in
the result's caveats when it bites. The `(a) - (b)` derivation for each
evaluation fixture is set out in the evaluation README so the one input the
engine takes on trust is itself checkable on paper.

## s 109N(2): the benchmark rate, and why the May figure

> The benchmark interest rate for the year of income is the Indicator Lending
> Rates-Bank variable housing loans interest rate last published by the
> Reserve Bank of Australia before the start of the year of income.

A year of income starts on 1 July. The RBA publishes statistical table F5 in
arrears, in the first week of the following month. The last figure published
**before** 1 July is therefore the **May** figure. The June figure is
published in early July, after the year of income has already begun, and
cannot be the rate the subsection points at.

For 2025-26 the difference is live: May 2025 was 8.37 per cent, June 2025 was
8.27 per cent, and 8.37 is the benchmark rate. 8.27 is a real benchmark rate,
for 2023-24, which makes the two easy to confuse in either direction.

**Runtime result:** `div7aloan/data/benchmark_rates.csv` is frozen, carries
its own `reviewed_until` and `reviewed_on` header, and records the RBA table,
series and month for every year. Every entry is a May figure, asserted as a
test across all eight years. Nothing in this repository reads the network, at
run time or in tests.

## s 109N(1)(b): which year's benchmark rate sets the floor

This is the one interpretive question in the build.

> (b) the rate of interest payable on the loan for years of income after the
> year in which the loan is made equals or exceeds the benchmark interest rate
> **for the year**;

Read against the chapeau, "A private company that makes a loan to an entity
in one of the private company's years of income ... if, **before the lodgment
day for the year of income** ...", "the year" is the year of income in which
the loan was made, and the whole of s 109N(1) is tested once, before that
year's lodgment day. It is not a test that recurs annually.

It is nonetheless common in practice to check, year by year, that the rate
being charged still meets the current benchmark, on the footing that an
agreement expressed to carry "the benchmark rate" must keep up with it. That
is a sensible commercial check. It is not the s 109N(1)(b) test.

**Reviewed position:** the Act's reading is followed. Confirmed by Ryan Duguid
on 31 August 2026.

**Runtime result:** `gate` without `--year` anchors each row to its own
`year_loan_made`, which is the statutory test. `gate --year <Y>` runs the
later-year comparison, and every result it produces carries a caveat naming
the divergence and the year the Act actually points at. Where the two readings
could differ, the engine follows the Act and labels the alternative.

Nothing turns on this for the repayment calculation. s 109E(6) uses the
*current* year's benchmark rate on its own terms, so a risen benchmark raises
the minimum yearly repayment whichever reading of s 109N(1)(b) is preferred.
That is the case the `rising_benchmark` fixture demonstrates.

## s 109N(3): the maximum term

The 25-year term requires both limbs of paragraph (a):

> (i) 100% of the value of the loan is secured by a mortgage over real
> property that has been registered in accordance with a law of a State or
> Territory; and
> (ii) when the loan is first made, the market value of that real property
> (less the amounts of any other liabilities secured over that property in
> priority to the loan) is at least 110% of the amount of the loan

Paragraph (b) gives 7 years for any other loan. A registered mortgage with
cover below 110 per cent does not shorten the term to something between the
two: the 25-year limb is simply unavailable and paragraph (b) applies.

"At least 110%" is inclusive, so cover of exactly 1.10 satisfies it.

**Runtime result:** the term limb is decided as above and both boundaries are
tested. Where the security facts are unestablished the limb is `UNKNOWN` only
if the loan's own term exceeds 7 years; at or below 7 years both limbs of
s 109N(3) permit the term, so the missing facts cannot change the answer and
forcing `UNKNOWN` there would be noise rather than caution.

Subsections 109N(3A) to (3D) reduce the maximum term where a loan refinances
an earlier one. Refinancing is **not modelled**. Any result that relies on the
25-year limb carries a caveat saying the term may be shorter if the loan
refinanced another.

## s 109E(1)(a) with s 109P: why the year of the loan is refused

s 109E(1)(a) reaches an amalgamated loan the company "made to the entity in an
**earlier** year of income". s 109P then provides that a private company is
not taken under s 109D to pay a dividend because of an amalgamated loan it
makes. Between them, the year in which the amalgamated loan is made carries no
minimum yearly repayment at all.

**Runtime result:** a repayment requested for the year the loan was made is
`REFUSED` with that reasoning, not answered with a number. A year earlier than
the year of the loan is refused for the same reason.

## s 109E(3)(b): why a non-complying loan is refused

A constituent loan of an amalgamated loan is one that

> (b) would cause the company to be taken under section 109D to pay a dividend
> to the entity at the end of the year, **apart from section 109N**

A loan that fails s 109N is therefore not a constituent loan of an amalgamated
loan, and s 109E produces no minimum yearly repayment for it. Where no other
Subdivision D provision applies, s 109D(1) instead treats it as a dividend in
the year it was made, for the amount unpaid before the lodgment day
(s 109D(1AA)).

That qualifier matters. Subdivision D holds other exclusions, ss 109J, 109K,
109L, 109M, 109NA and 109NB, none of which this engine models, so it does not
assert that a s 109D dividend necessarily arises. It asserts only that s 109E
has nothing to say.

**Runtime result:** `REFUSED`, with the Subdivision D qualifier stated in the
refusal message rather than implied.

## s 109R: which payments count

s 109R(2) removes a payment from the reckoning where a reasonable person would
conclude the entity intended to obtain a similar or larger loan, with
exceptions in subsections (3) to (7) for set-offs against dividends and
withholding payments, payments by a third party out of the borrower's
assessable income, and certain refinancings.

Every limb turns on intention and on what a reasonable person would conclude.
None of it is computable from a loan register.

**Runtime result:** the operator asserts the amount they are treating as
applied. Bank credits are not accepted as a substitute, and a missing figure
is `UNKNOWN`, not nil. Every repayment result (met, short or
refused) carries a caveat recording that s 109R was not applied.

## s 109D(6): the lodgment day

> the lodgment day for a private company's year of income is the earlier of:
> (a) the due date for lodgment of the private company's return of income for
> the year of income; and
> (b) the date of lodgment of the private company's return of income for the
> year of income.

Limb (a) needs a lodgment-program due date this engine does not hold, and
which varies by agent, entity and year.

**Runtime result:** the engine does not compute the lodgment day. The operator
asserts the boolean `terms_in_place_before_lodgment_day`, and the s 109N(1)
chapeau limb cites s 109D(6) so a reviewer can see exactly what was asserted.

## s 109D(5): loans made before 4 December 1997

Division 7A reaches a loan made before 4 December 1997 only where its terms
were varied on or after that day by extending the term or increasing the
amount, in which case the loan is treated as made on the new terms when the
variation occurred.

A year-of-income label cannot place a loan on one side of a date inside that
year.

**Runtime result:** a loan made in 1997-98 or earlier is `SKIPPED` with that
reason. 1997-98 is skipped precisely because it straddles the date.

## Why the shortfall is not the dividend

s 109E(2) makes the amount of the dividend the shortfall, **subject to
s 109Y**, which caps the total of the Division's dividends at the company's
distributable surplus. s 109E(1)(d) removes the dividend altogether where
s 109Q applies, on hardship. s 109RB gives the Commissioner a discretion to
disregard the Division's operation, and s 109RD a power to extend the period
for repayments.

None of those is modelled, and each can reduce or eliminate the figure.

**Runtime result:** the shortfall is reported as an *experimental
deemed-dividend exposure*, labelled a review aid on every surface that prints
it. The phrase "the ATO will assess" appears nowhere in this repository, and a
test asserts the README says so.

## Rounding: the Act prescribes none

Neither s 109E(5) nor s 109E(6) prescribes a rounding rule for the resulting
amount, and no regulation was found that provides one under either the
s 109E(5) regulation-making power or the s 109N(2) or s 109N(3) equivalents.

The engine therefore makes a choice, and states it rather than burying it:
money is quantised to cents with `ROUND_HALF_UP`. That is an engineering
decision, not a statutory rule, and it is deliberately not the ATO
calculator's rounding adopted silently. It appears in the README, in the
`rounding` field of every JSON result, and in the statutory trace of every
computed repayment.

A reviewer who reaches a figure one cent from this one has not necessarily
found an error in either.

## Rate provenance

The frozen table is cited to RBA statistical table **F5**, Indicator Lending
Rates, series **FILRHLBVS** (Housing loans; Banks; Variable; Standard;
Owner-occupier), and cross-checked against the published table at
<https://duguid.com.au/rates/div7a-benchmark-rate/>, last reviewed **28 August
2026**.

| Year of income | Rate | F5 FILRHLBVS figure |
| --- | --- | --- |
| 2026-27 | 8.77% | May 2026 |
| 2025-26 | 8.37% | May 2025 |
| 2024-25 | 8.77% | May 2024 |
| 2023-24 | 8.27% | May 2023 |
| 2022-23 | 4.77% | May 2022 |
| 2021-22 | 4.52% | May 2021 |
| 2020-21 | 4.52% | May 2020 |
| 2019-20 | 5.37% | May 2019 |

**This table has a shelf life by design.** `reviewed_until` is 2026-27. When
the May 2027 figure publishes in early June 2027, 2027-28 must be added and
the header re-reviewed, or every review of that year returns `UNKNOWN`. The
loader refuses a table carrying a rate past its own `reviewed_until`, so the
coverage claim cannot drift away from the rows it describes.

## Verification

- 269 tests pass across the rate table, the gate, the formula, the register,
  the CLI, the evaluation pack and the documentation.
- `GATES.md` records 17 completion gates, all met with recorded evidence,
  covering every "Done when" criterion and every required test in the build
  brief.
- The s 109E(6) implementation is checked against exact rational arithmetic.
- Three repayments are worked by hand in the evaluation README to twelve
  decimal places. Fixture 2 was re-performed from that README's printed text
  alone, importing nothing from the package, and every intermediate reproduces.
- No amount is emitted as a JSON number; the emitter refuses a float and a
  test proves the guard fires.

## What this review does not establish

It does not establish that any loan exists, that an entity is a shareholder or
associate, that an amalgamated loan was correctly formed, that a payment is a
genuine repayment, that a company has a distributable surplus, or that any
amount is or is not a dividend.

It does not cover s 109C payments, s 109CA provision of assets, s 109F
forgiven debts, the interposed-entity rules in ss 109T to 109X, s 109XA unpaid
present entitlements and the sub-trust material (PCG 2017/13, TD 2022/11),
s 109Q, s 109RB, s 109RD, s 109Y, substituted accounting periods, or public
companies.

It does not authorise a payment, a journal, a lodgment, a disclosure or a
compliance conclusion.

## Reviewer

Ryan Duguid, 31 August 2026. Written independently, in his own time, on his
own equipment. Provisional member of Chartered Accountants Australia and New
Zealand; that membership is not an endorsement of this software or this review
by CA ANZ.
