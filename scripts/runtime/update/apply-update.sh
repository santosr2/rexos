#!/bin/bash
# RexOS Update Application Script
# Applies downloaded updates safely with rollback support

set -euo pipefail

UPDATE_DIR="/rexos/updates"
BACKUP_DIR="/rexos/backups"
REXOS_ROOT="/rexos"
LOG_FILE="/var/log/rexos-update.log"
LOCK_FILE="/var/lock/rexos-update.lock"

log() {
    local msg="[$(date '+%Y-%m-%d %H:%M:%S')] $1"
    echo "$msg"
    echo "$msg" >> "$LOG_FILE"
}

error() {
    log "ERROR: $1"
    exit 1
}

# Acquire lock to prevent concurrent updates
acquire_lock() {
    exec 200>"$LOCK_FILE"
    if ! flock -n 200; then
        error "Another update is in progress"
    fi
}

# Release lock
release_lock() {
    flock -u 200 2>/dev/null || true
    rm -f "$LOCK_FILE"
}

# Cleanup on exit
cleanup() {
    release_lock
}
trap cleanup EXIT

# Verify update package
verify_package() {
    local package="$1"
    local sig_file="${package}.sig"

    log "Verifying update package..."

    # Check package exists
    [ -f "$package" ] || error "Package not found: $package"

    # Check signature
    if [ -f "$sig_file" ]; then
        local pubkey="/rexos/keys/update-signing.pub"
        if [ -f "$pubkey" ] && command -v openssl &>/dev/null; then
            if ! openssl dgst -sha256 -verify "$pubkey" -signature "$sig_file" "$package" &>/dev/null; then
                error "Signature verification failed!"
            fi
            log "Signature verified"
        else
            log "Warning: Signature verification skipped (no key or openssl)"
        fi
    else
        log "Warning: No signature file found"
    fi

    # Verify archive integrity
    if ! tar -tzf "$package" &>/dev/null; then
        error "Package is corrupted"
    fi

    log "Package verification complete"
}

# Create backup before update
create_backup() {
    local backup_name="pre-update-$(date '+%Y%m%d-%H%M%S')"
    local backup_path="$BACKUP_DIR/$backup_name"

    log "Creating backup: $backup_name"

    mkdir -p "$BACKUP_DIR"

    # Create backup of critical directories
    tar -czf "${backup_path}.tar.gz" \
        -C "$REXOS_ROOT" \
        --exclude='updates/*' \
        --exclude='backups/*' \
        --exclude='roms/*' \
        bin lib config 2>/dev/null || true

    # Save current version
    if [ -f "$REXOS_ROOT/version" ]; then
        cp "$REXOS_ROOT/version" "${backup_path}.version"
    fi

    echo "$backup_name" > "$BACKUP_DIR/last_backup"

    log "Backup created successfully"
}

