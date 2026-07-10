use crate::file_kinds::{file_kind_style, resolve_file_kind_color, FileEmphasis, FileKind};
use crate::report::{base16_preserved_items, dropped_items, ExportReport};
use palette16::{color, Base16Palette};
use semantic_roles::{role_color, Role, SemanticRoles};
use theme_ir::{brighten, parse_rgb, ResolvedTheme, Warning};

pub fn export_preview_script(
    theme: &ResolvedTheme,
    roles: &SemanticRoles,
    palette: &Base16Palette,
    warnings: Vec<Warning>,
) -> (String, ExportReport) {
    let mut output = String::new();
    output.push_str("#!/usr/bin/env bash\n");
    output.push_str("set -euo pipefail\n\n");
    output.push_str("script_dir=\"$(cd \"$(dirname \"${BASH_SOURCE[0]}\")\" && pwd)\"\n\n");
    output.push_str("rgb_swatch() {\n");
    output.push_str("  local name=\"$1\" base=\"$2\" hex=\"$3\" sgr=\"$4\"\n");
    output.push_str(
        "  printf '  %-7s %-7s %-9s \\033[%sm%-18s\\033[0m %s\\n' \"$name\" \"$base\" \"$hex\" \"$sgr\" \"████ text\" \"$hex\"\n",
    );
    output.push_str("}\n\n");
    output.push_str("ansi_swatch() {\n");
    output.push_str("  local code=\"$1\" name=\"$2\"\n");
    output.push_str(
        "  printf '  %-3s %-16s \\033[%sm%-18s\\033[0m\\n' \"$code\" \"$name\" \"$code\" \"████ text\"\n",
    );
    output.push_str("}\n\n");
    output.push_str("sample_line() {\n");
    output.push_str("  local label=\"$1\" sgr=\"$2\"\n");
    output.push_str("  printf '  %-14s \\033[%sm%s\\033[0m\\n' \"$label\" \"$sgr\" \"$label\"\n");
    output.push_str("}\n\n");
    output.push_str(&format!("echo 'Theme: {}'\n", sh_single_quote(&theme.name)));
    output.push_str("echo\n");
    output.push_str("echo 'Generated RGB palette'\n");
    for swatch in palette_swatches(palette) {
        output.push_str(&format!(
            "rgb_swatch '{}' '{}' '{}' '{}'\n",
            swatch.name,
            swatch.base,
            swatch.color,
            foreground_sgr(&swatch.color)
        ));
    }
    output.push_str("echo\n");
    output.push_str("echo 'Terminal ANSI palette'\n");
    for (code, name) in ansi_swatches() {
        output.push_str(&format!("ansi_swatch '{code}' '{name}'\n"));
    }
    output.push_str("echo\n");
    output.push_str("echo 'Foreground/background samples'\n");
    for sample in semantic_samples(roles, palette) {
        output.push_str(&format!(
            "sample_line '{}' '{}'\n",
            sample.label, sample.sgr
        ));
    }
    output.push_str("echo\n");
    output.push_str("echo 'Current directory with generated dircolors'\n");
    output.push_str("if command -v dircolors >/dev/null 2>&1 && command -v ls >/dev/null 2>&1 && [[ -f \"$script_dir/dircolors/theme.dircolors\" ]]; then\n");
    output.push_str("  eval \"$(dircolors \"$script_dir/dircolors/theme.dircolors\")\"\n");
    output.push_str("  export LS_COLORS\n");
    output.push_str("  ls --color=always -la\n");
    output.push_str("else\n");
    output.push_str(
        "  echo '  Skipped: dircolors, ls, or dircolors/theme.dircolors is unavailable.'\n",
    );
    output.push_str("fi\n");

    let report = ExportReport {
        exporter: "preview".to_owned(),
        source: theme.source_path.to_string(),
        preserved: base16_preserved_items(roles, palette),
        dropped: dropped_items(theme),
        warnings,
    };
    (output, report)
}

struct Swatch {
    name: &'static str,
    base: &'static str,
    color: String,
}

struct Sample {
    label: &'static str,
    sgr: String,
}

