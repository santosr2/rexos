# Quick Start Guide

Welcome to RexOS! This guide will get you up and running quickly.

## For Users

⚠️ **Currently in early development** - Not ready for end users yet. Check back later!

## For Developers

### Prerequisites

- Rust 1.70 or newer
- Basic familiarity with systems programming
- (Optional) An Anbernic handheld device for testing

### Setup Development Environment

1. **Clone the repository**
   ```bash
   git clone https://github.com/santosr2/rexos.git
   cd rexos
   ```

2. **Run the setup script**
   ```bash
   ./setup-dev.sh
   ```

3. **Build the project**
   ```bash
   cargo build
   ```

4. **Run tests**
   ```bash
   cargo test
   ```

### Project Structure Quick Tour

```
rexos/
├── core/hal/          # Hardware Abstraction Layer (start here!)
├── services/          # System services (coming soon)
├── docs/              # Documentation
│   ├── ARCHITECTURE.md    # System architecture
│   ├── DEVELOPMENT.md     # Development guide
│   └── FEATURES.md        # Feature specifications
└── tools/             # Build and deployment tools
```

### Your First Contribution

1. **Pick a task** from our Issues or check `docs/FEATURES.md`
2. **Create a branch**: `git checkout -b feature/my-feature`
3. **Make changes** and test
4. **Submit a PR** - we'll review and help!

### Key Resources

- **Architecture**: See `docs/ARCHITECTURE.md`
- **Features**: See `docs/FEATURES.md`
- **Development**: See `docs/DEVELOPMENT.md`
- **Contributing**: See `CONTRIBUTING.md`

### Example: Using the HAL

```rust
use rexos_hal::Device;

fn main() -> anyhow::Result<()> {
    // Auto-detect device
    let device = Device::detect()?;

    println!("Detected: {}", device.profile().name);
    println!("Chipset: {}", device.profile().chipset);
    println!("Resolution: {}x{}",
        device.profile().display.width,
        device.profile().display.height
    );

    Ok(())
}
```

### Development Workflow

```bash
# Watch mode (rebuilds on file changes)
cargo watch -x build

# Run specific tests
cargo test device

# Format code
cargo fmt

# Lint code
cargo clippy

# Build documentation
cargo doc --open
```

### Cross-Compilation

```bash
# For ARM64 devices (RG353, etc.)
cross build --target aarch64-unknown-linux-gnu --release

# For ARM32 devices
cross build --target armv7-unknown-linux-gnueabihf --release
```

### Need Help?

- **Read the docs** in the `docs/` folder
- **Open an issue** with questions
- **Join our community** (links coming soon)

### What to Work On?

Check these areas:
- 🔧 Hardware support for new devices
- 🎮 Emulator integration
- 📚 Game library management
- 🎨 UI/Frontend work
- 📝 Documentation improvements
- 🧪 Testing and bug reports

## Roadmap

**Phase 1 (Current)**: Core architecture and HAL development
**Phase 2**: System services and emulator integration
**Phase 3**: Full OS build and first release

---

**Status**: 🚧 Early Development (December 2025)

Thank you for your interest in Anbernic OS! 🎮
