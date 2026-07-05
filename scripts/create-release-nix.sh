#!/usr/bin/env bash
set -euo pipefail

owner="claes"
repo="helix-theme-transformer"
archive="generated-themes.zip"
out_file="generated-themes.nix"

if [[ $# -ne 1 ]]; then
  echo "Usage: $0 RELEASE_TAG" >&2
  exit 1
fi

release_tag="$1"

if [[ ! -f "$archive" ]]; then
  echo "Missing $archive. Run make generated-themes.zip first." >&2
  exit 1
fi

for command in nix sed unzip; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "Missing required command: $command" >&2
    exit 1
  fi
done

tmp_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

unzip -q "$archive" -d "$tmp_dir"
if [[ ! -f "$tmp_dir/generated-themes/manifest.json" ]]; then
  echo "Missing generated-themes/manifest.json in $archive." >&2
  exit 1
fi
archive_hash="$(nix hash path "$tmp_dir")"

nix_string() {
  printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g; s/\${/\\${/g'
}

{
  printf '{ pkgs }:\n'
  printf '\n'
  printf 'let\n'
  printf '  src = pkgs.fetchzip {\n'
  printf '    url = "https://github.com/%s/%s/releases/download/%s/%s";\n' \
    "$(nix_string "$owner")" \
    "$(nix_string "$repo")" \
    "$(nix_string "$release_tag")" \
    "$(nix_string "$archive")"
  printf '    hash = "%s";\n' "$(nix_string "$archive_hash")"
  printf '    stripRoot = false;\n'
  printf '  };\n'
  printf '\n'
  printf '  manifest =\n'
  printf '    builtins.fromJSON (builtins.readFile "${src}/generated-themes/manifest.json");\n'
  printf '\n'
  printf '  file = path: "${src}/generated-themes/${path}";\n'
  printf '\n'
  printf '  mapFiles = value:\n'
  printf '    if builtins.isAttrs value\n'
  printf '    then builtins.mapAttrs (_: mapFiles) value\n'
  printf '    else file value;\n'
  printf 'in\n'
  printf '{\n'
  printf '  inherit src manifest;\n'
  printf '\n'
  printf '  themes = builtins.mapAttrs (_: mapFiles) manifest.themes;\n'
  printf '}\n'
} > "$out_file"

echo "Generated $out_file for release tag $release_tag"
