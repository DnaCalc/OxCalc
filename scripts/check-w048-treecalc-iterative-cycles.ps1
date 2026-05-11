param(
    [string]$RunRoot = "docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-002"
)

$ErrorActionPreference = "Stop"
function Fail([string]$Message) { Write-Error $Message; exit 1 }
function Read-Json([string]$Path) { if (-not (Test-Path -LiteralPath $Path)) { Fail "missing $Path" }; Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json }

$summary = Read-Json (Join-Path $RunRoot "run_summary.json")
if ($summary.run_id -ne "w048-treecalc-cycles-002") { Fail "unexpected run_id $($summary.run_id)" }
if ([int]$summary.case_count -lt 37) { Fail "case_count too small: $($summary.case_count)" }
if ([int]$summary.expectation_mismatch_count -ne 0) { Fail "expectation mismatches present" }

$expected = @{
    "tc_w048_excel_iter_two_node_order_001" = @{ cells = @{ "2" = "11"; "3" = "22" }; order = @(3,2) };
    "tc_w048_excel_iter_three_node_order_001" = @{ cells = @{ "2" = "102"; "3" = "101"; "4" = "103" }; order = @(4,3,2) };
    "tc_w048_excel_iter_fraction_precision_001" = @{ cells = @{ "2" = "0.33333333333333331" }; order = @(2) };
    "tc_w048_excel_ctro_indirect_iterative_self_001" = @{ cells = @{ "2" = "1"; "3" = "A1" }; order = @(2) }
}

foreach ($caseId in $expected.Keys) {
    $caseRoot = Join-Path (Join-Path $RunRoot "cases") $caseId
    $result = Read-Json (Join-Path $caseRoot "result.json")
    if ($result.result_state -ne "published") { Fail "$caseId expected published observed $($result.result_state)" }
    if ($result.expectation_mismatches.Count -gt 0) { Fail "$caseId has expectation mismatches" }
    $published = Read-Json (Join-Path $caseRoot "published_values.json")
    $values = @{}
    foreach ($entry in $published) { $values[[string]$entry.node_id] = [string]$entry.value }
    foreach ($nodeId in $expected[$caseId].cells.Keys) {
        $actual = $values[$nodeId]
        $want = [string]$expected[$caseId].cells[$nodeId]
        if ($actual -ne $want) { Fail "$caseId node $nodeId expected $want observed $actual" }
    }
    $diagnostics = @($result.diagnostics)
    if (-not ($diagnostics | Where-Object { $_ -match "cycle.excel_match_iterative" })) { Fail "$caseId missing excel-match diagnostic" }
    if (-not ($diagnostics | Where-Object { $_ -match "cycle_iteration_trace" })) { Fail "$caseId missing iteration trace diagnostic" }
}

Write-Output "w048 treecalc iterative cycles ok: run=$($summary.run_id) cases=$($expected.Count) total=$($summary.case_count)"
