# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

RexOS is a Rust-based operating system for handheld retro gaming devices (Anbernic RG353M/V/VS, RG35XX series). It's a hybrid project: 60% Rust (core system), 30% shell scripts (maintenance), 10% C (emulator bridges).

## Build Commands

```bash
# Build all workspace members
cargo build

# Build specific package
cargo build -p rexos-hal

# Run tests with nextest (faster)
cargo nextest run --all-features

# Run tests with standard cargo
cargo test

# Run specific test with output
cargo test test_name -- --nocapture

# Format code
cargo fmt

# Lint code
cargo clippy --all-targets --all-features -- -D warnings

# Cross-compile for ARM64 target devices
cross build --target aarch64-unknown-linux-gnu --release

# Cross-compile for ARM32 target devices
cross build --target armv7-unknown-linux-gnueabihf --release

# Run pre-commit hooks
pre-commit run --all-files

# Run full CI locally
mise run ci
```

## Project Structure

```text
rexos/
├── crates/                    # Rust workspace crates
│   ├── rexos-hal/            # Hardware Abstraction Layer
│   ├── rexos-config/         # Configuration management
│   ├── rexos-storage/        # Storage and filesystem
│   ├── rexos-network/        # WiFi, Bluetooth, Hotspot
│   ├── rexos-update/         # OTA update system
│   ├── rexos-library/        # Game library management
│   ├── rexos-emulator/       # Emulator orchestration
│   ├── rexos-launcher/       # TUI launcher application
│   └── rexos-init/           # Init system (PID 1)
├── ffi/                       # C/FFI code
│   └── emulator-bridge/      # C bridge for emulators
├── scripts/                   # Shell scripts
│   ├── runtime/              # Runtime scripts (boot, install)
│   └── dev/                  # Development scripts
├── packaging/                 # Build/packaging configs
│   └── buildroot/            # Buildroot external tree
├── config/                    # Device configurations
├── docs/                      # Documentation
└── tests/                     # Integration tests
```

## Architecture

```text
┌─────────────────────────────────────────────┐
│            rexos-launcher (TUI)             │
├─────────────────────────────────────────────┤
│  rexos-library  │  rexos-emulator  │ update │
├─────────────────────────────────────────────┤
│      rexos-config  │  rexos-network         │
├─────────────────────────────────────────────┤
│    rexos-hal    │  rexos-storage            │
├─────────────────────────────────────────────┤
│              Linux Kernel                   │
├─────────────────────────────────────────────┤
│         Hardware (Anbernic devices)         │
└─────────────────────────────────────────────┘
```

## Workspace Crates

| Crate | Description |
|-------|-------------|
| `rexos-hal` | Hardware abstraction: Device detection, Display, Input, Audio, Power |
| `rexos-config` | Configuration management and device profiles |
| `rexos-storage` | Filesystem paths, ROM directories, save data |
| `rexos-network` | WiFi, Bluetooth, and hotspot management |
| `rexos-update` | OTA updates with signature verification |
| `rexos-library` | Game database, ROM scanning, metadata |
| `rexos-emulator` | RetroArch and standalone emulator management |
| `rexos-launcher` | TUI frontend using ratatui/crossterm |
| `rexos-init` | Minimal init system for device boot |

## Rust Edition & Version

- **Edition**: 2024 (latest)
- **MSRV**: 1.85
- Uses latest Rust 2024 features: if-let chains, let-else patterns

## Error Handling Pattern

Use `anyhow` for application errors and `thiserror` for library errors:

```rust
use anyhow::{Context, Result};

pub fn do_something() -> Result<()> {
    self.internal_operation()
        .context("Failed to do something")?;
    Ok(())
}
```

## Logging

Use `tracing` for all logging:

```rust
use tracing::{info, error, debug};
```

Enable debug logging: `RUST_LOG=debug cargo run`

## Target Devices

- **Primary**: Anbernic RG353M/V/VS (RK3566 chipset, aarch64)
- **Secondary**: Anbernic RG35XX series (armv7)

Cross-compilation targets:

- `aarch64-unknown-linux-gnu`
- `armv7-unknown-linux-gnueabihf`

## Code Style

### Rust 2024 Patterns

Use if-let chains for collapsible conditions:

```rust
// Good
if let Some(x) = opt && x > 0 {
    // ...
}

// Avoid
if let Some(x) = opt {
    if x > 0 {
        // ...
    }
}
```

Use `is_some_and`/`is_ok_and` instead of `map_or`:

```rust
// Good
opt.is_some_and(|x| x > 0)

// Avoid
opt.map_or(false, |x| x > 0)
```

### Shell Scripts

- Use `set -euo pipefail`
- Use functions for clarity
- POSIX compatible where possible

## Commit Message Convention

```text
<type>(<scope>): <subject>

Types: feat, fix, docs, style, refactor, perf, test, chore
Example: feat(hal): add support for RG353V analog sticks
```

## Custom Commands

Use `/help` to see available slash commands:

- `/build` - Build workspace
- `/test [crate]` - Run tests
- `/lint` - Run all linters
- `/cross-build [target]` - Cross-compile for ARM
- `/ci` - Run full CI pipeline locally
- `/security` - Run security audit
- `/docs` - Generate documentation
- `/new-crate <name>` - Create new workspace crate
