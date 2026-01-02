---
inclusion: manual
---

# Pre-Commit Checklist

Before making ANY git commit, ALWAYS run these checks:

## Required Checks

```bash
# 1. Format check (MUST pass)
cargo fmt --check

# 2. Clippy lint check (MUST pass)
cargo clippy --all-targets

# 3. If format fails, fix it:
cargo fmt
```

## Commit Rules

1. **Never commit without passing `cargo fmt --check`**
2. **Never commit without passing `cargo clippy --all-targets`**
3. If either check fails, fix the issues before committing
4. After fixing, re-run both checks to confirm

## Quick Pre-Commit Command

Run this single command before every commit:

```bash
cargo fmt --check && cargo clippy --all-targets
```

If it passes with no output, you're ready to commit.
