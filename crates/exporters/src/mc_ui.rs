//! Resolve semantic component styles and apply MC-specific readability policy.
use crate::report::{role_source, PreservedItem};
use palette16::{color, Base16Palette};
use semantic_roles::{role_color, Role, SemanticRoles, SemanticUiStyle, SemanticUiStyles};
use theme_ir::{luminance, mix, parse_rgb, Modifier, Warning};

#[derive(Clone, Debug)]
pub(super) struct UiStyle {
    pub fg: String,
    pub bg: String,
    pub attrs: Vec<String>,
    fg_source: Option<String>,
    bg_source: Option<String>,
    emphasis_source: Option<String>,
}

impl UiStyle {
    pub fn render(&self, quantized: bool) -> String {
        let (fg, bg) = if quantized {
            (token(&self.fg), token(&self.bg))
        } else {
            (self.fg.clone(), self.bg.clone())
        };
        let separator = if quantized { "," } else { ";" };
        let mut result = format!("{fg}{separator}{bg}");
        if !self.attrs.is_empty() {
            result.push_str(separator);
            result.push_str(&self.attrs.join("+"));
        }
        result
    }

    pub fn preserved(&self, target: &str) -> Vec<PreservedItem> {
        [
            ("fg", &self.fg_source),
            ("bg", &self.bg_source),
            ("attributes", &self.emphasis_source),
        ]
        .into_iter()
        .filter_map(|(property, source)| {
            source.as_ref().map(|source| PreservedItem {
                target: format!("{target}.{property}"),
                source: source.clone(),
            })
        })
        .collect()
    }
}

#[derive(Clone)]
pub(super) struct McUiStyles {
    menu: UiStyle,
    selected: UiStyle,
    popup: UiStyle,
    focused: UiStyle,
    inactive: UiStyle,
}

impl McUiStyles {
    pub fn new(
        ui: &SemanticUiStyles,
        roles: &SemanticRoles,
        palette: &Base16Palette,
        warnings: &mut Vec<Warning>,
    ) -> Self {
        let base = UiStyle {
            fg: role_color(roles, Role::Foreground)
                .unwrap_or(color(palette, "base05"))
                .to_owned(),
            bg: role_color(roles, Role::Background)
                .unwrap_or(color(palette, "base00"))
                .to_owned(),
            attrs: vec![],
            fg_source: roles.get(&Role::Foreground).and_then(role_source),
            bg_source: roles.get(&Role::Background).and_then(role_source),
            emphasis_source: None,
        };
        let mut menu = complete(&ui.menu, &base);
        let mut popup = complete(&ui.popup, &base);
        correct(&mut menu, None, 4.5, "menu._default_", warnings, false);
        correct(&mut popup, None, 4.5, "dialog._default_", warnings, false);

        let mut selected = [
            &ui.selected_menu,
            &ui.focused_control,
            &ui.primary_selection,
        ]
        .into_iter()
        .filter(|style| **style != SemanticUiStyle::default())
        .map(|style| complete(style, &menu))
        .find(|style| distinct(style, &menu))
        .unwrap_or_else(|| reversed(&menu));
        correct(
            &mut selected,
            Some(&menu.bg),
            4.5,
            "menu.menusel",
            warnings,
            false,
        );
        let candidate = complete(&ui.focused_control, &popup);
        let mut focused = if distinct(&candidate, &popup) {
            candidate
        } else {
            selected.clone()
        };
        correct(
            &mut focused,
            Some(&popup.bg),
            4.5,
            "dialog.dfocus",
            warnings,
            false,
        );
        let mut inactive = complete(&ui.inactive_text, &menu);
        inactive.bg.clone_from(&menu.bg);
        inactive.bg_source.clone_from(&menu.bg_source);
        if ui.inactive_text.fg.is_none() {
            inactive.fg = role_color(roles, Role::MutedForeground)
                .unwrap_or(color(palette, "base03"))
                .to_owned();
            inactive.fg_source = roles.get(&Role::MutedForeground).and_then(role_source);
        }
        correct(
            &mut inactive,
            None,
            3.0,
            "menu.menuinactive",
            warnings,
            false,
        );
        Self {
            menu,
            selected,
            popup,
            focused,
            inactive,
        }
    }

