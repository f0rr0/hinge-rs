#!/usr/bin/env bash
set -euo pipefail

cargo fmt -- --check
cargo fmt --manifest-path xtask/Cargo.toml -- --check
cargo metadata --format-version 1 --no-deps >/dev/null
cargo metadata --format-version 1 --manifest-path xtask/Cargo.toml --no-deps >/dev/null
