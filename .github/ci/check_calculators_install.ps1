# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

# Runs the calculators README's Windows install command as a reader would,
# with a fresh LOCALAPPDATA and no GitHub sign-in: it must install an
# executable that answers --version with the tag's version. Then runs it twice
# more with Invoke-WebRequest altering one download, the archive or its
# SHA256SUMS digest, as a tampered mirror would: each must refuse and install
# nothing. The MCP README must give the same command.
#
# usage: pwsh -File .github/ci/check_calculators_install.ps1
$ErrorActionPreference = 'Stop'

$readme = 'packages/australian-tax-calculators/README.md'
# The reviewed installer commit (release-policy v0.5.0); a README that names
# another fails here, before anything runs. Change it with the README pins.
$pin = 'd7039efe6d8c441f1f213c674902909e80535599'
$pattern = '\A& \(\[scriptblock\]::Create\(\(irm https://raw\.githubusercontent\.com/ryanduguid/release-policy/' + $pin + '/scripts/install\.ps1\)\)\) ryanduguid/australian-accounting australian-tax-calculators/v(?<version>[0-9]+\.[0-9]+\.[0-9]+) -ModifyPath\z'
$found = @(Get-Content -LiteralPath $readme | Where-Object { $_ -cmatch $pattern })
if ($found.Count -ne 1) {
    throw "$readme gives $($found.Count) Windows install commands pinned to release-policy $pin, not 1"
}
$command = $found[0]
if (@(Get-Content -LiteralPath 'apps/aus-accounting-mcp/README.md' | Where-Object { $_ -ceq $command }).Count -ne 1) {
    throw "apps/aus-accounting-mcp/README.md does not give $readme's Windows install command"
}
$version = ([regex]::Match($command, $pattern)).Groups['version'].Value

$work = Join-Path ([IO.Path]::GetTempPath()) ([IO.Path]::GetRandomFileName())
New-Item -ItemType Directory -Path $work | Out-Null
Remove-Item Env:GH_TOKEN, Env:GITHUB_TOKEN -ErrorAction SilentlyContinue

# Runs the command with its own LOCALAPPDATA and gh configuration.
function Invoke-Install([string]$Name) {
    $env:LOCALAPPDATA = Join-Path $work $Name
    $env:GH_CONFIG_DIR = Join-Path $env:LOCALAPPDATA 'gh'
    New-Item -ItemType Directory -Force -Path $env:GH_CONFIG_DIR | Out-Null
    & ([scriptblock]::Create($command))
}

Invoke-Install 'clean'
$installed = Join-Path $work 'clean\Programs\australian-tax-calculators\australian-tax-calculators.exe'
$reported = & $installed --version
if ($reported -cne "australian-tax-calculators $version") {
    throw "the installed executable reports '$reported', not version $version"
}

foreach ($altered in 'archive', 'sums') {
    # Found before the cmdlet, so the installer's Invoke-WebRequest runs this:
    # the real download, then one change. It reads $altered from this loop,
    # its caller's caller, as PowerShell scopes follow the call.
    function global:Invoke-WebRequest {
        param([switch]$UseBasicParsing, [string]$Uri, [string]$OutFile)
        Microsoft.PowerShell.Utility\Invoke-WebRequest -UseBasicParsing:$UseBasicParsing -Uri $Uri -OutFile $OutFile
        if ($altered -eq 'archive' -and $OutFile.EndsWith('.zip')) {
            $bytes = [IO.File]::ReadAllBytes($OutFile)
            $bytes[100] = $bytes[100] -bxor 0xFF
            [IO.File]::WriteAllBytes($OutFile, $bytes)
        }
        if ($altered -eq 'sums' -and $OutFile.EndsWith('SHA256SUMS')) {
            $lines = Get-Content -LiteralPath $OutFile | ForEach-Object {
                if ($_.EndsWith('windows-msvc.zip')) {
                    $first = if ($_[0] -eq '0') { '1' } else { '0' }
                    $first + $_.Substring(1)
                } else { $_ }
            }
            [IO.File]::WriteAllText($OutFile, (($lines -join "`n") + "`n"))
        }
    }
    $refused = $null
    try {
        Invoke-Install $altered
    } catch {
        $refused = $_.Exception.Message
    }
    Remove-Item Function:\Invoke-WebRequest
    if (-not $refused) {
        throw "installed with an altered $altered"
    }
    Write-Output $refused
    if (-not $refused.Contains('failed provenance verification')) {
        throw "an altered $altered was refused for another reason"
    }
    if (Test-Path -LiteralPath (Join-Path $work "$altered\Programs\australian-tax-calculators")) {
        throw "an altered $altered left an install behind"
    }
}
Remove-Item -LiteralPath $work -Recurse -Force
Write-Output "The install command installs $version and refuses an altered archive or SHA256SUMS."
# The refused runs leave gh's failure in $LASTEXITCODE, which pwsh would return.
exit 0
