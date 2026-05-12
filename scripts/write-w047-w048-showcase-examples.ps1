param(
  [string]$OutputPath = "docs/showcase/assets/w047_w048_showcase_examples.json"
)
$ErrorActionPreference = "Stop"
function Read-Json([string]$Path) {
  if (-not (Test-Path -LiteralPath $Path)) { throw "missing $Path" }
  Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json
}
function Write-Json($Value, [string]$Path, [int]$Depth = 60) {
  $parent = Split-Path -Parent $Path
  if (-not (Test-Path -LiteralPath $parent)) { New-Item -ItemType Directory -Force -Path $parent | Out-Null }
  $encoding = [System.Text.UTF8Encoding]::new($false)
  [System.IO.File]::WriteAllText((Resolve-Path -LiteralPath $parent).Path + "\" + (Split-Path -Leaf $Path), ($Value | ConvertTo-Json -Depth $Depth), $encoding)
}

$ctroResult = Read-Json "docs/test-runs/core-engine/treecalc-local/w047-ctro-dynamic-positive-publication-001/cases/tc_local_dynamic_target_switch_downstream_publish_001/post_edit/result.json"
$ctroTrace = Read-Json "docs/test-runs/core-engine/treecalc-local/w047-ctro-dynamic-positive-publication-001/cases/tc_local_dynamic_target_switch_downstream_publish_001/post_edit/trace.json"
$cycleReject = Read-Json "docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-002/cases/tc_w048_ctro_dynamic_release_reentry_downstream_001/result.json"
$cycleRelease = Read-Json "docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-002/cases/tc_w048_ctro_dynamic_release_reentry_downstream_001/post_edit/result.json"
$iter2 = Read-Json "docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-002/cases/tc_w048_excel_iter_two_node_order_001/result.json"
$iter3 = Read-Json "docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-002/cases/tc_w048_excel_iter_three_node_order_001/result.json"
$fraction = Read-Json "docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-002/cases/tc_w048_excel_iter_fraction_precision_001/result.json"
$conformance = Read-Json "docs/test-runs/core-engine/w048-conformance-002/w048_conformance_summary.json"

$examples = [ordered]@{
  schema_version = "oxcalc.showcase.w047_w048_examples.v1"
  generated_from = @(
    "w047-ctro-dynamic-positive-publication-001",
    "w048-treecalc-cycles-002",
    "w048-conformance-002"
  )
  examples = @(
    [ordered]@{
      id = "ctro_target_switch_downstream_publish"
      title = "CTRO target switch publishes value + overlay atomically"
      kind = "W047_CTRO"
      steps = @(
        [ordered]@{ label="Seed"; detail="Published dynamic owner points at old target node 2." },
        [ordered]@{ label="Edit/evaluate"; detail="Current graph resolves owner node 3 to new target node 4." },
        [ordered]@{ label="Diff overlay"; detail="Emit release_dynamic_dep [2,3] and activate_dynamic_dep [3,4]." },
        [ordered]@{ label="Publish"; detail="Publish owner node 3 = 7 and downstream node 5 = 8 with runtime effect." }
      )
      observed = [ordered]@{
        result_state = $ctroResult.result_state
        evaluation_order = @($ctroResult.evaluation_order)
        dependency_shape_updates = @($ctroResult.candidate_result.dependency_shape_updates)
        value_updates = $ctroResult.candidate_result.value_updates
        published_values = $ctroResult.published_values
        trace_labels = @($ctroTrace.events | ForEach-Object { $_.label })
      }
    },
    [ordered]@{
      id = "ctro_cycle_reject_then_release_reentry"
      title = "CTRO-created cycle rejects, later release re-enters and publishes"
      kind = "W048_CTRO_CYCLE"
      steps = @(
        [ordered]@{ label="Candidate self-cycle"; detail="Dynamic candidate creates a cycle; default profile rejects." },
        [ordered]@{ label="No publication"; detail="Rejected candidate has no publication bundle." },
        [ordered]@{ label="Release to acyclic target"; detail="Post-edit dynamic target points to acyclic node 3." },
        [ordered]@{ label="Publish downstream"; detail="Owner node 2 publishes 10 and downstream node 4 publishes 11." }
      )
      observed = [ordered]@{
        initial_result_state = $cycleReject.result_state
        initial_reject_kind = $cycleReject.reject_detail.kind
        initial_publication_bundle_is_null = ($null -eq $cycleReject.publication_bundle)
        post_edit_result_state = $cycleRelease.result_state
        post_edit_evaluation_order = @($cycleRelease.evaluation_order)
        post_edit_value_updates = $cycleRelease.candidate_result.value_updates
        post_edit_published_values = $cycleRelease.published_values
      }
    },
    [ordered]@{
      id = "excel_iterative_chain_order"
      title = "Single-host Excel-match iterative chain order"
      kind = "W048_EXCEL_MATCH_ITERATIVE"
      steps = @(
        [ordered]@{ label="Two-node order"; detail="Probe-compatible basis selects B1,A1 order." },
        [ordered]@{ label="Three-node order"; detail="Probe-compatible basis selects C1,B1,A1 order." },
        [ordered]@{ label="Fraction precision"; detail="Fractional fixture preserves 0.33333333333333331." }
      )
      observed = [ordered]@{
        two_node_evaluation_order = @($iter2.evaluation_order)
        two_node_updates = $iter2.candidate_result.value_updates
        three_node_evaluation_order = @($iter3.evaluation_order)
        three_node_updates = $iter3.candidate_result.value_updates
        fraction_updates = $fraction.candidate_result.value_updates
        diagnostics = @($iter2.diagnostics)
      }
    },
    [ordered]@{
      id = "w048_conformance_single_host_scope"
      title = "Final W048 status is complete for the accepted single-host scope"
      kind = "W048_CONFORMANCE"
      observed = [ordered]@{
        status = $conformance.status
        scope_completeness = $conformance.scope_completeness
        target_completeness = $conformance.target_completeness
        integration_completeness = $conformance.integration_completeness
        source_summaries = $conformance.source_summaries
        accepted_scope = $conformance.accepted_scope
        documented_limitations = @($conformance.documented_limitations)
      }
    }
  )
}
Write-Json $examples $OutputPath
Write-Output "w047/w048 showcase examples written: $OutputPath"
