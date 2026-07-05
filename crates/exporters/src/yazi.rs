use crate::file_kinds::{
    file_extension_groups, file_kind_style, push_file_kind_source, resolve_file_kind_color,
    FileEmphasis, FileKind,
};
use crate::report::{dropped_items, role_source, ExportReport, PreservedItem};
use palette16::{color, Base16Palette};
use semantic_roles::{role_color, Role, SemanticRoles};
use theme_ir::{ResolvedTheme, Warning};

pub fn export_yazi(
    theme: &ResolvedTheme,
    roles: &SemanticRoles,
    palette: &Base16Palette,
    warnings: Vec<Warning>,
) -> (String, ExportReport) {
    let mut output = String::new();
    output.push_str(&format!("# Generated from Helix theme: {}\n\n", theme.name));

    push_section(
        &mut output,
        "app",
        &[field("overall", style(None, Some(BACKGROUND), &[]))],
        roles,
        palette,
    );
    push_section(&mut output, "mgr", &manager_fields(), roles, palette);
    push_section(
        &mut output,
        "indicator",
        &indicator_fields(),
        roles,
        palette,
    );
    push_section(&mut output, "tabs", &tabs_fields(), roles, palette);
    push_section(&mut output, "mode", &mode_fields(), roles, palette);
    push_section(&mut output, "status", &status_fields(), roles, palette);
    push_section(&mut output, "which", &which_fields(), roles, palette);
    push_section(&mut output, "confirm", &confirm_fields(), roles, palette);
    push_section(&mut output, "spot", &spot_fields(), roles, palette);
    push_section(&mut output, "notify", &notify_fields(), roles, palette);
    push_section(&mut output, "pick", &pick_fields(), roles, palette);
    push_section(&mut output, "input", &input_fields(), roles, palette);
    push_section(&mut output, "cmp", &completion_fields(), roles, palette);
    push_section(&mut output, "tasks", &task_fields(), roles, palette);
    push_section(&mut output, "help", &help_fields(), roles, palette);
    push_filetype_rules(&mut output, roles, palette);

    let report = ExportReport {
        exporter: "yazi".to_owned(),
        source: theme.source_path.to_string(),
        preserved: yazi_preserved_items(roles),
        dropped: yazi_dropped_items(theme),
        warnings,
    };
    (output, report)
}

#[derive(Debug, Clone, Copy)]
struct YaziField {
    name: &'static str,
    style: YaziStyle,
}

#[derive(Debug, Clone, Copy)]
struct YaziStyle {
    fg: Option<YaziColor>,
    bg: Option<YaziColor>,
    attrs: &'static [YaziAttr],
}

#[derive(Debug, Clone, Copy)]
enum YaziAttr {
    Bold,
    Dim,
    Italic,
    Reversed,
}

#[derive(Debug, Clone, Copy)]
enum YaziColor {
    Role {
        roles: &'static [Role],
        fallback_base: &'static str,
    },
    FileKind(FileKind),
}

const BACKGROUND: YaziColor = role(&[Role::Background], "base00");
const SURFACE: YaziColor = role(&[Role::Surface], "base01");
const SELECTION: YaziColor = role(&[Role::Selection], "base02");
const FOREGROUND: YaziColor = role(&[Role::Foreground], "base05");
const MUTED: YaziColor = role(&[Role::MutedForeground], "base03");
const BRIGHT: YaziColor = role(&[Role::BrightForeground], "base07");
const SPECIAL: YaziColor = role(&[Role::Special], "base0C");
const KEYWORD: YaziColor = role(&[Role::Keyword], "base0E");
const STRING: YaziColor = role(&[Role::String], "base0B");
const INFO: YaziColor = role(&[Role::Info, Role::Special], "base0C");
const WARNING: YaziColor = role(&[Role::Warning, Role::Type], "base0A");
const ERROR: YaziColor = role(&[Role::Error], "base08");
const GIT_ADDED: YaziColor = role(&[Role::GitAdded, Role::String], "base0B");
const GIT_REMOVED: YaziColor = role(&[Role::GitRemoved, Role::Error], "base08");
const SYMLINK: YaziColor = file_kind(FileKind::Symlink);

const BOLD: &[YaziAttr] = &[YaziAttr::Bold];
const DIM: &[YaziAttr] = &[YaziAttr::Dim];
const ITALIC: &[YaziAttr] = &[YaziAttr::Italic];
const REVERSED: &[YaziAttr] = &[YaziAttr::Reversed];

