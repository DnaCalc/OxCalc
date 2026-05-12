param(
    [string]$RunId = "w048-excel-version-inventory-001",
    [string]$OutputRoot = "docs/test-runs/excel-cycles"
)

$ErrorActionPreference = "Stop"

function New-Dir([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path)) { New-Item -ItemType Directory -Force -Path $Path | Out-Null }
}
function Write-Json($Value, [string]$Path, [int]$Depth = 40) {
    $parent = Split-Path -Parent $Path
    New-Dir $parent
    $encoding = New-Object System.Text.UTF8Encoding($false)
    [System.IO.File]::WriteAllText((Resolve-Path -LiteralPath $parent).Path + "\" + (Split-Path -Leaf $Path), ($Value | ConvertTo-Json -Depth $Depth), $encoding)
}
function Read-RegSelected([string]$Path, [string[]]$Names) {
    try {
        if (-not (Test-Path -LiteralPath $Path)) { return $null }
        $item = Get-ItemProperty -LiteralPath $Path
        $props = [ordered]@{}
        foreach ($name in $Names) {
            $prop = $item.PSObject.Properties[$name]
            if ($null -ne $prop) { $props[$name] = $prop.Value }
        }
        return [ordered]@{ path = $Path; properties = $props }
    } catch {
        return [ordered]@{ path = $Path; error = $_.Exception.Message }
    }
}
function Get-FileVersionInfoSafe([string]$Path) {
    try {
        if (-not (Test-Path -LiteralPath $Path)) { return $null }
        $v = [System.Diagnostics.FileVersionInfo]::GetVersionInfo($Path)
        return [ordered]@{
            path = $Path
            file_version = $v.FileVersion
            product_version = $v.ProductVersion
            product_name = $v.ProductName
            company_name = $v.CompanyName
        }
    } catch {
        return [ordered]@{ path = $Path; error = $_.Exception.Message }
    }
}

$runRoot = Join-Path $OutputRoot $RunId
New-Dir $runRoot

$excel = $null
$com = [ordered]@{ available = $false }
try {
    $excel = New-Object -ComObject Excel.Application
    $excel.Visible = $false
    $build = $null
    $path = $null
    try { $build = [string]$excel.Build } catch {}
    try { $path = [string]$excel.Path } catch {}
    $exePath = if ($path) { Join-Path $path "EXCEL.EXE" } else { $null }
    $com = [ordered]@{
        available = $true
        version = [string]$excel.Version
        build = $build
        path = $path
        exe_path = $exePath
        exe_file_version = if ($exePath) { Get-FileVersionInfoSafe $exePath } else { $null }
    }
} catch {
    $com = [ordered]@{ available = $false; error = $_.Exception.Message }
} finally {
    if ($null -ne $excel) {
        try { $excel.Quit() } catch {}
        [System.Runtime.InteropServices.Marshal]::ReleaseComObject($excel) | Out-Null
    }
    [GC]::Collect()
    [GC]::WaitForPendingFinalizers()
}

$registry = @()
$registry += Read-RegSelected "Registry::HKEY_CLASSES_ROOT\Excel.Application\CurVer" @("(default)")
$registry += Read-RegSelected "Registry::HKEY_CLASSES_ROOT\Excel.Application" @("(default)")
$registry += Read-RegSelected "HKLM:\SOFTWARE\Microsoft\Office\ClickToRun\Configuration" @(
    "ClientVersionToReport",
    "VersionToReport",
    "ClientXnoneVersion",
    "Platform",
    "InstallationPath",
    "ProductReleaseIds",
    "AudienceData",
    "UpdateChannel",
    "UpdatesEnabled"
)
$registry += Read-RegSelected "HKLM:\SOFTWARE\WOW6432Node\Microsoft\Office\ClickToRun\Configuration" @(
    "ClientVersionToReport",
    "VersionToReport",
    "ClientXnoneVersion",
    "Platform",
    "InstallationPath",
    "ProductReleaseIds",
    "AudienceData",
    "UpdateChannel",
    "UpdatesEnabled"
)
$registry += Read-RegSelected "HKLM:\SOFTWARE\Microsoft\Office\16.0\Excel\InstallRoot" @("Path")
$registry += Read-RegSelected "HKLM:\SOFTWARE\WOW6432Node\Microsoft\Office\16.0\Excel\InstallRoot" @("Path")
$registry = @($registry | Where-Object { $null -ne $_ })

$roots = @()
foreach ($envName in @("ProgramFiles", "ProgramFiles(x86)")) {
    $value = [Environment]::GetEnvironmentVariable($envName)
    if ($value) { $roots += (Join-Path $value "Microsoft Office") }
}
$excelExeCandidates = @()
foreach ($root in ($roots | Select-Object -Unique)) {
    if (Test-Path -LiteralPath $root) {
        try {
            Get-ChildItem -LiteralPath $root -Filter EXCEL.EXE -Recurse -ErrorAction SilentlyContinue | ForEach-Object {
                $excelExeCandidates += (Get-FileVersionInfoSafe $_.FullName)
            }
        } catch {}
    }
}
$excelExeCandidates = @($excelExeCandidates | Where-Object { $_ } | Sort-Object path -Unique)

$distinctProductVersions = New-Object System.Collections.Generic.HashSet[string]
if ($com.available -and $com.exe_file_version.product_version) { [void]$distinctProductVersions.Add([string]$com.exe_file_version.product_version) }
foreach ($c in $excelExeCandidates) {
    if ($c.product_version) { [void]$distinctProductVersions.Add([string]$c.product_version) }
}

$summary = [ordered]@{
    schema_version = "oxcalc.w048.excel_version_inventory.v1"
    run_id = $RunId
    observation_time_utc = (Get-Date).ToUniversalTime().ToString("o")
    privacy_note = "Registry capture is limited to installation/version/channel fields and excludes user identity, tenant, activation, and MRU data."
    com_excel = $com
    registry = $registry
    excel_exe_candidates = $excelExeCandidates
    distinct_product_version_count = $distinctProductVersions.Count
    distinct_product_versions = @($distinctProductVersions)
    second_host_available_in_local_inventory = ($distinctProductVersions.Count -gt 1)
    blocker_disposition = if ($distinctProductVersions.Count -gt 1) { "Local inventory found more than one Excel product version; run W048 second-host probes before closing BLK-W048-EXCEL-VERSION." } else { "BLK-W048-EXCEL-VERSION remains open: local inventory found only one Excel product version; external second host/version packet or explicit user single-host scope acceptance is still required." }
}

Write-Json $summary (Join-Path $runRoot "inventory.json")
Write-Output "Wrote W048 Excel version inventory to $runRoot"
