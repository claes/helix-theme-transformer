use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use clap::{Parser, Subcommand, ValueEnum};
use exporters::{export_base16_yaml, export_bat_tmtheme, export_kitty, render_report};
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

            let (output, export_report) = match target {
                Target::Kitty => export_kitty(&resolved, &roles, &palette, warnings),
                Target::Bat => export_bat_tmtheme(&resolved, &roles, &palette, warnings),
                Target::Base16 => {
                    let output = export_base16_yaml(&palette)?;
                    let report = exporters::ExportReport {
                        exporter: "base16".to_owned(),
                        source: resolved.source_path.to_string(),
                        preserved: Vec::new(),
                        dropped: Vec::new(),
                        warnings,
                    };
                    (output, report)
                }
            };

            if let Some(path) = report_json {
                let json = serde_json::to_string_pretty(&export_report)?;
                std::fs::write(&path, json)
                    .with_context(|| format!("failed to write report JSON to {path}"))?;
            }
            if report {
                eprintln!("{}", render_report(&export_report));
            }
            if !dry_run {
                if let Some(path) = out {
                    std::fs::write(&path, output)
                        .with_context(|| format!("failed to write export to {path}"))?;
                } else {
                    print!("{output}");
                }
            }
        }
    }
    Ok(())
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
