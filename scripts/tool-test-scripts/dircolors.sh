#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$script_dir/common.sh"

require_theme_arg "$@"
theme="$1"
theme_file="$(theme_dir "$theme")/dircolors/theme.dircolors"

require_command dircolors
require_command ls
require_file "$theme_file"

eval "$(dircolors "$theme_file")"
export LS_COLORS

echo "Generated dircolors theme sample"
ls --color=always -la
echo
echo "Starting a temporary shell with LS_COLORS set. Exit to return."
exec bash --noprofile --norc
