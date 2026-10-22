<#
.SYNOPSIS
  Build and package PhotoHouse for Windows in the format House Studio installs.

.DESCRIPTION
  Produces, in $env:DIST (default: dist/release):
    photohouse-<version>-windows-<arch>-portable.zip

  The archive is flat: photohouse.exe (the editor) sits at its root, next to photohouse-cli.exe,
  the licences and README. House Studio unpacks it into its own apps folder (<apps>\photohouse\)
  and looks for photohouse.exe at the archive root (HOUSE-STUDIO crates/native/src/portable.rs).
  The "-portable.zip" suffix is how House Studio recognizes the package kind
  (PackageKind::PortableZip in HOUSE-STUDIO crates/core/src/platform.rs; a bare ".zip" is ignored).

  portable.txt is deliberately left out: House Studio owns the install folder and replaces its
  files on update, so settings stay in the normal per-user folder (%APPDATA%).

  The binaries link the C runtime statically (+crt-static), so no Visual C++ redistributable or
  extra DLL is needed. There is no code-signing certificate: sign.ps1 runs only when signing
  secrets are set and otherwise leaves the files unsigned with a warning.

  The version is [workspace.package] version in Cargo.toml; CI can override it with
  $env:PHOTOHOUSE_VERSION (a leading "v" is dropped).

  Needs: Rust with the MSVC toolchain and the target (rustup target add x86_64-pc-windows-msvc),
  Visual Studio Build Tools (C++ workload, Windows SDK: rc.exe embeds the icon and VERSIONINFO),
  PowerShell 5.1 or 7.

  The customized offline installer (PhotoHouse-Setup-<version>-x64.exe with the mascot) is a
  separate build: build-photohouse.ps1 (see PHOTOHOUSE-INSTALLER.md).

.EXAMPLE
  pwsh packaging/windows/package.ps1
  pwsh packaging/windows/package.ps1 -Arch x64 -SkipBuild
#>
param(
  [ValidateSet('x64', 'x86', 'arm64')] [string] $Arch = 'x64',
  [switch] $SkipBuild
)
$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$AppName = 'PhotoHouse'
$AppStem = 'photohouse'

function Invoke-Native([string] $What, [scriptblock] $Block) {
  Write-Output "==> $What"
  & $Block
  if ($LASTEXITCODE -ne 0) { throw "$What failed with exit code $LASTEXITCODE" }
}

# The version lives in one place: [workspace.package] version in the root Cargo.toml.
$Version = if ($env:PHOTOHOUSE_VERSION) { $env:PHOTOHOUSE_VERSION } else { $env:PHOTOCRAFT_VERSION }
if (-not $Version) {
  $inPkg = $false
  foreach ($line in Get-Content (Join-Path $Root 'Cargo.toml')) {
    if ($line -match '^\s*\[') { $inPkg = ($line.Trim() -eq '[workspace.package]'); continue }
    if ($inPkg -and $line -match '^\s*version\s*=\s*"([^"]+)"') { $Version = $Matches[1]; break }
  }
}
if (-not $Version) { throw 'could not read [workspace.package] version from Cargo.toml' }
$Version = $Version -replace '^[vV]', ''
if ($Version -notmatch '^\d+\.\d+\.\d+([-+][0-9A-Za-z.-]+)?$') { throw "version '$Version' is not X.Y.Z[-pre]" }

$Target = switch ($Arch) { 'x64' { 'x86_64-pc-windows-msvc' } 'x86' { 'i686-pc-windows-msvc' } 'arm64' { 'aarch64-pc-windows-msvc' } }
$Dist = if ($env:DIST) { $env:DIST } else { Join-Path $Root 'dist\release' }
$TargetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $Root 'target' }
New-Item -ItemType Directory -Force -Path $Dist | Out-Null

# Build provenance shown in About and --version (crates/engine/src/build_info.rs reads these
# upstream-named variables).
if (-not $env:PHOTOCRAFT_BUILD_SHA) { $env:PHOTOCRAFT_BUILD_SHA = (git -C $Root rev-parse HEAD 2>$null) }
if (-not $env:PHOTOCRAFT_BUILD_DATE) { $env:PHOTOCRAFT_BUILD_DATE = (Get-Date).ToUniversalTime().ToString('yyyy-MM-dd') }

Write-Output "$AppName $Version for Windows $Arch ($Target)"

if (-not $SkipBuild) {
  # Static CRT: no VC++ redistributable needed. Scoped to the target so host build scripts and
  # proc-macros are unaffected.
  $flagVar = 'CARGO_TARGET_' + ($Target.ToUpper() -replace '-', '_') + '_RUSTFLAGS'
  [Environment]::SetEnvironmentVariable($flagVar, '-C target-feature=+crt-static')
  # Fail the build (rather than warn) if the icon/VERSIONINFO can't be embedded.
  $env:PHOTOCRAFT_REQUIRE_WINRES = '1'
  Invoke-Native "cargo build ($Target)" { cargo build --release --locked -p photocraft -p photocraft-cli --features heif --target $Target }
}

$Bin = Join-Path $TargetDir "$Target\release"

