use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use exporters::{
    export_base16_yaml, export_bat_tmtheme, export_gitui, export_kitty, render_report,
};
use helix_theme::resolve_file;
use palette16::extract_base16;
use semantic_roles::derive_roles;

#[derive(Debug, Parser)]
#[command(name = "themeforge")]
#[command(about = "Convert Helix themes through a semantic intermediate representation")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Parse, resolve, and print a Helix theme as JSON.
    Inspect {
        /// Path to the Helix theme TOML file to inspect.
        theme: Utf8PathBuf,
        /// Directory used to resolve inherited Helix themes; may be repeated.
        #[arg(long = "theme-dir")]
        theme_dirs: Vec<Utf8PathBuf>,
        /// Pretty-print JSON output.
        #[arg(long)]
        pretty: bool,
    },
    /// Resolve inheritance and palette references, then print the resolved theme.
    Resolve {
        /// Path to the Helix theme TOML file to resolve.
        theme: Utf8PathBuf,
        /// Directory used to resolve inherited Helix themes; may be repeated.
        #[arg(long = "theme-dir")]
        theme_dirs: Vec<Utf8PathBuf>,
        /// Pretty-print JSON output.
        #[arg(long)]
        pretty: bool,
    },
    /// Export a Helix theme through semantic roles into all supported formats.
    Export {
        /// Path to the Helix theme TOML file to export.
        theme: Utf8PathBuf,
        /// Directory used to resolve inherited Helix themes; may be repeated.
        #[arg(long = "theme-dir")]
        theme_dirs: Vec<Utf8PathBuf>,
        /// Directory to receive generated output directories.
        #[arg(long = "out-dir")]
        out_dir: Utf8PathBuf,
        /// Treat parser, resolver, role, and palette warnings as errors.
        #[arg(long)]
        strict: bool,
        /// Print a human-readable export report to stderr.
        #[arg(long)]
        report: bool,
        /// Write a machine-readable export report as JSON.
        #[arg(long)]
        report_json: Option<Utf8PathBuf>,
        /// Run parsing, resolution, role derivation, and reporting without writing export output.
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Inspect {
            theme,
            theme_dirs,
            pretty,
        } => {
            let theme_dirs = theme_dirs_or_parent(&theme, theme_dirs);
            let resolved = resolve_file(&theme, &theme_dirs)?;
            print_json(&resolved, pretty)?;
        }
        Command::Resolve {
            theme,
            theme_dirs,
            pretty,
        } => {
            let theme_dirs = theme_dirs_or_parent(&theme, theme_dirs);
            let resolved = resolve_file(&theme, &theme_dirs)?;
            print_json(&resolved, pretty)?;
        }
        Command::Export {
            theme,
            theme_dirs,
            out_dir,
            strict,
            report,
            report_json,
            dry_run,
        } => {
            let theme_dirs = theme_dirs_or_parent(&theme, theme_dirs);
            let resolved = resolve_file(&theme, &theme_dirs)?;
            let (roles, role_warnings) = derive_roles(&resolved);
            let (palette, palette_warnings) = extract_base16(&roles);
            let mut warnings = resolved.warnings.clone();
            warnings.extend(role_warnings);
            warnings.extend(palette_warnings);
            if strict && !warnings.is_empty() {
                anyhow::bail!("strict mode failed with {} warning(s)", warnings.len());
            }

            let theme_name = file_stem(&resolved.name);
            let (kitty, kitty_report) = export_kitty(&resolved, &roles, &palette, warnings.clone());
            let base16 = export_base16_yaml(&palette)?;
            let base16_report = exporters::ExportReport {
                exporter: "base16".to_owned(),
                source: resolved.source_path.to_string(),
                preserved: Vec::new(),
                dropped: Vec::new(),
                warnings: warnings.clone(),
            };
            let (bat, bat_report) =
                export_bat_tmtheme(&resolved, &roles, &palette, warnings.clone());
            let gitui = export_gitui(&resolved, &roles, &palette, warnings);
            let generated = GeneratedExports {
                files: vec![
                    GeneratedFile {
                        relative_path: Utf8PathBuf::from(format!("kitty/{theme_name}.conf")),
                        contents: kitty,
                    },
                    GeneratedFile {
                        relative_path: Utf8PathBuf::from(format!("base16/{theme_name}.yaml")),
                        contents: base16,
                    },
                    GeneratedFile {
                        relative_path: Utf8PathBuf::from(format!("bat/{theme_name}.tmTheme")),
                        contents: bat,
                    },
                    GeneratedFile {
                        relative_path: Utf8PathBuf::from("gitui/theme.ron"),
                        contents: gitui.theme_ron,
                    },
                    GeneratedFile {
                        relative_path: Utf8PathBuf::from(format!(
                            "gitui/{}",
                            gitui.syntax_file_name
                        )),
                        contents: gitui.syntax_tmtheme,
                    },
                ],
                reports: vec![kitty_report, base16_report, bat_report, gitui.report],
            };

            if let Some(path) = report_json {
                let json = serde_json::to_string_pretty(&generated.reports)?;
                std::fs::write(&path, json)
                    .with_context(|| format!("failed to write report JSON to {path}"))?;
            }
            if report {
                for export_report in &generated.reports {
                    eprintln!("{}", render_report(export_report));
                }
            }
            if !dry_run {
                for export in generated.files {
                    let path = out_dir.join(&export.relative_path);
                    if let Some(parent) = path.parent() {
                        std::fs::create_dir_all(parent)
                            .with_context(|| format!("failed to create {parent}"))?;
                    }
                    std::fs::write(&path, export.contents)
                        .with_context(|| format!("failed to write {path}"))?;
                }
            }
        }
    }
    Ok(())
}

struct GeneratedExports {
    files: Vec<GeneratedFile>,
    reports: Vec<exporters::ExportReport>,
}

struct GeneratedFile {
    relative_path: Utf8PathBuf,
    contents: String,
}

fn file_stem(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect();
    if sanitized.is_empty() {
        "theme".to_owned()
    } else {
        sanitized
    }
}

fn theme_dirs_or_parent(theme: &Utf8PathBuf, theme_dirs: Vec<Utf8PathBuf>) -> Vec<Utf8PathBuf> {
    if theme_dirs.is_empty() {
        vec![theme
            .parent()
            .map(Utf8PathBuf::from)
            .unwrap_or_else(|| Utf8PathBuf::from("."))]
    } else {
        theme_dirs
    }
}

fn print_json<T: serde::Serialize>(value: &T, pretty: bool) -> Result<()> {
    if pretty {
        println!("{}", serde_json::to_string_pretty(value)?);
    } else {
        println!("{}", serde_json::to_string(value)?);
    }
    Ok(())
}
