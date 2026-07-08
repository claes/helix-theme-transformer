#!/usr/bin/env bash
set -euo pipefail

in_dir="${1:-generated-themes}"
out_dir="${2:-generated-themes-by-app}"

if [[ ! -d "$in_dir" ]]; then
  echo "Missing input directory: $in_dir" >&2
  exit 1
fi

if [[ -e "$out_dir" ]]; then
  echo "Output path already exists: $out_dir" >&2
  exit 1
fi

for command in basename cp dirname find mkdir sort; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "Missing required command: $command" >&2
    exit 1
  fi
done

copy_file() {
  local source="$1"
  local target="$2"

  if [[ -f "$source" ]]; then
    mkdir -p "$(dirname "$target")"
    cp "$source" "$target"
  fi
}

while IFS= read -r -d '' theme_dir; do
  theme_name="$(basename "$theme_dir")"

  copy_file "$theme_dir/kitty/theme.conf" "$out_dir/kitty/$theme_name.conf"
  copy_file "$theme_dir/base16/theme.yaml" "$out_dir/base16/$theme_name.yaml"
  copy_file "$theme_dir/bat/theme.tmTheme" "$out_dir/bat/$theme_name.tmTheme"
  copy_file "$theme_dir/gitui/theme.ron" "$out_dir/gitui/$theme_name/theme.ron"
  copy_file "$theme_dir/gitui/syntax.tmTheme" "$out_dir/gitui/$theme_name/syntax.tmTheme"
  copy_file "$theme_dir/mc/theme.ini" "$out_dir/mc/$theme_name.ini"
  copy_file "$theme_dir/mc/filehighlight.ini" "$out_dir/mc/$theme_name-filehighlight.ini"
  copy_file "$theme_dir/mc/colortable.env" "$out_dir/mc/$theme_name-colortable.env"
  copy_file "$theme_dir/dircolors/theme.dircolors" "$out_dir/dircolors/$theme_name.dircolors"
  copy_file "$theme_dir/yazi/theme.toml" "$out_dir/yazi/$theme_name/theme.toml"
  copy_file "$theme_dir/helix/theme.toml" "$out_dir/helix/$theme_name.toml"
done < <(find "$in_dir" -mindepth 1 -maxdepth 1 -type d -print0 | sort -z)

echo "Generated transposed theme tree at $out_dir"
