# RexOS Architecture

## System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                        User Interface                        │
│              (EmulationStation / Custom Frontend)            │
└───────────────────────────┬─────────────────────────────────┘
                            │
┌───────────────────────────┴─────────────────────────────────┐
│                     System Services (Rust)                   │
├──────────────┬──────────────┬──────────────┬────────────────┤
│   Library    │   Emulator   │    Update    │    Network     │
│   Manager    │   Launcher   │    Service   │    Services    │
└──────┬───────┴──────┬───────┴──────┬───────┴────────┬───────┘
       │              │              │                │
┌──────┴──────────────┴──────────────┴────────────────┴───────┐
│                  Core System (Rust)                          │
├──────────┬──────────┬──────────┬──────────┬─────────────────┤
│   HAL    │  Input   │ Display  │  Audio   │ Storage│ Power  │
└──────┬───┴──────┬───┴──────┬───┴──────┬───┴────┬───┴────┬───┘
       │          │          │          │        │        │
┌──────┴──────────┴──────────┴──────────┴────────┴────────┴───┐
│                    Linux Kernel                               │
│              (Custom minimal configuration)                   │
└──────────────────────────────────────────────────────────────┘
       │                                  │
┌──────┴───────┐                  ┌──────┴───────┐
│  Bootloader  │                  │   Hardware   │
│   (U-Boot)   │                  │   (Devices)  │
└──────────────┘                  └──────────────┘
```

## Boot Sequence

1. **U-Boot** (< 2s)
   - Hardware initialization
   - Kernel loading
   - Boot parameter passing

2. **Linux Kernel** (< 3s)
   - Device drivers initialization
   - Minimal services

3. **Core System** (< 2s)
   - HAL initialization
   - Input/display/audio setup
   - Mount storage

4. **System Services** (< 2s)
   - Library indexing
   - Network setup
   - Background services

5. **UI Launch** (< 1s)
   - EmulationStation or frontend
   - Theme loading
   - Ready for user

**Total Target: < 10 seconds**

## Data Flow

### Game Launch Sequence
```
User Selection (UI)
    ↓
Library Service (identify game)
    ↓
Emulator Service (select core/emulator)
    ↓
Core System (setup environment)
    ↓
Launch Emulator Process
    ↓
Monitor & Handle Exit
    ↓
Return to UI
```

## Key Design Decisions

### Why Rust for Core?
- **Memory Safety**: No segfaults or undefined behavior
- **Performance**: Zero-cost abstractions, as fast as C
- **Concurrency**: Safe concurrent programming
- **Modern**: Great tooling and package management

### Modular Architecture
- Each component is independent
- Clear interfaces between layers
- Easy to test and replace components
- Supports multiple frontends

### Hybrid Approach Benefits
1. **Reusability**: Leverage existing emulators (C/C++)
2. **Flexibility**: Shell scripts for quick tasks
3. **Safety**: Rust for critical system components
4. **Performance**: Right tool for each job

## Storage Layout

```
/boot/                    # Boot partition (FAT32)
├── uboot/               # U-Boot files
└── kernel/              # Kernel images

/                         # Root filesystem (ext4)
├── usr/
│   ├── bin/             # System binaries
│   └── lib/             # System libraries
├── etc/
│   └── anbernic/        # Configuration files
└── opt/
    ├── emulators/       # Emulator binaries
    └── cores/           # RetroArch cores

/roms/                   # Games partition (exFAT)
├── gb/
├── gba/
├── n64/
├── psx/
└── ...

/userdata/               # User data partition (ext4)
├── saves/              # Save states
├── config/             # User configs
├── screenshots/        # Screenshots
└── library.db          # Game library database
```

## Configuration System

All configuration in TOML format:
- `/etc/anbernic/system.toml` - System configuration
- `/etc/anbernic/devices/` - Device-specific configs
- `/userdata/config/` - User preferences

## Inter-Process Communication

- **D-Bus**: Service communication
- **Unix Sockets**: Fast local IPC
- **Shared Memory**: Performance-critical data

## Update System

```
Check Updates (daily)
    ↓
Download Package
    ↓
Verify Signature
    ↓
Create Backup Point
    ↓
Apply Update
    ↓
Verify System
    ↓
Reboot (if needed)
    ↓
Rollback if Failed
```

## Security Considerations

1. **Read-only Root**: System partition mounted read-only
2. **Signed Updates**: Cryptographically verified updates
3. **Sandboxing**: Emulators run with limited privileges
4. **Secure Boot**: Optional for advanced users
