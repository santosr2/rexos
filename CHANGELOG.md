# Changelog

All notable changes to RexOS will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project structure with Rust workspace
- Core Hardware Abstraction Layer (HAL) for device detection, display, input, audio, power
- Storage management with mount, partition, and file watcher support
- Configuration system with device profiles, emulator config, hotkeys, system settings
- Emulator service with RetroArch and standalone emulator support
- Game library service with SQLite database, ROM scanner, and metadata management
- Update service with OTA updates, signature verification, and rollback support
- Network service with WiFi, Bluetooth, and hotspot management
- C emulator bridge for RetroArch integration (performance monitoring, input remapping, audio)
- TUI game launcher application
- Custom init system for fast boot
- Shell scripts for system management (install, update, maintenance, utils)
- Buildroot external tree for minimal OS builds
- GitHub Actions CI/CD pipelines
- Pre-commit hooks for code quality
- Comprehensive documentation

### Target Devices
- Anbernic RG353M/V/VS (RK3566)
- Anbernic RG35XX series

## [0.1.0] - TBD

### Added
- First alpha release
- Basic boot to launcher functionality
- RetroArch integration
- Game library scanning
- WiFi connectivity
- Basic settings management

---

## Release Notes Format

### Types of Changes
- **Added** for new features
- **Changed** for changes in existing functionality
- **Deprecated** for soon-to-be removed features
- **Removed** for now removed features
- **Fixed** for any bug fixes
- **Security** for vulnerability fixes

[Unreleased]: https://github.com/santosr2/rexos/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/santosr2/rexos/releases/tag/v0.1.0
