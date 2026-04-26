# Contributing

## Development

Run the same checks as CI before opening a pull request:

```bash
cargo fmt -- --check
cargo fmt --manifest-path xtask/Cargo.toml -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --locked
cargo test --locked --manifest-path xtask/Cargo.toml
cargo run --locked --manifest-path xtask/Cargo.toml -- openapi
```

Generated API reference artifacts must be committed when API docs change:

- `openapi/hinge-api.openapi.json`
- `docs/api/openapi.json`
- `docs/api/index.html`
- `docs/api/versions.json`

## Git Hooks

This repo uses `prek`, a Rust Git hook runner.

```bash
brew install prek
prek install --prepare-hooks
```

If you do not use Homebrew, install a prebuilt `prek` binary from the GitHub
release or with `cargo binstall prek`. Use `cargo install prek --locked` only as
the source-build fallback.

`prek install` installs the configured `pre-commit`, `pre-push`, and
`commit-msg` hooks.

- `pre-commit` is intentionally lean: fast file hygiene, Rust formatting, and
  Cargo metadata validation.
- `commit-msg` uses `committed` to enforce Conventional Commit subjects. The
  configured hook downloads the upstream prebuilt binary for normal setup.
- `pre-push` is heavier: all-feature check, clippy, tests, xtask tests, and
  OpenAPI/Scalar regeneration validation.

The slowest release gates stay in CI: feature powerset, dependency policy,
package verification, semver compatibility, release automation, and docs
deployment.

CI is the source of truth. Local hooks are convenience checks and can be skipped;
the CI workflow reruns the pre-commit hook set, enforces Conventional Commits
for PRs and direct pushes, and runs the heavier Rust, docs, package, dependency,
workflow-lint, and semver gates independently.

In GitHub branch protection, require the `Required checks` status before merging
to make skipped local hooks irrelevant.

## Commit Format

Use Conventional Commits so release-plz can calculate versions and changelog
entries:

- `feat: add typed endpoint` for new public functionality.
- `fix: correct response deserialization` for bug fixes.
- `docs: update Scalar examples` for documentation-only changes.
- `ci: tighten release verification` for workflow-only changes.
- `feat!: change public client API` or a `BREAKING CHANGE:` footer for
  breaking changes.

Only release-worthy commits create release PRs and changelog entries:

- `feat:`, `fix:`, `perf:`, `refactor:`, `security:`
- `build(deps):` for runtime dependency updates
- any conventional commit marked breaking with `!`

Documentation, CI, test, style, and chore commits can merge without forcing a
crate release.

## Releases

Releases are manual GitHub Actions runs:

1. Merge normal PRs into `main` after CI passes.
2. Run **Release** with `command=release-pr`.
3. Review the generated version and changelog PR. Edit the changelog in that PR
   if the wording needs cleanup.
4. Merge the release PR.
5. Optionally run **Release** with `command=release` and `dry-run=true`.
6. Run **Release** with `command=release` and `dry-run=false` to publish the
   crate, tag the commit, and create the GitHub release.

The **Docs** workflow deploys the generated Scalar API reference to GitHub Pages
after relevant changes land on `main`. The root page is latest, and release
snapshots are generated from git tags under `/v/<version>/` during docs and
release deployment. Historical snapshots are not committed to `main`.

For the first crates.io publish, configure `CARGO_REGISTRY_TOKEN` in the GitHub
repository secrets. The workflow also grants `id-token: write` so the project
can move to crates.io trusted publishing later.
