param(
    [string]$RunRoot = "docs/test-runs/excel-cycles/w048-excel-multithread-variant-001"
)
$ErrorActionPreference = "Stop"
function Fail([string]$Message) { Write-Error $Message; exit 1 }
function Read-Json([string]$Path) { if (-not (Test-Path -LiteralPath $Path)) { Fail "missing $Path" }; Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json }
$s = Read-Json (Join-Path $RunRoot "observation.json")
if ($s.schema_version -ne "oxcalc.w048.excel_multithread_variant_probe.v1") { Fail "unexpected schema_version" }
if ([int]$s.observation_count -ne 4) { Fail "expected 4 multithread variant probes" }
if ($s.environment.multi_threaded_after -ne $true) { Fail "multithreaded calculation was not enabled" }
$expected = @{
    "excel_iter_two_node_order_001" = @{ "Sheet1!A1" = "3"; "Sheet1!B1" = "6" };
    "excel_iter_three_node_order_001" = @{ "Sheet1!A1" = "5"; "Sheet1!B1" = "107"; "Sheet1!C1" = "6" };
    "excel_iter_fraction_precision_001" = @{ "Sheet1!A1" = "0.49999989546242096" };
    "excel_ctro_indirect_iterative_self_001" = @{ "Sheet1!A1" = "10"; "Sheet1!B1" = "A1" }
}
foreach ($probeId in $expected.Keys) {
    $obs = @($s.observations | Where-Object { $_.probe_id -eq $probeId })
    if ($obs.Count -ne 1) { Fail "missing or duplicate probe $probeId" }
    $values = @{}
    foreach ($cell in $obs[0].cell_results) { $values[$cell.cell] = [string]$cell.value2 }
    foreach ($cellName in $expected[$probeId].Keys) {
        $actual = $values[$cellName]
        $want = [string]$expected[$probeId][$cellName]
        if ($actual -ne $want) { Fail "$probeId $cellName expected $want observed $actual" }
    }
}
Write-Output "w048 excel multithread variant packet ok: run=$($s.run_id) observations=$($s.observation_count) threads=$($s.environment.thread_count)"
