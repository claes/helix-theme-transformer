use serde::{Deserialize, Serialize};
use theme_ir::{Confidence, Modifier, ResolvedTheme, SemanticRoleValue, SourceProperty};

/// Component styles retain color pairs instead of mixing unrelated UI colors.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SemanticUiStyle {
    pub fg: Option<SemanticRoleValue>,
    pub bg: Option<SemanticRoleValue>,
    pub modifiers: Vec<Modifier>,
    pub underline: bool,
    pub emphasis_source: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SemanticUiStyles {
    pub menu: SemanticUiStyle,
    pub selected_menu: SemanticUiStyle,
    pub popup: SemanticUiStyle,
    pub focused_control: SemanticUiStyle,
    pub primary_selection: SemanticUiStyle,
    pub inactive_text: SemanticUiStyle,
}

pub fn derive_ui_styles(theme: &ResolvedTheme) -> SemanticUiStyles {
    let menu = read(theme, &["ui.menu"]);
    let mut selected_menu = read(theme, &["ui.menu.selected"]);
    // A partial selected style inherits its component's normal colors, not a
    // foreground or background from an unrelated Helix scope.
    if selected_menu != SemanticUiStyle::default() {
        inherit(&mut selected_menu.fg, &menu.fg);
        inherit(&mut selected_menu.bg, &menu.bg);
    }
    SemanticUiStyles {
        menu,
        selected_menu,
        popup: read(theme, &["ui.popup"]),
        focused_control: read(theme, &["ui.text.focus"]),
        primary_selection: read(theme, &["ui.selection.primary", "ui.selection"]),
        inactive_text: read(theme, &["ui.text.inactive"]),
    }
}

fn inherit(target: &mut Option<SemanticRoleValue>, source: &Option<SemanticRoleValue>) {
    if target.is_none() {
        *target = source.clone().map(|mut value| {
            value.confidence = Confidence::Fallback;
            value
        });
    }
}

fn read(theme: &ResolvedTheme, scopes: &[&str]) -> SemanticUiStyle {
    let mut result = SemanticUiStyle::default();
    for scope in scopes {
        let Some(style) = theme.scopes.get(*scope) else {
            continue;
        };
        for (target, color, property) in [
            (&mut result.fg, &style.fg, SourceProperty::Fg),
            (&mut result.bg, &style.bg, SourceProperty::Bg),
        ] {
            if target.is_none() {
                *target = color.as_ref().map(|color| SemanticRoleValue {
                    color: Some(color.clone()),
                    source_scope: Some((*scope).to_owned()),
                    source_property: Some(property),
                    confidence: if *scope == scopes[0] {
                        Confidence::Exact
                    } else {
                        Confidence::Fallback
                    },
                });
            }
        }
        if result.emphasis_source.is_none()
            && (!style.modifiers.is_empty() || style.underline.is_some())
        {
            result.modifiers = style.modifiers.clone();
            result.underline = style.underline.is_some();
            result.emphasis_source = Some((*scope).to_owned());
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::indexmap;
    use theme_ir::{Style, Underline};

    #[test]
    fn preserves_partial_component_styles_and_property_sources() {
        let theme = ResolvedTheme {
            name: "arbitrary".into(),
            source_path: "arbitrary.toml".into(),
            palette: Default::default(),
            warnings: vec![],
            scopes: indexmap! {
                "ui.menu".into() => Style { fg: Some("#222222".into()), bg: Some("#eeeeee".into()), ..Default::default() },
                "ui.menu.selected".into() => Style { bg: Some("#123456".into()), modifiers: vec![Modifier::Bold], underline: Some(Underline::default()), ..Default::default() },
                "ui.selection.primary".into() => Style { bg: Some("#abcdef".into()), ..Default::default() },
                "ui.selection".into() => Style { fg: Some("#ffffff".into()), bg: Some("#555555".into()), ..Default::default() },
            },
        };
        let ui = derive_ui_styles(&theme);
        let fg = ui.selected_menu.fg.unwrap();
        assert_eq!(fg.color.as_deref(), Some("#222222"));
        assert_eq!(fg.source_scope.as_deref(), Some("ui.menu"));
        assert_eq!(fg.confidence, Confidence::Fallback);
        assert_eq!(
            ui.selected_menu.bg.unwrap().source_property,
            Some(SourceProperty::Bg)
        );
        assert_eq!(ui.selected_menu.modifiers, vec![Modifier::Bold]);
        assert!(ui.selected_menu.underline);
        assert_eq!(
            ui.primary_selection.bg.unwrap().color.as_deref(),
            Some("#abcdef")
        );
        assert_eq!(
            ui.primary_selection.fg.unwrap().confidence,
            Confidence::Fallback
        );
        assert_eq!(ui.focused_control, SemanticUiStyle::default());
    }
}
