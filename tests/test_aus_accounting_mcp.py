import base64
import importlib
import json
import re
from pathlib import Path
from urllib.parse import parse_qs, urlparse

try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

import pytest

from aus_accounting_mcp.server import (
    refuse_div7a,
    calc_payday_super_deadline,
    generate_synthetic_sbr_fixture,
    get_ato_benchmarks,
    list_ato_benchmark_industries,
)

CANONICAL_REPOSITORY = "https://github.com/ryanduguid/aus-accounting-mcp"


def test_proof_package_surface_is_versioned_and_keeps_stdio_separate() -> None:
    root = Path(__file__).resolve().parents[1]
    project = tomllib.loads((root / "pyproject.toml").read_text(encoding="utf-8"))["project"]
    assert project["name"] == "aus-accounting-mcp"
    assert project["version"] == "0.1.6"
    assert project["scripts"] == {
        "aus-accounting-mcp": "aus_accounting_mcp.cli:main",
        "aus-accounting-mcp-demo": "aus_accounting_mcp.demo:main",
    }
    dev_dependencies = project["optional-dependencies"]["dev"]
    assert "Pillow==12.3.0" in dev_dependencies
    assert "twine==6.2.0" in dev_dependencies
    assert 'tomli>=2.0.1; python_version < "3.11"' in dev_dependencies


def test_build_backend_is_exactly_pinned_for_reproducible_wheels() -> None:
    root = Path(__file__).resolve().parents[1]
    build_system = tomllib.loads((root / "pyproject.toml").read_text(encoding="utf-8"))[
        "build-system"
    ]

    assert build_system == {
        "build-backend": "hatchling.build",
        "requires": ["hatchling==1.29.0"],
    }


def _workflow_sources(root: Path) -> dict[str, str]:
    workflow_dir = root / ".github" / "workflows"
    return {
        path.name: path.read_text(encoding="utf-8")
        for path in sorted(workflow_dir.iterdir())
        if path.is_file() and path.suffix in {".yml", ".yaml"}
    }


def _yaml_mapping_lines(source: str) -> list[str]:
    mapping_lines: list[str] = []
    block_scalar_indent: int | None = None
    block_scalar = re.compile(r"^\s*(?:-\s+)?[^:#][^:]*:\s*[|>][+-]?\s*(?:#.*)?$")

    for line in source.splitlines():
        stripped = line.strip()
        indent = len(line) - len(line.lstrip(" "))
        if block_scalar_indent is not None:
            if not stripped or indent > block_scalar_indent:
                continue
            block_scalar_indent = None

        if not stripped or stripped.startswith("#"):
            continue
        mapping_line = line.split(" #", 1)[0].rstrip()
        mapping_lines.append(mapping_line)
        if block_scalar.fullmatch(line):
            block_scalar_indent = indent

    return mapping_lines


def _workflow_jobs(source: str) -> dict[str, str]:
    jobs: dict[str, str] = {}
    current_name: str | None = None
    current_lines: list[str] = []
    in_jobs = False

    for line in source.splitlines():
        if not in_jobs:
            if re.fullmatch(r"jobs:\s*(?:#.*)?", line):
                in_jobs = True
            continue

        stripped = line.strip()
        indent = len(line) - len(line.lstrip(" "))
        if stripped and not stripped.startswith("#") and indent == 0:
            break

        job_header = re.fullmatch(r"  ([A-Za-z0-9_-]+):\s*(?:#.*)?", line)
        if job_header:
            if current_name is not None:
                jobs[current_name] = "\n".join(current_lines)
            current_name = job_header.group(1)
            current_lines = [line]
        elif current_name is not None:
            current_lines.append(line)

    if current_name is not None:
        jobs[current_name] = "\n".join(current_lines)
    return jobs


def _yaml_block(source: str, key: str, indent: int) -> str | None:
    lines = source.splitlines()
    spaces = " " * indent
    header = re.compile(rf"^{spaces}{re.escape(key)}:\s*(?:#.*)?$")
    start = next((index for index, line in enumerate(lines) if header.fullmatch(line)), None)
    if start is None:
        return None

    end = len(lines)
    for index in range(start + 1, len(lines)):
        line = lines[index]
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        line_indent = len(line) - len(line.lstrip(" "))
        if line_indent <= indent:
            end = index
            break
    return "\n".join(lines[start:end])


