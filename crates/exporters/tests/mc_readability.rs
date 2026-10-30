use camino::{Utf8Path, Utf8PathBuf};
use exporters::export_mc;
use helix_theme::{parse_str, resolve_file, resolve_raw};
use palette16::extract_base16;
use semantic_roles::{derive_roles, derive_ui_styles};
use theme_ir::{luminance, ResolvedTheme};

const THEMES: &[&str] = &[
    "zed_onelight",
    "adwaita-dark",
    "adwaita-light",
    "material_deep_ocean",
    "material_darker",
    "material_oceanic",
    "material_palenight",
    "catppuccin_mocha",
    "catppuccin_macchiato",
    "catppuccin_frappe",
    "catppuccin_latte",
];

fn export(theme: &ResolvedTheme) -> exporters::McExport {
    let (roles, _) = derive_roles(theme);
    let (palette, _) = extract_base16(&roles);
    export_mc(theme, &roles, &palette, &derive_ui_styles(theme), vec![])
}

fn contrast(a: &str, b: &str) -> f32 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

fn skin_value<'a>(skin: &'a str, section: &str, key: &str) -> &'a str {
    let section = skin.split(&format!("[{section}]\n")).nth(1).unwrap();
    section
        .lines()
        .take_while(|line| !line.starts_with('['))
        .find_map(|line| line.trim().strip_prefix(&format!("{key} = ")))
        .unwrap()
}

fn pair(value: &str, separator: char) -> (&str, &str) {
    let mut parts = value.split(separator);
    (parts.next().unwrap(), parts.next().unwrap())
}

fn decode(token: &str) -> String {
    if let Some(gray) = token.strip_prefix("gray") {
        let value = 8 + gray.parse::<u16>().unwrap() * 10;
        return format!("#{value:02x}{value:02x}{value:02x}");
    }
    let cube = [0, 95, 135, 175, 215, 255];
    let values: Vec<_> = token
        .strip_prefix("rgb")
        .unwrap()
        .chars()
        .map(|c| cube[c.to_digit(10).unwrap() as usize])
        .collect();
    format!("#{:02x}{:02x}{:02x}", values[0], values[1], values[2])
}

fn check_skin(skin: &str) {
    for (section, keys) in [
        (
            "menu",
            vec![
                "_default_",
                "menusel",
                "menuhot",
                "menuhotsel",
                "menuinactive",
            ],
        ),
        ("popupmenu", vec!["_default_", "menusel", "menutitle"]),
        (
            "dialog",
            vec!["_default_", "dfocus", "dhotnormal", "dhotfocus", "dtitle"],
        ),
    ] {
        for key in keys {
            let (fg, bg) = pair(skin_value(skin, section, key), ';');
            let minimum = if key == "menuinactive" { 3.0 } else { 4.5 };
            assert!(contrast(fg, bg) >= minimum, "{section}.{key}: {fg}/{bg}");
        }
        let selected = if section == "dialog" {
            "dfocus"
        } else {
            "menusel"
        };
        let (_, normal) = pair(skin_value(skin, section, "_default_"), ';');
        let (_, focused) = pair(skin_value(skin, section, selected), ';');
        assert!(
            contrast(normal, focused) >= 1.5,
            "{section} selection visibility"
        );
    }
    for (section, key) in [
        ("menu", "menuhot"),
        ("menu", "menuhotsel"),
        ("dialog", "dhotnormal"),
        ("dialog", "dhotfocus"),
    ] {
        assert!(skin_value(skin, section, key).contains("underline"));
    }
}

