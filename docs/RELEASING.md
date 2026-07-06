# Releasing sakurs

This document describes the release process for the sakurs project.

## Overview

The release process follows a PR-based workflow:
- **Release PR**: Version updates and changelog modifications
- **Tag Creation**: After PR merge, create tag on main branch
- **Automated Publishing**: GitHub Actions handles package publishing and release creation

## Prerequisites

Before starting a release:

1. **Ensure all tests pass**:
   ```bash
   cargo test --workspace
   cargo fmt --all -- --check
   cargo clippy --workspace -- -D warnings
   ```

2. **Update CHANGELOG.md**:
   - Add a new section for the version
   - Follow the format: `## [X.Y.Z] - YYYY-MM-DD`
   - Include all notable changes

3. **Verify package names**:
   - crates.io: `sakurs-core` (internal library), `sakurs-cli` (user-facing tool)
   - PyPI: `sakurs`
   
   Note: sakurs-core is published as a dependency for sakurs-cli but is not 
   intended for direct use due to unstable APIs.

4. **Check GitHub Secrets**:
   - `CARGO_REGISTRY_TOKEN` must be set

5. **Configure PyPI OIDC** (first release only):
   - Go to PyPI project settings
   - Add trusted publisher for GitHub Actions
   - Repository: `sog4be/sakurs`
   - Workflow: `.github/workflows/release.yml`
   - Environment: `pypi`

## Release Steps

### 1. Create Release PR

Create a new branch and update the version number:
```bash
# Create release branch
git checkout -b chore/release-vX.Y.Z

# The version is managed once in the workspace root Cargo.toml
# ([workspace.package] version); all three crates inherit it via
# `version.workspace = true`, and sakurs-py's pyproject.toml reads it
# dynamically through maturin. Edit the single line:
# macOS:
sed -i '' 's/^version = ".*"$/version = "X.Y.Z"/' Cargo.toml
# Linux: use sed -i without ''

# Refresh Cargo.lock with the new version
cargo check --workspace
```

### 2. Final Checks

```bash
# Verify the workspace version
grep "^version = " Cargo.toml

# Run final tests
cargo test --workspace
cargo publish --dry-run -p sakurs-core
cargo publish --dry-run -p sakurs-cli

# Update CHANGELOG.md if not already done
# Ensure the new version section is at the top
```

### 3. Create and Merge PR

```bash
# Commit version changes
git add -A
git commit -m "chore: release vX.Y.Z

- Update version from X.Y.Z-dev to X.Y.Z
- Update CHANGELOG.md for release"

# Push branch
git push origin chore/release-vX.Y.Z

# Create PR via GitHub
gh pr create --title "chore: release vX.Y.Z" \
  --body "Release vX.Y.Z with the following changes:
  
  - [List major changes from CHANGELOG]
  
  See CHANGELOG.md for full details."
```

Wait for PR approval and CI checks to pass, then merge.

### 4. Create Tag and Trigger Release

After the PR is merged:
```bash
# Switch to main and pull latest
git checkout main
git pull origin main

# Create annotated tag
git tag -a vX.Y.Z -m "Release vX.Y.Z"

# Push tag to trigger automated release
git push origin vX.Y.Z
```

### 5. Monitor Release

The GitHub Actions workflow will automatically:
1. Validate the tag format and version consistency
2. Run tests (formatting, clippy, and unit tests)
3. Publish `sakurs-core` to crates.io (as dependency)
4. Publish `sakurs-cli` to crates.io (user-facing tool)
5. Build Python wheels for multiple platforms
6. Upload wheels to PyPI as `sakurs`
7. Create a GitHub release with changelog

Monitor the progress at: https://github.com/sog4be/sakurs/actions

### 6. Post-Release

After successful release:

1. **Version stays at the released value** until the next release branch
   bumps it (this repository does not use `-dev` suffixes; the release
   workflow validates that the tag matches the workspace version, so an
   interim suffix would break tag validation).

2. **Verify packages**:
   - Check https://crates.io/crates/sakurs-core (verify it's published but not promoted)
   - Check https://crates.io/crates/sakurs-cli (main CLI tool)
   - Check https://pypi.org/project/sakurs/
   - Test installation: `pip install sakurs` and `cargo install sakurs-cli`

## Troubleshooting

### crates.io Publishing Fails

- **Authentication error**: Verify `CARGO_REGISTRY_TOKEN` is set correctly
- **Version exists**: Version was already published, bump version number
- **Dependencies**: Ensure all path dependencies are removed
- **sakurs-core not found**: If sakurs-cli fails, ensure sakurs-core was published first

### PyPI Publishing Fails

- **OIDC error**: Check trusted publisher configuration
- **Version exists**: Version already published, PyPI doesn't allow overwrites
- **Wheel building**: Check maturin and PyO3 compatibility

### Partial Release

If some packages fail to publish:
- The GitHub Release will still be created
- Manually publish failed packages:
  ```bash
  # For crates.io (publish in order)
  cd sakurs-core && cargo publish
  # Wait a few seconds for crates.io to index
  cd ../sakurs-cli && cargo publish
  
  # For PyPI (build wheels first)
  cd sakurs-py
  maturin build --release
  twine upload target/wheels/*
  ```

## Release Checklist

### Pre-Release PR
- [ ] All CI checks passing on main branch
- [ ] CHANGELOG.md updated with new version section
- [ ] Create release branch `chore/release-vX.Y.Z`
- [ ] Update version numbers in all Cargo.toml files
- [ ] Run `cargo test --workspace`
- [ ] Run `cargo publish --dry-run -p sakurs-core`
- [ ] Run `cargo publish --dry-run -p sakurs-cli`
- [ ] Create and merge release PR

### Release
- [ ] Pull latest main branch after PR merge
- [ ] Create annotated tag `vX.Y.Z`
- [ ] Push tag to trigger automated release
- [ ] Monitor GitHub Actions workflow
- [ ] Verify packages on crates.io and PyPI
- [ ] Verify GitHub Release was created

### Post-Release
- [ ] Create post-release PR to update documentation and add [Unreleased] section to CHANGELOG.md
- [ ] Merge post-release PR
- [ ] Announce release (if applicable)