def _job_run_scripts(job: str) -> list[str]:
    lines = job.splitlines()
    scripts: list[str] = []
    run_line = re.compile(r"^(?P<indent>\s*)(?:-\s+)?run:\s*(?P<body>.*)$")
    index = 0

    while index < len(lines):
        match = run_line.fullmatch(lines[index])
        if match is None:
            index += 1
            continue

        indent = len(match.group("indent"))
        body = match.group("body").strip()
        if re.fullmatch(r"[|>][+-]?", body):
            script_lines: list[str] = []
            index += 1
            while index < len(lines):
                line = lines[index]
                stripped = line.strip()
                line_indent = len(line) - len(line.lstrip(" "))
                if stripped and line_indent <= indent:
                    break
                if stripped and not stripped.startswith("#"):
                    script_lines.append(stripped)
                index += 1
            scripts.append("\n".join(script_lines))
            continue

        scripts.append(body.split(" #", 1)[0].rstrip())
        index += 1

    return scripts


def _pypi_publisher_uses(workflows: dict[str, str]) -> list[tuple[str, str]]:
    action = re.compile(
        r"^\s*(?:-\s+)?uses:\s*['\"]?"
        r"pypa/gh-action-pypi-publish@[^'\"\s#]+['\"]?$",
        re.IGNORECASE,
    )
    uses: list[tuple[str, str]] = []
    for filename, source in sorted(workflows.items()):
        for job_name, job in _workflow_jobs(source).items():
            uses.extend(
                (filename, job_name) for line in _yaml_mapping_lines(job) if action.fullmatch(line)
            )
    return uses


def _assert_registered_pypi_publisher(workflows: dict[str, str]) -> None:
    release = workflows["release.yml"]
    release_jobs = _workflow_jobs(release)

    assert "workflow_dispatch:" not in release
    assert not any(
        re.search(r"pypi|backfill", job_name, re.IGNORECASE) for job_name in release_jobs
    ), "release workflow must not contain a PyPI or backfill job"
    assert not any(
        re.fullmatch(r"\s*environment:\s*pypi", line, re.IGNORECASE)
        for line in _yaml_mapping_lines(release)
    ), "release workflow must not select the pypi environment"
    assert '  push:\n    tags:\n      - "v*"' in release
    release_job = release_jobs.get("release")
    assert release_job is not None
    assert (
        re.search(
            r"(?m)^    uses:\s*"
            r"ryanduguid/release-policy/\.github/workflows/release-python\.yml@",
            "\n".join(_yaml_mapping_lines(release_job)),
        )
        is not None
    )

    publisher_uses = _pypi_publisher_uses(workflows)
    assert len(publisher_uses) == 1, (
        "expected exactly one pypa/gh-action-pypi-publish use across every workflow, "
        f"found {publisher_uses}"
    )
    publisher_file, publisher_job_name = publisher_uses[0]
    assert publisher_file == "publish-pypi.yml", (
        "the trusted publisher use must be in publish-pypi.yml"
    )

    publisher = workflows[publisher_file]
    publisher_job = _workflow_jobs(publisher)[publisher_job_name]
    publisher_mapping = "\n".join(_yaml_mapping_lines(publisher_job))
    assert re.search(r"(?m)^    environment:\s*pypi\s*$", publisher_mapping, re.IGNORECASE), (
        "publishing job must select environment: pypi"
    )

    permissions = _yaml_block(publisher_job, "permissions", 4)
    assert permissions is not None, "publishing job must declare job permissions"
    assert re.search(r"(?m)^      id-token:\s*write\s*(?:#.*)?$", permissions), (
        "publishing job permissions must grant id-token: write"
    )
    assert any(
        "sha256sum --check SHA256SUMS" in script for script in _job_run_scripts(publisher_job)
    ), "publishing job must verify SHA256SUMS"
    assert re.search(r"\$\{\{[^}\n]*\binputs\.tag\b[^}\n]*\}\}", publisher_mapping), (
        "publishing job must consume inputs.tag"
    )

    dispatch = _yaml_block(publisher, "workflow_dispatch", 2)
    assert dispatch is not None, "publish workflow must allow manual dispatch"
    inputs = _yaml_block(dispatch, "inputs", 4)
    assert inputs is not None, "workflow_dispatch must declare inputs"
    tag = _yaml_block(inputs, "tag", 6)
    assert tag is not None, "workflow_dispatch must declare a tag input"
    assert re.search(r"(?m)^        required:\s*true\s*(?:#.*)?$", tag, re.IGNORECASE), (
        "workflow_dispatch tag input must be required"
    )
    assert re.search(r"(?m)^        type:\s*string\s*(?:#.*)?$", tag, re.IGNORECASE), (
        "workflow_dispatch tag input must be a string"
    )


