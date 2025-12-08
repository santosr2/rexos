#!/usr/bin/env bash
# QEMU ARM64 Testing Environment for RexOS
#
# This script sets up and runs QEMU to test RexOS components.
# It supports multiple modes: kernel boot, userspace testing, and image testing.
#
# Requirements:
#   - qemu-system-aarch64 (all platforms)
#   - Docker (for userspace testing on macOS)
#   - qemu-user-static (Linux only, for native userspace emulation)
#
# Usage:
#   ./run-qemu.sh [mode] [options]
#
# Modes:
#   userspace   - Run userspace tests (Docker on macOS, qemu-user on Linux)
#   kernel      - Boot a kernel image
#   image       - Boot a full disk image
#   shell       - Interactive shell in ARM64 environment
#   native      - Run tests natively (Apple Silicon only)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default configuration
MODE="${1:-userspace}"
MEMORY="${QEMU_MEMORY:-1024}"
CPUS="${QEMU_CPUS:-4}"
PROFILE="${REXOS_MOCK_DEVICE:-qemu_virt}"

# Detect platform
IS_MACOS=false
IS_LINUX=false
IS_ARM64=false

if [[ "$OSTYPE" == "darwin"* ]]; then
    IS_MACOS=true
    if [[ "$(uname -m)" == "arm64" ]]; then
        IS_ARM64=true
    fi
elif [[ "$OSTYPE" == "linux"* ]]; then
    IS_LINUX=true
    if [[ "$(uname -m)" == "aarch64" ]]; then
        IS_ARM64=true
    fi
fi

log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[OK]${NC} $*"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*"
}

