use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use exporters::{
    export_base16_format, export_bat_format, export_dircolors_format, export_gitui_format,
    export_kitty_format, export_mc_format, export_yazi_format, render_report,
};
use helix_theme::resolve_file;
use palette16::extract_base16;
use semantic_roles::derive_roles;

#[derive(Debug, Parser)]
#[command(name = "htt")]
#[command(about = "Transform Helix themes through a semantic intermediate representation")]
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
            let theme_dir_name = theme_file_stem(&theme);
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

            let formats = vec![
                export_kitty_format(&resolved, &roles, &palette, warnings.clone()),
                export_base16_format(&resolved, &roles, &palette, warnings.clone())?,
                export_bat_format(&resolved, &roles, &palette, warnings.clone()),
                export_gitui_format(&resolved, &roles, &palette, warnings.clone()),
                export_mc_format(&resolved, &roles, &palette, warnings.clone()),
                export_dircolors_format(&resolved, &roles, &palette, warnings.clone()),
                export_yazi_format(&resolved, &roles, &palette, warnings),
            ];
            let generated = GeneratedExports::from_formats(formats);

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
                let export_root = out_dir.join(theme_dir_name);
                for export in generated.files {
                    let path = export_root.join(Utf8PathBuf::from(&export.relative_path));
                    if let Some(parent) = path.parent() {
                        std::fs::create_dir_all(parent)
                            .with_context(|| format!("failed to create {parent}"))?;
                    }
                    write_exported_file(&path, &export)?;
                }
            }
        }
    }
    Ok(())
}

fn write_exported_file(path: &Utf8PathBuf, export: &exporters::ExportedFile) -> Result<()> {
    std::fs::write(path, &export.contents).with_context(|| format!("failed to write {path}"))?;
    set_executable_if_requested(path, export.executable)?;
    Ok(())
}

#[cfg(unix)]
fn set_executable_if_requested(path: &Utf8PathBuf, executable: bool) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    if executable {
        let mut permissions = std::fs::metadata(path)
            .with_context(|| format!("failed to read permissions for {path}"))?
            .permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(path, permissions)
            .with_context(|| format!("failed to set executable permissions on {path}"))?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn set_executable_if_requested(_path: &Utf8PathBuf, _executable: bool) -> Result<()> {
    Ok(())
}

struct GeneratedExports {
    files: Vec<exporters::ExportedFile>,
    reports: Vec<exporters::ExportReport>,
}

impl GeneratedExports {
    fn from_formats(formats: Vec<exporters::ExportedFormat>) -> Self {
        let mut files = Vec::new();
        let mut reports = Vec::new();
        for format in formats {
            files.extend(format.files);
            reports.push(format.report);
        }
        Self { files, reports }
    }
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

fn theme_file_stem(theme: &Utf8PathBuf) -> String {
    theme
        .file_stem()
        .map(file_stem)
        .unwrap_or_else(|| "theme".to_owned())
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[cfg(unix)]
    #[test]
    fn write_exported_file_sets_executable_mode() {
        use std::os::unix::fs::PermissionsExt;

        let path = temp_path("htt-executable-test.sh");
        let export = exporters::ExportedFile {
            relative_path: "base16/set-terminal-colors.sh".to_owned(),
            contents: "#!/usr/bin/env bash\n".to_owned(),
            executable: true,
        };

        write_exported_file(&path, &export).unwrap();

        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o755);

        std::fs::remove_file(path).unwrap();
    }

    fn temp_path(name: &str) -> Utf8PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        Utf8PathBuf::from(format!("/tmp/{name}-{nanos}"))
    }
}
