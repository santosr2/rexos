#!/bin/bash
# RexOS Backup Script
# Creates backups of saves, states, and configuration

set -e

BACKUP_DIR="/roms/backups"
REXOS_CONFIG="/etc/rexos"
DATE_STAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_NAME="rexos_backup_${DATE_STAMP}"

log() {
    echo "[BACKUP] $1"
}

# ============================================
# Create backup archive
# ============================================
create_backup() {
    local backup_type="${1:-full}"
    local backup_path="${BACKUP_DIR}/${BACKUP_NAME}"

    log "Creating $backup_type backup..."

    mkdir -p "$backup_path"

    case "$backup_type" in
        saves)
            # Backup save files only
            if [ -d /roms/saves ]; then
                cp -r /roms/saves "$backup_path/"
                log "Backed up saves"
            fi
            ;;
        states)
            # Backup save states only
            if [ -d /roms/states ]; then
                cp -r /roms/states "$backup_path/"
                log "Backed up states"
            fi
            ;;
        config)
            # Backup configuration only
            if [ -d "$REXOS_CONFIG" ]; then
                cp -r "$REXOS_CONFIG" "$backup_path/"
                log "Backed up config"
            fi
            if [ -d /home/ark/.config/retroarch ]; then
                mkdir -p "$backup_path/retroarch"
                cp /home/ark/.config/retroarch/*.cfg "$backup_path/retroarch/" 2>/dev/null || true
                log "Backed up RetroArch config"
            fi
            ;;
        full|*)
            # Full backup
            if [ -d /roms/saves ]; then
                cp -r /roms/saves "$backup_path/"
            fi
            if [ -d /roms/states ]; then
                cp -r /roms/states "$backup_path/"
            fi
            if [ -d "$REXOS_CONFIG" ]; then
                cp -r "$REXOS_CONFIG" "$backup_path/"
            fi
            if [ -d /home/ark/.config/retroarch ]; then
                mkdir -p "$backup_path/retroarch"
                cp /home/ark/.config/retroarch/*.cfg "$backup_path/retroarch/" 2>/dev/null || true
            fi
            log "Created full backup"
            ;;
    esac

    # Create compressed archive
    cd "$BACKUP_DIR"
    tar -czf "${BACKUP_NAME}.tar.gz" "$BACKUP_NAME"
    rm -rf "$backup_path"

    log "Backup created: ${BACKUP_DIR}/${BACKUP_NAME}.tar.gz"

    # Clean up old backups (keep last 5)
    ls -t "${BACKUP_DIR}"/rexos_backup_*.tar.gz 2>/dev/null | tail -n +6 | xargs rm -f 2>/dev/null || true
}

# ============================================
# Restore from backup
# ============================================
restore_backup() {
    local backup_file="$1"

    if [ ! -f "$backup_file" ]; then
        log "Error: Backup file not found: $backup_file"
        exit 1
    fi

    log "Restoring from: $backup_file"

    # Create temp directory
    local temp_dir="/tmp/rexos_restore_$$"
    mkdir -p "$temp_dir"

    # Extract backup
    tar -xzf "$backup_file" -C "$temp_dir"

    # Find the backup directory
    local backup_dir=$(ls "$temp_dir")

    # Restore saves
    if [ -d "$temp_dir/$backup_dir/saves" ]; then
        cp -r "$temp_dir/$backup_dir/saves/"* /roms/saves/ 2>/dev/null || true
        log "Restored saves"
    fi

    # Restore states
    if [ -d "$temp_dir/$backup_dir/states" ]; then
        cp -r "$temp_dir/$backup_dir/states/"* /roms/states/ 2>/dev/null || true
        log "Restored states"
    fi

    # Restore config
    if [ -d "$temp_dir/$backup_dir/rexos" ]; then
        cp -r "$temp_dir/$backup_dir/rexos/"* "$REXOS_CONFIG/" 2>/dev/null || true
        log "Restored RexOS config"
    fi

    # Restore RetroArch config
    if [ -d "$temp_dir/$backup_dir/retroarch" ]; then
        cp "$temp_dir/$backup_dir/retroarch/"*.cfg /home/ark/.config/retroarch/ 2>/dev/null || true
        log "Restored RetroArch config"
    fi

    # Cleanup
    rm -rf "$temp_dir"

    log "Restore complete"
}

# ============================================
# List available backups
# ============================================
list_backups() {
    log "Available backups:"
    ls -lh "${BACKUP_DIR}"/rexos_backup_*.tar.gz 2>/dev/null || echo "No backups found"
}

# ============================================
# Main
# ============================================
usage() {
    echo "Usage: $0 <command> [options]"
    echo ""
    echo "Commands:"
    echo "  create [type]    Create backup (full, saves, states, config)"
    echo "  restore <file>   Restore from backup file"
    echo "  list             List available backups"
    echo ""
}

main() {
    local command="${1:-create}"

    case "$command" in
        create)
            create_backup "${2:-full}"
            ;;
        restore)
            if [ -z "$2" ]; then
                usage
                exit 1
            fi
            restore_backup "$2"
            ;;
        list)
            list_backups
            ;;
        *)
            usage
            exit 1
            ;;
    esac
}

main "$@"