check_dependencies() {
    local mode="$1"
    local missing=()

    case "$mode" in
        userspace)
            if $IS_MACOS; then
                # macOS: need Docker for ARM64 emulation
                if ! command -v docker &>/dev/null; then
                    missing+=("docker")
                fi
            else
                # Linux: can use qemu-user
                if ! command -v qemu-aarch64 &>/dev/null && ! command -v qemu-aarch64-static &>/dev/null; then
                    missing+=("qemu-user or qemu-user-static")
                fi
            fi
            ;;
        kernel|image)
            if ! command -v qemu-system-aarch64 &>/dev/null; then
                missing+=("qemu-system-aarch64")
            fi
            ;;
        shell)
            if ! command -v docker &>/dev/null; then
                missing+=("docker")
            fi
            ;;
        native)
            if ! $IS_ARM64; then
                log_error "Native mode requires ARM64 hardware (Apple Silicon or aarch64 Linux)"
                exit 1
            fi
            ;;
    esac

    if [[ ${#missing[@]} -gt 0 ]]; then
        log_error "Missing dependencies: ${missing[*]}"
        echo ""
        echo "Install with:"
        if $IS_MACOS; then
            echo "  brew install qemu"
            echo "  brew install --cask docker"
        else
            echo "  Ubuntu: sudo apt install qemu-system-arm qemu-user-static docker.io"
            echo "  Arch:   sudo pacman -S qemu-system-aarch64 qemu-user-static docker"
        fi
        exit 1
    fi

    log_success "All dependencies found for '$mode' mode"
}

run_userspace_tests() {
    log_info "Running userspace tests..."

    cd "$PROJECT_ROOT"

    if $IS_MACOS; then
        run_userspace_tests_docker
    else
        run_userspace_tests_qemu_user
    fi

    log_success "Userspace tests completed"
}

run_userspace_tests_docker() {
    log_info "Using Docker for ARM64 emulation (macOS)..."

    # Check if Docker is running
    if ! docker info &>/dev/null; then
        log_error "Docker is not running. Please start Docker Desktop."
        exit 1
    fi

    log_info "Running tests with mock device profile: $PROFILE"

    # Use cross which handles Docker internally
    if command -v cross &>/dev/null; then
        REXOS_MOCK_DEVICE="$PROFILE" cross test --target aarch64-unknown-linux-gnu -- --nocapture
    else
        # Fallback: run directly in Docker container
        log_info "Cross not found, using Docker directly..."
        docker run --rm \
            --platform linux/arm64 \
            -v "${PROJECT_ROOT}:/workspace" \
            -w /workspace \
            -e "REXOS_MOCK_DEVICE=$PROFILE" \
            -e "CARGO_TERM_COLOR=always" \
            rust:latest \
            cargo test --all-features -- --nocapture
    fi
}

run_userspace_tests_qemu_user() {
    log_info "Using QEMU user-mode emulation (Linux)..."

    # Determine which qemu-user binary to use
    local qemu_bin=""
    if command -v qemu-aarch64-static &>/dev/null; then
        qemu_bin="qemu-aarch64-static"
    elif command -v qemu-aarch64 &>/dev/null; then
        qemu_bin="qemu-aarch64"
    else
        log_error "No qemu-aarch64 binary found"
        exit 1
    fi

    log_info "Using $qemu_bin for emulation"

    # Check for cross-compiled test binary
    local target_dir="${PROJECT_ROOT}/target/aarch64-unknown-linux-gnu/debug"

    if [[ ! -d "$target_dir" ]]; then
        log_warn "No cross-compiled binaries found. Building..."

        if command -v cross &>/dev/null; then
            cross build --target aarch64-unknown-linux-gnu
        else
            log_error "Please install 'cross' for cross-compilation:"
            echo "  cargo install cross"
            exit 1
        fi
    fi

    # Run tests with QEMU user-mode emulation
    export REXOS_MOCK_DEVICE="$PROFILE"
    export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUNNER="$qemu_bin -L /usr/aarch64-linux-gnu"

    log_info "Running tests with mock device profile: $PROFILE"

    if command -v cross &>/dev/null; then
        cross test --target aarch64-unknown-linux-gnu -- --nocapture
    else
        log_error "Cross not available. Use 'cargo test' for host testing."
        exit 1
    fi
}

run_kernel_mode() {
    local kernel="${2:-}"

    if [[ -z "$kernel" ]]; then
        log_error "Kernel mode requires a kernel image path"
        echo "Usage: $0 kernel /path/to/Image"
        exit 1
    fi

    if [[ ! -f "$kernel" ]]; then
        log_error "Kernel image not found: $kernel"
        exit 1
    fi

    log_info "Booting kernel: $kernel"
    log_info "Memory: ${MEMORY}M, CPUs: $CPUS"

    qemu-system-aarch64 \
        -M virt \
        -cpu cortex-a53 \
        -m "$MEMORY" \
        -smp "$CPUS" \
        -kernel "$kernel" \
        -append "console=ttyAMA0 root=/dev/vda rw" \
        -nographic \
        -serial mon:stdio
}

run_image_mode() {
    local image="${2:-}"

    if [[ -z "$image" ]]; then
        log_error "Image mode requires a disk image path"
        echo "Usage: $0 image /path/to/rexos.img"
        exit 1
    fi

    if [[ ! -f "$image" ]]; then
        log_error "Disk image not found: $image"
        exit 1
    fi

    log_info "Booting disk image: $image"
    log_info "Memory: ${MEMORY}M, CPUs: $CPUS"

    qemu-system-aarch64 \
        -M virt \
        -cpu cortex-a53 \
        -m "$MEMORY" \
        -smp "$CPUS" \
        -drive file="$image",format=raw,if=virtio \
        -device virtio-gpu-pci \
        -device virtio-keyboard-pci \
        -nographic \
        -serial mon:stdio
}

run_shell_mode() {
    log_info "Starting interactive ARM64 shell via Docker..."

    if ! docker info &>/dev/null; then
        log_error "Docker is not running. Please start Docker."
        exit 1
    fi

    docker run --rm -it \
        --platform linux/arm64 \
        -v "${PROJECT_ROOT}:/workspace" \
        -w /workspace \
        -e "REXOS_MOCK_DEVICE=$PROFILE" \
        -e "CARGO_TERM_COLOR=always" \
        rust:latest \
        /bin/bash
}

run_native_mode() {
    log_info "Running tests natively on ARM64 hardware..."

    if ! $IS_ARM64; then
        log_error "Native mode requires ARM64 hardware"
        log_info "On Intel Mac, use: $0 userspace"
        exit 1
    fi

    cd "$PROJECT_ROOT"
    export REXOS_MOCK_DEVICE="$PROFILE"

    log_info "Running tests with mock device profile: $PROFILE"

    if $IS_MACOS; then
        # Apple Silicon can run aarch64-apple-darwin natively
        log_info "Detected Apple Silicon - running native macOS tests"
        cargo test --all-features -- --nocapture
    else
        # Linux aarch64 - can run aarch64-unknown-linux-gnu natively
        log_info "Detected aarch64 Linux - running native Linux tests"
        cargo test --all-features -- --nocapture
    fi

    log_success "Native tests completed"
}

show_help() {
    local platform_info=""
    if $IS_MACOS; then
        if $IS_ARM64; then
            platform_info="Platform: macOS (Apple Silicon) - native ARM64 support"
        else
            platform_info="Platform: macOS (Intel) - uses Docker for ARM64"
        fi
    else
        if $IS_ARM64; then
            platform_info="Platform: Linux (aarch64) - native ARM64 support"
        else
            platform_info="Platform: Linux (x86_64) - uses qemu-user for ARM64"
        fi
    fi

    cat <<EOF
RexOS QEMU Testing Environment
$platform_info

Usage: $0 [mode] [options]

Modes:
  userspace   Run userspace tests (default)
              - macOS: Uses Docker with ARM64 container
              - Linux: Uses qemu-user-static
  kernel      Boot a kernel image in QEMU system emulation
  image       Boot a full disk image in QEMU system emulation
  shell       Interactive ARM64 shell via Docker
  native      Run tests natively (Apple Silicon / aarch64 Linux only)
  help        Show this help message

Environment Variables:
  QEMU_MEMORY          Memory allocation in MB (default: 1024)
  QEMU_CPUS            Number of CPUs (default: 4)
  REXOS_MOCK_DEVICE    Device profile to use (default: qemu_virt)

Available Device Profiles:
  rg353m      Anbernic RG353M (640x480, dual analog)
  rg353v      Anbernic RG353V (640x480, dual analog, touch)
  rg35xx      Anbernic RG35XX (640x480, no analog)
  rgb30       Powkiddy RGB30 (720x720 square)
  qemu_virt   QEMU virtual machine (default)
  desktop     Desktop development

Examples:
  # Run tests with default settings
  $0

  # Run tests as RG35XX (no analog sticks)
  REXOS_MOCK_DEVICE=rg35xx $0

  # Run native tests (Apple Silicon / aarch64 only)
  $0 native

  # Boot a kernel image
  $0 kernel path/to/Image

  # Boot a disk image with more memory
  QEMU_MEMORY=2048 $0 image path/to/rexos.img

  # Interactive ARM64 shell
  $0 shell

Platform-Specific Notes:
  macOS (Intel):    Requires Docker Desktop for ARM64 emulation
  macOS (Apple Si): Can run 'native' mode or Docker-based tests
  Linux (x86_64):   Uses qemu-user-static for userspace emulation
  Linux (aarch64):  Can run 'native' mode directly
EOF
}

main() {
    echo "=========================================="
    echo "  RexOS QEMU Testing Environment"
    echo "=========================================="
    echo ""

    # Show platform info
    if $IS_MACOS; then
        if $IS_ARM64; then
            log_info "Platform: macOS (Apple Silicon)"
        else
            log_info "Platform: macOS (Intel)"
        fi
    else
        if $IS_ARM64; then
            log_info "Platform: Linux (aarch64)"
        else
            log_info "Platform: Linux (x86_64)"
        fi
    fi
    echo ""

    case "$MODE" in
        userspace)
            check_dependencies "$MODE"
            run_userspace_tests
            ;;
        kernel)
            check_dependencies "$MODE"
            run_kernel_mode "$@"
            ;;
        image)
            check_dependencies "$MODE"
            run_image_mode "$@"
            ;;
        shell)
            check_dependencies "$MODE"
            run_shell_mode
            ;;
        native)
            check_dependencies "$MODE"
            run_native_mode
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            log_error "Unknown mode: $MODE"
            echo ""
            show_help
            exit 1
            ;;
    esac
}

main "$@"