fn check_table(table: &str) {
    let entries: std::collections::BTreeMap<_, _> = table
        .split('\'')
        .nth(1)
        .unwrap()
        .split(':')
        .map(|entry| entry.split_once('=').unwrap())
        .collect();
    for key in [
        "menunormal",
        "menusel",
        "menuhot",
        "menuhotsel",
        "menuinactive",
        "pmenunormal",
        "pmenusel",
        "pmenutitle",
        "dnormal",
        "dfocus",
        "dhotnormal",
        "dhotfocus",
        "dtitle",
    ] {
        let (fg, bg) = pair(entries[key], ',');
        let minimum = if key == "menuinactive" { 3.0 } else { 4.5 };
        assert!(
            contrast(&decode(fg), &decode(bg)) >= minimum,
            "quantized {key}"
        );
    }
    for (normal, selected) in [
        ("menunormal", "menusel"),
        ("pmenunormal", "pmenusel"),
        ("dnormal", "dfocus"),
    ] {
        let (_, a) = pair(entries[normal], ',');
        let (_, b) = pair(entries[selected], ',');
        assert!(
            contrast(&decode(a), &decode(b)) >= 1.5,
            "quantized {selected} visibility"
        );
    }
}

#[test]
fn theme_families_remain_readable_in_both_outputs() {
    let root = Utf8Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/mc");
    for name in THEMES {
        let theme = resolve_file(
            &root.join(format!("{name}.toml")),
            std::slice::from_ref(&root),
        )
        .unwrap();
        let ui = derive_ui_styles(&theme);
        let output = export(&theme);
        check_skin(&output.skin_ini);
        check_table(&output.colortable_env);
        assert_eq!(output, export(&theme), "deterministic {name}");
        if name.starts_with("catppuccin") {
            assert!(skin_value(&output.skin_ini, "menu", "menusel").contains("bold"));
        }
        if *name == "zed_onelight" {
            assert_eq!(
                ui.selected_menu.bg.as_ref().unwrap().color.as_deref(),
                Some("#5b79e3")
            );
        }
        if *name == "adwaita-light" {
            assert_eq!(
                ui.menu.bg.as_ref().unwrap().color.as_deref(),
                Some("#f6f5f4")
            );
        }
        if name.starts_with("material") || name.starts_with("adwaita") {
            let (fg, bg) = pair(skin_value(&output.skin_ini, "menu", "menusel"), ';');
            assert_eq!(
                Some(fg),
                ui.selected_menu.fg.as_ref().unwrap().color.as_deref()
            );
            assert_eq!(
                Some(bg),
                ui.selected_menu.bg.as_ref().unwrap().color.as_deref()
            );
        }
        let golden = root
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join(format!("golden/mc/{name}"));
        assert_eq!(
            output.skin_ini,
            std::fs::read_to_string(golden.join("theme.ini")).unwrap(),
            "{name} skin"
        );
        assert_eq!(
            output.colortable_env,
            std::fs::read_to_string(golden.join("colortable.env")).unwrap(),
            "{name} table"
        );
    }
}

#[test]
fn fallbacks_and_corrections_do_not_depend_on_theme_names() {
    let source = r##"
"ui.background" = { bg = "#eeeeee" }
"ui.text" = "#dddddd"
"ui.menu" = { fg = "#222222", bg = "#eeeeee" }
"ui.menu.selected" = { bg = "#ededed", modifiers = ["bold"] }
"ui.text.inactive" = "#eeeeee"
"diagnostic.warning" = "#ffff00"
"comment" = "#111111"
"##;
    let mut theme =
        resolve_raw(parse_str("arbitrary", Utf8PathBuf::from("arbitrary.toml"), source).unwrap());
    let first = export(&theme);
    check_skin(&first.skin_ini);
    check_table(&first.colortable_env);
    assert!(first
        .report
        .warnings
        .iter()
        .any(|w| w.code == "mc_ui_color_adjusted" && w.message.contains("menu.menusel")));
    assert!(!first
        .report
        .preserved
        .iter()
        .any(|p| p.target == "menu.menusel.bg"));
    assert!(first
        .report
        .preserved
        .iter()
        .any(|p| p.target == "menu.menusel.fg" && p.source == "ui.menu.fg"));
    theme.name = "catppuccin_latte".into();
    let second = export(&theme);
    assert_eq!(
        first.skin_ini.lines().skip(2).collect::<Vec<_>>(),
        second.skin_ini.lines().skip(2).collect::<Vec<_>>()
    );
    assert_eq!(first.colortable_env, second.colortable_env);
    theme.scopes.clear();
    check_skin(&export(&theme).skin_ini);
    check_table(&export(&theme).colortable_env);
}