def _valid_pypi_workflow_fixture() -> dict[str, str]:
    return {
        "release.yml": """\
name: Release
on:
  push:
    tags:
      - "v*"
jobs:
  release:
    uses: ryanduguid/release-policy/.github/workflows/release-python.yml@abc123
""",
        "publish-pypi.yml": """\
name: Publish to PyPI
on:
  workflow_dispatch:
    inputs:
      tag:
        required: true
        type: string
jobs:
  publish:
    runs-on: ubuntu-latest
    environment: pypi
    permissions:
      id-token: write
    steps:
      - name: Verify the release assets
        env:
          TAG: ${{ inputs.tag }}
        run: |
          sha256sum --check SHA256SUMS
      - name: Publish
        uses: pypa/gh-action-pypi-publish@abc123
""",
    }


def test_inlined_simulators_are_gone() -> None:
    for name in (
        "aus_accounting_mcp.engines.paydaysuper_sim",
        "aus_accounting_mcp.engines.benchmarks",
        "aus_accounting_mcp.engines.div7a",
    ):
        with pytest.raises(ModuleNotFoundError):
            importlib.import_module(name)


def test_payday_on_time_uses_payday_super_checker() -> None:
    payload = calc_payday_super_deadline(
        qe_day="2026-08-06",
        sg_amount="800.00",
        remitted="2026-08-07",
        received="2026-08-10",
        as_at="2026-08-21",
    )
    assert payload["ok"] is True
    assert payload["engine"] == "payday-super-checker"
    assert payload["law_content_date"] == "2026-08-15"
    assert payload["result"]["verdict"] == "ON_TIME"
    assert payload["result"]["experimental_sgc_high"] is None
    assert "clearing-house latency" in payload["disclaimer"]


def test_payday_without_fund_receipt_is_at_risk() -> None:
    payload = calc_payday_super_deadline(
        qe_day="2026-08-06",
        sg_amount="800.00",
        remitted="2026-08-07",
        as_at="2026-08-21",
    )
    assert payload["result"]["verdict"] == "AT_RISK"
    assert any("receipt by the fund" in c for c in payload["result"]["caveats"])


def test_payday_pre_regime_is_refused() -> None:
    with pytest.raises(ValueError, match="1 Jul 2026"):
        calc_payday_super_deadline(
            qe_day="2026-06-15",
            sg_amount="800.00",
            received="2026-06-20",
            as_at="2026-08-21",
        )


def test_payday_transition_period_cannot_be_confirmed_by_the_mcp() -> None:
    with pytest.raises(ValueError, match="this MCP cannot confirm"):
        calc_payday_super_deadline(
            qe_day="2026-07-09",
            sg_amount="800.00",
            received="2026-07-15",
            as_at="2026-08-10",
        )


def test_payday_rejects_non_decimal_amounts() -> None:
    with pytest.raises(ValueError, match="sg_amount"):
        calc_payday_super_deadline(
            qe_day="2026-08-06",
            sg_amount="nope",
            received="2026-08-10",
            as_at="2026-08-21",
        )


def test_ato_benchmarks_use_shipped_dataset() -> None:
    listed = list_ato_benchmark_industries(search="baker")
    assert listed["ok"] is True
    assert listed["engine"] == "ato-benchmark-compare"
    assert any(item["name"] == "Bakeries and hot bread shops" for item in listed["industries"])

    payload = get_ato_benchmarks(
        industry="Bakeries and hot bread shops",
        turnover="850000.00",
        # An explicit zero is evidence, so the ATO turnover rule is settled and
        # the ratios are definite. Omitting it is what makes them not_supplied.
        other_income="0",
        cost_of_sales="270000.00",
        cost_of_sales_labour="0",
        other_expense="437000.00",
        salary_wages="120000.00",
        contractor_commission="0",
        rent="40000.00",
        motor_vehicle="8000.00",
        associated_persons="0",
        w1="120000.00",
    )
    assert payload["ok"] is True
    assert payload["business_type"] == "Bakeries and hot bread shops"
    assert payload["complete_buckets"] is True
    assert payload["source"]["publisher"]
    assert payload["turnover"] == "850000.00"
    assert payload["turnover_basis"] == "sales of goods and services"
    statuses = {row["ratio"]: row["status"] for row in payload["ratios"]}
    assert statuses["cost_of_sales_to_turnover"] == "within"


