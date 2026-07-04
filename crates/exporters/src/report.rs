use palette16::Base16Palette;
use semantic_roles::{Role, SemanticRoles};
use serde::Serialize;
use theme_ir::{Confidence, ResolvedTheme, SourceProperty, Warning};

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

pub fn base16_preserved_items(
    roles: &SemanticRoles,
    palette: &Base16Palette,
) -> Vec<PreservedItem> {
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

pub fn role_source(value: &theme_ir::SemanticRoleValue) -> Option<String> {
    let scope = value.source_scope.as_ref()?;
    let property = value.source_property.unwrap_or(SourceProperty::Fg);
    Some(format!("{scope}.{property}"))
}

pub fn dropped_items(theme: &ResolvedTheme) -> Vec<String> {
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
