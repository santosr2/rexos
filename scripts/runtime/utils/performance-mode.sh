#!/usr/bin/env bash
# Script Name: performance-mode.sh
# Description: Switch between performance profiles in RexOS
# Author: RexOS Contributors
# Date: 2025-11-30

set -euo pipefail

readonly CURRENT_MODE_FILE="/var/lib/rexos/performance-mode"
readonly CPU_GOVERNOR_PATH="/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor"

log_info() {
    echo "[INFO] $*"
}

log_error() {
    echo "[ERROR] $*" >&2
}

get_current_mode() {
    if [[ -f "$CURRENT_MODE_FILE" ]]; then
        cat "$CURRENT_MODE_FILE"
    else
        echo "balanced"
    fi
}

set_powersave_mode() {
    log_info "Setting powersave mode..."

    # Set CPU governor
    if [[ -f "$CPU_GOVERNOR_PATH" ]]; then
        echo "powersave" > "$CPU_GOVERNOR_PATH" 2>/dev/null || log_error "Failed to set CPU governor"
    fi

    # Additional power saving measures
    # TODO: Implement brightness reduction, GPU throttling, etc.

    echo "powersave" > "$CURRENT_MODE_FILE"
    log_info "Powersave mode enabled - Maximum battery life"
}

set_balanced_mode() {
    log_info "Setting balanced mode..."

    if [[ -f "$CPU_GOVERNOR_PATH" ]]; then
        echo "ondemand" > "$CPU_GOVERNOR_PATH" 2>/dev/null || log_error "Failed to set CPU governor"
    fi

    echo "balanced" > "$CURRENT_MODE_FILE"
    log_info "Balanced mode enabled - Optimal performance/battery balance"
}

set_performance_mode() {
    log_info "Setting performance mode..."

    if [[ -f "$CPU_GOVERNOR_PATH" ]]; then
        echo "performance" > "$CPU_GOVERNOR_PATH" 2>/dev/null || log_error "Failed to set CPU governor"
    fi

    echo "performance" > "$CURRENT_MODE_FILE"
    log_info "Performance mode enabled - Maximum performance"
}

show_status() {
    local current_mode
    current_mode=$(get_current_mode)

    echo "Current Performance Mode: $current_mode"
    echo ""
    echo "Available modes:"
    echo "  powersave   - Maximum battery life (8-10 hours)"
    echo "  balanced    - Optimal balance (5-7 hours) [default]"
    echo "  performance - Maximum performance (3-4 hours)"
}

show_help() {
    cat << EOF
RexOS Performance Mode Switcher

Usage: $(basename "$0") [MODE]

Modes:
    powersave     - Enable power saving mode
    balanced      - Enable balanced mode (default)
    performance   - Enable performance mode
    status        - Show current mode

Options:
    -h, --help    - Show this help message

Examples:
    $(basename "$0") performance    # Switch to performance mode
    $(basename "$0") status          # Show current mode

EOF
}

main() {
    if [[ $# -eq 0 ]]; then
        show_status
        exit 0
    fi

    case "$1" in
        powersave)
            set_powersave_mode
            ;;
        balanced)
            set_balanced_mode
            ;;
        performance)
            set_performance_mode
            ;;
        status)
            show_status
            ;;
        -h|--help)
            show_help
            ;;
        *)
            log_error "Unknown mode: $1"
            show_help
            exit 1
            ;;
    esac
}

main "$@"
