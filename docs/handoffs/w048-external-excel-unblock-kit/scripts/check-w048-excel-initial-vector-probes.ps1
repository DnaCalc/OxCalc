param(
    [string]$RunRoot = "docs/test-runs/excel-cycles/w048-excel-initial-vector-001"
)
$ErrorActionPreference = "Stop"
function Fail([string]$Message) { Write-Error $Message; exit 1 }
function Read-Json([string]$Path) { if (-not (Test-Path -LiteralPath $Path)) { Fail "missing $Path" }; Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json }
$s = Read-Json (Join-Path $RunRoot "observation.json")
if ($s.schema_version -ne "oxcalc.w048.excel_initial_vector_probe.v1") { Fail "unexpected schema_version" }
if ([int]$s.observation_count -ne 8) { Fail "expected 8 initial-vector probes" }
foreach ($obs in $s.observations) {
    if (-not $obs.final_snapshot) { Fail "$($obs.probe_id) missing final snapshot" }
    $afterFormula = @($obs.records | Where-Object { $_.moment -eq "after_formula_assignment" })[0]
    if (-not $afterFormula) { Fail "$($obs.probe_id) missing after_formula_assignment" }
    if ($obs.formula -eq "=A1+1") {
        if ([string]$afterFormula.snapshot.value2 -ne "1") { Fail "$($obs.probe_id) expected after-formula Value2 1" }
        if ($obs.command -eq "none" -and [string]$obs.final_snapshot.value2 -ne "1") { Fail "$($obs.probe_id) expected final Value2 1" }
        if ($obs.command -ne "none" -and [string]$obs.final_snapshot.value2 -ne "11") { Fail "$($obs.probe_id) expected final Value2 11 after command $($obs.command)" }
    }
    if ($obs.formula -eq "=A1/2") {
        if ([string]$afterFormula.snapshot.value2 -ne "0") { Fail "$($obs.probe_id) expected after-formula Value2 0" }
        if ([string]$obs.final_snapshot.value2 -ne "0") { Fail "$($obs.probe_id) expected final Value2 0" }
    }
}
Write-Output "w048 excel initial-vector probe packet ok: run=$($s.run_id) observations=$($s.observation_count)"
