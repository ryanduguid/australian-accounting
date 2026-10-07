"""The comparison with the tables the ATO's calculators read, run on fabricated tables."""

import io
import json
import unittest
from contextlib import redirect_stdout
from unittest import mock

import rates

TC2 = ["DT_EFFECT", "DT_END", "TX_DESC", "AM_INC_MIN", "AM_INC_MAX", "AM_OFFSET", "PC_TAX_RATE"]
TC9 = ["NM_CALCN_TYPE", "DT_EFFECT", "DT_END", "AM_MIN_INCOME", "AM_MAX_INCOME", "PC_TAX_RATE_A"]
OPEN = "99999999999"


def tables(*tc2_rows, tc9_rows=()):
    def table(columns, rows):
        return {"columns": [{"name": c} for c in columns], "rows": [list(r) for r in rows]}
    return {"TC2TAXRTE": table(TC2, tc2_rows), "TC9GENTAC": table(TC9, tc9_rows)}


def record(rid, label, unit, value, **period):
    return {"id": rid, "label": label, "unit": unit, "value": value, "period": period}


SCALE_LABEL = "Resident individual income tax rates"
SCALE = [{"from": 0, "to": 18200, "base_tax": 0, "marginal_rate": 0},
         {"from": 18201, "to": 45000, "base_tax": 0, "marginal_rate": 15},
         {"from": 45001, "to": None, "base_tax": 4020, "marginal_rate": 30}]
SCALE_ROWS = [("2020-07-01", "9999-12-31", "Resident Full Year", "-99999999999", "18200", "0", "0"),
              ("2024-07-01", "2026-06-30", "Resident Full Year", "18201", "45000", "0", "0.16"),
              ("2026-07-01", "9999-12-31", "Resident Full Year", "45001", OPEN, "4020", "0.3"),
              ("2026-07-01", "9999-12-31", "Resident Full Year", "18201", "45000", "0", "0.15")]
LEVY_ROWS = [("2025-07-01", "9999-12-31", "Medicare Basic Levy", "35014", OPEN, "0", "0.02"),
             ("2025-07-01", "9999-12-31", "Medicare Basic Levy", "-99999999999", "28011", "0", "0"),
             ("2025-07-01", "9999-12-31", "Medicare Basic Levy", "28012", "35013", "0", "0.1")]


