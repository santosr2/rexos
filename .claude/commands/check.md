---
description: Quick compilation check without building
allowed-tools: Bash(cargo:*), Read
---

Run a quick compilation check on the workspace:

```bash
cargo check --workspace --all-features 2>&1
```

This is faster than a full build and catches most compilation errors.
