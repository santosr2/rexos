# Core System Components

This directory contains the core Rust-based system components that form the foundation of RexOS.

## Modules

### `hal/` - Hardware Abstraction Layer
- Device detection and initialization
- GPIO management
- Display controller abstraction
- Audio codec interface
- Power management interface
- Button/joystick input abstraction

### `input/` - Input Management
- Input event handling
- Controller mapping
- Hotkey management
- Touch screen support (where applicable)
- Gamepad calibration

### `display/` - Display & GPU Management
- Framebuffer management
- Display mode switching
- Brightness control
- Screen rotation
- GPU acceleration interface

### `audio/` - Audio System
- ALSA integration
- Volume control
- Audio routing
- Mixer management

### `storage/` - Storage & Filesystem
- SD card management
- ROM scanning
- Save state handling
- Cache management

### `power/` - Power Management
- Battery monitoring
- Power profiles
- Suspend/resume
- Thermal management
