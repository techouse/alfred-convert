#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly ROOT_DIR
cd "$ROOT_DIR"
MANIFEST_VERSION="$(awk '/^\[package\]$/ { p = 1; next } p && /^\[/ { exit } p && /^version = / { gsub(/["[:space:]]/, "", $3); print $3; exit }' Cargo.toml)"
readonly MANIFEST_VERSION
LOCK_VERSION="$(awk '/^\[\[package\]\]$/ { p = 1; name = ""; next } p && /^name = "alfred_convert"$/ { name = "alfred_convert"; next } p && name == "alfred_convert" && /^version = / { gsub(/["[:space:]]/, "", $3); print $3; exit }' Cargo.lock)"
readonly LOCK_VERSION
if [[ -z "$MANIFEST_VERSION" || -z "$LOCK_VERSION" ]]; then
  echo "Could not read the alfred_convert version from Cargo.toml and Cargo.lock" >&2
  exit 1
fi
if [[ "$MANIFEST_VERSION" != "$LOCK_VERSION" ]]; then
  echo "Cargo.toml version ($MANIFEST_VERSION) does not match Cargo.lock ($LOCK_VERSION)" >&2
  exit 1
fi
TAG_NAME="${1:-}"
if [[ -n "$TAG_NAME" && "$MANIFEST_VERSION" != "${TAG_NAME#v}" ]]; then
  echo "Cargo.toml version ($MANIFEST_VERSION) does not match tag (${TAG_NAME#v})" >&2
  exit 1
fi
echo "Version $MANIFEST_VERSION is consistent"
