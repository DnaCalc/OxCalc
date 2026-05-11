param(
    [Parameter(Mandatory=$true)]
    [string]$RunRoot,
    [int]$MinimumProbeCount = 12
)

$ErrorActionPreference = "Stop"

function Fail([string]$Message) {
    Write-Error $Message
    exit 1
}

function Read-JsonFile([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path)) { Fail "missing required file: $Path" }
    return Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json
}

$requiredTop = @("environment.json", "probe_plan.json", "observation.json", "normalization_notes.md", "raw_com_log.jsonl")
foreach ($name in $requiredTop) {
    $path = Join-Path $RunRoot $name
    if (-not (Test-Path -LiteralPath $path)) { Fail "missing required packet file: $path" }
}

$env = Read-JsonFile (Join-Path $RunRoot "environment.json")
$plan = Read-JsonFile (Join-Path $RunRoot "probe_plan.json")
$aggregate = Read-JsonFile (Join-Path $RunRoot "observation.json")

if ($aggregate.status -ne "observed") { Fail "aggregate observation status is not observed" }
if ([int]$aggregate.observation_count -lt $MinimumProbeCount) { Fail "observation_count $($aggregate.observation_count) below minimum $MinimumProbeCount" }
if ([int]$plan.probe_count -ne [int]$aggregate.observation_count) { Fail "probe_plan count and observation count differ" }
if (-not $env.probe_runner) { Fail "environment lacks probe_runner" }
if (-not $env.excel_version) { Fail "environment lacks excel_version" }

$mandatory = @(
    "excel_struct_self_001",
    "excel_struct_self_prior_001",
    "excel_struct_two_node_001",
    "excel_struct_three_node_001",
    "excel_struct_guarded_activation_001",
    "excel_iter_self_increment_001",
    "excel_iter_two_node_order_001",
    "excel_chain_edit_order_ab_001",
    "excel_chain_edit_order_ba_001",
    "excel_ctro_indirect_self_001",
    "excel_ctro_indirect_release_001",
    "excel_ctro_downstream_001"
)

if ($plan.probe_set -eq "bitexact") {
    $mandatory += @(
        "excel_iter_self_decay_001",
        "excel_iter_three_node_order_001",
        "excel_iter_oscillation_001",
        "excel_iter_non_numeric_prior_001",
        "excel_iter_fraction_precision_001",
        "excel_chain_full_rebuild_compare_001",
        "excel_ctro_indirect_iterative_self_001"
    )
}

$byId = @{}
foreach ($obs in $aggregate.observations) {
    if (-not $obs.probe_id) { Fail "observation without probe_id" }
    if ($byId.ContainsKey($obs.probe_id)) { Fail "duplicate observation for $($obs.probe_id)" }
    $byId[$obs.probe_id] = $obs

    $probeRoot = Join-Path (Join-Path $RunRoot "probes") $obs.probe_id
    foreach ($probeFile in @("workbook_before.xlsx", "workbook_after.xlsx", "cell_snapshots.json", "calc_chain_snapshot.json", "observation.json")) {
        $path = Join-Path $probeRoot $probeFile
        if (-not (Test-Path -LiteralPath $path)) { Fail "missing probe artifact: $path" }
    }
    if (-not $obs.cell_results -or $obs.cell_results.Count -lt 1) { Fail "observation $($obs.probe_id) lacks cell_results" }
    if (-not $obs.operation_steps -or $obs.operation_steps.Count -lt 1) { Fail "observation $($obs.probe_id) lacks operation_steps" }
    if (-not $obs.iteration_profile) { Fail "observation $($obs.probe_id) lacks iteration_profile" }
}

foreach ($id in $mandatory) {
    if (-not $byId.ContainsKey($id)) { Fail "mandatory probe missing: $id" }
}

$iterCount = @($aggregate.observations | Where-Object { $_.iteration_profile.enabled -eq $true }).Count
$dynamicCount = @($aggregate.observations | Where-Object { $_.cycle_kind -eq "dynamic_reference" }).Count
$chainBearingCount = @($aggregate.observations | Where-Object { $_.calc_chain.present -eq $true -and $_.calc_chain.entries.Count -gt 0 }).Count

if ($iterCount -lt 2) { Fail "insufficient iterative probes: $iterCount" }
if ($dynamicCount -lt 2) { Fail "insufficient dynamic-reference probes: $dynamicCount" }
if ($chainBearingCount -lt 1) { Fail "no saved calc-chain evidence found" }

Write-Output "w048 excel observation packet ok: run=$($aggregate.run_id) probes=$($aggregate.observation_count) iterative=$iterCount dynamic=$dynamicCount chain_bearing=$chainBearingCount"
