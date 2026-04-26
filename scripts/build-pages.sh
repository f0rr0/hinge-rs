#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: scripts/build-pages.sh <output-dir>" >&2
  exit 2
fi

repo_root="$(git rev-parse --show-toplevel)"
output_parent="$(mkdir -p "$(dirname "$1")" && cd "$(dirname "$1")" && pwd)"
output_dir="${output_parent}/$(basename "$1")"
rm -rf "$output_dir"
mkdir -p "$output_dir"

cd "$repo_root"
current_version="$(cargo run --quiet --locked --manifest-path xtask/Cargo.toml -- version)"

versions=()
while IFS= read -r tag; do
  version="${tag#v}"
  if [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+([-.+][0-9A-Za-z.-]+)?$ ]]; then
    versions+=("$version")
  fi
done < <(git tag --list 'v[0-9]*' --sort=-v:refname)

versions_csv="$(IFS=,; echo "${versions[*]}")"

cargo run --quiet --locked --manifest-path xtask/Cargo.toml -- openapi \
  --site-dir "$output_dir" \
  --site-current latest \
  --site-root ./ \
  --site-latest-version "$current_version" \
  --site-versions "$versions_csv" \
  --site-versions-url ./versions.json

tmp_dir="$(mktemp -d)"
cleanup() {
  for worktree in "$tmp_dir"/*; do
    [[ -d "$worktree" ]] || continue
    git -C "$repo_root" worktree remove --force "$worktree" >/dev/null 2>&1 || true
  done
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

for version in "${versions[@]}"; do
  tag="v${version}"
  worktree="${tmp_dir}/${tag}"
  git worktree add --detach "$worktree" "$tag" >/dev/null
  (
    cd "$worktree"
    cargo run --quiet --locked --manifest-path xtask/Cargo.toml -- openapi \
      --site-dir "$output_dir/v/$version" \
      --site-current "$version" \
      --site-root ../../ \
      --site-latest-version "$current_version" \
      --site-versions-url ../../versions.json \
      --no-versions-json
  )
done

echo "built versioned API docs in ${output_dir}"
