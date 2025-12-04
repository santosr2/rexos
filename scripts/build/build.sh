#!/bin/bash
# RexOS Build Script
# Builds all RexOS components for target devices

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/build"
DIST_DIR="$PROJECT_ROOT/dist"

# Default target
TARGET="${TARGET:-aarch64-unknown-linux-gnu}"
PROFILE="${PROFILE:-release}"

# Cross-compilation toolchain prefix
case "$TARGET" in
    aarch64-unknown-linux-gnu)
        CROSS_COMPILE="${CROSS_COMPILE:-aarch64-linux-gnu-}"
        ;;
    armv7-unknown-linux-gnueabihf)
        CROSS_COMPILE="${CROSS_COMPILE:-arm-linux-gnueabihf-}"
        ;;
    *)
        CROSS_COMPILE=""
        ;;
esac

log() {
    echo "[BUILD] $1"
}

error() {
    echo "[ERROR] $1" >&2
    exit 1
}

# Check dependencies
check_deps() {
    log "Checking build dependencies..."

    local deps=("cargo" "rustc")
    local missing=()

    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &>/dev/null; then
            missing+=("$dep")
        fi
    done

    if [ ${#missing[@]} -gt 0 ]; then
        error "Missing dependencies: ${missing[*]}"
    fi

    # Check for cross-compilation tools if needed
    if [ -n "$CROSS_COMPILE" ]; then
        if ! command -v "${CROSS_COMPILE}gcc" &>/dev/null; then
            log "Warning: Cross-compiler ${CROSS_COMPILE}gcc not found"
            log "Trying to use 'cross' tool instead..."

            if ! command -v cross &>/dev/null; then
                error "Neither cross-compiler nor 'cross' tool found. Install one of them."
            fi
        fi
    fi

    log "Dependencies OK"
}

# Clean build directory
clean() {
    log "Cleaning build directory..."
    rm -rf "$BUILD_DIR"
    rm -rf "$DIST_DIR"
    cargo clean --manifest-path "$PROJECT_ROOT/Cargo.toml" 2>/dev/null || true
    log "Clean complete"
}

# Build Rust components
build_rust() {
    log "Building Rust components for $TARGET ($PROFILE)..."

    local cargo_args=()

    if [ "$PROFILE" = "release" ]; then
        cargo_args+=("--release")
    fi

    # Use cross if available, otherwise cargo
    if command -v cross &>/dev/null && [ -n "$CROSS_COMPILE" ]; then
        cross build "${cargo_args[@]}" --target "$TARGET" \
            --manifest-path "$PROJECT_ROOT/Cargo.toml"
    else
        cargo build "${cargo_args[@]}" --target "$TARGET" \
            --manifest-path "$PROJECT_ROOT/Cargo.toml"
    fi

    log "Rust build complete"
}

# Build C components
build_c() {
    log "Building C components..."

    local c_bridge="$PROJECT_ROOT/c/emulator-bridge"

    if [ -d "$c_bridge" ]; then
        make -C "$c_bridge" clean 2>/dev/null || true
        make -C "$c_bridge" \
            CROSS_COMPILE="$CROSS_COMPILE" \
            BUILD_DIR="$BUILD_DIR/c/emulator-bridge"
    fi

    log "C build complete"
}

# Collect build artifacts
collect_artifacts() {
    log "Collecting build artifacts..."

    mkdir -p "$DIST_DIR/rexos/bin"
    mkdir -p "$DIST_DIR/rexos/lib"
    mkdir -p "$DIST_DIR/rexos/scripts"
    mkdir -p "$DIST_DIR/rexos/config"

    # Copy Rust binaries
    local rust_target_dir="$PROJECT_ROOT/target/$TARGET/$PROFILE"

    for bin in rexos-init rexos-launcher; do
        if [ -f "$rust_target_dir/$bin" ]; then
            cp "$rust_target_dir/$bin" "$DIST_DIR/rexos/bin/"
            log "  Copied: $bin"
        fi
    done

    # Copy C libraries
    if [ -f "$BUILD_DIR/c/emulator-bridge/librexos_bridge.so" ]; then
        cp "$BUILD_DIR/c/emulator-bridge/librexos_bridge.so" "$DIST_DIR/rexos/lib/"
        log "  Copied: librexos_bridge.so"
    fi

    if [ -f "$BUILD_DIR/c/emulator-bridge/librexos_bridge.a" ]; then
        cp "$BUILD_DIR/c/emulator-bridge/librexos_bridge.a" "$DIST_DIR/rexos/lib/"
        log "  Copied: librexos_bridge.a"
    fi

    # Copy scripts
    cp -r "$PROJECT_ROOT/scripts"/* "$DIST_DIR/rexos/scripts/" 2>/dev/null || true
    find "$DIST_DIR/rexos/scripts" -name "*.sh" -exec chmod +x {} \;

    # Copy default configs
    if [ -d "$PROJECT_ROOT/config" ]; then
        cp -r "$PROJECT_ROOT/config"/* "$DIST_DIR/rexos/config/" 2>/dev/null || true
    fi

    # Create version file
    local version=$(grep -m1 '^version' "$PROJECT_ROOT/Cargo.toml" | cut -d'"' -f2)
    local git_hash=$(git -C "$PROJECT_ROOT" rev-parse --short HEAD 2>/dev/null || echo "unknown")

    echo "$version-$git_hash" > "$DIST_DIR/rexos/version"

    log "Artifacts collected"
}

# Create distribution package
create_package() {
    log "Creating distribution package..."

    local version=$(cat "$DIST_DIR/rexos/version")
    local package_name="rexos-${version}-${TARGET}.tar.gz"

    tar -czf "$DIST_DIR/$package_name" -C "$DIST_DIR" rexos

    log "Package created: $DIST_DIR/$package_name"

    # Create checksum
    (cd "$DIST_DIR" && sha256sum "$package_name" > "${package_name}.sha256")

    log "Checksum created: $DIST_DIR/${package_name}.sha256"
}

# Run tests
run_tests() {
    log "Running tests..."

    cargo test --manifest-path "$PROJECT_ROOT/Cargo.toml" --all

    log "Tests complete"
}

# Run lints
run_lint() {
    log "Running lints..."

    cargo fmt --manifest-path "$PROJECT_ROOT/Cargo.toml" --all -- --check
    cargo clippy --manifest-path "$PROJECT_ROOT/Cargo.toml" --all -- -D warnings

    log "Lint complete"
}

# Full build
full_build() {
    check_deps
    build_rust
    build_c
    collect_artifacts
    create_package

    log ""
    log "=========================================="
    log "Build complete!"
    log "=========================================="
    log "Target: $TARGET"
    log "Profile: $PROFILE"
    log "Output: $DIST_DIR"
    log ""
}

# Usage
usage() {
    echo "Usage: $0 [command] [options]"
    echo ""
    echo "Commands:"
    echo "  all         Full build (default)"
    echo "  rust        Build Rust components only"
    echo "  c           Build C components only"
    echo "  package     Create distribution package"
    echo "  test        Run tests"
    echo "  lint        Run lints"
    echo "  clean       Clean build artifacts"
    echo ""
    echo "Environment variables:"
    echo "  TARGET      Target triple (default: aarch64-unknown-linux-gnu)"
    echo "  PROFILE     Build profile: debug|release (default: release)"
    echo ""
    echo "Supported targets:"
    echo "  aarch64-unknown-linux-gnu      RG353 series (64-bit ARM)"
    echo "  armv7-unknown-linux-gnueabihf  RG35XX series (32-bit ARM)"
    echo ""
    echo "Examples:"
    echo "  $0 all"
    echo "  TARGET=armv7-unknown-linux-gnueabihf $0 all"
    echo "  PROFILE=debug $0 rust"
}

main() {
    local command="${1:-all}"

    case "$command" in
        all)
            full_build
            ;;
        rust)
            check_deps
            build_rust
            ;;
        c)
            build_c
            ;;
        package)
            collect_artifacts
            create_package
            ;;
        test)
            run_tests
            ;;
        lint)
            run_lint
            ;;
        clean)
            clean
            ;;
        -h|--help|help)
            usage
            ;;
        *)
            usage
            exit 1
            ;;
    esac
}

main "$@"
