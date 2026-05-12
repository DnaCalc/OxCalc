param(
  [string]$SummaryPath = "docs/test-runs/core-engine/w048-conformance-002/w048_conformance_summary.json"
)
$ErrorActionPreference = "Stop"
$errors = New-Object System.Collections.Generic.List[string]
function Add-Err([string]$m) { $errors.Add($m) | Out-Null }
function Read-Json([string]$p) { if (-not (Test-Path $p)) { throw "Missing JSON: $p" }; Get-Content $p -Raw | ConvertFrom-Json }
$s = Read-Json $SummaryPath
if ($s.schema_version -ne "oxcalc.w048.conformance_summary.v2") { Add-Err "unexpected schema_version" }
if ($s.status -notin @("passed_with_named_excel_blockers", "passed_with_named_excel_version_blocker")) { Add-Err "status expected passed_with_named_excel_version_blocker observed $($s.status)" }
if (@($s.errors).Count -ne 0) { Add-Err "summary has errors: $($s.errors -join '; ')" }
if ($s.scope_completeness -ne "scope_partial") { Add-Err "scope_completeness must remain scope_partial while blockers exist" }
if ($s.target_completeness -ne "target_partial") { Add-Err "target_completeness must remain target_partial while blockers exist" }
if ($s.integration_completeness -ne "partial") { Add-Err "integration_completeness must remain partial while blockers exist" }
$src = $s.source_summaries
if ([int]$src.excel_core_observation_count -lt 12) { Add-Err "core Excel observation count below 12" }
if ([int]$src.excel_bitexact_observation_count -lt 19) { Add-Err "bitexact Excel observation count below 19" }
if ([int]$src.tracecalc_scenario_count -ne 38) { Add-Err "TraceCalc scenario_count expected 38" }
if ([int]$src.tracecalc_passed -ne 38) { Add-Err "TraceCalc passed count expected 38" }
if ([int]$src.treecalc_case_count -ne 37) { Add-Err "TreeCalc case_count expected 37" }
if ([int]$src.treecalc_expectation_mismatch_count -ne 0) { Add-Err "TreeCalc expectation mismatches expected 0" }
if ([int]$src.graph_case_count -ne 37 -or [int]$src.graph_layer_count -ne 111) { Add-Err "graph coverage expected reopened 37 cases / 111 layers" }
if ([int]$src.graph_cycle_region_count -lt 24) { Add-Err "graph cycle region count below reopened floor 24" }
if ([int]$src.graph_check_error_count -ne 0) { Add-Err "graph check errors expected 0" }
foreach ($id in @("tc_w048_excel_iter_two_node_order_001","tc_w048_excel_iter_three_node_order_001","tc_w048_excel_iter_fraction_precision_001","tc_w048_excel_ctro_indirect_iterative_self_001")) {
  if (@($s.tracecalc_iterative_fixtures) -notcontains $id) { Add-Err "missing TraceCalc iterative fixture $id" }
  if (@($s.treecalc_iterative_fixtures) -notcontains $id) { Add-Err "missing TreeCalc iterative fixture $id" }
}
if (-not (Test-Path $s.multithread_variant_run)) { Add-Err "missing multithread variant run" }
if (@($s.blockers) -notcontains "BLK-W048-EXCEL-VERSION") { Add-Err "missing blocker BLK-W048-EXCEL-VERSION" }
if ($s.status -eq "passed_with_named_excel_version_blocker" -and (@($s.blockers) -contains "BLK-W048-EXCEL-ROOT")) { Add-Err "cleared root blocker should not remain in conformance blockers" }
if ($s.status -eq "passed_with_named_excel_blockers" -and (@($s.blockers) -notcontains "BLK-W048-EXCEL-ROOT")) { Add-Err "legacy blocker status missing BLK-W048-EXCEL-ROOT" }
if ($errors.Count -gt 0) { Write-Host "w048 conformance FAILED"; $errors | ForEach-Object { Write-Host "ERROR: $_" }; exit 1 }
Write-Host "w048 conformance passed with named blocker disposition: $SummaryPath"
