# RexOS Roadmap

This document tracks the remaining work and planned features for RexOS.

## High Priority

### rexos-launcher (TUI Frontend)

- [ ] Implement `ui` module - UI components
- [ ] Implement `input` module - input handling
- [ ] Implement `state` module - application state management
- [ ] Settings screen implementation:
  - [ ] Brightness adjustment
  - [ ] Volume control
  - [ ] WiFi management UI
  - [ ] Bluetooth management UI
  - [ ] Updates interface

### rexos-network (WiFi & Bluetooth)

- [ ] Integrate wpa_supplicant interaction (`wpa_socket`, `wpa_config`)
- [ ] Implement Bluetooth HCI interface utilization
- [ ] WiFi connection management
- [ ] Bluetooth pairing workflow

## Medium Priority

### rexos-init (Boot System)

- [ ] Implement `services` module for service management
- [ ] Integrate shutdown module with main loop
- [ ] Implement SIGUSR2 signal handler
- [ ] Add proper signal handling for graceful shutdown

### rexos-update (OTA Updates)

- [ ] Integrate `HashVerifier` for update validation
- [ ] Implement `CertificateVerifier` for HTTPS pinning
- [ ] Wire up build tools (`generate_keypair`, `sign_data`)
- [ ] End-to-end update verification pipeline

### rexos-config (Configuration)

- [ ] Hotkey binding system
- [ ] Theme system
- [ ] Network configuration binding
- [ ] Per-emulator configuration UI

## Lower Priority

### rexos-library (Game Database)

- [ ] Integrate `parse_gamelist_xml()` for EmulationStation format
- [ ] Consider migration to quick-xml or roxmltree
- [ ] Metadata scraping integration

### rexos-storage (Storage Management)

- [ ] Secondary SD card auto-detection
- [ ] Hot-swap detection and handling
- [ ] Partition auto-detection improvements

### rexos-hal (Hardware Abstraction)

- [ ] Add support for newer Anbernic devices:
  - [ ] RG503S
  - [ ] RG35XX Plus
  - [ ] RG35XX H
  - [ ] RG40XX series
- [ ] Investigate support for non-Anbernic handhelds

### rexos-emulator (Emulator Management)

- [ ] Complete RetroArch core mappings:
  - [ ] Sega CD
  - [ ] Saturn
  - [ ] Dreamcast
  - [ ] Amiga
  - [ ] DOS
  - [ ] Arcade variants
- [ ] State loading/saving integration
- [ ] 32-bit architecture validation

## Completed

- [x] Project structure and workspace setup
- [x] Hardware abstraction layer (core functionality)
- [x] Configuration management framework
- [x] Storage and filesystem paths
- [x] Game library scanning
- [x] Emulator orchestration (basic)
- [x] TUI launcher (basic navigation)
- [x] Init system (core boot process)
- [x] Cross-compilation support (ARM64/ARM32)
- [x] Device detection for RG353 and RG35XX series
