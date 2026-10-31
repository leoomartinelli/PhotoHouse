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
$workspaceManifest = Get-Content -Raw -LiteralPath (Join-Path $source 'Cargo.toml')
$versionMatch = [regex]::Match($workspaceManifest, '(?ms)^\[workspace\.package\]\s*\r?\nversion\s*=\s*"(\d+\.\d+\.\d+)"')
if (-not $versionMatch.Success) { throw 'Workspace version was not found' }
$version = $versionMatch.Groups[1].Value
$fontLicenseInclude = Join-Path $OutputDir 'craft-font-licenses.wxi'
$fontLicenseEntries = @()
if ($env:CRAFT_FONTS_DIR) {
    $fontRoot = (Resolve-Path -LiteralPath $env:CRAFT_FONTS_DIR).Path
    $fontManifest = Join-Path $fontRoot 'fonts/manifest.txt'
    $licensePaths = @(Get-Content -LiteralPath $fontManifest | ForEach-Object {
        if ($_.Trim() -and -not $_.TrimStart().StartsWith('#')) {
            $fields = $_.Split('|')
            if ($fields.Count -lt 6) { throw 'Invalid craft-fonts manifest row' }
            $relativeLicense = $fields[5].Trim()
            $licensePath = [IO.Path]::GetFullPath((Join-Path $fontRoot $relativeLicense))
            if (-not $licensePath.StartsWith($fontRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
                throw 'Font licence path escapes craft-fonts checkout'
            }
            (Resolve-Path -LiteralPath $licensePath).Path
        }
    } | Sort-Object -Unique)
    $licenseNumber = 0
    foreach ($licensePath in $licensePaths) {
        $family = Split-Path (Split-Path $licensePath -Parent) -Leaf
        $safeFamily = $family -replace '[^A-Za-z0-9-]', '-'
        $xmlPath = [Security.SecurityElement]::Escape($licensePath)
        $fontLicenseEntries += ('<File Id="CraftFontLicense{0}" Name="OFL-{1}.txt" Source="{2}" />' -f $licenseNumber, $safeFamily, $xmlPath)
        $licenseNumber++
    }
    $attributionPath = [Security.SecurityElement]::Escape((Join-Path $fontRoot 'ATTRIBUTION.md'))
    $fontLicenseEntries += ('<File Id="CraftFontAttribution" Name="CRAFT-FONTS-ATTRIBUTION.md" Source="{0}" />' -f $attributionPath)
}
$fontLicenseXml = '<Include xmlns="http://schemas.microsoft.com/wix/2006/wi">' + "`n" + ($fontLicenseEntries -join "`n") + "`n</Include>"
[IO.File]::WriteAllText($fontLicenseInclude, $fontLicenseXml, (New-Object Text.UTF8Encoding($false)))
$msi = Join-Path $OutputDir "PhotoHouse-$version-x64.msi"
foreach ($file in @('PhotoHouse.exe', 'photocraft-cli.exe')) {
    if (-not (Test-Path -LiteralPath (Join-Path $BinDir $file))) { throw "Missing $file in $BinDir" }
}
if (-not $SkipMsi) {
    & (Join-Path $WixDir 'candle.exe') -nologo -arch x64 "-dVersion=$version" "-dBinDir=$BinDir" "-dSourceDir=$source" "-dIconPath=$PSScriptRoot/photohouse.ico" "-dFontLicensesInclude=$fontLicenseInclude" -out "$OutputDir/photohouse.wixobj" "$PSScriptRoot/photohouse.wxs"
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
