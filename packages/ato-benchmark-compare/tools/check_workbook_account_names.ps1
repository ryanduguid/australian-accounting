param([string]$Workbook, [string]$OutputPath, [switch]$RecordBaseline)
$ErrorActionPreference = 'Stop'
if (Get-Process EXCEL -ErrorAction SilentlyContinue) { throw 'Excel is already running.' }
$hash = (Get-FileHash -LiteralPath $Workbook).Hash
$excel = New-Object -ComObject Excel.Application
$excel.Visible = $false
$excel.DisplayAlerts = $false
$excel.AutomationSecurity = 3
$results = @()
try {
    foreach ($names in @(@('Sales*', 'Sales-domestic'), @('Sales?', 'SalesA'), @('Sales~*', 'Sales*'), @('Sales*', ' SALES* '), @('Sales?', 'Sales?'), @('Sales~', 'Sales~'))) {
        $book = $excel.Workbooks.Open($Workbook, 0, $true)
        try {
            foreach ($pair in @(@('P&L Import', 'tblPnl'), @('Mapping', 'tblMapping'))) {
                $sheet = $book.Worksheets.Item($pair[0])
                $column = $sheet.ListObjects.Item($pair[1]).ListColumns.Item('Account').DataBodyRange
                $column.ClearContents()
                $column.Cells.Item(1, 1).Value2 = $names[0]
                $column.Cells.Item(2, 1).Value2 = $names[1]
            }
            $excel.CalculateFullRebuild()
            $review = $book.Worksheets.Item('Review Checks')
            $expected = if ($names[0].Trim().ToLowerInvariant() -eq $names[1].Trim().ToLowerInvariant()) { 2 } else { 0 }
            $results += [ordered]@{names=$names; expected=$expected; pnl=$review.Range('C5').Value2; mapping=$review.Range('C6').Value2}
        } finally { $book.Close($false) }
    }
    [ordered]@{excel_version=$excel.Version; excel_build=$excel.Build; source_sha256=$hash; cases=$results} |
        ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $OutputPath -Encoding utf8
} finally {
    $excel.Quit()
    [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($excel)
    $book=$null; $sheet=$null; $column=$null; $review=$null; $excel=$null
    [GC]::Collect(); [GC]::WaitForPendingFinalizers()
}
if ((Get-FileHash -LiteralPath $Workbook).Hash -ne $hash) { throw 'Source workbook changed.' }
if (-not $RecordBaseline -and @($results | Where-Object { $_.pnl -ne $_.expected -or $_.mapping -ne $_.expected }).Count) {
    throw 'An account-name control failed.'
}
$results | ConvertTo-Json -Compress
