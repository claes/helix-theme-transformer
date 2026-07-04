use camino::Utf8PathBuf;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Warning {
    pub code: String,
    pub message: String,
    pub scope: Option<String>,
}

impl Warning {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            scope: None,
        }
    }

    pub fn scoped(
        code: impl Into<String>,
        message: impl Into<String>,
        scope: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            scope: Some(scope.into()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnderlineStyle {
    Line,
    Curl,
    Dashed,
    Dotted,
    DoubleLine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Modifier {
    Bold,
    Italic,
    Dim,
    CrossedOut,
    Reversed,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Underline {
    pub color: Option<String>,
    pub style: Option<UnderlineStyle>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Style {
    pub fg: Option<String>,
    pub bg: Option<String>,
    pub underline: Option<Underline>,
    pub modifiers: Vec<Modifier>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedTheme {
    pub name: String,
    pub source_path: Utf8PathBuf,
    pub palette: IndexMap<String, String>,
    pub scopes: IndexMap<String, Style>,
    pub warnings: Vec<Warning>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticRoleValue {
    pub color: Option<String>,
    pub source_scope: Option<String>,
    pub source_property: Option<SourceProperty>,
    pub confidence: Confidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    Exact,
    Fallback,
    Inferred,
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceProperty {
    Fg,
    Bg,
    UnderlineColor,
}

impl fmt::Display for SourceProperty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceProperty::Fg => f.write_str("fg"),
            SourceProperty::Bg => f.write_str("bg"),
            SourceProperty::UnderlineColor => f.write_str("underline.color"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ColorError {
    #[error("expected color to start with #")]
    MissingHash,
    #[error("unsupported hex color length")]
    UnsupportedLength,
    #[error("invalid hex color component")]
    InvalidHex,
}

pub fn normalize_hex(input: &str) -> Result<String, ColorError> {
    let hex = input.strip_prefix('#').ok_or(ColorError::MissingHash)?;
    if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ColorError::InvalidHex);
    }

    let lower = hex.to_ascii_lowercase();
    match lower.len() {
        3 => Ok(format!(
            "#{0}{0}{1}{1}{2}{2}",
            &lower[0..1],
            &lower[1..2],
            &lower[2..3]
        )),
        4 => Ok(format!(
            "#{0}{0}{1}{1}{2}{2}{3}{3}",
            &lower[0..1],
            &lower[1..2],
            &lower[2..3],
            &lower[3..4]
        )),
        6 | 8 => Ok(format!("#{lower}")),
        _ => Err(ColorError::UnsupportedLength),
    }
}

pub fn parse_rgb(color: &str) -> Result<Rgb, ColorError> {
    let normalized = normalize_hex(color)?;
    let hex = normalized
        .strip_prefix('#')
        .ok_or(ColorError::MissingHash)?;
    Ok(Rgb {
        r: u8::from_str_radix(&hex[0..2], 16).map_err(|_| ColorError::InvalidHex)?,
        g: u8::from_str_radix(&hex[2..4], 16).map_err(|_| ColorError::InvalidHex)?,
        b: u8::from_str_radix(&hex[4..6], 16).map_err(|_| ColorError::InvalidHex)?,
    })
}

pub fn format_rgb(rgb: Rgb) -> String {
    format!("#{:02x}{:02x}{:02x}", rgb.r, rgb.g, rgb.b)
}

pub fn mix(a: &str, b: &str, b_weight: f32) -> String {
    let a = parse_rgb(a).unwrap_or(Rgb { r: 0, g: 0, b: 0 });
    let b = parse_rgb(b).unwrap_or(Rgb {
        r: 255,
        g: 255,
        b: 255,
    });
    let weight = b_weight.clamp(0.0, 1.0);
    let inv = 1.0 - weight;
    format_rgb(Rgb {
        r: ((a.r as f32 * inv) + (b.r as f32 * weight)).round() as u8,
        g: ((a.g as f32 * inv) + (b.g as f32 * weight)).round() as u8,
        b: ((a.b as f32 * inv) + (b.b as f32 * weight)).round() as u8,
    })
}

pub fn lighten(color: &str, amount: f32) -> String {
    mix(color, "#ffffff", amount)
}

pub fn darken(color: &str, amount: f32) -> String {
    mix(color, "#000000", amount)
}

pub fn brighten(color: &str) -> String {
    lighten(color, 0.14)
}

pub fn luminance(color: &str) -> f32 {
    let rgb = parse_rgb(color).unwrap_or(Rgb { r: 0, g: 0, b: 0 });
    let channel = |value: u8| {
        let value = value as f32 / 255.0;
        if value <= 0.03928 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    (0.2126 * channel(rgb.r)) + (0.7152 * channel(rgb.g)) + (0.0722 * channel(rgb.b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_hex_forms() {
        assert_eq!(normalize_hex("#ABC").unwrap(), "#aabbcc");
        assert_eq!(normalize_hex("#abcd").unwrap(), "#aabbccdd");
        assert_eq!(normalize_hex("#AABBCC").unwrap(), "#aabbcc");
    }

    #[test]
    fn mixes_rgb_colors() {
        assert_eq!(mix("#000000", "#ffffff", 0.5), "#808080");
        assert_eq!(darken("#808080", 0.5), "#404040");
        assert_eq!(lighten("#000000", 0.25), "#404040");
    }
}
