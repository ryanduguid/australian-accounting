# Gates: div7a-loan-review v1

OWNS: div7aloan/**, tests/**, examples/**, evaluation/**, .github/**, *.md, *.toml, *.cff, LICENSE

Scope: the complete v1 engine reviewing ITAA 1936 s 109N loan terms and the s 109E minimum yearly repayment, against every "Done when" criterion and every required test in the build brief. Declared prerequisites: Python 3.10+ on PATH, and a POSIX shell (run with `--shell bash`); commands use `&&` chains and single-quoted `python -c`, which stock `cmd.exe` does not parse.

Negative control for G13, run 31 August 2026: G13 is an existence check, and its output is the constant `GATE_FILES_OK`, so its evidence fingerprint is the same whichever file list it ran. The list was therefore exercised directly: moving `docs/primary-source-review-2026-08-31.md` out of the tree turned G13 to `FAIL` (and G2 and G15 with it, since `tests/test_docs.py` reads that file), giving `UNMET: 3`; restoring it returned the ledger to `ALL MET`. Re-run that control after any edit to G13's file list, because a stale approval plus a constant marker cannot show the change on its own.

- [x] G0: this ledger states outcomes that can fail
  CHECK: node /c/Users/-/.claude/skills/unlazy/scripts/gate-lint.mjs GATES.md
  EXPECT: LINT OK
  EVIDENCE: exit=0; shell=C:\Program Files\Git\usr\bin\bash.exe; cwd=C:\div7a-loan-review; path=bc4508a9523d/36 entries; EXPECT=matched; output-sha256=1b7cccd68f673aaa14281c0be39203ce0431ecda493d90298a0312619aad548a; output-bytes=151

- [x] G1: the distribution installs from source into a clean environment and its console script runs
  CHECK: rm -rf /c/Users/-/AppData/Local/Temp/claude/C--/dd5a9b22-7565-4cd3-be7e-42e8eff802e4/scratchpad/gatevenv && python -m venv /c/Users/-/AppData/Local/Temp/claude/C--/dd5a9b22-7565-4cd3-be7e-42e8eff802e4/scratchpad/gatevenv && /c/Users/-/AppData/Local/Temp/claude/C--/dd5a9b22-7565-4cd3-be7e-42e8eff802e4/scratchpad/gatevenv/Scripts/python -m pip install -q . && /c/Users/-/AppData/Local/Temp/claude/C--/dd5a9b22-7565-4cd3-be7e-42e8eff802e4/scratchpad/gatevenv/Scripts/div7a-loan-review rate --year 2026-27 --format json > /dev/null && echo GATE_INSTALL_OK
  EXPECT: GATE_INSTALL_OK
  EVIDENCE: exit=0; shell=C:\Program Files\Git\usr\bin\bash.exe; cwd=C:\div7a-loan-review; path=bc4508a9523d/36 entries; EXPECT=matched; output-sha256=8bb5874bdc3d887c89cb73c7687b58d2034989f58da882d7c4bc3513e01cb0b1; output-bytes=251

- [x] G2: the whole test suite passes
  CHECK: python -m pytest -q && echo GATE_SUITE_OK
  EXPECT: GATE_SUITE_OK
  EVIDENCE: exit=0; shell=C:\Program Files\Git\usr\bin\bash.exe; cwd=C:\div7a-loan-review; path=bc4508a9523d/36 entries; EXPECT=matched; output-sha256=4aff5c1642ff273bafd320bc80b4510829da74e926b9cdc1591684e901a86a57; output-bytes=451

- [x] G3: every frozen benchmark year matches the published table and reads a May figure
  CHECK: python -m pytest "tests/test_rates.py::test_every_frozen_year_matches_the_published_table" "tests/test_rates.py::test_every_year_reads_the_may_figure" -q && echo GATE_RATES_OK
  EXPECT: GATE_RATES_OK
  EVIDENCE: exit=0; shell=C:\Program Files\Git\usr\bin\bash.exe; cwd=C:\div7a-loan-review; path=bc4508a9523d/36 entries; EXPECT=matched; output-sha256=6accaa8b812ff55a4ff943731d715c9a6f0a3896eb4c373e9d499cf2836e4b39; output-bytes=115

- [x] G4: 2025-26 takes the May 2025 figure 8.37 and not the June 2025 figure 8.27
  CHECK: python -m pytest "tests/test_rates.py::test_2025_26_uses_the_may_figure_not_the_june_one" "tests/test_rates.py::test_2023_24_is_the_year_that_carries_0_0827" "tests/test_cli.py::test_the_rate_command_reports_the_may_figure_and_its_provenance" -q && echo GATE_TRAP_OK
  EXPECT: GATE_TRAP_OK
  EVIDENCE: exit=0; shell=C:\Program Files\Git\usr\bin\bash.exe; cwd=C:\div7a-loan-review; path=bc4508a9523d/36 entries; EXPECT=matched; output-sha256=596610e4ed62585b0bef88094ecaf1d6f78dbb9213151b06cf3b8d6fffd82da6; output-bytes=113

- [x] G5: a year outside the frozen table is UNKNOWN unless a reviewed override supplies verified_until and a citation
  CHECK: python -m pytest "tests/test_rates.py::test_year_before_coverage_is_unknown_with_a_reason" "tests/test_rates.py::test_year_after_coverage_is_unknown_with_a_reason" "tests/test_rates.py::test_override_extends_coverage" "tests/test_rates.py::test_override_without_citation_is_refused" "tests/test_rates.py::test_override_without_verified_until_is_refused" "tests/test_rates.py::test_override_cannot_reach_past_its_own_verified_until" -q && echo GATE_FAILCLOSED_OK
  EXPECT: GATE_FAILCLOSED_OK
  EVIDENCE: exit=0; shell=C:\Program Files\Git\usr\bin\bash.exe; cwd=C:\div7a-loan-review; path=bc4508a9523d/36 entries; EXPECT=matched; output-sha256=1c1b314eaa17c1b86c8bd805a378eae991ea8b95c887a89684a31401fe2a8aaf; output-bytes=119

- [x] G6: the s 109N gate decides every boundary the brief names and never coerces unknown to complying
  CHECK: python -m pytest "tests/test_gate.py::test_seven_year_unsecured_at_exactly_the_benchmark_is_complying" "tests/test_gate.py::test_seven_year_unsecured_one_basis_point_under_is_not_complying" "tests/test_gate.py::test_twenty_five_year_loan_without_a_registered_mortgage_is_not_complying" "tests/test_gate.py::test_twenty_five_year_loan_with_mortgage_but_109_per_cent_cover_is_not_complying" "tests/test_gate.py::test_twenty_five_year_loan_with_mortgage_and_110_per_cent_cover_is_complying" "tests/test_gate.py::test_written_agreement_unknown_does_not_become_false" "tests/test_gate.py::test_an_unestablished_fact_gives_unknown_never_complying" -q && echo GATE_109N_OK
  EXPECT: GATE_109N_OK
  EVIDENCE: exit=0; shell=C:\Program Files\Git\usr\bin\bash.exe; cwd=C:\div7a-loan-review; path=bc4508a9523d/36 entries; EXPECT=matched; output-sha256=ea27af4eb556a71c225ce51b809e536365df2685c2489dc29d1e99a283bcffe7; output-bytes=114

- [x] G7: the s 109E(6) formula agrees with independent exact rational arithmetic
  CHECK: python -m pytest "tests/test_myr.py::test_formula_agrees_with_exact_rational_arithmetic" "tests/test_myr.py::test_formula_regression" "tests/test_myr.py::test_a_single_remaining_year_is_principal_plus_one_year_of_interest" -q && echo GATE_FORMULA_OK
  EXPECT: GATE_FORMULA_OK
  EVIDENCE: exit=0; shell=C:\Program Files\Git\usr\bin\bash.exe; cwd=C:\div7a-loan-review; path=bc4508a9523d/36 entries; EXPECT=matched; output-sha256=9a67b84c2f455c9b26751e7b9839e7b4be57047f8165c1d1903452f59e479307; output-bytes=117

- [x] G8: the repayment verdicts are exact to the cent and the exposure flag equals the shortfall
  CHECK: python -m pytest "tests/test_myr.py::test_repayment_met_exactly_to_the_cent" "tests/test_myr.py::test_repayment_short_by_one_dollar" "tests/test_myr.py::test_repayment_short_by_one_cent_is_still_short" "tests/test_myr.py::test_an_overpayment_does_not_produce_a_negative_shortfall" -q && echo GATE_VERDICT_OK
  EXPECT: GATE_VERDICT_OK
  EVIDENCE: exit=0; shell=C:\Program Files\Git\usr\bin\bash.exe; cwd=C:\div7a-loan-review; path=bc4508a9523d/36 entries; EXPECT=matched; output-sha256=866d6bab91ca665eb4302a81fc6406b88fcefbed320580624b2e25bcc7cef0b9; output-bytes=116

- [x] G9: every refusal the brief requires is refused, and no refusal emits a repayment figure
  CHECK: python -m pytest "tests/test_myr.py::test_a_nil_remaining_term_is_refused_without_dividing_by_zero" "tests/test_myr.py::test_the_year_of_the_loan_is_refused" "tests/test_myr.py::test_a_non_complying_gate_refuses_the_repayment_figure" "tests/test_myr.py::test_an_unknown_gate_refuses_the_repayment_figure" "tests/test_myr.py::test_a_missing_unpaid_balance_is_unknown" "tests/test_myr.py::test_missing_payments_are_unknown_not_nil" "tests/test_register.py::test_a_loan_made_before_4_december_1997_is_skipped" -q && echo GATE_REFUSE_OK
  EXPECT: GATE_REFUSE_OK
  EVIDENCE: exit=0; shell=C:\Program Files\Git\usr\bin\bash.exe; cwd=C:\div7a-loan-review; path=bc4508a9523d/36 entries; EXPECT=matched; output-sha256=55574e3db66f7c4a8e354a405eebed01d1bb1abec8c031a5b280d01b301831b6; output-bytes=115

- [x] G10: no amount is emitted as a JSON number anywhere in the CLI output
  CHECK: python -m pytest "tests/test_cli.py::test_json_output_carries_no_floats" "tests/test_cli.py::test_no_amount_is_written_as_a_json_number" "tests/test_cli.py::test_amounts_are_strings_with_two_decimal_places" "tests/test_cli.py::test_the_emitter_refuses_a_float_it_is_handed" -q && echo GATE_NOFLOAT_OK
  EXPECT: GATE_NOFLOAT_OK
  EVIDENCE: exit=0; shell=C:\Program Files\Git\usr\bin\bash.exe; cwd=C:\div7a-loan-review; path=bc4508a9523d/36 entries; EXPECT=matched; output-sha256=a2d2d2463821fffa65aac8c0672e321e0185c782254c2da0972c3ebe63aee6d3; output-bytes=117

- [x] G11: both sample registers behave as the README describes, including their exit codes
  CHECK: python -m pytest "tests/test_register.py::test_the_clean_sample_runs_to_a_verdict_with_no_flags" "tests/test_register.py::test_the_mixed_sample_carries_unknowns_and_a_non_complying_loan" "tests/test_register.py::test_the_mixed_sample_leads_with_its_exposure" -q && python -m div7aloan.cli review --input examples/sample_loans_myr_met.csv --year 2026-27 > /dev/null && { python -m div7aloan.cli review --input examples/sample_loans_mixed.csv --year 2026-27 > /dev/null; [ $? -eq 2 ]; } && echo GATE_SAMPLES_OK
  EXPECT: GATE_SAMPLES_OK
  EVIDENCE: exit=0; shell=C:\Program Files\Git\usr\bin\bash.exe; cwd=C:\div7a-loan-review; path=bc4508a9523d/36 entries; EXPECT=matched; output-sha256=4e9b8e019b8857c6640269acf20402c0a7e048cd49cb64422d64d84d1d384b7d; output-bytes=116

- [x] G12: every evaluation fixture reproduces its pinned expectation and the hand-worked figures appear in the evaluation README
  CHECK: python -m pytest tests/test_evaluation_pack.py -q && echo GATE_EVALPACK_OK
  EXPECT: GATE_EVALPACK_OK
  EVIDENCE: exit=0; shell=C:\Program Files\Git\usr\bin\bash.exe; cwd=C:\div7a-loan-review; path=bc4508a9523d/36 entries; EXPECT=matched; output-sha256=39a8cd5d13922087cee5082419eefad98c8bc69facd562176597891ac93bd31b; output-bytes=129

- [x] G13: the house files, the frozen rate table and both samples are present on disk
  CHECK: python -c 'import pathlib,sys; want=["README.md","DISCLAIMER.md","SECURITY.md","CITATION.cff","LICENSE","pyproject.toml",".github/workflows/ci.yml","div7aloan/data/benchmark_rates.csv","div7aloan/py.typed","evaluation/div7a_myr/README.md","evaluation/div7a_myr/expected_results.json","docs/primary-source-review-2026-08-31.md","examples/sample_loans_myr_met.csv","examples/sample_loans_mixed.csv"]; missing=[f for f in want if not pathlib.Path(f).is_file()]; sys.exit("missing: "+", ".join(missing)) if missing else print("GATE_FILES_OK")'
  EXPECT: GATE_FILES_OK
  EVIDENCE: exit=0; shell=C:\Program Files\Git\usr\bin\bash.exe; cwd=C:\div7a-loan-review; path=bc4508a9523d/36 entries; EXPECT=matched; output-sha256=c3cb5e0c624df1d4bcd49dec1f55b76e70ab87003da4c65ab5e80ea07b83f48a; output-bytes=15

- [x] G14: the package imports nothing outside the standard library and makes no network call
  CHECK: python -c 'import ast,pathlib,sys; ext=set(); [ext.update({a.name.split(".")[0] for a in n.names}) if isinstance(n,ast.Import) else (ext.add(n.module.split(".")[0]) if isinstance(n,ast.ImportFrom) and n.level==0 and n.module else None) for p in pathlib.Path("div7aloan").rglob("*.py") for n in ast.walk(ast.parse(p.read_text(encoding="utf-8")))]; third=sorted(ext-set(sys.stdlib_module_names)); sys.exit("third-party imports: "+", ".join(third)) if third else print("no third-party imports")' && python -c 'import pathlib,re,sys; bad=[str(p) for p in pathlib.Path("div7aloan").rglob("*.py") if re.search(r"requests\.|urllib|urlopen|socket\.|http://", p.read_text(encoding="utf-8"))]; sys.exit("network calls in: "+", ".join(bad)) if bad else print("GATE_SEALED_OK")'
  EXPECT: GATE_SEALED_OK
  EVIDENCE: exit=0; shell=C:\Program Files\Git\usr\bin\bash.exe; cwd=C:\div7a-loan-review; path=bc4508a9523d/36 entries; EXPECT=matched; output-sha256=7e4cdbf3d328c711b79a873f6978091ecff357f98e93d062d2f86ea12a3f20ec; output-bytes=40

- [x] G15: the documentation and the primary-source review state the refusals, the rounding mode, the source trail and the authorship the brief requires
  CHECK: python -m pytest tests/test_docs.py -q && echo GATE_DOCS_OK
  EXPECT: GATE_DOCS_OK
  EVIDENCE: exit=0; shell=C:\Program Files\Git\usr\bin\bash.exe; cwd=C:\div7a-loan-review; path=bc4508a9523d/36 entries; EXPECT=matched; output-sha256=d45566c31fa2fe0ea6a32ed90aaf1ae1b079bb06b4a1ba6e9580189001af9235; output-bytes=195

- [x] G16: the evaluation README carries enough working for a human reviewer to re-perform each hand-worked repayment on paper, without running the code
  EVIDENCE: Fixture 2 re-performed on 2026-08-31 from the README's printed text alone, importing nothing from div7aloan: transcribing only P=250000.00, r=0.0827 and n=7, all six printed intermediates reproduce exactly (1+r=1.082700000000; (1+r)^7=1.744042072513; (1/(1+r))^7=0.573380663093; denominator=0.426619336907; numerator=20675.000000; quotient=48462.407142), giving 48462.41 and a shortfall of 1000.00 as printed. The README also derives each fixture's remaining term from s 109E(6)(a) minus (b) in its own table, so the one input the engine takes on trust is itself checkable on paper. G12 separately proves the printed figures match what the engine computes. RESIDUAL OWNER JUDGEMENT: whether an accountant reading s 109N and s 109E finds the write-up followable is Ryan's call, not a machine's.
