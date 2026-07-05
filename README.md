# Helix Theme Transformer

`htt` transforms Helix theme TOML files into theme files for other tools.

Supported exports:

- Kitty
- Base16 YAML
- Bat
- GitUI
- Midnight Commander
- dircolors

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
Each supported format is written into its own subdirectory.

## NixOS and Home Manager

Released theme archives include a `generated-themes.nix` file for NixOS flakes.
Add it as a non-flake input:

```nix
inputs.htt-themes-nix = {
  url = "https://github.com/claes/helix-theme-transformer/releases/download/generated-themes-latest/generated-themes.nix";
  flake = false;
};
```

Then import it with `pkgs` and use the generated file attributes:

```nix
let
  httThemes = import htt-themes-nix { inherit pkgs; };
in {
  xdg.configFile."kitty/current-theme.conf".source =
    httThemes.themes."adwaita-dark".kitty.theme;
}
```

# Contributing

This program has been created with help of OpenAI codex. Pull requests for export to
other tools are welcome. Make sure to update Specification.md as well as test cases.

# License

This program is distributed under the same license as Helix editor, MPL-2.0.  
