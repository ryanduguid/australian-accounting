# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

# Fill a disposable copy of the workbook with fabricated rows and print
# what Excel calculates, for comparison with the checker on the same rows. The
# rows and overall status it reports are what tests/data/workbook-returned.json
# and workbook-events.json hold; each file says how to run it. -Events names a
# CSV in the checker's --events columns to fill the Events sheet with.
param([string]$Workbook, [string]$Csv, [string]$Out, [string]$AsAt, [string]$Events)
$ErrorActionPreference = 'Stop'
$copy = Join-Path ([IO.Path]::GetDirectoryName($Out)) ('wb-copy-' + [Guid]::NewGuid().ToString('N') + '.xlsx')
Copy-Item -LiteralPath $Workbook -Destination $copy
$rows = Import-Csv -LiteralPath $Csv
$excel = New-Object -ComObject Excel.Application
$excel.Visible = $false
$excel.DisplayAlerts = $false
try {
    $book = $excel.Workbooks.Open($copy)
    $reg = $book.Worksheets.Item('Register')
    $summary = $book.Worksheets.Item('Summary')
    $headers = @{}
    for ($c = 1; $c -le $reg.UsedRange.Columns.Count; $c++) {
        $headers[[string]$reg.Cells.Item(1, $c).Value2] = $c
    }
    $serial = { param($text) ([datetime]::ParseExact($text, 'yyyy-MM-dd', $null) - [datetime]'1899-12-30').TotalDays }
    $inputs = 'employee_id', 'payment_date', 'sg_amount', 'remitted_date', 'remitted_amount', 'matched_amount',
        'fund_received_date', 'first_contribution_to_fund', 'out_of_cycle', 'next_standard_payday',
        'defined_benefit', 'returned_by_fund'
    $dates = 'payment_date', 'remitted_date', 'fund_received_date', 'next_standard_payday'
    $amounts = 'sg_amount', 'remitted_amount', 'matched_amount'
    $table = $reg.ListObjects.Item('tblLines')
    if ($rows.Count -gt $table.ListRows.Count) { throw "only $($table.ListRows.Count) rows" }
    # An empty table row reads as BLOCKED, so the table must hold the fixture's rows and no others.
    while ($table.ListRows.Count -gt $rows.Count) { $table.ListRows.Item($table.ListRows.Count).Delete() | Out-Null }
    if ($table.ListRows.Count -ne $rows.Count) { throw "table holds $($table.ListRows.Count) rows, not $($rows.Count)" }
    for ($i = 0; $i -lt $rows.Count; $i++) {
        $r = $i + 2
        foreach ($name in $inputs) {
            $cell = $reg.Cells.Item($r, $headers[$name])
            $value = $rows[$i].$name
            if ($value -eq '' -or $null -eq $value) { $cell.ClearContents() | Out-Null }
            elseif ($dates -contains $name) { $cell.Value2 = [double](& $serial $value) }
            elseif ($amounts -contains $name) { $cell.Value2 = [double]$value }
            else { $cell.Value2 = [string]$value }
        }
    }
    if ($Events) {
        $evs = @(Import-Csv -LiteralPath $Events)
        $sheet = $book.Worksheets.Item('Events')
        $evTable = $sheet.ListObjects.Item('tblEvents')
        while ($evTable.ListRows.Count -lt $evs.Count) { $evTable.ListRows.Add() | Out-Null }
        if ($evTable.ListRows.Count -ne $evs.Count) { throw "tblEvents holds $($evTable.ListRows.Count) rows, not $($evs.Count)" }
        $evHeaders = @{}
        for ($c = 1; $c -le $evTable.ListColumns.Count; $c++) { $evHeaders[[string]$sheet.Cells.Item(1, $c).Value2] = $c }
        $evDates = 'payment_date', 'event_date', 'receipt_date', 'notified_date'
        for ($i = 0; $i -lt $evs.Count; $i++) {
            foreach ($name in 'employee_id', 'payment_date', 'event', 'event_date', 'amount', 'receipt_date', 'notified_date') {
                $cell = $sheet.Cells.Item($i + 2, $evHeaders[$name])
                $value = $evs[$i].$name
                if ($value -eq '' -or $null -eq $value) { $cell.ClearContents() | Out-Null }
                elseif ($evDates -contains $name) { $cell.Value2 = [double](& $serial $value) }
                elseif ($name -eq 'amount') { $cell.Value2 = [double]$value }
                else { $cell.Value2 = [string]$value }
            }
        }
    }
    $summary.Range('B2').Value2 = [double](& $serial $AsAt)
    $excel.CalculateFull()
    $out_rows = for ($i = 0; $i -lt $rows.Count; $i++) {
        $r = $i + 2
        [pscustomobject]@{
            employee_id = $reg.Cells.Item($r, $headers['employee_id']).Text
            payment_date = $reg.Cells.Item($r, $headers['payment_date']).Text
            verdict = $reg.Cells.Item($r, $headers['Verdict']).Value2
            between = $reg.Cells.Item($r, $headers['Unassessable_between']).Value2
            exposed = $reg.Cells.Item($r, $headers['Exposed']).Value2
            shortfall = $reg.Cells.Item($r, $headers['Final_shortfall']).Text
            due = $reg.Cells.Item($r, $headers['Final_due']).Text
            problem = $reg.Cells.Item($r, $headers['Row_problem']).Value2
        }
    }
    $checks = $book.Worksheets.Item('Review Checks')
    $overall = $checks.Range('B16').Value2
    $undecided = $checks.Range('C6').Value2
    $out_rows | Export-Csv -LiteralPath $Out -NoTypeInformation
    "overall=$overall undecided=$undecided"
    if ($Events) {
        $sheet = $book.Worksheets.Item('Events')
        for ($i = 0; $i -lt $evs.Count; $i++) {
            "event row $($i + 2): problem=$($sheet.Cells.Item($i + 2, $evHeaders['Ev_problem']).Value2) lines=$($sheet.Cells.Item($i + 2, $evHeaders['Ev_lines']).Value2)"
        }
        "event_check=$($checks.Range('B15').Value2) event_problems=$($checks.Range('C15').Value2)"
    }
    $book.Close($false)
}
finally {
    $excel.Quit()
    [Runtime.InteropServices.Marshal]::ReleaseComObject($excel) | Out-Null
    Remove-Item -LiteralPath $copy -ErrorAction SilentlyContinue
}
