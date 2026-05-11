param(
    [string]$RunId = "w048-excel-root-report-001",
    [string]$OutputRoot = "docs/test-runs/excel-cycles"
)

$ErrorActionPreference = "Stop"

function New-Dir([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path)) { New-Item -ItemType Directory -Force -Path $Path | Out-Null }
}
function Write-Json($Value, [string]$Path, [int]$Depth = 30) {
    $encoding = New-Object System.Text.UTF8Encoding($false)
    [System.IO.File]::WriteAllText((Resolve-Path -LiteralPath (Split-Path -Parent $Path)).Path + "\" + (Split-Path -Leaf $Path), ($Value | ConvertTo-Json -Depth $Depth), $encoding)
}
function Get-CircularAddress($Excel) {
    try {
        $cr = $Excel.CircularReference
        if ($null -eq $cr) { return [ordered]@{ status = "null"; address = $null; error = $null } }
        return [ordered]@{ status = "range"; address = ($cr.Worksheet.Name + "!" + $cr.Address($false, $false)); error = $null }
    } catch {
        return [ordered]@{ status = "error"; address = $null; error = $_.Exception.Message }
    }
}
function Get-CellSnapshot($Sheet, [string[]]$Cells) {
    $rows = @()
    foreach ($cell in $Cells) {
        $range = $Sheet.Range($cell)
        $formula = $null
        $value2 = $null
        $text = $null
        try { $formula = [string]$range.Formula } catch {}
        try { $value2 = $range.Value2 } catch {}
        try { $text = [string]$range.Text } catch {}
        $rows += [ordered]@{
            cell = "Sheet1!$cell"
            formula = $formula
            value2 = $value2
            text = $text
        }
    }
    return $rows
}

$runRoot = Join-Path $OutputRoot $RunId
New-Dir $runRoot

$probes = @(
    [ordered]@{ id="root_self_no_iteration"; iteration=$false; cells=@("A1"); edits=@(@{target="A1"; formula="=A1+1"}) },
    [ordered]@{ id="root_self_iteration"; iteration=$true; cells=@("A1"); edits=@(@{target="A1"; formula="=A1+1"}) },
    [ordered]@{ id="root_two_node_ab"; iteration=$false; cells=@("A1","B1"); edits=@(@{target="A1"; formula="=B1+1"}, @{target="B1"; formula="=A1+1"}) },
    [ordered]@{ id="root_two_node_ba"; iteration=$false; cells=@("A1","B1"); edits=@(@{target="B1"; formula="=A1+1"}, @{target="A1"; formula="=B1+1"}) },
    [ordered]@{ id="root_three_node"; iteration=$false; cells=@("A1","B1","C1"); edits=@(@{target="A1"; formula="=B1+1"}, @{target="B1"; formula="=C1+1"}, @{target="C1"; formula="=A1+1"}) }
)

$excel = $null
$observations = @()
$environment = [ordered]@{}
try {
    $excel = New-Object -ComObject Excel.Application
    $excel.Visible = $false
    $excel.DisplayAlerts = $false
    $excel.AskToUpdateLinks = $false
    $excelBuild = $null
    try { $excelBuild = [string]$excel.Build } catch {}
    $environment = [ordered]@{
        excel_version = [string]$excel.Version
        build = $excelBuild
        platform = "Windows"
        locale = [System.Globalization.CultureInfo]::CurrentCulture.Name
        probe_runner = "scripts/run-w048-excel-root-report-probes.ps1"
        observation_time_utc = (Get-Date).ToUniversalTime().ToString("o")
        display_alerts = $false
        visible = $false
        note = "Targets Application.CircularReference report-cell/root behavior using COM without UI automation."
    }

    foreach ($probe in $probes) {
        $probeDir = Join-Path (Join-Path $runRoot "probes") $probe.id
        New-Dir $probeDir
        $wb = $excel.Workbooks.Add()
        $closed = $false
        try {
            $ws = $wb.Worksheets.Item(1)
            $ws.Name = "Sheet1"
            try { $excel.Iteration = [bool]$probe.iteration } catch {}
            try { $excel.MaxIterations = 5 } catch {}
            try { $excel.MaxChange = 0.001 } catch {}
            $records = @()
            $records += [ordered]@{ moment="initial"; circular_reference=Get-CircularAddress $excel; cells=Get-CellSnapshot $ws $probe.cells }
            $editIndex = 0
            foreach ($edit in $probe.edits) {
                $editIndex++
                $ws.Range($edit.target).Formula = $edit.formula
                $records += [ordered]@{ moment="after_edit_$editIndex"; target=$edit.target; formula=$edit.formula; circular_reference=Get-CircularAddress $excel; cells=Get-CellSnapshot $ws $probe.cells }
            }
            foreach ($command in @("worksheet_calculate", "application_calculate", "calculate_full", "calculate_full_rebuild")) {
                try {
                    switch ($command) {
                        "worksheet_calculate" { $ws.Calculate() }
                        "application_calculate" { $excel.Calculate() }
                        "calculate_full" { $excel.CalculateFull() }
                        "calculate_full_rebuild" { $excel.CalculateFullRebuild() }
                    }
                } catch {}
                $records += [ordered]@{ moment="after_$command"; circular_reference=Get-CircularAddress $excel; cells=Get-CellSnapshot $ws $probe.cells }
            }
            $obs = [ordered]@{
                probe_id = $probe.id
                iteration_enabled = [bool]$probe.iteration
                status = "observed"
                records = $records
                reported_addresses = @($records | ForEach-Object { $_.circular_reference.address } | Where-Object { $_ })
                null_report_count = @($records | Where-Object { $_.circular_reference.status -eq "null" }).Count
            }
            Write-Json $obs (Join-Path $probeDir "observation.json")
            $observations += $obs
            $wb.Close($false)
            $closed = $true
        } finally {
            if (-not $closed) { try { $wb.Close($false) } catch {} }
            [System.Runtime.InteropServices.Marshal]::ReleaseComObject($wb) | Out-Null
        }
    }
} finally {
    if ($null -ne $excel) {
        try { $excel.Quit() } catch {}
        [System.Runtime.InteropServices.Marshal]::ReleaseComObject($excel) | Out-Null
    }
    [GC]::Collect()
    [GC]::WaitForPendingFinalizers()
}

$summary = [ordered]@{
    schema_version = "oxcalc.w048.excel_root_report_probe.v1"
    run_id = $RunId
    status = "observed_object_model_null_for_all_variants"
    environment = $environment
    observation_count = @($observations).Count
    observations = $observations
    blocker_disposition = "BLK-W048-EXCEL-ROOT remains open: this COM packet did not produce a non-null Application.CircularReference range. UI warning capture or another public object-model route is still required for exact report-cell/root behavior."
}
Write-Json $environment (Join-Path $runRoot "environment.json")
Write-Json $summary (Join-Path $runRoot "observation.json")
Write-Output "Wrote W048 Excel root/report probe packet to $runRoot"
