param([switch]$FromTheirs)
# PhotoHouse: every UI translation catalogue is upstream's with "PhotoCraft" -> "PhotoHouse".
# -FromTheirs: during a merge, first take upstream's version of conflicted catalogues.
$enc = New-Object System.Text.UTF8Encoding($false)
$dir = Join-Path (git rev-parse --show-toplevel) 'crates/ui-egui/src/i18n'
if ($FromTheirs) {
    git diff --name-only --diff-filter=U -- 'crates/ui-egui/src/i18n/*.tsv' | ForEach-Object { git checkout --theirs -- $_ }
}
Get-ChildItem $dir -Filter *.tsv | ForEach-Object {
    $t = [IO.File]::ReadAllText($_.FullName, $enc)
    $n = $t.Replace('PhotoCraft', 'PhotoHouse')
    if ($n -cne $t) { [IO.File]::WriteAllText($_.FullName, $n, $enc); "rebranded $($_.Name)" }
}
