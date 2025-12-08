# RexOS Testing Guide

This directory contains tools and configurations for testing RexOS without real hardware.

## Quick Start

```bash
# Run all unit tests (host machine)
cargo test

# Run tests with a specific mock device profile
REXOS_MOCK_DEVICE=rg353m cargo test

# Run tests with QEMU/Docker (ARM64 emulation)
./testing/qemu/run-qemu.sh

# Run native tests (Apple Silicon / aarch64 Linux only)
./testing/qemu/run-qemu.sh native
```

## Platform Support

| Platform | Native Tests | Emulated ARM64 Tests |
|----------|--------------|---------------------|
| macOS (Apple Silicon) | `./run-qemu.sh native` | Docker |
| macOS (Intel) | N/A | Docker |
| Linux (x86_64) | N/A | qemu-user-static |
| Linux (aarch64) | `./run-qemu.sh native` | N/A |

## Testing Methods

### 1. Host Testing with Mock HAL (Recommended for Development)

The mock HAL allows you to test all RexOS components on your development machine without emulation.

```rust
use rexos_hal::mock::{MockHal, MockProfile};

#[test]
fn test_display_brightness() {
    let mut hal = MockHal::new(MockProfile::Rg353m);

    hal.display.set_brightness(100).unwrap();
    assert_eq!(hal.display.get_brightness(), 100);
}

#[test]
fn test_input_simulation() {
    let hal = MockHal::new(MockProfile::Rg353m);

    // Simulate button press
    hal.input.press_button(Button::A);
    assert!(hal.input.is_pressed(Button::A));

    // Simulate analog stick
    hal.input.set_left_stick(10000, -5000);
    let state = hal.input.get_state();
    assert_eq!(state.left_stick.x, 10000);
}
```

### 2. Environment-Based Profile Selection

Set `REXOS_MOCK_DEVICE` to test device-specific behavior:

```bash
# Test as RG353M (default dual-analog device)
REXOS_MOCK_DEVICE=rg353m cargo test

# Test as RG35XX (no analog sticks)
REXOS_MOCK_DEVICE=rg35xx cargo test

# Test as RGB30 (square display)
REXOS_MOCK_DEVICE=rgb30 cargo test
```

Available profiles:
- `rg353m` - Anbernic RG353M (640x480, dual analog)
- `rg353v` - Anbernic RG353V (640x480, dual analog, touchscreen)
- `rg353vs` - Anbernic RG353VS (640x480, single analog)
- `rg35xx` - Anbernic RG35XX (640x480, no analog, no L2/R2)
- `rgb30` - Powkiddy RGB30 (720x720 square display)
- `rg503` - Anbernic RG503 (960x544 OLED)
- `rg351p` - Anbernic RG351P (480x320)
- `qemu_virt` - QEMU virtual machine
- `desktop` - Desktop development

### 3. ARM64 Testing (Platform-Aware)

The `run-qemu.sh` script automatically detects your platform and uses the best method:

```bash
# Show available options
./testing/qemu/run-qemu.sh help

# Run userspace tests (auto-detects platform)
./testing/qemu/run-qemu.sh userspace

# Run native tests (Apple Silicon / aarch64 only)
./testing/qemu/run-qemu.sh native

# Interactive ARM64 shell via Docker
./testing/qemu/run-qemu.sh shell
```

**macOS Setup:**

```bash
# Install Docker Desktop (required for ARM64 emulation on Intel)
brew install --cask docker

# Optional: Install cross for Rust cross-compilation
cargo install cross
```

**Linux Setup:**

```bash
# Ubuntu/Debian
sudo apt install qemu-user-static docker.io

# Arch
sudo pacman -S qemu-user-static docker
```

### 4. QEMU System Emulation

Boot a full RexOS image in QEMU (works on all platforms):

```bash
# Boot kernel only
./testing/qemu/run-qemu.sh kernel path/to/Image

# Boot full disk image
./testing/qemu/run-qemu.sh image path/to/rexos.img
```

### 5. Cross-Compilation Testing

```bash
# Install cross
cargo install cross

# Build for ARM64
cross build --target aarch64-unknown-linux-gnu

# Run tests on ARM64 (uses Docker)
cross test --target aarch64-unknown-linux-gnu
```

## Device Profiles

Custom device profiles are stored in `testing/profiles/` as TOML files:

```toml
# testing/profiles/custom.toml
id = "custom_device"
name = "My Custom Device"
chipset = "RK3566"
architecture = "aarch64"
analog_sticks = 2
battery_capacity = 4000

buttons = ["up", "down", "left", "right", "a", "b", "x", "y", "l1", "l2", "r1", "r2", "start", "select"]
quirks = ["custom_quirk"]

[display]
width = 800
height = 480
format = "RGB888"
refresh_rate = 60
```

Load a custom profile:

```rust
use rexos_hal::mock::MockDevice;
use std::path::Path;

let device = MockDevice::from_profile_file(Path::new("testing/profiles/custom.toml"))?;
```

## Test Categories

### Unit Tests

```bash
cargo test --lib
```

### Integration Tests

```bash
cargo test --test '*'
```

### Device-Specific Tests

```bash
# Test all profiles
for profile in rg353m rg35xx rgb30 rg503; do
    echo "Testing $profile..."
    REXOS_MOCK_DEVICE=$profile cargo test
done
```

## CI Integration

The mock HAL is used in CI to test without hardware:

```yaml
# .github/workflows/ci.yml
- name: Run tests
  run: cargo test
  env:
    REXOS_MOCK_DEVICE: qemu_virt
```

## Limitations

The mock HAL simulates hardware behavior but cannot replicate:

- Actual display rendering (use SDL2 for visual testing)
- Real audio output
- Physical button feel/response time
- Hardware-specific timing issues
- GPU acceleration

For final validation, test on real hardware.

## Troubleshooting

### QEMU not found

```bash
# macOS
brew install qemu

# Ubuntu
sudo apt install qemu-system-arm qemu-user-static
```

### Cross-compilation fails

```bash
# Install cross
cargo install cross

# Ensure Docker is running (cross uses Docker)
docker info
```

### Tests fail with "device not found"

Ensure you're using the mock HAL in tests:

```rust
#[cfg(test)]
fn get_hal() -> MockHal {
    MockHal::from_env()  // Uses REXOS_MOCK_DEVICE or defaults to Desktop
}
```
