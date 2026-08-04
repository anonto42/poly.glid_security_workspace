param(
    [Parameter(Mandatory = $true)]
    [string]$PackageDirectory
)

$ErrorActionPreference = "Stop"

foreach ($requiredFile in @(
    "polyglid-desktop.exe",
    "README.md",
    "LICENSE-MIT",
    "LICENSE-APACHE",
    "runtime-directories.md"
)) {
    if (-not (Test-Path (Join-Path $PackageDirectory $requiredFile) -PathType Leaf)) {
        throw "Windows package is missing $requiredFile"
    }
}

if (-not (Select-String -Path (Join-Path $PackageDirectory "runtime-directories.md") -Pattern "POLYGLID_DATA_DIR" -Quiet)) {
    throw "Windows package runtime documentation is incomplete"
}

Write-Host "Windows package validation passed: $PackageDirectory"
