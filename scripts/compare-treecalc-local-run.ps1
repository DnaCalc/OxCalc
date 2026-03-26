param(
    [Parameter(Mandatory = $true)]
    [string]$CandidateRunId,

    [string]$BaselineRunId = "w025-treecalc-local-baseline",

    [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
)

$ErrorActionPreference = "Stop"

function Get-RunRoot([string]$runId) {
    return Join-Path $RepoRoot "docs/test-runs/core-engine/treecalc-local/$runId"
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

function Get-RunSummaryProjection($summary) {
    return [ordered]@{
        case_count = $summary.case_count
        expectation_mismatch_count = $summary.expectation_mismatch_count
        result_counts = Normalize-Object $summary.result_counts
    }
}

function Get-CaseIndexProjection($caseIndex) {
    $projection = [ordered]@{}
    foreach ($entry in $caseIndex) {
        $projection[$entry.case_id] = [ordered]@{
            result_state = $entry.result_state
            conformance_state = $entry.conformance_state
            tags = @($entry.tags)
            has_post_edit = ($null -ne $entry.artifact_paths.post_edit)
        }
    }
    return $projection
}

function Get-ResultProjection($result) {
    $candidateProjection = $null
    if ($null -ne $result.candidate_result) {
        $candidateProjection = [ordered]@{
            target_set = @($result.candidate_result.target_set)
            value_updates = Normalize-Object $result.candidate_result.value_updates
            runtime_effects = Normalize-Object $result.candidate_result.runtime_effects
        }
    }

    $publicationProjection = $null
    if ($null -ne $result.publication_bundle) {
        $publicationProjection = [ordered]@{
            published_view_delta = Normalize-Object $result.publication_bundle.published_view_delta
            published_runtime_effects = Normalize-Object $result.publication_bundle.published_runtime_effects
            trace_markers = @($result.publication_bundle.trace_markers)
        }
    }

    $rejectProjection = $null
    if ($null -ne $result.reject_detail) {
        $rejectProjection = [ordered]@{
            kind = $result.reject_detail.kind
            detail = $result.reject_detail.detail
        }
    }

    return [ordered]@{
        result_state = $result.result_state
        evaluation_order = @($result.evaluation_order)
        diagnostics = @($result.diagnostics)
        candidate_result = $candidateProjection
        publication_bundle = $publicationProjection
        reject_detail = $rejectProjection
    }
}

function Get-TraceProjection($trace) {
    return @(
        foreach ($event in $trace.events) {
            [ordered]@{
                step_id = $event.step_id
                label = $event.label
                node_id = $event.node_id
                kind = $event.kind
                overlay_kind = $event.overlay_kind
                result_state = $event.result_state
                target_set = Normalize-Object $event.target_set
            }
        }
    )
}

function Get-ExplainProjection($explain) {
    $rejectProjection = $null
    if ($null -ne $explain.reject_detail) {
        $rejectProjection = [ordered]@{
            kind = $explain.reject_detail.kind
            detail = $explain.reject_detail.detail
        }
    }

    return [ordered]@{
        conformance_state = $explain.conformance_state
        mismatch_count = $explain.mismatch_count
        mismatches = Normalize-Object $explain.mismatches
        notes = @($explain.notes)
        reject_detail = $rejectProjection
        runtime_effects = Normalize-Object $explain.runtime_effects
        runtime_effect_overlays = Normalize-Object $explain.runtime_effect_overlays
        publication_bundle = Normalize-Object $explain.publication_bundle
    }
}

function Compare-CaseArtifacts([string]$labelPrefix, [string]$baselineCaseRoot, [string]$candidateCaseRoot, [ref]$failures) {
    Compare-Projection "$labelPrefix.result" `
        (Get-ResultProjection (Read-JsonFile (Join-Path $baselineCaseRoot "result.json"))) `
        (Get-ResultProjection (Read-JsonFile (Join-Path $candidateCaseRoot "result.json"))) `
        ([ref]$failures.Value)

    Compare-Projection "$labelPrefix.published_values" `
        (Read-JsonFile (Join-Path $baselineCaseRoot "published_values.json")) `
        (Read-JsonFile (Join-Path $candidateCaseRoot "published_values.json")) `
        ([ref]$failures.Value)

    Compare-Projection "$labelPrefix.runtime_effects" `
        (Read-JsonFile (Join-Path $baselineCaseRoot "runtime_effects.json")) `
        (Read-JsonFile (Join-Path $candidateCaseRoot "runtime_effects.json")) `
        ([ref]$failures.Value)

    Compare-Projection "$labelPrefix.runtime_effect_overlays" `
        (Read-JsonFile (Join-Path $baselineCaseRoot "runtime_effect_overlays.json")) `
        (Read-JsonFile (Join-Path $candidateCaseRoot "runtime_effect_overlays.json")) `
        ([ref]$failures.Value)

    Compare-Projection "$labelPrefix.trace" `
        (Get-TraceProjection (Read-JsonFile (Join-Path $baselineCaseRoot "trace.json"))) `
        (Get-TraceProjection (Read-JsonFile (Join-Path $candidateCaseRoot "trace.json"))) `
        ([ref]$failures.Value)

    Compare-Projection "$labelPrefix.engine_diff" `
        (Read-JsonFile (Join-Path $baselineCaseRoot "engine_diff.json")) `
        (Read-JsonFile (Join-Path $candidateCaseRoot "engine_diff.json")) `
        ([ref]$failures.Value)

    Compare-Projection "$labelPrefix.explain" `
        (Get-ExplainProjection (Read-JsonFile (Join-Path $baselineCaseRoot "explain.json"))) `
        (Get-ExplainProjection (Read-JsonFile (Join-Path $candidateCaseRoot "explain.json"))) `
        ([ref]$failures.Value)
}

$candidateRoot = Get-RunRoot $CandidateRunId
$baselineRoot = Get-RunRoot $BaselineRunId

if (-not (Test-Path $candidateRoot)) {
    throw "Candidate run root not found: $candidateRoot"
}

if (-not (Test-Path $baselineRoot)) {
    throw "Baseline run root not found: $baselineRoot"
}

$failures = @()

$baselineRunSummary = Read-JsonFile (Join-Path $baselineRoot "run_summary.json")
$candidateRunSummary = Read-JsonFile (Join-Path $candidateRoot "run_summary.json")
$baselineCaseIndex = Read-JsonFile (Join-Path $baselineRoot "case_index.json")
$candidateCaseIndex = Read-JsonFile (Join-Path $candidateRoot "case_index.json")
$baselineConformanceSummary = Read-JsonFile (Join-Path $baselineRoot "conformance/conformance_summary.json")
$candidateConformanceSummary = Read-JsonFile (Join-Path $candidateRoot "conformance/conformance_summary.json")
$baselineExplainIndex = Read-JsonFile (Join-Path $baselineRoot "conformance/explain_index.json")
$candidateExplainIndex = Read-JsonFile (Join-Path $candidateRoot "conformance/explain_index.json")

Compare-Projection "run_summary" `
    (Get-RunSummaryProjection $baselineRunSummary) `
    (Get-RunSummaryProjection $candidateRunSummary) `
    ([ref]$failures)

Compare-Projection "case_index" `
    (Get-CaseIndexProjection $baselineCaseIndex) `
    (Get-CaseIndexProjection $candidateCaseIndex) `
    ([ref]$failures)

Compare-Projection "conformance_summary" `
    ([ordered]@{
        case_count = $baselineConformanceSummary.case_count
        mismatch_case_count = $baselineConformanceSummary.mismatch_case_count
        expectation_mismatch_count = $baselineConformanceSummary.expectation_mismatch_count
        conformance_pass_count = $baselineConformanceSummary.conformance_pass_count
    }) `
    ([ordered]@{
        case_count = $candidateConformanceSummary.case_count
        mismatch_case_count = $candidateConformanceSummary.mismatch_case_count
        expectation_mismatch_count = $candidateConformanceSummary.expectation_mismatch_count
        conformance_pass_count = $candidateConformanceSummary.conformance_pass_count
    }) `
    ([ref]$failures)

Compare-Projection "conformance.explain_index" `
    (Normalize-Object ($baselineExplainIndex | ForEach-Object {
        [ordered]@{
            case_id = $_.case_id
            conformance_state = $_.conformance_state
            result_state = $_.result_state
        }
    })) `
    (Normalize-Object ($candidateExplainIndex | ForEach-Object {
        [ordered]@{
            case_id = $_.case_id
            conformance_state = $_.conformance_state
            result_state = $_.result_state
        }
    })) `
    ([ref]$failures)

foreach ($baselineEntry in $baselineCaseIndex) {
    $caseId = $baselineEntry.case_id
    $candidateEntry = $candidateCaseIndex | Where-Object { $_.case_id -eq $caseId } | Select-Object -First 1
    if ($null -eq $candidateEntry) {
        $failures += "Missing candidate case: $caseId"
        continue
    }

    $baselineCaseRoot = Join-Path $baselineRoot "cases/$caseId"
    $candidateCaseRoot = Join-Path $candidateRoot "cases/$caseId"

    if (-not (Test-Path $candidateCaseRoot)) {
        $failures += "Missing candidate case root: $caseId"
        continue
    }

    Compare-CaseArtifacts $caseId $baselineCaseRoot $candidateCaseRoot ([ref]$failures)

    $baselineHasPostEdit = $null -ne $baselineEntry.artifact_paths.post_edit
    $candidateHasPostEdit = $null -ne $candidateEntry.artifact_paths.post_edit
    if ($baselineHasPostEdit -ne $candidateHasPostEdit) {
        $failures += "Mismatch: $caseId.post_edit_presence"
        continue
    }

    if ($baselineHasPostEdit) {
        $baselinePostEditRoot = Join-Path $baselineCaseRoot "post_edit"
        $candidatePostEditRoot = Join-Path $candidateCaseRoot "post_edit"

        if (-not (Test-Path $candidatePostEditRoot)) {
            $failures += "Missing candidate post-edit root: $caseId"
            continue
        }

        Compare-Projection "$caseId.post_edit.edit_outcomes" `
            (Read-JsonFile (Join-Path $baselinePostEditRoot "edit_outcomes.json")) `
            (Read-JsonFile (Join-Path $candidatePostEditRoot "edit_outcomes.json")) `
            ([ref]$failures)

        Compare-Projection "$caseId.post_edit.result" `
            (Read-JsonFile (Join-Path $baselinePostEditRoot "result.json")) `
            (Read-JsonFile (Join-Path $candidatePostEditRoot "result.json")) `
            ([ref]$failures)

        Compare-Projection "$caseId.post_edit.runtime_effects" `
            (Read-JsonFile (Join-Path $baselinePostEditRoot "runtime_effects.json")) `
            (Read-JsonFile (Join-Path $candidatePostEditRoot "runtime_effects.json")) `
            ([ref]$failures)

        Compare-Projection "$caseId.post_edit.runtime_effect_overlays" `
            (Read-JsonFile (Join-Path $baselinePostEditRoot "runtime_effect_overlays.json")) `
            (Read-JsonFile (Join-Path $candidatePostEditRoot "runtime_effect_overlays.json")) `
            ([ref]$failures)

        Compare-Projection "$caseId.post_edit.trace" `
            (Get-TraceProjection (Read-JsonFile (Join-Path $baselinePostEditRoot "trace.json"))) `
            (Get-TraceProjection (Read-JsonFile (Join-Path $candidatePostEditRoot "trace.json"))) `
            ([ref]$failures)

        Compare-Projection "$caseId.post_edit.explain" `
            (Get-ExplainProjection (Read-JsonFile (Join-Path $baselinePostEditRoot "explain.json"))) `
            (Get-ExplainProjection (Read-JsonFile (Join-Path $candidatePostEditRoot "explain.json"))) `
            ([ref]$failures)
    }
}

if ($failures.Count -gt 0) {
    $failures | ForEach-Object { Write-Error $_ }
    exit 1
}

Write-Output "TreeCalc local run parity check passed for '$CandidateRunId' against baseline '$BaselineRunId'."
