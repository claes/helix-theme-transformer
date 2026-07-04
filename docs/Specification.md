# Specification: Helix Theme Semantic Converter

## Goal

Build a CLI tool that uses Helix editor themes as the source format and converts them into other theme formats through a semantic intermediate representation.

Initial targets:

1. Helix TOML input
2. Semantic IR
3. base16 palette extraction
4. Kitty terminal theme export
5. bat `.tmTheme` syntax highlighting theme export

The core principle is:

> Derive target colors from Helix semantic scope assignments, not from arbitrary palette color names.

## Non-goals

Do not attempt full visual equivalence across all applications.

Do not treat Helix palette names such as `red`, `blue`, or `green` as authoritative semantic roles.

Do not require Helix themes to define exactly 16 colors.

Do not mutate source themes.

---

# 1. Input Format

Input files are Helix theme TOML files.

A Helix theme may contain:

```toml
inherits = "theme_name"

[palette]
bg = "#1f2335"
fg = "#c0caf5"
red = "#f7768e"

"ui.background" = { bg = "bg" }
"ui.text" = "fg"
"keyword" = { fg = "red", modifiers = ["italic"] }
"function" = "blue"
"diagnostic.error" = { underline = { color = "red", style = "curl" } }
```

Supported style forms:

```toml
"scope" = "palette_name"
"scope" = "#rrggbb"
"scope" = { fg = "palette_name", bg = "palette_name" }
"scope" = { fg = "#rrggbb", modifiers = ["bold", "italic"] }
"scope" = { underline = { color = "red", style = "curl" } }
```

---

# 2. Theme Resolution

Before exporting, resolve the theme into a fully materialized form.

## 2.1 Inheritance

If a theme has:

```toml
inherits = "base16_default"
```

then load the inherited theme first, then overlay the child theme.

Inherited theme names are resolved by searching theme directories in priority order.

For single-theme commands, `--theme-dir` is optional and repeatable:

```bash
themeforge export kitty path/to/theme.toml --theme-dir user/themes --theme-dir builtin/themes
```

If no `--theme-dir` is provided, use the input theme file's parent directory as the only theme search directory.

Rules:

1. Parent palette entries are inherited.
2. Child palette entries override parent palette entries with the same key.
3. Parent scope styles are inherited.
4. Child scope styles override parent scope styles with the same key.
5. Exporters must only consume resolved themes.
6. Exporters must not need to understand `inherits`.

## 2.2 Palette references

All palette references must be resolved to concrete hex colors.

Example:

```toml
[palette]
red = "#ff0000"

"keyword" = "red"
```

becomes:

```json
{
  "keyword": {
    "fg": "#ff0000"
  }
}
```

If a palette reference cannot be resolved, emit a warning.

## 2.3 Color validation

Accept these forms initially:

```text
#rgb
#rrggbb
#rgba
#rrggbbaa
```

Normalize internally to lowercase `#rrggbb` where possible.

If alpha is present, preserve it internally but warn when exporting to targets that do not support alpha.

---

# 3. Internal Data Model

Implement an internal semantic model independent of TOML syntax.

Suggested TypeScript-style model:

```ts
type HexColor = string;

type UnderlineStyle =
  | "line"
  | "curl"
  | "dashed"
  | "dotted"
  | "double_line";

type Modifier =
  | "bold"
  | "italic"
  | "dim"
  | "crossed_out"
  | "reversed";

type Style = {
  fg?: HexColor;
  bg?: HexColor;
  underline?: {
    color?: HexColor;
    style?: UnderlineStyle;
  };
  modifiers?: Modifier[];
};

type ResolvedTheme = {
  name: string;
  sourcePath: string;
  palette: Record<string, HexColor>;
  scopes: Record<string, Style>;
  warnings: Warning[];
};

type Warning = {
  code: string;
  message: string;
  scope?: string;
};
```

---

# 4. Semantic Roles

Create a semantic role layer between Helix and exporters.

The semantic role layer should answer questions such as:

```text
What is the theme's background color?
What is the theme's foreground color?
What color represents keywords?
What color represents functions?
What color represents errors?
```

## 4.1 Canonical semantic roles

Implement these roles first:

```text
background
surface
selection
foreground
muted_foreground
bright_foreground
cursor

comment
keyword
function
type
variable
parameter
string
number
constant
operator
special

error
warning
info
hint

git_added
git_modified
git_removed
```

Each role should contain:

