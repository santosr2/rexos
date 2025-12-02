#!/bin/bash
# RexOS Startup Script
# Based on ArkOS startup patterns

set -e

REXOS_LOG="/var/log/rexos.log"
REXOS_CONFIG="/etc/rexos"
ROMS_DIR="/roms"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$REXOS_LOG"
}

# Ensure log directory exists
mkdir -p "$(dirname "$REXOS_LOG")"

log "RexOS starting..."

# ============================================
# 1. Check and expand filesystem if needed
# ============================================
check_filesystem() {
    log "Checking filesystem..."

    # Check if this is first boot
    if [ -f "/etc/rexos/.first_boot" ]; then
        log "First boot detected, expanding filesystem..."

        # Expand root partition to fill SD card
        ROOT_PART=$(findmnt -n -o SOURCE /)
        ROOT_DEV=$(echo "$ROOT_PART" | sed 's/[0-9]*$//')
        PART_NUM=$(echo "$ROOT_PART" | grep -o '[0-9]*$')

        if [ -n "$ROOT_DEV" ] && [ -n "$PART_NUM" ]; then
            # Resize partition
            parted -s "$ROOT_DEV" resizepart "$PART_NUM" 100%

            # Resize filesystem
            resize2fs "$ROOT_PART"

            log "Filesystem expanded successfully"
        fi

        rm -f "/etc/rexos/.first_boot"
    fi
}

# ============================================
# 2. Mount ROMs partition
# ============================================
mount_roms() {
    log "Mounting ROMs partition..."

    # Primary SD card ROMs partition
    if [ -b /dev/mmcblk0p2 ]; then
        if ! mountpoint -q "$ROMS_DIR"; then
            mount /dev/mmcblk0p2 "$ROMS_DIR" 2>/dev/null || true
        fi
    fi

    # Secondary SD card (if present)
    if [ -b /dev/mmcblk1p1 ]; then
        mkdir -p /roms2
        if ! mountpoint -q /roms2; then
            mount /dev/mmcblk1p1 /roms2 2>/dev/null || true
        fi
        log "Secondary SD card mounted at /roms2"
    fi

    # Create standard directories if missing
    for dir in bios saves states screenshots; do
        mkdir -p "$ROMS_DIR/$dir"
    done
}

# ============================================
# 3. Apply device-specific settings
# ============================================
apply_device_settings() {
    log "Applying device settings..."

    # Detect device
    DEVICE_MODEL=""
    if [ -f /sys/firmware/devicetree/base/model ]; then
        DEVICE_MODEL=$(cat /sys/firmware/devicetree/base/model | tr -d '\0')
    fi

    log "Detected device: $DEVICE_MODEL"

    # Apply device-specific kernel parameters
    case "$DEVICE_MODEL" in
        *RG353*)
            # RG353 series settings
            echo 4 > /sys/class/graphics/fb0/blank 2>/dev/null || true
            echo 0 > /sys/class/graphics/fb0/blank 2>/dev/null || true
            ;;
        *RG351*)
            # RG351 series settings
            ;;
        *RGB30*)
            # RGB30 (square display) settings
            ;;
    esac
}

# ============================================
# 4. Set up display
# ============================================
setup_display() {
    log "Setting up display..."

    # Read saved brightness or use default
    BRIGHTNESS=180
    if [ -f "$REXOS_CONFIG/brightness" ]; then
        BRIGHTNESS=$(cat "$REXOS_CONFIG/brightness")
    fi

    # Apply brightness
    for bl in /sys/class/backlight/*/brightness; do
        if [ -f "$bl" ]; then
            echo "$BRIGHTNESS" > "$bl" 2>/dev/null || true
            log "Set brightness to $BRIGHTNESS"
            break
        fi
    done
}

# ============================================
# 5. Set up audio
# ============================================
setup_audio() {
    log "Setting up audio..."

    # Read saved volume or use default
    VOLUME=70
    if [ -f "$REXOS_CONFIG/volume" ]; then
        VOLUME=$(cat "$REXOS_CONFIG/volume")
    fi

    # Apply volume using amixer
    amixer sset Master "${VOLUME}%" 2>/dev/null || \
    amixer sset Playback "${VOLUME}%" 2>/dev/null || true

    log "Set volume to ${VOLUME}%"
}

# ============================================
# 6. Set performance profile
# ============================================
setup_performance() {
    log "Setting up performance profile..."

    # Default to balanced (schedutil or ondemand)
    GOVERNOR="schedutil"
    if [ -f "$REXOS_CONFIG/governor" ]; then
        GOVERNOR=$(cat "$REXOS_CONFIG/governor")
    fi

    # Apply to all CPUs
    for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
        if [ -f "$cpu" ]; then
            echo "$GOVERNOR" > "$cpu" 2>/dev/null || true
        fi
    done

    log "Set CPU governor to $GOVERNOR"
}

# ============================================
# 7. Start services
# ============================================
start_services() {
    log "Starting services..."

    # Start network services if enabled
    if [ -f "$REXOS_CONFIG/wifi_enabled" ]; then
        systemctl start NetworkManager 2>/dev/null || true
    fi

    # Start SSH if enabled
    if [ -f "$REXOS_CONFIG/ssh_enabled" ]; then
        systemctl start ssh 2>/dev/null || true
        log "SSH enabled"
    fi

    # Start Samba if enabled
    if [ -f "$REXOS_CONFIG/samba_enabled" ]; then
        systemctl start smbd 2>/dev/null || true
        log "Samba enabled"
    fi
}

# ============================================
# 8. Launch frontend
# ============================================
launch_frontend() {
    log "Launching frontend..."

    # Check which frontend to launch
    FRONTEND="emulationstation"
    if [ -f "$REXOS_CONFIG/frontend" ]; then
        FRONTEND=$(cat "$REXOS_CONFIG/frontend")
    fi

    # Check for button hold to launch alternative frontend
    # Hold B to boot to RetroArch menu
    # (Production would check GPIO or evdev here)

    case "$FRONTEND" in
        retroarch)
            log "Launching RetroArch..."
            exec /usr/bin/retroarch
            ;;
        *)
            log "Launching EmulationStation..."
            exec /usr/bin/emulationstation
            ;;
    esac
}

# ============================================
# Main execution
# ============================================
main() {
    check_filesystem
    mount_roms
    apply_device_settings
    setup_display
    setup_audio
    setup_performance
    start_services

    log "RexOS startup complete"

    # Launch frontend
    launch_frontend
}

main "$@"
