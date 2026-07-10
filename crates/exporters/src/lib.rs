mod base16;
mod bat;
mod dircolors;
mod file_kinds;
mod gitui;
mod kitty;
mod mc;
mod report;
mod yazi;

use palette16::Base16Palette;
use semantic_roles::SemanticRoles;
use theme_ir::{ResolvedTheme, Warning};

pub use base16::{export_base16_terminal_script, export_base16_yaml};
pub use bat::export_bat_tmtheme;
pub use dircolors::export_dircolors;
pub use gitui::{export_gitui, GituiTheme};
pub use kitty::export_kitty;
pub use mc::{export_mc, export_mc_skin, McExport};
pub use report::{render_report, ExportReport};
pub use yazi::export_yazi;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportedFile {
    pub relative_path: String,
    pub contents: String,
    pub executable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportedFormat {
    pub files: Vec<ExportedFile>,
    pub report: ExportReport,
}

pub fn export_kitty_format(
    theme: &ResolvedTheme,
    roles: &SemanticRoles,
    palette: &Base16Palette,
    warnings: Vec<Warning>,
) -> ExportedFormat {
    let (contents, report) = export_kitty(theme, roles, palette, warnings);
    single_file_format("kitty/theme.conf".to_owned(), contents, report)
}

pub fn export_base16_format(
    theme: &ResolvedTheme,
    roles: &SemanticRoles,
    palette: &Base16Palette,
    warnings: Vec<Warning>,
) -> anyhow::Result<ExportedFormat> {
    let yaml = export_base16_yaml(palette)?;
    let shell = export_base16_terminal_script(theme, roles, palette);
    let report = ExportReport {
        exporter: "base16".to_owned(),
        source: theme.source_path.to_string(),
        preserved: Vec::new(),
        dropped: Vec::new(),
        warnings,
    };
    Ok(ExportedFormat {
        files: vec![
            ExportedFile {
                relative_path: "base16/theme.yaml".to_owned(),
                contents: yaml,
                executable: false,
            },
            ExportedFile {
                relative_path: "base16/set-terminal-colors.sh".to_owned(),
                contents: shell,
                executable: true,
            },
        ],
        report,
    })
}

pub fn export_bat_format(
    theme: &ResolvedTheme,
    roles: &SemanticRoles,
    palette: &Base16Palette,
    warnings: Vec<Warning>,
) -> ExportedFormat {
    let (contents, report) = export_bat_tmtheme(theme, roles, palette, warnings);
    single_file_format("bat/theme.tmTheme".to_owned(), contents, report)
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
                executable: false,
            },
            ExportedFile {
                relative_path: format!("gitui/{}", gitui.syntax_file_name),
                contents: gitui.syntax_tmtheme,
                executable: false,
            },
        ],
        report: gitui.report,
    }
}

pub fn export_mc_format(
    theme: &ResolvedTheme,
    roles: &SemanticRoles,
    palette: &Base16Palette,
    warnings: Vec<Warning>,
) -> ExportedFormat {
    let mc = mc::export_mc(theme, roles, palette, warnings);
    ExportedFormat {
        files: vec![
            ExportedFile {
                relative_path: "mc/theme.ini".to_owned(),
                contents: mc.skin_ini,
                executable: false,
            },
            ExportedFile {
                relative_path: "mc/filehighlight.ini".to_owned(),
                contents: mc.filehighlight_ini,
                executable: false,
            },
            ExportedFile {
                relative_path: "mc/colortable.env".to_owned(),
                contents: mc.colortable_env,
                executable: false,
            },
        ],
        report: mc.report,
    }
}

pub fn export_dircolors_format(
    theme: &ResolvedTheme,
    roles: &SemanticRoles,
    palette: &Base16Palette,
    warnings: Vec<Warning>,
) -> ExportedFormat {
    let (contents, report) = export_dircolors(theme, roles, palette, warnings);
    single_file_format("dircolors/theme.dircolors".to_owned(), contents, report)
}

pub fn export_yazi_format(
    theme: &ResolvedTheme,
    roles: &SemanticRoles,
    palette: &Base16Palette,
    warnings: Vec<Warning>,
) -> ExportedFormat {
    let (contents, report) = export_yazi(theme, roles, palette, warnings);
    single_file_format("yazi/theme.toml".to_owned(), contents, report)
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
            executable: false,
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
        let (theme, roles, palette, _) = minimal_pipeline();
        let yaml = export_base16_yaml(&palette).unwrap();
        assert_eq!(
            yaml,
            include_str!("../../../tests/golden/base16/minimal.yaml")
        );
        let shell = export_base16_terminal_script(&theme, &roles, &palette);
        assert_eq!(
            shell,
            include_str!("../../../tests/golden/base16/set-terminal-colors.sh")
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
        assert_eq!(gitui.syntax_file_name, "syntax.tmTheme");
        assert_eq!(
            gitui.syntax_tmtheme,
            include_str!("../../../tests/golden/bat/minimal.tmTheme")
        );
    }

    #[test]
    fn exported_format_paths_are_stable() {
        let (theme, roles, palette, warnings) = minimal_pipeline();

        let kitty = export_kitty_format(&theme, &roles, &palette, warnings.clone());
        assert_eq!(kitty.files[0].relative_path, "kitty/theme.conf");

        let base16 = export_base16_format(&theme, &roles, &palette, warnings.clone()).unwrap();
        assert_eq!(
            base16
                .files
                .iter()
                .map(|file| file.relative_path.as_str())
                .collect::<Vec<_>>(),
            vec!["base16/theme.yaml", "base16/set-terminal-colors.sh"]
        );
        assert!(!base16.files[0].executable);
        assert!(base16.files[1].executable);

        let bat = export_bat_format(&theme, &roles, &palette, warnings.clone());
        assert_eq!(bat.files[0].relative_path, "bat/theme.tmTheme");

        let gitui = export_gitui_format(&theme, &roles, &palette, warnings.clone());
        assert_eq!(gitui.files[0].relative_path, "gitui/theme.ron");
        assert_eq!(gitui.files[1].relative_path, "gitui/syntax.tmTheme");

        let mc = export_mc_format(&theme, &roles, &palette, warnings.clone());
        assert_eq!(mc.files[0].relative_path, "mc/theme.ini");
        assert_eq!(mc.files[1].relative_path, "mc/filehighlight.ini");
        assert_eq!(mc.files[2].relative_path, "mc/colortable.env");

        let dircolors = export_dircolors_format(&theme, &roles, &palette, warnings.clone());
        assert_eq!(
            dircolors.files[0].relative_path,
            "dircolors/theme.dircolors"
        );

        let yazi = export_yazi_format(&theme, &roles, &palette, warnings);
        assert_eq!(yazi.files[0].relative_path, "yazi/theme.toml");
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

    #[test]
    fn exports_yazi_golden_minimal() {
        let (theme, roles, palette, mut warnings) = minimal_pipeline();
        warnings.extend(theme.warnings.clone());
        let (yazi, _) = export_yazi(&theme, &roles, &palette, warnings);
        assert_eq!(
            yazi,
            include_str!("../../../tests/golden/yazi/minimal.toml")
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