```ts
type SemanticRoleValue = {
  color?: HexColor;
  sourceScope?: string;
  sourceProperty?: "fg" | "bg" | "underline.color";
  confidence: "exact" | "fallback" | "inferred" | "missing";
};
```

## 4.2 Role derivation rule

Use Helix scope assignments as the primary source.

Do not infer semantic meaning from palette names unless all scope-based rules fail.

Example:

```toml
[palette]
banana = "#7aa2f7"

"function" = "banana"
```

The function role is `#7aa2f7`, even though the palette name is `banana`.

---

# 5. Helix Scope → Semantic Role Mapping

For each semantic role, check scopes in priority order.

Use the first usable color.

## 5.1 UI roles

```yaml
background:
  - ["ui.background", "bg"]
  - ["ui.background", "fg"]

surface:
  - ["ui.statusline", "bg"]
  - ["ui.popup", "bg"]
  - ["ui.menu", "bg"]
  - ["ui.window", "bg"]

selection:
  - ["ui.selection", "bg"]
  - ["ui.cursor.select", "bg"]

foreground:
  - ["ui.text", "fg"]
  - ["ui.text.focus", "fg"]

muted_foreground:
  - ["comment", "fg"]
  - ["ui.linenr", "fg"]
  - ["ui.text.inactive", "fg"]

bright_foreground:
  - ["ui.cursor", "fg"]
  - ["ui.text.focus", "fg"]
  - ["markup.heading", "fg"]

cursor:
  - ["ui.cursor.primary", "bg"]
  - ["ui.cursor", "bg"]
  - ["ui.cursor.primary", "fg"]
  - ["ui.cursor", "fg"]
```

## 5.2 Syntax roles

```yaml
comment:
  - ["comment", "fg"]

keyword:
  - ["keyword", "fg"]
  - ["keyword.control", "fg"]
  - ["keyword.directive", "fg"]
  - ["keyword.operator", "fg"]

function:
  - ["function", "fg"]
  - ["function.method", "fg"]
  - ["function.builtin", "fg"]
  - ["constructor", "fg"]

type:
  - ["type", "fg"]
  - ["type.builtin", "fg"]
  - ["constructor", "fg"]

variable:
  - ["variable", "fg"]
  - ["variable.other", "fg"]

parameter:
  - ["variable.parameter", "fg"]
  - ["parameter", "fg"]

string:
  - ["string", "fg"]
  - ["string.special", "fg"]

number:
  - ["constant.numeric", "fg"]
  - ["number", "fg"]

constant:
  - ["constant", "fg"]
  - ["constant.builtin", "fg"]
  - ["constant.character", "fg"]

operator:
  - ["operator", "fg"]
  - ["keyword.operator", "fg"]

special:
  - ["special", "fg"]
  - ["tag", "fg"]
  - ["attribute", "fg"]
  - ["namespace", "fg"]
```

## 5.3 Diagnostic roles

```yaml
error:
  - ["diagnostic.error", "fg"]
  - ["diagnostic.error", "underline.color"]
  - ["error", "fg"]

warning:
  - ["diagnostic.warning", "fg"]
  - ["diagnostic.warning", "underline.color"]
  - ["warning", "fg"]

info:
  - ["diagnostic.info", "fg"]
  - ["diagnostic.info", "underline.color"]
  - ["info", "fg"]

hint:
  - ["diagnostic.hint", "fg"]
  - ["diagnostic.hint", "underline.color"]
  - ["hint", "fg"]
```

## 5.4 Git roles

```yaml
git_added:
  - ["diff.plus", "fg"]
  - ["ui.statusline.insert", "fg"]

git_modified:
  - ["diff.delta", "fg"]
  - ["ui.statusline.normal", "fg"]

git_removed:
  - ["diff.minus", "fg"]
  - ["ui.statusline.select", "fg"]
```

---

# 6. Base16-like 16 Color Extraction

Create a derived 16-color palette from semantic roles.

This is not the source of truth. It is a projection.

## 6.1 Mapping

```yaml
base00: background
base01: surface
base02: selection
base03: muted_foreground
base04: muted_foreground
base05: foreground
base06: bright_foreground
base07: bright_foreground

base08: error
base09: number
base0A: type
base0B: string
base0C: special
base0D: function
base0E: keyword
base0F: operator
```

## 6.2 Fallbacks

If a role is missing:

```text
base00 fallback: #000000
base05 fallback: #ffffff
base01 fallback: darken(base00, 5%) or lighten(base00, 5%) depending on background luminance
base02 fallback: surface
base03 fallback: comment or muted_foreground or mix(base00, base05, 40%)
base04 fallback: mix(base00, base05, 55%)
base06 fallback: mix(base00, base05, 80%)
base07 fallback: mix(base00, base05, 95%)

base08 fallback: error or keyword
base09 fallback: number or constant
base0A fallback: warning or type
base0B fallback: string or git_added
base0C fallback: special or info
base0D fallback: function
base0E fallback: keyword
base0F fallback: operator or constant
```

Implement simple RGB mixing, lighten, darken, and luminance helpers.

---

# 7. Kitty Exporter

Generate a Kitty `.conf` file.

## 7.1 Mapping

```text
foreground              ← base05
background              ← base00
selection_foreground    ← base05
selection_background    ← base02
cursor                  ← cursor or base05
cursor_text_color       ← base00

color0                  ← base00
color1                  ← base08
color2                  ← base0B
color3                  ← base0A
color4                  ← base0D
color5                  ← base0E
color6                  ← base0C
color7                  ← base05

color8                  ← base03
color9                  ← brighten(base08)
color10                 ← brighten(base0B)
color11                 ← brighten(base0A)
color12                 ← brighten(base0D)
color13                 ← brighten(base0E)
color14                 ← brighten(base0C)
color15                 ← base07
```

## 7.2 Output example

```conf
# Generated from Helix theme: my-theme
foreground #c0caf5
background #1f2335

selection_foreground #c0caf5
selection_background #2f334d

cursor #7aa2f7
cursor_text_color #1f2335

color0 #1f2335
color1 #f7768e
color2 #9ece6a
color3 #e0af68
color4 #7aa2f7
color5 #bb9af7
color6 #7dcfff
color7 #c0caf5

color8 #565f89
color9 #ff8fa3
color10 #b4f07c
color11 #ffd479
color12 #8cb6ff
color13 #d0a8ff
color14 #8ff3ff
color15 #ffffff
```

---

# 8. bat Exporter

Generate a Sublime Text `.tmTheme` XML property-list file for bat.

bat custom syntax highlighting themes must be `.tmTheme` files. Newer `.sublime-color-scheme` files are not supported by bat for custom themes.

The generated file is intended to be installed under:

```bash
$(bat --config-dir)/themes
```

After installation, users must rebuild bat's cache:

```bash
bat cache --build
```

bat uses the `.tmTheme` filename as the theme name.

## 8.1 Architecture rule

The bat exporter must consume semantic roles and the derived palette. It must not inspect Helix TOML scopes directly and must not implement a direct `Helix TOML → bat` conversion.

The required flow is:

```text
Helix TOML
   ↓
Resolved Helix Theme
   ↓
Semantic Roles
   ↓
Derived 16-color palette
   ↓
bat .tmTheme exporter
```

## 8.2 Global settings

Map global `.tmTheme` settings from semantic roles first, then Base16-like fallback colors:

```text
background    ← background or base00
foreground    ← foreground or base05
caret         ← cursor or base05
selection     ← selection or base02
lineHighlight ← surface or base01
```

## 8.3 Syntax scope settings

Map semantic roles to Sublime-compatible scope selectors:

```text
comment                          ← comment
keyword                          ← keyword
entity.name.function             ← function
support.function                 ← function
storage.type                     ← type
entity.name.type                 ← type
variable                         ← variable
variable.parameter               ← parameter
string                           ← string
constant.numeric                 ← number
constant.language                ← constant
constant.other                   ← constant
keyword.operator                 ← operator
entity.name.tag                  ← special
support                          ← special
variable.language                ← special
invalid                          ← error
```

Each syntax setting should include a `foreground` color when the corresponding semantic role has a color.

If a semantic role is missing, exporters should use the relevant Base16-like fallback where one exists and emit a warning when the mapping is materially degraded.

## 8.4 Font styles and loss

`.tmTheme` can represent some text styling but not all Helix style data.

Supported mappings:

```text
bold          → fontStyle "bold"
italic        → fontStyle "italic"
bold+italic   → fontStyle "bold italic"
```

Unsupported or lossy Helix style data must be reported in the export report:

```text
dim
crossed_out
reversed
scope background colors that are not represented in the chosen .tmTheme setting
underline color
underline style
```