def test_ato_omitted_buckets_are_not_treated_as_zero() -> None:
    payload = get_ato_benchmarks(
        industry="Bakeries and hot bread shops",
        turnover="850000.00",
        other_income="0",
        cost_of_sales="270000.00",
    )
    statuses = {row["ratio"]: row["status"] for row in payload["ratios"]}
    # other_income is supplied as an evidenced zero so that the denominator is
    # settled and this test keeps isolating its own subject: a supplied bucket
    # produces a ratio, an omitted one does not. Withholding every ratio when
    # other_income is omitted is covered by the other_income tests below.
    assert statuses["cost_of_sales_to_turnover"] == "within"
    assert statuses["rent_to_turnover"] == "not_supplied"
    assert statuses["total_expenses_to_turnover"] == "not_supplied"
    assert "rent" in payload["omitted_buckets"]
    assert payload["complete_buckets"] is False
    # An omitted bucket reached the engine as a zero, so its total and the
    # figures asserted below are reported as unknown rather than as amounts.
    assert payload["bucket_totals"]["rent"] is None
    assert payload["figures"]["total_expenses"] is None
    assert payload["figures"]["total_expenses_for_ratio"] is None
    assert payload["figures"]["labour"] is None
    assert payload["figures"]["payments_to_associated_persons"] is None
    # A supplied bucket is still evidenced, and an explicit zero for other income
    # settles the income side, so the denominator and the figures behind it stay
    # definite. Omitting it is covered by the other_income tests below.
    assert payload["bucket_totals"]["cost_of_sales"] == "270000.00"
    assert payload["figures"]["cost_of_sales_for_ratio"] == "270000.00"
    assert payload["figures"]["total_business_income"] == "850000.00"
    assert payload["figures"]["other_business_income"] == "0"


def _bakery_with_other_income(other_income: str | None) -> dict:
    """Sales of $400,000 against other income of $500,000.

    Other income above sales is exactly what moves the ATO denominator off the
    sales label and onto total business income, so this pair of figures sits on
    either side of the fallback an omitted bucket would hide.
    """
    figures = {
        "industry": "Bakeries and hot bread shops",
        "turnover": "400000.00",
        "cost_of_sales": "120000.00",
        "cost_of_sales_labour": "0",
        "salary_wages": "90000.00",
        "contractor_commission": "0",
        "associated_persons": "0",
        "rent": "30000.00",
        "motor_vehicle": "6000.00",
        "other_expense": "100000.00",
    }
    if other_income is not None:
        figures["other_income"] = other_income
    return get_ato_benchmarks(**figures)


def test_ato_other_income_can_move_the_turnover_band() -> None:
    # test_ato_omitted_other_income_withholds_every_ratio shows the denominator
    # moving a ratio within one band. It also moves the band itself, and with it
    # the published ATO range a business is judged against, which is why a
    # withheld row cannot carry a benchmark range either. Here the same expenses
    # sit above the range on one denominator and below it on the other.
    nil = _bakery_with_other_income("0")
    assert nil["turnover"] == "400000.00"
    assert nil["turnover_basis"] == "sales of goods and services"
    assert nil["turnover_band"]["band"] == "low"
    nil_rows = {row["ratio"]: row for row in nil["ratios"]}
    assert nil_rows["cost_of_sales_to_turnover"]["value"] == "0.3000"
    assert nil_rows["total_expenses_to_turnover"]["status"] == "above"

    crossed = _bakery_with_other_income("500000.00")
    assert crossed["turnover"] == "900000.00"
    assert crossed["turnover_basis"] == "total business income"
    assert crossed["turnover_band"]["band"] == "high"
    crossed_rows = {row["ratio"]: row for row in crossed["ratios"]}
    assert crossed_rows["cost_of_sales_to_turnover"]["value"] == "0.1333"
    # Same expenses, opposite finding against the ATO range.
    assert crossed_rows["total_expenses_to_turnover"]["status"] == "below"
    assert (
        crossed_rows["total_expenses_to_turnover"]["benchmark_min"]
        != nil_rows["total_expenses_to_turnover"]["benchmark_min"]
    )


