use crate::file_kinds::{
    file_kind_style, push_file_kind_source, resolve_file_kind_color, FileEmphasis, FileKind,
};
use crate::report::{role_source, ExportReport, PreservedItem};
use palette16::{color, Base16Palette};
use semantic_roles::{role_color, Role, SemanticRoles};
use theme_ir::{parse_rgb, ResolvedTheme, Warning};

pub fn export_dircolors(
    theme: &ResolvedTheme,
    roles: &SemanticRoles,
    palette: &Base16Palette,
    warnings: Vec<Warning>,
) -> (String, ExportReport) {
    let mut output = String::new();
    output.push_str(&format!("# Generated from Helix theme: {}\n", theme.name));
    output.push_str("COLORTERM ?*\n");
    output.push_str("TERM *color*\n");
    output.push_str("TERM *direct*\n");
    output.push_str("TERM xterm*\n");
    output.push_str("TERM screen*\n");
    output.push_str("TERM tmux*\n\n");

    for entry in core_entries() {
        output.push_str(&format!(
            "{} {}\n",
            entry.key,
            render_sgr(entry.style, roles, palette)
        ));
    }

    output.push('\n');
    for group in extension_groups() {
        output.push_str(&format!("# {}\n", group.name));
        let sgr = render_sgr(group.style, roles, palette);
        for ext in group.extensions {
            output.push_str(&format!("*.{ext} {sgr}\n"));
        }
        output.push('\n');
    }

    let report = ExportReport {
        exporter: "dircolors".to_owned(),
        source: theme.source_path.to_string(),
        preserved: dircolors_preserved_items(roles),
        dropped: vec![
            "Helix syntax scopes collapse to LS_COLORS file classes and extensions".to_owned(),
            "LS_COLORS supports SGR attributes only".to_owned(),
            "Truecolor output requires compatible dircolors, ls, and terminal behavior".to_owned(),
        ],
        warnings,
    };
    (output, report)
}

#[derive(Debug, Clone, Copy)]
struct LsEntry {
    key: &'static str,
    style: LsStyle,
}

#[derive(Debug, Clone, Copy)]
struct ExtensionGroup {
    name: &'static str,
    extensions: &'static [&'static str],
    style: LsStyle,
}

#[derive(Debug, Clone, Copy)]
struct LsStyle {
    attrs: &'static [&'static str],
    fg: Option<LsColor>,
    bg: Option<LsColor>,
}

#[derive(Debug, Clone, Copy)]
enum LsColor {
    Role {
        roles: &'static [Role],
        fallback_base: &'static str,
    },
    FileKind(FileKind),
}

const BACKGROUND: LsColor = LsColor::Role {
    roles: &[Role::Background],
    fallback_base: "base00",
};
const FOREGROUND: LsColor = LsColor::Role {
    roles: &[Role::Foreground],
    fallback_base: "base05",
};
const FILE_SETUID: LsColor = LsColor::FileKind(FileKind::Setuid);
const FILE_SETGID: LsColor = LsColor::FileKind(FileKind::Setgid);
const FILE_WRITABLE_DIR: LsColor = LsColor::FileKind(FileKind::WritableDir);
const FILE_STICKY_DIR: LsColor = LsColor::FileKind(FileKind::StickyDir);

const BOLD: &[&str] = &["01"];

const fn style(attrs: &'static [&str], fg: Option<LsColor>, bg: Option<LsColor>) -> LsStyle {
    LsStyle { attrs, fg, bg }
}

const fn entry(key: &'static str, style: LsStyle) -> LsEntry {
    LsEntry { key, style }
}

fn core_entries() -> Vec<LsEntry> {
    vec![
        entry("RESET", style(&[], None, None)),
        entry("DIR", style_from_kind(FileKind::Directory)),
        entry("LINK", style_from_kind(FileKind::Symlink)),
        entry("MULTIHARDLINK", style(&[], Some(FOREGROUND), None)),
        entry("FIFO", style_from_kind(FileKind::Fifo)),
        entry("SOCK", style_from_kind(FileKind::Socket)),
        entry("DOOR", style_from_kind(FileKind::Socket)),
        entry("BLK", style_from_kind(FileKind::Device)),
        entry("CHR", style_from_kind(FileKind::Device)),
        entry("ORPHAN", style_from_kind(FileKind::BrokenLink)),
        entry("MISSING", style_from_kind(FileKind::Missing)),
        entry("SETUID", style(BOLD, Some(FOREGROUND), Some(FILE_SETUID))),
        entry("SETGID", style(BOLD, Some(BACKGROUND), Some(FILE_SETGID))),
        entry("CAPABILITY", style(&[], Some(FOREGROUND), None)),
        entry(
            "STICKY_OTHER_WRITABLE",
            style(&[], Some(BACKGROUND), Some(FILE_WRITABLE_DIR)),
        ),
        entry(
            "OTHER_WRITABLE",
            style(&[], Some(FILE_STICKY_DIR), Some(FILE_WRITABLE_DIR)),
        ),
        entry(
            "STICKY",
            style(&[], Some(FOREGROUND), Some(FILE_STICKY_DIR)),
        ),
        entry("EXEC", style_from_kind(FileKind::Executable)),
    ]
}

