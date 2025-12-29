---
description: Build RexOS workspace (all crates)
allowed-tools: Bash(cargo:*), Read
---

Build the entire RexOS workspace and report results:

```bash
cargo build --workspace 2>&1
```

If the build fails, analyze the errors and suggest fixes.