## 8.5 Output example

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>name</key>
  <string>my-theme</string>
  <key>settings</key>
  <array>
    <dict>
      <key>settings</key>
      <dict>
        <key>background</key>
        <string>#1f2335</string>
        <key>foreground</key>
        <string>#c0caf5</string>
        <key>caret</key>
        <string>#7aa2f7</string>
        <key>selection</key>
        <string>#2f334d</string>
      </dict>
    </dict>
    <dict>
      <key>name</key>
      <string>Keyword</string>
      <key>scope</key>
      <string>keyword</string>
      <key>settings</key>
      <dict>
        <key>foreground</key>
        <string>#bb9af7</string>
      </dict>
    </dict>
  </array>
</dict>
</plist>
```

---

# 9. Loss Report

Every export should produce a report.

Example:

```text
Export report: kitty
Source: tokyo-night.toml

Preserved:
  background: ui.background.bg
  foreground: ui.text.fg
  color1: diagnostic.error.underline.color
  color4: function.fg

Dropped:
  91 syntax scopes
  modifier italic
  modifier bold
  underline curl

Warnings:
  Missing role: git_modified
  Inferred base01 from background
```

Provide reports as:

1. Human-readable text
2. Optional JSON via `--report-json`

---

# 10. CLI

Suggested commands:

```bash
themeforge inspect path/to/theme.toml
themeforge resolve path/to/theme.toml
themeforge resolve path/to/theme.toml --theme-dir user/themes --theme-dir builtin/themes
themeforge export kitty path/to/theme.toml --out theme.conf
themeforge export base16 path/to/theme.toml --out theme.yaml
themeforge export bat path/to/theme.toml --out theme.tmTheme
themeforge batch-export kitty path/to/themes --out-dir generated/kitty
```

## 10.1 Inheritance lookup

`--theme-dir` is optional and may be passed more than once.

When provided, inherited theme names are searched in the given directories in command-line order.

When omitted, inherited theme names are searched in the input theme file's parent directory.

## 10.2 Useful options

```text
--strict
  Treat warnings as errors.

--report
  Print human-readable loss report.

--report-json path
  Write machine-readable report.

--pretty
  Pretty-print JSON/TOML output.

--dry-run
  Parse, resolve, and report without writing files.
```

---

# 11. Tests

## 11.1 Unit tests

Test:

1. TOML parsing
2. Palette resolution
3. Inheritance resolution
4. Color normalization
5. Scope-to-role derivation
6. Base16 extraction
7. Kitty export
8. bat `.tmTheme` export
9. Loss report generation

## 11.2 Fixture themes

Create fixtures for:

```text
minimal.toml
inherits-parent.toml
inherits-child.toml
literal-colors.toml
palette-references.toml
missing-palette-reference.toml
modifiers.toml
underline.toml
rich-theme.toml
```

## 11.3 Golden output tests

For selected fixture themes, compare generated outputs against committed snapshots:

```text
tests/golden/kitty/minimal.conf
tests/golden/base16/minimal.yaml
tests/golden/bat/minimal.tmTheme
```

---

# 12. Acceptance Criteria

The tool is acceptable when:

1. It can parse a real Helix theme TOML file.
2. It can resolve inherited themes.
3. It resolves palette references to concrete colors.
4. It derives semantic roles from scope assignments.
5. It exports a valid Kitty theme.
6. It exports a Base16-like 16-color palette.
7. It exports a valid bat `.tmTheme` theme.
8. It emits a clear loss report.
9. It includes unit tests and golden output tests.
10. It does not rely on palette names as primary semantic meaning.

---

# 13. Recommended Implementation Order

Implement in this order:

```text
1. Project scaffold
2. TOML parser
3. Internal Style and Theme types
4. Palette resolver
5. Inheritance resolver
6. Semantic role derivation
7. 16-color extraction
8. Kitty exporter
9. CLI
10. bat exporter
11. Reports
12. Batch export
13. Golden tests
```

---

# 14. Design Principle

The architecture should be:

```text
Helix TOML
   ↓
Resolved Helix Theme
   ↓
Semantic Roles
   ↓
Derived 16-color palette
   ↓
Exporters
   ├── Kitty
   ├── Base16-like YAML
   └── bat .tmTheme
```

Never implement direct one-off conversion logic such as:

```text
Helix TOML → Kitty
Helix TOML → bat
```

Instead, always go through the semantic model.

This keeps the system extensible for future exporters such as Zed, Alacritty, Ghostty, Neovim, WezTerm, Sublime Text, and Base24.
