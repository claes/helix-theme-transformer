# Helix Theme Transformer

`htt` transforms Helix theme TOML files into theme files for other tools.

Supported exports:

- Kitty
- Base16 YAML
- Bat
- GitUI
- Midnight Commander
- dircolors
- Yazi
- Zellij (component-based KDL themes)

Since Helix themes are targetting language syntax features rather than specific
base colors, when converting to a theme format that is centered on colors,
those colors are generated heuristically. This means that applications may not
always be colored accurately relative the original theme. On the other hand, it will
be easier to maintain consistent theming between applications. 
 
## Usage

Export one Helix theme into all supported formats:

```bash
htt export path/to/helix/theme.toml --out-dir generated-themes-directory
```

The output directory will contain one directory named after the source theme file.
Each supported format is written into its own subdirectory with stable filenames
such as `kitty/theme.conf` and `bat/theme.tmTheme`.

## Zellij

Copy `generated-themes/<theme>/zellij/theme.kdl` into your Zellij configuration's
`themes/` directory with a unique filename, such as `<theme>.kdl`. Set
`theme "<theme>"` in `config.kdl`, using the activation name in the generated
file's comment. Zellij discovers themes by their internal name.

The exporter uses the current [component-based theme format](https://zellij.dev/documentation/themes.html),
with separate styles for tabs, text, lists, tables, frames, and exit codes.
It preserves semantic UI color pairs, falls back to semantic roles and the
derived palette, and reports unsupported styling and discarded alpha.
It does not change colors to correct contrast.

Preview a generated theme in an isolated, disposable Zellij session:

```bash
./scripts/tool-test-scripts/zellij.sh adwaita-dark
For switching themes through symlinks, link `themes/current.kdl` to the desired
`generated-themes/<theme>/zellij/theme-static.kdl` and set `theme "static"` in
`config.kdl`. Change the link to select another theme without changing the
configured name. The static variant differs only in its internal theme node name;
all comments retain the real theme name, including the named variant's activation
example. Use `theme "static"` for this variant regardless of that comment.

```

Run this inside `nix develop` (or prefix the command with `nix develop -c`).
If the generated tree predates Zellij support, regenerate it with
`./scripts/generate-from-helix-themes.sh` first. The preview shows navigation
instructions, active/inactive tabs and frames, and successful/failed exit codes.
Press `Ctrl+q` to quit.

Release manifests expose `themes.<theme>.zellij.theme` and
`themes.<theme>.zellij.theme-static` (also available through the generated Nix
attributes). The transposition helper writes `zellij/<theme>.kdl` and
`zellij/<theme>-static.kdl`.

For the static variant with Home Manager, use:

```nix
xdg.configFile."zellij/themes/current.kdl".source =
  httThemes.themes."adwaita-dark".zellij.theme-static;
```

Set `theme "static"` in `config.kdl` for this configuration.

# Contributing

This program has been created with help of OpenAI codex. Pull requests for export to
other tools are welcome. Make sure to update Specification.md as well as test cases.

# License

This program is distributed under the same license as Helix editor, MPL-2.0.  
