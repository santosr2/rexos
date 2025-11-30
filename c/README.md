# C Components

This directory contains C-based components for RexOS, primarily used for:
- Hardware-specific optimizations
- Emulator bridges and integration
- Performance-critical operations
- Interfacing with existing C/C++ libraries

## Structure

```
c/
├── emulator-bridge/    # Bridges to emulators (RetroArch, etc.)
├── hardware/           # Hardware-specific drivers and utilities
├── lib/               # Shared C libraries
└── tools/             # Build and testing tools
```

## Why C?

C is used in RexOS for:

1. **Emulator Integration**: Most emulators are written in C/C++
2. **Hardware Access**: Direct hardware manipulation when needed
3. **Performance**: Hot paths that need maximum speed
4. **Legacy Compatibility**: Interfacing with existing libraries
5. **Small Footprint**: Minimal runtime overhead

## Guidelines

- Keep C code minimal and focused
- Provide Rust FFI bindings in `core/` modules
- Document memory management carefully
- Use modern C (C11/C17) features
- Run valgrind for memory leak detection

## Building C Components

```bash
# Build all C components
cd c
make all

# Build specific component
make emulator-bridge

# Run tests
make test

# Clean build artifacts
make clean
```

## Integration with Rust

C functions are exposed to Rust via FFI (Foreign Function Interface):

```c
// C side (emulator-bridge/launcher.h)
extern "C" {
    int launch_emulator(const char* core_path, const char* rom_path);
}
```

```rust
// Rust side (services/emulator/src/lib.rs)
extern "C" {
    fn launch_emulator(core_path: *const c_char, rom_path: *const c_char) -> c_int;
}
```

## Safety Considerations

- All C code must be wrapped in `unsafe` blocks in Rust
- Input validation happens on the Rust side
- Memory allocated in C must be freed in C
- Use `repr(C)` for structs passed across FFI boundary

## Current Components

### 🔧 Emulator Bridge (Planned)
- RetroArch core loader
- Standalone emulator launcher
- Process monitoring
- Save state management

### 🔧 Hardware Utilities (Planned)
- GPIO access helpers
- Display framebuffer manipulation
- Audio routing
- Power management hooks

## Example: Emulator Bridge

```c
// emulator-bridge/launcher.c
#include <stdio.h>
#include <stdlib.h>
#include "launcher.h"

int launch_emulator(const char* core_path, const char* rom_path) {
    // Validate inputs
    if (!core_path || !rom_path) {
        return -1;
    }
    
    // Launch emulator process
    // ... implementation ...
    
    return 0;
}
```

## Maintenance

- **Responsibility**: C components (10% of codebase)
- **Focus**: Performance, hardware access, emulator integration
- **Testing**: Unit tests + integration tests with Rust
- **Documentation**: Keep this README updated

## Resources

- [C11 Standard](https://en.cppreference.com/w/c/11)
- [Rust FFI Guide](https://doc.rust-lang.org/nomicon/ffi.html)
- [RetroArch API](https://docs.libretro.com/)