    pub fn quantized(&self, warnings: &mut Vec<Warning>) -> Self {
        let mut result = self.clone();
        correct(
            &mut result.menu,
            None,
            4.5,
            "menu._default_",
            warnings,
            true,
        );
        correct(
            &mut result.popup,
            None,
            4.5,
            "dialog._default_",
            warnings,
            true,
        );
        correct(
            &mut result.selected,
            Some(&result.menu.bg),
            4.5,
            "menu.menusel",
            warnings,
            true,
        );
        correct(
            &mut result.focused,
            Some(&result.popup.bg),
            4.5,
            "dialog.dfocus",
            warnings,
            true,
        );
        // Inactive entries share the rendered menu background.
        result.inactive.bg.clone_from(&result.menu.bg);
        result.inactive.bg_source.clone_from(&result.menu.bg_source);
        correct(
            &mut result.inactive,
            None,
            3.0,
            "menu.menuinactive",
            warnings,
            true,
        );
        result
    }

    pub fn entry(&self, section: &str, key: &str) -> Option<UiStyle> {
        let style = match (section, key) {
            ("menu" | "popupmenu", "_default_") => &self.menu,
            ("menu" | "popupmenu", "menusel") => &self.selected,
            ("menu", "menuhot") => &self.menu,
            ("menu", "menuhotsel") => &self.selected,
            ("menu", "menuinactive") => &self.inactive,
            ("popupmenu", "menutitle") => &self.menu,
            ("dialog", "_default_" | "dhotnormal" | "dtitle") => &self.popup,
            ("dialog", "dfocus" | "dhotfocus") => &self.focused,
            _ => return None,
        };
        let mut style = style.clone();
        if matches!(key, "menuhot" | "menuhotsel" | "dhotnormal" | "dhotfocus")
            && !style.attrs.iter().any(|attr| attr == "underline")
        {
            style.attrs.push("underline".to_owned());
            style.emphasis_source = None;
        }
        if matches!(key, "menutitle" | "dtitle") && !style.attrs.iter().any(|attr| attr == "bold") {
            style.attrs.push("bold".to_owned());
            style.emphasis_source = None;
        }
        Some(style)
    }

    pub fn table_entry(&self, key: &str) -> Option<UiStyle> {
        let (section, field) = match key {
            "menunormal" => ("menu", "_default_"),
            "menusel" | "menuhot" | "menuhotsel" | "menuinactive" => ("menu", key),
            "pmenunormal" => ("popupmenu", "_default_"),
            "pmenusel" => ("popupmenu", "menusel"),
            "pmenutitle" => ("popupmenu", "menutitle"),
            "dnormal" => ("dialog", "_default_"),
            "dfocus" | "dhotnormal" | "dhotfocus" | "dtitle" => ("dialog", key),
            _ => return None,
        };
        self.entry(section, field)
    }
}

fn complete(style: &SemanticUiStyle, base: &UiStyle) -> UiStyle {
    let mut result = base.clone();
    for (value, target, source) in [
        (&style.fg, &mut result.fg, &mut result.fg_source),
        (&style.bg, &mut result.bg, &mut result.bg_source),
    ] {
        if let Some(value) = value {
            if let Some(color) = &value.color {
                *target = color.clone();
                *source = role_source(value);
            }
        }
    }
    if style.emphasis_source.is_some() {
        result.attrs.clear();
        for (modifier, attr) in [(Modifier::Bold, "bold"), (Modifier::Italic, "italic")] {
            if style.modifiers.contains(&modifier) {
                result.attrs.push(attr.to_owned());
            }
        }
        if style.underline {
            result.attrs.push("underline".to_owned());
        }
        result.emphasis_source = style.emphasis_source.clone();
    }
    if style.modifiers.contains(&Modifier::Reversed) {
        result = reversed(&result);
        // Reversal was materialized, not exported as an attribute.
        result.emphasis_source = None;
    }
    result
}

fn reversed(style: &UiStyle) -> UiStyle {
    let mut result = style.clone();
    std::mem::swap(&mut result.fg, &mut result.bg);
    // Reversed colors are derived, not exact copies of their target properties.
    result.fg_source = None;
    result.bg_source = None;
    result
}

fn distinct(style: &UiStyle, normal: &UiStyle) -> bool {
    style.fg != normal.fg || style.bg != normal.bg || style.attrs != normal.attrs
}

pub(super) fn contrast(a: &str, b: &str) -> f32 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

