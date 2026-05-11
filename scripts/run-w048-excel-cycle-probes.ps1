param(
    [string]$RunId = "w048-excel-cycles-001",
    [string]$OutputRoot = "docs/test-runs/excel-cycles",
    [ValidateSet("core")]
    [string]$ProbeSet = "core"
)

$ErrorActionPreference = "Stop"

function New-Dir([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path)) {
        New-Item -ItemType Directory -Force -Path $Path | Out-Null
    }
}

function Write-Json($Value, [string]$Path, [int]$Depth = 20) {
    $Value | ConvertTo-Json -Depth $Depth | Set-Content -Encoding UTF8 -Path $Path
}

function To-CellSnapshot($Sheet, [string[]]$Cells) {
    $rows = @()
    foreach ($addr in $Cells) {
        $range = $Sheet.Range($addr)
        $value = $null
        $text = $null
        $formula = $null
        $errorText = $null
        try { $value = $range.Value2 } catch { $errorText = $_.Exception.Message }
        try { $text = [string]$range.Text } catch {}
        try { $formula = [string]$range.Formula } catch {}
        $rows += [ordered]@{
            cell = "Sheet1!$addr"
            formula = $formula
            value2 = $value
            text = $text
            error = $errorText
        }
    }
    return $rows
}

function Get-CircularReferenceAddress($Excel) {
    try {
        $cr = $Excel.CircularReference
        if ($null -eq $cr) { return $null }
        return $cr.Worksheet.Name + "!" + $cr.Address($false, $false)
    } catch {
        return $null
    }
}

