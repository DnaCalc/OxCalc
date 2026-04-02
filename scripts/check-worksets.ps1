$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true

Push-Location (Join-Path $PSScriptRoot "..")
try {
    $registerPath = "docs/WORKSET_REGISTER.md"
    if (-not (Test-Path $registerPath)) {
        throw "check-worksets: missing $registerPath"
    }

    if (-not (Test-Path ".beads")) {
        throw "check-worksets: missing .beads workspace"
    }

    $content = Get-Content $registerPath -Raw

    $worksetMatches = [regex]::Matches(
        $content,
        '^###\s+(W\d{3})\s+',
        [System.Text.RegularExpressions.RegexOptions]::Multiline
    )
    if ($worksetMatches.Count -eq 0) {
        throw "check-worksets: no Wnnn workset headings found"
    }

    $worksetIds = @($worksetMatches | ForEach-Object { $_.Groups[1].Value })
    $duplicate = $worksetIds | Group-Object | Where-Object { $_.Count -gt 1 } | Select-Object -First 1
    if ($duplicate) {
        throw "check-worksets: duplicate workset id '$($duplicate.Name)'"
    }

    foreach ($id in $worksetIds) {
        $purposePattern = '(?ms)^###\s+' + [regex]::Escape($id) + '\s+.*?1\.\s+purpose:'
        $dependsPattern = '(?ms)^###\s+' + [regex]::Escape($id) + '\s+.*?2\.\s+depends_on:'
        $closurePattern = '(?ms)^###\s+' + [regex]::Escape($id) + '\s+.*?5\.\s+closure_condition:'
        $epicsPattern = '(?ms)^###\s+' + [regex]::Escape($id) + '\s+.*?6\.\s+initial_epic_lanes:'
        $rolloutPattern = '(?ms)^###\s+' + [regex]::Escape($id) + '\s+.*?7\.\s+rollout_mode:'

        if ($content -notmatch $purposePattern) {
            throw "check-worksets: missing purpose field for $id"
        }
        if ($content -notmatch $dependsPattern) {
            throw "check-worksets: missing depends_on field for $id"
        }
        if ($content -notmatch $closurePattern) {
            throw "check-worksets: missing closure_condition field for $id"
        }
        if ($content -notmatch $epicsPattern) {
            throw "check-worksets: missing initial_epic_lanes field for $id"
        }
        if ($content -notmatch $rolloutPattern) {
            throw "check-worksets: missing rollout_mode field for $id"
        }
    }

    $beadSummaryText = "beads=unavailable"
    try {
        $statsJson = br stats --robot 2>$null
        if ($LASTEXITCODE -eq 0 -and $statsJson) {
            $stats = $statsJson | ConvertFrom-Json
            $summary = $stats.summary
            if ($summary) {
                $beadSummaryText = @(
                    "beads",
                    "total=$($summary.total_issues)",
                    "open=$($summary.open_issues)",
                    "in_progress=$($summary.in_progress_issues)",
                    "ready=$($summary.ready_issues)",
                    "blocked=$($summary.blocked_issues)",
                    "deferred=$($summary.deferred_issues)",
                    "closed=$($summary.closed_issues)"
                ) -join ", "
            }
        }
    }
    catch {
        # Keep the validator usable as a register shape check even if br stats is unavailable.
    }

    Write-Host "check-worksets: ok (worksets=$($worksetIds.Count); $beadSummaryText)"
}
finally {
    Pop-Location
}
