#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$script_dir/common.sh"

require_theme_arg "$@"
theme="$1"
skin_file="$(theme_dir "$theme")/mc/$theme.ini"
tmp_dir="$(make_temp_dir)"
tmp_config_home="$tmp_dir/config"
tmp_data_home="$tmp_dir/share"
tmp_cache_home="$tmp_dir/cache"

cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

require_command mc
require_file "$skin_file"

mkdir -p "$tmp_config_home/mc" "$tmp_data_home/mc/skins" "$tmp_cache_home/mc"
cp "$skin_file" "$tmp_data_home/mc/skins/$theme.ini"

XDG_CONFIG_HOME="$tmp_config_home" \
  XDG_DATA_HOME="$tmp_data_home" \
  XDG_CACHE_HOME="$tmp_cache_home" \
  mc --skin "$theme"
