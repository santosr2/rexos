#!/bin/bash
# RexOS Shutdown Script
# Saves state and cleanly shuts down

set -e

REXOS_CONFIG="/etc/rexos"
REXOS_LOG="/var/log/rexos.log"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] SHUTDOWN: $1" | tee -a "$REXOS_LOG"
}

log "RexOS shutting down..."

# ============================================
# 1. Save current settings
# ============================================
save_settings() {
    log "Saving settings..."

    mkdir -p "$REXOS_CONFIG"

    # Save current brightness
    for bl in /sys/class/backlight/*/brightness; do
        if [ -f "$bl" ]; then
            cat "$bl" > "$REXOS_CONFIG/brightness"
            break
        fi
    done

    # Save current volume
    amixer get Master 2>/dev/null | grep -o '[0-9]*%' | head -1 | tr -d '%' > "$REXOS_CONFIG/volume" || true

    # Save CPU governor
    if [ -f /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor ]; then
        cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor > "$REXOS_CONFIG/governor"
    fi
}

# ============================================
# 2. Sync filesystems
# ============================================
sync_filesystems() {
    log "Syncing filesystems..."
    sync
}

# ============================================
# 3. Stop services
# ============================================
stop_services() {
    log "Stopping services..."

    # Stop EmulationStation/frontend if running
    killall emulationstation 2>/dev/null || true
    killall retroarch 2>/dev/null || true

    # Stop network services
    systemctl stop smbd 2>/dev/null || true
    systemctl stop ssh 2>/dev/null || true
}

# ============================================
# 4. Unmount ROMs partition
# ============================================
unmount_roms() {
    log "Unmounting ROMs..."

    # Unmount secondary SD
    if mountpoint -q /roms2; then
        umount /roms2 2>/dev/null || true
    fi

    # Unmount primary ROMs
    if mountpoint -q /roms; then
        umount /roms 2>/dev/null || true
    fi
}

# ============================================
# 5. Perform shutdown/reboot
# ============================================
do_shutdown() {
    local action="${1:-poweroff}"

    log "Executing $action..."

    case "$action" in
        reboot)
            /sbin/reboot
            ;;
        *)
            /sbin/poweroff
            ;;
    esac
}

# ============================================
# Main
# ============================================
main() {
    local action="${1:-poweroff}"

    save_settings
    sync_filesystems
    stop_services
    unmount_roms

    log "Shutdown complete"

    do_shutdown "$action"
}

main "$@"
