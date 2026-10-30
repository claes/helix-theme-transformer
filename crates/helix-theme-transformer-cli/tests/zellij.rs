use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

struct TempDir(PathBuf);
impl TempDir {
    fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("htt-zellij-{}-{nanos}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn cli_exports_zellij_and_reports_and_respects_dry_run() {
    let temp = TempDir::new();
    // File stems containing spaces are sanitized for paths, but retained as theme names.
    let source = temp.0.join("my theme.toml");
    fs::write(
        &source,
        include_str!("../../../tests/fixtures/minimal.toml"),
    )
    .unwrap();
    let report = temp.0.join("report.json");
    let output_dir = temp.0.join("generated");
    let run = |dry: bool| {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_htt"));
        cmd.arg("export")
            .arg(&source)
            .arg("--out-dir")
            .arg(&output_dir)
            .arg("--report-json")
            .arg(&report)
            .arg("--report");
        if dry {
            cmd.arg("--dry-run");
        }
        let result = cmd.output().unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(result.stdout.is_empty());
        assert!(String::from_utf8_lossy(&result.stderr).contains("Export report: zellij"));
        let reports: serde_json::Value =
            serde_json::from_slice(&fs::read(&report).unwrap()).unwrap();
        assert_eq!(reports.as_array().unwrap().len(), 8);
        assert!(reports
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["exporter"] == "zellij"));
    };
    run(true);
    assert!(!output_dir.exists());
    run(false);
    let kdl = fs::read_to_string(output_dir.join("my-theme/zellij/theme.kdl")).unwrap();
    assert!(kdl.contains("// Activate with: theme \"my theme\""));
    assert!(kdl.contains("    \"my theme\" {"));
    for path in ["kitty/theme.conf", "yazi/theme.toml", "base16/theme.yaml"] {
        assert!(output_dir.join("my-theme").join(path).exists());
    }
}

#[test]
fn cli_strict_still_rejects_upstream_warnings_before_writing() {
    let temp = TempDir::new();
    let source = temp.0.join("empty.toml");
    fs::write(&source, "").unwrap();
    let output_dir = temp.0.join("generated");
    let result = Command::new(env!("CARGO_BIN_EXE_htt"))
        .arg("export")
        .arg(source)
        .arg("--out-dir")
        .arg(&output_dir)
        .arg("--strict")
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("strict mode failed"));
    assert!(!output_dir.exists());
}

#[cfg(unix)]
#[test]
fn release_manifest_and_transposition_include_only_existing_zellij_themes() {
    // The release helper already requires jq; skip this integration check when unavailable.
    if Command::new("jq").arg("--version").output().is_err() {
        return;
    }
    let temp = TempDir::new();
    let theme_dir = temp.0.join("generated-themes/one/zellij");
    fs::create_dir_all(&theme_dir).unwrap();
    fs::write(theme_dir.join("theme.kdl"), "themes { one {} }\n").unwrap();
    fs::create_dir_all(temp.0.join("generated-themes/two/kitty")).unwrap();
    fs::write(temp.0.join("generated-themes/two/kitty/theme.conf"), "").unwrap();
    for script in [
        "create-release-manifest.sh",
        "transpose-generated-themes.sh",
    ] {
        let result = Command::new("bash")
            .arg(root().join("scripts").join(script))
            .current_dir(&temp.0)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(temp.0.join("generated-themes/manifest.json")).unwrap())
            .unwrap();
    assert_eq!(
        manifest["themes"]["one"]["zellij"]["theme"],
        "one/zellij/theme.kdl"
    );
    assert!(manifest["themes"]["two"].get("zellij").is_none());
    assert_eq!(
        fs::read(temp.0.join("generated-themes-by-app/zellij/one.kdl")).unwrap(),
        fs::read(theme_dir.join("theme.kdl")).unwrap()
    );
    assert!(!temp
        .0
        .join("generated-themes-by-app/zellij/two.kdl")
        .exists());
}
