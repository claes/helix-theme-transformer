use crate::bat::export_bat_tmtheme;
use crate::file_kinds::{push_file_kind_source, resolve_file_kind_color, FileKind};
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
    let (syntax_tmtheme, _) = export_bat_tmtheme(theme, roles, palette, Vec::new());
    let theme_ron = render_theme_ron("syntax", roles, palette);
    let report = ExportReport {
        exporter: "gitui".to_owned(),
        source: theme.source_path.to_string(),
        preserved: gitui_preserved_items(roles),
        dropped: dropped_items(theme),
        warnings,
    };

    GituiTheme {
        theme_ron,
        syntax_file_name: "syntax.tmTheme".to_owned(),
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
            color_for_mapping(&mapping, roles, palette)
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
    color: GituiColor,
}

#[derive(Debug, Clone, Copy)]
enum GituiColor {
    Role {
        roles: &'static [Role],
        fallback_base: &'static str,
    },
    FileKind(FileKind),
}

fn gitui_mappings() -> Vec<GituiMapping> {
    vec![
        GituiMapping {
            field: "selected_tab",
            color: role(&[Role::Special], "base0C"),
        },
        GituiMapping {
            field: "command_fg",
            color: role(&[Role::Foreground], "base05"),
        },
        GituiMapping {
            field: "selection_bg",
            color: role(&[Role::Selection], "base02"),
        },
        GituiMapping {
            field: "selection_fg",
            color: role(&[Role::Foreground], "base05"),
        },
        GituiMapping {
            field: "cmdbar_bg",
            color: role(&[Role::Surface], "base01"),
        },
        GituiMapping {
            field: "disabled_fg",
            color: role(&[Role::MutedForeground], "base03"),
        },
        GituiMapping {
            field: "diff_line_add",
            color: GituiColor::FileKind(FileKind::GitAdded),
        },
        GituiMapping {
            field: "diff_line_delete",
            color: GituiColor::FileKind(FileKind::GitRemoved),
        },
        GituiMapping {
            field: "diff_file_added",
            color: GituiColor::FileKind(FileKind::GitAdded),
        },
        GituiMapping {
            field: "diff_file_removed",
            color: GituiColor::FileKind(FileKind::GitRemoved),
        },
        GituiMapping {
            field: "diff_file_moved",
            color: GituiColor::FileKind(FileKind::GitMoved),
        },
        GituiMapping {
            field: "diff_file_modified",
            color: GituiColor::FileKind(FileKind::GitModified),
        },
        GituiMapping {
            field: "commit_hash",
            color: role(&[Role::Constant], "base09"),
        },
        GituiMapping {
            field: "commit_time",
            color: role(&[Role::Info, Role::MutedForeground], "base03"),
        },
        GituiMapping {
            field: "commit_author",
            color: role(&[Role::Variable, Role::Function], "base0D"),
        },
        GituiMapping {
            field: "danger_fg",
            color: role(&[Role::Error], "base08"),
        },
        GituiMapping {
            field: "push_gauge_bg",
            color: role(&[Role::Selection], "base02"),
        },
        GituiMapping {
            field: "push_gauge_fg",
            color: role(&[Role::Foreground], "base05"),
        },
        GituiMapping {
            field: "tag_fg",
            color: role(&[Role::Special], "base0C"),
        },
        GituiMapping {
            field: "branch_fg",
            color: role(&[Role::Type], "base0A"),
        },
        GituiMapping {
            field: "block_title_focused",
            color: role(&[Role::BrightForeground], "base07"),
        },
    ]
}

const fn role(roles: &'static [Role], fallback_base: &'static str) -> GituiColor {
    GituiColor::Role {
        roles,
        fallback_base,
    }
}

fn color_for_mapping<'a>(
    mapping: &GituiMapping,
    roles: &'a SemanticRoles,
    palette: &'a Base16Palette,
) -> &'a str {
    match mapping.color {
        GituiColor::Role {
            roles: color_roles,
            fallback_base,
        } => {
            for role in color_roles {
                if let Some(color) = role_color(roles, *role) {
                    return color;
                }
            }
            color(palette, fallback_base)
        }
        GituiColor::FileKind(kind) => resolve_file_kind_color(kind, roles, palette),
    }
}

fn gitui_preserved_items(roles: &SemanticRoles) -> Vec<PreservedItem> {
    let mut items = Vec::new();
    for mapping in gitui_mappings() {
        match mapping.color {
            GituiColor::Role {
                roles: color_roles, ..
            } => {
                for role in color_roles {
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
            GituiColor::FileKind(kind) => {
                push_file_kind_source(&mut items, roles, mapping.field, kind);
            }
        }
    }
    items
}

fn ron_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