fn correct(
    style: &mut UiStyle,
    surrounding: Option<&str>,
    threshold: f32,
    target: &str,
    warnings: &mut Vec<Warning>,
    quantized: bool,
) {
    let original = style.clone();
    if quantized {
        style.bg = nearest(&style.bg, |_| true);
        style.fg = nearest(&style.fg, |_| true);
    }
    if let Some(surrounding) = surrounding {
        if contrast(&style.bg, surrounding) < 1.5 {
            style.bg = adjusted(&style.bg, surrounding, 1.5, quantized);
        }
    }
    if contrast(&style.fg, &style.bg) < threshold {
        style.fg = adjusted(&style.fg, &style.bg, threshold, quantized);
    }
    if style.fg != original.fg {
        style.fg_source = None;
    }
    if style.bg != original.bg {
        style.bg_source = None;
    }
    if style.fg != original.fg || style.bg != original.bg {
        let target = if quantized {
            match target {
                "menu._default_" => "menunormal",
                "menu.menusel" => "menusel",
                "menu.menuinactive" => "menuinactive",
                "dialog._default_" => "dnormal",
                "dialog.dfocus" => "dfocus",
                _ => target,
            }
        } else {
            target
        };
        warnings.push(Warning::new("mc_ui_color_adjusted", format!(
            "{}{}: {} on {} -> {} on {} ({}; text contrast >= {threshold}:1, state background contrast >= 1.5:1 when applicable)",
            if quantized { "MC_COLOR_TABLE." } else { "" }, target,
            original.fg, original.bg, style.fg, style.bg,
            if quantized { "256-color quantization/readability" } else { "readability" },
        )));
    }
}

fn adjusted(original: &str, against: &str, minimum: f32, quantized: bool) -> String {
    if quantized {
        return nearest(original, |candidate| {
            contrast(candidate, against) >= minimum
        });
    }
    for amount in 0..=255 {
        for endpoint in ["#000000", "#ffffff"] {
            let candidate = mix(original, endpoint, amount as f32 / 255.0);
            if contrast(&candidate, against) >= minimum {
                return candidate;
            }
        }
    }
    unreachable!("black or white always meets MC contrast thresholds")
}

fn cube(component: u8) -> u8 {
    [0, 95, 135, 175, 215, 255][component as usize]
}

fn palette_colors() -> impl Iterator<Item = String> {
    (0..216)
        .map(|index| {
            format!(
                "#{:02x}{:02x}{:02x}",
                cube(index / 36),
                cube(index / 6 % 6),
                cube(index % 6)
            )
        })
        .chain((0..24).map(|index| {
            let value = 8 + 10 * index;
            format!("#{value:02x}{value:02x}{value:02x}")
        }))
}

fn distance(a: &str, b: &str) -> u32 {
    let a = parse_rgb(a).expect("resolved semantic RGB");
    let b = parse_rgb(b).expect("MC palette RGB");
    [(a.r, b.r), (a.g, b.g), (a.b, b.b)]
        .into_iter()
        .map(|(a, b)| (i32::from(a) - i32::from(b)).pow(2) as u32)
        .sum()
}

fn nearest(original: &str, allowed: impl Fn(&str) -> bool) -> String {
    palette_colors()
        .filter(|candidate| allowed(candidate))
        .min_by_key(|candidate| (distance(original, candidate), candidate.clone()))
        .expect("MC palette contains black and white")
}