# Check both binaries' PE headers before packaging. Machine (COFF header) must match -Arch, so an
# x64 build can never ship labelled arm64. Subsystem (optional header): 2 = Windows GUI, 3 = console.
# The app must be GUI (no console window opens with it); the CLI must stay console so its output
# reaches the terminal.
function Get-PeHeader([string] $Path) {
  $bytes = [System.IO.File]::ReadAllBytes($Path)
  $pe = [BitConverter]::ToInt32($bytes, 0x3C)
  return @{ Machine = [BitConverter]::ToUInt16($bytes, $pe + 4); Subsystem = [BitConverter]::ToUInt16($bytes, $pe + 0x5C) }
}
$Machine = switch ($Arch) { 'x64' { 0x8664 } 'x86' { 0x14C } 'arm64' { 0xAA64 } }
foreach ($check in @(@('photocraft.exe', 2), @('photocraft-cli.exe', 3))) {
  $path = Join-Path $Bin $check[0]
  if (-not (Test-Path -LiteralPath $path)) { throw "$path is missing (build it, or drop -SkipBuild)" }
  $h = Get-PeHeader $path
  if ($h.Machine -ne $Machine) { throw "$($check[0]) is for machine 0x$('{0:X}' -f $h.Machine), expected 0x$('{0:X}' -f $Machine) ($Arch)" }
  if ($h.Subsystem -ne $check[1]) { throw "$($check[0]) has PE subsystem $($h.Subsystem), expected $($check[1])" }
  Write-Output "ok $($check[0]): $Arch, PE subsystem $($h.Subsystem)"
}

# ---- stage: the archive root ---------------------------------------------------------------------
# The cargo packages keep the upstream names; the shipped files carry PhotoHouse's.
$Stage = Join-Path $TargetDir "windows-package\$AppStem-$Version-windows-$Arch-portable"
Remove-Item -Recurse -Force $Stage -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $Stage | Out-Null
Copy-Item (Join-Path $Bin 'photocraft.exe') (Join-Path $Stage "$AppStem.exe")
Copy-Item (Join-Path $Bin 'photocraft-cli.exe') (Join-Path $Stage "$AppStem-cli.exe")
foreach ($f in 'README.md', 'LICENSE', 'LICENSE-MIT', 'LICENSE-APACHE', 'NOTICE') {
  $p = Join-Path $Root $f
  if (Test-Path $p) { Copy-Item $p $Stage }
}
# Builds made with craft-fonts (CRAFT_FONTS_DIR) embed its OFL-1.1 fonts: ship each font's
# licence as OFL-<family-dir>.txt.
if ($env:CRAFT_FONTS_DIR) {
  Get-ChildItem -Path (Join-Path $env:CRAFT_FONTS_DIR 'fonts') -Directory -ErrorAction SilentlyContinue | ForEach-Object {
    $lic = Join-Path $_.FullName 'OFL.txt'
    if (Test-Path $lic) { Copy-Item $lic (Join-Path $Stage "OFL-$($_.Name).txt") }
  }
}

& (Join-Path $PSScriptRoot 'sign.ps1') (Join-Path $Stage "$AppStem.exe") (Join-Path $Stage "$AppStem-cli.exe")

# ---- zip -----------------------------------------------------------------------------------------
# Built with System.IO.Compression rather than Compress-Archive: entries are added from the
# folder's *contents* (no top-level folder), with forward slashes on every PowerShell version.
$Zip = Join-Path $Dist "$AppStem-$Version-windows-$Arch-portable.zip"
Remove-Item -Force $Zip -ErrorAction SilentlyContinue
Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem
$archive = [System.IO.Compression.ZipFile]::Open($Zip, [System.IO.Compression.ZipArchiveMode]::Create)
try {
  $base = (Resolve-Path -LiteralPath $Stage).Path.TrimEnd('\') + '\'
  Get-ChildItem -LiteralPath $Stage -Recurse -File | Sort-Object FullName | ForEach-Object {
    $entry = $_.FullName.Substring($base.Length) -replace '\\', '/'
    [void][System.IO.Compression.ZipFileExtensions]::CreateEntryFromFile($archive, $_.FullName, $entry, [System.IO.Compression.CompressionLevel]::Optimal)
  }
} finally { $archive.Dispose() }

# Check the archive the way House Studio reads it: photohouse.exe at the root, no folders above it.
$archive = [System.IO.Compression.ZipFile]::OpenRead($Zip)
try {
  $names = @($archive.Entries | ForEach-Object { $_.FullName })
} finally { $archive.Dispose() }
if ($names -notcontains "$AppStem.exe") { throw "$Zip has no $AppStem.exe at its root (entries: $($names -join ', '))" }
if ($names | Where-Object { $_ -match '\\|:' -or $_.StartsWith('/') -or $_ -match '(^|/)\.\.(/|$)' }) { throw "$Zip has an unsafe entry name" }
Write-Output "zip entries: $($names -join ', ')"

# Smoke-test the CLI when this machine can run it (an ARM64 build made on x64 can't run here).
$HostArch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString().ToLowerInvariant()
if ($Arch -ne 'arm64' -or $HostArch -eq 'arm64') {
  Invoke-Native "$AppStem-cli --version" { & (Join-Path $Stage "$AppStem-cli.exe") --version }
} else {
  Write-Output "skipping $AppStem-cli --version: an $Arch build doesn't run on this $HostArch machine"
}
$hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $Zip).Hash.ToLowerInvariant()
Write-Output "sha256 $hash  $([IO.Path]::GetFileName($Zip))"
Get-Item $Zip | Format-Table Name, Length
