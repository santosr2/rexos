# RexOS Architecture Guide

This document describes the overall architecture of RexOS, a Rust-based operating system for handheld retro gaming devices.

## System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                        User Interface                        │
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │  TUI Launcher   │  │ EmulationStation│  │   Settings   │ │
│  └────────┬────────┘  └────────┬────────┘  └──────┬───────┘ │
└───────────┼─────────────────────┼─────────────────┼─────────┘
            │                     │                 │
┌───────────▼─────────────────────▼─────────────────▼─────────┐
│                      System Services                         │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌────────┐ │
│  │ Library │ │ Emulator│ │ Update  │ │ Network │ │ Config │ │
│  └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘ └───┬────┘ │
└───────┼───────────┼───────────┼───────────┼──────────┼──────┘
        │           │           │           │          │
┌───────▼───────────▼───────────▼───────────▼──────────▼──────┐
│                     Core System (Rust)                       │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌────────┐ │
│  │   HAL   │ │ Storage │ │  Input  │ │ Display │ │  Audio │ │
│  └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘ └───┬────┘ │
└───────┼───────────┼───────────┼───────────┼──────────┼──────┘
        │           │           │           │          │
┌───────▼───────────▼───────────▼───────────▼──────────▼──────┐
│                       Linux Kernel                           │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌────────┐ │
│  │  Sysfs  │ │  Input  │ │   DRM   │ │  ALSA   │ │  MMC   │ │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └────────┘ │
└─────────────────────────────────────────────────────────────┘
        │           │           │           │          │
┌───────▼───────────▼───────────▼───────────▼──────────▼──────┐
│                        Hardware                              │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌────────┐ │
│  │   SoC   │ │ Buttons │ │   LCD   │ │ Speaker │ │ SD Card│ │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Component Details

### Core Layer (`core/`)

#### Hardware Abstraction Layer (`core/hal`)

The HAL provides device-agnostic interfaces to hardware:

- **Device Detection**: Auto-detection of device model (RG353, RG35XX, etc.)
- **Device Profiles**: Hardware-specific configurations
- **Display**: Framebuffer and GPU management
- **Input**: Button and analog stick handling
- **Audio**: Sound system integration
- **Power**: Battery monitoring and power states

```rust
use rexos_hal::{Device, DeviceProfile};

let device = Device::detect()?;
println!("Running on: {}", device.profile().name);
```

#### Storage (`core/storage`)

File system and storage management:

- **Mount Manager**: Mount/unmount external storage
- **Partition Manager**: Partition detection and management
- **Watcher**: File system change monitoring

#### Configuration (`core/config`)

System configuration management:

- **Device Profiles**: Hardware-specific settings
- **Emulator Config**: Per-core and per-game settings
- **Hotkeys**: Button combination mappings
- **System Config**: Global system settings

### Services Layer (`services/`)

#### Library Service (`services/library`)

Game library management:

- **Scanner**: ROM file discovery and indexing
- **Database**: SQLite-based game database
- **Metadata**: Game information management

```rust
use rexos_library::{Library, ScanOptions};

let library = Library::open("/rexos/data/library.db")?;
library.scan("/rexos/roms", ScanOptions::default()).await?;
```

#### Emulator Service (`services/emulator`)

Emulator orchestration:

- **Launcher**: Start/stop emulators
- **RetroArch**: RetroArch integration and configuration
- **Standalone**: Support for standalone emulators

#### Update Service (`services/update`)

Over-the-air update system:

- **Checker**: Version comparison and update detection
- **Downloader**: Resumable downloads with progress
- **Installer**: Safe update installation with rollback
- **Verification**: Ed25519 signatures and SHA256 hashes

#### Network Service (`services/network`)

Network connectivity management:

- **WiFi**: wpa_supplicant integration
- **Bluetooth**: bluetoothctl wrapper
- **Hotspot**: Access point mode

### C Bridge (`c/emulator-bridge`)

Low-level emulator integration:

- **Performance Monitoring**: CPU, GPU, memory, battery stats
- **Input Remapping**: Button mapping and virtual devices
- **Audio Bridge**: ALSA integration
- **RetroArch Hooks**: Hotkey handling and display control

### Binary Applications (`bin/`)

#### rexos-init

PID 1 init system:

1. Mount essential filesystems (proc, sys, dev)
2. Initialize hardware (display, input, audio)
3. Start system services
4. Launch frontend application

#### rexos-launcher

TUI game browser:

- System and game navigation
- Game launching
- Settings access

## Data Flow

### Game Launch Sequence

```
User selects game
        │
        ▼
┌───────────────┐
│ TUI Launcher  │
└───────┬───────┘
        │ Request game launch
        ▼
┌───────────────┐
│Emulator Service│
└───────┬───────┘
        │ Determine core/emulator
        ▼
┌───────────────┐
│ Config Service│
└───────┬───────┘
        │ Load game config
        ▼
┌───────────────┐
│ C Bridge      │──────────────────┐
└───────┬───────┘                  │
        │ Execute RetroArch        │ Monitor performance
        ▼                          │
┌───────────────┐                  │
│  RetroArch    │◄─────────────────┘
└───────────────┘
```

### Update Sequence

```
┌───────────────┐
│ Update Checker│
└───────┬───────┘
        │ Check for updates
        ▼
┌───────────────┐
│ Update Server │
└───────┬───────┘
        │ Manifest + package URL
        ▼
┌───────────────┐
│  Downloader   │
└───────┬───────┘
        │ Download package
        ▼
┌───────────────┐
│  Verifier     │
└───────┬───────┘
        │ Verify signature
        ▼
┌───────────────┐
│  Installer    │
└───────┬───────┘
        │ Create backup, apply update
        ▼
┌───────────────┐
│   Reboot      │
└───────────────┘
```

## Error Handling

RexOS uses a consistent error handling pattern across all modules:

```rust
use anyhow::{Context, Result};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("configuration not found: {0}")]
    ConfigNotFound(String),

    #[error("device not supported: {0}")]
    UnsupportedDevice(String),
}

pub fn my_function() -> Result<()> {
    do_something()
        .context("Failed to do something")?;
    Ok(())
}
```

## Logging

All modules use the `tracing` crate:

```rust
use tracing::{info, warn, error, debug};

info!("Starting RexOS");
debug!(device = %device_name, "Device detected");
error!(?err, "Failed to initialize display");
```

Enable debug logging:

```bash
RUST_LOG=debug ./rexos-launcher
```

## Cross-Compilation

RexOS supports two target architectures:

| Target | Devices | Toolchain |
|--------|---------|-----------|
| `aarch64-unknown-linux-gnu` | RG353 series | `aarch64-linux-gnu-` |
| `armv7-unknown-linux-gnueabihf` | RG35XX series | `arm-linux-gnueabihf-` |

Using `cross`:

```bash
cargo install cross
cross build --release --target aarch64-unknown-linux-gnu
```

## Testing

### Unit Tests

```bash
cargo test --all
```

### Integration Tests

```bash
cargo test --all -- --ignored
```

### Hardware Tests

Hardware-specific tests require the actual device or QEMU emulation.
