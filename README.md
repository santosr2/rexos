# RexOS - Retro Experience Operating System

> A modern, lightweight operating system for handheld retro gaming devices, inspired by ArkOS but built with Rust for performance, safety, and maintainability.

**RexOS** = **R**etro **Ex**perience **OS** - Delivering the ultimate retro gaming experience on handheld devices.

## 🎮 Project Vision

RexOS aims to provide a streamlined, performant, and user-friendly operating system for handheld gaming devices like Anbernic RG series, with a focus on:
- Fast boot times (< 10 seconds)
- Efficient resource management
- Easy game library management
- Seamless emulator integration
- Modern development practices

## 🏗️ Architecture

### Core Components

**Language Distribution:**

- **Rust (60%)**: Core system, hardware abstraction, game library manager
- **Shell (30%)**: System scripts, updates, maintenance tools  
- **C (10%)**: Emulator bridges, hardware-specific optimizations

### Technology Stack

- **Build System**: Buildroot (minimal, purpose-built)
- **Bootloader**: U-Boot (optimized)
- **Kernel**: Linux 5.10 (Rockchip BSP, custom config)
- **C Library**: glibc (for emulator compatibility)
- **Init**: Custom `rexos-init` (fast boot, no systemd)
- **Core Utilities**: BusyBox (~1MB vs 50MB for coreutils)
- **Core System**: Rust-based system services
- **Frontend**: Custom Rust TUI launcher
- **Emulators**: RetroArch + standalone emulators
- **Root FS**: SquashFS (read-only, ~50MB compressed)

### Target Specifications

| Metric | Target |
|--------|--------|
| Root filesystem | < 100MB |
| Boot time | < 5 seconds |
| RAM usage (idle) | < 100MB |
| Power consumption | Optimized for battery |

## ✨ Key Features

### Phase 1 (MVP) - Completed

- [x] Project structure
- [x] Boot system (< 10 second boot)
- [x] Hardware abstraction layer (HAL) in Rust
- [x] Basic input handling (buttons, analog sticks)
- [x] Display/framebuffer management
- [x] Audio system integration
- [x] Game library scanner with SQLite database
- [x] RetroArch core management
- [x] Save state management
- [x] Init system and TUI launcher

### Phase 2 (Enhanced Features) - Completed

- [x] WiFi/Bluetooth management
- [x] Over-the-air (OTA) updates with signature verification
- [x] C emulator bridge for RetroArch integration
- [x] Performance profiles (battery/performance modes)
- [x] Hotkey system
- [x] Storage management (mount, partition, watch)
- [ ] Cloud save sync
- [ ] Scrapers (game metadata/artwork)
- [ ] Theme engine
- [ ] Screenshot/video recording
- [ ] Achievement system (RetroAchievements)

### Phase 3 (Advanced Features)

- [ ] Port management system (similar to PortMaster)
- [ ] Shader management
- [ ] Netplay support
- [ ] Remote play
- [ ] Custom overlay system
- [ ] Multi-language support
- [ ] Accessibility features
- [ ] Advanced power management
- [ ] BIOS/firmware manager
- [ ] Automatic ROM organization

## 📁 Project Structure

```text
rexos/
├── core/                 # Rust core system components
│   ├── hal/             # Hardware Abstraction Layer
│   ├── input/           # Input management
│   ├── display/         # Display/GPU management
│   ├── audio/           # Audio system
│   ├── storage/         # Storage & filesystem
│   └── power/           # Power management
├── services/            # System services (Rust)
│   ├── library/         # Game library manager
│   ├── emulator/        # Emulator launcher/manager
│   ├── update/          # Update system
│   └── network/         # Network services
├── ui/                  # User interface
│   ├── frontend/        # Main UI (EmulationStation or custom)
│   └── themes/          # Theme support
├── scripts/             # Shell scripts
│   ├── install/         # Installation scripts
│   ├── update/          # Update scripts
│   └── maintenance/     # System maintenance
├── emulators/           # Emulator integration
│   ├── retroarch/       # RetroArch integration
│   └── standalone/      # Standalone emulators
├── tools/               # Development & build tools
│   ├── buildroot/       # Buildroot configuration
│   └── deploy/          # Deployment tools
├── docs/                # Documentation
│   ├── dev/             # Developer guides
│   ├── user/            # User manuals
│   └── api/             # API documentation
└── tests/               # Test suites
    ├── unit/            # Unit tests
    ├── integration/     # Integration tests
    └── hardware/        # Hardware-specific tests
```

