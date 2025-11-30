# Development Guide

## Prerequisites

### Required Tools
```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Cross-compilation targets
rustup target add aarch64-unknown-linux-gnu
rustup target add armv7-unknown-linux-gnueabihf

# Additional tools
cargo install cargo-watch
cargo install cross
```

### System Dependencies (Ubuntu/Debian)
```bash
sudo apt update
sudo apt install -y \
    build-essential \
    gcc-aarch64-linux-gnu \
    g++-aarch64-linux-gnu \
    qemu-user-static \
    pkg-config \
    libssl-dev
```

## Project Structure

```
rexos/
├── core/               # Core Rust modules
│   ├── hal/           # Hardware Abstraction Layer
│   ├── input/         # Input handling
│   ├── display/       # Display management
│   ├── audio/         # Audio system
│   ├── storage/       # Storage operations
│   └── power/         # Power management
├── services/          # System services
└── tools/             # Development tools
```

## Building

### Development Build (x86_64)
```bash
# Build all workspace members
cargo build

# Build specific package
cargo build -p rexos-hal

# Run tests
cargo test

# Watch mode (auto-rebuild on changes)
cargo watch -x build
```

### Cross-Compilation (ARM64)
```bash
# Using cross for easier cross-compilation
cross build --target aarch64-unknown-linux-gnu --release

# Or with standard cargo
cargo build --target aarch64-unknown-linux-gnu --release
```

### Full System Image
```bash
# Coming soon - will use Buildroot
cd tools/buildroot
make anbernic_defconfig
make
```

## Development Workflow

### 1. Setup Development Environment
```bash
git clone https://github.com/santosr2/rexos.git
cd rexos
cargo build
```

### 2. Create a New Module
```bash
# Create new library in core/
cargo new --lib core/mymodule

# Add to workspace in root Cargo.toml
# members = [..., "core/mymodule"]
```

### 3. Testing

#### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        assert_eq!(2 + 2, 4);
    }
}
```

#### Integration Tests
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### 4. Hardware Testing
```bash
# Deploy to device via SSH
./tools/deploy.sh <device-ip>

# Or manual deployment
scp target/aarch64-unknown-linux-gnu/release/binary device:/tmp/
ssh device "/tmp/binary"
```

## Code Style

### Rust Guidelines
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for formatting: `cargo fmt`
- Use `clippy` for linting: `cargo clippy`

### Example
```rust
use anyhow::{Context, Result};
use tracing::{info, error};

pub struct MyComponent {
    config: Config,
}

impl MyComponent {
    pub fn new(config: Config) -> Result<Self> {
        info!("Initializing component");
        Ok(Self { config })
    }

    pub fn do_something(&self) -> Result<()> {
        self.internal_operation()
            .context("Failed to do something")?;
        Ok(())
    }

    fn internal_operation(&self) -> Result<()> {
        // Implementation
        Ok(())
    }
}
```

## Debugging

### Enable Debug Logging
```bash
RUST_LOG=debug cargo run
```

### Using GDB (cross-debugging)
```bash
# On device
gdbserver :2345 ./binary

# On host
gdb-multiarch target/aarch64-unknown-linux-gnu/debug/binary
(gdb) target remote device:2345
```

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make changes and test
4. Run formatting: `cargo fmt`
5. Run linting: `cargo clippy`
6. Commit: `git commit -am 'Add my feature'`
7. Push: `git push origin feature/my-feature`
8. Create Pull Request

## Performance Profiling

```bash
# CPU profiling with perf
perf record -g ./binary
perf report

# Memory profiling with valgrind
valgrind --leak-check=full ./binary
```

## Useful Commands

```bash
# Check for outdated dependencies
cargo outdated

# Security audit
cargo audit

# Generate documentation
cargo doc --open

# Clean build artifacts
cargo clean

# Update dependencies
cargo update
```

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust Embedded Book](https://rust-embedded.github.io/book/)
- [Linux Device Drivers](https://lwn.net/Kernel/LDD3/)
- [ARM Architecture Reference](https://developer.arm.com/documentation)
