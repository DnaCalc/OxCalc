param(
  [string]$DecisionPath = "docs/spec/core-engine/w048-cycles/W048_ITERATIVE_PROFILE_DECISION.json"
)
$ErrorActionPreference = "Stop"
$errors = New-Object System.Collections.Generic.List[string]
function Add-Err([string]$m) { $errors.Add($m) | Out-Null }
function Read-Json([string]$p) { if (-not (Test-Path $p)) { throw "Missing JSON: $p" }; Get-Content $p -Raw | ConvertFrom-Json }
$d = Read-Json $DecisionPath
if ($d.schema_version -ne "oxcalc.w048.iterative_profile_decision.v2") { Add-Err "unexpected schema_version" }
if ($d.status -ne "excel_match_profile_specified_with_named_open_blockers") { Add-Err "unexpected status" }
if ($d.default_profile.profile_id -ne "cycle.non_iterative_stage1") { Add-Err "missing default non-iterative profile" }
if ($d.default_profile.cycle_terminal_state -ne "reject_candidate") { Add-Err "default terminal state must reject candidate" }
if (($d.default_profile.publication_rule -as [string]) -notmatch "no_new_cycle_values") { Add-Err "default publication rule must prohibit new cycle values" }

$e = $d.excel_match_profile
if ($e.profile_id -ne "cycle.excel_match_iterative") { Add-Err "missing Excel-match profile" }
if ($e.admission -ne "implementation_target_for_reopened_w048") { Add-Err "Excel-match profile must be the reopened W048 implementation target" }
$evidenceRuns = @($e.evidence_runs)
foreach ($run in @("docs/test-runs/excel-cycles/w048-excel-cycles-001/observation.json", "docs/test-runs/excel-cycles/w048-excel-cycles-bitexact-001/observation.json")) {
  if ($evidenceRuns -notcontains $run) { Add-Err "missing evidence run $run" }
  if (-not (Test-Path $run)) { Add-Err "evidence run path not found: $run" }
}
if ($e.root_policy.status -ne "blocked_for_report_cell_exactness") { Add-Err "root policy must preserve report-cell blocker" }
if (-not $e.root_policy.blocker) { Add-Err "root policy blocker missing" }
if ($e.initial_value_policy.status -ne "observed_for_declared_self_cycle_prior_states") { Add-Err "initial vector status must cover declared prior states" }
if (($e.initial_value_policy.published_prior_numeric -as [string]) -notmatch "do not survive") { Add-Err "numeric-prior initial vector disposition missing" }
if (($e.initial_value_policy.text_or_non_numeric_prior -as [string]) -notmatch "do_not_survive|do not survive") { Add-Err "nonnumeric-prior initial vector disposition missing" }
if ($e.member_order_policy.policy -notmatch "saved_calc_chain_order") { Add-Err "member order policy must use saved calc-chain candidate" }
if ($e.update_model.policy -ne "chain_ordered_sequential_region_update") { Add-Err "Excel update model must be chain ordered sequential candidate" }
if ($e.thread_mode_policy.single_thread_observed -ne "implemented_declared_fixture_profile") { Add-Err "single-thread profile disposition missing" }
if ($e.thread_mode_policy.multithread_observed -ne "observed_variant_not_yet_implemented_in_tracecalc_treecalc") { Add-Err "multithread variant disposition missing" }
if (-not (Test-Path $e.thread_mode_policy.multithread_evidence_run)) { Add-Err "multithread evidence run missing" }
if ($e.stop_metric.kind -notmatch "maximum_absolute") { Add-Err "Excel stop metric must name max absolute delta" }
if ([double]$e.stop_metric.default_max_change -ne 0.001) { Add-Err "Excel default max_change must be 0.001" }
foreach ($k in @("converged","max_iterations_without_convergence","oscillation","non_numeric_or_error","dynamic_release")) { if (-not ($e.terminal_states.PSObject.Properties.Name -contains $k)) { Add-Err "missing Excel terminal state $k" } }
$diagnostics = @($e.diagnostics)
foreach ($diag in @("cycle_region_id","cycle_member_order","initial_value_vector","iteration_index","max_member_delta","terminal_state","publication_decision","evidence_run_id")) { if ($diagnostics -notcontains $diag) { Add-Err "missing Excel diagnostic $diag" } }

$o = $d.oxcalc_deterministic_profile
$required = @{
  profile_id = "cycle.iterative_deterministic_v0";
  admission = "explicit_opt_in_only";
  root_policy = "canonical_node_id_root";
  update_model = "jacobi_snapshot_region_function";
  publication_rule = "publish_only_after_converged_complete_region_and_dependent_wave"
}
foreach ($k in $required.Keys) { if ($o.$k -ne $required[$k]) { Add-Err "oxcalc_deterministic_profile.$k expected '$($required[$k])' observed '$($o.$k)'" } }
foreach ($k in @("cycle_members","non_numeric_or_error_prior","missing_boundary_value")) { if (-not ($o.initial_value_policy.PSObject.Properties.Name -contains $k)) { Add-Err "missing deterministic initial_value_policy.$k" } }
if ($o.stop_metric.kind -ne "max_absolute_member_delta") { Add-Err "deterministic stop metric must be max_absolute_member_delta" }
if ([double]$o.stop_metric.default_max_change -ne 0.001) { Add-Err "deterministic default max_change must be 0.001" }
if ([int]$o.iteration_bound.default_max_iterations -ne 100) { Add-Err "default max_iterations must be 100" }
foreach ($k in @("converged","max_iterations_without_convergence","divergence_detected","oscillation_detected","non_numeric_result")) { if (-not ($o.terminal_states.PSObject.Properties.Name -contains $k)) { Add-Err "missing deterministic terminal state $k" } }
if ($o.terminal_states.converged -ne "accept_candidate_and_publish_whole_region_atomically") { Add-Err "deterministic converged terminal must publish whole region atomically" }
foreach ($p in $o.terminal_states.PSObject.Properties) { if ($p.Name -ne "converged" -and $p.Value -ne "reject_candidate_no_publication") { Add-Err "$($p.Name) terminal must reject with no publication" } }
if (-not $o.semantic_equivalence_requirement) { Add-Err "missing semantic_equivalence_requirement" }

$fixtures = @($d.falsification_fixtures)
foreach ($id in @("excel_iter_two_node_order_001","excel_iter_three_node_order_001","excel_iter_fraction_precision_001","excel_ctro_indirect_iterative_self_001")) {
  if (-not ($fixtures | Where-Object { $_.probe_id -eq $id })) { Add-Err "missing falsification fixture $id" }
}
$blockers = @($d.open_blockers_before_final_excel_match_claim)
foreach ($id in @("BLK-W048-EXCEL-ROOT","BLK-W048-EXCEL-VERSION")) {
  if (-not ($blockers | Where-Object { $_ -match $id })) { Add-Err "missing blocker $id" }
}
if ($errors.Count -gt 0) { Write-Host "w048 iterative profile decision FAILED"; $errors | ForEach-Object { Write-Host "ERROR: $_" }; exit 1 }
Write-Host "w048 iterative profile decision ok"
