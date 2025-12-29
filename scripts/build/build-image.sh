#!/bin/bash
# RexOS Full Image Build Script
# Builds a complete bootable SD card image using Buildroot
#
# Prerequisites:
#   - Linux host (or Docker/WSL2)
#   - ~15GB free disk space
#   - Internet connection for downloads
#
# Usage:
#   ./build-image.sh              # Full build
#   ./build-image.sh clean        # Clean and rebuild
#   ./build-image.sh menuconfig   # Configure buildroot
#   ./build-image.sh download     # Download sources only

set -euo pipefail

# Project paths
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BUILDROOT_EXT="$PROJECT_ROOT/packaging/buildroot"

# Buildroot version and paths
BUILDROOT_VERSION="${BUILDROOT_VERSION:-2024.02.1}"
BUILDROOT_URL="https://buildroot.org/downloads/buildroot-${BUILDROOT_VERSION}.tar.xz"
BUILD_BASE="${BUILD_BASE:-$PROJECT_ROOT/build}"
BUILDROOT_DIR="$BUILD_BASE/buildroot-${BUILDROOT_VERSION}"
OUTPUT_DIR="$BUILD_BASE/output"

# Target device (default: rg353)
DEVICE="${DEVICE:-rg353}"
DEFCONFIG="rexos_${DEVICE}_defconfig"

# Build options
PARALLEL_JOBS="${PARALLEL_JOBS:-$(nproc 2>/dev/null || echo 4)}"
DOWNLOAD_ONLY=false
CLEAN_BUILD=false
MENUCONFIG=false

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "${GREEN}[BUILD]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

log_step() {
    echo -e "${BLUE}==>${NC} $1"
}

