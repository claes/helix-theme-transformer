.PHONY: build check clippy fmt test download-helix-themes generate-themes clean distclean

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

generate-themes: build download-helix-themes
	./scripts/generate-from-helix-themes.sh

clean:
	rm -rf generated-themes

distclean: clean
	rm -rf helix-themes
	cargo clean
