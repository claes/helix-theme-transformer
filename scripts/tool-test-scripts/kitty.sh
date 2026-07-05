#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$script_dir/common.sh"

require_theme_arg "$@"
theme="$1"
theme_file="$(theme_dir "$theme")/kitty/$theme.conf"

require_command kitty
require_file "$theme_file"

kitty --config "$theme_file" sh -lc '
  printf "\nGenerated kitty theme sample\n\n"
  for code in 30 31 32 33 34 35 36 37 90 91 92 93 94 95 96 97; do
    printf "\033[%sm color %s \033[0m\n" "$code" "$code"
  done
  printf "\n"
  exec "${SHELL:-sh}"
'
