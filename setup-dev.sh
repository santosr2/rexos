#!/usr/bin/env fish
# Quick start script for RexOS development

echo "🎮 RexOS (Retro Experience OS) Development Setup"
echo "================================"
echo ""

# Check for Rust
if not type -q rustc
    echo "❌ Rust is not installed!"
    echo "Install from: https://rustup.rs/"
    exit 1
end

echo "✓ Rust installed: "(rustc --version)

# Check for required targets
set TARGET_ARM64 "aarch64-unknown-linux-gnu"
set TARGET_ARM32 "armv7-unknown-linux-gnueabihf"

echo ""
echo "Checking cross-compilation targets..."

if not rustup target list --installed | grep -q $TARGET_ARM64
    echo "Installing $TARGET_ARM64..."
    rustup target add $TARGET_ARM64
else
    echo "✓ $TARGET_ARM64 installed"
end

if not rustup target list --installed | grep -q $TARGET_ARM32
    echo "Installing $TARGET_ARM32..."
    rustup target add $TARGET_ARM32
else
    echo "✓ $TARGET_ARM32 installed"
end

# Check for cargo-watch
if not type -q cargo-watch
    echo ""
    echo "Installing cargo-watch for development..."
    cargo install cargo-watch
end

# Check for cross (optional but recommended)
if not type -q cross
    echo ""
    echo "📦 Optional: Install 'cross' for easier cross-compilation"
    echo "   cargo install cross"
end

echo ""
echo "✅ RexOS development environment is ready!"
echo ""
echo "Quick commands:"
echo "  cargo build              # Build for host (development)"
echo "  cargo test               # Run tests"
echo "  cargo watch -x build     # Auto-rebuild on changes"
echo "  cross build --target aarch64-unknown-linux-gnu --release"
echo ""
echo "Next steps:"
echo "  1. Read docs/DEVELOPMENT.md for detailed guide"
echo "  2. Explore core/hal for the hardware abstraction layer"
echo "  3. Check docs/FEATURES.md for planned features"
echo ""
echo "Happy coding! 🚀"
