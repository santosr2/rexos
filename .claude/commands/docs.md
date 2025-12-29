---
description: Generate and open Rust documentation
allowed-tools: Bash(cargo:*), Read
---

Generate documentation for all RexOS crates:

```bash
cargo doc --no-deps --workspace --all-features 2>&1
echo ""
echo "Documentation generated at: target/doc/"
echo "Open: target/doc/rexos_hal/index.html"
```
