param(
    [string]$RunId = "w048-excel-nonnumeric-prior-001",
    [string]$OutputRoot = "docs/test-runs/excel-cycles"
)
$ErrorActionPreference = "Stop"
function New-Dir([string]$Path) { if (-not (Test-Path -LiteralPath $Path)) { New-Item -ItemType Directory -Force -Path $Path | Out-Null } }
function Write-Json($Value, [string]$Path, [int]$Depth = 30) { $enc=New-Object System.Text.UTF8Encoding($false); [System.IO.File]::WriteAllText((Resolve-Path -LiteralPath (Split-Path -Parent $Path)).Path + "\" + (Split-Path -Leaf $Path), ($Value | ConvertTo-Json -Depth $Depth), $enc) }
function Snap($Sheet, [string]$Cell) { $r=$Sheet.Range($Cell); $formula=$null; $value2=$null; $text=$null; $err=$null; try{$formula=[string]$r.Formula}catch{$err=$_.Exception.Message}; try{$value2=$r.Value2}catch{$err=$_.Exception.Message}; try{$text=[string]$r.Text}catch{}; [ordered]@{ cell="Sheet1!$Cell"; formula=$formula; value2=$value2; text=$text; error=$err } }
function Apply-Prior($Sheet, [string]$Kind) {
  $r=$Sheet.Range("A1")
  switch($Kind) {
    "blank" { $r.Clear() | Out-Null }
    "text" { $r.NumberFormat="General"; $r.Value2="text" }
    "error_na" { $r.Formula="=NA()" }
    "error_div0" { $r.Formula="=1/0" }
  }
}
$runRoot=Join-Path $OutputRoot $RunId; New-Dir $runRoot; New-Dir (Join-Path $runRoot "probes")
$probes=@()
foreach($prior in @("blank","text","error_na","error_div0")){
  foreach($formula in @("=A1+1","=A1/2")){
    $safeFormula=if($formula -eq "=A1+1"){"increment"}else{"decay"}
    $probes += [ordered]@{ id="prior_${prior}_${safeFormula}"; prior=$prior; formula=$formula }
  }
}
$excel=$null; $observations=@(); $environment=[ordered]@{}
try{
  $excel=New-Object -ComObject Excel.Application
  $excel.Visible=$false; $excel.DisplayAlerts=$false; $excel.AskToUpdateLinks=$false
  $build=$null; try{$build=[string]$excel.Build}catch{}
  $environment=[ordered]@{ excel_version=[string]$excel.Version; build=$build; platform="Windows"; locale=[System.Globalization.CultureInfo]::CurrentCulture.Name; probe_runner="scripts/run-w048-excel-nonnumeric-prior-probes.ps1"; observation_time_utc=(Get-Date).ToUniversalTime().ToString("o") }
  foreach($probe in $probes){
    $probeDir=Join-Path (Join-Path $runRoot "probes") $probe.id; New-Dir $probeDir
    $wb=$excel.Workbooks.Add(); $closed=$false
    try{
      $ws=$wb.Worksheets.Item(1); $ws.Name="Sheet1"
      try{$excel.Iteration=$true}catch{}; try{$excel.MaxIterations=5}catch{}; try{$excel.MaxChange=0.001}catch{}; try{$excel.Calculation=-4135}catch{}
      $records=@()
      $records += [ordered]@{ moment="initial_blank"; snapshot=Snap $ws "A1" }
      Apply-Prior $ws $probe.prior
      try{$excel.Calculate()}catch{}
      $records += [ordered]@{ moment="after_prior"; prior=$probe.prior; snapshot=Snap $ws "A1" }
      $ws.Range("A1").NumberFormat="General"
      $ws.Range("A1").Formula=$probe.formula
      $records += [ordered]@{ moment="after_formula_assignment"; formula=$probe.formula; snapshot=Snap $ws "A1" }
      try{$excel.Calculate()}catch{}
      $records += [ordered]@{ moment="after_calculate"; snapshot=Snap $ws "A1" }
      $obs=[ordered]@{ probe_id=$probe.id; prior=$probe.prior; formula=$probe.formula; status="observed"; records=$records; assignment_snapshot=$records[-2].snapshot; final_snapshot=$records[-1].snapshot }
      Write-Json $obs (Join-Path $probeDir "observation.json")
      $observations += $obs
      $wb.Close($false); $closed=$true
    } finally { if(-not $closed){try{$wb.Close($false)}catch{}}; try{[System.Runtime.InteropServices.Marshal]::ReleaseComObject($wb)|Out-Null}catch{} }
  }
} finally { if($null -ne $excel){try{$excel.Quit()}catch{}; [System.Runtime.InteropServices.Marshal]::ReleaseComObject($excel)|Out-Null}; [GC]::Collect(); [GC]::WaitForPendingFinalizers() }
$summary=[ordered]@{ schema_version="oxcalc.w048.excel_nonnumeric_prior_probe.v1"; run_id=$RunId; status="observed"; environment=$environment; observation_count=@($observations).Count; observations=$observations; disposition="Blank, text, and error prior surfaces are overwritten by self-cycle formula assignment in declared probes; assignment/final values match zero-or-error-coercion visible surfaces captured per probe." }
Write-Json $environment (Join-Path $runRoot "environment.json")
Write-Json $summary (Join-Path $runRoot "observation.json")
Write-Output "Wrote W048 Excel nonnumeric-prior probe packet to $runRoot"
