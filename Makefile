SHELL := /bin/bash

.PHONY: help build build-release build-release-binary build-arm64 build-x86_64 build-universal fmt fmt-check clippy test licenses package version-check ci clean

help:
	@printf '%-20s %s\n' 'Target' 'Description'
	@printf '%-20s %s\n' '------' '-----------'
	@printf '%-20s %s\n' 'build' 'Build the debug Rust executable.'
	@printf '%-20s %s\n' 'build-release' 'Build the native release workflow directory.'
	@printf '%-20s %s\n' 'build-universal' 'Build arm64 and Intel slices and combine workflow.'
	@printf '%-20s %s\n' 'fmt' 'Format Rust source files.'
	@printf '%-20s %s\n' 'fmt-check' 'Check Rust formatting.'
	@printf '%-20s %s\n' 'clippy' 'Run strict Clippy checks.'
	@printf '%-20s %s\n' 'test' 'Run all Rust tests.'
	@printf '%-20s %s\n' 'licenses' 'Generate third-party license notices.'
	@printf '%-20s %s\n' 'package' 'Create a universal .alfredworkflow package.'
	@printf '%-20s %s\n' 'version-check' 'Verify Cargo and optional tag versions agree.'
	@printf '%-20s %s\n' 'ci' 'Run all local CI checks.'
	@printf '%-20s %s\n' 'clean' 'Remove Rust and workflow build output.'

build:
	cargo build --locked

build-release:
	@$(MAKE) build-release-binary
	./scripts/package-workflow.sh target/release/alfred_convert

build-release-binary:
	./scripts/build-release.sh

build-arm64:
	CARGO_BUILD_TARGET=aarch64-apple-darwin MACOSX_DEPLOYMENT_TARGET=11.0 $(MAKE) build-release-binary

build-x86_64:
	CARGO_BUILD_TARGET=x86_64-apple-darwin MACOSX_DEPLOYMENT_TARGET=10.15 $(MAKE) build-release-binary

build-universal: build-arm64 build-x86_64
	@set -euo pipefail; \
	mkdir -p target/universal-apple-darwin/release; \
	lipo -create target/aarch64-apple-darwin/release/alfred_convert target/x86_64-apple-darwin/release/alfred_convert -output target/universal-apple-darwin/release/workflow; \
	chmod 755 target/universal-apple-darwin/release/workflow; \
	./scripts/package-workflow.sh target/universal-apple-darwin/release/workflow; \
	echo 'Created universal build/dist/workflow'

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all --check

clippy:
	cargo clippy --all-targets --all-features --locked -- -D warnings

test:
	cargo test --all-targets --locked

licenses:
	@mkdir -p build
	cargo-about generate --locked --fail --output-file build/THIRD_PARTY_LICENSES.html about.hbs

package: build-universal
	@set -euo pipefail; \
	VERSION="$$(awk '/^\[package\]$$/ { p = 1; next } p && /^\[/ { exit } p && /^version = / { gsub(/["[:space:]]/, "", $$3); print $$3; exit }' Cargo.toml)"; \
	WORKFLOW_NAME="$${WORKFLOW_NAME:-convert}"; \
	ARCHIVE="build/$${WORKFLOW_NAME}-v$${VERSION}.alfredworkflow"; \
	TEMP_ARCHIVE="$${ARCHIVE}.tmp.zip"; \
	trap 'rm -f "$$TEMP_ARCHIVE"' EXIT; \
	rm -f "$$TEMP_ARCHIVE"; \
	(cd build/dist && zip -qr "../$${WORKFLOW_NAME}-v$${VERSION}.alfredworkflow.tmp.zip" . -x '.env' 'exchange_rates_cache/*' 'image_cache/*' 'update_cache/*' '*_cache/*' '*_cache_keys.json' 'workflow_intel'); \
	mv -f "$$TEMP_ARCHIVE" "$$ARCHIVE"; \
	trap - EXIT; \
	echo "Created $$ARCHIVE"

version-check:
	./scripts/version-check.sh

ci: fmt-check test clippy version-check licenses

clean:
	cargo clean
	rm -rf build
