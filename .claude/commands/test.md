---
description: Run tests for RexOS (optionally specify a crate)
allowed-tools: Bash(cargo:*), Read
argument-hint: [crate-name]
---

Run tests for RexOS. If a crate name is provided, test only that crate.

```bash
if [ -n "$1" ]; then
  cargo nextest run -p "$1" --all-features 2>&1
else
  cargo nextest run --workspace --all-features 2>&1
fi
```

Analyze any test failures and suggest fixes.
