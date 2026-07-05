#!/usr/bin/env bash

generated_root="generated-themes"

die() {
  echo "$*" >&2
  exit 1
}

usage() {
  local script_name
  script_name="$(basename "$0")"
  die "Usage: $script_name THEME"
}

require_theme_arg() {
  [[ $# -eq 1 ]] || usage
  [[ -n "$1" ]] || usage
}

require_command() {
  local command="$1"
  command -v "$command" >/dev/null 2>&1 || die "Missing required command: $command"
}

require_file() {
  local path="$1"
  [[ -f "$path" ]] || die "Missing $path. Run make generate-themes first."
}

require_dir() {
  local path="$1"
  [[ -d "$path" ]] || die "Missing $path. Run make generate-themes first."
}

theme_dir() {
  local theme="$1"
  printf '%s/%s' "$generated_root" "$theme"
}

make_temp_dir() {
  local tmp_dir
  tmp_dir="$(mktemp -d)"
  printf '%s' "$tmp_dir"
}
