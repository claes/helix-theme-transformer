#!/usr/bin/env bash
set -euo pipefail

themes_dir="generated-themes"
out_file="$themes_dir/manifest.json"

if [[ ! -d "$themes_dir" ]]; then
  echo "Missing $themes_dir. Run make generate-themes first." >&2
  exit 1
fi

for command in basename find jq sort; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "Missing required command: $command" >&2
    exit 1
  fi
done

tmp_file="$(mktemp)"
cleanup() {
  rm -f "$tmp_file"
}
trap cleanup EXIT

add_file() {
  local theme="$1"
  local tool="$2"
  local attr="$3"
  local path="$4"

  if [[ -f "$themes_dir/$path" ]]; then
    printf '%s\t%s\t%s\t%s\n' "$theme" "$tool" "$attr" "$path" >> "$tmp_file"
  fi
}

while IFS= read -r -d '' theme_dir; do
  theme_name="$(basename "$theme_dir")"

  add_file "$theme_name" "kitty" "theme" "$theme_name/kitty/theme.conf"
  add_file "$theme_name" "base16" "theme" "$theme_name/base16/theme.yaml"
  add_file "$theme_name" "base16" "shell" "$theme_name/base16/set-terminal-colors.sh"
  add_file "$theme_name" "bat" "theme" "$theme_name/bat/theme.tmTheme"
  add_file "$theme_name" "gitui" "theme" "$theme_name/gitui/theme.ron"
  add_file "$theme_name" "gitui" "syntax" "$theme_name/gitui/syntax.tmTheme"
  add_file "$theme_name" "mc" "theme" "$theme_name/mc/theme.ini"
  add_file "$theme_name" "mc" "filehighlight" "$theme_name/mc/filehighlight.ini"
  add_file "$theme_name" "mc" "colortable" "$theme_name/mc/colortable.env"
  add_file "$theme_name" "dircolors" "theme" "$theme_name/dircolors/theme.dircolors"
  add_file "$theme_name" "yazi" "theme" "$theme_name/yazi/theme.toml"
  add_file "$theme_name" "helix" "theme" "$theme_name/helix/theme.toml"
done < <(find "$themes_dir" -mindepth 1 -maxdepth 1 -type d -print0 | sort -z)

jq -Rn '
  [inputs | split("\t") | {theme: .[0], tool: .[1], attr: .[2], path: .[3]}] as $rows
  | {themes: (reduce $rows[] as $row ({}; .[$row.theme][$row.tool][$row.attr] = $row.path))}
' < "$tmp_file" > "$out_file"

jq . "$out_file" >/dev/null

echo "Generated $out_file"
