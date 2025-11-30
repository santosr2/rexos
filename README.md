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

- **Bootloader**: U-Boot (optimized)
- **Kernel**: Linux (custom minimal build)
- **Core System**: Rust-based system services
- **Frontend**: EmulationStation (fork) or custom Rust TUI
- **Emulators**: RetroArch, standalone emulators
- **Build System**: Buildroot/Yocto

## ✨ Key Features

### Phase 1 (MVP)
- [x] Project structure
- [ ] Boot system (< 10 second boot)
- [ ] Hardware abstraction layer (HAL) in Rust
- [ ] Basic input handling (buttons, analog sticks)
- [ ] Display/framebuffer management
- [ ] Audio system integration
- [ ] Game library scanner
- [ ] EmulationStation integration
- [ ] RetroArch core management
- [ ] Save state management
- [ ] Basic settings UI

### Phase 2 (Enhanced Features)
- [ ] WiFi/Bluetooth management
- [ ] Over-the-air (OTA) updates
- [ ] Cloud save sync
- [ ] Scrapers (game metadata/artwork)
- [ ] Theme engine
- [ ] Performance profiles (battery/performance modes)
- [ ] Screenshot/video recording
- [ ] Achievement system (RetroAchievements)
- [ ] Sleep/suspend optimization

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

```
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

### Building (Coming Soon)
```bash
# Clone repository
git clone https://github.com/santosr2/rexos.git
cd rexos

# Build core system
cd core
cargo build --release --target aarch64-unknown-linux-gnu

# Build full OS image (requires buildroot setup)
# Documentation coming soon
```

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

**Status**: 🚧 Early Development - Not Ready for Use

**Join us**: [Discord/Matrix community link coming soon]
