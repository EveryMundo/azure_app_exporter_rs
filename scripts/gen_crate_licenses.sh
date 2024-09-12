#!/usr/bin/env sh

set -eu

SCRIPT_DIR="$(dirname "$0")"

cargo about generate --manifest-path "$SCRIPT_DIR/../Cargo.toml" "$SCRIPT_DIR/about.hbs" > "$SCRIPT_DIR/crate_licenses.html"
