param(
    [Parameter(Mandatory = $true, Position = 0)]
    [string]$SpecPath,

    [Parameter(Position = 1)]
    [string]$ConfigPath,

    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$ExtraTlcArgs
)

$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true

function Resolve-Java {
    $java = Get-Command java -ErrorAction SilentlyContinue
    if ($null -eq $java) {
        throw "java was not found on PATH."
    }

    return $java.Source
}

function Resolve-TlaJar {
    $repoRoot = Split-Path -Parent $PSScriptRoot

    $candidates = @()
    if (-not [string]::IsNullOrWhiteSpace($env:TLA2TOOLS_JAR)) {
        $candidates += $env:TLA2TOOLS_JAR
    }

    $candidates += @(
        (Join-Path $repoRoot "tools/tla/tla2tools.jar"),
        "C:\Program Files\TLA+ Toolbox\tla2tools.jar",
        "C:\Program Files\TLA Toolbox\tla2tools.jar",
        (Join-Path $env:LOCALAPPDATA "Programs\TLA+\tla2tools.jar")
    )

    foreach ($candidate in $candidates) {
        if (-not [string]::IsNullOrWhiteSpace($candidate) -and (Test-Path $candidate)) {
            return (Resolve-Path $candidate).Path
        }
    }

    throw "Could not resolve tla2tools.jar. Run scripts/bootstrap-tla-tools.ps1 or set TLA2TOOLS_JAR."
}

function Resolve-TlcInvocation {
    param(
        [string]$RequestedSpecPath,
        [string]$RequestedConfigPath,
        [string[]]$RequestedExtraArgs
    )

    $resolvedSpec = (Resolve-Path $RequestedSpecPath).Path
    $specDirectory = Split-Path -Parent $resolvedSpec
    $specFileName = Split-Path -Leaf $resolvedSpec

    $resolvedConfig = $null
    if (-not [string]::IsNullOrWhiteSpace($RequestedConfigPath)) {
        $resolvedConfig = (Resolve-Path $RequestedConfigPath).Path
    }
    else {
        $defaultConfig = Join-Path $specDirectory ([System.IO.Path]::GetFileNameWithoutExtension($specFileName) + ".cfg")
        if (Test-Path $defaultConfig) {
            $resolvedConfig = (Resolve-Path $defaultConfig).Path
        }
    }

    $arguments = @()
    if ($null -ne $resolvedConfig) {
        $arguments += @("-config", $resolvedConfig)
    }

    if ($RequestedExtraArgs.Count -gt 0) {
        $arguments += $RequestedExtraArgs
    }

    $arguments += $specFileName

    return @{
        WorkingDirectory = $specDirectory
        Arguments = [string[]]$arguments
    }
}

$java = Resolve-Java
$jar = Resolve-TlaJar
$invocation = Resolve-TlcInvocation -RequestedSpecPath $SpecPath -RequestedConfigPath $ConfigPath -RequestedExtraArgs $ExtraTlcArgs
$javaArgs = @("-XX:+UseParallelGC", "-cp", $jar, "tlc2.TLC") + $invocation.Arguments

Push-Location $invocation.WorkingDirectory
try {
    $process = Start-Process -FilePath $java -ArgumentList $javaArgs -NoNewWindow -Wait -PassThru
    exit $process.ExitCode
}
finally {
    Pop-Location
}
