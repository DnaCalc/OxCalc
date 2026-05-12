param(
  [string]$IssuesPath = ".beads/issues.jsonl"
)
$ErrorActionPreference = "Stop"
$errors = New-Object System.Collections.Generic.List[string]
function Add-Err([string]$m) { $errors.Add($m) | Out-Null }
function Read-Json([string]$p) { if (-not (Test-Path $p)) { throw "Missing JSON: $p" }; Get-Content $p -Raw | ConvertFrom-Json }
$issues = @{}
Get-Content $IssuesPath | ForEach-Object { if ($_ -and $_.Trim()) { $o = $_ | ConvertFrom-Json; $issues[$o.id] = $o } }
if ($issues["calc-zci1"].status -ne "closed") { Add-Err "W048 parent calc-zci1 should be closed after accepted single-host final audit" }
if ($issues["calc-zci1.9"].status -ne "closed") { Add-Err "calc-zci1.9 should be closed before later W048 work proceeds" }
foreach ($id in @("calc-zci1.10","calc-zci1.11","calc-zci1.12","calc-zci1.13","calc-zci1.14","calc-zci1.15","calc-zci1.17","calc-zci1.18","calc-zci1.20")) {
  if ($issues[$id].status -ne "closed") { Add-Err "$id should be closed after reopened W048 child-bead execution" }
}
if ($issues["calc-zci1.16"].status -ne "closed") { Add-Err "calc-zci1.16 should be closed after worksheet-scoped root/report evidence" }
if ($issues["calc-zci1.19"].status -ne "closed") { Add-Err "calc-zci1.19 should be closed after explicit single-host scope acceptance" }
$workset = Get-Content "docs/worksets/W048_CIRCULAR_DEPENDENCY_CALCULATION_PROCESSING.md" -Raw
if ($workset -notmatch 'Status:\s+`closed_single_host_scope`') { Add-Err "W048 workset must be closed_single_host_scope" }
if ($workset -notmatch "bit-exact iterative") { Add-Err "W048 workset must name bit-exact iterative target" }
if ($workset -match "iterative-profile selection or explicit deferral") { Add-Err "stale deferral closure clause remains" }
if ($workset -notmatch "BLK-W048-EXCEL-ROOT") { Add-Err "W048 workset must name cleared root/report lane" }
if ($workset -notmatch "w048-excel-root-report-002") { Add-Err "W048 workset must cite root/report repair evidence" }
if ($workset -notmatch "single-host") { Add-Err "W048 workset must document accepted single-host scope" }
if ($workset -notmatch "BLK-W048-EXCEL-VERSION") { Add-Err "W048 workset must name version-repeat limitation" }
$packet = Get-Content "docs/spec/core-engine/w048-cycles/W048_REOPEN_SCOPE_AUDIT_AND_REPAIR_PLAN.md" -Raw
if ($packet -notmatch "Fresh-Eyes Review") { Add-Err "reopen packet must include fresh-eyes review" }
if ($packet -notmatch "PowerShell/Rust/C#|PowerShell, Rust, or C#") { Add-Err "reopen packet must require non-Python tooling" }
$audit = Get-Content "docs/spec/core-engine/w048-cycles/W048_WHOLE_WORKSET_FRESH_EYES_AUDIT.md" -Raw
if ($audit -notmatch "w048-excel-root-report-002") { Add-Err "whole-workset audit must preserve root/report repair evidence" }
$finalAudit = Get-Content "docs/spec/core-engine/w048-cycles/W048_SINGLE_HOST_SCOPE_ACCEPTANCE_AND_FINAL_AUDIT.md" -Raw
if ($finalAudit -notmatch "single observed Excel host/version") { Add-Err "final audit must document single-host scope" }
if ($finalAudit -notmatch "No broad cross-version Excel compatibility claim") { Add-Err "final audit must document cross-version limitation" }
if ($errors.Count -gt 0) { Write-Host "w048 single-host closure audit FAILED"; $errors | ForEach-Object { Write-Host "ERROR: $_" }; exit 1 }
Write-Host "w048 single-host closure audit ok"
