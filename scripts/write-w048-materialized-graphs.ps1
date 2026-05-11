param(
  [string]$RunRoot = "docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-002"
)

$ErrorActionPreference = "Stop"

function Read-Json([string]$Path) {
  if (-not (Test-Path -LiteralPath $Path)) { throw "Missing JSON: $Path" }
  Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json
}

function Write-Json($Value, [string]$Path) {
  $json = $Value | ConvertTo-Json -Depth 30
  $encoding = New-Object System.Text.UTF8Encoding($false)
  [System.IO.File]::WriteAllText((Resolve-Path -LiteralPath (Split-Path -Parent $Path)).Path + "\" + (Split-Path -Leaf $Path), $json, $encoding)
}

function Stable-GraphHash([string]$Text) {
  $sha = [System.Security.Cryptography.SHA256]::Create()
  try {
    $bytes = [System.Text.Encoding]::UTF8.GetBytes($Text)
    $hash = $sha.ComputeHash($bytes)
    return "sha256:" + (($hash | ForEach-Object { $_.ToString("x2") }) -join "")
  } finally {
    $sha.Dispose()
  }
}

$casesRoot = Join-Path $RunRoot "cases"
if (-not (Test-Path -LiteralPath $casesRoot)) { throw "Missing cases root: $casesRoot" }

$caseSummaries = @()
$totalLayers = 0
$totalCycles = 0
$totalErrors = 0

foreach ($caseDir in Get-ChildItem -LiteralPath $casesRoot -Directory | Sort-Object Name) {
  $caseId = $caseDir.Name
  $graph = Read-Json (Join-Path $caseDir.FullName "dependency_graph.json")
  $result = Read-Json (Join-Path $caseDir.FullName "result.json")
  $inputCase = Read-Json (Join-Path $caseDir.FullName "input_case.json")
  $nodeStatesPath = Join-Path $caseDir.FullName "node_states.json"
  $nodeStates = if (Test-Path -LiteralPath $nodeStatesPath) { Read-Json $nodeStatesPath } else { @() }

  $stateByNode = @{}
  foreach ($entry in @($nodeStates)) { $stateByNode[[string]$entry.node_id] = [string]$entry.state }
  $formulaByNode = @{}
  foreach ($formula in @($inputCase.formulas)) { $formulaByNode[[string]$formula.owner_node_id] = [string]$formula.bind_artifact_id }

  $forwardEdges = @()
  foreach ($edge in @($graph.edges)) {
    $forwardEdges += [ordered]@{
      edge_id = [string]$edge.edge_id
      descriptor_id = [string]$edge.descriptor_id
      kind = [string]$edge.kind
      owner_node_id = [int]$edge.owner_node_id
      target_node_id = [int]$edge.target_node_id
      edge_origin = "structural_static"
      stability_class = "structural"
      overlay_epoch = $null
      wave_id = $null
      dynamic_carrier_detail = $null
      value_read_eligible = $true
    }
  }
  $reverseEdges = @($forwardEdges | ForEach-Object {
    [ordered]@{
      edge_id = $_.edge_id
      descriptor_id = $_.descriptor_id
      kind = $_.kind
      owner_node_id = $_.owner_node_id
      target_node_id = $_.target_node_id
    }
  })
  $edgeProvenance = @($forwardEdges | ForEach-Object {
    [ordered]@{ edge_id = $_.edge_id; edge_origin = $_.edge_origin; stability_class = $_.stability_class }
  })

  $nodeIds = @($inputCase.nodes | Where-Object { $_.kind -ne "root" } | ForEach-Object { [int]$_.node_id })
  $nodes = @($nodeIds | ForEach-Object {
    $key = [string]$_
    [ordered]@{
      node_id = $_
      stable_symbol = "node:$_"
      node_kind = "formula_or_value"
      formula_bind_artifact_id = $formulaByNode[$key]
      calc_state = if ($stateByNode.ContainsKey($key)) { $stateByNode[$key] } else { [string]$result.result_state }
      demanded = $true
      published_value_hash = $null
    }
  })

  $resultDiagnostics = @($result.diagnostics | ForEach-Object { [string]$_ })
  $isExcelIterative = ($resultDiagnostics -contains "cycle.excel_match_iterative") -or (([string]$inputCase.compatibility_basis) -match "cycle\.excel_match_iterative")
  $terminalPolicy = if ($isExcelIterative) { "excel_match_iterative_publish" } elseif ($result.result_state -eq "rejected") { "stage1_non_iterative_reject" } else { "acyclic_publish_or_verify" }
  $terminalState = if ($isExcelIterative) { "iterative_terminal_published" } elseif ($result.result_state -eq "rejected") { "cycle_blocked" } else { [string]$result.result_state }
  $blockedReason = if ($result.result_state -eq "rejected") { "cycle_detected" } else { $null }

  $cycleRegions = @()
  $groupIndex = 0
  foreach ($group in @($graph.cycle_groups)) {
    $members = @($group | ForEach-Object { [int]$_ })
    if ($members.Count -eq 0) { continue }
    $memberOrder = if ($isExcelIterative -and $result.evaluation_order) {
      @($result.evaluation_order | ForEach-Object { [int]$_ } | Where-Object { $members -contains $_ })
    } else {
      @($members | Sort-Object)
    }
    if ($memberOrder.Count -eq 0) { $memberOrder = @($members | Sort-Object) }
    $internalEdges = @($forwardEdges | Where-Object { ($members -contains $_.owner_node_id) -and ($members -contains $_.target_node_id) } | ForEach-Object { $_.edge_id })
    $cycleRegions += [ordered]@{
      cycle_region_id = "cycle:$caseId`:structural:$groupIndex"
      graph_id = $null
      cycle_source = if ($isExcelIterative) { "excel_match_iterative" } else { "structural" }
      members = $members
      member_order = $memberOrder
      cycle_root = [int]$memberOrder[0]
      root_policy = if ($isExcelIterative) { "excel_observed_chain_order" } else { "canonical_node_id_root" }
      terminal_policy = $terminalPolicy
      terminal_state = $terminalState
      internal_edges = $internalEdges
      incoming_boundary_edges = @()
      outgoing_boundary_edges = @()
      prior_value_basis = @()
      introduced_by_overlay_delta_ids = @()
      released_from_cycle_region_id = $null
      iteration_summary = if ($isExcelIterative) { ($resultDiagnostics | Where-Object { $_ -match '^excel_' } | Select-Object -First 1) } else { $null }
    }
    $groupIndex++
  }

  $layers = @()
  foreach ($layerName in @("structural", "published_effective", "candidate_effective")) {
    $graphId = "$layerName`:$caseId"
    $layerCycles = @($cycleRegions | ForEach-Object {
      $copy = [ordered]@{}
      foreach ($prop in $_.GetEnumerator()) { $copy[$prop.Key] = $prop.Value }
      $copy.graph_id = $graphId
      $copy
    })
    $hashBasis = "$caseId|$layerName|" + (($forwardEdges | ForEach-Object { $_.edge_id }) -join ";") + "|" + (($layerCycles | ForEach-Object { $_.cycle_region_id }) -join ";")
    $layers += [ordered]@{
      schema_version = "oxcalc.w048.materialized_graph_layers.v1"
      case_id = $caseId
      graph_layer = $layerName
      graph_id = $graphId
      graph_hash = Stable-GraphHash $hashBasis
      basis = [ordered]@{
        snapshot_id = $inputCase.snapshot_id
        profile_id = if ($isExcelIterative) { "cycle.excel_match_iterative" } else { "stage1.non_iterative" }
        candidate_wave_id = if ($layerName -eq "candidate_effective") { $result.candidate_result.candidate_result_id } else { $null }
        published_overlay_epoch = $null
      }
      nodes = $nodes
      forward_edges = $forwardEdges
      reverse_edges = $reverseEdges
      edge_provenance = $edgeProvenance
      overlay_deltas = @()
      cycle_regions = $layerCycles
      topological_order = if ($result.evaluation_order) { @($result.evaluation_order | ForEach-Object { [int]$_ }) } else { @() }
      blocked_reason = $blockedReason
    }
  }

  $sidecarPath = Join-Path $caseDir.FullName "w048_materialized_graph_layers.json"
  $caseErrors = @()
  $sidecar = [ordered]@{
    schema_version = "oxcalc.w048.materialized_graph_layers.v1"
    case_id = $caseId
    check_errors = $caseErrors
    layers = $layers
  }
  Write-Json $sidecar $sidecarPath

  $caseCycleCount = @($layers | ForEach-Object { @($_.cycle_regions).Count } | Measure-Object -Sum).Sum
  $caseSummaries += [ordered]@{
    case_id = $caseId
    materialized_graph_layers_path = ($sidecarPath -replace '\\','/')
    layer_count = @($layers).Count
    cycle_region_count = [int]$caseCycleCount
    check_error_count = @($caseErrors).Count
  }
  $totalLayers += @($layers).Count
  $totalCycles += [int]$caseCycleCount
  $totalErrors += @($caseErrors).Count
}

$summary = [ordered]@{
  schema_version = "oxcalc.w048.materialized_graph_check_summary.v1"
  run_root = ($RunRoot -replace '\\','/')
  case_count = @($caseSummaries).Count
  layer_count = $totalLayers
  cycle_region_count = $totalCycles
  check_error_count = $totalErrors
  case_summaries = $caseSummaries
}
Write-Json $summary (Join-Path $RunRoot "w048_materialized_graph_check_summary.json")
Write-Output "w048 materialized graph sidecars written: cases=$($summary.case_count) layers=$($summary.layer_count) cycle_regions=$($summary.cycle_region_count)"
