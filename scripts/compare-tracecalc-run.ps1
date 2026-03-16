param(
    [Parameter(Mandatory = $true)]
    [string]$CandidateRunId,

    [string]$BaselineRunId = "w014-stage1-widening-baseline",

    [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
)

$ErrorActionPreference = "Stop"

function Get-RunRoot([string]$runId) {
    return Join-Path $RepoRoot "docs/test-runs/core-engine/tracecalc-reference-machine/$runId"
}

function Read-JsonFile([string]$path) {
    return Get-Content $path -Raw | ConvertFrom-Json -Depth 100
}

function Normalize-Object($value) {
    if ($null -eq $value) {
        return $null
    }

    if ($value -is [System.Collections.IDictionary]) {
        $ordered = [ordered]@{}
        foreach ($key in ($value.Keys | Sort-Object)) {
            $ordered[$key] = Normalize-Object $value[$key]
        }
        return $ordered
    }

    if ($value -is [System.Collections.IEnumerable] -and -not ($value -is [string])) {
        $items = @()
        foreach ($item in $value) {
            $items += ,(Normalize-Object $item)
        }
        return $items
    }

    if ($value.PSObject -and $value.PSObject.Properties.Count -gt 0) {
        $ordered = [ordered]@{}
        foreach ($property in ($value.PSObject.Properties | Sort-Object Name)) {
            $ordered[$property.Name] = Normalize-Object $property.Value
        }
        return $ordered
    }

    return $value
}

function Compare-Projection([string]$label, $baselineValue, $candidateValue, [ref]$failures) {
    $baselineJson = ConvertTo-Json (Normalize-Object $baselineValue) -Depth 100 -Compress
    $candidateJson = ConvertTo-Json (Normalize-Object $candidateValue) -Depth 100 -Compress
    if ($baselineJson -ne $candidateJson) {
        $failures.Value += "Mismatch: $label"
    }
}

function Get-TraceProjection($trace) {
    return @(
        foreach ($event in $trace.events) {
            [ordered]@{
                step_id = $event.step_id
                label = $event.label
                normalized_event_family = $event.normalized_event_family
            }
        }
    )
}

function Get-EngineDiffIndex($engineDiff) {
    $index = @{}
    foreach ($entry in $engineDiff) {
        $index[$entry.scenario_id] = [ordered]@{
            oracle_result_state = $entry.oracle_result_state
            engine_result_state = $entry.engine_result_state
            mismatches = @(
                foreach ($mismatch in $entry.mismatches) {
                    [ordered]@{
                        kind = $mismatch.kind
                        mismatch_kind = $mismatch.mismatch_kind
                        severity_class = $mismatch.severity_class
                        required_equality_surface = $mismatch.required_equality_surface
                    }
                }
            )
        }
    }
    return $index
}

$candidateRoot = Get-RunRoot $CandidateRunId
$baselineRoot = Get-RunRoot $BaselineRunId

if (-not (Test-Path $candidateRoot)) {
    throw "Candidate run root not found: $candidateRoot"
}

if (-not (Test-Path $baselineRoot)) {
    throw "Baseline run root not found: $baselineRoot"
}

$baselineSelection = Read-JsonFile (Join-Path $baselineRoot "manifest_selection.json")
$candidateSelection = Read-JsonFile (Join-Path $candidateRoot "manifest_selection.json")
$baselineScenarioIds = @($baselineSelection | ForEach-Object { $_.scenario_id })
$candidateScenarioIds = @($candidateSelection | ForEach-Object { $_.scenario_id })

$failures = @()

Compare-Projection "manifest_selection.scenario_ids" $baselineScenarioIds $candidateScenarioIds ([ref]$failures)

$baselineDiff = Get-EngineDiffIndex (Read-JsonFile (Join-Path $baselineRoot "conformance/engine_diff.json"))
$candidateDiff = Get-EngineDiffIndex (Read-JsonFile (Join-Path $candidateRoot "conformance/engine_diff.json"))

foreach ($scenarioId in $baselineScenarioIds) {
    $baselineScenarioRoot = Join-Path $baselineRoot "scenarios/$scenarioId"
    $candidateScenarioRoot = Join-Path $candidateRoot "scenarios/$scenarioId"

    if (-not (Test-Path $candidateScenarioRoot)) {
        $failures += "Missing candidate scenario root: $scenarioId"
        continue
    }

    $baselineResult = Read-JsonFile (Join-Path $baselineScenarioRoot "result.json")
    $candidateResult = Read-JsonFile (Join-Path $candidateScenarioRoot "result.json")
    $baselineTrace = Read-JsonFile (Join-Path $baselineScenarioRoot "trace.json")
    $candidateTrace = Read-JsonFile (Join-Path $candidateScenarioRoot "trace.json")
    $baselineCounters = Read-JsonFile (Join-Path $baselineScenarioRoot "counters.json")
    $candidateCounters = Read-JsonFile (Join-Path $candidateScenarioRoot "counters.json")
    $baselinePublished = Read-JsonFile (Join-Path $baselineScenarioRoot "published_view.json")
    $candidatePublished = Read-JsonFile (Join-Path $candidateScenarioRoot "published_view.json")
    $baselinePinned = Read-JsonFile (Join-Path $baselineScenarioRoot "pinned_views.json")
    $candidatePinned = Read-JsonFile (Join-Path $candidateScenarioRoot "pinned_views.json")
    $baselineRejects = Read-JsonFile (Join-Path $baselineScenarioRoot "rejects.json")
    $candidateRejects = Read-JsonFile (Join-Path $candidateScenarioRoot "rejects.json")

    Compare-Projection "$scenarioId.result_state" $baselineResult.result_state $candidateResult.result_state ([ref]$failures)
    Compare-Projection "$scenarioId.replay_projection.replay_classes" $baselineResult.replay_projection.replay_classes $candidateResult.replay_projection.replay_classes ([ref]$failures)
    Compare-Projection "$scenarioId.replay_projection.required_equality_surfaces" $baselineResult.replay_projection.required_equality_surfaces $candidateResult.replay_projection.required_equality_surfaces ([ref]$failures)
    Compare-Projection "$scenarioId.trace_projection" (Get-TraceProjection $baselineTrace) (Get-TraceProjection $candidateTrace) ([ref]$failures)
    Compare-Projection "$scenarioId.counters" $baselineCounters.counters $candidateCounters.counters ([ref]$failures)
    Compare-Projection "$scenarioId.published_view" $baselinePublished.node_values $candidatePublished.node_values ([ref]$failures)
    Compare-Projection "$scenarioId.pinned_views" $baselinePinned.views $candidatePinned.views ([ref]$failures)
    Compare-Projection "$scenarioId.rejects" $baselineRejects.rejects $candidateRejects.rejects ([ref]$failures)
    Compare-Projection "$scenarioId.engine_diff" $baselineDiff[$scenarioId] $candidateDiff[$scenarioId] ([ref]$failures)
}

if ($failures.Count -gt 0) {
    $failures | ForEach-Object { Write-Error $_ }
    exit 1
}

Write-Output "TraceCalc run parity check passed for '$CandidateRunId' against baseline '$BaselineRunId'."
