#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$script_dir/common.sh"

require_theme_arg "$@"
theme="$1"
theme_file="$(theme_dir "$theme")/yazi/theme.toml"
tmp_dir="$(make_temp_dir)"

cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

require_command yazi
require_file "$theme_file"

mkdir -p "$tmp_dir/yazi"
cp "$theme_file" "$tmp_dir/yazi/theme.toml"

XDG_CONFIG_HOME="$tmp_dir" yazi
