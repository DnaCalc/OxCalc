param(
    [string]$RunRoot = "docs/test-runs/excel-cycles/w048-excel-root-report-001"
)
$ErrorActionPreference = "Stop"
function Fail([string]$Message) { Write-Error $Message; exit 1 }
function Read-Json([string]$Path) { if (-not (Test-Path -LiteralPath $Path)) { Fail "missing $Path" }; Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json }
$s = Read-Json (Join-Path $RunRoot "observation.json")
if ($s.schema_version -ne "oxcalc.w048.excel_root_report_probe.v1") { Fail "unexpected schema_version" }
if ([int]$s.observation_count -lt 5) { Fail "expected at least 5 root/report probes" }
foreach ($probeId in @("root_self_no_iteration", "root_self_iteration", "root_two_node_ab", "root_two_node_ba", "root_three_node")) {
    $obs = @($s.observations | Where-Object { $_.probe_id -eq $probeId })
    if ($obs.Count -ne 1) { Fail "missing or duplicate probe $probeId" }
    if (@($obs[0].records).Count -lt 5) { Fail "$probeId has too few records" }
}
if (-not (($s.blocker_disposition -as [string]) -match "BLK-W048-EXCEL-ROOT remains open")) { Fail "blocker disposition missing" }
Write-Output "w048 excel root/report probe packet ok: run=$($s.run_id) observations=$($s.observation_count) status=$($s.status)"
