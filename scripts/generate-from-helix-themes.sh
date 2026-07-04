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
mkdir -p "$out_dir"
cat >"$out_dir/CREDITS" <<'CREDITS'
Helix theme files
=================

The Helix theme source files in this directory were copied from the Helix editor
repository:

https://github.com/helix-editor/helix/tree/master/runtime/themes

Copyright for those original theme files belongs to their respective authors and
the Helix project contributors.

Generated theme files were produced by Themeforge from those Helix theme files.
CREDITS

count=0
while IFS= read -r -d '' theme_file; do
  theme_name="$(basename "$theme_file" .toml)"
  echo "Generating theme from $theme_file"
  "$themeforge_bin" export "$theme_file" --out-dir "$out_dir"
  mkdir -p "$out_dir/$theme_name/helix"
  cp "$theme_file" "$out_dir/$theme_name/helix/"
  count=$((count + 1))
done < <(find "$themes_dir" -type f -name '*.toml' -print0 | sort -z)

echo "Generated themes for $count Helix theme file(s) into $out_dir"