fn extension_groups() -> Vec<ExtensionGroup> {
    vec![
        ExtensionGroup {
            name: "archives and compressed files",
            extensions: &[
                "7z", "ace", "alz", "apk", "arc", "arj", "bz", "bz2", "cab", "cpio", "crate",
                "deb", "gz", "jar", "lha", "lrz", "lz", "lz4", "lzma", "lzo", "rar", "rpm", "tar",
                "tbz", "tbz2", "tgz", "tlz", "txz", "xz", "zip", "zst",
            ],
            style: style_from_kind(FileKind::Archive),
        },
        ExtensionGroup {
            name: "images and video",
            extensions: &[
                "avif", "bmp", "gif", "jpeg", "jpg", "jxl", "mkv", "mov", "mp4", "mpeg", "mpg",
                "png", "svg", "svgz", "tif", "tiff", "webm", "webp",
            ],
            style: style_from_kind(FileKind::ImageVideo),
        },
        ExtensionGroup {
            name: "audio",
            extensions: &[
                "aac", "flac", "m4a", "mid", "midi", "mp3", "ogg", "opus", "wav",
            ],
            style: style_from_kind(FileKind::Audio),
        },
        ExtensionGroup {
            name: "documents",
            extensions: &[
                "djvu", "doc", "docx", "epub", "md", "odf", "odt", "pdf", "rtf", "tex", "txt",
            ],
            style: style_from_kind(FileKind::Document),
        },
        ExtensionGroup {
            name: "source code",
            extensions: &[
                "c", "cc", "clj", "cpp", "cs", "css", "go", "h", "hpp", "html", "java", "js",
                "jsx", "lua", "nix", "php", "py", "rb", "rs", "scss", "sh", "ts", "tsx", "vim",
                "zig",
            ],
            style: style_from_kind(FileKind::Source),
        },
        ExtensionGroup {
            name: "temporary and logs",
            extensions: &["bak", "cache", "log", "old", "orig", "tmp"],
            style: style_from_kind(FileKind::Temporary),
        },
    ]
}

fn style_from_kind(kind: FileKind) -> LsStyle {
    let style = file_kind_style(kind);
    let attrs = match style.emphasis {
        FileEmphasis::Bold | FileEmphasis::Dangerous => BOLD,
        FileEmphasis::Normal | FileEmphasis::Muted | FileEmphasis::Background => &[],
    };
    LsStyle {
        attrs,
        fg: Some(LsColor::FileKind(kind)),
        bg: None,
    }
}

fn render_sgr(style: LsStyle, roles: &SemanticRoles, palette: &Base16Palette) -> String {
    let mut parts = Vec::new();
    parts.extend(style.attrs.iter().map(|attr| (*attr).to_owned()));
    if let Some(fg) = style.fg {
        parts.push(foreground_sgr(resolve_color(fg, roles, palette)));
    }
    if let Some(bg) = style.bg {
        parts.push(background_sgr(resolve_color(bg, roles, palette)));
    }
    if parts.is_empty() {
        "0".to_owned()
    } else {
        parts.join(";")
    }
}

fn foreground_sgr(color: &str) -> String {
    truecolor_sgr("38", color)
}

fn background_sgr(color: &str) -> String {
    truecolor_sgr("48", color)
}

fn truecolor_sgr(prefix: &str, color: &str) -> String {
    let rgb = parse_rgb(color).expect("resolved exporter colors should be valid rgb hex colors");
    format!("{prefix};2;{};{};{}", rgb.r, rgb.g, rgb.b)
}

fn resolve_color<'a>(
    ls_color: LsColor,
    roles: &'a SemanticRoles,
    palette: &'a Base16Palette,
) -> &'a str {
    match ls_color {
        LsColor::Role {
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
        LsColor::FileKind(kind) => resolve_file_kind_color(kind, roles, palette),
    }
}

fn dircolors_preserved_items(roles: &SemanticRoles) -> Vec<PreservedItem> {
    let mut items = Vec::new();
    for entry in core_entries() {
        push_source(&mut items, roles, entry.key, entry.style.fg);
        push_source(&mut items, roles, entry.key, entry.style.bg);
    }
    for group in extension_groups() {
        push_source(&mut items, roles, group.name, group.style.fg);
        push_source(&mut items, roles, group.name, group.style.bg);
    }
    items
}

fn push_source(
    items: &mut Vec<PreservedItem>,
    roles: &SemanticRoles,
    target: &str,
    color: Option<LsColor>,
) {
    let Some(color) = color else {
        return;
    };
    match color {
        LsColor::Role {
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
        LsColor::FileKind(kind) => push_file_kind_source(items, roles, target, kind),
    }
}
