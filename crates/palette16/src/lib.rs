use indexmap::IndexMap;
use semantic_roles::{role_color, Role, SemanticRoles};
use serde::{Deserialize, Serialize};
use theme_ir::{darken, lighten, luminance, mix, Confidence, SemanticRoleValue, Warning};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Base16Color {
    pub color: String,
    pub source_role: Option<Role>,
    pub confidence: Confidence,
}

pub type Base16Palette = IndexMap<String, Base16Color>;

pub fn extract_base16(roles: &SemanticRoles) -> (Base16Palette, Vec<Warning>) {
    let mut palette = IndexMap::new();
    let mut warnings = Vec::new();

    let base00 = direct_or_default(roles, Role::Background, "#000000", "base00", &mut warnings);
    let base05 = direct_or_default(roles, Role::Foreground, "#ffffff", "base05", &mut warnings);
    let base01 = choose(roles, &[Role::Surface]).unwrap_or_else(|| {
        warnings.push(Warning::new(
            "inferred_base16_color",
            "Inferred base01 from background",
        ));
        Base16Color {
            color: if luminance(&base00.color) > 0.5 {
                darken(&base00.color, 0.05)
            } else {
                lighten(&base00.color, 0.05)
            },
            source_role: Some(Role::Background),
            confidence: Confidence::Inferred,
        }
    });
    let base02 = choose(roles, &[Role::Selection])
        .unwrap_or_else(|| fallback("base02", &base01, &mut warnings));
    let base03 = choose(roles, &[Role::MutedForeground, Role::Comment]).unwrap_or_else(|| {
        inferred_mix("base03", &base00.color, &base05.color, 0.40, &mut warnings)
    });
    let base04 = choose(roles, &[Role::MutedForeground]).unwrap_or_else(|| {
        inferred_mix("base04", &base00.color, &base05.color, 0.55, &mut warnings)
    });
    let base06 = choose(roles, &[Role::BrightForeground]).unwrap_or_else(|| {
        inferred_mix("base06", &base00.color, &base05.color, 0.80, &mut warnings)
    });
    let base07 = choose(roles, &[Role::BrightForeground]).unwrap_or_else(|| {
        inferred_mix("base07", &base00.color, &base05.color, 0.95, &mut warnings)
    });
    let base08 = choose_required(
        roles,
        &[Role::Error, Role::Keyword],
        &base05,
        "base08",
        &mut warnings,
    );
    let base09 = choose_required(
        roles,
        &[Role::Number, Role::Constant],
        &base05,
        "base09",
        &mut warnings,
    );
    let base0a = choose_required(
        roles,
        &[Role::Warning, Role::Type],
        &base05,
        "base0A",
        &mut warnings,
    );
    let base0b = choose_required(
        roles,
        &[Role::String, Role::GitAdded],
        &base05,
        "base0B",
        &mut warnings,
    );
    let base0c = choose_required(
        roles,
        &[Role::Special, Role::Info],
        &base05,
        "base0C",
        &mut warnings,
    );
    let base0d = choose_required(roles, &[Role::Function], &base05, "base0D", &mut warnings);
    let base0e = choose_required(roles, &[Role::Keyword], &base05, "base0E", &mut warnings);
    let base0f = choose_required(
        roles,
        &[Role::Operator, Role::Constant],
        &base05,
        "base0F",
        &mut warnings,
    );

    insert(&mut palette, "base00", base00);
    insert(&mut palette, "base01", base01);
    insert(&mut palette, "base02", base02);
    insert(&mut palette, "base03", base03);
    insert(&mut palette, "base04", base04);
    insert(&mut palette, "base05", base05);
    insert(&mut palette, "base06", base06);
    insert(&mut palette, "base07", base07);
    insert(&mut palette, "base08", base08);
    insert(&mut palette, "base09", base09);
    insert(&mut palette, "base0A", base0a);
    insert(&mut palette, "base0B", base0b);
    insert(&mut palette, "base0C", base0c);
    insert(&mut palette, "base0D", base0d);
    insert(&mut palette, "base0E", base0e);
    insert(&mut palette, "base0F", base0f);

    (palette, warnings)
}

fn insert(palette: &mut Base16Palette, key: &str, color: Base16Color) {
    palette.insert(key.to_owned(), color);
}

fn direct_or_default(
    roles: &SemanticRoles,
    role: Role,
    default: &str,
    key: &str,
    warnings: &mut Vec<Warning>,
) -> Base16Color {
    choose(roles, &[role]).unwrap_or_else(|| {
        warnings.push(Warning::new(
            "fallback_base16_color",
            format!("Using default fallback for {key}"),
        ));
        Base16Color {
            color: default.to_owned(),
            source_role: None,
            confidence: Confidence::Fallback,
        }
    })
}

fn choose_required(
    roles: &SemanticRoles,
    candidates: &[Role],
    fallback_color: &Base16Color,
    key: &str,
    warnings: &mut Vec<Warning>,
) -> Base16Color {
    choose(roles, candidates).unwrap_or_else(|| fallback(key, fallback_color, warnings))
}

fn choose(roles: &SemanticRoles, candidates: &[Role]) -> Option<Base16Color> {
    for role in candidates {
        if let Some(color) = role_color(roles, *role) {
            return Some(Base16Color {
                color: color.to_owned(),
                source_role: Some(*role),
                confidence: Confidence::Exact,
            });
        }
    }
    None
}

fn fallback(key: &str, color: &Base16Color, warnings: &mut Vec<Warning>) -> Base16Color {
    warnings.push(Warning::new(
        "fallback_base16_color",
        format!("Using fallback color for {key}"),
    ));
    Base16Color {
        color: color.color.clone(),
        source_role: color.source_role,
        confidence: Confidence::Fallback,
    }
}

fn inferred_mix(
    key: &str,
    base00: &str,
    base05: &str,
    amount: f32,
    warnings: &mut Vec<Warning>,
) -> Base16Color {
    warnings.push(Warning::new(
        "inferred_base16_color",
        format!("Inferred {key} by mixing background and foreground"),
    ));
    Base16Color {
        color: mix(base00, base05, amount),
        source_role: None,
        confidence: Confidence::Inferred,
    }
}

pub fn color<'a>(palette: &'a Base16Palette, key: &str) -> &'a str {
    palette
        .get(key)
        .map(|entry| entry.color.as_str())
        .unwrap_or("#000000")
}

pub fn compact_colors(palette: &Base16Palette) -> IndexMap<String, String> {
    palette
        .iter()
        .map(|(key, value)| (key.clone(), value.color.clone()))
        .collect()
}

#[allow(dead_code)]
fn _keeps_semantic_role_value_in_docs(_: &SemanticRoleValue) {}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::indexmap;

    fn value(color: &str) -> SemanticRoleValue {
        SemanticRoleValue {
            color: Some(color.to_owned()),
            source_scope: None,
            source_property: None,
            confidence: Confidence::Exact,
        }
    }

    #[test]
    fn extracts_base16_from_roles() {
        let roles = indexmap! {
            Role::Background => value("#101010"),
            Role::Foreground => value("#eeeeee"),
            Role::Function => value("#3366ff"),
            Role::Keyword => value("#cc33cc"),
            Role::String => value("#33aa33"),
        };
        let (palette, _) = extract_base16(&roles);
        assert_eq!(color(&palette, "base00"), "#101010");
        assert_eq!(color(&palette, "base05"), "#eeeeee");
        assert_eq!(color(&palette, "base0D"), "#3366ff");
        assert_eq!(color(&palette, "base0E"), "#cc33cc");
        assert_eq!(color(&palette, "base0B"), "#33aa33");
    }
}
