#!/usr/bin/env bash
set -euo pipefail

generated_docs=(
  openapi/hinge-api.openapi.json
  docs/api/openapi.json
  docs/api/index.html
  docs/api/versions.json
)

if ! git diff --quiet -- "${generated_docs[@]}" ||
  ! git diff --cached --quiet -- "${generated_docs[@]}"; then
  echo "generated API docs have uncommitted changes; commit them before pushing:" >&2
  git diff --name-only -- "${generated_docs[@]}" >&2
  git diff --cached --name-only -- "${generated_docs[@]}" >&2
  exit 1
fi

cargo clippy --all-targets --all-features -- -D warnings
cargo test --locked
cargo run --locked --manifest-path xtask/Cargo.toml -- openapi
git diff --exit-code -- "${generated_docs[@]}"
