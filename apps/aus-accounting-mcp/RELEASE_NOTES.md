# Unreleased

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
