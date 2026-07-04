use crate::report::{base16_preserved_items, dropped_items, ExportReport};
use palette16::{color, Base16Palette};
use semantic_roles::{role_color, Role, SemanticRoles};
use theme_ir::{brighten, ResolvedTheme, Warning};

pub fn export_kitty(
    theme: &ResolvedTheme,
    roles: &SemanticRoles,
    palette: &Base16Palette,
    warnings: Vec<Warning>,
) -> (String, ExportReport) {
    let cursor = role_color(roles, Role::Cursor).unwrap_or(color(palette, "base05"));
    let lines = [
        format!("# Generated from Helix theme: {}", theme.name),
        format!("foreground {}", color(palette, "base05")),
        format!("background {}", color(palette, "base00")),
        String::new(),
        format!("selection_foreground {}", color(palette, "base05")),
        format!("selection_background {}", color(palette, "base02")),
        String::new(),
        format!("cursor {cursor}"),
        format!("cursor_text_color {}", color(palette, "base00")),
        String::new(),
        format!("color0 {}", color(palette, "base00")),
        format!("color1 {}", color(palette, "base08")),
        format!("color2 {}", color(palette, "base0B")),
        format!("color3 {}", color(palette, "base0A")),
        format!("color4 {}", color(palette, "base0D")),
        format!("color5 {}", color(palette, "base0E")),
        format!("color6 {}", color(palette, "base0C")),
        format!("color7 {}", color(palette, "base05")),
        String::new(),
        format!("color8 {}", color(palette, "base03")),
        format!("color9 {}", brighten(color(palette, "base08"))),
        format!("color10 {}", brighten(color(palette, "base0B"))),
        format!("color11 {}", brighten(color(palette, "base0A"))),
        format!("color12 {}", brighten(color(palette, "base0D"))),
        format!("color13 {}", brighten(color(palette, "base0E"))),
        format!("color14 {}", brighten(color(palette, "base0C"))),
        format!("color15 {}", color(palette, "base07")),
    ];
    let output = format!("{}\n", lines.join("\n"));
    let report = ExportReport {
        exporter: "kitty".to_owned(),
        source: theme.source_path.to_string(),
        preserved: base16_preserved_items(roles, palette),
        dropped: dropped_items(theme),
        warnings,
    };
    (output, report)
}
