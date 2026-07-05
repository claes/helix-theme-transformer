.PHONY: build check clippy fmt test download-helix-themes generate-themes release-nix clean distclean

RELEASE_TAG ?= generated-themes-latest

build:
	cargo build

check:
	cargo check

clippy:
	cargo clippy --all-targets --all-features

fmt:
	cargo fmt

test:
	cargo test --all

download-helix-themes:
	./scripts/download-helix-themes.sh

helix-themes:
	./scripts/download-helix-themes.sh

generated-themes: helix-themes
	cargo build
	./scripts/generate-from-helix-themes.sh

generate-themes: generated-themes

generated-themes/manifest.json: generated-themes scripts/create-release-manifest.sh
	./scripts/create-release-manifest.sh

generated-themes.zip: generated-themes/manifest.json
	rm -f generated-themes.zip
	zip -r generated-themes.zip generated-themes

generated-themes.nix: generated-themes.zip scripts/create-release-nix.sh
	./scripts/create-release-nix.sh "$(RELEASE_TAG)"

release-nix: generated-themes.nix

clean:
	rm -rf generated-themes
	rm -f generated-themes.zip generated-themes.nix

distclean: clean
	rm -rf helix-themes
	cargo clean
