#!/usr/bin/env bash
set -euo pipefail

# Generated from Helix theme: minimal
# Applies the derived Base16/ANSI palette to the current terminal session.

set_ansi_color() {
  local index="$1" color="$2"
  printf '\033]4;%s;%s\007' "$index" "$color"
}

set_dynamic_color() {
  local code="$1" color="$2"
  printf '\033]%s;%s\007' "$code" "$color"
}

set_ansi_color 0 '#1f2335'
set_ansi_color 1 '#f7768e'
set_ansi_color 2 '#9ece6a'
set_ansi_color 3 '#e0af68'
set_ansi_color 4 '#7aa2f7'
set_ansi_color 5 '#bb9af7'
set_ansi_color 6 '#7dcfff'
set_ansi_color 7 '#c0caf5'
set_ansi_color 8 '#565f89'
set_ansi_color 9 '#f8899e'
set_ansi_color 10 '#acd57f'
set_ansi_color 11 '#e4ba7d'
set_ansi_color 12 '#8daff8'
set_ansi_color 13 '#c5a8f8'
set_ansi_color 14 '#8fd6ff'
set_ansi_color 15 '#b8c2eb'

set_dynamic_color 10 '#c0caf5'
set_dynamic_color 11 '#1f2335'
set_dynamic_color 12 '#7aa2f7'
set_dynamic_color 17 '#3b4261'
set_dynamic_color 19 '#c0caf5'

# Cursor text should contrast with the cursor color; xterm OSC has no portable setter for it.
# Intended cursor text color: #1f2335