# Extract and apply update
apply_update() {
    local package="$1"
    local extract_dir="$UPDATE_DIR/extract"

    log "Extracting update package..."

    # Clean extract directory
    rm -rf "$extract_dir"
    mkdir -p "$extract_dir"

    # Extract package
    tar -xzf "$package" -C "$extract_dir"

    # Check for update manifest
    local manifest="$extract_dir/manifest.json"
    if [ -f "$manifest" ]; then
        log "Found update manifest"

        # Parse version from manifest
        local new_version=$(grep -oP '"version"\s*:\s*"\K[^"]+' "$manifest" || echo "unknown")
        log "Updating to version: $new_version"
    fi

    # Run pre-update script if exists
    if [ -f "$extract_dir/pre-update.sh" ]; then
        log "Running pre-update script..."
        chmod +x "$extract_dir/pre-update.sh"
        if ! "$extract_dir/pre-update.sh"; then
            error "Pre-update script failed"
        fi
    fi

    # Apply file updates
    log "Applying file updates..."

    if [ -d "$extract_dir/bin" ]; then
        cp -rf "$extract_dir/bin"/* "$REXOS_ROOT/bin/" 2>/dev/null || true
    fi

    if [ -d "$extract_dir/lib" ]; then
        cp -rf "$extract_dir/lib"/* "$REXOS_ROOT/lib/" 2>/dev/null || true
    fi

    if [ -d "$extract_dir/config" ]; then
        # Merge configs carefully (don't overwrite user settings)
        find "$extract_dir/config" -type f | while read -r file; do
            local rel_path="${file#$extract_dir/config/}"
            local dest="$REXOS_ROOT/config/$rel_path"

            if [ -f "$dest" ]; then
                # Backup existing config
                cp "$dest" "${dest}.bak"
            fi

            mkdir -p "$(dirname "$dest")"
            cp "$file" "$dest"
        done
    fi

    if [ -d "$extract_dir/scripts" ]; then
        cp -rf "$extract_dir/scripts"/* "$REXOS_ROOT/scripts/" 2>/dev/null || true
        chmod -R +x "$REXOS_ROOT/scripts/"
    fi

    # Update version file
    if [ -f "$extract_dir/version" ]; then
        cp "$extract_dir/version" "$REXOS_ROOT/version"
    fi

    # Run post-update script if exists
    if [ -f "$extract_dir/post-update.sh" ]; then
        log "Running post-update script..."
        chmod +x "$extract_dir/post-update.sh"
        if ! "$extract_dir/post-update.sh"; then
            log "Warning: Post-update script failed"
        fi
    fi

    # Cleanup
    rm -rf "$extract_dir"

    log "Update applied successfully"
}

# Rollback to previous version
rollback() {
    log "Rolling back to previous version..."

    # Find last backup
    if [ ! -f "$BACKUP_DIR/last_backup" ]; then
        error "No backup found for rollback"
    fi

    local backup_name=$(cat "$BACKUP_DIR/last_backup")
    local backup_file="$BACKUP_DIR/${backup_name}.tar.gz"

    if [ ! -f "$backup_file" ]; then
        error "Backup file not found: $backup_file"
    fi

    log "Restoring from backup: $backup_name"

    # Extract backup
    tar -xzf "$backup_file" -C "$REXOS_ROOT"

    # Restore version
    if [ -f "$BACKUP_DIR/${backup_name}.version" ]; then
        cp "$BACKUP_DIR/${backup_name}.version" "$REXOS_ROOT/version"
    fi

    log "Rollback complete"
}

# Clean old updates and backups
cleanup_old() {
    log "Cleaning up old updates and backups..."

    # Keep only last 3 backups
    ls -t "$BACKUP_DIR"/*.tar.gz 2>/dev/null | tail -n +4 | while read -r file; do
        rm -f "$file"
        rm -f "${file%.tar.gz}.version"
    done

    # Remove downloaded update packages after successful update
    rm -f "$UPDATE_DIR"/*.tar.gz
    rm -f "$UPDATE_DIR"/*.sig

    log "Cleanup complete"
}

# Check if reboot is required
check_reboot_required() {
    local extract_dir="$UPDATE_DIR/extract"

    # Check manifest for reboot requirement
    if [ -f "$extract_dir/manifest.json" ]; then
        if grep -q '"reboot_required"\s*:\s*true' "$extract_dir/manifest.json"; then
            return 0
        fi
    fi

    # Check if kernel or critical libs updated
    if [ -d "$extract_dir/lib" ] && ls "$extract_dir/lib"/*.so* &>/dev/null; then
        return 0
    fi

    return 1
}

# Usage
usage() {
    echo "Usage: $0 <command> [package]"
    echo ""
    echo "Commands:"
    echo "  apply <package>  Apply an update package"
    echo "  rollback         Rollback to previous version"
    echo "  cleanup          Clean old updates and backups"
    echo "  verify <package> Verify an update package"
    echo ""
    echo "Examples:"
    echo "  $0 apply /rexos/updates/rexos-1.2.0.tar.gz"
    echo "  $0 rollback"
}

main() {
    local command="${1:-}"

    case "$command" in
        apply)
            local package="${2:-}"
            [ -z "$package" ] && { usage; exit 1; }

            acquire_lock

            verify_package "$package"
            create_backup
            apply_update "$package"
            cleanup_old

            if check_reboot_required; then
                log "Reboot required to complete update"
                echo "REBOOT_REQUIRED"
            fi
            ;;
        rollback)
            acquire_lock
            rollback
            ;;
        cleanup)
            cleanup_old
            ;;
        verify)
            local package="${2:-}"
            [ -z "$package" ] && { usage; exit 1; }
            verify_package "$package"
            ;;
        *)
            usage
            exit 1
            ;;
    esac
}

main "$@"
