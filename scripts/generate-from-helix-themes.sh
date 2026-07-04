#!/usr/bin/env bash
set -euo pipefail

themes_dir="helix-themes"
out_dir="generated-themes"
themeforge_bin="target/debug/themeforge"

if [[ ! -d "$themes_dir" ]]; then
  echo "Missing $themes_dir. Run ./scripts/download-helix-themes.sh first." >&2
  exit 1
fi

cargo build --package themeforge-cli

count=0
while IFS= read -r -d '' theme_file; do
  echo "Generating theme from $theme_file"
  "$themeforge_bin" export "$theme_file" --out-dir "$out_dir"
  count=$((count + 1))
done < <(find "$themes_dir" -type f -name '*.toml' -print0 | sort -z)

echo "Generated themes for $count Helix theme file(s) into $out_dir"
