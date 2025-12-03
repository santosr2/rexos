# RexOS Project Structure

This document describes the folder structure following best practices for Rust workspaces, C projects, and shell scripts.

## Directory Layout

```
rexos/
├── .claude/                    # Claude Code configuration
├── .github/                    # GitHub Actions, templates
│   └── workflows/              # CI/CD pipelines
├── packaging/                  # OS packaging and distribution
│   ├── buildroot/              # Buildroot external tree
│   │   ├── board/              # Board-specific configs
│   │   ├── configs/            # Defconfig files
│   │   └── packages/           # Custom packages
│   └── cross/                  # Cross-compilation configs
├── config/                     # Runtime configuration files
│   ├── defaults/               # Default configs shipped with OS
│   └── retroarch/              # RetroArch configuration
├── crates/                     # Rust workspace members
│   ├── rexos-config/           # Configuration management
│   ├── rexos-emulator/         # Emulator launcher service
│   ├── rexos-hal/              # Hardware Abstraction Layer
│   ├── rexos-init/             # Init system binary
│   ├── rexos-launcher/         # TUI launcher binary
│   ├── rexos-library/          # Game library service
│   ├── rexos-network/          # Network management
│   ├── rexos-storage/          # Storage management
│   └── rexos-update/           # OTA update service
├── docs/                       # Documentation
│   ├── api/                    # API documentation
│   ├── dev/                    # Developer guides
│   └── user/                   # User documentation
├── ffi/                        # Foreign Function Interface (C code)
│   └── emulator-bridge/        # C bridge for emulators
│       ├── include/            # Public headers
│       ├── src/                # Implementation
│       └── Makefile
├── scripts/                    # Shell scripts
│   ├── build/                  # Build automation
│   ├── dev/                    # Development helpers
│   ├── install/                # Installation scripts
│   └── runtime/                # Runtime scripts (shipped with OS)
│       ├── maintenance/        # Backup, cleanup, update
│       ├── system/             # Startup, shutdown
│       └── utils/              # User utilities
├── tests/                      # Integration tests
│   └── integration/
├── Cargo.toml                  # Workspace manifest
├── Cargo.lock                  # Dependency lock file
├── Cross.toml                  # Cross-compilation config
├── mise.toml                   # Tool management
├── LICENSE
├── README.md
├── CHANGELOG.md
├── CONTRIBUTING.md
└── VERSION
```

## Key Directories

### `/crates/` - Rust Workspace
All Rust crates in a single `crates/` directory (Rust convention for workspaces):
- **Libraries** (`rexos-*`): Reusable components
- **Binaries** (`rexos-init`, `rexos-launcher`): Executables

### `/ffi/` - C/C++ Code
Foreign Function Interface code follows C project conventions:
- `include/` - Public header files
- `src/` - Implementation files
- `Makefile` - Build system

### `/packaging/` - OS Packaging

All packaging and distribution configuration:

- Buildroot external tree for OS image creation
- Cross-compilation configs
- Board-specific files

### `/scripts/` - Shell Scripts

Organized by purpose:

- `build/` - Build automation (not shipped)
- `dev/` - Development helpers (not shipped)
- `install/` - One-time installation
- `runtime/` - Shipped with OS, run at runtime

### `/config/` - Configuration

Runtime configuration files:

- `defaults/` - Default configs
- Per-application configs

## Naming Conventions

### Rust

- Crate names: `rexos-{component}` (kebab-case)
- Module files: `snake_case.rs`
- Binary names: `rexos-{name}`

### C

- Source files: `snake_case.c`
- Header files: `snake_case.h`
- Public headers in `include/`

### Shell Scripts

- File names: `kebab-case.sh`
- Executable scripts have `#!/usr/bin/env bash` shebang
- All scripts pass `shellcheck`

### Configuration

- TOML for Rust configs
- INI for RetroArch
- Shell for environment
