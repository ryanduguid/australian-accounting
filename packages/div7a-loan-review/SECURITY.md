# Security policy

## Supported versions

Security fixes are applied to the latest version on the default branch.

## Reporting a vulnerability

Please use this repository's private vulnerability-reporting feature. Do not
open a public issue for a suspected security vulnerability. Include a clear
description, reproduction steps, impact, and any suggested mitigation.

A valid report will be acknowledged within seven days, and a fix and
disclosure timeline coordinated with the reporter.

## Local path trust boundary

This is a single-user CLI, not a sandbox or a service. Its `--input` and
`--rates-override` paths are selected by the invoking operating-system user
and intentionally may refer to any file that user can read. Do not run it with
elevated privileges, and do not pass path arguments taken from a less-trusted
user, web request, queue, or other tenant without first enforcing a
caller-specific safe root.

Both paths are opened read-only. The tool writes no files: it prints to
standard output, and redirecting that output is the caller's decision.

## No network

Nothing in this repository makes a network request, at runtime or in tests.
The benchmark interest rate table is frozen in
`div7aloan/data/benchmark_rates.csv` and reviewed by hand. A year outside its
coverage is reported as `UNKNOWN` rather than fetched. A reviewed override is
read from a local file the operator supplies.

This is deliberate. A rate scraped at runtime is a rate nobody reviewed, and
section 109N(2) turns on reading one specific monthly figure correctly.

## Untrusted input

Register CSVs are parsed with the standard library `csv` module and are never
evaluated. Every cell is converted through explicit `Decimal` and tri-state
parsers that refuse values they do not recognise, rather than falling back to
a default. A rate override is parsed as JSON and refused unless it carries
`verified_until` and a citation.

The tool does not resolve, follow, or fetch any URL found in an input file.
Values from an input file are treated as data, never as instructions.

## Output hygiene

Console and JSON output reproduce the `loan_id` and `borrower_reference` you
supply. If those carry client identifiers, so will the output. Treat a
generated review as a private workpaper: keep it in the same
access-controlled location as the register it came from, and do not commit
client outputs to this repository.
