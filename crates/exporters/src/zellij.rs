use crate::report::{role_source, ExportReport, PreservedItem};
use palette16::Base16Palette;
use semantic_roles::{Role, SemanticRoles, SemanticUiStyle, SemanticUiStyles};
use theme_ir::{
    normalize_hex, parse_rgb, Confidence, Modifier, ResolvedTheme, SemanticRoleValue, Warning,
};

/// Export current component-based Zellij themes from the semantic pipeline.
pub fn export_zellij(
    theme: &ResolvedTheme,
    roles: &SemanticRoles,
    palette: &Base16Palette,
    ui: &SemanticUiStyles,
    warnings: Vec<Warning>,
) -> (String, ExportReport) {
    let name = quote(&theme.name);
    let mut output = format!("// Generated from Helix theme: {name}\n// Activate with: theme {name}\nthemes {{\n    {name} {{\n");
    let mut report = ExportReport {
        exporter: "zellij".into(),
        source: theme.source_path.to_string(),
        preserved: vec![],
        dropped: vec!["Helix syntax scopes collapse to Zellij UI component colors".into()],
        warnings,
    };
    let empty = SemanticUiStyle::default();
    let fg = &[Role::Foreground][..];
    let bg = &[Role::Background][..];
    let function = &[Role::Function][..];
    let components = [
        ("text_unselected", &empty, fg, "base05", bg, "base00"),
        (
            "text_selected",
            &ui.primary_selection,
            fg,
            "base05",
            &[Role::Selection],
            "base02",
        ),
        (
            "ribbon_unselected",
            &ui.menu,
            fg,
            "base05",
            &[Role::Surface],
            "base01",
        ),
        (
            "ribbon_selected",
            &ui.selected_menu,
            fg,
            "base05",
            &[Role::Selection],
            "base02",
        ),
        (
            "table_title",
            &ui.focused_control,
            function,
            "base0D",
            &[Role::Surface],
            "base01",
        ),
        (
            "table_cell_unselected",
            &ui.menu,
            fg,
            "base05",
            &[Role::Surface],
            "base01",
        ),
        (
            "table_cell_selected",
            &ui.selected_menu,
            fg,
            "base05",
            &[Role::Selection],
            "base02",
        ),
        (
            "list_unselected",
            &ui.menu,
            fg,
            "base05",
            &[Role::Surface],
            "base01",
        ),
        (
            "list_selected",
            &ui.selected_menu,
            fg,
            "base05",
            &[Role::Selection],
            "base02",
        ),
        (
            "frame_unselected",
            &ui.inactive_text,
            &[Role::MutedForeground],
            "base03",
            bg,
            "base00",
        ),
        (
            "frame_selected",
            &ui.focused_control,
            function,
            "base0D",
            bg,
            "base00",
        ),
        (
            "frame_highlight",
            &empty,
            &[Role::Warning],
            "base0A",
            bg,
            "base00",
        ),
        (
            "exit_code_success",
            &empty,
            &[Role::GitAdded, Role::String],
            "base0B",
            bg,
            "base00",
        ),
        (
            "exit_code_error",
            &empty,
            &[Role::Error],
            "base08",
            bg,
            "base00",
        ),
    ];
    for (component, style, fg_roles, fg_base, bg_roles, bg_base) in components {
        let mut foreground = resolve(style.fg.as_ref(), fg_roles, fg_base, roles, palette);
        let mut background = resolve(style.bg.as_ref(), bg_roles, bg_base, roles, palette);
        if style.modifiers.contains(&Modifier::Reversed) {
            std::mem::swap(&mut foreground, &mut background);
        }
        output.push_str(&format!("        {component} {{\n"));
        emit(&mut output, &mut report, component, "base", foreground);
        emit(
            &mut output,
            &mut report,
            component,
            "background",
            background,
        );
        for (property, role, base) in [
            ("emphasis_0", Role::Warning, "base0A"),
            ("emphasis_1", Role::Function, "base0D"),
            ("emphasis_2", Role::String, "base0B"),
            ("emphasis_3", Role::Keyword, "base0E"),
        ] {
            emit(
                &mut output,
                &mut report,
                component,
                property,
                resolve(None, &[role], base, roles, palette),
            );
        }
        output.push_str("        }\n");
    }
    output.push_str("        multiplayer_user_colors {\n");
    for (i, base) in [
        "base0E", "base0D", "base0B", "base0A", "base0C", "base08", "base09", "base0F", "base05",
        "base07",
    ]
    .iter()
    .enumerate()
    {
        emit(
            &mut output,
            &mut report,
            "multiplayer_user_colors",
            &format!("player_{}", i + 1),
            resolve(None, &[], base, roles, palette),
        );
    }
    output.push_str("        }\n    }\n}\n");
    if theme
        .scopes
        .values()
        .any(|s| s.modifiers.iter().any(|m| *m != Modifier::Reversed))
    {
        report.dropped.push("Helix bold, italic, dim, and crossed_out modifiers are not represented in Zellij theme colors".into());
    }
    if theme.scopes.values().any(|s| s.underline.is_some()) {
        report.dropped.push(
            "Helix underline colors and styles are not represented as Zellij underline styling"
                .into(),
        );
    }
    if theme
        .scopes
        .values()
        .any(|s| s.modifiers.contains(&Modifier::Reversed))
    {
        report.dropped.push(
            "Helix reversed styling is materialized only for mapped semantic UI components".into(),
        );
    }
    (output, report)
}

