param(
    [Parameter(Mandatory = $true)]
    [string]$Archive
)

$ErrorActionPreference = "Stop"

$packageDirectory = "package"

if ((Test-Path $Archive) -or (Test-Path $packageDirectory)) {
    throw "Refusing to overwrite an existing archive or package directory"
}

New-Item -ItemType Directory $packageDirectory | Out-Null
Copy-Item target/release/polyglid-desktop.exe $packageDirectory/
Copy-Item README.md, LICENSE-MIT, LICENSE-APACHE $packageDirectory/
@'
# PolyGlid runtime directories

The first launch creates the configuration, cache, logs, plugins, reports,
database, and default workspace directories. Existing installations are
opened idempotently and upgraded through the database migration system.

Set `POLYGLID_DATA_DIR` or `POLYGLID_WORKSPACE_ROOT` before launching to use
portable or isolated locations.
'@ | Set-Content -Path "$packageDirectory/runtime-directories.md" -Encoding utf8
& "$PSScriptRoot/verify-windows-package.ps1" -PackageDirectory $packageDirectory
Compress-Archive -Path "$packageDirectory/*" -DestinationPath $Archive
