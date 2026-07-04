use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use theme_ir::{Confidence, ResolvedTheme, SemanticRoleValue, SourceProperty, Style, Warning};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Background,
    Surface,
    Selection,
    Foreground,
    MutedForeground,
    BrightForeground,
    Cursor,
    Comment,
    Keyword,
    Function,
    Type,
    Variable,
    Parameter,
    String,
    Number,
    Constant,
    Operator,
    Special,
    Error,
    Warning,
    Info,
    Hint,
    GitAdded,
    GitModified,
    GitRemoved,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Role::Background => "background",
            Role::Surface => "surface",
            Role::Selection => "selection",
            Role::Foreground => "foreground",
            Role::MutedForeground => "muted_foreground",
            Role::BrightForeground => "bright_foreground",
            Role::Cursor => "cursor",
            Role::Comment => "comment",
            Role::Keyword => "keyword",
            Role::Function => "function",
            Role::Type => "type",
            Role::Variable => "variable",
            Role::Parameter => "parameter",
            Role::String => "string",
            Role::Number => "number",
            Role::Constant => "constant",
            Role::Operator => "operator",
            Role::Special => "special",
            Role::Error => "error",
            Role::Warning => "warning",
            Role::Info => "info",
            Role::Hint => "hint",
            Role::GitAdded => "git_added",
            Role::GitModified => "git_modified",
            Role::GitRemoved => "git_removed",
        }
    }
}

pub type SemanticRoles = IndexMap<Role, SemanticRoleValue>;

pub fn derive_roles(theme: &ResolvedTheme) -> (SemanticRoles, Vec<Warning>) {
    let mut roles = IndexMap::new();
    let mut warnings = Vec::new();
    for role in all_roles() {
        let value = derive_role(theme, role);
        if value.confidence == Confidence::Missing {
            warnings.push(Warning::new(
                "missing_role",
                format!("Missing semantic role: {}", role.as_str()),
            ));
        }
        roles.insert(role, value);
    }
    (roles, warnings)
}

fn derive_role(theme: &ResolvedTheme, role: Role) -> SemanticRoleValue {
    for &(scope, property) in mappings(role) {
        if let Some(style) = theme.scopes.get(scope) {
            if let Some(color) = read_property(style, property) {
                return SemanticRoleValue {
                    color: Some(color.to_owned()),
                    source_scope: Some(scope.to_owned()),
                    source_property: Some(property),
                    confidence: Confidence::Exact,
                };
            }
        }
    }
    SemanticRoleValue {
        color: None,
        source_scope: None,
        source_property: None,
        confidence: Confidence::Missing,
    }
}

fn read_property(style: &Style, property: SourceProperty) -> Option<&str> {
    match property {
        SourceProperty::Fg => style.fg.as_deref(),
        SourceProperty::Bg => style.bg.as_deref(),
        SourceProperty::UnderlineColor => style.underline.as_ref()?.color.as_deref(),
    }
}

pub fn all_roles() -> [Role; 25] {
    [
        Role::Background,
        Role::Surface,
        Role::Selection,
        Role::Foreground,
        Role::MutedForeground,
        Role::BrightForeground,
        Role::Cursor,
        Role::Comment,
        Role::Keyword,
        Role::Function,
        Role::Type,
        Role::Variable,
        Role::Parameter,
        Role::String,
        Role::Number,
        Role::Constant,
        Role::Operator,
        Role::Special,
        Role::Error,
        Role::Warning,
        Role::Info,
        Role::Hint,
        Role::GitAdded,
        Role::GitModified,
        Role::GitRemoved,
    ]
}

