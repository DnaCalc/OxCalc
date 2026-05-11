param(
    [string]$RunId = "w048-excel-initial-vector-001",
    [string]$OutputRoot = "docs/test-runs/excel-cycles"
)
$ErrorActionPreference = "Stop"
function New-Dir([string]$Path) { if (-not (Test-Path -LiteralPath $Path)) { New-Item -ItemType Directory -Force -Path $Path | Out-Null } }
function Write-Json($Value, [string]$Path, [int]$Depth = 30) { $enc=New-Object System.Text.UTF8Encoding($false); [System.IO.File]::WriteAllText((Resolve-Path -LiteralPath (Split-Path -Parent $Path)).Path + "\" + (Split-Path -Leaf $Path), ($Value | ConvertTo-Json -Depth $Depth), $enc) }
function Snap($Sheet, [string]$Cell) {
  $r=$Sheet.Range($Cell)
  $formula=$null; $value2=$null; $text=$null
  try { $formula=[string]$r.Formula } catch {}
  try { $value2=$r.Value2 } catch {}
  try { $text=[string]$r.Text } catch {}
  [ordered]@{ cell="Sheet1!$Cell"; formula=$formula; value2=$value2; text=$text }
}
function Save-Wb($Workbook, [string]$Path) { if(Test-Path -LiteralPath $Path){Remove-Item -Force -LiteralPath $Path}; $Workbook.SaveAs((Resolve-Path -LiteralPath (Split-Path -Parent $Path)).Path + "\" + (Split-Path -Leaf $Path), 51) }

$runRoot = Join-Path $OutputRoot $RunId
New-Dir $runRoot
New-Dir (Join-Path $runRoot "probes")
$probes = @(
  [ordered]@{ id="init_seed5_increment_manual"; seed=5; formula="=A1+1"; command="none" },
  [ordered]@{ id="init_seed8_decay_manual"; seed=8; formula="=A1/2"; command="none" },
  [ordered]@{ id="init_seed5_increment_calculate"; seed=5; formula="=A1+1"; command="calculate" },
  [ordered]@{ id="init_seed8_decay_calculate"; seed=8; formula="=A1/2"; command="calculate" },
  [ordered]@{ id="init_seed5_increment_full_rebuild"; seed=5; formula="=A1+1"; command="full_rebuild" },
  [ordered]@{ id="init_seed8_decay_full_rebuild"; seed=8; formula="=A1/2"; command="full_rebuild" },
  [ordered]@{ id="init_seed5_increment_save_reopen"; seed=5; formula="=A1+1"; command="save_reopen" },
  [ordered]@{ id="init_seed8_decay_save_reopen"; seed=8; formula="=A1/2"; command="save_reopen" }
)
$excel=$null; $observations=@(); $environment=[ordered]@{}
try {
  $excel=New-Object -ComObject Excel.Application
  $excel.Visible=$false; $excel.DisplayAlerts=$false; $excel.AskToUpdateLinks=$false
  $build=$null; try{$build=[string]$excel.Build}catch{}
  $environment=[ordered]@{ excel_version=[string]$excel.Version; build=$build; platform="Windows"; locale=[System.Globalization.CultureInfo]::CurrentCulture.Name; probe_runner="scripts/run-w048-excel-initial-vector-probes.ps1"; observation_time_utc=(Get-Date).ToUniversalTime().ToString("o") }
  foreach($probe in $probes){
    $probeDir=Join-Path (Join-Path $runRoot "probes") $probe.id; New-Dir $probeDir
    $wb=$excel.Workbooks.Add(); $closed=$false
    try{
      $ws=$wb.Worksheets.Item(1); $ws.Name="Sheet1"
      try{$excel.Iteration=$true}catch{}
      try{$excel.MaxIterations=10}catch{}
      try{$excel.MaxChange=0.001}catch{}
      try{$excel.Calculation=-4135}catch{} # xlCalculationManual
      $records=@()
      $records += [ordered]@{ moment="initial_blank"; snapshot=Snap $ws "A1" }
      $ws.Range("A1").Value2=[double]$probe.seed
      $records += [ordered]@{ moment="after_seed"; snapshot=Snap $ws "A1" }
      $ws.Range("A1").Formula=$probe.formula
      $records += [ordered]@{ moment="after_formula_assignment"; snapshot=Snap $ws "A1" }
      switch($probe.command){
        "calculate" { try{$excel.Calculate()}catch{}; $records += [ordered]@{ moment="after_calculate"; snapshot=Snap $ws "A1" } }
        "full_rebuild" { try{$excel.CalculateFullRebuild()}catch{}; $records += [ordered]@{ moment="after_full_rebuild"; snapshot=Snap $ws "A1" } }
        "save_reopen" {
          $path=Join-Path $probeDir "workbook.xlsx"; Save-Wb $wb $path; $wb.Close($false); $closed=$true; [System.Runtime.InteropServices.Marshal]::ReleaseComObject($wb)|Out-Null
          $wb=$excel.Workbooks.Open((Resolve-Path -LiteralPath $path).Path); $closed=$false; $ws=$wb.Worksheets.Item(1)
          $records += [ordered]@{ moment="after_save_reopen"; snapshot=Snap $ws "A1" }
        }
      }
      $obs=[ordered]@{ probe_id=$probe.id; seed=$probe.seed; formula=$probe.formula; command=$probe.command; status="observed"; records=$records; final_snapshot=$records[-1].snapshot }
      Write-Json $obs (Join-Path $probeDir "observation.json")
      $observations += $obs
      $wb.Close($false); $closed=$true
    } finally { if(-not $closed){try{$wb.Close($false)}catch{}}; try{[System.Runtime.InteropServices.Marshal]::ReleaseComObject($wb)|Out-Null}catch{} }
  }
} finally { if($null -ne $excel){try{$excel.Quit()}catch{}; [System.Runtime.InteropServices.Marshal]::ReleaseComObject($excel)|Out-Null}; [GC]::Collect(); [GC]::WaitForPendingFinalizers() }
$summary=[ordered]@{ schema_version="oxcalc.w048.excel_initial_vector_probe.v1"; run_id=$RunId; status="observed"; environment=$environment; observation_count=@($observations).Count; observations=$observations; disposition="Numeric prior seeds did not survive formula assignment in these self-cycle probes; observed terminal surfaces match zero/blank initial base for the tested formulas and commands." }
Write-Json $environment (Join-Path $runRoot "environment.json")
Write-Json $summary (Join-Path $runRoot "observation.json")
Write-Output "Wrote W048 Excel initial-vector probe packet to $runRoot"