## 🎯 Supported Devices (Planned)

### Initial Target

- **Anbernic RG353M/V/VS**: RK3566 chipset
- **Anbernic RG35XX series**: ARM-based devices

### Future Support

- Anbernic RG351 series
- Anbernic RG552
- Other similar ARM-based handhelds

## 🚀 Getting Started

### Prerequisites

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Cross-compilation tools
rustup target add aarch64-unknown-linux-gnu
rustup target add armv7-unknown-linux-gnueabihf

# Build dependencies
# (Will be documented as project develops)
```

### Building

```bash
# Clone repository
git clone https://github.com/santosr2/rexos.git
cd rexos

# Build all components (native)
./scripts/build/build.sh all

# Build for ARM64 (RG353 series)
TARGET=aarch64-unknown-linux-gnu ./scripts/build/build.sh all

# Build for ARM32 (RG35XX series)
TARGET=armv7-unknown-linux-gnueabihf ./scripts/build/build.sh all

# Build C emulator bridge only
make -C c/emulator-bridge

# Run tests
cargo test --all

# Create distribution package
./scripts/build/build.sh package
```

### Installation

1. Download the appropriate image for your device from releases
2. Flash to an SD card:
   ```bash
   sudo ./scripts/build/flash-image.sh flash rexos-<version>.img.gz /dev/sdX
   ```
3. Insert the SD card and power on your device

## 🤝 Contributing

We welcome contributions! Areas where help is needed:

- Rust systems programming
- Linux kernel customization
- Hardware driver development
- EmulationStation theming
- Documentation
- Testing on various devices

## 📝 Development Principles

1. **Safety First**: Leverage Rust's safety guarantees
2. **Performance**: Optimize for battery life and responsiveness
3. **Modularity**: Clear separation of concerns
4. **User-Friendly**: Simple for users, powerful for developers
5. **Open**: Transparent development and community-driven

## 🔧 Technical Decisions

### Why Rust?

- Memory safety without garbage collection
- Zero-cost abstractions
- Excellent embedded systems support
- Modern tooling and package management
- Growing embedded/gaming community

### Why Hybrid Approach?

- Reuse proven shell script patterns from ArkOS
- Leverage existing C-based emulators
- Gradual migration path
- Practical for hardware interfacing

## 📚 Resources & References

- [ArkOS Project](https://github.com/christianhaitian/arkos) - Inspiration
- [RetroArch](https://www.retroarch.com/) - Emulation frontend
- [EmulationStation](https://emulationstation.org/) - UI framework
- [Rust Embedded Book](https://rust-embedded.github.io/book/)

## 📄 License

This project will be licensed under MIT License (TBD - to be decided with community input).

## 🗺️ Roadmap

**Q1 2025**: Project setup, core architecture, HAL development
**Q2 2025**: Basic boot system, EmulationStation integration
**Q3 2025**: First alpha release for RG353 series
**Q4 2025**: Beta release with full feature set

---

**Status**: 🟡 Active Development - Core functionality implemented

**Join us**: [Discord/Matrix community link coming soon]

## 📊 Project Metrics

| Component | Status | Coverage |
|-----------|--------|----------|
| Core HAL | ✅ Complete | 80% |
| Storage | ✅ Complete | 75% |
| Config | ✅ Complete | 85% |
| Emulator Service | ✅ Complete | 70% |
| Library Service | ✅ Complete | 75% |
| Update Service | ✅ Complete | 80% |
| Network Service | ✅ Complete | 75% |
| C Bridge | ✅ Complete | N/A |
| Shell Scripts | ✅ Complete | N/A |
| CI/CD | ✅ Complete | N/A |
