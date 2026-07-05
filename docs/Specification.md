# Specification: Helix Theme Semantic Converter

## Goal

Build a CLI tool that uses Helix editor themes as the source format and converts them into other theme formats through a semantic intermediate representation.

Initial targets:

1. Helix TOML input
2. Semantic IR
3. base16 palette extraction
4. Kitty terminal theme export
5. bat `.tmTheme` syntax highlighting theme export
6. gitui `theme.ron` and `.tmTheme` export
7. Midnight Commander skin export
8. GNU `dircolors` database export
9. Yazi `theme.toml` export

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
htt export path/to/theme.toml --out-dir generated --theme-dir user/themes --theme-dir builtin/themes
```

If no `--theme-dir` is provided, use the input theme file's parent directory as the only theme search directory.

Rules:

1. Parent palette entries are inherited.
2. Child palette entries override parent palette entries with the same key.
3. Parent scope styles are inherited.
4. Child scope styles override parent scope styles with the same key.
5. Inheritance must merge raw palette references and scope styles before resolving palette references to concrete colors.
6. Exporters must only consume resolved themes.
7. Exporters must not need to understand `inherits`.

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

Suggested Rust model:

```rust
type HexColor = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnderlineStyle {
    Line,
    Curl,
    Dashed,
    Dotted,
    DoubleLine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Modifier {
    Bold,
    Italic,
    Dim,
    CrossedOut,
    Reversed,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct Underline {
    color: Option<HexColor>,
    style: Option<UnderlineStyle>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct Style {
    fg: Option<HexColor>,
    bg: Option<HexColor>,
    underline: Option<Underline>,
    modifiers: Vec<Modifier>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedTheme {
    name: String,
    source_path: camino::Utf8PathBuf,
    palette: indexmap::IndexMap<String, HexColor>,
    scopes: indexmap::IndexMap<String, Style>,
    warnings: Vec<Warning>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Warning {
    code: String,
    message: String,
    scope: Option<String>,
}
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
directory

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

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
struct SemanticRoleValue {
    color: Option<HexColor>,
    source_scope: Option<String>,
    source_property: Option<SourceProperty>,
    confidence: Confidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceProperty {
    Fg,
    Bg,
    UnderlineColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Confidence {
    Exact,
    Fallback,
    Inferred,
    Missing,
}
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
  - ["ui.background", "fg"]

muted_foreground:
  - ["comment", "fg"]
  - ["ui.linenr", "fg"]
  - ["ui.text.inactive", "fg"]

bright_foreground:
  []

Helix does not have a reliable neutral bright-foreground UI scope. Do not derive this role from cursor text, focused text, or headings; those are commonly accent colors. Let palette extraction infer bright neutral colors from background and foreground when this role is missing.

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

# 7. Shared File Kind Styling

File-oriented exporters must map target-specific file classes and statuses to shared `FileKind` values before resolving colors.

Required mappings:

```text
Directory       -> Directory, then Function/base0D, bold
Symlink         -> Special/base0C, bold
Executable      -> String/base0B, bold
Fifo            -> Warning/base0A
Socket          -> Keyword/base0E, bold
Device          -> Warning/base0A, bold
BrokenLink      -> Error/base08, bold
Missing         -> MutedForeground/base03
Setuid          -> Error/base08 background
Setgid          -> Warning/base0A background
WritableDir     -> GitAdded/base0B background
StickyDir       -> Special/base0C background
Archive         -> Number/base09
ImageVideo      -> Keyword/base0E, bold
Audio           -> Special/base0C
Document        -> String/base0B
Source          -> Keyword/base0E
Database        -> Type/base0A
Temporary       -> MutedForeground/base03
GitAdded        -> GitAdded/base0B
GitModified     -> GitModified or Warning/base0A
GitRemoved      -> GitRemoved or Error/base08
GitMoved        -> Special/base0C
```

This follows common terminal conventions for directories, symlinks, executables, devices, and Git status. Archives intentionally use `Number/base09` instead of error red so they do not conflict with broken links or removed files.

Each exporter may render emphasis using target-specific syntax, but semantic roles and fallback colors must come from the shared file kind style table.

---

# 8. Kitty Exporter

Generate a Kitty `.conf` file.

## 8.1 Mapping

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

## 8.2 Output example

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

# 9. bat Exporter

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

## 9.1 Architecture rule

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

## 9.2 Global settings

Map global `.tmTheme` settings from semantic roles first, then Base16-like fallback colors:

```text
background    ← background or base00
foreground    ← foreground or base05
caret         ← cursor or base05
selection     ← selection or base02
lineHighlight ← surface or base01
```

## 9.3 Syntax scope settings

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

## 9.4 Font styles and loss

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

## 9.5 Output example

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

# 10. Gitui Exporter

Generate a gitui theme directory containing both UI and syntax theme files.

The user provides a parent output directory. Helix Theme Transformer must create a theme directory from the input TOML file name, then create a `gitui/` directory inside it:

```text
<out-dir>/<theme-file-name>/gitui/theme.ron
<out-dir>/<theme-file-name>/gitui/<resolved-theme-name>.tmTheme
```

`theme.ron` is a gitui RON patch file for UI colors. The `.tmTheme` file is a TextMate syntax highlighting theme. The RON file must reference the generated syntax theme by file stem:

```ron
(
  syntax: Some("my-theme"),
)
```

If the resolved theme name is `my-theme`, the generated syntax file must be:

```text
my-theme.tmTheme
```

## 10.1 Architecture rule

The gitui exporter must consume semantic roles and the derived palette. It must not inspect Helix TOML scopes directly and must not implement a direct `Helix TOML → gitui` conversion.

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
gitui exporter
   ├── theme.ron
   └── <resolved-theme-name>.tmTheme
```

The syntax `.tmTheme` file should reuse the same TextMate exporter behavior used for the bat exporter.

## 10.2 UI color mapping

Map gitui RON fields from semantic roles first, then Base16-like fallback colors:

```text
selected_tab         ← special or base0C
command_fg           ← foreground or base05
selection_bg         ← selection or base02
selection_fg         ← foreground or base05
cmdbar_bg            ← surface or base01
disabled_fg          ← muted_foreground or base03

diff_line_add        ← git_added or base0B
diff_line_delete     ← git_removed or error or base08
diff_file_added      ← git_added or base0B
diff_file_removed    ← git_removed or error or base08
diff_file_moved      ← special or base0C
diff_file_modified   ← git_modified or warning or base0A

commit_hash          ← constant or base09
commit_time          ← info or muted_foreground or base03
commit_author        ← variable or function or base0D

danger_fg            ← error or base08
push_gauge_bg        ← selection or base02
push_gauge_fg        ← foreground or base05
tag_fg               ← special or base0C
branch_fg            ← type or base0A
block_title_focused  ← bright_foreground or base07
```

## 10.3 RON output

Generate a deterministic RON patch. Every generated color value must be wrapped in `Some(...)`.

Example:

```ron
(
  selected_tab: Some("#7dcfff"),
  command_fg: Some("#c0caf5"),
  selection_bg: Some("#3b4261"),
  selection_fg: Some("#c0caf5"),
  cmdbar_bg: Some("#292e42"),
  disabled_fg: Some("#565f89"),
  diff_line_add: Some("#9ece6a"),
  diff_line_delete: Some("#f7768e"),
  diff_file_added: Some("#9ece6a"),
  diff_file_removed: Some("#f7768e"),
  diff_file_moved: Some("#7dcfff"),
  diff_file_modified: Some("#e0af68"),
  commit_hash: Some("#ff9e64"),
  commit_time: Some("#7dcfff"),
  commit_author: Some("#7aa2f7"),
  danger_fg: Some("#f7768e"),
  push_gauge_bg: Some("#3b4261"),
  push_gauge_fg: Some("#c0caf5"),
  tag_fg: Some("#7dcfff"),
  branch_fg: Some("#e0af68"),
  block_title_focused: Some("#c0caf5"),
  syntax: Some("my-theme"),
)
```

## 10.4 CLI output behavior

The single export command always generates gitui output along with every other supported format.

Example:

```bash
htt export path/to/my-theme.toml --out-dir generated
```

This creates:

```text
generated/my-theme/gitui/theme.ron
generated/my-theme/gitui/<resolved-theme-name>.tmTheme
```

The gitui exporter must not require individual output file options for the RON or `.tmTheme` files.

---

# 11. Midnight Commander Exporter

Generate Midnight Commander skin, file classification, and color table files:

```text
<out-dir>/<theme-file-name>/mc/<resolved-theme-name>.ini
<out-dir>/<theme-file-name>/mc/filehighlight.ini
<out-dir>/<theme-file-name>/mc/colortable.env
```

Midnight Commander user skins are commonly installed under:

```text
~/.local/share/mc/skins/
```

The skin can then be selected with:

```bash
mc -S <resolved-theme-name>
```

## 11.1 Architecture rule

The Midnight Commander exporter must consume semantic roles and the derived palette. It must not inspect Helix TOML scopes directly and must not implement a direct `Helix TOML -> Midnight Commander` conversion.

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
Midnight Commander exporter
   ├── <resolved-theme-name>.ini
   ├── filehighlight.ini
   └── colortable.env
```

## 11.2 Skin header

Generate truecolor skins first. This preserves resolved Helix colors without quantizing to the 256-color terminal palette.

The description must begin with the resolved theme name, followed by generated provenance:

```ini
[skin]
    description = my-theme - generated by Helix Theme Transformer
    truecolors = true
```

## 11.3 Color syntax

Midnight Commander skin colors use:

```text
foreground;background;attributes
```

Attributes are optional. Multiple attributes are joined with `+`.

Example:

```ini
[core]
    _default_ = #c0caf5;#1f2335
    selected = #c0caf5;#3b4261
    marked = #e0af68;#1f2335;bold
```

## 11.4 Line drawing

Generated Midnight Commander skins must include a `[Lines]` section with explicit UTF-8 line drawing characters. Do not rely on Midnight Commander's fallback line definitions for generated skins.

Use the standard single-line and double-line border characters from bundled Midnight Commander skins, including:

```ini
[Lines]
    horiz = ─
    vert = │
    lefttop = ┌
    righttop = ┐
    leftbottom = └
    rightbottom = ┘
    cross = ┼
    dhoriz = ═
    dvert = ║
```

## 11.5 UI mapping

Map Midnight Commander fields from semantic roles first, then Base16-like fallback colors.

Use these sections initially:

```text
core
dialog
error
filehighlight
menu
popupmenu
buttonbar
statusbar
help
editor
viewer
diffviewer
```

Important mappings:

```text
core._default_      ← foreground;background
core.selected       ← foreground;selection
core.marked         ← warning;background;bold
core.header         ← type;background;bold

dialog._default_    ← foreground;surface
dialog.dfocus       ← background;selection;bold

error._default_     ← foreground;error
error.errdfocus     ← background;warning;bold

menu._default_      ← foreground;surface
menu.menusel        ← foreground;selection

statusbar._default_ ← background;special

editor._default_    ← foreground;background
editor.editmarked   ← foreground;selection
editor.bookmark     ← foreground;error
editor.bookmarkfound ← background;git_added

viewer._default_    ← foreground;background
viewer.viewunderline ← special;background;underline
viewer.viewselected ← foreground;selection

diffviewer.added    ← background;git_added
diffviewer.changed  ← background;git_modified
diffviewer.removed  ← foreground;git_removed
diffviewer.error    ← foreground;error
```

## 11.6 File highlighting

Generate `filehighlight.ini` from shared file extension groups so MC classifies files consistently with `dircolors`.

The file must include MC structural classes:

```text
executable
directory
device
special
stalelink
symlink
hardlink
```

It must also cover every shared extension group used by `dircolors`:

```text
archive
doc
source
media
graph
database
temp
```

`ImageVideo` maps to MC `graph`; `Audio` maps to MC `media`.

## 11.7 Color table environment file

Generate `colortable.env` as a shell source file that exports `MC_COLOR_TABLE`:

```sh
export MC_COLOR_TABLE='normal=white,default:selected=black,cyan'
```

Users can enable it with:

```bash
source generated-themes/my-theme/mc/colortable.env
mc
```

The value must use the `mc(1)` `MC_COLOR_TABLE` format:

```text
<keyword>=<fgcolor>,<bgcolor>,<attributes>:<keyword>=...
```

Generate entries for the documented MC UI keys, including normal, selection, menu, dialog, error, help, viewer, editor, popup menu, button bar, and status bar keys.

`MC_COLOR_TABLE` does not support truecolor hex values. Convert resolved semantic colors to the nearest supported 256-color MC token, preferring `gray0` to `gray23` for grayscale colors and `rgb000` to `rgb555` for color cube values.

## 11.8 Loss reporting

The Midnight Commander exporter should report:

1. Helix syntax scopes collapse to Midnight Commander UI fields.
2. Helix underline styles collapse to Midnight Commander `underline`.
3. Helix `dim`, `crossed_out`, and `reversed` modifiers are not represented.
4. Alpha is unsupported by Midnight Commander skins.
5. `MC_COLOR_TABLE` quantizes truecolor values to MC 256-color tokens.

---

# 12. Dircolors Exporter

Generate a GNU `dircolors` database file:

```text
<out-dir>/<theme-file-name>/dircolors/<resolved-theme-name>.dircolors
```

The exported file must be a `dircolors` input database, not a raw `LS_COLORS` environment assignment.

Users can load it with:

```bash
eval "$(dircolors generated/<theme-file-name>/dircolors/<resolved-theme-name>.dircolors)"
```

## 12.1 Architecture rule

The dircolors exporter must consume semantic roles and the derived palette. It must not inspect Helix TOML scopes directly and must not implement a direct `Helix TOML -> dircolors` conversion.

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
dircolors exporter
   └── <resolved-theme-name>.dircolors
```

## 12.2 Truecolor SGR output

Use truecolor SGR sequences so resolved Helix Theme Transformer colors can be preserved directly.

Foreground colors use:

```text
38;2;<r>;<g>;<b>
```

Background colors use:

```text
48;2;<r>;<g>;<b>
```

Attributes are prepended in deterministic order:

```text
01;38;2;158;206;106
04;38;2;125;207;255
38;2;192;202;245;48;2;59;66;97
```

This requires compatible `dircolors`, `ls`, and terminal behavior.

## 12.3 File format

Start the database with terminal filters:

```text
# Generated from Helix theme: my-theme
COLORTERM ?*
TERM *color*
TERM *direct*
TERM xterm*
TERM screen*
TERM tmux*
```

Then emit core file type keys and extension mappings.

## 12.4 Core file type mapping

Map file classes from semantic roles first, then Base16-like fallback colors.

Preserve the distinction used by GNU defaults:

```text
directories       blue-like
symlinks          cyan-like
sockets/doors     magenta-like
executables       green-like
warnings/devices  yellow-like
broken links      red-like
```

Concrete mapping:

```text
RESET                   ← 0
DIR                     ← function or git_added
LINK                    ← special
MULTIHARDLINK           ← foreground
FIFO                    ← warning
SOCK                    ← keyword
DOOR                    ← keyword
BLK                     ← warning
CHR                     ← warning
ORPHAN                  ← error
MISSING                 ← muted_foreground
SETUID                  ← foreground on error
SETGID                  ← background on warning
CAPABILITY              ← foreground
STICKY_OTHER_WRITABLE   ← background on git_added
OTHER_WRITABLE          ← special on git_added
STICKY                  ← foreground on special
EXEC                    ← function or git_added
```

## 12.5 Extension mapping

Use `*.<extension>` keys accepted by `dircolors`.

Initial extension groups:

```text
archives/compressed  ← error, bold
images/video         ← special, bold
audio                ← special
documents            ← string or warning
source/code          ← keyword or function
temp/cache/logs      ← muted_foreground
```

Example:

```text
*.rs 38;2;187;154;247
*.zip 01;38;2;247;118;142
*.png 01;38;2;125;207;255
```

## 12.6 Loss reporting

The dircolors exporter should report:

1. Helix syntax scopes collapse to LS_COLORS file classes and extensions.
2. LS_COLORS supports SGR attributes only.
3. Truecolor output requires compatible `dircolors`, `ls`, and terminal behavior.

---

# 13. Yazi Exporter

Generate a Yazi direct theme file:

```text
<out-dir>/<theme-file-name>/yazi/theme.toml
```

The Yazi exporter must consume semantic roles and the derived palette. It must not inspect Helix TOML scopes directly and must not implement a direct `Helix TOML -> Yazi` conversion.

The generated file is a Yazi `theme.toml`, not a flavor directory. It should style core UI sections such as `[app]`, `[mgr]`, `[mode]`, `[status]`, `[tabs]`, dialogs, notifications, completion, tasks, help, and `[filetype]`.

`[filetype].rules` must reuse the shared `FileKind` mapping so equivalent file classes and extension groups stay consistent with `dircolors`, Midnight Commander, and gitui.

Yazi uses the first matching filetype rule, so broader fallback rules must be emitted after more specific file type, extension, and directory rules.

Yazi syntax preview theming is out of scope for the direct `theme.toml` exporter. A future flavor exporter may generate `flavor.toml` and `tmtheme.xml`.

The Yazi exporter should report:

1. Which semantic roles were preserved in Yazi fields.
2. Which file kind mappings were used.
3. That flavor metadata and syntax preview theming were not generated.

---

# 14. Loss Report

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

# 15. CLI

Suggested commands:

```bash
htt inspect path/to/theme.toml
htt resolve path/to/theme.toml
htt resolve path/to/theme.toml --theme-dir user/themes --theme-dir builtin/themes
htt export path/to/my-theme.toml --out-dir generated
```

The export command always generates all supported formats. It never writes exported theme files to stdout.

Each exporter returns a list of generated files plus one export report. Single-file exporters return a one-item file list; multi-file exporters return all files for that format.

`--out-dir` identifies the parent directory where Helix Theme Transformer creates one theme directory. The theme directory name is derived automatically from the input TOML file name without its extension, after applying the same filename sanitization used for generated files.

The output directory layout is:

```text
generated/
  my-theme/
    kitty/<resolved-theme-name>.conf
    base16/<resolved-theme-name>.yaml
    bat/<resolved-theme-name>.tmTheme
    gitui/theme.ron
    gitui/<resolved-theme-name>.tmTheme
    mc/<resolved-theme-name>.ini
    mc/filehighlight.ini
    mc/colortable.env
    dircolors/<resolved-theme-name>.dircolors
    yazi/theme.toml
```

## 14.1 Inheritance lookup

`--theme-dir` is optional and may be passed more than once.

When provided, inherited theme names are searched in the given directories in command-line order.

When omitted, inherited theme names are searched in the input theme file's parent directory.

## 14.2 Useful options

```text
--strict
  Treat warnings as errors.

--report
  Print human-readable loss report.

--report-json path
  Write a machine-readable array of export reports.

--pretty
  Pretty-print JSON/TOML output.

--dry-run
  Parse, resolve, and report without writing files.
```

---

# 15. Release Artifacts

Generated theme releases are distributed as a GitHub Release containing at least:

```text
generated-themes.zip
generated-themes.nix
```

`generated-themes.zip` contains the generated theme tree:

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
    yazi/theme.toml
    helix/<theme-file-name>.toml
```

The `helix/` directory preserves the source Helix theme TOML file used to generate the exported files. The top-level `CREDITS` file must identify the Helix repository theme directory as the source of the Helix theme files and credit the original authors and Helix project contributors.

`manifest.json` stores release data as relative paths inside `generated-themes/`. It must include only generated files that exist.

Example:

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
      "yazi": {
        "theme": "adwaita-dark/yazi/theme.toml"
      },
      "helix": {
        "theme": "adwaita-dark/helix/adwaita-dark.toml"
      }
    }
  }
}
```

## 15.1 Nix release file

`generated-themes.nix` is generated for NixOS and Home Manager users. It fetches the released zip with `pkgs.fetchzip`, embeds the fixed-output hash for the unpacked archive, reads `generated-themes/manifest.json`, and exposes stable attributes for generated theme files.

The file has this shape:

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

The public attribute shape is defined by `manifest.json`. Theme names are JSON object keys, so names with hyphens or other non-identifier characters remain valid when accessed as quoted Nix attributes.

## 15.2 Home Manager consumption

A NixOS flake using Home Manager can consume released themes by importing `generated-themes.nix` as a non-flake input:

```nix
inputs.htt-themes-nix = {
  url = "https://github.com/claes/helix-theme-transformer/releases/download/generated-themes-latest/generated-themes.nix";
  flake = false;
};
```

Then use the generated attributes as Home Manager file sources:

```nix
let
  httThemes = import htt-themes-nix { inherit pkgs; };
in {
  xdg.configFile."kitty/current-theme.conf".source =
    httThemes.themes."adwaita-dark".kitty.theme;
}
```

`generated-themes.nix` should be generated by a script and release workflow, not hand-maintained. The Makefile release target should be runnable from a clean checkout and produce both release artifacts.

---

# 16. Tests

## 16.1 Unit tests

Test:

1. TOML parsing
2. Palette resolution
3. Inheritance resolution
4. Color normalization
5. Scope-to-role derivation
6. Base16 extraction
7. Kitty export
8. bat `.tmTheme` export
9. gitui `theme.ron` export
10. Midnight Commander skin export
11. GNU `dircolors` export
12. Yazi `theme.toml` export
13. Loss report generation

## 16.2 Fixture themes

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

## 16.3 Golden output tests

For selected fixture themes, compare generated outputs against committed snapshots:

```text
tests/golden/kitty/minimal.conf
tests/golden/base16/minimal.yaml
tests/golden/bat/minimal.tmTheme
tests/golden/gitui/theme.ron
tests/golden/mc/minimal.ini
tests/golden/mc/filehighlight.ini
tests/golden/mc/colortable.env
tests/golden/dircolors/minimal.dircolors
tests/golden/yazi/minimal.toml
```

## 16.4 Interactive tool test scripts

Every exporter for an interactive tool must include a matching tool test script under:

```text
scripts/tool-test-scripts/
```

The script must accept a generated theme name as its first argument:

```bash
./scripts/tool-test-scripts/<tool>.sh adwaita-dark
```

It should invoke the target tool with the generated theme files from `generated-themes/<theme>/...` and show a useful interactive or visual sample for tuning the exporter output.

When the target tool requires configuration files or cache files, the script should prefer temporary isolated config/cache directories instead of mutating the user's real configuration.

Exporters for non-interactive data formats, such as Base16 YAML, do not require a tool test script unless a practical visual preview tool is added.

---

# 17. Acceptance Criteria

The tool is acceptable when:

1. It can parse a real Helix theme TOML file.
2. It can resolve inherited themes.
3. It resolves palette references to concrete colors.
4. It derives semantic roles from scope assignments.
5. It exports a valid Kitty theme.
6. It exports a Base16-like 16-color palette.
7. It exports a valid bat `.tmTheme` theme.
8. It exports a valid gitui theme directory containing `theme.ron` and a matching `.tmTheme` syntax file.
9. It exports valid Midnight Commander skin, file highlighting, and color table files.
10. It exports a valid GNU `dircolors` database for LS_COLORS.
11. It exports a valid Yazi `theme.toml`.
12. It emits a clear loss report.
13. It includes unit tests and golden output tests.
14. It does not rely on palette names as primary semantic meaning.
15. It can generate release artifacts containing `generated-themes.zip` and a parseable `generated-themes.nix` for NixOS/Home Manager consumption.
16. Every interactive tool exporter has a matching script in `scripts/tool-test-scripts/` for locally previewing and tuning generated themes.
17. File-oriented exporters share the same `FileKind` style mapping for equivalent file types and statuses.

---

# 18. Recommended Implementation Order

Implement in this order:

```text
1. Project scaffold
2. TOML parser
3. Internal Style and Theme types
4. Palette resolver
5. Inheritance resolver
6. Semantic role derivation
7. 16-color extraction
8. Shared file kind styling
9. Kitty exporter
10. CLI
11. bat exporter
12. gitui exporter
13. Midnight Commander exporter
14. dircolors exporter
15. Yazi exporter
16. Reports
17. Batch export
18. Release Nix artifact generation
19. Golden tests
20. Interactive tool test scripts
```

---

# 19. Design Principle

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
   ├── bat .tmTheme
   ├── gitui
   │   ├── theme.ron
   │   └── <resolved-theme-name>.tmTheme
   ├── Midnight Commander skin
   ├── dircolors
   └── yazi theme.toml
```

Never implement direct one-off conversion logic such as:

```text
Helix TOML → Kitty
Helix TOML → bat
Helix TOML → gitui
Helix TOML → Midnight Commander
Helix TOML → dircolors
Helix TOML → Yazi
```

Instead, always go through the semantic model.

This keeps the system extensible for future exporters such as Zed, Alacritty, Ghostty, Neovim, WezTerm, Sublime Text, and Base24.
