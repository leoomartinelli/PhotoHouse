#!/usr/bin/env bash
# Anexa os instaladores de uma pasta à release $TAG, trocando o prefixo photocraft por PhotoHouse.
set -euo pipefail
dir="$1"
shopt -s nullglob
arquivos=()
for f in "$dir"/*.{AppImage,zsync,deb,rpm,tar.gz,dmg,zip,exe,msi}; do
  nome="$(basename "$f")"
  novo="${nome/#photocraft-cli-/PhotoHouse-cli-}"
  novo="${novo/#photocraft-/PhotoHouse-}"
  [ "$nome" != "$novo" ] && mv "$f" "$dir/$novo"
  arquivos+=("$dir/$novo")
done
[ ${#arquivos[@]} -gt 0 ] || { echo "Nenhum instalador em $dir" >&2; ls -la "$dir" >&2; exit 1; }
printf '%s\n' "${arquivos[@]}"
gh release upload "$TAG" "${arquivos[@]}" --clobber
