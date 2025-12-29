#!/bin/sh
# Script Name: first-boot.sh
# Description: First boot configuration for RexOS
# Author: RexOS Contributors
# Runs once on first boot to set up user data directories

set -e

REXOS_ROOT="/rexos"
DATA_DIR="${REXOS_ROOT}/data"
FIRST_BOOT_FLAG="${DATA_DIR}/.first-boot-done"
LOG_DIR="/var/log"

log_info() {
    echo "[FIRST-BOOT] $*"
    echo "$(date '+%Y-%m-%d %H:%M:%S') [INFO] $*" >> "${LOG_DIR}/first-boot.log" 2>/dev/null || true
}

log_error() {
    echo "[FIRST-BOOT] ERROR: $*" >&2
    echo "$(date '+%Y-%m-%d %H:%M:%S') [ERROR] $*" >> "${LOG_DIR}/first-boot.log" 2>/dev/null || true
}

check_first_boot() {
    if [ -f "$FIRST_BOOT_FLAG" ]; then
        log_info "Not first boot, skipping setup"
        exit 0
    fi
}

mount_data_partition() {
    log_info "Mounting data partition..."

    # Try to mount by partition label first
    if ! mountpoint -q "${DATA_DIR}" 2>/dev/null; then
        if mount PARTLABEL=data "${DATA_DIR}" 2>/dev/null; then
            log_info "  Data partition mounted by label"
        elif mount -t ext4 /dev/mmcblk0p5 "${DATA_DIR}" 2>/dev/null; then
            log_info "  Data partition mounted by device"
        else
            log_error "Could not mount data partition!"
            return 1
        fi
    fi

    return 0
}

detect_device() {
    log_info "Detecting device..."

    # Read device model from device tree
    if [ -f "/sys/firmware/devicetree/base/model" ]; then
        MODEL=$(cat /sys/firmware/devicetree/base/model | tr -d '\0')
        log_info "  Device model: $MODEL"
    else
        MODEL="Unknown"
    fi

    # Detect specific device variant
    case "$MODEL" in
        *"RG353PS"*)
            DEVICE="rg353ps"
            ;;
        *"RG353P"*)
            DEVICE="rg353p"
            ;;
        *"RG353V"*)
            DEVICE="rg353v"
            ;;
        *"RG353M"*)
            DEVICE="rg353m"
            ;;
        *"RK3566"*)
            DEVICE="rk3566-generic"
            ;;
        *)
            DEVICE="unknown"
            ;;
    esac

    log_info "  Detected: $DEVICE"

    # Save device info
    mkdir -p "${DATA_DIR}/system"
    echo "$DEVICE" > "${DATA_DIR}/system/device"
    echo "$MODEL" > "${DATA_DIR}/system/model"
}

setup_directories() {
    log_info "Setting up directories on data partition..."

    # Create directory structure on data partition
    mkdir -p "${DATA_DIR}/saves"      # Save files
    mkdir -p "${DATA_DIR}/states"     # Save states
    mkdir -p "${DATA_DIR}/screenshots"
    mkdir -p "${DATA_DIR}/config"     # User config overrides
    mkdir -p "${DATA_DIR}/bios"       # BIOS files (user can add)
    mkdir -p "${DATA_DIR}/system"     # System state
    mkdir -p "${DATA_DIR}/logs"       # Persistent logs

    # Set permissions
    chmod 755 "${DATA_DIR}"
    chmod 755 "${DATA_DIR}/saves"
    chmod 755 "${DATA_DIR}/states"
    chmod 755 "${DATA_DIR}/screenshots"
    chmod 755 "${DATA_DIR}/config"
    chmod 755 "${DATA_DIR}/bios"
    chmod 755 "${DATA_DIR}/system"
    chmod 755 "${DATA_DIR}/logs"

    log_info "  Directories created"
}

create_rom_folders() {
    log_info "Creating ROM folders..."

    ROM_DIR="${REXOS_ROOT}/roms"

    # Check if roms partition is mounted
    if mountpoint -q "${ROM_DIR}" 2>/dev/null || mount PARTLABEL=roms "${ROM_DIR}" 2>/dev/null; then
        # Create folders for each supported system
        for sys in gb gbc gba nes snes n64 nds psx psp \
                   sega-ms sega-gg sega-md sega-cd sega-32x dreamcast \
                   pce arcade mame neogeo \
                   atari2600 atari7800 lynx \
                   dos scummvm; do
            mkdir -p "${ROM_DIR}/${sys}"
        done

        # Create README
        cat > "${ROM_DIR}/README.txt" << 'EOF'
RexOS ROM Directory

Place your ROM files in the appropriate system folders:

Nintendo:
  gb/      - Game Boy
  gbc/     - Game Boy Color
  gba/     - Game Boy Advance
  nes/     - NES / Famicom
  snes/    - Super Nintendo
  n64/     - Nintendo 64
  nds/     - Nintendo DS

Sega:
  sega-ms/ - Master System
  sega-gg/ - Game Gear
  sega-md/ - Genesis / Mega Drive
  sega-cd/ - Sega CD
  sega-32x/- Sega 32X
  dreamcast/ - Dreamcast

Sony:
  psx/     - PlayStation
  psp/     - PlayStation Portable

Other:
  pce/     - PC Engine / TurboGrafx-16
  arcade/  - Arcade (FBNeo)
  mame/    - MAME
  neogeo/  - Neo Geo

Computer:
  dos/     - MS-DOS
  scummvm/ - ScummVM

Note: Some systems require BIOS files.
Place BIOS files in /rexos/data/bios/
EOF

        log_info "  ROM folders created"
    else
        log_error "Could not mount roms partition"
    fi
}

initialize_config() {
    log_info "Initializing user configuration..."

    # Copy default config to data partition for user modifications
    if [ -d "${REXOS_ROOT}/config" ] && [ ! -f "${DATA_DIR}/config/.initialized" ]; then
        # Copy retroarch config that can be modified
        mkdir -p "${DATA_DIR}/config/retroarch"

        # Create user-modifiable settings file
        cat > "${DATA_DIR}/config/user-settings.toml" << 'EOF'
# RexOS User Settings
# Modify this file to customize your RexOS experience

[display]
# brightness = 70  # 0-100

[audio]
# volume = 80  # 0-100

[system]
# timezone = "UTC"
EOF

        touch "${DATA_DIR}/config/.initialized"
        log_info "  Configuration initialized"
    fi
}

mark_complete() {
    date > "$FIRST_BOOT_FLAG"
    log_info "First boot configuration completed"
}

main() {
    log_info "=================================="
    log_info "RexOS First Boot Configuration"
    log_info "=================================="

    check_first_boot

    # Mount data partition if not already mounted
    mount_data_partition || {
        log_error "Failed to mount data partition. Continuing with limited setup."
    }

    detect_device
    setup_directories
    create_rom_folders
    initialize_config
    mark_complete

    log_info "=================================="
    log_info "Setup complete! System ready."
    log_info "=================================="
}

main "$@"
