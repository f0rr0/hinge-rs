#!/usr/bin/env bash
set -euo pipefail

cargo check --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo test --locked
cargo test --locked --manifest-path xtask/Cargo.toml
cargo run --locked --manifest-path xtask/Cargo.toml -- openapi
git diff --exit-code -- \
  openapi/hinge-api.openapi.json \
  docs/api/openapi.json \
  docs/api/index.html \
  docs/api/versions.json
