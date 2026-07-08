#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$script_dir/common.sh"

require_theme_arg "$@"
theme="$1"
theme_file="$(theme_dir "$theme")/helix/theme.toml"
sample_file="tests/fixtures/minimal.toml"
tmp_dir="$(make_temp_dir)"

cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

require_command hx
require_file "$theme_file"
require_file "$sample_file"

mkdir -p "$tmp_dir/helix/themes"
cp "$theme_file" "$tmp_dir/helix/themes/$theme.toml"
cat > "$tmp_dir/helix/config.toml" <<HELIX_CONFIG
theme = "$theme"
HELIX_CONFIG

XDG_CONFIG_HOME="$tmp_dir" hx "$sample_file"
