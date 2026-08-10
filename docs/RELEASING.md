# Releasing sakurs

This document describes the PR-based release process for sakurs. A release PR
prepares immutable version metadata and release notes; an annotated tag on the
merged commit triggers publishing through GitHub Actions.

## Prerequisites

Before creating the release PR:

1. Confirm that the latest `main` workflows are green.
2. Confirm that the `CARGO_REGISTRY_TOKEN` repository secret exists. crates.io
   tokens expire after about a year, so an authentication failure should be
   treated as a token-rotation issue even if a previous release succeeded.
3. Confirm that PyPI has the GitHub Actions trusted publisher configured for:
   - Repository: `sog4be/sakurs`
   - Workflow: `.github/workflows/release.yml`
   - Environment: `pypi`
4. Confirm that a maintainer is available to approve the protected `pypi`
   environment. OIDC trust and the environment approval are separate: the
   `upload-pypi` job pauses for manual approval on every release.
5. Confirm that the target version does not already exist on crates.io, PyPI,
   or as a GitHub tag. Published versions cannot be overwritten.

The published packages are:

- crates.io: `sakurs-core`, followed by `sakurs-cli`
- PyPI: `sakurs`

`sakurs-core` is published first because `sakurs-cli` depends on the same
version of it.

## Release Steps

### 1. Create the release branch and update versions

Start from the latest `main`:

```bash
git switch main
git pull --ff-only origin main
git switch -c chore/release-vX.Y.Z
```

Update both version declarations that must change for a release:

1. `[workspace.package] version` in the root `Cargo.toml`
2. The `sakurs-core` dependency version in `sakurs-cli/Cargo.toml`

All three crates inherit their package version through
`version.workspace = true`, and maturin reads the Python package version from
the Rust package metadata. The CLI dependency requirement is independent and
must be updated explicitly.

Refresh `Cargo.lock` after editing the manifests:

```bash
cargo check --workspace
```

The expected version-related diff is therefore the root `Cargo.toml`,
`sakurs-cli/Cargo.toml`, and `Cargo.lock`.

### 2. Finalize the changelog and documentation

Move the current `[Unreleased]` entries into a dated release section and leave
a new, empty `[Unreleased]` section above it:

```markdown
## [Unreleased]

## [X.Y.Z] - YYYY-MM-DD

### Changed

- ...
```

Add or update the link definitions at the bottom of `CHANGELOG.md`:

```markdown
[Unreleased]: https://github.com/sog4be/sakurs/compare/vX.Y.Z...HEAD
[X.Y.Z]: https://github.com/sog4be/sakurs/releases/tag/vX.Y.Z
```

Use the actual release date. Update README files, API examples, compatibility
notices, and other version-specific documentation in the same release PR. The
release workflow requires a non-empty `## [X.Y.Z]` changelog section and uses
that section as the GitHub Release body.

### 3. Run release verification

Run the repository's mandatory checks:

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo check --workspace
```

Verify the packages without depending on an unpublished `sakurs-core` version:

```bash
cargo package --locked -p sakurs-core
CORE_PATH="$PWD/sakurs-core"
cargo package --locked -p sakurs-cli \
  --config "patch.crates-io.sakurs-core.path='$CORE_PATH'"
```

Cargo removes the CLI's local `path` entry when it creates the publishable
manifest. Without the command-local patch, even `cargo package --no-verify`
then attempts to resolve `sakurs-core X.Y.Z` from crates.io and fails before
that version has been published. The patch lets Cargo fully build-verify the
CLI archive against the same checkout. It does not modify the source manifest
or the archive: the packaged dependency remains the exact `X.Y.Z` registry
requirement. The core archive is independently build-verified by the preceding
command.

Verify that locally built artifacts report the exact release version:

```bash
VERSION=X.Y.Z

test "$(cargo run --quiet -p sakurs-cli -- --version)" = "sakurs $VERSION"

cd sakurs-py
uv sync --extra test --no-install-project --locked
uv run --no-sync maturin build --release --features extension-module -o dist
WHEEL_FILE=$(find dist -type f -name "sakurs-${VERSION}-*.whl" -print -quit)
test -n "$WHEEL_FILE"
uv pip install --python .venv/bin/python --force-reinstall "$WHEEL_FILE"
EXPECTED_VERSION="$VERSION" .venv/bin/python -c \
  'import os, sakurs; assert sakurs.__version__ == os.environ["EXPECTED_VERSION"]'
