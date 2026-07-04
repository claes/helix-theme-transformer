# AGENTS.md

## Source of Truth

Read `docs/Specification.md` before making architectural changes.

That document defines the theme conversion pipeline, semantic model, role mapping, fallback behavior, exporters, CLI, reports, tests, and acceptance criteria.

This file only contains implementation guidance for agents working in this repository.

## Core Rule

Do not bypass the architecture from `docs/Specification.md`.

All conversions must follow this pipeline:

```text
Helix TOML
  → Resolved Helix Theme
  → Semantic Roles
  → Derived 16-color palette
  → Exporters
```

Do not implement direct one-off conversions such as `Helix TOML → Kitty`.

## Rust Implementation Guidance

Prefer a Rust workspace with small crates or modules:

```text
crates/
  theme-ir/          # shared data model
  helix-theme/       # Helix TOML parser and resolver
  semantic-roles/    # scope → role derivation
  palette16/         # Base16-like extraction
  exporters/         # Kitty, Base16, future exporters
  themeforge-cli/    # command-line interface
```

Recommended crates:

```toml
toml = "0.8"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
clap = { version = "4", features = ["derive"] }
thiserror = "1"
anyhow = "1"
camino = "1"
indexmap = "2"
similar-asserts = "1"
insta = "1"
```

Implementation preferences:

* Use `serde` for TOML parsing.
* Use `thiserror` for library errors.
* Use `anyhow` only in the CLI layer.
* Use `camino::Utf8PathBuf` for paths where practical.
* Use `indexmap::IndexMap` when stable output ordering matters.
* Keep exporters deterministic.
* Keep color utilities pure and unit-tested.
* Prefer explicit structs over loosely typed maps after parsing.
* Preserve warnings separately from fatal errors.
* Do not panic on malformed themes; return structured errors or warnings.

## Testing

For any meaningful change, add or update tests.

Prioritize:

* parser tests
* inheritance tests
* palette resolution tests
* semantic role derivation tests
* 16-color extraction tests
* Kitty golden output tests
* loss report tests

Use snapshot/golden tests for generated output.

## Commands

Before finishing changes, run:

```bash
cargo fmt
cargo clippy --all-targets --all-features
cargo test --all
```

## Documentation

Update `docs/Specification.md` when changing:

* semantic roles
* Helix scope mappings
* fallback behavior
* exporter behavior
* CLI behavior
* report format

Keep this file short. Do not duplicate the specification here.