struct Color<'a> {
    value: &'a str,
    source: Option<String>,
}

fn from_value(value: &SemanticRoleValue) -> Option<Color<'_>> {
    Some(Color {
        value: value.color.as_deref()?,
        source: role_source(value),
    })
}

fn resolve<'a>(
    preferred: Option<&'a SemanticRoleValue>,
    candidates: &[Role],
    base: &str,
    roles: &'a SemanticRoles,
    palette: &'a Base16Palette,
) -> Color<'a> {
    if let Some(color) = preferred.and_then(from_value) {
        return color;
    }
    for role in candidates {
        if let Some(color) = roles.get(role).and_then(from_value) {
            return color;
        }
    }
    if let Some(entry) = palette.get(base) {
        // An inferred palette entry may carry a source role without preserving its RGB.
        let source = entry
            .source_role
            .and_then(|role| roles.get(&role))
            .filter(|value| {
                entry.confidence != Confidence::Inferred
                    && value.color.as_deref() == Some(entry.color.as_str())
            })
            .and_then(role_source);
        Color {
            value: &entry.color,
            source,
        }
    } else {
        Color {
            value: "#000000",
            source: None,
        }
    }
}

fn emit(
    output: &mut String,
    report: &mut ExportReport,
    component: &str,
    property: &str,
    color: Color<'_>,
) {
    let target = format!("{component}.{property}");
    let rgb = match parse_rgb(color.value) {
        Ok(rgb) => rgb,
        Err(error) => {
            report.warnings.push(Warning::new(
                "zellij_invalid_color",
                format!("{target}: {} ({error}); using black", color.value),
            ));
            output.push_str(&format!("            {property} 0 0 0\n"));
            return;
        }
    };
    output.push_str(&format!(
        "            {property} {} {} {}\n",
        rgb.r, rgb.g, rgb.b
    ));
    if normalize_hex(color.value).is_ok_and(|value| value.len() == 9) {
        report.warnings.push(Warning::new(
            "zellij_alpha_unsupported",
            format!("{target}: discarded alpha from {}", color.value),
        ));
    } else if let Some(source) = color.source {
        report.preserved.push(PreservedItem { target, source });
    }
}

