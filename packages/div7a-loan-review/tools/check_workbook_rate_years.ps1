param([string]$Workbook, [string]$OutputPath)
$ErrorActionPreference = 'Stop'
if (Get-Process EXCEL -ErrorAction SilentlyContinue) { throw 'Excel is already running.' }
$before = (Get-FileHash -LiteralPath $Workbook).Hash
$excel = New-Object -ComObject Excel.Application
$excel.Visible = $false
$excel.DisplayAlerts = $false
$excel.AutomationSecurity = 3
$results = @()
try {
    foreach ($case in @('unique years', 'duplicate years', 'duplicate years with different rates')) {
        $book = $excel.Workbooks.Open($Workbook, 0, $true)
        try {
            $rates = $book.Worksheets.Item('Rates')
            if ($case -ne 'unique years') { $rates.Range('A3').Value2 = $rates.Range('A2').Value2 }
            if ($case -eq 'duplicate years with different rates') { $rates.Range('B3').Value2 = 0.5 }
            $excel.CalculateFullRebuild()
            $check = $book.Worksheets.Item('Review Checks')
            $expected = if ($case -eq 'unique years') { 'PASS' } else { 'BLOCKED' }
            $results += [ordered]@{case=$case; expected=$expected; actual=$check.Range('B4').Text; overall=$check.Range('B10').Text}
        } finally { $book.Close($false) }
    }
    [ordered]@{excel_version=$excel.Version; excel_build=$excel.Build; source_sha256=$before; cases=$results} |
        ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $OutputPath -Encoding utf8
    $results | ConvertTo-Json -Compress
} finally {
    $excel.Quit()
    [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($excel)
    $book=$null; $rates=$null; $check=$null; $excel=$null
    [GC]::Collect(); [GC]::WaitForPendingFinalizers()
}
if ((Get-FileHash -LiteralPath $Workbook).Hash -ne $before) { throw 'Source workbook changed.' }

if (@($results | Where-Object { $_.actual -ne $_.expected }).Count) { throw 'Rate-year regression failed.' }
