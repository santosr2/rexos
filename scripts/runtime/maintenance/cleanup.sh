#!/bin/bash
# RexOS Cleanup Script
# Removes temporary files and frees disk space

set -euo pipefail

ROMS_DIR="/roms"
TMP_DIRS=("/tmp" "/var/tmp" "/var/cache")
LOG_DIR="/var/log"

log() {
    echo "[CLEANUP] $1"
}

# Calculate disk usage
disk_usage() {
    local path="$1"
    du -sh "$path" 2>/dev/null | cut -f1
}

# Get available space
available_space() {
    df -h "$1" 2>/dev/null | awk 'NR==2 {print $4}'
}

# Clean temporary files
clean_temp() {
    log "Cleaning temporary files..."
    local freed=0

    for dir in "${TMP_DIRS[@]}"; do
        if [ -d "$dir" ]; then
            # Remove files older than 1 day
            find "$dir" -type f -mtime +1 -delete 2>/dev/null || true
            # Remove empty directories
            find "$dir" -type d -empty -delete 2>/dev/null || true
        fi
    done

    log "Temporary files cleaned"
}

# Clean old logs
clean_logs() {
    log "Cleaning old logs..."

    if [ -d "$LOG_DIR" ]; then
        # Rotate and compress logs
        find "$LOG_DIR" -name "*.log" -size +10M -exec truncate -s 0 {} \; 2>/dev/null || true

        # Remove old rotated logs
        find "$LOG_DIR" -name "*.log.[0-9]*" -mtime +7 -delete 2>/dev/null || true
        find "$LOG_DIR" -name "*.gz" -mtime +30 -delete 2>/dev/null || true
    fi

    log "Logs cleaned"
}

# Clean RetroArch temporary files
clean_retroarch() {
    log "Cleaning RetroArch cache..."

    local ra_config="/home/ark/.config/retroarch"

    if [ -d "$ra_config" ]; then
        # Clean shader cache
        rm -rf "$ra_config/shaders/cache"/* 2>/dev/null || true

        # Clean thumbnails cache
        rm -rf "$ra_config/thumbnails/.cache" 2>/dev/null || true

        # Clean core info cache
        rm -rf "$ra_config/.tmp"/* 2>/dev/null || true
    fi

    log "RetroArch cache cleaned"
}

# Clean orphaned save states (games that no longer exist)
clean_orphaned_states() {
    log "Checking for orphaned save states..."

    local states_dir="$ROMS_DIR/states"
    local saves_dir="$ROMS_DIR/saves"
    local orphaned=0

    if [ ! -d "$states_dir" ]; then
        return
    fi

    # Find state files
    find "$states_dir" -name "*.state*" -type f 2>/dev/null | while read -r state_file; do
        local base_name=$(basename "$state_file" | sed 's/\.state[0-9]*$//')

        # Check if corresponding ROM exists in any system directory
        local rom_found=false

        for sys_dir in "$ROMS_DIR"/*/; do
            if [ -d "$sys_dir" ] && [ "$(basename "$sys_dir")" != "states" ] && [ "$(basename "$sys_dir")" != "saves" ]; then
                if find "$sys_dir" -name "$base_name.*" -type f 2>/dev/null | grep -q .; then
                    rom_found=true
                    break
                fi
            fi
        done

        if [ "$rom_found" = false ]; then
            log "  Orphaned: $state_file"
            ((orphaned++))
        fi
    done

    log "Found $orphaned orphaned state files"
}

# Clean duplicate ROMs
find_duplicates() {
    log "Checking for duplicate ROMs..."

    for sys_dir in "$ROMS_DIR"/*/; do
        if [ -d "$sys_dir" ] && [ "$(basename "$sys_dir")" != "bios" ] && \
           [ "$(basename "$sys_dir")" != "saves" ] && [ "$(basename "$sys_dir")" != "states" ]; then

            # Find files with same size (potential duplicates)
            find "$sys_dir" -type f -printf '%s %p\n' 2>/dev/null | \
                sort -n | uniq -D -w 20 | while read -r size file; do
                log "  Potential duplicate: $file ($size bytes)"
            done
        fi
    done
}

# Clean package manager cache
clean_package_cache() {
    log "Cleaning package cache..."

    # APT cache
    if command -v apt-get &>/dev/null; then
        apt-get clean 2>/dev/null || true
        apt-get autoclean 2>/dev/null || true
    fi

    # Pacman cache
    if command -v pacman &>/dev/null; then
        pacman -Sc --noconfirm 2>/dev/null || true
    fi

    log "Package cache cleaned"
}

# Main cleanup routine
full_cleanup() {
    local before=$(available_space "$ROMS_DIR")

    clean_temp
    clean_logs
    clean_retroarch
    clean_package_cache

    local after=$(available_space "$ROMS_DIR")

    log ""
    log "Cleanup complete!"
    log "Space before: $before"
    log "Space after:  $after"
}

# Usage
usage() {
    echo "Usage: $0 <command>"
    echo ""
    echo "Commands:"
    echo "  all           Full cleanup"
    echo "  temp          Clean temporary files"
    echo "  logs          Clean old logs"
    echo "  retroarch     Clean RetroArch cache"
    echo "  orphans       Find orphaned save states"
    echo "  duplicates    Find duplicate ROMs"
    echo ""
}

main() {
    local command="${1:-all}"

    case "$command" in
        all)
            full_cleanup
            ;;
        temp)
            clean_temp
            ;;
        logs)
            clean_logs
            ;;
        retroarch)
            clean_retroarch
            ;;
        orphans)
            clean_orphaned_states
            ;;
        duplicates)
            find_duplicates
            ;;
        *)
            usage
            exit 1
            ;;
    esac
}

main "$@"
