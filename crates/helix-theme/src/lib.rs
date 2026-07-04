use camino::{Utf8Path, Utf8PathBuf};
use indexmap::IndexMap;
use theme_ir::{normalize_hex, Modifier, ResolvedTheme, Style, Underline, UnderlineStyle, Warning};
use toml::Value;

#[derive(Debug, thiserror::Error)]
pub enum HelixThemeError {
    #[error("failed to read theme {path}: {source}")]
    Read {
        path: Utf8PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse TOML: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("theme path is not valid UTF-8: {0}")]
    NonUtf8Path(String),
    #[error("inherited theme cycle detected at {0}")]
    InheritanceCycle(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawTheme {
    pub name: String,
    pub source_path: Utf8PathBuf,
    pub inherits: Option<String>,
    pub palette: IndexMap<String, String>,
    pub scopes: IndexMap<String, RawStyle>,
    pub warnings: Vec<Warning>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RawStyle {
    pub fg: Option<String>,
    pub bg: Option<String>,
    pub underline: Option<RawUnderline>,
    pub modifiers: Vec<Modifier>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RawUnderline {
    pub color: Option<String>,
    pub style: Option<UnderlineStyle>,
}

pub fn parse_str(
    name: impl Into<String>,
    source_path: Utf8PathBuf,
    input: &str,
) -> Result<RawTheme, HelixThemeError> {
    let value: Value = toml::from_str(input)?;
    let table = value.as_table().cloned().unwrap_or_default();
    let inherits = table
        .get("inherits")
        .and_then(Value::as_str)
        .map(str::to_owned);

    let mut warnings = Vec::new();
    let mut palette = IndexMap::new();
    if let Some(Value::Table(entries)) = table.get("palette") {
        for (key, value) in entries {
            match value.as_str() {
                Some(color) => {
                    palette.insert(key.clone(), color.to_owned());
                }
                None => warnings.push(Warning::new(
                    "unsupported_palette_value",
                    format!("Palette entry `{key}` is not a string"),
                )),
            }
        }
    }

    let mut scopes = IndexMap::new();
    for (key, value) in table {
        if key == "inherits" || key == "palette" {
            continue;
        }
        match parse_style(&value) {
            Some(style) => {
                scopes.insert(key, style);
            }
            None => warnings.push(Warning::scoped(
                "unsupported_style",
                "Scope style is not a supported Helix style form",
                key,
            )),
        }
    }

    Ok(RawTheme {
        name: name.into(),
        source_path,
        inherits,
        palette,
        scopes,
        warnings,
    })
}

pub fn load_raw(path: &Utf8Path) -> Result<RawTheme, HelixThemeError> {
    let input = std::fs::read_to_string(path).map_err(|source| HelixThemeError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let name = path.file_stem().unwrap_or("theme").to_owned();
    parse_str(name, path.to_path_buf(), &input)
}

pub fn resolve_file(
    path: &Utf8Path,
    theme_dir: &Utf8Path,
) -> Result<ResolvedTheme, HelixThemeError> {
    resolve_file_inner(path, theme_dir, &mut Vec::new())
}

fn resolve_file_inner(
    path: &Utf8Path,
    theme_dir: &Utf8Path,
    stack: &mut Vec<String>,
) -> Result<ResolvedTheme, HelixThemeError> {
    let raw = load_raw(path)?;
    if stack.contains(&raw.name) {
        return Err(HelixThemeError::InheritanceCycle(raw.name));
    }
    stack.push(raw.name.clone());

    let resolved = if let Some(parent) = raw.inherits.as_deref() {
        let parent_path = theme_dir.join(format!("{parent}.toml"));
        let parent = resolve_file_inner(&parent_path, theme_dir, stack)?;
        overlay_and_resolve(parent, raw)
    } else {
        resolve_raw(raw)
    };
    stack.pop();
    Ok(resolved)
}

fn overlay_and_resolve(parent: ResolvedTheme, child: RawTheme) -> ResolvedTheme {
    let mut raw = RawTheme {
        name: child.name,
        source_path: child.source_path,
        inherits: None,
        palette: parent.palette,
        scopes: parent
            .scopes
            .into_iter()
            .map(|(scope, style)| (scope, RawStyle::from(style)))
            .collect(),
        warnings: parent.warnings,
    };
    raw.palette.extend(child.palette);
    raw.scopes.extend(child.scopes);
    raw.warnings.extend(child.warnings);
    resolve_raw(raw)
}

pub fn resolve_raw(raw: RawTheme) -> ResolvedTheme {
    let mut warnings = raw.warnings;
    let mut palette = IndexMap::new();
    for (name, value) in raw.palette {
        match normalize_hex(&value) {
            Ok(color) => {
                if color.len() == 9 {
                    warnings.push(Warning::new(
                        "alpha_color",
                        format!("Palette color `{name}` includes alpha"),
                    ));
                }
                palette.insert(name, color);
            }
            Err(_) => warnings.push(Warning::new(
                "invalid_palette_color",
                format!("Palette entry `{name}` is not a supported color"),
            )),
        }
    }

    let mut scopes = IndexMap::new();
    for (scope, style) in raw.scopes {
        let resolved = resolve_style(&scope, style, &palette, &mut warnings);
        scopes.insert(scope, resolved);
    }

    ResolvedTheme {
        name: raw.name,
        source_path: raw.source_path,
        palette,
        scopes,
        warnings,
    }
}

fn resolve_style(
    scope: &str,
    style: RawStyle,
    palette: &IndexMap<String, String>,
    warnings: &mut Vec<Warning>,
) -> Style {
    Style {
        fg: style
            .fg
            .as_deref()
            .and_then(|value| resolve_color(scope, "fg", value, palette, warnings)),
        bg: style
            .bg
            .as_deref()
            .and_then(|value| resolve_color(scope, "bg", value, palette, warnings)),
        underline: style.underline.map(|underline| Underline {
            color: underline.color.as_deref().and_then(|value| {
                resolve_color(scope, "underline.color", value, palette, warnings)
            }),
            style: underline.style,
        }),
        modifiers: style.modifiers,
    }
}

fn resolve_color(
    scope: &str,
    property: &str,
    value: &str,
    palette: &IndexMap<String, String>,
    warnings: &mut Vec<Warning>,
) -> Option<String> {
    if let Ok(color) = normalize_hex(value) {
        if color.len() == 9 {
            warnings.push(Warning::scoped(
                "alpha_color",
                format!("Color `{property}` includes alpha"),
                scope,
            ));
        }
        return Some(color);
    }
    if let Some(color) = palette.get(value) {
        return Some(color.clone());
    }
    warnings.push(Warning::scoped(
        "missing_palette_reference",
        format!("Could not resolve palette reference `{value}` for `{property}`"),
        scope,
    ));
    None
}

fn parse_style(value: &Value) -> Option<RawStyle> {
    if let Some(color) = value.as_str() {
        return Some(RawStyle {
            fg: Some(color.to_owned()),
            ..RawStyle::default()
        });
    }
    let table = value.as_table()?;
    Some(RawStyle {
        fg: table.get("fg").and_then(Value::as_str).map(str::to_owned),
        bg: table.get("bg").and_then(Value::as_str).map(str::to_owned),
        underline: table.get("underline").and_then(parse_underline),
        modifiers: table
            .get("modifiers")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .filter_map(parse_modifier)
            .collect(),
    })
}

fn parse_underline(value: &Value) -> Option<RawUnderline> {
    let table = value.as_table()?;
    Some(RawUnderline {
        color: table
            .get("color")
            .and_then(Value::as_str)
            .map(str::to_owned),
        style: table
            .get("style")
            .and_then(Value::as_str)
            .and_then(parse_underline_style),
    })
}

fn parse_modifier(value: &str) -> Option<Modifier> {
    match value {
        "bold" => Some(Modifier::Bold),
        "italic" => Some(Modifier::Italic),
        "dim" => Some(Modifier::Dim),
        "crossed_out" => Some(Modifier::CrossedOut),
        "reversed" => Some(Modifier::Reversed),
        _ => None,
    }
}

fn parse_underline_style(value: &str) -> Option<UnderlineStyle> {
    match value {
        "line" => Some(UnderlineStyle::Line),
        "curl" => Some(UnderlineStyle::Curl),
        "dashed" => Some(UnderlineStyle::Dashed),
        "dotted" => Some(UnderlineStyle::Dotted),
        "double_line" => Some(UnderlineStyle::DoubleLine),
        _ => None,
    }
}

impl From<Style> for RawStyle {
    fn from(style: Style) -> Self {
        Self {
            fg: style.fg,
            bg: style.bg,
            underline: style.underline.map(|underline| RawUnderline {
                color: underline.color,
                style: underline.style,
            }),
            modifiers: style.modifiers,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_resolves_palette_references() {
        let raw = parse_str(
            "demo",
            Utf8PathBuf::from("demo.toml"),
            r##"
            "function" = "banana"
            "ui.background" = { bg = "#111" }
            "diagnostic.error" = { underline = { color = "banana", style = "curl" } }
            "keyword" = { fg = "#abc", modifiers = ["bold", "italic"] }

            [palette]
            banana = "#7AA2F7"
            "##,
        )
        .unwrap();

        let resolved = resolve_raw(raw);
        assert_eq!(resolved.scopes["function"].fg.as_deref(), Some("#7aa2f7"));
        assert_eq!(
            resolved.scopes["ui.background"].bg.as_deref(),
            Some("#111111")
        );
        assert_eq!(
            resolved.scopes["diagnostic.error"]
                .underline
                .as_ref()
                .and_then(|u| u.color.as_deref()),
            Some("#7aa2f7")
        );
        assert_eq!(resolved.scopes["keyword"].modifiers.len(), 2);
    }

    #[test]
    fn warns_on_missing_palette_reference() {
        let raw = parse_str(
            "demo",
            Utf8PathBuf::from("demo.toml"),
            r#""keyword" = "missing""#,
        )
        .unwrap();
        let resolved = resolve_raw(raw);
        assert!(resolved.scopes["keyword"].fg.is_none());
        assert_eq!(resolved.warnings[0].code, "missing_palette_reference");
    }
}
