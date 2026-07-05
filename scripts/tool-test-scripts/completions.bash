#!/usr/bin/env bash

if ! type complete >/dev/null 2>&1; then
  return 0 2>/dev/null || exit 0
fi

_htt_tool_test_theme_names() {
  local generated_root="generated-themes"
  local theme_dir

  [[ -d "$generated_root" ]] || return 0

  for theme_dir in "$generated_root"/*; do
    [[ -d "$theme_dir" ]] || continue
    basename "$theme_dir"
  done
}

_htt_tool_test_complete_theme() {
  local cur
  cur="${COMP_WORDS[COMP_CWORD]}"

  if [[ "$COMP_CWORD" -eq 1 ]]; then
    mapfile -t COMPREPLY < <(compgen -W "$(_htt_tool_test_theme_names)" -- "$cur")
  else
    COMPREPLY=()
  fi
}

_htt_tool_test_register_completion() {
  local script="$1"

  complete -F _htt_tool_test_complete_theme "$script"
  complete -F _htt_tool_test_complete_theme "./$script"
}

_htt_tool_test_register_completion "scripts/tool-test-scripts/kitty.sh"
_htt_tool_test_register_completion "scripts/tool-test-scripts/bat.sh"
_htt_tool_test_register_completion "scripts/tool-test-scripts/gitui.sh"
_htt_tool_test_register_completion "scripts/tool-test-scripts/mc.sh"
_htt_tool_test_register_completion "scripts/tool-test-scripts/dircolors.sh"
_htt_tool_test_register_completion "scripts/tool-test-scripts/helix.sh"

unset -f _htt_tool_test_register_completion