def test_ato_omitted_other_income_withholds_the_turnover_band_and_ranges() -> None:
    # test_ato_omitted_other_income_withholds_every_ratio covers the ratios.
    # This covers everything else the same denominator decides, which would
    # otherwise be published as definite beside those not_supplied rows.
    payload = _bakery_with_other_income(None)
    assert payload["turnover"] is None
    assert payload["turnover_basis"] is None
    assert payload["turnover_band"] is None
    assert payload["figures"]["total_business_income"] is None
    assert payload["figures"]["other_business_income"] is None
    for row in payload["ratios"]:
        assert row["status"] == "not_supplied", row["ratio"]
        assert row["value"] is None
        assert row["percent"] is None
        # The range belongs to a turnover band that was never established.
        assert row["benchmark_min"] is None
        assert row["benchmark_max"] is None
    assert "other_income" in payload["omitted_buckets"]
    assert any("are withheld for the same reason" in note for note in payload["notes"])
    # The buckets the operator did supply are still reported as supplied.
    assert payload["figures"]["sales_of_goods_and_services"] == "400000.00"
    assert payload["bucket_totals"]["cost_of_sales"] == "120000.00"
    assert payload["complete_buckets"] is True


def test_ato_omitted_other_income_withholds_notes_that_quote_turnover() -> None:
    # Sales below the lowest published band make the engine conclude, from a
    # denominator it was handed rather than given, that the ATO benchmarks do
    # not apply to this business at all. Nulling the structured fields while
    # publishing that sentence would leave the same unevidenced denominator in
    # prose, so the note is withheld with them.
    figures = {
        "industry": "Bakeries and hot bread shops",
        "turnover": "50000.00",
        "cost_of_sales": "15000.00",
    }
    omitted = get_ato_benchmarks(**figures)
    assert not any("50,000.00" in note for note in omitted["notes"])
    assert not any("do not apply" in note for note in omitted["notes"])
    assert any("are withheld for the same reason" in note for note in omitted["notes"])

    # It would have been the wrong conclusion. The same expenses with the bucket
    # supplied put this business inside a published band.
    supplied = get_ato_benchmarks(**figures, other_income="500000.00")
    assert supplied["turnover"] == "550000.00"
    assert supplied["turnover_band"]["band"] == "medium"

    # An evidenced nil keeps the engine's note, because there the figure it
    # quotes is one the operator established.
    nil = get_ato_benchmarks(**figures, other_income="0")
    assert nil["turnover_band"] is None
    assert any("below the lowest published range" in note for note in nil["notes"])


def test_ato_w1_without_associated_persons_is_not_supplied() -> None:
    # The engine computes labour as W1 less payments to associated persons plus
    # contractor commission, so an omitted associates bucket taints the ratio.
    payload = get_ato_benchmarks(
        industry="Bakeries and hot bread shops",
        turnover="850000.00",
        # Supplied so the labour rule is what decides this ratio, not the
        # separate other_income gate on the denominator.
        other_income="0",
        salary_wages="120000.00",
        contractor_commission="0",
        cost_of_sales_labour="0",
        w1="200000.00",
    )
    statuses = {row["ratio"]: row["status"] for row in payload["ratios"]}
    assert statuses["labour_to_turnover"] == "not_supplied"


def test_ato_w1_does_not_substitute_for_salary_wages() -> None:
    # The engine takes the greater of W1 and the rebuilt salary and wages label,
    # so with salary_wages omitted nobody knows which side wins and the labour it
    # returns is only a lower bound. Publishing that as an amount would contradict
    # the same payload, which reports the salary_wages bucket as unknown.
    payload = get_ato_benchmarks(
        industry="Bakeries and hot bread shops",
        turnover="850000.00",
        contractor_commission="0",
        cost_of_sales_labour="0",
        associated_persons="0",
        w1="200000.00",
    )
    assert payload["bucket_totals"]["salary_wages"] is None
    assert payload["figures"]["labour"] is None
    statuses = {row["ratio"]: row["status"] for row in payload["ratios"]}
    assert statuses["labour_to_turnover"] == "not_supplied"