.venv/bin/python -m pytest tests/
cd ..
```

### 4. Create and merge the release PR

Commit the release preparation using the repository's conventional commit
format, push the branch, and create a PR using every section of
`.github/PULL_REQUEST_TEMPLATE.md`:

```bash
git status --short
git diff
git add -p
git diff --cached
git status --short
git commit -m "chore: release vX.Y.Z"
git push origin chore/release-vX.Y.Z
```

The PR should summarize user-visible changes, breaking compatibility changes,
security fixes, package verification, and documentation updates. Wait for the
required review and all CI checks before merging.

### 5. Tag the merged release commit

After the release PR is merged, update local `main` and verify that the version
and changelog at `HEAD` are the intended release state:

```bash
git switch main
git pull --ff-only origin main
git status --short
git log -1 --oneline
grep '^version = ' Cargo.toml
```

Create and push an annotated tag on that exact commit:

```bash
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push origin vX.Y.Z
```

Do not move or recreate a release tag after any registry has accepted a
package. The tag identifies the immutable source used for all published
artifacts.

### 6. Monitor the automated release

The `Release` workflow will:

1. Validate the tag, workspace versions, lockfile, and matching changelog section.
2. Run formatting, Clippy, Rust tests, and a compilation check.
3. In parallel, build-verify the `sakurs-core` and `sakurs-cli` packages and
   build all four `cp310-abi3` wheels: Linux x86_64, Windows x86_64, macOS
   x86_64, and macOS ARM64. The CLI verification uses a command-local patch for
   the as-yet-unpublished core. The Intel macOS wheel is cross-compiled on an
   ARM runner.
4. Publish `sakurs-core` to crates.io after every preflight and wheel build passes.
5. Publish `sakurs-cli` to crates.io after the core publish succeeds.
6. Pause for a maintainer to approve the protected `pypi` environment.
7. Publish the prebuilt wheels to PyPI through OIDC.
8. Create the GitHub Release only after every package publisher succeeds.

Monitor the run at <https://github.com/sog4be/sakurs/actions>. Approve the
`pypi` deployment from the workflow's **Review deployments** prompt when it is
ready.

### 7. Verify the published release

Check the registry and release pages:

- <https://crates.io/crates/sakurs-core>
- <https://crates.io/crates/sakurs-cli>
- <https://pypi.org/project/sakurs/>
- <https://github.com/sog4be/sakurs/releases>

Then install the exact version from the public registries in a clean temporary
environment and run small smoke tests:

```bash
VERSION=X.Y.Z
SMOKE_ROOT=$(mktemp -d)

python3 -m venv "$SMOKE_ROOT/venv"
"$SMOKE_ROOT/venv/bin/pip" install "sakurs==$VERSION"
EXPECTED_VERSION="$VERSION" "$SMOKE_ROOT/venv/bin/python" -c \
  'import os, sakurs; assert sakurs.__version__ == os.environ["EXPECTED_VERSION"]; assert sakurs.split("One. Two.") == ["One.", "Two."]'

cargo install sakurs-cli --version "$VERSION" --locked --root "$SMOKE_ROOT/cargo"
"$SMOKE_ROOT/cargo/bin/sakurs" --version
printf 'One. Two.\n' | "$SMOKE_ROOT/cargo/bin/sakurs" process -i -
```

Keep the workspace version at the released value until the next release PR;
this repository does not use a development-version suffix.

## Troubleshooting

### A publish job fails

Prefer **Re-run failed jobs** on the existing tag workflow. The workflow is
designed to resume safely: crates.io publishing detects an already-published
matching version, and the PyPI upload uses `skip-existing`. Allow time for the
crates.io index to expose a newly published core before retrying a CLI failure.

If the failure is caused by an expired `CARGO_REGISTRY_TOKEN`, rotate the
repository secret and re-run the failed jobs. If it is an OIDC error, verify
the PyPI trusted publisher's repository, workflow, and environment names, then
re-run the failed jobs.

Do not delete or move the tag to pick up a workflow fix after a partial
publication. Registry releases are immutable; stop and assess the published
state before considering any manual recovery or a patch release.

### `upload-pypi` is waiting

This is expected while the protected `pypi` environment awaits maintainer
approval. Approve the pending deployment in the Actions UI. No registry
credential is needed because the upload uses OIDC.

### A version already exists

Do not overwrite or reuse it. Confirm which artifacts were published, complete
an interrupted workflow through failed-job retry where possible, or prepare a
new patch version.

## Release Checklist

### Release PR

- [ ] Latest `main` CI is green
- [ ] Target version is unused on GitHub, crates.io, and PyPI
- [ ] Root workspace version updated
- [ ] `sakurs-cli` dependency on `sakurs-core` updated
- [ ] `Cargo.lock` refreshed
- [ ] Dated changelog section added with a new empty `[Unreleased]` section
- [ ] Version-specific README and API documentation updated
- [ ] Mandatory Rust checks pass
- [ ] `sakurs-core` package build-verification passes with `--locked`
- [ ] `sakurs-cli` package build-verification passes with the command-local
      core patch
- [ ] Local CLI and installed wheel report exactly `X.Y.Z`
- [ ] Python tests pass against the installed wheel
- [ ] Release PR approved, CI-green, and merged

### Tag workflow

- [ ] Latest release merge commit checked out on `main`
- [ ] Annotated `vX.Y.Z` tag pushed once
- [ ] Validation, tests, core, CLI, and all wheel builds succeed
- [ ] Protected `pypi` deployment manually approved
- [ ] PyPI upload succeeds
- [ ] GitHub Release is created with the intended changelog

### Published artifacts

- [ ] `sakurs-core==X.Y.Z` is available on crates.io
- [ ] `sakurs-cli==X.Y.Z` is available on crates.io
- [ ] `sakurs==X.Y.Z` and all four wheels are available on PyPI
- [ ] Clean, version-pinned Python smoke test passes
- [ ] Clean, version-pinned CLI smoke test passes