const fn role(roles: &'static [Role], fallback_base: &'static str) -> YaziColor {
    YaziColor::Role {
        roles,
        fallback_base,
    }
}

const fn file_kind(kind: FileKind) -> YaziColor {
    YaziColor::FileKind(kind)
}

const fn style(
    fg: Option<YaziColor>,
    bg: Option<YaziColor>,
    attrs: &'static [YaziAttr],
) -> YaziStyle {
    YaziStyle { fg, bg, attrs }
}

const fn field(name: &'static str, style: YaziStyle) -> YaziField {
    YaziField { name, style }
}

fn manager_fields() -> Vec<YaziField> {
    vec![
        field("cwd", style(Some(SPECIAL), None, BOLD)),
        field("find_keyword", style(Some(WARNING), None, BOLD)),
        field("find_position", style(Some(INFO), None, &[])),
        field("symlink_target", style(Some(SYMLINK), None, &[])),
        field("marker_copied", style(Some(GIT_ADDED), None, &[])),
        field("marker_cut", style(Some(GIT_REMOVED), None, &[])),
        field("marker_marked", style(Some(KEYWORD), None, BOLD)),
        field("marker_selected", style(Some(SPECIAL), None, BOLD)),
        field("count_copied", style(Some(GIT_ADDED), None, BOLD)),
        field("count_cut", style(Some(GIT_REMOVED), None, BOLD)),
        field("count_selected", style(Some(SPECIAL), None, BOLD)),
        field("border_style", style(Some(MUTED), None, &[])),
    ]
}

fn indicator_fields() -> Vec<YaziField> {
    vec![
        field("parent", style(Some(MUTED), Some(SURFACE), &[])),
        field("current", style(Some(BRIGHT), Some(SELECTION), BOLD)),
        field("preview", style(Some(MUTED), Some(SURFACE), &[])),
    ]
}

fn tabs_fields() -> Vec<YaziField> {
    vec![
        field("active", style(Some(BRIGHT), Some(SELECTION), BOLD)),
        field("inactive", style(Some(MUTED), Some(SURFACE), &[])),
    ]
}

fn mode_fields() -> Vec<YaziField> {
    vec![
        field("normal_main", style(Some(BACKGROUND), Some(SPECIAL), BOLD)),
        field("normal_alt", style(Some(SPECIAL), Some(SURFACE), &[])),
        field("select_main", style(Some(BACKGROUND), Some(KEYWORD), BOLD)),
        field("select_alt", style(Some(KEYWORD), Some(SURFACE), &[])),
        field("unset_main", style(Some(BACKGROUND), Some(MUTED), BOLD)),
        field("unset_alt", style(Some(MUTED), Some(SURFACE), &[])),
    ]
}

fn status_fields() -> Vec<YaziField> {
    vec![
        field("overall", style(Some(FOREGROUND), Some(SURFACE), &[])),
        field("perm_type", style(Some(SPECIAL), None, BOLD)),
        field("perm_read", style(Some(STRING), None, &[])),
        field("perm_write", style(Some(WARNING), None, &[])),
        field("perm_exec", style(Some(KEYWORD), None, BOLD)),
        field("perm_sep", style(Some(MUTED), None, &[])),
        field("progress_label", style(Some(FOREGROUND), None, &[])),
        field("progress_normal", style(Some(SPECIAL), Some(SURFACE), &[])),
        field("progress_error", style(Some(ERROR), Some(SURFACE), BOLD)),
    ]
}

fn which_fields() -> Vec<YaziField> {
    vec![
        field("mask", style(Some(MUTED), Some(BACKGROUND), &[])),
        field("cand", style(Some(SPECIAL), None, BOLD)),
        field("rest", style(Some(FOREGROUND), None, &[])),
        field("desc", style(Some(MUTED), None, &[])),
        field("separator_style", style(Some(MUTED), None, &[])),
    ]
}

fn confirm_fields() -> Vec<YaziField> {
    vec![
        field("border", style(Some(SPECIAL), None, &[])),
        field("title", style(Some(BRIGHT), None, BOLD)),
        field("body", style(Some(FOREGROUND), None, &[])),
        field("list", style(Some(FOREGROUND), None, &[])),
        field("btn_yes", style(Some(BACKGROUND), Some(GIT_ADDED), BOLD)),
        field("btn_no", style(Some(BACKGROUND), Some(ERROR), BOLD)),
    ]
}