fn mappings(role: Role) -> &'static [(&'static str, SourceProperty)] {
    use SourceProperty::{Bg, Fg, UnderlineColor};
    match role {
        Role::Background => &[("ui.background", Bg), ("ui.background", Fg)],
        Role::Surface => &[
            ("ui.statusline", Bg),
            ("ui.popup", Bg),
            ("ui.menu", Bg),
            ("ui.window", Bg),
        ],
        Role::Selection => &[("ui.selection", Bg), ("ui.cursor.select", Bg)],
        Role::Foreground => &[("ui.text", Fg), ("ui.text.focus", Fg)],
        Role::MutedForeground => &[("comment", Fg), ("ui.linenr", Fg), ("ui.text.inactive", Fg)],
        Role::BrightForeground => &[
            ("ui.cursor", Fg),
            ("ui.text.focus", Fg),
            ("markup.heading", Fg),
        ],
        Role::Cursor => &[
            ("ui.cursor.primary", Bg),
            ("ui.cursor", Bg),
            ("ui.cursor.primary", Fg),
            ("ui.cursor", Fg),
        ],
        Role::Comment => &[("comment", Fg)],
        Role::Keyword => &[
            ("keyword", Fg),
            ("keyword.control", Fg),
            ("keyword.directive", Fg),
            ("keyword.operator", Fg),
        ],
        Role::Function => &[
            ("function", Fg),
            ("function.method", Fg),
            ("function.builtin", Fg),
            ("constructor", Fg),
        ],
        Role::Type => &[("type", Fg), ("type.builtin", Fg), ("constructor", Fg)],
        Role::Variable => &[("variable", Fg), ("variable.other", Fg)],
        Role::Parameter => &[("variable.parameter", Fg), ("parameter", Fg)],
        Role::String => &[("string", Fg), ("string.special", Fg)],
        Role::Number => &[("constant.numeric", Fg), ("number", Fg)],
        Role::Constant => &[
            ("constant", Fg),
            ("constant.builtin", Fg),
            ("constant.character", Fg),
        ],
        Role::Operator => &[("operator", Fg), ("keyword.operator", Fg)],
        Role::Special => &[
            ("special", Fg),
            ("tag", Fg),
            ("attribute", Fg),
            ("namespace", Fg),
        ],
        Role::Error => &[
            ("diagnostic.error", Fg),
            ("diagnostic.error", UnderlineColor),
            ("error", Fg),
        ],
        Role::Warning => &[
            ("diagnostic.warning", Fg),
            ("diagnostic.warning", UnderlineColor),
            ("warning", Fg),
        ],
        Role::Info => &[
            ("diagnostic.info", Fg),
            ("diagnostic.info", UnderlineColor),
            ("info", Fg),
        ],
        Role::Hint => &[
            ("diagnostic.hint", Fg),
            ("diagnostic.hint", UnderlineColor),
            ("hint", Fg),
        ],
        Role::GitAdded => &[("diff.plus", Fg), ("ui.statusline.insert", Fg)],
        Role::GitModified => &[("diff.delta", Fg), ("ui.statusline.normal", Fg)],
        Role::GitRemoved => &[("diff.minus", Fg), ("ui.statusline.select", Fg)],
    }
}

pub fn role_color(roles: &SemanticRoles, role: Role) -> Option<&str> {
    roles.get(&role)?.color.as_deref()
}

#[cfg(test)]
mod tests {
    use super::*;
    use camino::Utf8PathBuf;
    use indexmap::indexmap;
    use theme_ir::{Style, Underline};

    #[test]
    fn derives_from_scope_not_palette_name() {
        let theme = ResolvedTheme {
            name: "demo".to_owned(),
            source_path: Utf8PathBuf::from("demo.toml"),
            palette: indexmap! {"banana".to_owned() => "#7aa2f7".to_owned()},
            scopes: indexmap! {"function".to_owned() => Style { fg: Some("#7aa2f7".to_owned()), ..Style::default() }},
            warnings: Vec::new(),
        };
        let (roles, _) = derive_roles(&theme);
        assert_eq!(
            roles.get(&Role::Function).and_then(|v| v.color.as_deref()),
            Some("#7aa2f7")
        );
        assert_eq!(
            roles.get(&Role::Keyword).unwrap().confidence,
            Confidence::Missing
        );
    }

    #[test]
    fn diagnostic_uses_underline_color_fallback() {
        let theme = ResolvedTheme {
            name: "demo".to_owned(),
            source_path: Utf8PathBuf::from("demo.toml"),
            palette: IndexMap::new(),
            scopes: indexmap! {
                "diagnostic.error".to_owned() => Style {
                    underline: Some(Underline { color: Some("#ff0000".to_owned()), style: None }),
                    ..Style::default()
                }
            },
            warnings: Vec::new(),
        };
        let (roles, _) = derive_roles(&theme);
        let error = roles.get(&Role::Error).unwrap();
        assert_eq!(error.color.as_deref(), Some("#ff0000"));
        assert_eq!(error.source_property, Some(SourceProperty::UnderlineColor));
    }
}
