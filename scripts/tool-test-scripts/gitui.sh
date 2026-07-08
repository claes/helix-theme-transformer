#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$script_dir/common.sh"

require_theme_arg "$@"
theme="$1"
source_dir="$(theme_dir "$theme")/gitui"
theme_ron="$source_dir/theme.ron"
syntax_theme="$source_dir/syntax.tmTheme"
tmp_dir="$(make_temp_dir)"

cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

require_command gitui
require_file "$theme_ron"
require_file "$syntax_theme"

mkdir -p "$tmp_dir/gitui"
cp "$theme_ron" "$tmp_dir/gitui/theme.ron"
cp "$syntax_theme" "$tmp_dir/gitui/"

XDG_CONFIG_HOME="$tmp_dir" gitui
