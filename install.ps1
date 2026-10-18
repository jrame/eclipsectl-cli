param([string]$InstallDir = (Join-Path $env:USERPROFILE '.local\bin'))
$ErrorActionPreference = 'Stop'
$binary = Join-Path $PSScriptRoot 'eclipsectl.exe'
if (-not (Test-Path -LiteralPath $binary)) {
    if (-not (Test-Path -LiteralPath (Join-Path $PSScriptRoot 'Cargo.toml'))) { throw 'Cannot find eclipsectl.exe or Cargo.toml beside this installer.' }
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { throw 'Rust is required for source installation; alternatively download a release archive.' }
    Push-Location $PSScriptRoot
    try {
        cargo build --locked --release
        if ($LASTEXITCODE -ne 0) { throw 'Cargo build failed.' }
    } finally { Pop-Location }
    $binary = Join-Path $PSScriptRoot 'target\release\eclipsectl.exe'
}
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
$destination = Join-Path $InstallDir 'eclipsectl.exe'
Copy-Item -LiteralPath $binary -Destination $destination -Force
Write-Output "Installed: $destination"
if (($env:PATH -split ';') -notcontains $InstallDir) { Write-Output "Add $InstallDir to PATH to use eclipsectl." }
& $destination --version
if ($LASTEXITCODE -ne 0) { throw 'Installed binary verification failed.' }