def test_ato_partial_labour_picture_is_not_supplied() -> None:
    payload = get_ato_benchmarks(
        industry="Bakeries and hot bread shops",
        turnover="850000.00",
        other_income="0",
        salary_wages="120000.00",
    )
    statuses = {row["ratio"]: row["status"] for row in payload["ratios"]}
    assert statuses["labour_to_turnover"] == "not_supplied"


def test_ato_complete_labour_picture_is_evidenced() -> None:
    # other_income is supplied so the denominator is established and this test
    # isolates the labour buckets, which are its subject.
    payload = get_ato_benchmarks(
        industry="Bakeries and hot bread shops",
        turnover="850000.00",
        other_income="0",
        salary_wages="120000.00",
        contractor_commission="0",
        cost_of_sales_labour="0",
    )
    rows = {row["ratio"]: row for row in payload["ratios"]}
    assert rows["labour_to_turnover"]["status"] != "not_supplied"
    assert rows["labour_to_turnover"]["value"] is not None


def test_ato_omitted_other_income_withholds_every_ratio() -> None:
    # The ATO rule divides by sales, or by total business income once other
    # income exceeds sales. An omitted other_income reaches the engine as zero,
    # which picks sales, the smallest denominator the rule can select, so every
    # ratio built on it is an upper bound rather than an amount. These same
    # figures read 35.00% "within" against sales and 8.75% "below" once other
    # income of 300000.00 moves the denominator, so publishing either as
    # definite states a verdict the figures do not support.
    omitted = get_ato_benchmarks(
        industry="Bakeries and hot bread shops",
        turnover="100000.00",
        cost_of_sales="35000.00",
    )
    statuses = {row["ratio"]: row["status"] for row in omitted["ratios"]}
    assert set(statuses.values()) == {"not_supplied"}
    assert any("other_business_income was omitted" in note for note in omitted["notes"])

    # Establishing the figure restores the ratio, and the two candidate
    # denominators disagree on the verdict.
    against_sales = get_ato_benchmarks(
        industry="Bakeries and hot bread shops",
        turnover="100000.00",
        cost_of_sales="35000.00",
        other_income="0",
    )
    rows = {row["ratio"]: row for row in against_sales["ratios"]}
    assert rows["cost_of_sales_to_turnover"]["status"] == "within"
    assert rows["cost_of_sales_to_turnover"]["percent"] == "35.00%"

    against_total_income = get_ato_benchmarks(
        industry="Bakeries and hot bread shops",
        turnover="100000.00",
        cost_of_sales="35000.00",
        other_income="300000.00",
    )
    rows = {row["ratio"]: row for row in against_total_income["ratios"]}
    assert rows["cost_of_sales_to_turnover"]["status"] == "below"
    assert rows["cost_of_sales_to_turnover"]["percent"] == "8.75%"


def test_ato_refuses_turnover_only() -> None:
    with pytest.raises(ValueError, match="no expense figures"):
        get_ato_benchmarks(industry="Bakeries and hot bread shops", turnover="850000.00")


def test_ato_unknown_industry_is_refused() -> None:
    with pytest.raises(ValueError, match="no ATO business type"):
        get_ato_benchmarks(
            industry="interstellar freight",
            turnover="100000.00",
            cost_of_sales="0",
        )


def test_div7a_is_refused() -> None:
    payload = refuse_div7a("Alice", "HoldingCo Pty Ltd", "50000.00")
    assert payload["ok"] is False
    assert payload["available"] is False
    assert payload["reviewed_engine"] is False
    assert payload["code"] == "ERR_POLICY_DIV7A_REFUSED"
    assert "payday-super-checker" in payload["reason"]


def test_synthetic_sbr_fixtures_are_labelled() -> None:
    ctr = generate_synthetic_sbr_fixture("CTR", revenue_or_sales="1000000.00")
    assert ctr["synthetic"] is True
    assert ctr["not_a_lodgment"] is True
    assert ctr["form_type"] == "CTR_AU_2025"
    assert ctr["income_statement"]["gross_profit"] == "600000.00"

    bas = generate_synthetic_sbr_fixture("BAS", revenue_or_sales="110000.00")
    assert bas["gst_labels"]["1A_gst_on_sales"] == "10000.00"