fn token(color: &str) -> String {
    let rgb = parse_rgb(color).expect("MC palette RGB");
    if rgb.r == rgb.g
        && rgb.g == rgb.b
        && (8..=238).contains(&rgb.r)
        && (rgb.r - 8).is_multiple_of(10)
    {
        return format!("gray{}", (rgb.r - 8) / 10);
    }
    let index = |value| {
        (0..6)
            .find(|i| cube(*i) == value)
            .expect("MC cube component")
    };
    format!("rgb{}{}{}", index(rgb.r), index(rgb.g), index(rgb.b))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn semantic_color(color: &str) -> theme_ir::SemanticRoleValue {
        theme_ir::SemanticRoleValue {
            color: Some(color.to_owned()),
            source_scope: None,
            source_property: None,
            confidence: theme_ir::Confidence::Exact,
        }
    }

    #[test]
    fn absent_and_indistinguishable_selection_styles_follow_fallback_order() {
        let roles = indexmap::indexmap! {
            Role::Foreground => semantic_color("#ffffff"),
            Role::Background => semantic_color("#111111"),
        };
        let (palette, _) = palette16::extract_base16(&roles);
        let mut ui = SemanticUiStyles::default();
        ui.selected_menu.fg = Some(semantic_color("#ffffff"));
        ui.selected_menu.bg = Some(semantic_color("#111111"));
        ui.focused_control.fg = Some(semantic_color("#ffffff"));
        ui.focused_control.bg = Some(semantic_color("#2266aa"));
        ui.primary_selection.fg = Some(semantic_color("#111111"));
        ui.primary_selection.bg = Some(semantic_color("#aaaaaa"));
        let mut warnings = vec![];
        let styles = McUiStyles::new(&ui, &roles, &palette, &mut warnings);
        assert_eq!(styles.selected.bg, "#2266aa");
        ui.focused_control = SemanticUiStyle::default();
        let styles = McUiStyles::new(&ui, &roles, &palette, &mut warnings);
        assert_eq!(styles.selected.bg, "#aaaaaa");
        ui.primary_selection = SemanticUiStyle::default();
        let styles = McUiStyles::new(&ui, &roles, &palette, &mut warnings);
        assert_eq!(styles.selected.fg, "#111111");
        assert_eq!(styles.selected.bg, "#ffffff");
        assert!(styles.selected.fg_source.is_none());
    }

    #[test]
    fn contrast_and_correction_use_actual_rgb_values() {
        assert!((contrast("#000000", "#ffffff") - 21.0).abs() < 0.001);
        assert_eq!(contrast("#123456", "#123456"), 1.0);
        for (fg, bg) in [
            ("#eeeeee", "#eaeaed"),
            ("#777777", "#888888"),
            ("#222222", "#222222"),
        ] {
            let corrected = adjusted(fg, bg, 4.5, false);
            assert!(contrast(&corrected, bg) >= 4.5);
            let quantized_bg = nearest(bg, |_| true);
            let quantized_fg = adjusted(fg, &quantized_bg, 4.5, true);
            assert!(contrast(&quantized_fg, &quantized_bg) >= 4.5);
        }
        assert_eq!(nearest("#5f87af", |_| true), "#5f87af");
        assert_eq!(token("#5f87af"), "rgb123");
        assert_eq!(token("#eeeeee"), "gray23");
    }

    #[test]
    fn reversed_components_materialize_colors_and_preserve_supported_emphasis() {
        let base = UiStyle {
            fg: "#ffffff".into(),
            bg: "#000000".into(),
            attrs: vec![],
            fg_source: Some("ui.text.fg".into()),
            bg_source: Some("ui.background.bg".into()),
            emphasis_source: None,
        };
        let input = SemanticUiStyle {
            modifiers: vec![Modifier::Reversed, Modifier::Bold, Modifier::Italic],
            underline: true,
            emphasis_source: Some("ui.menu.selected".into()),
            ..Default::default()
        };
        let output = complete(&input, &base);
        assert_eq!(output.fg, "#000000");
        assert_eq!(output.bg, "#ffffff");
        assert_eq!(output.attrs, vec!["bold", "italic", "underline"]);
        assert!(output.preserved("menu.menusel").is_empty());
    }

    #[test]
    fn passing_pairs_are_preserved_and_adjusted_properties_lose_exact_provenance() {
        let mut style = UiStyle {
            fg: "#000000".into(),
            bg: "#ffffff".into(),
            attrs: vec!["bold".into()],
            fg_source: Some("ui.menu.fg".into()),
            bg_source: Some("ui.menu.bg".into()),
            emphasis_source: Some("ui.menu".into()),
        };
        let mut warnings = vec![];
        correct(
            &mut style,
            None,
            4.5,
            "menu._default_",
            &mut warnings,
            false,
        );
        assert!(warnings.is_empty());
        assert!(style.fg_source.is_some());
        correct(
            &mut style,
            Some("#ffffff"),
            4.5,
            "menu.menusel",
            &mut warnings,
            false,
        );
        assert!(contrast(&style.bg, "#ffffff") >= 1.5);
        assert!(contrast(&style.fg, &style.bg) >= 4.5);
        assert!(style.bg_source.is_none());
        assert!(style.fg_source.is_some());
        assert_eq!(style.attrs, vec!["bold"]);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("#000000 on #ffffff ->"));
    }
}