fn spot_fields() -> Vec<YaziField> {
    vec![
        field("border", style(Some(SPECIAL), None, &[])),
        field("title", style(Some(BRIGHT), None, BOLD)),
        field("tbl_col", style(Some(BRIGHT), Some(SELECTION), BOLD)),
        field("tbl_cell", style(Some(FOREGROUND), Some(SURFACE), &[])),
    ]
}

fn notify_fields() -> Vec<YaziField> {
    vec![
        field("title_info", style(Some(INFO), None, BOLD)),
        field("title_warn", style(Some(WARNING), None, BOLD)),
        field("title_error", style(Some(ERROR), None, BOLD)),
    ]
}

fn pick_fields() -> Vec<YaziField> {
    vec![
        field("border", style(Some(SPECIAL), None, &[])),
        field("active", style(Some(BRIGHT), Some(SELECTION), BOLD)),
        field("inactive", style(Some(FOREGROUND), None, &[])),
    ]
}

fn input_fields() -> Vec<YaziField> {
    vec![
        field("border", style(Some(SPECIAL), None, &[])),
        field("title", style(Some(BRIGHT), None, BOLD)),
        field("value", style(Some(FOREGROUND), None, &[])),
        field("selected", style(Some(BRIGHT), Some(SELECTION), &[])),
    ]
}

fn completion_fields() -> Vec<YaziField> {
    vec![
        field("border", style(Some(SPECIAL), None, &[])),
        field("active", style(Some(BRIGHT), Some(SELECTION), BOLD)),
        field("inactive", style(Some(FOREGROUND), None, &[])),
    ]
}

fn task_fields() -> Vec<YaziField> {
    vec![
        field("border", style(Some(SPECIAL), None, &[])),
        field("title", style(Some(BRIGHT), None, BOLD)),
        field("hovered", style(Some(BRIGHT), Some(SELECTION), &[])),
    ]
}

fn help_fields() -> Vec<YaziField> {
    vec![
        field("on", style(Some(SPECIAL), None, BOLD)),
        field("run", style(Some(STRING), None, &[])),
        field("desc", style(Some(FOREGROUND), None, &[])),
        field("hovered", style(Some(BRIGHT), Some(SELECTION), &[])),
        field("footer", style(Some(MUTED), None, ITALIC)),
    ]
}

fn push_section(
    output: &mut String,
    name: &str,
    fields: &[YaziField],
    roles: &SemanticRoles,
    palette: &Base16Palette,
) {
    output.push_str(&format!("[{name}]\n"));
    for field in fields {
        output.push_str(&format!(
            "{} = {}\n",
            field.name,
            render_style(field.style, roles, palette)
        ));
    }
    output.push('\n');
}

fn push_filetype_rules(output: &mut String, roles: &SemanticRoles, palette: &Base16Palette) {
    output.push_str("[filetype]\n");
    output.push_str("rules = [\n");
    for rule in filetype_rules() {
        output.push_str(&format!(
            "  {{ {}, {} }},\n",
            rule.matcher,
            render_style_body(style_from_file_kind(rule.kind), roles, palette)
        ));
    }
    for group in file_extension_groups() {
        for ext in group.extensions {
            output.push_str(&format!(
                "  {{ url = \"*.{}\", {} }},\n",
                toml_escape(ext),
                render_style_body(style_from_file_kind(group.kind), roles, palette)
            ));
        }
    }
    output.push_str(&format!(
        "  {{ url = \"*/\", {} }},\n",
        render_style_body(style_from_file_kind(FileKind::Directory), roles, palette)
    ));
    output.push_str(&format!(
        "  {{ url = \"*\", {} }},\n",
        render_style_body(style(Some(FOREGROUND), None, &[]), roles, palette)
    ));
    output.push_str("]\n");
}

#[derive(Debug, Clone, Copy)]
struct FiletypeRule {
    matcher: &'static str,
    kind: FileKind,
}

fn filetype_rules() -> &'static [FiletypeRule] {
    &[
        FiletypeRule {
            matcher: "url = \"*\", is = \"orphan\"",
            kind: FileKind::BrokenLink,
        },
        FiletypeRule {
            matcher: "url = \"*\", is = \"link\"",
            kind: FileKind::Symlink,
        },
        FiletypeRule {
            matcher: "url = \"*\", is = \"exec\"",
            kind: FileKind::Executable,
        },
        FiletypeRule {
            matcher: "url = \"*\", is = \"fifo\"",
            kind: FileKind::Fifo,
        },
        FiletypeRule {
            matcher: "url = \"*\", is = \"sock\"",
            kind: FileKind::Socket,
        },
        FiletypeRule {
            matcher: "url = \"*\", is = \"block\"",
            kind: FileKind::Device,
        },
        FiletypeRule {
            matcher: "url = \"*\", is = \"char\"",
            kind: FileKind::Device,
        },
        FiletypeRule {
            matcher: "url = \"*\", is = \"sticky\"",
            kind: FileKind::StickyDir,
        },
    ]
}

