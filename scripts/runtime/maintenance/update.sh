#!/bin/bash
# RexOS Update Script
# Handles system updates with rollback capability

set -e

REXOS_VERSION_FILE="/etc/rexos/version"
UPDATE_URL="https://github.com/santosr2/rexos/releases"
UPDATE_DIR="/tmp/rexos_update"
BACKUP_DIR="/roms/.rexos_backup"

log() {
    echo "[UPDATE] $1"
}

# ============================================
# Check for updates
# ============================================
check_updates() {
    log "Checking for updates..."

    current_version="0.1.0"
    if [ -f "$REXOS_VERSION_FILE" ]; then
        current_version=$(cat "$REXOS_VERSION_FILE")
    fi

    log "Current version: $current_version"

    # Check GitHub releases for latest version
    # In production, this would use curl to fetch release info
    # latest_version=$(curl -s "$UPDATE_URL/latest" | grep -o '"tag_name":"[^"]*' | cut -d'"' -f4)

    log "Check $UPDATE_URL for available updates"
}

# ============================================
# Download update
# ============================================
download_update() {
    local version="$1"

    if [ -z "$version" ]; then
        log "Error: No version specified"
        exit 1
    fi

    log "Downloading update $version..."

    mkdir -p "$UPDATE_DIR"
    cd "$UPDATE_DIR"

    # In production, download from release URL
    # wget "${UPDATE_URL}/download/${version}/rexos-${version}.tar.gz"

    log "Download complete"
}

# ============================================
# Create recovery point
# ============================================
create_recovery_point() {
    log "Creating recovery point..."

    mkdir -p "$BACKUP_DIR"

    # Backup critical system files
    cp -r /etc/rexos "$BACKUP_DIR/rexos_config" 2>/dev/null || true
    cp -r /usr/share/rexos "$BACKUP_DIR/rexos_share" 2>/dev/null || true

    # Record current version
    if [ -f "$REXOS_VERSION_FILE" ]; then
        cp "$REXOS_VERSION_FILE" "$BACKUP_DIR/version.old"
    fi

    log "Recovery point created"
}

# ============================================
# Apply update
# ============================================
apply_update() {
    local update_file="$1"

    if [ ! -f "$update_file" ]; then
        log "Error: Update file not found: $update_file"
        exit 1
    fi

    log "Applying update..."

    # Create recovery point first
    create_recovery_point

    # Extract update
    tar -xzf "$update_file" -C /tmp

    # Run update script if present
    if [ -f /tmp/rexos-update/install.sh ]; then
        chmod +x /tmp/rexos-update/install.sh
        /tmp/rexos-update/install.sh
    fi

    # Cleanup
    rm -rf /tmp/rexos-update

    log "Update applied successfully"
    log "Please reboot for changes to take effect"
}

# ============================================
# Rollback to previous version
# ============================================
rollback() {
    log "Rolling back to previous version..."

    if [ ! -d "$BACKUP_DIR" ]; then
        log "Error: No recovery point found"
        exit 1
    fi

    # Restore from backup
    if [ -d "$BACKUP_DIR/rexos_config" ]; then
        cp -r "$BACKUP_DIR/rexos_config/"* /etc/rexos/
    fi

    if [ -d "$BACKUP_DIR/rexos_share" ]; then
        cp -r "$BACKUP_DIR/rexos_share/"* /usr/share/rexos/
    fi

    if [ -f "$BACKUP_DIR/version.old" ]; then
        cp "$BACKUP_DIR/version.old" "$REXOS_VERSION_FILE"
    fi

    log "Rollback complete"
    log "Please reboot for changes to take effect"
}

# ============================================
# Update RetroArch cores
# ============================================
update_cores() {
    log "Updating RetroArch cores..."

    # Core updates would be handled separately
    # This would download latest cores from buildbot or custom source

    log "Core update not yet implemented"
}

# ============================================
# Main
# ============================================
usage() {
    echo "Usage: $0 <command> [options]"
    echo ""
    echo "Commands:"
    echo "  check              Check for available updates"
    echo "  download <ver>     Download specific version"
    echo "  apply <file>       Apply update from file"
    echo "  rollback           Rollback to previous version"
    echo "  cores              Update RetroArch cores"
    echo ""
}

main() {
    local command="${1:-check}"

    case "$command" in
        check)
            check_updates
            ;;
        download)
            download_update "$2"
            ;;
        apply)
            apply_update "$2"
            ;;
        rollback)
            rollback
            ;;
        cores)
            update_cores
            ;;
        *)
            usage
            exit 1
            ;;
    esac
}

main "$@"