class ATOTablesTest(unittest.TestCase):
    def test_matching_figures_produce_no_finding(self):
        records = [record("scale", SCALE_LABEL, "tax_scale", SCALE, income_year="2026-27"),
                   record("sg", "Super guarantee percentage (general)", "percent", 12,
                          effective_from="2026-07-01", effective_to="2027-06-30"),
                   record("div7a", "Division 7A benchmark interest rate", "percent", 8.77,
                          income_year="2026-27")]
        lines, findings = rates.ato_table_report(records, tables(
            *SCALE_ROWS, ("2025-07-01", "9999-12-31", "Superannuation Guarantee Rate", "0", "0", "0", "0.12"),
            tc9_rows=[("DIV7A", "2026-07-01", "2027-06-30", "0", "0", "0.0877"),
                      ("DIV7A", "2025-07-01", "2026-06-30", "0", "0", "0.0837")]))
        self.assertEqual(findings, [])
        self.assertEqual(lines[:3], ["scale: matches TC2TAXRTE", "sg: matches TC2TAXRTE",
                                    "div7a: matches TC9GENTAC"])
        self.assertTrue(lines[-1].startswith("3 record(s) compared, 0 finding(s)."))


    def test_the_medicare_figures_come_from_the_basic_levy_rows(self):
        records = [record("lower", "Medicare levy low-income lower threshold, single, not SAPTO", "AUD",
                          28011, income_year="2025-26"),
                   record("upper", "Medicare levy low-income upper threshold, single, not SAPTO", "AUD",
                          35013, income_year="2025-26"),
                   record("rate", "Medicare levy rate", "percent", 2, as_at="2026-09-24")]
        self.assertEqual(rates.ato_table_report(records, tables(*LEVY_ROWS))[1], [])


    def test_a_different_figure_is_a_finding_that_shows_both(self):
        records = [record("fbt", "Fringe benefits tax rate", "percent", 48, effective_from="2026-04-01"),
                   record("scale", SCALE_LABEL, "tax_scale", SCALE, income_year="2025-26")]
        _, findings = rates.ato_table_report(records, tables(
            ("2017-04-01", "9999-03-31", "FBT rate", "0", "0", "0", "0.47"), *SCALE_ROWS))
        self.assertEqual(findings, [
            "fbt: TC2TAXRTE shows 47; the record has 48",
            "scale: TC2TAXRTE shows 16c for each $1 over $18,200; the record has "
            "15c for each $1 over $18,200; $4,020 plus 30c for each $1 over $45,000"])


    def test_a_missing_or_doubled_row_is_a_finding(self):
        records = [record("div7a", "Division 7A benchmark interest rate", "percent", 9,
                          income_year="2027-28"),
                   record("fbt", "Fringe benefits tax rate", "percent", 47, effective_from="2026-04-01")]
        fbt = ("2017-04-01", "9999-03-31", "FBT rate", "0", "0", "0", "0.47")
        _, findings = rates.ato_table_report(records, tables(
            fbt, fbt, tc9_rows=[("DIV7A", "2026-07-01", "2027-06-30", "0", "0", "0.0877")]))
        self.assertEqual(findings, [
            "div7a: cannot read TC9GENTAC {'NM_CALCN_TYPE': 'DIV7A'}: no row in force on 2027-07-01",
            "fbt: cannot read TC2TAXRTE {'TX_DESC': 'FBT rate'}: 2 rows in force, expected 1"])


    def test_records_the_tables_do_not_hold_are_listed_without_a_finding(self):
        lines, findings = rates.ato_table_report(
            [record("gst", "GST registration threshold", "AUD", 75000, as_at="2026-09-24")], tables())
        self.assertEqual(findings, [])
        self.assertEqual(lines, ["0 record(s) compared, 0 finding(s). Not in the ATO tables: gst"])


    def test_every_label_the_comparison_reads_belongs_to_a_record(self):
        self.assertLessEqual(set(rates.ATO_TABLE_FIGURES), {r["label"] for r in rates.load_records()})


    def test_the_command_reads_both_tables_and_exits_1_on_a_finding(self):
        served = tables(("2017-04-01", "9999-03-31", "FBT rate", "0", "0", "0", "0.5"))
        connections = []

        def fake_connection(host, timeout):
            self.assertEqual(host, "onlineservices.ato.gov.au")
            self.assertEqual(timeout, 60)
            connection = mock.Mock()
            name = list(served)[len(connections)]
            connection.getresponse.return_value.status = 200
            connection.getresponse.return_value.read.return_value = json.dumps(served[name]).encode()
            connections.append(connection)
            return connection

        output = io.StringIO()
        with mock.patch.object(rates, "HTTPSConnection", side_effect=fake_connection), mock.patch.object(
            rates, "load_records", return_value=[
                record("fbt", "Fringe benefits tax rate", "percent", 47,
                       effective_from="2026-04-01")]), redirect_stdout(output):
            self.assertEqual(rates.main(["ato-tables"]), 1)
        for connection, name in zip(connections, ("TC2TAXRTE", "TC9GENTAC"), strict=True):
            connection.request.assert_called_once_with(
                "GET", f"/cdn/static-data/codes-tables/{name}.json")
            connection.close.assert_called_once_with()
        self.assertIn("fbt: TC2TAXRTE shows 50; the record has 47", output.getvalue())

    def test_an_unreadable_table_ends_the_command_with_exit_1(self):
        output = io.StringIO()
        with mock.patch.object(rates, "HTTPSConnection", side_effect=OSError("network unreachable")), \
                redirect_stdout(output):
            self.assertEqual(rates.main(["ato-tables"]), 1)
        self.assertEqual(output.getvalue(), "cannot read the ATO tables: network unreachable\n")

    def test_redirects_and_http_errors_end_the_command_without_following_them(self):
        for status in (301, 302, 404, 500):
            with self.subTest(status=status), mock.patch.object(rates, "HTTPSConnection") as factory:
                connection = factory.return_value
                response = connection.getresponse.return_value
                response.status = status
                output = io.StringIO()
                with redirect_stdout(output):
                    self.assertEqual(rates.main(["ato-tables"]), 1)
                self.assertEqual(output.getvalue(),
                                 f"cannot read the ATO tables: TC2TAXRTE: HTTP {status}\n")
                factory.assert_called_once_with("onlineservices.ato.gov.au", timeout=60)
                connection.request.assert_called_once_with(
                    "GET", "/cdn/static-data/codes-tables/TC2TAXRTE.json")
                response.read.assert_not_called()
                connection.close.assert_called_once_with()

    def test_failed_requests_and_invalid_json_close_the_connection(self):
        for failed_request in (True, False):
            with self.subTest(failed_request=failed_request), \
                    mock.patch.object(rates, "HTTPSConnection") as factory:
                connection = factory.return_value
                if failed_request:
                    connection.request.side_effect = rates.HTTPException("invalid response")
                else:
                    connection.getresponse.return_value.status = 200
                    connection.getresponse.return_value.read.return_value = b"invalid JSON"
                output = io.StringIO()
                with redirect_stdout(output):
                    self.assertEqual(rates.main(["ato-tables"]), 1)
                self.assertTrue(output.getvalue().startswith("cannot read the ATO tables: "))
                connection.close.assert_called_once_with()
