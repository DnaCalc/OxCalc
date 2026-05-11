param(
  [string]$IssuesPath = ".beads/issues.jsonl"
)
$ErrorActionPreference = "Stop"
$errors = New-Object System.Collections.Generic.List[string]
function Add-Err([string]$m) { $errors.Add($m) | Out-Null }
function Read-Json([string]$p) { if (-not (Test-Path $p)) { throw "Missing JSON: $p" }; Get-Content $p -Raw | ConvertFrom-Json }
$issues = @{}
Get-Content $IssuesPath | ForEach-Object { if ($_ -and $_.Trim()) { $o = $_ | ConvertFrom-Json; $issues[$o.id] = $o } }
if ($issues["calc-zci1"].status -ne "open") { Add-Err "reopened W048 parent calc-zci1 should be open during active reopened scope" }
if ($issues["calc-zci1.9"].status -ne "closed") { Add-Err "calc-zci1.9 should be closed before later W048 work proceeds" }
foreach ($id in @("calc-zci1.10","calc-zci1.11","calc-zci1.12","calc-zci1.13","calc-zci1.14","calc-zci1.15","calc-zci1.17","calc-zci1.18","calc-zci1.20")) {
  if ($issues[$id].status -ne "closed") { Add-Err "$id should be closed after reopened W048 child-bead execution" }
}
foreach ($id in @("calc-zci1.16","calc-zci1.19")) {
  if ($issues[$id].status -ne "blocked") { Add-Err "$id should be blocked pending external Excel evidence or user scope acceptance" }
}
$workset = Get-Content "docs/worksets/W048_CIRCULAR_DEPENDENCY_CALCULATION_PROCESSING.md" -Raw
if ($workset -notmatch 'Status:\s+`reopened_in_progress`') { Add-Err "W048 workset must be reopened_in_progress" }
if ($workset -notmatch "bit-exact iterative") { Add-Err "W048 workset must name bit-exact iterative target" }
if ($workset -match "iterative-profile selection or explicit deferral") { Add-Err "stale deferral closure clause remains" }
if ($workset -notmatch "BLK-W048-EXCEL-ROOT") { Add-Err "W048 workset must name active root/report blocker" }
if ($workset -notmatch "BLK-W048-EXCEL-VERSION") { Add-Err "W048 workset must name active version-repeat blocker" }
$packet = Get-Content "docs/spec/core-engine/w048-cycles/W048_REOPEN_SCOPE_AUDIT_AND_REPAIR_PLAN.md" -Raw
if ($packet -notmatch "Fresh-Eyes Review") { Add-Err "reopen packet must include fresh-eyes review" }
if ($packet -notmatch "PowerShell/Rust/C#|PowerShell, Rust, or C#") { Add-Err "reopen packet must require non-Python tooling" }
$audit = Get-Content "docs/spec/core-engine/w048-cycles/W048_WHOLE_WORKSET_FRESH_EYES_AUDIT.md" -Raw
if ($audit -notmatch "BLK-W048-EXCEL-ROOT") { Add-Err "whole-workset audit must preserve root/report blocker" }
if ($audit -notmatch "BLK-W048-EXCEL-VERSION") { Add-Err "whole-workset audit must preserve version-repeat blocker" }
if ($errors.Count -gt 0) { Write-Host "w048 reopened closure-state audit FAILED"; $errors | ForEach-Object { Write-Host "ERROR: $_" }; exit 1 }
Write-Host "w048 reopened closure-state audit ok"
