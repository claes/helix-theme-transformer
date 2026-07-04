use indexmap::IndexMap;
use palette16::{color, compact_colors, Base16Palette};
use semantic_roles::{role_color, Role, SemanticRoles};
use serde::Serialize;
use theme_ir::{brighten, Confidence, Modifier, ResolvedTheme, SourceProperty, Warning};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExportReport {
    pub exporter: String,
    pub source: String,
    pub preserved: Vec<PreservedItem>,
    pub dropped: Vec<String>,
    pub warnings: Vec<Warning>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PreservedItem {
    pub target: String,
    pub source: String,
}

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
        preserved: preserved_items(roles, palette),
        dropped: dropped_items(theme),
        warnings,
    };
    (output, report)
}

pub fn export_base16_yaml(palette: &Base16Palette) -> Result<String, serde_yaml::Error> {
    serde_yaml::to_string(&compact_colors(palette))
}

pub fn export_bat_tmtheme(
    theme: &ResolvedTheme,
    roles: &SemanticRoles,
    palette: &Base16Palette,
    warnings: Vec<Warning>,
) -> (String, ExportReport) {
    let mut output = String::new();
    output.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    output.push_str("<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n");
    output.push_str("<plist version=\"1.0\">\n");
    output.push_str("<dict>\n");
    push_key_string(&mut output, 1, "name", &theme.name);
    output.push_str("  <key>settings</key>\n");
    output.push_str("  <array>\n");
    output.push_str("    <dict>\n");
    output.push_str("      <key>settings</key>\n");
    output.push_str("      <dict>\n");
    push_key_string(
        &mut output,
        4,
        "background",
        role_color(roles, Role::Background).unwrap_or(color(palette, "base00")),
    );
    push_key_string(
        &mut output,
        4,
        "foreground",
        role_color(roles, Role::Foreground).unwrap_or(color(palette, "base05")),
    );
    push_key_string(
        &mut output,
        4,
        "caret",
        role_color(roles, Role::Cursor).unwrap_or(color(palette, "base05")),
    );
    push_key_string(
        &mut output,
        4,
        "selection",
        role_color(roles, Role::Selection).unwrap_or(color(palette, "base02")),
    );
    push_key_string(
        &mut output,
        4,
        "lineHighlight",
        role_color(roles, Role::Surface).unwrap_or(color(palette, "base01")),
    );
    output.push_str("      </dict>\n");
    output.push_str("    </dict>\n");

    for mapping in bat_scope_mappings() {
        let fallback = mapping
            .fallback_base
            .map(|base| color(palette, base))
            .unwrap_or(color(palette, "base05"));
        let color = role_color(roles, mapping.role).unwrap_or(fallback);
        push_scope_setting(&mut output, theme, roles, mapping, color);
    }

    output.push_str("  </array>\n");
    output.push_str("</dict>\n");
    output.push_str("</plist>\n");

    let report = ExportReport {
        exporter: "bat".to_owned(),
        source: theme.source_path.to_string(),
        preserved: bat_preserved_items(roles),
        dropped: dropped_items(theme),
        warnings,
    };
    (output, report)
}

pub fn render_report(report: &ExportReport) -> String {
    let mut output = format!(
        "Export report: {}\nSource: {}\n\n",
        report.exporter, report.source
    );
    output.push_str("Preserved:\n");
    if report.preserved.is_empty() {
        output.push_str("  none\n");
    } else {
        for item in &report.preserved {
            output.push_str(&format!("  {}: {}\n", item.target, item.source));
        }
    }
    output.push_str("\nDropped:\n");
    if report.dropped.is_empty() {
        output.push_str("  none\n");
    } else {
        for item in &report.dropped {
            output.push_str(&format!("  {item}\n"));
        }
    }
    output.push_str("\nWarnings:\n");
    if report.warnings.is_empty() {
        output.push_str("  none\n");
    } else {
        for warning in &report.warnings {
            output.push_str(&format!("  {}: {}\n", warning.code, warning.message));
        }
    }
    output
}

fn preserved_items(roles: &SemanticRoles, palette: &Base16Palette) -> Vec<PreservedItem> {
    let mut items = Vec::new();
    for (target, base) in [
        ("background", "base00"),
        ("foreground", "base05"),
        ("color1", "base08"),
        ("color2", "base0B"),
        ("color3", "base0A"),
        ("color4", "base0D"),
        ("color5", "base0E"),
        ("color6", "base0C"),
    ] {
        if let Some(source) = source_for_base(roles, palette, base) {
            items.push(PreservedItem {
                target: target.to_owned(),
                source,
            });
        }
    }
    if let Some(cursor) = roles.get(&Role::Cursor) {
        if cursor.confidence != Confidence::Missing {
            if let Some(source) = role_source(cursor) {
                items.push(PreservedItem {
                    target: "cursor".to_owned(),
                    source,
                });
            }
        }
    }
    items
}

