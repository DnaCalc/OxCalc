param(
    [string]$RunRoot = "docs/test-runs/excel-cycles/w048-excel-version-inventory-001"
)
$ErrorActionPreference = "Stop"
function Fail([string]$Message) { Write-Error $Message; exit 1 }
function Read-Json([string]$Path) { if (-not (Test-Path -LiteralPath $Path)) { Fail "missing $Path" }; Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json }
$s = Read-Json (Join-Path $RunRoot "inventory.json")
if ($s.schema_version -ne "oxcalc.w048.excel_version_inventory.v1") { Fail "unexpected schema_version" }
if (-not $s.com_excel.available) { Fail "COM Excel unavailable; cannot verify local host inventory" }
if (-not $s.com_excel.version) { Fail "missing COM Excel version" }
if (-not $s.com_excel.build) { Fail "missing COM Excel build" }
if (@($s.distinct_product_versions).Count -lt 1) { Fail "expected at least one observed Excel product version" }
if (-not (($s.blocker_disposition -as [string]) -match "BLK-W048-EXCEL-VERSION")) { Fail "missing version blocker disposition" }
if (($s | ConvertTo-Json -Depth 60) -match "@") { Fail "inventory appears to contain identity data" }
Write-Output "w048 excel version inventory ok: run=$($s.run_id) com=$($s.com_excel.version)/$($s.com_excel.build) product_versions=$(@($s.distinct_product_versions).Count) second_host_available=$($s.second_host_available_in_local_inventory)"