fn quote(value: &str) -> String {
    let mut result = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c.is_control() || matches!(c, '\u{2028}' | '\u{2029}') => {
                result.push_str(&format!("\\u{{{:x}}}", c as u32))
            }
            c => result.push(c),
        }
    }
    result.push('"');
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use helix_theme::{parse_str, resolve_raw};
    use palette16::extract_base16;
    use semantic_roles::{derive_roles, derive_ui_styles};

    fn pipeline(
        input: &str,
    ) -> (
        ResolvedTheme,
        SemanticRoles,
        Base16Palette,
        SemanticUiStyles,
    ) {
        let theme = resolve_raw(parse_str("minimal", "minimal.toml".into(), input).unwrap());
        let (roles, _) = derive_roles(&theme);
        let (palette, _) = extract_base16(&roles);
        let ui = derive_ui_styles(&theme);
        (theme, roles, palette, ui)
    }

    fn export(input: &str) -> (String, ExportReport) {
        let (theme, roles, palette, ui) = pipeline(input);
        export_zellij(&theme, &roles, &palette, &ui, vec![])
    }

    fn components(output: &str) -> kdl::KdlDocument {
        output
            .parse::<kdl::KdlDocument>()
            .unwrap()
            .get("themes")
            .unwrap()
            .children()
            .unwrap()
            .nodes()[0]
            .children()
            .unwrap()
            .clone()
    }

    fn rgb(doc: &kdl::KdlDocument, component: &str, property: &str) -> Vec<i64> {
        doc.get(component)
            .unwrap()
            .children()
            .unwrap()
            .get(property)
            .unwrap()
            .entries()
            .iter()
            .map(|entry| entry.value().as_i64().unwrap())
            .collect()
    }

    #[test]
    fn golden_and_complete_valid_kdl() {
        let (output, _) = export(include_str!("../../../tests/fixtures/minimal.toml"));
        assert_eq!(
            output,
            include_str!("../../../tests/golden/zellij/minimal.kdl")
        );
        let doc = components(&output);
        assert_eq!(doc.nodes().len(), 15);
        for node in &doc.nodes()[..14] {
            let properties = node.children().unwrap();
            assert_eq!(properties.nodes().len(), 6);
            for property in properties.nodes() {
                assert_eq!(property.entries().len(), 3);
                assert!(property
                    .entries()
                    .iter()
                    .all(|entry| (0..=255).contains(&entry.value().as_i64().unwrap())));
            }
        }
        assert_eq!(
            doc.get("multiplayer_user_colors")
                .unwrap()
                .children()
                .unwrap()
                .nodes()
                .len(),
            10
        );
        assert_eq!(rgb(&doc, "text_unselected", "background"), [31, 35, 53]);
        assert_eq!(
            rgb(&doc, "multiplayer_user_colors", "player_1"),
            [187, 154, 247]
        );
    }

    #[test]
    fn partial_styles_and_reversed_pairs_keep_property_provenance() {
        let (output, report) = export(
            r##"
"ui.text" = "#abcdef"
"ui.menu" = { fg = "#123456", bg = "#334455" }
"ui.menu.selected" = { bg = "#456789", modifiers = ["reversed", "bold"] }
"ui.selection.primary" = { bg = "#778899" }
"ui.selection" = { fg = "#aabbcc" }
"ui.text.focus" = "#fedcba"
"ui.text.inactive" = "#987654"
"function" = "#987abc"
"diagnostic.error" = { underline = { color = "#ff0000", style = "curl" } }
"string" = "#00ff00"
"diff.plus" = "#22aa33"
"diagnostic.warning" = "#ffaa00"
"type" = "#0000ff"
"##,
        );
        let doc = components(&output);
        assert_eq!(rgb(&doc, "ribbon_selected", "base"), [69, 103, 137]);
        assert_eq!(rgb(&doc, "ribbon_selected", "background"), [18, 52, 86]);
        assert_eq!(rgb(&doc, "text_selected", "base"), [170, 187, 204]);
        assert_eq!(rgb(&doc, "text_selected", "background"), [119, 136, 153]);
        assert_eq!(rgb(&doc, "frame_selected", "base"), [254, 220, 186]);
        assert_eq!(rgb(&doc, "frame_unselected", "base"), [152, 118, 84]);
        assert_eq!(rgb(&doc, "exit_code_success", "base"), [34, 170, 51]);
        assert_eq!(rgb(&doc, "frame_highlight", "base"), [255, 170, 0]);
        assert!(report
            .preserved
            .iter()
            .any(|p| p.target == "ribbon_selected.background" && p.source == "ui.menu.fg"));
        assert!(report
            .preserved
            .iter()
            .any(|p| p.target == "exit_code_error.base"
                && p.source == "diagnostic.error.underline.color"));
        assert!(report.dropped.iter().any(|s| s.contains("reversed")));
        assert!(report.dropped.iter().any(|s| s.contains("underline")));
        assert!(report.dropped.iter().any(|s| s.contains("bold")));
    }

    #[test]
    fn missing_styles_and_roles_use_palette_without_false_preservation() {
        let (output, report) = export("");
        let doc = components(&output);
        assert_eq!(rgb(&doc, "text_unselected", "base"), [255, 255, 255]);
        assert_eq!(rgb(&doc, "text_unselected", "background"), [0, 0, 0]);
        assert!(report.preserved.is_empty());
        let (_, report) =
            export("\"ui.background\" = { bg = \"#000000\" }\n\"ui.text\" = \"#ffffff\"");
        assert!(!report
            .preserved
            .iter()
            .any(|p| p.target == "ribbon_unselected.background"));
        assert!(!report
            .preserved
            .iter()
            .any(|p| p.target == "multiplayer_user_colors.player_10"));
    }

    #[test]
    fn alpha_is_dropped_with_warning_and_not_preserved() {
        let (output, report) = export("\"ui.text\" = \"#abcd\"");
        assert_eq!(
            rgb(&components(&output), "text_unselected", "base"),
            [170, 187, 204]
        );
        assert!(report
            .warnings
            .iter()
            .any(|w| w.code == "zellij_alpha_unsupported"));
        assert!(!report
            .preserved
            .iter()
            .any(|p| p.target == "text_unselected.base"));
    }

    #[test]
    fn names_round_trip_without_injecting_kdl_or_comments() {
        let (mut theme, roles, palette, ui) = pipeline("");
        theme.name = "quote\"\\\n\r\t\u{0001}\u{2028}雪".into();
        let (output, _) = export_zellij(&theme, &roles, &palette, &ui, vec![]);
        let doc: kdl::KdlDocument = output.parse().unwrap();
        assert_eq!(doc.nodes().len(), 1);
        assert_eq!(
            doc.get("themes").unwrap().children().unwrap().nodes()[0]
                .name()
                .value(),
            theme.name
        );
        let activation = output
            .lines()
            .nth(1)
            .unwrap()
            .strip_prefix("// Activate with: ")
            .unwrap();
        let activation: kdl::KdlDocument = activation.parse().unwrap();
        assert_eq!(
            activation.get("theme").unwrap().entries()[0]
                .value()
                .as_string(),
            Some(theme.name.as_str())
        );
    }

    #[test]
    fn format_path_and_inherited_warnings_are_stable() {
        let (theme, roles, palette, ui) = pipeline("");
        let warning = Warning::new("upstream", "example");
        let format =
            crate::export_zellij_format(&theme, &roles, &palette, &ui, vec![warning.clone()]);
        assert_eq!(format.files.len(), 1);
        assert_eq!(format.files[0].relative_path, "zellij/theme.kdl");
        assert!(!format.files[0].executable);
        assert_eq!(format.report.exporter, "zellij");
        assert_eq!(format.report.warnings, vec![warning]);
    }

    #[test]
    fn real_dark_and_light_themes_and_preview_layout_parse() {
        for input in [
            include_str!("../../../tests/fixtures/mc/adwaita-dark.toml"),
            include_str!("../../../tests/fixtures/mc/adwaita-light.toml"),
        ] {
            let (output, _) = export(input);
            assert_eq!(components(&output).nodes().len(), 15);
        }
        let script = include_str!("../../../scripts/tool-test-scripts/zellij.sh");
        let layout = script
            .split("<<'KDL'\n")
            .nth(2)
            .unwrap()
            .split("\nKDL")
            .next()
            .unwrap();
        let layout: kdl::KdlDocument = layout.parse().unwrap();
        assert!(layout.get("layout").is_some());
    }
}
