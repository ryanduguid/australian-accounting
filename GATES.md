# Gates: Division 7A MCP integration

OWNS: GATES.md, AGENTS.md, CITATION.cff, pyproject.toml, uv.lock, aus_accounting_mcp/**, scripts/**, tests/**, README.md, DISCLAIMER.md, CONTEXT.md, CONTRIBUTING.md, RELEASE_NOTES.md, compatibility.json, server.json, docs/**

Scope: release the reviewed Division 7A engine and expose token-efficient benchmark-rate and single-loan review surfaces through thin, fail-closed MCP adapters.

- [x] G0: this ledger states outcomes that can fail
  CHECK: node "%USERPROFILE%\.codex\skills\unlazy\scripts\gate-lint.mjs" GATES.md
  EXPECT: LINT OK
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=C:\Users\-\Documents\Codex\2026-08-31\does\work\github-actioning\aus-accounting-mcp; path=d9cef65bfc6f/30 entries; EXPECT=matched; output-sha256=48630b7361dd44ee870917b12c3d19b9d7bdea738aaca16bb04d4cab83b772d2; output-bytes=8

- [x] G1: div7a-loan-review 0.1.0 is available from PyPI with a matching GitHub release
  CHECK: python -c "import json,subprocess,urllib.request; p=json.load(urllib.request.urlopen('https://pypi.org/pypi/div7a-loan-review/0.1.0/json')); tag=subprocess.check_output(['gh','api','repos/ryanduguid/div7a-loan-review/releases/tags/v0.1.0','--jq','.tag_name'],text=True).strip(); assert p['info']['version']=='0.1.0' and tag=='v0.1.0'; print('DIV7A_ENGINE_RELEASE_OK')"
  EXPECT: DIV7A_ENGINE_RELEASE_OK
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=C:\Users\-\Documents\Codex\2026-08-31\does\work\github-actioning\aus-accounting-mcp; path=d9cef65bfc6f/30 entries; EXPECT=matched; output-sha256=924e117c7d29d133a9782e9b06474af78afb3ba8b8fa7fe06a08abd0a75af20e; output-bytes=25

- [x] G2: the MCP exposes reviewed Division 7A rate and single-loan tools while unsupported matters remain refused
  CHECK: uv run --locked --extra dev pytest -q tests/test_div7a.py tests/test_server_money.py tests/test_engine_versions.py && echo MCP_DIV7A_TOOLS_OK
  EXPECT: MCP_DIV7A_TOOLS_OK
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=C:\Users\-\Documents\Codex\2026-08-31\does\work\github-actioning\aus-accounting-mcp; path=d9cef65bfc6f/30 entries; EXPECT=matched; output-sha256=c207ae3bff4cff67607d6b3b7f35e48ed56687ded245e47136410457a83d62cd; output-bytes=121

- [x] G3: the complete MCP regression suite passes
  CHECK: uv run --locked --extra dev pytest -q && echo MCP_REGRESSION_OK
  EXPECT: MCP_REGRESSION_OK
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=C:\Users\-\Documents\Codex\2026-08-31\does\work\github-actioning\aus-accounting-mcp; path=d9cef65bfc6f/30 entries; EXPECT=matched; output-sha256=bd7e5018a8cb303dcc97cc1659787c5dba71c1439212e7206c64dcc797e21a2f; output-bytes=202

- [x] G4: repository-defined lint and type checks pass
  CHECK: uv run --locked --extra dev ruff check aus_accounting_mcp tests && uv run --locked --extra dev mypy aus_accounting_mcp && echo MCP_STATIC_CHECKS_OK
  EXPECT: MCP_STATIC_CHECKS_OK
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=C:\Users\-\Documents\Codex\2026-08-31\does\work\github-actioning\aus-accounting-mcp; path=d9cef65bfc6f/30 entries; EXPECT=matched; output-sha256=e538f8a0aaf83e6adbd2f15abe34d6f158a1b3dfbc4553b461cde2c5bea2e2e5; output-bytes=86

- [x] G5: the exact lock, built distribution, and registered-tool demo work together
  CHECK: uv lock --check && uv run --locked --extra dev python -m build && uv run --locked aus-accounting-mcp-demo && echo MCP_PACKAGE_DEMO_OK
  EXPECT: MCP_PACKAGE_DEMO_OK
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=C:\Users\-\Documents\Codex\2026-08-31\does\work\github-actioning\aus-accounting-mcp; path=d9cef65bfc6f/30 entries; EXPECT=matched; output-sha256=001879bab5cfb79d33462efaf57eac110c2cf6bd6be346b93b36bfeca494f5bd; output-bytes=4849

- [x] G6: the final patch has no whitespace errors
  CHECK: git diff --check && echo MCP_DIFF_OK
  EXPECT: MCP_DIFF_OK
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=C:\Users\-\Documents\Codex\2026-08-31\does\work\github-actioning\aus-accounting-mcp; path=d9cef65bfc6f/30 entries; EXPECT=matched; output-sha256=b50b3a8297ad85bb06e2544103520577993707029adc4524ad8a17bd1547af76; output-bytes=2719

- [x] G7: compact Division 7A results retain outcomes and use less than half the full JSON payload
  CHECK: uv run --locked --extra dev pytest -q tests/test_div7a.py -k summary && echo MCP_DIV7A_SUMMARY_OK
  EXPECT: MCP_DIV7A_SUMMARY_OK
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=C:\Users\-\Documents\Codex\2026-08-31\does\work\github-actioning\aus-accounting-mcp; path=d9cef65bfc6f/30 entries; EXPECT=matched; output-sha256=295bf66bd9000f35f331e7daece120312b6852da91ec07ff3be2011c2a7da7bf; output-bytes=136
