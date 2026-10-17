# Read-only, advisory rebrand audit. Findings require manual review.
# Resolve the repository from this script, independently of the caller's cwd.
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
$enc = New-Object System.Text.UTF8Encoding($false)
$keys = @{}
$catalogs = Join-Path $root 'crates/ui-egui/src/i18n'
Get-ChildItem -LiteralPath $catalogs -Filter *.tsv | ForEach-Object {
    foreach ($line in [IO.File]::ReadAllLines($_.FullName, $enc)) {
        if ($line.StartsWith('#')) { continue }
        $cols = $line.Split("`t")
        if ($cols.Count -ge 2 -and $cols[1].Contains('PhotoHouse') -and $cols[1] -ne 'PhotoHouse') {
            $keys[$cols[1]] = 1
        }
    }
}
foreach ($k in ($keys.Keys | Sort-Object)) {
    foreach ($part in $k.Split('|')) {
        $old = $part.Replace('PhotoHouse', 'PhotoCraft')
        if ($old -eq $part) { continue }
        git -C $root grep -n -F -e "`"$old" -- 'crates/*.rs' 'apps/*.rs' |
            ForEach-Object { "RENAME: $_" }
        if ($LASTEXITCODE -gt 1) { throw 'git grep failed during translation audit.' }
    }
}
# User-visible literals are candidates; upstream credits may be legitimate.
git -C $root grep -n -F 'PhotoCraft' -- 'crates/ui-egui/src/*.rs' 'apps/photocraft/src/*.rs' |
    Where-Object { $_ -notmatch '^\S+:\d+:\s*//' -and $_ -match '"[^"]*PhotoCraft' } |
    ForEach-Object { "CHECK:  $_" }
if ($LASTEXITCODE -gt 1) { throw 'git grep failed during UI audit.' }
# No match (git grep exit code 1) is successful for this advisory audit.
exit 0
