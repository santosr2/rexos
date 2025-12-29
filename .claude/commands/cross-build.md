---
description: Cross-compile RexOS for ARM target devices
allowed-tools: Bash(cross:*), Bash(cargo:*), Read
argument-hint: [arm64|arm32|all]
---

Cross-compile RexOS for target handheld devices.

Target options:
- `arm64` or `aarch64`: Build for RG353M/V/VS (RK3566)
- `arm32` or `armv7`: Build for RG35XX series
- `all`: Build for all targets

```bash
case "${1:-all}" in
  arm64|aarch64)
    echo "Building for aarch64 (RG353 series)..."
    cross build --target aarch64-unknown-linux-gnu --release 2>&1
    ;;
  arm32|armv7)
    echo "Building for armv7 (RG35XX series)..."
    cross build --target armv7-unknown-linux-gnueabihf --release 2>&1
    ;;
  all|*)
    echo "Building for all ARM targets..."
    cross build --target aarch64-unknown-linux-gnu --release 2>&1
    cross build --target armv7-unknown-linux-gnueabihf --release 2>&1
    ;;
esac
```

Report build status and binary locations.
