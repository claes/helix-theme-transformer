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
struct LsColor {
    roles: &'static [Role],
    fallback_base: &'static str,
}

impl LsColor {
    const fn new(roles: &'static [Role], fallback_base: &'static str) -> Self {
        Self {
            roles,
            fallback_base,
        }
    }
}

const BACKGROUND: LsColor = LsColor::new(&[Role::Background], "base00");
const FOREGROUND: LsColor = LsColor::new(&[Role::Foreground], "base05");
const MUTED_FOREGROUND: LsColor = LsColor::new(&[Role::MutedForeground], "base03");
const KEYWORD: LsColor = LsColor::new(&[Role::Keyword], "base0E");
const FUNCTION: LsColor = LsColor::new(&[Role::Function, Role::GitAdded], "base0D");
const STRING: LsColor = LsColor::new(&[Role::String], "base0B");
const SPECIAL: LsColor = LsColor::new(&[Role::Special], "base0C");
const ERROR: LsColor = LsColor::new(&[Role::Error], "base08");
const WARNING: LsColor = LsColor::new(&[Role::Warning, Role::Type], "base0A");
const GIT_ADDED: LsColor = LsColor::new(&[Role::GitAdded], "base0B");

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
        entry("DIR", style(BOLD, Some(FUNCTION), None)),
        entry("LINK", style(BOLD, Some(SPECIAL), None)),
        entry("MULTIHARDLINK", style(&[], Some(FOREGROUND), None)),
        entry("FIFO", style(&[], Some(WARNING), None)),
        entry("SOCK", style(BOLD, Some(KEYWORD), None)),
        entry("DOOR", style(BOLD, Some(KEYWORD), None)),
        entry("BLK", style(BOLD, Some(WARNING), None)),
        entry("CHR", style(BOLD, Some(WARNING), None)),
        entry("ORPHAN", style(BOLD, Some(ERROR), None)),
        entry("MISSING", style(&[], Some(MUTED_FOREGROUND), None)),
        entry("SETUID", style(BOLD, Some(FOREGROUND), Some(ERROR))),
        entry("SETGID", style(BOLD, Some(BACKGROUND), Some(WARNING))),
        entry("CAPABILITY", style(&[], Some(FOREGROUND), None)),
        entry(
            "STICKY_OTHER_WRITABLE",
            style(&[], Some(BACKGROUND), Some(GIT_ADDED)),
        ),
        entry("OTHER_WRITABLE", style(&[], Some(SPECIAL), Some(GIT_ADDED))),
        entry("STICKY", style(&[], Some(FOREGROUND), Some(SPECIAL))),
        entry("EXEC", style(BOLD, Some(FUNCTION), None)),
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
            style: style(BOLD, Some(ERROR), None),
        },
        ExtensionGroup {
            name: "images and video",
            extensions: &[
                "avif", "bmp", "gif", "jpeg", "jpg", "jxl", "mkv", "mov", "mp4", "mpeg", "mpg",
                "png", "svg", "svgz", "tif", "tiff", "webm", "webp",
            ],
            style: style(BOLD, Some(SPECIAL), None),
        },
        ExtensionGroup {
            name: "audio",
            extensions: &[
                "aac", "flac", "m4a", "mid", "midi", "mp3", "ogg", "opus", "wav",
            ],
            style: style(&[], Some(SPECIAL), None),
        },
        ExtensionGroup {
            name: "documents",
            extensions: &[
                "djvu", "doc", "docx", "epub", "md", "odf", "odt", "pdf", "rtf", "tex", "txt",
            ],
            style: style(&[], Some(STRING), None),
        },
        ExtensionGroup {
            name: "source code",
            extensions: &[
                "c", "cc", "clj", "cpp", "cs", "css", "go", "h", "hpp", "html", "java", "js",
                "jsx", "lua", "nix", "php", "py", "rb", "rs", "scss", "sh", "ts", "tsx", "vim",
                "zig",
            ],
            style: style(&[], Some(KEYWORD), None),
        },
        ExtensionGroup {
            name: "temporary and logs",
            extensions: &["bak", "cache", "log", "old", "orig", "tmp"],
            style: style(&[], Some(MUTED_FOREGROUND), None),
        },
    ]
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
    for role in ls_color.roles {
        if let Some(color) = role_color(roles, *role) {
            return color;
        }
    }
    color(palette, ls_color.fallback_base)
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
    for role in color.roles {
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
