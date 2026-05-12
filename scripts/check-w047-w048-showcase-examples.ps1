param(
  [string]$Path = "docs/showcase/assets/w047_w048_showcase_examples.json"
)
$ErrorActionPreference = "Stop"
function Fail([string]$Message) { Write-Error $Message; exit 1 }
if (-not (Test-Path -LiteralPath $Path)) { Fail "missing $Path" }
$s = Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json
if ($s.schema_version -ne "oxcalc.showcase.w047_w048_examples.v1") { Fail "unexpected schema_version" }
$examples = @($s.examples)
foreach ($id in @("ctro_target_switch_downstream_publish", "ctro_cycle_reject_then_release_reentry", "excel_iterative_chain_order", "w048_conformance_single_host_scope")) {
  if (-not ($examples | Where-Object { $_.id -eq $id })) { Fail "missing example $id" }
}
$ctro = $examples | Where-Object { $_.id -eq "ctro_target_switch_downstream_publish" } | Select-Object -First 1
if ($ctro.observed.result_state -ne "published") { Fail "ctro target switch should publish" }
if (($ctro.observed.value_updates.'3' -as [string]) -ne "7") { Fail "ctro target switch owner value mismatch" }
if (($ctro.observed.value_updates.'5' -as [string]) -ne "8") { Fail "ctro target switch downstream value mismatch" }
if (@($ctro.observed.dependency_shape_updates).Count -ne 2) { Fail "ctro target switch should carry two shape updates" }
$cycle = $examples | Where-Object { $_.id -eq "ctro_cycle_reject_then_release_reentry" } | Select-Object -First 1
if ($cycle.observed.initial_result_state -ne "rejected") { Fail "cycle example should initially reject" }
if ($cycle.observed.initial_reject_kind -ne "SyntheticCycleReject") { Fail "cycle example reject kind mismatch" }
if (-not [bool]$cycle.observed.initial_publication_bundle_is_null) { Fail "cycle rejection should have no publication bundle" }
if ($cycle.observed.post_edit_result_state -ne "published") { Fail "cycle release/re-entry should publish" }
if (($cycle.observed.post_edit_value_updates.'2' -as [string]) -ne "10") { Fail "cycle release owner value mismatch" }
if (($cycle.observed.post_edit_value_updates.'4' -as [string]) -ne "11") { Fail "cycle release downstream value mismatch" }
$iter = $examples | Where-Object { $_.id -eq "excel_iterative_chain_order" } | Select-Object -First 1
if (($iter.observed.two_node_updates.'2' -as [string]) -ne "11") { Fail "two-node A1 value mismatch" }
if (($iter.observed.two_node_updates.'3' -as [string]) -ne "22") { Fail "two-node B1 value mismatch" }
if (($iter.observed.fraction_updates.'2' -as [string]) -ne "0.33333333333333331") { Fail "fraction precision mismatch" }
$scope = $examples | Where-Object { $_.id -eq "w048_conformance_single_host_scope" } | Select-Object -First 1
if ($scope.observed.status -ne "passed_single_host_scoped") { Fail "conformance status mismatch" }
if ($scope.observed.integration_completeness -ne "integrated_single_host") { Fail "integration status mismatch" }
Write-Output "w047/w048 showcase examples ok: examples=$($examples.Count) path=$Path"