def test_client_snippets_use_uvx_from_pypi() -> None:
    root = Path(__file__).resolve().parents[1]
    expected_args = ["aus-accounting-mcp"]
    for name in ("cursor_mcp.json", "claude_desktop_config.json", "antigravity_config.json"):
        payload = json.loads((root / "clients" / name).read_text(encoding="utf-8"))
        server = payload["mcpServers"]["aus-accounting"]
        assert server["command"] == "uvx"
        assert server["args"] == expected_args
    readme = (root / "README.md").read_text(encoding="utf-8")
    disclaimer = (root / "DISCLAIMER.md").read_text(encoding="utf-8")
    assert "uvx aus-accounting-mcp" in readme
    assert f"git+{CANONICAL_REPOSITORY}" not in readme
    cursor_link = re.search(r"https://cursor\.com/en/install-mcp\?[^)]+", readme)
    assert cursor_link is not None
    cursor_config = parse_qs(urlparse(cursor_link.group(0)).query)["config"]
    assert len(cursor_config) == 1
    assert json.loads(base64.urlsafe_b64decode(cursor_config[0]).decode("utf-8")) == {
        "command": "uvx",
        "args": expected_args,
    }
    assert "<!-- mcp-name: io.github.ryanduguid/aus-accounting -->" in readme
    assert "DISCLAIMER.md" in readme
    assert "glama.ai/mcp/servers/ryanduguid/au-tax-mcp-server" in readme
    assert "not tax" in disclaimer.lower()
    assert "synthetic: true" in disclaimer
    citation = (root / "CITATION.cff").read_text(encoding="utf-8")
    assert CANONICAL_REPOSITORY in citation
    glama = json.loads((root / "glama.json").read_text(encoding="utf-8"))
    assert glama["maintainers"] == ["ryanduguid"]
    pyproject = (root / "pyproject.toml").read_text(encoding="utf-8")
    assert 'version = "0.1.6"' in pyproject
    assert "uvx from PyPI" in pyproject
    # The engines stay pinned to an exact version, which is what the commit pins
    # used to buy. They cannot be pinned by URL: PyPI rejects a distribution
    # whose metadata carries a direct reference, so a git pin here would make
    # this package unpublishable and silently undo its own release process.
    assert "payday-super-checker==" in pyproject
    assert "ato-benchmark-compare==" in pyproject
    dependencies = pyproject.split("dependencies = [", 1)[1].split("]", 1)[0]
    assert "git+" not in dependencies
    assert "allow-direct-references" not in pyproject


def test_unreleased_source_metadata_stays_separate_from_published_metadata() -> None:
    root = Path(__file__).resolve().parents[1]
    project = tomllib.loads((root / "pyproject.toml").read_text(encoding="utf-8"))["project"]
    citation = (root / "CITATION.cff").read_text(encoding="utf-8")
    release_notes = (root / "RELEASE_NOTES.md").read_text(encoding="utf-8")
    server = json.loads((root / "server.json").read_text(encoding="utf-8"))
    published_version = server["version"]

    assert project["version"] == "0.1.6"
    assert re.search(r"(?m)^version: 0\.1\.6$", citation)
    assert not re.search(r"(?m)^date-released:", citation)
    assert release_notes.startswith("# v0.1.6\n")
    assert published_version == server["packages"][0]["version"] == "0.1.5"
    assert project["version"] != published_version


def test_active_repository_metadata_uses_canonical_identity() -> None:
    root = Path(__file__).resolve().parents[1]
    project = tomllib.loads((root / "pyproject.toml").read_text(encoding="utf-8"))["project"]
    server = json.loads((root / "server.json").read_text(encoding="utf-8"))
    citation = (root / "CITATION.cff").read_text(encoding="utf-8")
    readme = (root / "README.md").read_text(encoding="utf-8")

    assert project["urls"]["Repository"] == f"{CANONICAL_REPOSITORY}.git"
    assert server["repository"]["url"] == CANONICAL_REPOSITORY
    assert CANONICAL_REPOSITORY in citation
    assert "repository aus-accounting-mcp" in readme


