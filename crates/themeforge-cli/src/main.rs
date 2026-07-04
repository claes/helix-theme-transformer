use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use clap::{Parser, Subcommand, ValueEnum};
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
    /// Export a Helix theme through semantic roles into a target format.
    Export {
        /// Export format to generate.
        target: Target,
        /// Path to the Helix theme TOML file to export.
        theme: Utf8PathBuf,
        /// Directory used to resolve inherited Helix themes; may be repeated.
        #[arg(long = "theme-dir")]
        theme_dirs: Vec<Utf8PathBuf>,
        /// File path to write the exported theme; stdout is used when omitted.
        #[arg(long)]
        out: Option<Utf8PathBuf>,
        /// Directory to receive generated output directories for multi-file exporters.
        #[arg(long = "out-dir")]
        out_dir: Option<Utf8PathBuf>,
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

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Target {
    /// Generate a Kitty terminal .conf theme.
    Kitty,
    /// Generate a Base16-like YAML palette.
    Base16,
    /// Generate a bat-compatible Sublime .tmTheme file.
    Bat,
    /// Generate a gitui theme directory containing theme.ron and a .tmTheme file.
    Gitui,
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
            target,
            theme,
            theme_dirs,
            out,
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
            if !matches!(target, Target::Gitui) && out_dir.is_some() {
                anyhow::bail!("--out-dir is only supported for gitui export");
            }

            let export = match target {
                Target::Kitty => {
                    ExportOutput::SingleFile(export_kitty(&resolved, &roles, &palette, warnings))
                }
                Target::Bat => ExportOutput::SingleFile(export_bat_tmtheme(
                    &resolved, &roles, &palette, warnings,
                )),
                Target::Base16 => {
                    let output = export_base16_yaml(&palette)?;
                    let report = exporters::ExportReport {
                        exporter: "base16".to_owned(),
                        source: resolved.source_path.to_string(),
                        preserved: Vec::new(),
                        dropped: Vec::new(),
                        warnings,
                    };
                    ExportOutput::SingleFile((output, report))
                }
                Target::Gitui => {
                    if out.is_some() {
                        anyhow::bail!("gitui export uses --out-dir instead of --out");
                    }
                    let out_dir = out_dir
                        .as_ref()
                        .context("gitui export requires --out-dir")?;
                    ExportOutput::Gitui {
                        parent_dir: out_dir.clone(),
                        theme: export_gitui(&resolved, &roles, &palette, warnings),
                    }
                }
            };
            let export_report = export.report();

            if let Some(path) = report_json {
                let json = serde_json::to_string_pretty(export_report)?;
                std::fs::write(&path, json)
                    .with_context(|| format!("failed to write report JSON to {path}"))?;
            }
            if report {
                eprintln!("{}", render_report(export_report));
            }
            if !dry_run {
                match export {
                    ExportOutput::SingleFile((output, _)) => {
                        if let Some(path) = out {
                            std::fs::write(&path, output)
                                .with_context(|| format!("failed to write export to {path}"))?;
                        } else {
                            print!("{output}");
                        }
                    }
                    ExportOutput::Gitui { parent_dir, theme } => {
                        let gitui_dir = parent_dir.join("gitui");
                        std::fs::create_dir_all(&gitui_dir)
                            .with_context(|| format!("failed to create {gitui_dir}"))?;
                        let theme_path = gitui_dir.join("theme.ron");
                        std::fs::write(&theme_path, theme.theme_ron)
                            .with_context(|| format!("failed to write {theme_path}"))?;
                        let syntax_path = gitui_dir.join(theme.syntax_file_name);
                        std::fs::write(&syntax_path, theme.syntax_tmtheme)
                            .with_context(|| format!("failed to write {syntax_path}"))?;
                    }
                }
            }
        }
    }
    Ok(())
}

enum ExportOutput {
    SingleFile((String, exporters::ExportReport)),
    Gitui {
        parent_dir: Utf8PathBuf,
        theme: exporters::GituiTheme,
    },
}

impl ExportOutput {
    fn report(&self) -> &exporters::ExportReport {
        match self {
            ExportOutput::SingleFile((_, report)) => report,
            ExportOutput::Gitui { theme, .. } => &theme.report,
        }
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
