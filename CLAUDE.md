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

# Run tests
cargo test

# Run specific test with output
cargo test test_name -- --nocapture

# Format code
cargo fmt

# Lint code
cargo clippy

# Cross-compile for ARM64 target devices
cross build --target aarch64-unknown-linux-gnu --release

# Build C emulator bridge
cd c/emulator-bridge && make
```

## Architecture

```text
UI Layer (EmulationStation)
    ↓
System Services (Rust) - Library, Emulator, Update, Network
    ↓
Core System (Rust) - HAL, Input, Display, Audio, Storage, Power
    ↓
Linux Kernel
    ↓
Hardware (Anbernic devices)
```

### Core Components (`core/`)

The Hardware Abstraction Layer (`core/hal/`) is the only implemented module. It provides:

- `Device` - device auto-detection and profile management
- `Display` - framebuffer/GPU management
- `InputManager` - buttons and analog sticks
- `AudioManager` - audio system
- `PowerManager` - battery and power states

Other core modules (input, display, audio, storage, power) are planned but not yet implemented in the workspace.

### Workspace Structure

The root `Cargo.toml` defines a workspace with shared dependencies. Currently only `core/hal` is active. New modules should:

1. Be created with `cargo new --lib core/modulename`
2. Be added to the `members` array in root `Cargo.toml`
3. Use `workspace = true` for common dependencies

### Error Handling Pattern

Use `anyhow` for application errors and `thiserror` for library errors:

```rust
use anyhow::{Context, Result};

pub fn do_something() -> Result<()> {
    self.internal_operation()
        .context("Failed to do something")?;
    Ok(())
}
```

### Logging

Use `tracing` for all logging:

```rust
use tracing::{info, error, debug};
```

Enable debug logging: `RUST_LOG=debug cargo run`

## Target Devices

- Primary: Anbernic RG353M/V/VS (RK3566 chipset, aarch64)
- Secondary: Anbernic RG35XX series (armv7)

Cross-compilation targets:

- `aarch64-unknown-linux-gnu`
- `armv7-unknown-linux-gnueabihf`

## Commit Message Convention

```text
<type>(<scope>): <subject>

Types: feat, fix, docs, style, refactor, perf, test, chore
Example: feat(hal): add support for RG353V analog sticks
```
