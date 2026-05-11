param(
  [string]$RunRoot = "docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-001"
)
$ErrorActionPreference = "Stop"
$errors = New-Object System.Collections.Generic.List[string]
function Add-Err([string]$m) { $errors.Add($m) | Out-Null }
function Read-Json([string]$p) { if (-not (Test-Path $p)) { throw "Missing JSON: $p" }; Get-Content $p -Raw | ConvertFrom-Json }
$summaryPath = Join-Path $RunRoot "w048_materialized_graph_check_summary.json"
$summary = Read-Json $summaryPath
if ([int]$summary.case_count -le 0) { Add-Err "case_count must be positive" }
if ([int]$summary.layer_count -ne ([int]$summary.case_count * 3)) { Add-Err "layer_count must equal case_count * 3" }
if ([int]$summary.check_error_count -ne 0) { Add-Err "check_error_count expected 0 observed $($summary.check_error_count)" }
if ($RunRoot -match "w048-treecalc-cycles-001") {
  if ([int]$summary.case_count -ne 33) { Add-Err "case_count expected 33 observed $($summary.case_count)" }
  if ([int]$summary.layer_count -ne 99) { Add-Err "layer_count expected 99 observed $($summary.layer_count)" }
  if ([int]$summary.cycle_region_count -lt 12) { Add-Err "cycle_region_count expected at least 12 observed $($summary.cycle_region_count)" }
}
$caseDirs = Get-ChildItem (Join-Path $RunRoot "cases") -Directory
$layerCount = 0
$cycleCount = 0
foreach ($case in $caseDirs) {
  $layerFile = Join-Path $case.FullName "w048_materialized_graph_layers.json"
  if (-not (Test-Path $layerFile)) { continue }
  $doc = Read-Json $layerFile
  foreach ($layer in @($doc.layers)) {
    $layerCount++
    $forward = @{}
    foreach ($edge in @($layer.forward_edges)) {
      $key = "$($edge.target_node_id)|$($edge.owner_node_id)|$($edge.edge_id)|$($edge.descriptor_id)"
      $forward[$key] = $true
    }
    $reverse = @{}
    foreach ($edge in @($layer.reverse_edges)) {
      $key = "$($edge.target_node_id)|$($edge.owner_node_id)|$($edge.edge_id)|$($edge.descriptor_id)"
      $reverse[$key] = $true
    }
    foreach ($key in $forward.Keys) { if (-not $reverse.ContainsKey($key)) { Add-Err "$($case.Name):$($layer.graph_layer) missing reverse $key" } }
    foreach ($key in $reverse.Keys) { if (-not $forward.ContainsKey($key)) { Add-Err "$($case.Name):$($layer.graph_layer) extra reverse $key" } }
    foreach ($region in @($layer.cycle_regions)) {
      $cycleCount++
      if (@($region.members).Count -eq 0) { Add-Err "$($case.Name):$($layer.graph_layer) empty cycle region" }
      if (-not $region.cycle_region_id) { Add-Err "$($case.Name):$($layer.graph_layer) missing cycle_region_id" }
      if (-not $region.terminal_policy) { Add-Err "$($case.Name):$($layer.graph_layer) missing terminal_policy" }
    }
  }
}
if ($layerCount -ne [int]$summary.layer_count) { Add-Err "observed layer files count $layerCount differs from summary $($summary.layer_count)" }
if ($cycleCount -ne [int]$summary.cycle_region_count) { Add-Err "observed cycle region count $cycleCount differs from summary $($summary.cycle_region_count)" }
if ($errors.Count -gt 0) { Write-Host "w048 materialized graph check FAILED"; $errors | ForEach-Object { Write-Host "ERROR: $_" }; exit 1 }
Write-Host "w048 materialized graph check ok: cases=$($summary.case_count) layers=$($summary.layer_count) summary=$summaryPath"