fn source_for_base(
    roles: &SemanticRoles,
    palette: &Base16Palette,
    base_key: &str,
) -> Option<String> {
    let role = palette.get(base_key)?.source_role?;
    role_source(roles.get(&role)?)
}

fn role_source(value: &theme_ir::SemanticRoleValue) -> Option<String> {
    let scope = value.source_scope.as_ref()?;
    let property = value.source_property.unwrap_or(SourceProperty::Fg);
    Some(format!("{scope}.{property}"))
}

#[derive(Debug, Clone, Copy)]
struct BatScopeMapping {
    name: &'static str,
    scope: &'static str,
    role: Role,
    fallback_base: Option<&'static str>,
}

fn bat_scope_mappings() -> &'static [BatScopeMapping] {
    &[
        BatScopeMapping {
            name: "Comment",
            scope: "comment",
            role: Role::Comment,
            fallback_base: Some("base03"),
        },
        BatScopeMapping {
            name: "Keyword",
            scope: "keyword",
            role: Role::Keyword,
            fallback_base: Some("base0E"),
        },
        BatScopeMapping {
            name: "Function",
            scope: "entity.name.function",
            role: Role::Function,
            fallback_base: Some("base0D"),
        },
        BatScopeMapping {
            name: "Support Function",
            scope: "support.function",
            role: Role::Function,
            fallback_base: Some("base0D"),
        },
        BatScopeMapping {
            name: "Storage Type",
            scope: "storage.type",
            role: Role::Type,
            fallback_base: Some("base0A"),
        },
        BatScopeMapping {
            name: "Type",
            scope: "entity.name.type",
            role: Role::Type,
            fallback_base: Some("base0A"),
        },
        BatScopeMapping {
            name: "Variable",
            scope: "variable",
            role: Role::Variable,
            fallback_base: Some("base05"),
        },
        BatScopeMapping {
            name: "Parameter",
            scope: "variable.parameter",
            role: Role::Parameter,
            fallback_base: Some("base05"),
        },
        BatScopeMapping {
            name: "String",
            scope: "string",
            role: Role::String,
            fallback_base: Some("base0B"),
        },
        BatScopeMapping {
            name: "Number",
            scope: "constant.numeric",
            role: Role::Number,
            fallback_base: Some("base09"),
        },
        BatScopeMapping {
            name: "Language Constant",
            scope: "constant.language",
            role: Role::Constant,
            fallback_base: Some("base09"),
        },
        BatScopeMapping {
            name: "Other Constant",
            scope: "constant.other",
            role: Role::Constant,
            fallback_base: Some("base09"),
        },
        BatScopeMapping {
            name: "Operator",
            scope: "keyword.operator",
            role: Role::Operator,
            fallback_base: Some("base0F"),
        },
        BatScopeMapping {
            name: "Tag",
            scope: "entity.name.tag",
            role: Role::Special,
            fallback_base: Some("base0C"),
        },
        BatScopeMapping {
            name: "Support",
            scope: "support",
            role: Role::Special,
            fallback_base: Some("base0C"),
        },
        BatScopeMapping {
            name: "Language Variable",
            scope: "variable.language",
            role: Role::Special,
            fallback_base: Some("base0C"),
        },
        BatScopeMapping {
            name: "Invalid",
            scope: "invalid",
            role: Role::Error,
            fallback_base: Some("base08"),
        },
    ]
}

fn push_scope_setting(
    output: &mut String,
    theme: &ResolvedTheme,
    roles: &SemanticRoles,
    mapping: &BatScopeMapping,
    color: &str,
) {
    output.push_str("    <dict>\n");
    push_key_string(output, 3, "name", mapping.name);
    push_key_string(output, 3, "scope", mapping.scope);
    output.push_str("      <key>settings</key>\n");
    output.push_str("      <dict>\n");
    push_key_string(output, 4, "foreground", color);
    if let Some(font_style) = bat_font_style(theme, roles, mapping.role) {
        push_key_string(output, 4, "fontStyle", &font_style);
    }
    output.push_str("      </dict>\n");
    output.push_str("    </dict>\n");
}

