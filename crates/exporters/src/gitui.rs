use crate::bat::export_bat_tmtheme;
use crate::report::{dropped_items, role_source, ExportReport, PreservedItem};
use palette16::{color, Base16Palette};
use semantic_roles::{role_color, Role, SemanticRoles};
use theme_ir::{ResolvedTheme, Warning};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GituiTheme {
    pub theme_ron: String,
    pub syntax_file_name: String,
    pub syntax_tmtheme: String,
    pub report: ExportReport,
}

pub fn export_gitui(
    theme: &ResolvedTheme,
    roles: &SemanticRoles,
    palette: &Base16Palette,
    warnings: Vec<Warning>,
) -> GituiTheme {
    let syntax_name = syntax_name(theme);
    let (syntax_tmtheme, _) = export_bat_tmtheme(theme, roles, palette, Vec::new());
    let theme_ron = render_theme_ron(&syntax_name, roles, palette);
    let report = ExportReport {
        exporter: "gitui".to_owned(),
        source: theme.source_path.to_string(),
        preserved: gitui_preserved_items(roles),
        dropped: dropped_items(theme),
        warnings,
    };

    GituiTheme {
        theme_ron,
        syntax_file_name: format!("{syntax_name}.tmTheme"),
        syntax_tmtheme,
        report,
    }
}

fn render_theme_ron(syntax_name: &str, roles: &SemanticRoles, palette: &Base16Palette) -> String {
    let mut output = String::new();
    output.push_str("(\n");
    for mapping in gitui_mappings() {
        output.push_str(&format!(
            "  {}: Some(\"{}\"),\n",
            mapping.field,
            color_for_mapping(mapping, roles, palette)
        ));
    }
    output.push_str(&format!(
        "  syntax: Some(\"{}\"),\n",
        ron_escape(syntax_name)
    ));
    output.push_str(")\n");
    output
}

#[derive(Debug, Clone, Copy)]
struct GituiMapping {
    field: &'static str,
    roles: &'static [Role],
    fallback_base: &'static str,
}

fn gitui_mappings() -> &'static [GituiMapping] {
    &[
        GituiMapping {
            field: "selected_tab",
            roles: &[Role::Special],
            fallback_base: "base0C",
        },
        GituiMapping {
            field: "command_fg",
            roles: &[Role::Foreground],
            fallback_base: "base05",
        },
        GituiMapping {
            field: "selection_bg",
            roles: &[Role::Selection],
            fallback_base: "base02",
        },
        GituiMapping {
            field: "selection_fg",
            roles: &[Role::Foreground],
            fallback_base: "base05",
        },
        GituiMapping {
            field: "cmdbar_bg",
            roles: &[Role::Surface],
            fallback_base: "base01",
        },
        GituiMapping {
            field: "disabled_fg",
            roles: &[Role::MutedForeground],
            fallback_base: "base03",
        },
        GituiMapping {
            field: "diff_line_add",
            roles: &[Role::GitAdded],
            fallback_base: "base0B",
        },
        GituiMapping {
            field: "diff_line_delete",
            roles: &[Role::GitRemoved, Role::Error],
            fallback_base: "base08",
        },
        GituiMapping {
            field: "diff_file_added",
            roles: &[Role::GitAdded],
            fallback_base: "base0B",
        },
        GituiMapping {
            field: "diff_file_removed",
            roles: &[Role::GitRemoved, Role::Error],
            fallback_base: "base08",
        },
        GituiMapping {
            field: "diff_file_moved",
            roles: &[Role::Special],
            fallback_base: "base0C",
        },
        GituiMapping {
            field: "diff_file_modified",
            roles: &[Role::GitModified, Role::Warning],
            fallback_base: "base0A",
        },
        GituiMapping {
            field: "commit_hash",
            roles: &[Role::Constant],
            fallback_base: "base09",
        },
        GituiMapping {
            field: "commit_time",
            roles: &[Role::Info, Role::MutedForeground],
            fallback_base: "base03",
        },
        GituiMapping {
            field: "commit_author",
            roles: &[Role::Variable, Role::Function],
            fallback_base: "base0D",
        },
        GituiMapping {
            field: "danger_fg",
            roles: &[Role::Error],
            fallback_base: "base08",
        },
        GituiMapping {
            field: "push_gauge_bg",
            roles: &[Role::Selection],
            fallback_base: "base02",
        },
        GituiMapping {
            field: "push_gauge_fg",
            roles: &[Role::Foreground],
            fallback_base: "base05",
        },
        GituiMapping {
            field: "tag_fg",
            roles: &[Role::Special],
            fallback_base: "base0C",
        },
        GituiMapping {
            field: "branch_fg",
            roles: &[Role::Type],
            fallback_base: "base0A",
        },
        GituiMapping {
            field: "block_title_focused",
            roles: &[Role::BrightForeground],
            fallback_base: "base07",
        },
    ]
}

fn color_for_mapping<'a>(
    mapping: &GituiMapping,
    roles: &'a SemanticRoles,
    palette: &'a Base16Palette,
) -> &'a str {
    for role in mapping.roles {
        if let Some(color) = role_color(roles, *role) {
            return color;
        }
    }
    color(palette, mapping.fallback_base)
}

fn gitui_preserved_items(roles: &SemanticRoles) -> Vec<PreservedItem> {
    let mut items = Vec::new();
    for mapping in gitui_mappings() {
        for role in mapping.roles {
            if let Some(value) = roles.get(role) {
                if let Some(source) = role_source(value) {
                    items.push(PreservedItem {
                        target: mapping.field.to_owned(),
                        source,
                    });
                    break;
                }
            }
        }
    }
    items
}

fn syntax_name(theme: &ResolvedTheme) -> String {
    theme
        .name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect()
}

fn ron_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