fn palette_swatches(palette: &Base16Palette) -> Vec<Swatch> {
    vec![
        swatch("color0", "base00", color(palette, "base00").to_owned()),
        swatch("color1", "base08", color(palette, "base08").to_owned()),
        swatch("color2", "base0B", color(palette, "base0B").to_owned()),
        swatch("color3", "base0A", color(palette, "base0A").to_owned()),
        swatch("color4", "base0D", color(palette, "base0D").to_owned()),
        swatch("color5", "base0E", color(palette, "base0E").to_owned()),
        swatch("color6", "base0C", color(palette, "base0C").to_owned()),
        swatch("color7", "base05", color(palette, "base05").to_owned()),
        swatch("color8", "base03", color(palette, "base03").to_owned()),
        swatch("color9", "bright08", brighten(color(palette, "base08"))),
        swatch("color10", "bright0B", brighten(color(palette, "base0B"))),
        swatch("color11", "bright0A", brighten(color(palette, "base0A"))),
        swatch("color12", "bright0D", brighten(color(palette, "base0D"))),
        swatch("color13", "bright0E", brighten(color(palette, "base0E"))),
        swatch("color14", "bright0C", brighten(color(palette, "base0C"))),
        swatch("color15", "base07", color(palette, "base07").to_owned()),
    ]
}

fn swatch(name: &'static str, base: &'static str, color: String) -> Swatch {
    Swatch { name, base, color }
}

fn ansi_swatches() -> &'static [(&'static str, &'static str)] {
    &[
        ("30", "black"),
        ("31", "red"),
        ("32", "green"),
        ("33", "yellow"),
        ("34", "blue"),
        ("35", "magenta"),
        ("36", "cyan"),
        ("37", "white"),
        ("90", "bright black"),
        ("91", "bright red"),
        ("92", "bright green"),
        ("93", "bright yellow"),
        ("94", "bright blue"),
        ("95", "bright magenta"),
        ("96", "bright cyan"),
        ("97", "bright white"),
    ]
}

fn semantic_samples(roles: &SemanticRoles, palette: &Base16Palette) -> Vec<Sample> {
    vec![
        sample(
            "normal text",
            style_sgr(
                &[],
                role_or_base(roles, palette, Role::Foreground, "base05"),
                None,
            ),
        ),
        sample(
            "selected text",
            style_sgr(
                &[],
                role_or_base(roles, palette, Role::Foreground, "base05"),
                Some(role_or_base(roles, palette, Role::Selection, "base02")),
            ),
        ),
        sample(
            "muted text",
            style_sgr(
                &[],
                role_or_base(roles, palette, Role::MutedForeground, "base03"),
                None,
            ),
        ),
        sample(
            "warning",
            style_sgr(
                &["01"],
                role_or_base(roles, palette, Role::Warning, "base0A"),
                None,
            ),
        ),
        sample(
            "error",
            style_sgr(
                &["01"],
                role_or_base(roles, palette, Role::Error, "base08"),
                None,
            ),
        ),
        file_sample("directory", FileKind::Directory, roles, palette),
        file_sample("executable", FileKind::Executable, roles, palette),
        file_sample("symlink", FileKind::Symlink, roles, palette),
    ]
}

fn sample(label: &'static str, sgr: String) -> Sample {
    Sample { label, sgr }
}

fn file_sample(
    label: &'static str,
    kind: FileKind,
    roles: &SemanticRoles,
    palette: &Base16Palette,
) -> Sample {
    let kind_style = file_kind_style(kind);
    let attrs = match kind_style.emphasis {
        FileEmphasis::Bold | FileEmphasis::Dangerous => &["01"][..],
        FileEmphasis::Normal | FileEmphasis::Muted | FileEmphasis::Background => &[][..],
    };
    sample(
        label,
        style_sgr(attrs, resolve_file_kind_color(kind, roles, palette), None),
    )
}

fn role_or_base<'a>(
    roles: &'a SemanticRoles,
    palette: &'a Base16Palette,
    role: Role,
    fallback_base: &str,
) -> &'a str {
    role_color(roles, role).unwrap_or_else(|| color(palette, fallback_base))
}

fn style_sgr(attrs: &[&str], fg: &str, bg: Option<&str>) -> String {
    let mut parts = attrs
        .iter()
        .map(|attr| (*attr).to_owned())
        .collect::<Vec<_>>();
    parts.push(foreground_sgr(fg));
    if let Some(bg) = bg {
        parts.push(background_sgr(bg));
    }
    parts.join(";")
}

fn foreground_sgr(color: &str) -> String {
    truecolor_sgr("38", color)
}

fn background_sgr(color: &str) -> String {
    truecolor_sgr("48", color)
}

fn truecolor_sgr(prefix: &str, color: &str) -> String {
    let rgb = parse_rgb(color).expect("resolved exporter colors should be valid rgb hex colors");
    format!("{prefix};2;{};{};{}", rgb.r, rgb.g, rgb.b)
}

fn sh_single_quote(value: &str) -> String {
    value.replace('\'', "'\\''")
}
