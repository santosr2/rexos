#!/bin/bash
# RexOS Libretro Cores Download Script
# Downloads pre-built libretro cores from the libretro buildbot

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
CORES_DIR="${CORES_DIR:-$PROJECT_ROOT/build/cores}"
ARCH="${ARCH:-aarch64}"

# Libretro buildbot URL base
BUILDBOT_URL="https://buildbot.libretro.com/nightly/linux/${ARCH}/latest"

# Essential cores for RexOS
CORES=(
    # Nintendo
    "mgba_libretro.so.zip"              # Game Boy Advance
    "gambatte_libretro.so.zip"          # Game Boy / Game Boy Color
    "snes9x_libretro.so.zip"            # Super Nintendo
    "nestopia_libretro.so.zip"          # NES
    "mupen64plus_next_libretro.so.zip"  # Nintendo 64
    "desmume_libretro.so.zip"           # Nintendo DS
    "fceumm_libretro.so.zip"            # NES (alternate)

    # Sega
    "genesis_plus_gx_libretro.so.zip"   # Genesis/Mega Drive, Master System, Game Gear
    "picodrive_libretro.so.zip"         # 32X, Genesis (fast)

    # Sony
    "pcsx_rearmed_libretro.so.zip"      # PlayStation
    "ppsspp_libretro.so.zip"            # PSP

    # NEC
    "mednafen_pce_libretro.so.zip"      # PC Engine / TurboGrafx-16

    # Arcade
    "fbneo_libretro.so.zip"             # Final Burn Neo (CPS1/2/3, Neo Geo, etc.)
    "mame2003_plus_libretro.so.zip"     # MAME 2003+

    # Other
    "stella_libretro.so.zip"            # Atari 2600
    "prosystem_libretro.so.zip"         # Atari 7800
)

# Optional cores (disabled by default, enable with --all)
OPTIONAL_CORES=(
    "flycast_libretro.so.zip"           # Dreamcast (resource intensive)
    "beetle_psx_libretro.so.zip"        # PlayStation (more accurate)
    "bsnes_libretro.so.zip"             # SNES (more accurate)
    "dosbox_pure_libretro.so.zip"       # DOS
    "scummvm_libretro.so.zip"           # ScummVM
)

log() {
    echo "[CORES] $1"
}

error() {
    echo "[ERROR] $1" >&2
    exit 1
}

download_core() {
    local core="$1"
    local url="${BUILDBOT_URL}/${core}"
    local output_zip="${CORES_DIR}/${core}"
    local so_name="${core%.zip}"

    if [ -f "${CORES_DIR}/${so_name}" ]; then
        log "  Skipping ${so_name} (already exists)"
        return 0
    fi

    log "  Downloading ${core}..."

    if curl -fsSL --connect-timeout 10 --max-time 120 -o "$output_zip" "$url"; then
        # Extract the .so file
        if unzip -q -o "$output_zip" -d "$CORES_DIR" 2>/dev/null; then
            rm -f "$output_zip"
            log "  ✓ ${so_name}"
            return 0
        else
            log "  ✗ Failed to extract ${core}"
            rm -f "$output_zip"
            return 1
        fi
    else
        log "  ✗ Failed to download ${core}"
        return 1
    fi
}

download_info_files() {
    log "Downloading core info files..."

    local info_url="https://buildbot.libretro.com/assets/frontend/info.zip"
    local info_zip="${CORES_DIR}/info.zip"
    local info_dir="${CORES_DIR}/info"

    mkdir -p "$info_dir"

    if curl -fsSL --connect-timeout 10 --max-time 60 -o "$info_zip" "$info_url"; then
        unzip -q -o "$info_zip" -d "$info_dir" 2>/dev/null || true
        rm -f "$info_zip"
        log "Core info files downloaded"
    else
        log "Warning: Could not download core info files"
    fi
}

main() {
    local download_optional=false

    # Parse arguments
    while [ $# -gt 0 ]; do
        case "$1" in
            --all|-a)
                download_optional=true
                shift
                ;;
            --arch)
                ARCH="$2"
                shift 2
                ;;
            --output|-o)
                CORES_DIR="$2"
                shift 2
                ;;
            --help|-h)
                echo "Usage: $0 [options]"
                echo ""
                echo "Options:"
                echo "  --all, -a         Download all cores including optional ones"
                echo "  --arch ARCH       Target architecture: aarch64, armv7-neon-hf (default: aarch64)"
                echo "  --output, -o DIR  Output directory (default: build/cores)"
                echo "  --help, -h        Show this help"
                echo ""
                echo "Architectures:"
                echo "  aarch64           64-bit ARM (RG353 series)"
                echo "  armv7-neon-hf     32-bit ARM with NEON (RG35XX series)"
                exit 0
                ;;
            *)
                error "Unknown option: $1"
                ;;
        esac
    done

    # Check dependencies
    for cmd in curl unzip; do
        if ! command -v "$cmd" &>/dev/null; then
            error "Required command not found: $cmd"
        fi
    done

    # Create output directory
    mkdir -p "$CORES_DIR"

    log "Downloading libretro cores for ${ARCH}..."
    log "Output directory: ${CORES_DIR}"
    log ""

    # Track success/failure
    local success=0
    local failed=0

    # Download essential cores
    log "Essential cores:"
    for core in "${CORES[@]}"; do
        if download_core "$core"; then
            ((success++)) || true
        else
            ((failed++)) || true
        fi
    done

    # Download optional cores if requested
    if [ "$download_optional" = true ]; then
        log ""
        log "Optional cores:"
        for core in "${OPTIONAL_CORES[@]}"; do
            if download_core "$core"; then
                ((success++)) || true
            else
                ((failed++)) || true
            fi
        done
    fi

    # Download info files
    log ""
    download_info_files

    log ""
    log "=========================================="
    log "Download complete!"
    log "=========================================="
    log "  Successful: ${success}"
    log "  Failed: ${failed}"
    log "  Output: ${CORES_DIR}"
    log ""

    if [ $failed -gt 0 ]; then
        log "Some cores failed to download. They may not be available for ${ARCH}."
        log "Check https://buildbot.libretro.com/nightly/linux/${ARCH}/latest/"
    fi

    # Create manifest
    log "Creating cores manifest..."
    {
        echo "# RexOS Libretro Cores Manifest"
        echo "# Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
        echo "# Architecture: ${ARCH}"
        echo ""
        for so in "${CORES_DIR}"/*.so; do
            if [ -f "$so" ]; then
                basename "$so"
            fi
        done
    } > "${CORES_DIR}/manifest.txt"

    log "Manifest created: ${CORES_DIR}/manifest.txt"
}

main "$@"
