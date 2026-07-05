#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$script_dir/common.sh"

require_theme_arg "$@"
theme="$1"
theme_dir_path="$(theme_dir "$theme")/bat"
theme_file="$theme_dir_path/$theme.tmTheme"
tmp_dir="$(make_temp_dir)"
tmp_cache_home="$tmp_dir/cache"
sample_file="$tmp_dir/theme-sample.rs"

cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

require_command bat
require_file "$theme_file"

mkdir -p "$tmp_dir/themes" "$tmp_cache_home"
cp "$theme_file" "$tmp_dir/themes/$theme.tmTheme"
cat > "$sample_file" <<'RUST'
//! Theme sample for checking syntax, punctuation, strings, and diagnostics.

use std::{collections::BTreeMap, fmt::Display, path::PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

pub struct ThemePreview<'a, T>
where
    T: Display,
{
    pub name: &'a str,
    pub palette: BTreeMap<&'a str, &'a str>,
    pub selected: Option<T>,
}

impl<'a, T: Display> ThemePreview<'a, T> {
    pub fn render(&self, path: PathBuf) -> Result<String, Box<dyn std::error::Error>> {
        let accent = self.palette.get("accent").copied().unwrap_or("#7aa2f7");
        let count = self.palette.len() + 42;
        let message = format!("{}: {} colors from {:?}", self.name, count, path);

        match self.selected.as_ref() {
            Some(value) if count > 16 => println!("selected = {value}; accent = {accent}"),
            Some(_) => eprintln!("warning: small palette"),
            None => return Err("missing selected value".into()),
        }

        Ok(message)
    }
}

macro_rules! swatch {
    ($name:literal => $hex:literal) => {
        ($name, $hex)
    };
}

fn main() {
    let palette = BTreeMap::from([
        swatch!("background" => "#1f2335"),
        swatch!("foreground" => "#c0caf5"),
        swatch!("accent" => "#7aa2f7"),
    ]);

    let preview = ThemePreview {
        name: "adwaita-dark",
        palette,
        selected: Some(Severity::Warning),
    };

    if let Err(error) = preview.render("/tmp/theme.rs".into()) {
        panic!("render failed: {error}");
    }
}
RUST

BAT_CONFIG_DIR="$tmp_dir" XDG_CACHE_HOME="$tmp_cache_home" bat cache --build
BAT_CONFIG_DIR="$tmp_dir" XDG_CACHE_HOME="$tmp_cache_home" bat --theme "$theme" "$sample_file"