function Save-Workbook($Workbook, [string]$Path) {
    if (Test-Path -LiteralPath $Path) { Remove-Item -Force -LiteralPath $Path }
    $Workbook.SaveAs((Resolve-Path -LiteralPath (Split-Path -Parent $Path)).Path + "\" + (Split-Path -Leaf $Path), 51)
}

function Get-CalcChainSnapshot([string]$WorkbookPath) {
    $result = [ordered]@{
        present = $false
        entries = @()
        raw_xml = $null
        note = $null
    }
    try {
        Add-Type -AssemblyName System.IO.Compression.FileSystem -ErrorAction SilentlyContinue
        $zip = [System.IO.Compression.ZipFile]::OpenRead((Resolve-Path -LiteralPath $WorkbookPath).Path)
        try {
            $entry = $zip.Entries | Where-Object { $_.FullName -eq "xl/calcChain.xml" } | Select-Object -First 1
            if ($null -ne $entry) {
                $reader = New-Object System.IO.StreamReader($entry.Open())
                try { $xml = $reader.ReadToEnd() } finally { $reader.Close() }
                $result.present = $true
                $result.raw_xml = $xml
                $matches = [regex]::Matches($xml, '<c[^>]*r="([^"]+)"[^>]*/?>')
                foreach ($m in $matches) { $result.entries += $m.Groups[1].Value }
            } else {
                $result.note = "xl/calcChain.xml absent in saved workbook"
            }
        } finally { $zip.Dispose() }
    } catch {
        $result.note = $_.Exception.Message
    }
    return $result
}

$probes = @(
    [ordered]@{ id="excel_struct_self_001"; cycle_kind="structural"; iteration=$false; max_iterations=100; max_change=0.001; cells=@("A1"); steps=@(@{op="set_formula"; target="A1"; formula="=A1+1"}); questions=@("direct self-cycle report cell", "non-iterative displayed value") },
    [ordered]@{ id="excel_struct_self_prior_001"; cycle_kind="structural"; iteration=$false; max_iterations=100; max_change=0.001; cells=@("A1"); steps=@(@{op="set_value"; target="A1"; value=5}, @{op="calculate_full_rebuild"}, @{op="set_formula"; target="A1"; formula="=A1+1"}); questions=@("prior value retention", "direct self-cycle report cell") },
    [ordered]@{ id="excel_struct_two_node_001"; cycle_kind="structural"; iteration=$false; max_iterations=100; max_change=0.001; cells=@("A1","B1"); steps=@(@{op="set_formula"; target="A1"; formula="=B1+1"}, @{op="set_formula"; target="B1"; formula="=A1+1"}); questions=@("reported root/order", "non-iteration values") },
    [ordered]@{ id="excel_struct_three_node_001"; cycle_kind="structural"; iteration=$false; max_iterations=100; max_change=0.001; cells=@("A1","B1","C1"); steps=@(@{op="set_formula"; target="A1"; formula="=B1+1"}, @{op="set_formula"; target="B1"; formula="=C1+1"}, @{op="set_formula"; target="C1"; formula="=A1+1"}); questions=@("SCC report order", "chain order") },
    [ordered]@{ id="excel_struct_guarded_activation_001"; cycle_kind="structural"; iteration=$false; max_iterations=100; max_change=0.001; cells=@("A1","B1"); steps=@(@{op="set_value"; target="B1"; value=0}, @{op="set_formula"; target="A1"; formula="=IF(B1=0,0,A1+1)"}, @{op="calculate_full_rebuild"}, @{op="set_value"; target="B1"; value=1}); questions=@("guarded activation warning timing", "last-successful behavior") },
    [ordered]@{ id="excel_iter_self_increment_001"; cycle_kind="structural"; iteration=$true; max_iterations=5; max_change=0.001; cells=@("A1"); steps=@(@{op="set_value"; target="A1"; value=0}, @{op="calculate_full_rebuild"}, @{op="set_formula"; target="A1"; formula="=A1+1"}); questions=@("max-iteration terminal", "starting value") },
    [ordered]@{ id="excel_iter_two_node_order_001"; cycle_kind="structural"; iteration=$true; max_iterations=1; max_change=0.001; cells=@("A1","B1"); steps=@(@{op="set_value"; target="A1"; value=1}, @{op="set_value"; target="B1"; value=10}, @{op="calculate_full_rebuild"}, @{op="set_formula"; target="A1"; formula="=B1+1"}, @{op="set_formula"; target="B1"; formula="=A1*2"}); questions=@("Jacobi versus sequential update", "member order") },
    [ordered]@{ id="excel_chain_edit_order_ab_001"; cycle_kind="structural"; iteration=$false; max_iterations=100; max_change=0.001; cells=@("A1","B1"); steps=@(@{op="set_formula"; target="A1"; formula="=B1+1"}, @{op="set_formula"; target="B1"; formula="=A1+1"}); questions=@("edit order AB root") },
    [ordered]@{ id="excel_chain_edit_order_ba_001"; cycle_kind="structural"; iteration=$false; max_iterations=100; max_change=0.001; cells=@("A1","B1"); steps=@(@{op="set_formula"; target="B1"; formula="=A1+1"}, @{op="set_formula"; target="A1"; formula="=B1+1"}); questions=@("edit order BA root") },
    [ordered]@{ id="excel_ctro_indirect_self_001"; cycle_kind="dynamic_reference"; iteration=$false; max_iterations=100; max_change=0.001; cells=@("A1","B1"); steps=@(@{op="set_value"; target="B1"; value="A1"}, @{op="set_formula"; target="A1"; formula="=INDIRECT(B1)+1"}); questions=@("dynamic self-cycle behavior", "CTRO analog root") },
    [ordered]@{ id="excel_ctro_indirect_release_001"; cycle_kind="dynamic_reference"; iteration=$false; max_iterations=100; max_change=0.001; cells=@("A1","B1","C1"); steps=@(@{op="set_value"; target="C1"; value=10}, @{op="set_value"; target="B1"; value="A1"}, @{op="set_formula"; target="A1"; formula="=INDIRECT(B1)+1"}, @{op="calculate_full_rebuild"}, @{op="set_value"; target="B1"; value="C1"}); questions=@("dynamic cycle release", "re-entry timing") },
    [ordered]@{ id="excel_ctro_downstream_001"; cycle_kind="dynamic_reference"; iteration=$false; max_iterations=100; max_change=0.001; cells=@("A1","B1","C1","D1"); steps=@(@{op="set_value"; target="C1"; value=10}, @{op="set_value"; target="B1"; value="A1"}, @{op="set_formula"; target="A1"; formula="=INDIRECT(B1)+1"}, @{op="set_formula"; target="D1"; formula="=A1+1"}, @{op="calculate_full_rebuild"}, @{op="set_value"; target="B1"; value="C1"}); questions=@("downstream retained/stale/recomputed value", "release propagation") }
)

$runRoot = Join-Path $OutputRoot $RunId
New-Dir $runRoot
New-Dir (Join-Path $runRoot "probes")
$rawLogPath = Join-Path $runRoot "raw_com_log.jsonl"
if (Test-Path -LiteralPath $rawLogPath) { Remove-Item -Force -LiteralPath $rawLogPath }

$excel = $null
$observations = @()
$environment = [ordered]@{}
try {
    $excel = New-Object -ComObject Excel.Application
    $excel.Visible = $false
    $excel.DisplayAlerts = $false
    $excel.AskToUpdateLinks = $false
    $environment = [ordered]@{
        excel_version = [string]$excel.Version
        build = $null
        channel = "unknown"
        platform = "Windows"
        workbook_calculation_mode = "manual"
        application_calculation_mode = "manual"
        iteration_enabled = $false
        max_iterations = 100
        max_change = 0.001
        multi_threaded_calculation_enabled = $null
        thread_count = $null
        precision_as_displayed = $false
        calculate_before_save = $false
        locale = [System.Globalization.CultureInfo]::CurrentCulture.Name
        probe_runner = "scripts/run-w048-excel-cycle-probes.ps1"
        observation_time_utc = (Get-Date).ToUniversalTime().ToString("o")
    }
    try { $environment.build = [string]$excel.Build } catch {}
    try { $environment.multi_threaded_calculation_enabled = [bool]$excel.MultiThreadedCalculation.Enabled } catch {}
    try { $environment.thread_count = [int]$excel.MultiThreadedCalculation.ThreadCount } catch {}
    try { $excel.Calculation = -4135 } catch { $environment.application_calculation_mode = "manual_requested_but_set_failed: $($_.Exception.Message)" }
    try { $excel.MultiThreadedCalculation.Enabled = $false } catch {}
    try { $environment.multi_threaded_calculation_enabled = [bool]$excel.MultiThreadedCalculation.Enabled } catch {}
    try { $environment.thread_count = [int]$excel.MultiThreadedCalculation.ThreadCount } catch {}

    foreach ($probe in $probes) {
        $probeDir = Join-Path (Join-Path $runRoot "probes") $probe.id
        New-Dir $probeDir
        try { $excel.Iteration = [bool]$probe.iteration } catch {}
        try { $excel.MaxIterations = [int]$probe.max_iterations } catch {}
        try { $excel.MaxChange = [double]$probe.max_change } catch {}
        $workbook = $excel.Workbooks.Add()
        try { $excel.Calculation = -4135 } catch {}
        try {
            try { $workbook.PrecisionAsDisplayed = $false } catch {}
            try { $workbook.CalculateBeforeSave = $false } catch {}
            $sheet = $workbook.Worksheets.Item(1)
            $sheet.Name = "Sheet1"
            $stepRecords = @()
            $workbookClosed = $false
            Save-Workbook $workbook (Join-Path $probeDir "workbook_before.xlsx")
            $stepIndex = 0
            foreach ($step in $probe.steps) {
                $stepIndex += 1
                $stepCells = @()
                $stepOp = $step["op"]
                if ($step.ContainsKey("target")) { $stepCells = @($step["target"]) } else { $stepCells = $probe.cells }
                $before = To-CellSnapshot $sheet $stepCells
                switch ($stepOp) {
                    "set_value" {
                        $targetRange = $sheet.Range($step["target"])
                        if ($step["value"] -is [string]) {
                            $targetRange.NumberFormat = "@"
                            $targetRange.Value2 = [string]$step["value"]
                        } else {
                            $targetRange.Value2 = [double]$step["value"]
                        }
                    }
                    "set_formula" { $sheet.Range($step["target"]).Formula = $step["formula"] }
                    "calculate" { $excel.Calculate() }
                    "calculate_full_rebuild" { $excel.CalculateFullRebuild() }
                    default { throw "Unknown step op $stepOp" }
                }
                if ($stepOp -ne "calculate" -and $stepOp -ne "calculate_full_rebuild") {
                    try { $excel.CalculateFullRebuild() } catch {}
                }
                $after = To-CellSnapshot $sheet $stepCells
                $targetField = $null
                $formulaField = $null
                $valueField = $null
                if ($step.ContainsKey("target")) { $targetField = "Sheet1!" + $step["target"] }
                if ($step.ContainsKey("formula")) { $formulaField = $step["formula"] }
                if ($step.ContainsKey("value")) { $valueField = $step["value"] }
                $calcCommand = "Application.CalculateFullRebuild"
                if ($stepOp -eq "calculate") { $calcCommand = "Application.Calculate" }
                $calcState = $null
                try { $calcState = [string]$excel.CalculationState } catch {}
                $record = [ordered]@{
                    probe_id = $probe.id
                    step_index = $stepIndex
                    operation = $stepOp
                    target = $targetField
                    formula = $formulaField
                    value = $valueField
                    before = $before
                    after = $after
                    calculation_command = $calcCommand
                    circular_reference = Get-CircularReferenceAddress $excel
                    calculation_state = $calcState
                }
                $stepRecords += $record
                ($record | ConvertTo-Json -Depth 20 -Compress) | Add-Content -Encoding UTF8 -Path $rawLogPath
            }
            try { $excel.CalculateFullRebuild() } catch {}
            Save-Workbook $workbook (Join-Path $probeDir "workbook_after.xlsx")
            $snapshots = @(To-CellSnapshot $sheet $probe.cells)
            $reported = @()
            $cr = Get-CircularReferenceAddress $excel
            if ($cr) { $reported += $cr }
            $workbook.Close($false)
            $workbookClosed = $true
            $chain = Get-CalcChainSnapshot (Join-Path $probeDir "workbook_after.xlsx")
            $rootHypothesis = "unknown"
            if ($reported.Count -gt 0) { $rootHypothesis = "observed_application_circular_reference" }
            $initialValueHypothesis = "unknown"
            if (($probe.id -like "*prior*") -or [bool]$probe.iteration) { $initialValueHypothesis = "published_prior_value_probe" }
            $obs = [ordered]@{
                probe_id = $probe.id
                run_id = $RunId
                status = "observed"
                cycle_kind = $probe.cycle_kind
                iteration_profile = [ordered]@{
                    enabled = [bool]$probe.iteration
                    max_iterations = [int]$probe.max_iterations
                    max_change = [double]$probe.max_change
                }
                questions = $probe.questions
                reported_cycle_cells = $reported
                cell_results = $snapshots
                calc_chain = $chain
                chain_sensitive = "unknown"
                root_hypothesis = $rootHypothesis
                update_model_hypothesis = "unknown"
                initial_value_hypothesis = $initialValueHypothesis
                operation_steps = $stepRecords
                notes = @("Generated by clean-room COM black-box probe harness; no internal Excel implementation evidence used.")
            }
            Write-Json $obs (Join-Path $probeDir "observation.json")
            Write-Json $snapshots (Join-Path $probeDir "cell_snapshots.json")
            Write-Json $chain (Join-Path $probeDir "calc_chain_snapshot.json")
            $observations += $obs
        } finally {
            if (-not $workbookClosed) {
                try { $workbook.Close($false) } catch {}
            }
            [System.Runtime.InteropServices.Marshal]::ReleaseComObject($workbook) | Out-Null
        }
    }
} catch {
    $environment = [ordered]@{
        excel_available = $false
        probe_runner = "scripts/run-w048-excel-cycle-probes.ps1"
        observation_time_utc = (Get-Date).ToUniversalTime().ToString("o")
        error = $_.Exception.Message
    }
    Write-Json $environment (Join-Path $runRoot "environment.json")
    throw
} finally {
    if ($null -ne $excel) {
        try { $excel.Quit() } catch {}
        [System.Runtime.InteropServices.Marshal]::ReleaseComObject($excel) | Out-Null
    }
    [GC]::Collect()
    [GC]::WaitForPendingFinalizers()
}

Write-Json $environment (Join-Path $runRoot "environment.json")
Write-Json ([ordered]@{ run_id=$RunId; probe_set=$ProbeSet; probe_count=$probes.Count; probes=$probes }) (Join-Path $runRoot "probe_plan.json")
Write-Json ([ordered]@{ run_id=$RunId; status="observed"; observation_count=$observations.Count; observations=$observations }) (Join-Path $runRoot "observation.json")

$notes = @"
# W048 Excel Cycle Probe Normalization Notes

Run id: $RunId

This packet was generated by `scripts/run-w048-excel-cycle-probes.ps1` through Excel COM automation. It is black-box evidence: the runner records workbook edits, calculation commands, `Application.CircularReference`, cell snapshots, saved workbooks, and saved `xl/calcChain.xml` presence/entries when present.

Scope of this packet:
1. structural self/two/three-node cycles;
2. prior-value and guarded activation probes;
3. first iterative probes for max-iteration and update-order discrimination;
4. edit-order chain/root probes;
5. INDIRECT/dynamic-reference probes for CTRO analog self-cycle, release, and downstream dependent behavior.

Interpretation limits:
1. observations are tied to the environment recorded in `environment.json`;
2. multi-threaded variants are intentionally not claimed by this packet;
3. `Application.CircularReference` exposes a reported cell but is not treated as internal algorithm evidence;
4. absence of `xl/calcChain.xml` is recorded as an artifact observation, not a claim that Excel has no calculation-chain state.
"@
$notes | Set-Content -Encoding UTF8 -Path (Join-Path $runRoot "normalization_notes.md")

Write-Output "Wrote W048 Excel cycle probe packet to $runRoot"