fn style_from_file_kind(kind: FileKind) -> YaziStyle {
    let file_style = file_kind_style(kind);
    let attrs = match file_style.emphasis {
        FileEmphasis::Bold | FileEmphasis::Dangerous => BOLD,
        FileEmphasis::Muted => DIM,
        FileEmphasis::Background => REVERSED,
        FileEmphasis::Normal => &[],
    };
    style(Some(YaziColor::FileKind(kind)), None, attrs)
}

fn render_style(style: YaziStyle, roles: &SemanticRoles, palette: &Base16Palette) -> String {
    format!("{{ {} }}", render_style_body(style, roles, palette))
}

fn render_style_body(style: YaziStyle, roles: &SemanticRoles, palette: &Base16Palette) -> String {
    let mut parts = Vec::new();
    if let Some(fg) = style.fg {
        parts.push(format!("fg = \"{}\"", resolve_color(fg, roles, palette)));
    }
    if let Some(bg) = style.bg {
        parts.push(format!("bg = \"{}\"", resolve_color(bg, roles, palette)));
    }
    for attr in style.attrs {
        parts.push(format!("{} = true", attr_name(*attr)));
    }
    parts.join(", ")
}

fn attr_name(attr: YaziAttr) -> &'static str {
    match attr {
        YaziAttr::Bold => "bold",
        YaziAttr::Dim => "dim",
        YaziAttr::Italic => "italic",
        YaziAttr::Reversed => "reversed",
    }
}

fn resolve_color<'a>(
    yazi_color: YaziColor,
    roles: &'a SemanticRoles,
    palette: &'a Base16Palette,
) -> &'a str {
    match yazi_color {
        YaziColor::Role {
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
        YaziColor::FileKind(kind) => resolve_file_kind_color(kind, roles, palette),
    }
}

fn yazi_preserved_items(roles: &SemanticRoles) -> Vec<PreservedItem> {
    let mut items = Vec::new();
    for (section, fields) in [
        (
            "app",
            vec![field("overall", style(None, Some(BACKGROUND), &[]))],
        ),
        ("mgr", manager_fields()),
        ("indicator", indicator_fields()),
        ("tabs", tabs_fields()),
        ("mode", mode_fields()),
        ("status", status_fields()),
        ("which", which_fields()),
        ("confirm", confirm_fields()),
        ("spot", spot_fields()),
        ("notify", notify_fields()),
        ("pick", pick_fields()),
        ("input", input_fields()),
        ("cmp", completion_fields()),
        ("tasks", task_fields()),
        ("help", help_fields()),
    ] {
        for field in fields {
            push_style_sources(
                &mut items,
                roles,
                &format!("{section}.{}", field.name),
                field.style,
            );
        }
    }
    for rule in filetype_rules() {
        push_file_kind_source(&mut items, roles, rule.matcher, rule.kind);
    }
    for group in file_extension_groups() {
        push_file_kind_source(&mut items, roles, group.name, group.kind);
    }
    items
}

fn push_style_sources(
    items: &mut Vec<PreservedItem>,
    roles: &SemanticRoles,
    target: &str,
    style: YaziStyle,
) {
    push_color_source(items, roles, target, style.fg);
    push_color_source(items, roles, target, style.bg);
}

fn push_color_source(
    items: &mut Vec<PreservedItem>,
    roles: &SemanticRoles,
    target: &str,
    color: Option<YaziColor>,
) {
    let Some(color) = color else {
        return;
    };
    match color {
        YaziColor::Role {
            roles: color_roles, ..
        } => {
            for role in color_roles {
                if let Some(value) = roles.get(role) {
                    if let Some(source) = role_source(value) {
                        items.push(PreservedItem {
                            target: target.to_owned(),
                            source,
                        });
                        break;
                    }
                }
            }
        }
        YaziColor::FileKind(kind) => push_file_kind_source(items, roles, target, kind),
    }
}

fn yazi_dropped_items(theme: &ResolvedTheme) -> Vec<String> {
    let mut dropped = dropped_items(theme);
    dropped.push("Yazi flavor metadata".to_owned());
    dropped.push("Yazi syntax preview tmTheme".to_owned());
    dropped
}

fn toml_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
