use crate::report::{dropped_items, role_source, ExportReport, PreservedItem};
use palette16::{color, Base16Palette};
use semantic_roles::{role_color, Role, SemanticRoles};
use theme_ir::{Modifier, ResolvedTheme, Warning};

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

#[derive(Debug, Clone, Copy)]
struct BatScopeMapping {
    name: &'static str,
    scope: &'static str,
    role: Role,
    fallback_base: Option<&'static str>,
    report_target: bool,
}

fn bat_scope_mappings() -> &'static [BatScopeMapping] {
    &[
        BatScopeMapping {
            name: "Comment",
            scope: "comment",
            role: Role::Comment,
            fallback_base: Some("base03"),
            report_target: true,
        },
        BatScopeMapping {
            name: "Keyword",
            scope: "keyword",
            role: Role::Keyword,
            fallback_base: Some("base0E"),
            report_target: true,
        },
        BatScopeMapping {
            name: "Function",
            scope: "entity.name.function",
            role: Role::Function,
            fallback_base: Some("base0D"),
            report_target: true,
        },
        BatScopeMapping {
            name: "Support Function",
            scope: "support.function",
            role: Role::Function,
            fallback_base: Some("base0D"),
            report_target: false,
        },
        BatScopeMapping {
            name: "Storage Type",
            scope: "storage.type",
            role: Role::Type,
            fallback_base: Some("base0A"),
            report_target: true,
        },
        BatScopeMapping {
            name: "Type",
            scope: "entity.name.type",
            role: Role::Type,
            fallback_base: Some("base0A"),
            report_target: false,
        },
        BatScopeMapping {
            name: "Variable",
            scope: "variable",
            role: Role::Variable,
            fallback_base: Some("base05"),
            report_target: false,
        },
        BatScopeMapping {
            name: "Parameter",
            scope: "variable.parameter",
            role: Role::Parameter,
            fallback_base: Some("base05"),
            report_target: false,
        },
        BatScopeMapping {
            name: "String",
            scope: "string",
            role: Role::String,
            fallback_base: Some("base0B"),
            report_target: true,
        },
        BatScopeMapping {
            name: "Number",
            scope: "constant.numeric",
            role: Role::Number,
            fallback_base: Some("base09"),
            report_target: true,
        },
        BatScopeMapping {
            name: "Language Constant",
            scope: "constant.language",
            role: Role::Constant,
            fallback_base: Some("base09"),
            report_target: false,
        },
        BatScopeMapping {
            name: "Other Constant",
            scope: "constant.other",
            role: Role::Constant,
            fallback_base: Some("base09"),
            report_target: false,
        },
        BatScopeMapping {
            name: "Operator",
            scope: "keyword.operator",
            role: Role::Operator,
            fallback_base: Some("base0F"),
            report_target: true,
        },
        BatScopeMapping {
            name: "Tag",
            scope: "entity.name.tag",
            role: Role::Special,
            fallback_base: Some("base0C"),
            report_target: true,
        },
        BatScopeMapping {
            name: "Support",
            scope: "support",
            role: Role::Special,
            fallback_base: Some("base0C"),
            report_target: false,
        },
        BatScopeMapping {
            name: "Language Variable",
            scope: "variable.language",
            role: Role::Special,
            fallback_base: Some("base0C"),
            report_target: false,
        },
        BatScopeMapping {
            name: "Invalid",
            scope: "invalid",
            role: Role::Error,
            fallback_base: Some("base08"),
            report_target: true,
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
    for mapping in bat_scope_mappings() {
        if !mapping.report_target {
            continue;
        }
        if let Some(value) = roles.get(&mapping.role) {
            if let Some(source) = role_source(value) {
                items.push(PreservedItem {
                    target: mapping.scope.to_owned(),
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
