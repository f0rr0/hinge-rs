#!/usr/bin/env sh
set -eu

mode="${1:-ci}"
root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

run() {
  printf '\n==> %s\n' "$*"
  "$@"
}

require() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

run_prek() {
  require prek
  stage="${1:?run_prek requires a stage}"
  shift
  run prek run --all-files --stage "$stage" "$@"
}

generated_docs() {
  printf '%s\n' \
    openapi/hinge-api.openapi.json \
    docs/api/openapi.json \
    docs/api/index.html \
    docs/api/versions.json
}

check_generated_docs() {
  run git diff --exit-code -- $(generated_docs)
}

build_docs() {
  base="${RUNNER_TEMP:-}"
  if [ -z "$base" ]; then
    base="$(mktemp -d)"
  fi
  run scripts/build-pages.sh "$base/pages"
}

case "$mode" in
  pre-commit)
    require prek
    run prek validate-config prek.toml
    run_prek pre-commit
    ;;

  pre-push)
    run_prek pre-push
    ;;

  clippy)
    require cargo
    run cargo clippy --all-targets --all-features -- -D warnings
    ;;

  test)
    require cargo
    run cargo test --locked
    ;;

  xtask-test)
    require cargo
    run cargo test --locked --manifest-path xtask/Cargo.toml
    ;;

  docs)
    require cargo
    run cargo run --locked --manifest-path xtask/Cargo.toml -- openapi
    check_generated_docs
    build_docs
    ;;

  package)
    require cargo
    run cargo package --locked
    ;;

  ci)
    run scripts/validate.sh pre-commit
    run scripts/validate.sh clippy
    run scripts/validate.sh test
    run scripts/validate.sh xtask-test
    run scripts/validate.sh docs
    run scripts/validate.sh package
    ;;

  release)
    run scripts/validate.sh pre-commit
    run scripts/validate.sh clippy
    run scripts/validate.sh test
    run scripts/validate.sh xtask-test
    run scripts/validate.sh docs
    run scripts/validate.sh package
    ;;

  *)
    cat >&2 <<'MSG'
usage: scripts/validate.sh <mode>

modes:
  pre-commit  validate prek config and run pre-commit hooks
  pre-push    run pre-push hooks
  clippy      run clippy with warnings denied
  test        run crate tests
  xtask-test  run xtask tests
  docs        regenerate and validate OpenAPI/Scalar docs
  package     verify the crates.io package
  ci          focused local/CI package checks
  release     pre-release verification
MSG
    exit 2
    ;;
esac
