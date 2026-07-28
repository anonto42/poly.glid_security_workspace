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
Compress-Archive -Path "$packageDirectory/*" -DestinationPath $Archive