fn bat_font_style(theme: &ResolvedTheme, roles: &SemanticRoles, role: Role) -> Option<String> {
    let source_scope = roles.get(&role)?.source_scope.as_ref()?;
    let style = theme.scopes.get(source_scope.as_str())?;
    let bold = style.modifiers.contains(&Modifier::Bold);
    let italic = style.modifiers.contains(&Modifier::Italic);
    match (bold, italic) {
        (true, true) => Some("bold italic".to_owned()),
        (true, false) => Some("bold".to_owned()),
        (false, true) => Some("italic".to_owned()),
        (false, false) => None,
    }
}

fn bat_preserved_items(roles: &SemanticRoles) -> Vec<PreservedItem> {
    let mut items = Vec::new();
    for (target, role) in [
        ("background", Role::Background),
        ("foreground", Role::Foreground),
        ("caret", Role::Cursor),
        ("selection", Role::Selection),
        ("lineHighlight", Role::Surface),
        ("comment", Role::Comment),
        ("keyword", Role::Keyword),
        ("entity.name.function", Role::Function),
        ("storage.type", Role::Type),
        ("string", Role::String),
        ("constant.numeric", Role::Number),
        ("keyword.operator", Role::Operator),
        ("entity.name.tag", Role::Special),
        ("invalid", Role::Error),
    ] {
        if let Some(value) = roles.get(&role) {
            if let Some(source) = role_source(value) {
                items.push(PreservedItem {
                    target: target.to_owned(),
                    source,
                });
            }
        }
    }
    items
}

fn push_key_string(output: &mut String, indent_level: usize, key: &str, value: &str) {
    let indent = "  ".repeat(indent_level);
    output.push_str(&format!("{indent}<key>{}</key>\n", xml_escape(key)));
    output.push_str(&format!("{indent}<string>{}</string>\n", xml_escape(value)));
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn dropped_items(theme: &ResolvedTheme) -> Vec<String> {
    let modifier_count = theme
        .scopes
        .values()
        .filter(|style| !style.modifiers.is_empty())
        .count();
    let underline_count = theme
        .scopes
        .values()
        .filter(|style| style.underline.is_some())
        .count();
    let mut dropped = Vec::new();
    if modifier_count > 0 {
        dropped.push(format!("{modifier_count} modifier-bearing scopes"));
    }
    if underline_count > 0 {
        dropped.push(format!("{underline_count} underline-bearing scopes"));
    }
    let syntax_count = theme
        .scopes
        .keys()
        .filter(|scope| !scope.starts_with("ui.") && !scope.starts_with("diagnostic."))
        .count();
    if syntax_count > 16 {
        dropped.push(format!("{} syntax scopes", syntax_count.saturating_sub(16)));
    }
    dropped
}

pub fn yaml_from_map(map: IndexMap<String, String>) -> Result<String, serde_yaml::Error> {
    serde_yaml::to_string(&map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use camino::Utf8PathBuf;
    use helix_theme::{parse_str, resolve_raw};
    use palette16::extract_base16;
    use semantic_roles::derive_roles;

    #[test]
    fn exports_kitty_golden_minimal() {
        let (theme, roles, palette, mut warnings) = minimal_pipeline();
        warnings.extend(theme.warnings.clone());
        let (kitty, _) = export_kitty(&theme, &roles, &palette, warnings);
        assert_eq!(
            kitty,
            include_str!("../../../tests/golden/kitty/minimal.conf")
        );
    }

    #[test]
    fn exports_base16_golden_minimal() {
        let (_, _, palette, _) = minimal_pipeline();
        let yaml = export_base16_yaml(&palette).unwrap();
        assert_eq!(
            yaml,
            include_str!("../../../tests/golden/base16/minimal.yaml")
        );
    }

    #[test]
    fn exports_bat_golden_minimal() {
        let (theme, roles, palette, mut warnings) = minimal_pipeline();
        warnings.extend(theme.warnings.clone());
        let (tmtheme, _) = export_bat_tmtheme(&theme, &roles, &palette, warnings);
        assert_eq!(
            tmtheme,
            include_str!("../../../tests/golden/bat/minimal.tmTheme")
        );
    }

    fn minimal_pipeline() -> (
        theme_ir::ResolvedTheme,
        semantic_roles::SemanticRoles,
        palette16::Base16Palette,
        Vec<theme_ir::Warning>,
    ) {
        let raw = parse_str(
            "minimal",
            Utf8PathBuf::from("minimal.toml"),
            include_str!("../../../tests/fixtures/minimal.toml"),
        )
        .unwrap();
        let theme = resolve_raw(raw);
        let (roles, mut warnings) = derive_roles(&theme);
        let (palette, palette_warnings) = extract_base16(&roles);
        warnings.extend(palette_warnings);
        (theme, roles, palette, warnings)
    }
}
