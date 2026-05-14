# W050 X5 Evidence Roots And Validation Manifest

This root is the compact index for W050 checked evidence and validation
commands. It does not regenerate artifacts.

Validation command:

```powershell
$manifest = 'docs/test-runs/core-engine/w050-x5-evidence-roots-and-validation-manifest-001/validation_manifest.json'
$json = Get-Content $manifest -Raw | ConvertFrom-Json
foreach ($root in $json.checked_evidence_roots.path) { if (-not (Test-Path $root)) { throw "Missing evidence root: $root" } }
foreach ($packet in $json.handoff_packets.path) { if (-not (Test-Path $packet)) { throw "Missing handoff packet: $packet" } }
```

All commands listed in the manifest are read-only with respect to tracked
baseline artifacts unless an explicit future regeneration run id is declared.
