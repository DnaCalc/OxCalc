param(
    [string]$RunRoot = "docs/test-runs/excel-cycles/w048-excel-nonnumeric-prior-001"
)
$ErrorActionPreference = "Stop"
function Fail([string]$Message) { Write-Error $Message; exit 1 }
function Read-Json([string]$Path) { if (-not (Test-Path -LiteralPath $Path)) { Fail "missing $Path" }; Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json }
$s = Read-Json (Join-Path $RunRoot "observation.json")
if ($s.schema_version -ne "oxcalc.w048.excel_nonnumeric_prior_probe.v1") { Fail "unexpected schema_version" }
if ([int]$s.observation_count -ne 8) { Fail "expected 8 nonnumeric prior probes" }
foreach ($prior in @("blank", "text", "error_na", "error_div0")) {
    foreach ($kind in @("increment", "decay")) {
        $id = "prior_${prior}_${kind}"
        $obs = @($s.observations | Where-Object { $_.probe_id -eq $id })
        if ($obs.Count -ne 1) { Fail "missing or duplicate probe $id" }
        if ($kind -eq "increment") {
            if ([string]$obs[0].assignment_snapshot.value2 -ne "1") { Fail "$id expected assignment Value2 1" }
            if ([string]$obs[0].final_snapshot.value2 -ne "6") { Fail "$id expected final Value2 6 with MaxIterations=5" }
        } else {
            if ([string]$obs[0].assignment_snapshot.value2 -ne "0") { Fail "$id expected assignment Value2 0" }
            if ([string]$obs[0].final_snapshot.value2 -ne "0") { Fail "$id expected final Value2 0" }
        }
    }
}
Write-Output "w048 excel nonnumeric-prior probe packet ok: run=$($s.run_id) observations=$($s.observation_count)"
