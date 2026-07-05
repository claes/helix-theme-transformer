use crate::report::{role_source, PreservedItem};
use palette16::{color, Base16Palette};
use semantic_roles::{role_color, Role, SemanticRoles};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileKind {
    Directory,
    Symlink,
    Executable,
    Fifo,
    Socket,
    Device,
    BrokenLink,
    Missing,
    Setuid,
    Setgid,
    WritableDir,
    StickyDir,
    Archive,
    ImageVideo,
    Audio,
    Document,
    Source,
    Database,
    Temporary,
    GitAdded,
    GitModified,
    GitRemoved,
    GitMoved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileEmphasis {
    Normal,
    Bold,
    Muted,
    Dangerous,
    Background,
}

#[derive(Debug, Clone, Copy)]
pub struct FileKindStyle {
    pub roles: &'static [Role],
    pub fallback_base: &'static str,
    pub emphasis: FileEmphasis,
}

pub fn file_kind_style(kind: FileKind) -> FileKindStyle {
    match kind {
        FileKind::Directory => style(&[Role::Function], "base0D", FileEmphasis::Bold),
        FileKind::Symlink => style(&[Role::Special], "base0C", FileEmphasis::Bold),
        FileKind::Executable => style(
            &[Role::String, Role::GitAdded],
            "base0B",
            FileEmphasis::Bold,
        ),
        FileKind::Fifo => style(&[Role::Warning], "base0A", FileEmphasis::Normal),
        FileKind::Socket => style(&[Role::Keyword], "base0E", FileEmphasis::Bold),
        FileKind::Device => style(&[Role::Warning], "base0A", FileEmphasis::Bold),
        FileKind::BrokenLink => style(&[Role::Error], "base08", FileEmphasis::Dangerous),
        FileKind::Missing => style(&[Role::MutedForeground], "base03", FileEmphasis::Muted),
        FileKind::Setuid => style(&[Role::Error], "base08", FileEmphasis::Background),
        FileKind::Setgid => style(&[Role::Warning], "base0A", FileEmphasis::Background),
        FileKind::WritableDir => style(&[Role::GitAdded], "base0B", FileEmphasis::Background),
        FileKind::StickyDir => style(&[Role::Special], "base0C", FileEmphasis::Background),
        FileKind::Archive => style(&[Role::Number], "base09", FileEmphasis::Normal),
        FileKind::ImageVideo => style(&[Role::Keyword], "base0E", FileEmphasis::Bold),
        FileKind::Audio => style(&[Role::Special], "base0C", FileEmphasis::Normal),
        FileKind::Document => style(&[Role::String], "base0B", FileEmphasis::Normal),
        FileKind::Source => style(&[Role::Keyword], "base0E", FileEmphasis::Normal),
        FileKind::Database => style(&[Role::Type], "base0A", FileEmphasis::Normal),
        FileKind::Temporary => style(&[Role::MutedForeground], "base03", FileEmphasis::Muted),
        FileKind::GitAdded => style(&[Role::GitAdded], "base0B", FileEmphasis::Normal),
        FileKind::GitModified => style(
            &[Role::GitModified, Role::Warning],
            "base0A",
            FileEmphasis::Normal,
        ),
        FileKind::GitRemoved => style(
            &[Role::GitRemoved, Role::Error],
            "base08",
            FileEmphasis::Dangerous,
        ),
        FileKind::GitMoved => style(&[Role::Special], "base0C", FileEmphasis::Normal),
    }
}

pub fn resolve_file_kind_color<'a>(
    kind: FileKind,
    roles: &'a SemanticRoles,
    palette: &'a Base16Palette,
) -> &'a str {
    let style = file_kind_style(kind);
    resolve_file_style_color(style, roles, palette)
}

pub fn resolve_file_style_color<'a>(
    style: FileKindStyle,
    roles: &'a SemanticRoles,
    palette: &'a Base16Palette,
) -> &'a str {
    for role in style.roles {
        if let Some(color) = role_color(roles, *role) {
            return color;
        }
    }
    color(palette, style.fallback_base)
}

pub fn push_file_kind_source(
    items: &mut Vec<PreservedItem>,
    roles: &SemanticRoles,
    target: &str,
    kind: FileKind,
) {
    let style = file_kind_style(kind);
    for role in style.roles {
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

const fn style(
    roles: &'static [Role],
    fallback_base: &'static str,
    emphasis: FileEmphasis,
) -> FileKindStyle {
    FileKindStyle {
        roles,
        fallback_base,
        emphasis,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_file_kind_fallbacks_match_specification() {
        assert_eq!(file_kind_style(FileKind::Directory).fallback_base, "base0D");
        assert_eq!(
            file_kind_style(FileKind::Executable).fallback_base,
            "base0B"
        );
        assert_eq!(file_kind_style(FileKind::Archive).fallback_base, "base09");
        assert_eq!(file_kind_style(FileKind::Source).fallback_base, "base0E");
        assert_eq!(
            file_kind_style(FileKind::GitRemoved).fallback_base,
            "base08"
        );
    }
}
