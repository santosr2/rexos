# System Services

Rust-based system services that run as daemons or background processes.

## Services

### `library/` - Game Library Manager
- ROM scanning and indexing
- Metadata management
- Artwork/thumbnail handling
- Collections and favorites
- Recently played tracking

### `emulator/` - Emulator Launcher & Manager
- Emulator process management
- Core selection
- Save state management
- Configuration per-game/per-system
- Performance monitoring

### `update/` - Update System
- OTA update checking
- Package verification
- Rollback capability
- Component updates (cores, BIOS, themes)

### `network/` - Network Services
- WiFi management
- Bluetooth support
- Samba/file sharing
- SSH server
- Web-based file manager