def test_readme_has_stable_proof_anchor_and_mapping() -> None:
    root = Path(__file__).resolve().parents[1]
    readme = (root / "README.md").read_text(encoding="utf-8")
    assert "## 30-second proof\n" in readme
    assert readme.index("## 30-second proof") < readme.index("## Install")
    for text in (
        "![Animated terminal proof of synthetic BAS output and Division 7A refusal](docs/quick-proof.gif)",
        "uv run --locked aus-accounting-mcp-demo",
        "[checked text transcript](docs/quick-proof.txt)",
        "Expected structured success:",
        "synthetic: true",
        "not_a_lodgment: true",
        "Expected structured refusal:",
        "ERR_POLICY_DIV7A_REFUSED",
        "not a lodgment",
        "human review",
        "repository aus-accounting-mcp",
        "aus-accounting-mcp",
        "aus-accounting-mcp-demo",
        "io.github.ryanduguid/aus-accounting",
        "https://pypi.org/project/aus-accounting-mcp/0.1.5/",
        "https://registry.modelcontextprotocol.io/v0.1/servers/io.github.ryanduguid%2Faus-accounting/versions/0.1.5",
        "[compatibility.json](compatibility.json)",
    ):
        assert text in readme


def test_server_metadata_publishes_exact_pypi_release() -> None:
    root = Path(__file__).resolve().parents[1]
    server = json.loads((root / "server.json").read_text(encoding="utf-8"))

    assert server["version"] == "0.1.5"
    assert server["packages"] == [
        {
            "registryType": "pypi",
            "identifier": "aus-accounting-mcp",
            "version": "0.1.5",
            "transport": {"type": "stdio"},
        }
    ]


def test_pypi_route_rejects_a_second_publisher_workflow() -> None:
    workflows = _valid_pypi_workflow_fixture()
    workflows["backfill.yaml"] = """\
name: Backfill PyPI
jobs:
  backfill:
    runs-on: ubuntu-latest
    steps:
      - uses: pypa/gh-action-pypi-publish@def456
"""

    with pytest.raises(AssertionError, match="exactly one"):
        _assert_registered_pypi_publisher(workflows)


def test_pypi_route_rejects_controls_parked_in_a_dummy_job() -> None:
    workflows = _valid_pypi_workflow_fixture()
    workflows["publish-pypi.yml"] = """\
name: Publish to PyPI
on:
  workflow_dispatch:
    inputs:
      tag:
        required: true
        type: string
jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: pypa/gh-action-pypi-publish@abc123
  dummy:
    runs-on: ubuntu-latest
    environment: pypi
    permissions:
      id-token: write
    steps:
      - env:
          TAG: ${{ inputs.tag }}
        run: |
          sha256sum --check SHA256SUMS
"""

    with pytest.raises(AssertionError, match="publishing job"):
        _assert_registered_pypi_publisher(workflows)


def test_pypi_route_rejects_an_unused_manual_tag() -> None:
    workflows = _valid_pypi_workflow_fixture()
    workflows["publish-pypi.yml"] = workflows["publish-pypi.yml"].replace(
        "${{ inputs.tag }}", "${{ github.ref_name }}"
    )

    with pytest.raises(AssertionError, match="inputs.tag"):
        _assert_registered_pypi_publisher(workflows)


def test_release_workflows_use_registered_pypi_publisher() -> None:
    root = Path(__file__).resolve().parents[1]
    readme = (root / "README.md").read_text(encoding="utf-8")

    _assert_registered_pypi_publisher(_workflow_sources(root))

    assert "[CITATION.cff](CITATION.cff)" in readme
    assert (
        f"[v0.1.5 release record]({CANONICAL_REPOSITORY}/releases/tag/v0.1.5)"
        in readme
    )


def test_registry_publisher_is_pinned_and_checksum_verified() -> None:
    root = Path(__file__).resolve().parents[1]
    workflow = (root / ".github" / "workflows" / "publish-mcp.yml").read_text(encoding="utf-8")

    assert "releases/latest" not in workflow
    assert "/download/v1.8.1/mcp-publisher_linux_amd64.tar.gz" in workflow
    assert "a06c9096dcb9727c13555b6be26c7effa707b01f06a4c561ba7a3635443cf2cc" in workflow
    assert "sha256sum --check" in workflow
    assert "workflow_dispatch:" in workflow
    assert not re.search(r"(?m)^  push:\s*$", workflow)
    assert (
        workflow.index("sha256sum --check")
        < workflow.index("tar --extract")
        < workflow.index("./mcp-publisher login")
    )
