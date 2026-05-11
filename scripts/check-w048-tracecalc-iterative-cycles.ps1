param(
    [string]$RunRoot = "docs/test-runs/core-engine/tracecalc-reference-machine/w048-tracecalc-cycles-004"
)

$ErrorActionPreference = "Stop"
function Fail([string]$Message) { Write-Error $Message; exit 1 }
function Read-Json([string]$Path) { if (-not (Test-Path -LiteralPath $Path)) { Fail "missing $Path" }; Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json }

$summary = Read-Json (Join-Path $RunRoot "run_summary.json")
if ($summary.run_id -ne "w048-tracecalc-cycles-004") { Fail "unexpected run_id $($summary.run_id)" }
if ([int]$summary.scenario_count -lt 38) { Fail "scenario_count too small: $($summary.scenario_count)" }
if ([int]$summary.result_counts.passed -ne [int]$summary.scenario_count) { Fail "not all scenarios passed" }

$expected = @{
    "tc_w048_excel_iter_two_node_order_001" = @{ cells = @{ A1 = "11"; B1 = "22" }; labels = @("cycle_iteration_trace", "excel_match_iteration_trace_emitted", "excel_match_iterative_candidate_emitted", "candidate_published") };
    "tc_w048_excel_iter_three_node_order_001" = @{ cells = @{ A1 = "102"; B1 = "101"; C1 = "103" }; labels = @("cycle_iteration_trace", "excel_match_iteration_trace_emitted", "excel_match_iterative_candidate_emitted", "candidate_published") };
    "tc_w048_excel_iter_fraction_precision_001" = @{ cells = @{ A1 = "0.33333333333333331" }; labels = @("cycle_iteration_trace", "excel_match_iteration_trace_emitted", "excel_match_iterative_candidate_emitted", "candidate_published") };
    "tc_w048_excel_ctro_indirect_iterative_self_001" = @{ cells = @{ A1 = "1"; B1 = "A1" }; labels = @("cycle_iteration_trace", "excel_match_iteration_trace_emitted", "ctro_iterative_self_cycle_observed", "excel_match_iterative_candidate_emitted", "candidate_published") }
}

foreach ($scenarioId in $expected.Keys) {
    $result = Read-Json (Join-Path $RunRoot "scenarios/$scenarioId/result.json")
    if ($result.result_state -ne "passed") { Fail "$scenarioId did not pass" }
    $scenarioRoot = Join-Path $RunRoot "scenarios/$scenarioId"
    $published = Read-Json (Join-Path $scenarioRoot "published_view.json")
    $trace = Read-Json (Join-Path $scenarioRoot "trace.json")
    $counters = Read-Json (Join-Path $scenarioRoot "counters.json")
    $values = @{}
    foreach ($entry in $published.node_values) { $values[$entry.node_id] = [string]$entry.value }
    foreach ($cell in $expected[$scenarioId].cells.Keys) {
        $actual = $values[$cell]
        $want = [string]$expected[$scenarioId].cells[$cell]
        if ($actual -ne $want) { Fail "$scenarioId $cell expected $want observed $actual" }
    }
    $labels = @($trace.events | ForEach-Object { $_.label })
    foreach ($label in $expected[$scenarioId].labels) {
        if ($labels -notcontains $label) { Fail "$scenarioId missing trace label $label" }
    }
    $counterMap = @{}
    foreach ($entry in $counters.counters) { $counterMap[$entry.counter] = [int]$entry.value }
    if ($counterMap["cycle.iteration_trace_events"] -ne 1) { Fail "$scenarioId missing exactly one iteration trace counter" }
    if ($counterMap["publications_committed"] -ne 1) { Fail "$scenarioId missing publication counter" }
}

Write-Output "w048 tracecalc iterative cycles ok: run=$($summary.run_id) scenarios=$($expected.Count) total=$($summary.scenario_count)"
