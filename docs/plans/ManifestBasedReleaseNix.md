# Manifest-Based Release Nix Plan

Status: implemented.

This plan changes release generation so `generated-themes.zip` contains
`generated-themes/manifest.json`, while `generated-themes.nix` contains only
stable logic that reads the manifest and maps relative paths to files inside
the fetched archive.

## Goal

Keep the public Nix consumer interface stable while simplifying generated Nix.

Consumers should still use attributes like:

```nix
themes."adwaita-dark".kitty.theme
themes."adwaita-dark".mc.colortable
```

## Release Layout

Release assets stay:

```text
generated-themes.zip
generated-themes.nix
```

The zip should contain:

```text
generated-themes/
  manifest.json
  CREDITS
  <theme-file-name>/
    kitty/<resolved-theme-name>.conf
    base16/<resolved-theme-name>.yaml
    bat/<resolved-theme-name>.tmTheme
    gitui/theme.ron
    gitui/<resolved-theme-name>.tmTheme
    mc/<resolved-theme-name>.ini
    mc/filehighlight.ini
    mc/colortable.env
    dircolors/<resolved-theme-name>.dircolors
    helix/<theme-file-name>.toml
```

## Manifest Shape

`manifest.json` stores relative paths only:

```json
{
  "themes": {
    "adwaita-dark": {
      "kitty": {
        "theme": "adwaita-dark/kitty/adwaita-dark.conf"
      },
      "base16": {
        "theme": "adwaita-dark/base16/adwaita-dark.yaml"
      },
      "bat": {
        "theme": "adwaita-dark/bat/adwaita-dark.tmTheme"
      },
      "gitui": {
        "theme": "adwaita-dark/gitui/theme.ron",
        "syntax": "adwaita-dark/gitui/adwaita-dark.tmTheme"
      },
      "mc": {
        "theme": "adwaita-dark/mc/adwaita-dark.ini",
        "filehighlight": "adwaita-dark/mc/filehighlight.ini",
        "colortable": "adwaita-dark/mc/colortable.env"
      },
      "dircolors": {
        "theme": "adwaita-dark/dircolors/adwaita-dark.dircolors"
      },
      "helix": {
        "theme": "adwaita-dark/helix/adwaita-dark.toml"
      }
    }
  }
}
```

## Implementation Steps

1. Update `docs/Specification.md` to describe the implemented manifest-based
   release layout.
2. Add or extend a bash script to generate
   `generated-themes/manifest.json`.
3. Make the manifest generator scan `generated-themes/<theme>/...` and include
   only files that exist.
4. Update `Makefile` dependencies:

   ```make
   generated-themes/manifest.json: generated-themes
   generated-themes.zip: generated-themes/manifest.json
   generated-themes.nix: generated-themes.zip
   ```

5. Simplify `scripts/create-release-nix.sh` so it no longer emits one Nix
   attribute per theme file.
6. Generate a mostly static `generated-themes.nix` that fetches the zip, reads
   `manifest.json`, and maps manifest paths to files inside the fetched
   archive.
7. Keep the existing Nix attribute shape stable for consumers.
8. Keep the GitHub workflow publishing only `generated-themes.zip` and
   `generated-themes.nix`.

## Target Generated Nix Shape

```nix
{ pkgs }:

let
  src = pkgs.fetchzip {
    url = "https://github.com/claes/helix-theme-transformer/releases/download/generated-themes-latest/generated-themes.zip";
    hash = "sha256-...";
    stripRoot = false;
  };

  manifest =
    builtins.fromJSON (builtins.readFile "${src}/generated-themes/manifest.json");

  file = path: "${src}/generated-themes/${path}";

  mapFiles = value:
    if builtins.isAttrs value
    then builtins.mapAttrs (_: mapFiles) value
    else file value;
in
{
  inherit src manifest;

  themes = builtins.mapAttrs (_: mapFiles) manifest.themes;
}
```

## Validation

Run:

```bash
cargo fmt
cargo test --all
cargo clippy --all-targets --all-features
make clean
make release-nix
nix-instantiate --parse generated-themes.nix
nix flake check
```

Also validate:

```bash
jq . generated-themes/manifest.json
```

Optionally evaluate one known generated attribute from `generated-themes.nix`.
