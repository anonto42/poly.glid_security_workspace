param(
    [Parameter(Mandatory = $true)]
    [string]$PackageDirectory,

    [Parameter(Mandatory = $true)]
    [string]$Archive,

    [Parameter(Mandatory = $true)]
    [string]$Version
)

$ErrorActionPreference = "Stop"

if (-not (Get-Command wix -ErrorAction SilentlyContinue)) {
    throw "WiX v4 CLI is required to build the MSI. Install the 'wix' .NET tool first."
}

if (-not (Test-Path $PackageDirectory -PathType Container)) {
    throw "Package directory does not exist: $PackageDirectory"
}

if (Test-Path $Archive) {
    throw "Refusing to overwrite existing MSI: $Archive"
}

& wix build "$PSScriptRoot/windows/PolyGlid.wxs" `
    -d "PackageDirectory=$(Resolve-Path $PackageDirectory)" `
    -d "Version=$Version" `
    -o $Archive

if (-not (Test-Path $Archive -PathType Leaf)) {
    throw "WiX completed without creating the MSI: $Archive"
}

Write-Host "Windows MSI created: $Archive"
