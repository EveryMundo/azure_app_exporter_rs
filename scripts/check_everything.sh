#!/usr/bin/env sh

set -eu

MANIFEST="$(dirname "$0")/../Cargo.toml"

cargo deny --manifest-path "$MANIFEST" check advisories
cargo deny --manifest-path "$MANIFEST" check licenses
