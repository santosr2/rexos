---
description: Run all linters (clippy, fmt check, pre-commit)
allowed-tools: Bash(cargo:*), Bash(pre-commit:*), Read
---

Run comprehensive linting on the RexOS codebase:

1. Check Rust formatting:
```bash
cargo fmt --all -- --check 2>&1
```

2. Run Clippy with strict warnings:
```bash
cargo clippy --all-targets --all-features -- -D warnings 2>&1
```

3. Run pre-commit hooks:
```bash
pre-commit run --all-files 2>&1 | head -100
```

Report any issues found and suggest fixes.
