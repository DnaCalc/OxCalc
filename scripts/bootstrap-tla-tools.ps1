param(
    [string]$Version = "latest",
    [string]$DestinationDir = "tools/tla",
    [switch]$Force
)

$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true
$ProgressPreference = "SilentlyContinue"

$repoRoot = Split-Path -Parent $PSScriptRoot
$destinationRoot = Join-Path $repoRoot $DestinationDir
if (-not (Test-Path $destinationRoot)) {
    New-Item -ItemType Directory -Path $destinationRoot -Force | Out-Null
}

$jarPath = Join-Path $destinationRoot "tla2tools.jar"
$versionPath = Join-Path $destinationRoot "version.txt"

if (-not $Force -and (Test-Path $jarPath) -and (Test-Path $versionPath)) {
    $installedVersion = (Get-Content $versionPath -ErrorAction SilentlyContinue | Select-Object -First 1)
    if (-not [string]::IsNullOrWhiteSpace($installedVersion)) {
        Write-Host "TLA+ tools already bootstrapped at '$jarPath' ($installedVersion)."
        exit 0
    }
}

$headers = @{ "User-Agent" = "OxCalc-TLC-Bootstrap" }
if ($Version -eq "latest") {
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/tlaplus/tlaplus/releases/latest" -Headers $headers
}
else {
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/tlaplus/tlaplus/releases/tags/$Version" -Headers $headers
}

$asset = $release.assets | Where-Object { $_.name -eq "tla2tools.jar" } | Select-Object -First 1
if ($null -eq $asset) {
    throw "Could not find tla2tools.jar asset in release '$($release.tag_name)'."
}

Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $jarPath
Set-Content -Path $versionPath -Value $release.tag_name

Write-Host "Bootstrapped $($release.tag_name) to '$jarPath'."
