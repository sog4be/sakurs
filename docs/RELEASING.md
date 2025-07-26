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

### 1. Create Release PR

Create a new branch and update version numbers:
```bash
# Create release branch
git checkout -b chore/release-vX.Y.Z

# Update version in all Cargo.toml files
# In workspace root
# macOS:
sed -i '' 's/version = ".*-dev"/version = "X.Y.Z"/' Cargo.toml
# Linux:
# sed -i 's/version = ".*-dev"/version = "X.Y.Z"/' Cargo.toml

# In each crate
# macOS:
sed -i '' 's/version = ".*-dev"/version = "X.Y.Z"/' sakurs-core/Cargo.toml
sed -i '' 's/version = ".*-dev"/version = "X.Y.Z"/' sakurs-cli/Cargo.toml
sed -i '' 's/version = ".*-dev"/version = "X.Y.Z"/' sakurs-py/Cargo.toml
# Linux: use sed -i without ''
```

### 2. Final Checks

```bash
# Verify versions match
grep -h "^version = " */Cargo.toml Cargo.toml

# Run final tests
cargo test --workspace
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
3. Publish `sakurs-cli` to crates.io
4. Build Python wheels for multiple platforms
5. Upload wheels to PyPI as `sakurs`
6. Create a GitHub release with changelog

Monitor the progress at: https://github.com/sog4be/sakurs/actions

### 6. Post-Release

After successful release:

1. **Create post-release PR for next development version**:
   ```bash
   # Create new branch
   git checkout -b chore/prepare-next-dev
   
   # Update all versions to next dev version
   # macOS:
   sed -i '' 's/version = "X.Y.Z"/version = "X.Y.(Z+1)-dev"/' */Cargo.toml Cargo.toml
   # Linux:
   # sed -i 's/version = "X.Y.Z"/version = "X.Y.(Z+1)-dev"/' */Cargo.toml Cargo.toml
   
   # Commit and push
   git add -A
   git commit -m "chore: prepare for next development iteration
   
   - Bump version to X.Y.(Z+1)-dev"
   
   git push origin chore/prepare-next-dev
   
   # Create PR
   gh pr create --title "chore: prepare for next development iteration" \
     --body "Bump version to X.Y.(Z+1)-dev for continued development."
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

### Pre-Release PR
- [ ] All CI checks passing on main branch
- [ ] CHANGELOG.md updated with new version section
- [ ] Create release branch `chore/release-vX.Y.Z`
- [ ] Update version numbers in all Cargo.toml files
- [ ] Run `cargo test --workspace`
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
- [ ] Create post-release PR to bump to next dev version
- [ ] Merge post-release PR
- [ ] Announce release (if applicable)