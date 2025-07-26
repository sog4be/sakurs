# Releasing sakurs

This document describes the release process for the sakurs project.

## Overview

The release process is semi-automated:
- **Manual**: Version updates and tag creation
- **Automated**: Package publishing and GitHub release creation

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
   - crates.io: `sakurs-cli`
   - PyPI: `sakurs`

4. **Check GitHub Secrets**:
   - `CARGO_REGISTRY_TOKEN` must be set

5. **Configure PyPI OIDC** (first release only):
   - Go to PyPI project settings
   - Add trusted publisher for GitHub Actions
   - Repository: `sog4be/sakurs`
   - Workflow: `.github/workflows/release.yml`
   - Environment: `pypi`

## Release Steps

### 1. Update Version Numbers

Update version in all Cargo.toml files:
```bash
# In workspace root
sed -i '' 's/version = ".*-dev"/version = "X.Y.Z"/' Cargo.toml

# In each crate
sed -i '' 's/version = ".*-dev"/version = "X.Y.Z"/' sakurs-core/Cargo.toml
sed -i '' 's/version = ".*-dev"/version = "X.Y.Z"/' sakurs-cli/Cargo.toml
sed -i '' 's/version = ".*-dev"/version = "X.Y.Z"/' sakurs-py/Cargo.toml
```

### 2. Final Checks

```bash
# Verify versions match
grep -h "^version = " */Cargo.toml Cargo.toml

# Run final tests
cargo test --workspace
cargo publish --dry-run -p sakurs-cli
```

### 3. Commit and Tag

```bash
# Commit version changes
git add -A
git commit -m "chore: release v$VERSION"

# Create annotated tag
git tag -a v$VERSION -m "Release v$VERSION"

# Push to trigger release
git push origin main
git push origin v$VERSION
```

### 4. Monitor Release

The GitHub Actions workflow will automatically:
1. Validate the tag format and version consistency
2. Publish `sakurs-cli` to crates.io
3. Build Python wheels for multiple platforms
4. Upload wheels to PyPI as `sakurs`
5. Create a GitHub release with changelog

Monitor the progress at: https://github.com/sog4be/sakurs/actions

### 5. Post-Release

After successful release:

1. **Update to next development version**:
   ```bash
   # Update all versions to next dev version
   sed -i '' 's/version = "X.Y.Z"/version = "X.Y.Z-dev"/' */Cargo.toml Cargo.toml
   git add -A
   git commit -m "chore: prepare for next development iteration"
   git push origin main
   ```

2. **Verify packages**:
   - Check https://crates.io/crates/sakurs-cli
   - Check https://pypi.org/project/sakurs/
   - Test installation: `pip install sakurs` and `cargo install sakurs-cli`

## Troubleshooting

### crates.io Publishing Fails

- **Authentication error**: Verify `CARGO_REGISTRY_TOKEN` is set correctly
- **Version exists**: Version was already published, bump version number
- **Dependencies**: Ensure all path dependencies are removed

### PyPI Publishing Fails

- **OIDC error**: Check trusted publisher configuration
- **Version exists**: Version already published, PyPI doesn't allow overwrites
- **Wheel building**: Check maturin and PyO3 compatibility

### Partial Release

If some packages fail to publish:
- The GitHub Release will still be created
- Manually publish failed packages:
  ```bash
  # For crates.io
  cd sakurs-cli && cargo publish
  
  # For PyPI (build wheels first)
  cd sakurs-py
  maturin build --release
  twine upload target/wheels/*
  ```

## Release Checklist

- [ ] All CI checks passing
- [ ] CHANGELOG.md updated
- [ ] Version numbers updated in all Cargo.toml files
- [ ] Dry-run publish successful
- [ ] Tag created and pushed
- [ ] Monitor GitHub Actions workflow
- [ ] Verify packages on crates.io and PyPI
- [ ] Update to next dev version
- [ ] Announce release (if applicable)