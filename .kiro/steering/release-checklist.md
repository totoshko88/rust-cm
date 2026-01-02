---
inclusion: manual
---

# Release Checklist

When preparing a release (creating tags, merging release branches), verify ALL packaging files are updated.

## Version Files to Update

For release version `X.Y.Z`, update these files:

### 1. Cargo.toml (workspace)
```toml
version = "X.Y.Z"
```

### 2. CHANGELOG.md
- Add new section `## [X.Y.Z] - YYYY-MM-DD`
- List all changes under appropriate categories (Added, Changed, Fixed, etc.)
- Add version link at bottom of file

### 3. debian/changelog
```
rustconn (X.Y.Z-1) unstable; urgency=medium

  * Release X.Y.Z
  * [list changes]

 -- Author <email>  Day, DD Mon YYYY HH:MM:SS +ZZZZ
```

### 4. Flatpak Manifests (BOTH files!)
Update `tag:` in both:
- `packaging/flatpak/io.github.totoshko88.RustConn.yml`
- `packaging/flathub/io.github.totoshko88.RustConn.yml`

```yaml
sources:
  - type: git
    url: https://github.com/totoshko88/RustConn.git
    tag: vX.Y.Z
```

### 5. AppImage (if version in script)
Check `packaging/appimage/` for version references.

### 6. OBS (openSUSE Build Service)
Check `packaging/obs/` for version references in spec files.

## Pre-Release Verification

```bash
# 1. All tests pass
cargo test

# 2. Format check
cargo fmt --check

# 3. Clippy check
cargo clippy --all-targets

# 4. Release build succeeds
cargo build --release
```

## Release Workflow

1. Create release branch: `git checkout -b X.Y.Z`
2. Update all version files listed above
3. Commit: `git commit -m "Release X.Y.Z: [summary]"`
4. Switch to main: `git checkout main`
5. Merge: `git merge X.Y.Z --no-ff -m "Merge branch 'X.Y.Z' for release vX.Y.Z"`
6. Tag: `git tag -a vX.Y.Z -m "Release vX.Y.Z"`
7. Push: `git push origin main && git push origin vX.Y.Z`

## Common Mistakes to Avoid

- ❌ Forgetting to update Flatpak manifests (causes CI failure)
- ❌ Forgetting `cargo fmt` before commit (causes CI failure)
- ❌ Mismatched versions between Cargo.toml and packaging files
- ❌ Missing CHANGELOG entry for the release
- ❌ debian/changelog with wrong date format
