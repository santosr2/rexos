# Emulator Bridge

C-based bridge for launching and managing emulators in RexOS.

## Purpose

This component provides a high-performance interface between RexOS services and emulator processes:
- RetroArch core management
- Standalone emulator launching
- Process monitoring and control
- Performance profiling

## Building

```bash
make
```

## Files

- `launcher.h` - Public API for emulator launching
- `launcher.c` - Implementation
- `retroarch.c` - RetroArch-specific integration
- `process.c` - Process management utilities

## API

### launch_emulator
```c
int launch_emulator(const char* core_path, const char* rom_path);
```
Launches an emulator with specified core and ROM.

**Returns**: 0 on success, -1 on error

### monitor_emulator
```c
int monitor_emulator(pid_t pid);
```
Monitors emulator process and returns exit code.

## Integration

Used by `services/emulator/` Rust service via FFI.
