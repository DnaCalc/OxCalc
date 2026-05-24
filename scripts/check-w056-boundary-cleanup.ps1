param(
    [string[]]$Roots = @("src", "docs")
)

$ErrorActionPreference = "Stop"

$pattern = "OxCalcTreeEnvironment|OxCalcTreeDocument|OxCalcTreeRecalcRequest|OxCalcTreeRecalcResult|OxCalcTreeRuntimeFacade|LiveOxCalcTreeBridge|OxCalcTreeBridge|TreeRecalcRequest|TreeRecalcResult|PreparedFormulaCatalog|PreparedFormulaReferenceCarrier"
$hits = & rg -n $pattern @Roots -S --no-messages

if ($LASTEXITCODE -eq 1) {
    Write-Host "W056 boundary cleanup guard passed."
    exit 0
}

if ($LASTEXITCODE -ne 0) {
    throw "rg failed while checking W056 boundary cleanup guard."
}

Write-Error "W056 boundary cleanup guard failed:`n$($hits -join [Environment]::NewLine)"
exit 1
