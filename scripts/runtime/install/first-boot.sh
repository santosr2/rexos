#!/usr/bin/env bash
# Script Name: first-boot.sh
# Description: First boot configuration for RexOS
# Author: RexOS Contributors
# Date: 2025-11-30

set -euo pipefail

readonly LOG_FILE="/var/log/rexos/first-boot.log"
readonly CONFIG_DIR="/etc/rexos"
readonly FIRST_BOOT_FLAG="/var/lib/rexos/.first-boot-done"

log_info() {
    echo "[INFO] $(date '+%Y-%m-%d %H:%M:%S') - $*" | tee -a "$LOG_FILE"
}

log_error() {
    echo "[ERROR] $(date '+%Y-%m-%d %H:%M:%S') - $*" >&2 | tee -a "$LOG_FILE"
}

check_first_boot() {
    if [[ -f "$FIRST_BOOT_FLAG" ]]; then
        log_info "Not first boot, exiting"
        exit 0
    fi
}

detect_device() {
    log_info "Detecting device..."
    # TODO: Implement actual device detection
    # This would read from device tree or other sources
    echo "rg353m" > "$CONFIG_DIR/device"
    log_info "Device detected: RG353M (placeholder)"
}

setup_directories() {
    log_info "Setting up directories..."
    mkdir -p /roms
    mkdir -p /userdata/saves
    mkdir -p /userdata/config
    mkdir -p /userdata/screenshots
    chown -R ark:ark /roms /userdata
}

initialize_database() {
    log_info "Initializing game library database..."
    # TODO: Create initial database schema
    touch /userdata/library.db
    chown ark:ark /userdata/library.db
}

mark_complete() {
    mkdir -p "$(dirname "$FIRST_BOOT_FLAG")"
    date > "$FIRST_BOOT_FLAG"
    log_info "First boot configuration completed"
}

main() {
    log_info "RexOS First Boot Configuration"
    log_info "==============================="

    check_first_boot
    detect_device
    setup_directories
    initialize_database
    mark_complete

    log_info "System ready!"
}

main "$@"
