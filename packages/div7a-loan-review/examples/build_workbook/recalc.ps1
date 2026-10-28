# The desktop Excel pass: open the built workbook at $env:DIV7A_WORKBOOK once,
# record the Excel build on Sources & Version, recalculate, save and print
# overall=<Review Checks B10>. Run by build_workbook through -EncodedCommand.
# Any failure, including in cleanup, exits non-zero; the first error wins.
$ErrorActionPreference = 'Stop'
$t = 0; while ((Get-Process EXCEL -ErrorAction SilentlyContinue) -and $t -lt 60) { Start-Sleep 1; $t++ }
if (Get-Process EXCEL -ErrorAction SilentlyContinue) {
  throw 'Excel is already running; refusing to share its COM server.'
}
Add-Type -Namespace Div7aBuild -Name Native -MemberDefinition '[DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(System.IntPtr hWnd, out uint processId);'
# Excel rejects calls while it is still opening or calculating (RPC_E_CALL_REJECTED,
# 0x80010001) and can hand back a null workbook from Open on a cold start. Each
# step asks again rather than failing the build on a busy signal.
function Invoke-Com([scriptblock]$Action) {
  for ($attempt = 0; $attempt -lt 40; $attempt++) {
    try { return (& $Action) }
    catch [System.Runtime.InteropServices.COMException] {
      if ($_.Exception.HResult -ne -2147418111) { throw }
      Start-Sleep -Milliseconds 500
    }
  }
  throw 'Excel kept rejecting the call'
}
# Wait up to 10 seconds for an owned Excel to exit, otherwise stop it and
# confirm it exited, then close the handle. Returns the first error, or $null;
# never throws, so every step runs.
function Stop-OwnedExcel($Process) {
  $err = $null
  $exited = $false
  try { $exited = $Process.WaitForExit(10000) } catch { $err = $_ }
  if (-not $exited) {
    try { $Process.Kill() }
    catch {
      # Excel can exit by itself between the wait and Kill, which then throws
      # InvalidOperationException; that is a clean stop, not a failure.
      $killError = $_
      $gone = $false
      try { $gone = $Process.HasExited } catch {}
      $raced = $gone -and $killError.Exception.GetBaseException() -is [System.InvalidOperationException]
      if (-not $raced -and $null -eq $err) { $err = $killError }
    }
    try { if (-not $Process.WaitForExit(10000)) { throw 'Excel was stopped but did not exit within 10 seconds.' } }
    catch { if ($null -eq $err) { $err = $_ } }
  }
  try { $Process.Dispose() } catch { if ($null -eq $err) { $err = $_ } }
  $err
}
$launched = Get-Date
$xl = New-Object -ComObject Excel.Application
$excel = $null; $wb = $null; $sources = $null; $status = $null; $failure = $null
try {
  # Excel is a single-use COM server, so New-Object starts a new instance;
  # ownership is still confirmed before this build quits or stops anything.
  # The process cannot exit while this live COM reference is held, so its id
  # is stable until the handle is open; the open handle then keeps the id from
  # being reused. A process started before this launch is not ours.
  [uint32]$excelId = 0
  $hwnd = Invoke-Com { $xl.Hwnd }
  [void][Div7aBuild.Native]::GetWindowThreadProcessId([System.IntPtr]$hwnd, [ref]$excelId)
  if ($excelId -eq 0) { throw 'Could not find the Excel process behind the COM server.' }
  $candidate = Get-Process -Id $excelId
  [void]$candidate.Handle
  if ($candidate.ProcessName -ne 'EXCEL' -or $candidate.StartTime -lt $launched) {
    throw "Process $excelId is not an Excel started by this build; refusing to own it."
  }
  $excel = $candidate
  $xl.Visible = $false; $xl.DisplayAlerts = $false; $xl.AutomationSecurity = 3
  for ($attempt = 0; $attempt -lt 5 -and $null -eq $wb; $attempt++) {
    $wb = Invoke-Com { $xl.Workbooks.Open($env:DIV7A_WORKBOOK) }
    if ($null -eq $wb) { Start-Sleep -Seconds 2 }
  }
  if ($null -eq $wb) { throw 'Excel returned no workbook from Workbooks.Open after 5 attempts' }
  $sources = Invoke-Com { $wb.Worksheets.Item('Sources & Version') }
  Invoke-Com { $sources.Range('B3').Value2 = 'Excel ' + $xl.Version + ' build ' + $xl.Build } | Out-Null
  Invoke-Com { $xl.CalculateFullRebuild() } | Out-Null
  $tries = 0
  while ((Invoke-Com { $xl.CalculationState }) -ne 0 -and $tries -lt 600) { Start-Sleep -Milliseconds 100; $tries++ }
  if ((Invoke-Com { $xl.CalculationState }) -ne 0) { throw 'Excel did not finish calculating within 60 seconds.' }
  $status = Invoke-Com { $wb.Worksheets.Item('Review Checks').Range('B10').Text }
  if (@('PASS', 'REVIEW', 'BLOCKED') -cnotcontains $status) { throw "Unexpected overall status '$status'." }
  Invoke-Com { $wb.Save() } | Out-Null
  Invoke-Com { $wb.Close($false) } | Out-Null
} catch { $failure = $_ }
# Every cleanup step runs whatever failed before it, and the first error wins.
# Only a confirmed own Excel is quit or stopped; with ownership unconfirmed the
# COM objects are released and the build has already failed.
if ($null -ne $excel) { try { $xl.Quit() } catch { if ($null -eq $failure) { $failure = $_ } } }
foreach ($com in @($sources, $wb, $xl)) {
  if ($null -ne $com) {
    try { [void][System.Runtime.InteropServices.Marshal]::FinalReleaseComObject($com) }
    catch { if ($null -eq $failure) { $failure = $_ } }
  }
}
# Queue finalisers for the wrappers PowerShell made along the way; there is no
# wait for them, so the stop below cannot hang behind one.
[GC]::Collect()
if ($null -ne $excel) {
  $stopError = Stop-OwnedExcel $excel
  if ($null -eq $failure) { $failure = $stopError }
}
if ($null -ne $failure) { throw $failure }
Write-Output ('overall=' + $status)
