param([string]$Msi = (Join-Path $PSScriptRoot '../../../dist/PhotoHouse-0.8.2-x64.msi'))
$ErrorActionPreference = 'Stop'
$Msi = (Resolve-Path $Msi).Path
if (Test-Path 'HKCU:/Software/PhotoHouse') { throw 'Existing PhotoHouse installation: refusing to modify it during the test.' }
$workspace = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$testRoot = Join-Path $workspace ('installer-test-' + [guid]::NewGuid().ToString('N'))
$app = Join-Path $testRoot 'app'
$desktop = [Environment]::GetFolderPath('DesktopDirectory')
$menu = [Environment]::GetFolderPath('Programs')
foreach($folder in @($desktop,$menu)) {
    if(Test-Path (Join-Path $folder 'PhotoHouse.lnk')) { throw 'Existing shortcut: refusing to overwrite it during tests.' }
}
New-Item -ItemType Directory -Path $app -Force | Out-Null
$installer = New-Object -ComObject WindowsInstaller.Installer
$db = $installer.OpenDatabase($Msi,0)
$view = $db.OpenView("SELECT Value FROM Property WHERE Property = 'ProductCode'")
$view.Execute()
$productCode = $view.Fetch().StringData(1)
$view.Close()
$db = $null
function Invoke-Msi([string[]]$Arguments, [string]$LogName) {
    $log = Join-Path $testRoot $LogName
    $process = Start-Process -FilePath "$env:SystemRoot/System32/msiexec.exe" -ArgumentList ($Arguments + @('/qn','/norestart','REBOOT=ReallySuppress','/L*v',('"' + $log + '"'))) -WindowStyle Hidden -Wait -PassThru
    if ($process.ExitCode -notin @(0,3010)) { throw "MSI returned $($process.ExitCode); see $log" }
}
$installed = $false
try {
    Invoke-Msi @('/i',('"'+$Msi+'"'),('INSTALLFOLDER="'+$app+'"')) 'install.log'
    $installed = $true
    foreach($name in @('PhotoHouse.exe','photocraft-cli.exe')) {
        $original = (Get-FileHash -LiteralPath (Join-Path $workspace $name)).Hash
        $copied = (Get-FileHash -LiteralPath (Join-Path $app $name)).Hash
        if($original -ne $copied) { throw "Hash mismatch: $name" }
    }
    $shell = New-Object -ComObject WScript.Shell
    foreach($folder in @($desktop,$menu)) {
        $shortcut = $shell.CreateShortcut((Join-Path $folder 'PhotoHouse.lnk'))
        if($shortcut.TargetPath -ne (Join-Path $app 'PhotoHouse.exe')) { throw 'Incorrect shortcut target' }
    }
    $version = & (Join-Path $app 'photocraft-cli.exe') --version
    if($LASTEXITCODE -ne 0) { throw 'Installed CLI did not launch' }
    # Per-user MSI products are enumerated through Windows Installer, not necessarily
    # the conventional HKCU Uninstall key used by non-MSI installers.
    if($installer.ProductState($productCode) -ne 5) { throw 'MSI product is not registered as installed' }
    if($installer.ProductInfo($productCode, 'ProductName') -ne 'PhotoHouse') { throw 'Incorrect registered product name' }
    if(-not (Test-Path $installer.ProductInfo($productCode, 'LocalPackage'))) { throw 'Missing cached package for uninstall' }
    Invoke-Msi @('/fa', $productCode) 'repair.log'
    # MSI must preserve user-created files while removing only package-owned files.
    Set-Content -LiteralPath (Join-Path $app 'user-document.txt') -Value 'preserve me'
    Invoke-Msi @('/x', $productCode) 'uninstall.log'
    $installed = $false
    if(Test-Path (Join-Path $app 'PhotoHouse.exe')) { throw 'Executable remained after uninstall' }
    if(Test-Path (Join-Path $desktop 'PhotoHouse.lnk')) { throw 'Desktop shortcut remained after uninstall' }
    if(Test-Path (Join-Path $menu 'PhotoHouse.lnk')) { throw 'Start shortcut remained after uninstall' }
    if($installer.ProductState($productCode) -ne -1) { throw 'Uninstall registration remained' }
    if(-not (Test-Path (Join-Path $app 'user-document.txt'))) { throw 'Uninstall removed a user document' }
    @{result='PASS';package=$Msi;productCode=$productCode;cli=$version;checks=@('install','payload hashes','desktop shortcut','start shortcut','CLI launch','uninstall registry','repair','uninstall','user file preservation');logs=$testRoot} | ConvertTo-Json | Tee-Object -FilePath (Join-Path $workspace 'dist/installation-test.json')
} finally {
    if($installed) { Invoke-Msi @('/x', $productCode) 'cleanup.log' }
}
