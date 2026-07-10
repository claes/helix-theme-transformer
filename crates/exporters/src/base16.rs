use palette16::{color, compact_colors, Base16Palette};
use semantic_roles::{role_color, Role, SemanticRoles};
use theme_ir::{brighten, ResolvedTheme};

pub fn export_base16_yaml(palette: &Base16Palette) -> Result<String, serde_yaml::Error> {
    serde_yaml::to_string(&compact_colors(palette))
}

pub fn export_base16_terminal_script(
    theme: &ResolvedTheme,
    roles: &SemanticRoles,
    palette: &Base16Palette,
) -> String {
    let cursor = role_color(roles, Role::Cursor).unwrap_or(color(palette, "base05"));
    let mut output = String::new();
    output.push_str("#!/usr/bin/env bash\n");
    output.push_str("set -euo pipefail\n\n");
    output.push_str(&format!("# Generated from Helix theme: {}\n", theme.name));
    output
        .push_str("# Applies the derived Base16/ANSI palette to the current terminal session.\n\n");
    output.push_str("set_ansi_color() {\n");
    output.push_str("  local index=\"$1\" color=\"$2\"\n");
    output.push_str("  printf '\\033]4;%s;%s\\007' \"$index\" \"$color\"\n");
    output.push_str("}\n\n");
    output.push_str("set_dynamic_color() {\n");
    output.push_str("  local code=\"$1\" color=\"$2\"\n");
    output.push_str("  printf '\\033]%s;%s\\007' \"$code\" \"$color\"\n");
    output.push_str("}\n\n");

    for (index, value) in ansi_colors(palette) {
        output.push_str(&format!("set_ansi_color {index} '{value}'\n"));
    }
    output.push('\n');
    output.push_str(&format!(
        "set_dynamic_color 10 '{}'\n",
        color(palette, "base05")
    ));
    output.push_str(&format!(
        "set_dynamic_color 11 '{}'\n",
        color(palette, "base00")
    ));
    output.push_str(&format!("set_dynamic_color 12 '{cursor}'\n"));
    output.push_str(&format!(
        "set_dynamic_color 17 '{}'\n",
        color(palette, "base02")
    ));
    output.push_str(&format!(
        "set_dynamic_color 19 '{}'\n",
        color(palette, "base05")
    ));
    output.push('\n');
    output.push_str("# Cursor text should contrast with the cursor color; xterm OSC has no portable setter for it.\n");
    output.push_str(&format!(
        "# Intended cursor text color: {}\n",
        color(palette, "base00")
    ));
    output
}

fn ansi_colors(palette: &Base16Palette) -> Vec<(u8, String)> {
    vec![
        (0, color(palette, "base00").to_owned()),
        (1, color(palette, "base08").to_owned()),
        (2, color(palette, "base0B").to_owned()),
        (3, color(palette, "base0A").to_owned()),
        (4, color(palette, "base0D").to_owned()),
        (5, color(palette, "base0E").to_owned()),
        (6, color(palette, "base0C").to_owned()),
        (7, color(palette, "base05").to_owned()),
        (8, color(palette, "base03").to_owned()),
        (9, brighten(color(palette, "base08"))),
        (10, brighten(color(palette, "base0B"))),
        (11, brighten(color(palette, "base0A"))),
        (12, brighten(color(palette, "base0D"))),
        (13, brighten(color(palette, "base0E"))),
        (14, brighten(color(palette, "base0C"))),
        (15, color(palette, "base07").to_owned()),
    ]
}
