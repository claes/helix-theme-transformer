mod base16;
mod bat;
mod dircolors;
mod file_kinds;
mod gitui;
mod kitty;
mod mc;
mod report;

use palette16::Base16Palette;
use semantic_roles::SemanticRoles;
use theme_ir::{ResolvedTheme, Warning};

pub use base16::export_base16_yaml;
pub use bat::export_bat_tmtheme;
pub use dircolors::export_dircolors;
pub use gitui::{export_gitui, GituiTheme};
pub use kitty::export_kitty;
pub use mc::{export_mc, export_mc_skin, McExport};
pub use report::{render_report, ExportReport};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportedFile {
    pub relative_path: String,
    pub contents: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportedFormat {
    pub files: Vec<ExportedFile>,
    pub report: ExportReport,
}

pub fn export_kitty_format(
    file_name: &str,
    theme: &ResolvedTheme,
    roles: &SemanticRoles,
    palette: &Base16Palette,
    warnings: Vec<Warning>,
) -> ExportedFormat {
    let (contents, report) = export_kitty(theme, roles, palette, warnings);
    single_file_format(format!("kitty/{file_name}.conf"), contents, report)
}

pub fn export_base16_format(
    file_name: &str,
    theme: &ResolvedTheme,
    palette: &Base16Palette,
    warnings: Vec<Warning>,
) -> anyhow::Result<ExportedFormat> {
    let contents = export_base16_yaml(palette)?;
    let report = ExportReport {
        exporter: "base16".to_owned(),
        source: theme.source_path.to_string(),
        preserved: Vec::new(),
        dropped: Vec::new(),
        warnings,
    };
    Ok(single_file_format(
        format!("base16/{file_name}.yaml"),
        contents,
        report,
    ))
}

pub fn export_bat_format(
    file_name: &str,
    theme: &ResolvedTheme,
    roles: &SemanticRoles,
    palette: &Base16Palette,
    warnings: Vec<Warning>,
) -> ExportedFormat {
    let (contents, report) = export_bat_tmtheme(theme, roles, palette, warnings);
    single_file_format(format!("bat/{file_name}.tmTheme"), contents, report)
}

pub fn export_gitui_format(
    theme: &ResolvedTheme,
    roles: &SemanticRoles,
    palette: &Base16Palette,
    warnings: Vec<Warning>,
) -> ExportedFormat {
    let gitui = export_gitui(theme, roles, palette, warnings);
    ExportedFormat {
        files: vec![
            ExportedFile {
                relative_path: "gitui/theme.ron".to_owned(),
                contents: gitui.theme_ron,
            },
            ExportedFile {
                relative_path: format!("gitui/{}", gitui.syntax_file_name),
                contents: gitui.syntax_tmtheme,
            },
        ],
        report: gitui.report,
    }
}

pub fn export_mc_format(
    file_name: &str,
    theme: &ResolvedTheme,
    roles: &SemanticRoles,
    palette: &Base16Palette,
    warnings: Vec<Warning>,
) -> ExportedFormat {
    let mc = mc::export_mc(theme, roles, palette, warnings);
    ExportedFormat {
        files: vec![
            ExportedFile {
                relative_path: format!("mc/{file_name}.ini"),
                contents: mc.skin_ini,
            },
            ExportedFile {
                relative_path: "mc/filehighlight.ini".to_owned(),
                contents: mc.filehighlight_ini,
            },
            ExportedFile {
                relative_path: "mc/colortable.env".to_owned(),
                contents: mc.colortable_env,
            },
        ],
        report: mc.report,
    }
}

pub fn export_dircolors_format(
    file_name: &str,
    theme: &ResolvedTheme,
    roles: &SemanticRoles,
    palette: &Base16Palette,
    warnings: Vec<Warning>,
) -> ExportedFormat {
    let (contents, report) = export_dircolors(theme, roles, palette, warnings);
    single_file_format(format!("dircolors/{file_name}.dircolors"), contents, report)
}

fn single_file_format(
    relative_path: String,
    contents: String,
    report: ExportReport,
) -> ExportedFormat {
    ExportedFormat {
        files: vec![ExportedFile {
            relative_path,
            contents,
        }],
        report,
    }
}

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

    #[test]
    fn exports_gitui_golden_minimal() {
        let (theme, roles, palette, mut warnings) = minimal_pipeline();
        warnings.extend(theme.warnings.clone());
        let gitui = export_gitui(&theme, &roles, &palette, warnings);
        assert_eq!(
            gitui.theme_ron,
            include_str!("../../../tests/golden/gitui/theme.ron")
        );
        assert_eq!(gitui.syntax_file_name, "minimal.tmTheme");
        assert_eq!(
            gitui.syntax_tmtheme,
            include_str!("../../../tests/golden/bat/minimal.tmTheme")
        );
    }

    #[test]
    fn exports_mc_golden_minimal() {
        let (theme, roles, palette, mut warnings) = minimal_pipeline();
        warnings.extend(theme.warnings.clone());
        let mc = export_mc(&theme, &roles, &palette, warnings);
        assert_eq!(
            mc.skin_ini,
            include_str!("../../../tests/golden/mc/minimal.ini")
        );
        assert_eq!(
            mc.filehighlight_ini,
            include_str!("../../../tests/golden/mc/filehighlight.ini")
        );
        assert_eq!(
            mc.colortable_env,
            include_str!("../../../tests/golden/mc/colortable.env")
        );
    }

    #[test]
    fn exports_dircolors_golden_minimal() {
        let (theme, roles, palette, mut warnings) = minimal_pipeline();
        warnings.extend(theme.warnings.clone());
        let (dircolors, _) = export_dircolors(&theme, &roles, &palette, warnings);
        assert_eq!(
            dircolors,
            include_str!("../../../tests/golden/dircolors/minimal.dircolors")
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
