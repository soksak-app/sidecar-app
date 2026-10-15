# The build, the test and the record checks of the application sidecar (docs/features.md).
.PHONY: build test release hooks

SOK ?= sok

node_modules: package.json
	pnpm install
	touch node_modules

build:
	cargo build --release
	mkdir -p build
	cp target/release/soksak-app build/soksak-app

test: node_modules
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings
	cargo test
	node_modules/.bin/soksak-records

# The release of the sidecar for the registry, made by sok of core from sidecar.json and the build.
release: build
	@test -n "$(OUT)" || { echo "make release OUT=<folder>" >&2; exit 2; }
	$(SOK) sidecar release . $(OUT)

# Sets this repository to use .githooks, which checks commit messages and records at commit time.
hooks:
	@git config core.hooksPath .githooks
