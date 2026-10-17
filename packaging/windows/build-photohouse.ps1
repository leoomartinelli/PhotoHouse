<# Build the customized, offline PhotoHouse installer. WiX 3.14.1 tools are build-time only. #>
param(
    [string]$BinDir = (Join-Path $PSScriptRoot '../../..'),
    [string]$WixDir = (Join-Path $PSScriptRoot '../../../installer-tools/wix'),
    [string]$OutputDir = (Join-Path $PSScriptRoot '../../../dist'),
    [switch]$SkipMsi
)
$ErrorActionPreference = 'Stop'
$source = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$BinDir = (Resolve-Path $BinDir).Path
$WixDir = (Resolve-Path $WixDir).Path
New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
$OutputDir = (Resolve-Path $OutputDir).Path
$manifest = Join-Path $PSScriptRoot 'photohouse-setup/Cargo.toml'
$version = '0.8.2'
$msi = Join-Path $OutputDir "PhotoHouse-$version-x64.msi"
foreach ($file in @('PhotoHouse.exe', 'photocraft-cli.exe')) {
    if (-not (Test-Path -LiteralPath (Join-Path $BinDir $file))) { throw "Missing $file in $BinDir" }
}
if (-not $SkipMsi) {
    & (Join-Path $WixDir 'candle.exe') -nologo -arch x64 "-dVersion=$version" "-dBinDir=$BinDir" "-dSourceDir=$source" "-dIconPath=$PSScriptRoot/photohouse.ico" -out "$OutputDir/photohouse.wixobj" "$PSScriptRoot/photohouse.wxs"
    if ($LASTEXITCODE -ne 0) { throw 'WiX compilation failed' }
    & (Join-Path $WixDir 'light.exe') -nologo -out $msi "$OutputDir/photohouse.wixobj"
    if ($LASTEXITCODE -ne 0) { throw 'MSI linking/validation failed' }
}
if (-not (Test-Path -LiteralPath $msi)) { throw "Missing MSI: $msi" }
$previousMsi = $env:PHOTOHOUSE_MSI
$previousTarget = $env:CARGO_TARGET_DIR
try {
    $env:PHOTOHOUSE_MSI = $msi
    $env:CARGO_TARGET_DIR = Join-Path $source 'target/photohouse-setup'
    cargo build --offline --locked --release --manifest-path $manifest
    if ($LASTEXITCODE -ne 0) { throw 'Installer compilation failed' }
    $setup = Join-Path $OutputDir "PhotoHouse-Setup-$version-x64.exe"
    Copy-Item -LiteralPath "$env:CARGO_TARGET_DIR/release/photohouse-setup.exe" -Destination $setup -Force
    $hashes = @($setup, $msi) | ForEach-Object { $h = Get-FileHash -Algorithm SHA256 -LiteralPath $_; "$($h.Hash.ToLower())  $([IO.Path]::GetFileName($_))" }
    $hashes | Set-Content -LiteralPath "$OutputDir/SHA256SUMS.txt" -Encoding ascii
    Write-Output "Installer: $setup"
} finally {
    $env:PHOTOHOUSE_MSI = $previousMsi
    $env:CARGO_TARGET_DIR = $previousTarget
}
