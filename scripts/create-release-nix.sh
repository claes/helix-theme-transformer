#!/usr/bin/env bash
set -euo pipefail

owner="claes"
repo="helix-theme-transformer"
themes_dir="generated-themes"
archive="generated-themes.zip"
out_file="generated-themes.nix"

if [[ $# -ne 1 ]]; then
  echo "Usage: $0 RELEASE_TAG" >&2
  exit 1
fi

release_tag="$1"

if [[ ! -d "$themes_dir" ]]; then
  echo "Missing $themes_dir. Run make generate-themes first." >&2
  exit 1
fi

if [[ ! -f "$archive" ]]; then
  echo "Missing $archive. Create it with: zip -r $archive $themes_dir" >&2
  exit 1
fi

for command in find nix sed sort unzip; do
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
archive_hash="$(nix hash path "$tmp_dir")"

nix_string() {
  printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g; s/\${/\\${/g'
}

emit_file_attr() {
  local indent="$1"
  local attr="$2"
  local path="$3"

  if [[ -f "$themes_dir/$path" ]]; then
    printf '%s%s = file "%s";\n' "$indent" "$attr" "$(nix_string "$path")"
  fi
}

emit_tool_attr() {
  local indent="$1"
  local tool="$2"
  local attr="$3"
  local path="$4"

  if [[ -f "$themes_dir/$path" ]]; then
    printf '%s%s = {\n' "$indent" "$tool"
    emit_file_attr "$indent  " "$attr" "$path"
    printf '%s};\n' "$indent"
  fi
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
  printf '  file = path: "${src}/generated-themes/${path}";\n'
  printf 'in\n'
  printf '{\n'
  printf '  inherit src;\n'
  printf '\n'
  printf '  themes = {\n'

  while IFS= read -r -d '' theme_dir; do
    theme_name="$(basename "$theme_dir")"
    escaped_theme_name="$(nix_string "$theme_name")"

    printf '    "%s" = {\n' "$escaped_theme_name"
    emit_tool_attr "      " "kitty" "theme" "$theme_name/kitty/$theme_name.conf"
    emit_tool_attr "      " "base16" "theme" "$theme_name/base16/$theme_name.yaml"
    emit_tool_attr "      " "bat" "theme" "$theme_name/bat/$theme_name.tmTheme"

    if [[ -f "$themes_dir/$theme_name/gitui/theme.ron" || -f "$themes_dir/$theme_name/gitui/$theme_name.tmTheme" ]]; then
      printf '      gitui = {\n'
      emit_file_attr "        " "theme" "$theme_name/gitui/theme.ron"
      emit_file_attr "        " "syntax" "$theme_name/gitui/$theme_name.tmTheme"
      printf '      };\n'
    fi

    if [[ -f "$themes_dir/$theme_name/mc/$theme_name.ini" || -f "$themes_dir/$theme_name/mc/filehighlight.ini" || -f "$themes_dir/$theme_name/mc/colortable.env" ]]; then
      printf '      mc = {\n'
      emit_file_attr "        " "theme" "$theme_name/mc/$theme_name.ini"
      emit_file_attr "        " "filehighlight" "$theme_name/mc/filehighlight.ini"
      emit_file_attr "        " "colortable" "$theme_name/mc/colortable.env"
      printf '      };\n'
    fi

    emit_tool_attr "      " "dircolors" "theme" "$theme_name/dircolors/$theme_name.dircolors"
    emit_tool_attr "      " "helix" "theme" "$theme_name/helix/$theme_name.toml"
    printf '    };\n'
  done < <(find "$themes_dir" -mindepth 1 -maxdepth 1 -type d -print0 | sort -z)

  printf '  };\n'
  printf '}\n'
} > "$out_file"

echo "Generated $out_file for release tag $release_tag"
