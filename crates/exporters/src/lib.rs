mod base16;
mod bat;
mod kitty;
mod report;

pub use base16::export_base16_yaml;
pub use bat::export_bat_tmtheme;
pub use kitty::export_kitty;
pub use report::{render_report, ExportReport};

#[cfg(test)]
mod tests {
    use super::*;
    use camino::Utf8PathBuf;
    use helix_theme::{parse_str, resolve_raw};
    use palette16::extract_base16;
    use semantic_roles::derive_roles;

    #[test]
    fn exports_kitty_golden_minimal() {
        let (theme, roles, palette, mut warnings) = minimal_pipeline();
        warnings.extend(theme.warnings.clone());
        let (kitty, _) = export_kitty(&theme, &roles, &palette, warnings);
        assert_eq!(
            kitty,
            include_str!("../../../tests/golden/kitty/minimal.conf")
        );
    }

    #[test]
    fn exports_base16_golden_minimal() {
        let (_, _, palette, _) = minimal_pipeline();
        let yaml = export_base16_yaml(&palette).unwrap();
        assert_eq!(
            yaml,
            include_str!("../../../tests/golden/base16/minimal.yaml")
        );
    }

    #[test]
    fn exports_bat_golden_minimal() {
        let (theme, roles, palette, mut warnings) = minimal_pipeline();
        warnings.extend(theme.warnings.clone());
        let (tmtheme, _) = export_bat_tmtheme(&theme, &roles, &palette, warnings);
        assert_eq!(
            tmtheme,
            include_str!("../../../tests/golden/bat/minimal.tmTheme")
        );
    }

    fn minimal_pipeline() -> (
        theme_ir::ResolvedTheme,
        semantic_roles::SemanticRoles,
        palette16::Base16Palette,
        Vec<theme_ir::Warning>,
    ) {
        let raw = parse_str(
            "minimal",
            Utf8PathBuf::from("minimal.toml"),
            include_str!("../../../tests/fixtures/minimal.toml"),
        )
        .unwrap();
        let theme = resolve_raw(raw);
        let (roles, mut warnings) = derive_roles(&theme);
        let (palette, palette_warnings) = extract_base16(&roles);
        warnings.extend(palette_warnings);
        (theme, roles, palette, warnings)
    }
}
