param(
  [string]$LedgerPath = "docs/spec/core-engine/w048-cycles/W048_INNOVATION_OPPORTUNITY_LEDGER.json"
)
$ErrorActionPreference = "Stop"
$errors = New-Object System.Collections.Generic.List[string]
function Add-Err([string]$m) { $errors.Add($m) | Out-Null }
function Read-Json([string]$p) { if (-not (Test-Path $p)) { throw "Missing JSON: $p" }; Get-Content $p -Raw | ConvertFrom-Json }
$data = Read-Json $LedgerPath
if ($data.schema_version -ne "oxcalc.w048.innovation_opportunity_ledger.v1") { Add-Err "unexpected schema_version" }
if (($data.default_rule -as [string]) -notmatch "No innovation profile is default Excel compatibility") { Add-Err "default rule must prohibit default Excel-compatibility promotion" }
$entries = @($data.entries)
if ($entries.Count -lt 6) { Add-Err "expected at least six innovation entries" }
$requiredFields = @("profile_id","status","target_problem","excel_relation","semantic_contract","termination_or_rejection_policy","graph_requirements","required_tests","formal_obligations","admission_gate")
$ids = @{}
foreach ($entry in $entries) {
  $profile = if ($entry.profile_id) { [string]$entry.profile_id } else { "<missing>" }
  if ($ids.ContainsKey($profile)) { Add-Err "duplicate profile_id $profile" } else { $ids[$profile] = $true }
  foreach ($field in $requiredFields) { if (-not ($entry.PSObject.Properties.Name -contains $field)) { Add-Err "$profile missing $field" } }
  foreach ($field in @("graph_requirements","required_tests","formal_obligations")) { if (@($entry.$field).Count -eq 0) { Add-Err "$profile $field must be non-empty list" } }
  if ($entry.excel_relation -eq "excel_match_default") { Add-Err "$profile incorrectly marked as default Excel match" }
  if (@("candidate_profile_not_admitted","diagnostic_profile_partially_exercised","optimization_profile_partially_exercised") -notcontains $entry.status) { Add-Err "$profile has unsupported status $($entry.status)" }
}
foreach ($required in @("cycle.fixed_point.monotone_lfp.v0","cycle.recurrence.explicit_state.v0","cycle.diagnostics.materialized_region.v0","cycle.retained_prior.safe_state.v0","cycle.iterative.bounded_with_oscillation_diagnostics.v0","cycle.release.local_frontier_repair.v0")) { if (-not $ids.ContainsKey($required)) { Add-Err "missing profile $required" } }
if ($errors.Count -gt 0) { Write-Host "w048 innovation ledger FAILED"; $errors | ForEach-Object { Write-Host "ERROR: $_" }; exit 1 }
Write-Host "w048 innovation ledger ok"
