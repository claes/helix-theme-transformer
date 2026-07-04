# Helix Theme Transformer

`htt` transforms Helix theme TOML files into theme files for other tools.

Supported exports:

- Kitty
- Base16 YAML
- Bat
- GitUI
- Midnight Commander
- dircolors

## Usage

Export one Helix theme into all supported formats:

```bash
htt export path/to/theme.toml --out-dir generated-themes
```

The output directory contains one directory named after the source theme file.
Each supported format is written into its own subdirectory.

