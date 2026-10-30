#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$script_dir/common.sh"
require_theme_arg "$@"
theme_file="$(theme_dir "$1")/zellij/theme.kdl"
require_command zellij
require_file "$theme_file"
tmp_dir="$(make_temp_dir)"
session="htt-preview-${tmp_dir##*/}"

cleanup() {
  XDG_CACHE_HOME="$tmp_dir/cache" zellij kill-session "$session" >/dev/null 2>&1 || true
  rm -rf "$tmp_dir"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

mkdir -p "$tmp_dir/config" "$tmp_dir/cache" "$tmp_dir/data"
cat "$theme_file" > "$tmp_dir/config/config.kdl"
# The exporter supplies a correctly quoted KDL activation directive.
sed -n '2s|^// Activate with: ||p' "$theme_file" >> "$tmp_dir/config/config.kdl"
cat >> "$tmp_dir/config/config.kdl" <<'KDL'
session_serialization false
pane_frames true
KDL
cat > "$tmp_dir/layout.kdl" <<'KDL'
layout {
    default_tab_template {
        pane size=1 borderless=true { plugin location="zellij:tab-bar"; }
        children
        pane size=2 borderless=true { plugin location="zellij:status-bar"; }
    }
    tab name="Theme preview" focus=true {
        pane name="Focused frame" command="sh" size="60%" focus=true {
            args "-c" "printf 'Generated Zellij theme preview\n\nCtrl+q: quit    Alt+arrows: change focused pane\nCtrl+t, then arrows: switch tabs\nCtrl+p: highlight the focused frame in pane mode\nEsc: return to normal mode\n'; while read -r line; do :; done"
        }
        pane split_direction="vertical" {
            pane name="Success exit code" command="sh" close_on_exit=false {
                args "-c" "printf 'Successful command\n'; exit 0"
            }
            pane name="Error exit code" command="sh" close_on_exit=false {
                args "-c" "printf 'Failed command\n'; exit 1"
            }
        }
    }
    tab name="Another tab" {
        pane name="Selected and unselected tabs" command="sh" {
            args "-c" "printf 'Compare this selected tab with the inactive Theme preview tab.\n\nCtrl+q: quit    Ctrl+t, then arrows: switch tabs\n'; while read -r line; do :; done"
        }
    }
}
KDL

env -u ZELLIJ -u ZELLIJ_SESSION_NAME XDG_CACHE_HOME="$tmp_dir/cache" \
  zellij --config-dir "$tmp_dir/config" --config "$tmp_dir/config/config.kdl" \
  --data-dir "$tmp_dir/data" \
  --session "$session" --new-session-with-layout "$tmp_dir/layout.kdl"