# Check build dependencies
check_dependencies() {
    log_step "Checking build dependencies..."

    local deps=("wget" "tar" "make" "gcc" "g++" "patch" "perl" "python3" "unzip" "rsync" "cpio" "bc")
    local missing=()

    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &>/dev/null; then
            missing+=("$dep")
        fi
    done

    if [ ${#missing[@]} -gt 0 ]; then
        log_error "Missing dependencies: ${missing[*]}"
        echo ""
        echo "Install with:"
        echo "  Ubuntu/Debian: sudo apt install build-essential wget unzip rsync cpio bc python3 libncurses-dev"
        echo "  Fedora: sudo dnf install make gcc gcc-c++ unzip rsync cpio bc python3 ncurses-devel"
        echo "  Arch: sudo pacman -S base-devel wget unzip rsync cpio bc python ncurses"
        exit 1
    fi

    # Check for Rust (for building RexOS components)
    if ! command -v cargo &>/dev/null; then
        log_warn "Rust not found. RexOS components will be built inside Buildroot."
        log_warn "For faster builds, install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    fi

    log "Dependencies OK"
}

# Download Buildroot
download_buildroot() {
    log_step "Downloading Buildroot ${BUILDROOT_VERSION}..."

    mkdir -p "$BUILD_BASE"

    if [ -d "$BUILDROOT_DIR" ]; then
        log "Buildroot already downloaded"
        return 0
    fi

    local tarball="$BUILD_BASE/buildroot-${BUILDROOT_VERSION}.tar.xz"

    if [ ! -f "$tarball" ]; then
        log "Downloading from $BUILDROOT_URL..."
        wget -q --show-progress -O "$tarball" "$BUILDROOT_URL"
    fi

    log "Extracting Buildroot..."
    tar -xf "$tarball" -C "$BUILD_BASE"

    log "Buildroot ready"
}

# Download pre-built libretro cores (optional, speeds up build)
download_cores() {
    log_step "Downloading pre-built libretro cores..."

    if [ -x "$SCRIPT_DIR/download-cores.sh" ]; then
        "$SCRIPT_DIR/download-cores.sh" --output "$BUILD_BASE/cores" --arch aarch64
    else
        log_warn "download-cores.sh not found, cores will be downloaded during build"
    fi
}

# Configure Buildroot
configure_buildroot() {
    log_step "Configuring Buildroot for $DEVICE..."

    mkdir -p "$OUTPUT_DIR"

    # Check if defconfig exists
    if [ ! -f "$BUILDROOT_EXT/configs/$DEFCONFIG" ]; then
        log_error "Device config not found: $BUILDROOT_EXT/configs/$DEFCONFIG"
        log_error "Available configs:"
        ls -1 "$BUILDROOT_EXT/configs/" 2>/dev/null || echo "  (none)"
        exit 1
    fi

    # Run defconfig
    make -C "$BUILDROOT_DIR" \
        O="$OUTPUT_DIR" \
        BR2_EXTERNAL="$BUILDROOT_EXT" \
        "$DEFCONFIG"

    log "Configuration complete"
}

# Run menuconfig for interactive configuration
run_menuconfig() {
    log_step "Running menuconfig..."

    if [ ! -f "$OUTPUT_DIR/.config" ]; then
        configure_buildroot
    fi

    make -C "$BUILDROOT_DIR" \
        O="$OUTPUT_DIR" \
        BR2_EXTERNAL="$BUILDROOT_EXT" \
        menuconfig
}

# Build the image
build_image() {
    log_step "Building RexOS image (this will take a while)..."

    # Configure if not done
    if [ ! -f "$OUTPUT_DIR/.config" ]; then
        configure_buildroot
    fi

    # Copy pre-downloaded cores if available
    if [ -d "$BUILD_BASE/cores" ] && ls "$BUILD_BASE/cores"/*.so >/dev/null 2>&1; then
        log "Copying pre-downloaded libretro cores..."
        mkdir -p "$PROJECT_ROOT/build/cores"
        cp "$BUILD_BASE/cores"/*.so "$PROJECT_ROOT/build/cores/" 2>/dev/null || true
        if [ -d "$BUILD_BASE/cores/info" ]; then
            mkdir -p "$PROJECT_ROOT/build/cores/info"
            cp -r "$BUILD_BASE/cores/info"/* "$PROJECT_ROOT/build/cores/info/" 2>/dev/null || true
        fi
    fi

    # Run the build
    local start_time=$(date +%s)

    make -C "$BUILDROOT_DIR" \
        O="$OUTPUT_DIR" \
        BR2_EXTERNAL="$BUILDROOT_EXT" \
        -j"$PARALLEL_JOBS" \
        all

    local end_time=$(date +%s)
    local duration=$((end_time - start_time))

    log "Build completed in $(printf '%dh %dm %ds' $((duration/3600)) $((duration%3600/60)) $((duration%60)))"
}

# Clean build directory
clean_build() {
    log_step "Cleaning build directory..."

    if [ -d "$OUTPUT_DIR" ]; then
        make -C "$BUILDROOT_DIR" \
            O="$OUTPUT_DIR" \
            BR2_EXTERNAL="$BUILDROOT_EXT" \
            clean 2>/dev/null || true
    fi

    rm -rf "$OUTPUT_DIR"

    log "Clean complete"
}

# Show build info
show_info() {
    echo ""
    echo "========================================"
    echo "RexOS Image Builder"
    echo "========================================"
    echo "Buildroot version: $BUILDROOT_VERSION"
    echo "Target device:     $DEVICE"
    echo "Build directory:   $BUILD_BASE"
    echo "Output directory:  $OUTPUT_DIR"
    echo "Parallel jobs:     $PARALLEL_JOBS"
    echo ""
}

# Show results
show_results() {
    local image_dir="$OUTPUT_DIR/images"

    echo ""
    echo "========================================"
    echo "Build Complete!"
    echo "========================================"

    if [ -f "$image_dir/rexos-${DEVICE}.img" ]; then
        local size=$(du -h "$image_dir/rexos-${DEVICE}.img" | cut -f1)
        echo "Image: $image_dir/rexos-${DEVICE}.img ($size)"
    fi

    if [ -f "$image_dir/rexos-${DEVICE}.img.gz" ]; then
        local size=$(du -h "$image_dir/rexos-${DEVICE}.img.gz" | cut -f1)
        echo "Compressed: $image_dir/rexos-${DEVICE}.img.gz ($size)"
    fi

    echo ""
    echo "To flash to SD card:"
    echo "  gunzip -c $image_dir/rexos-${DEVICE}.img.gz | sudo dd of=/dev/sdX bs=4M status=progress"
    echo ""
    echo "Replace /dev/sdX with your SD card device (check with 'lsblk')"
    echo ""
}

# Usage
usage() {
    echo "Usage: $0 [command] [options]"
    echo ""
    echo "Commands:"
    echo "  all         Full build (default)"
    echo "  clean       Clean build directory"
    echo "  menuconfig  Run interactive configuration"
    echo "  download    Download sources only"
    echo "  cores       Download pre-built libretro cores"
    echo ""
    echo "Options:"
    echo "  -j N        Use N parallel jobs (default: auto)"
    echo "  -d DEVICE   Target device: rg353, rg35xx (default: rg353)"
    echo "  -h          Show this help"
    echo ""
    echo "Environment variables:"
    echo "  BUILDROOT_VERSION  Buildroot version (default: 2024.02.1)"
    echo "  BUILD_BASE         Build base directory (default: \$PROJECT/build)"
    echo "  PARALLEL_JOBS      Number of parallel jobs"
    echo "  DEVICE             Target device"
    echo ""
    echo "Examples:"
    echo "  $0                     # Build RexOS for RG353"
    echo "  $0 -d rg35xx          # Build for RG35XX"
    echo "  $0 clean all          # Clean rebuild"
    echo "  $0 menuconfig         # Configure interactively"
}

main() {
    local command="all"

    # Parse arguments
    while [ $# -gt 0 ]; do
        case "$1" in
            all|build)
                command="build"
                shift
                ;;
            clean)
                CLEAN_BUILD=true
                shift
                ;;
            menuconfig)
                command="menuconfig"
                shift
                ;;
            download)
                DOWNLOAD_ONLY=true
                shift
                ;;
            cores)
                command="cores"
                shift
                ;;
            -j)
                PARALLEL_JOBS="$2"
                shift 2
                ;;
            -d|--device)
                DEVICE="$2"
                DEFCONFIG="rexos_${DEVICE}_defconfig"
                shift 2
                ;;
            -h|--help|help)
                usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done

    show_info

    # Execute based on command
    case "$command" in
        menuconfig)
            check_dependencies
            download_buildroot
            run_menuconfig
            ;;
        cores)
            download_cores
            ;;
        build|all)
            check_dependencies

            if [ "$CLEAN_BUILD" = true ]; then
                clean_build
            fi

            download_buildroot

            if [ "$DOWNLOAD_ONLY" = true ]; then
                log "Download complete (sources only mode)"
                exit 0
            fi

            download_cores
            build_image
            show_results
            ;;
    esac
}

main "$@"
