param(
  [string]$SummaryPath = "docs/test-runs/core-engine/formal/w048-cycle-artifacts-001/w048_formal_cycle_checker_summary.json"
)
$ErrorActionPreference = "Stop"
$errors = New-Object System.Collections.Generic.List[string]
function Add-Err([string]$m) { $errors.Add($m) | Out-Null }
function Read-Json([string]$p) { if (-not (Test-Path $p)) { throw "Missing JSON: $p" }; Get-Content $p -Raw | ConvertFrom-Json }
$s = Read-Json $SummaryPath
if ($s.schema_version -ne "oxcalc.w048.formal_cycle_checker_summary.v1") { Add-Err "unexpected schema_version" }
if ($s.status -ne "passed") { Add-Err "status expected passed observed $($s.status)" }
if (@($s.errors).Count -ne 0) { Add-Err "summary has errors: $($s.errors -join '; ')" }
if ([int]$s.graph_summary.case_count -ne 33) { Add-Err "case_count expected 33" }
if ([int]$s.graph_summary.layer_count -ne 99) { Add-Err "layer_count expected 99" }
if ([int]$s.graph_summary.cycle_region_count -ne 12) { Add-Err "cycle_region_count expected 12" }
if ([int]$s.graph_summary.check_error_count -ne 0) { Add-Err "check_error_count expected 0" }
foreach ($name in @("forward_reverse_converse","cycle_regions_nonempty","cycle_reject_no_publication","release_reentry_publishes_owner_and_downstream")) { if ($s.obligations.$name -ne $true) { Add-Err "obligation $name is not true" } }
$release = $s.case_results.tc_w048_ctro_dynamic_release_reentry_downstream_001
if ($release.initial_state -ne "rejected" -or $release.reject_kind -ne "SyntheticCycleReject") { Add-Err "release case initial reject mismatch" }
if ($release.post_edit_state -ne "published") { Add-Err "release case post-edit must publish" }
if ($release.post_edit_published_values.'2' -ne "10" -or $release.post_edit_published_values.'4' -ne "11") { Add-Err "release case post-edit values expected 10/11" }
if ($errors.Count -gt 0) { Write-Host "w048 formal cycle checker FAILED"; $errors | ForEach-Object { Write-Host "ERROR: $_" }; exit 1 }
Write-Host "w048 formal cycle checker passed: $SummaryPath"
