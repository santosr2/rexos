#!/bin/bash
# RexOS WiFi Toggle Script
# Quick WiFi on/off and connection management

set -euo pipefail

WIFI_INTERFACE="${WIFI_INTERFACE:-wlan0}"
WPA_CONF="/etc/wpa_supplicant/wpa_supplicant.conf"
NETWORKS_DIR="/rexos/config/networks"

log() {
    echo "[WIFI] $1"
}

# Check if WiFi interface exists
check_interface() {
    if ! ip link show "$WIFI_INTERFACE" &>/dev/null; then
        log "Error: WiFi interface $WIFI_INTERFACE not found"
        exit 1
    fi
}

# Get WiFi status
get_status() {
    if ip link show "$WIFI_INTERFACE" | grep -q "state UP"; then
        local ssid=$(iwgetid -r 2>/dev/null || echo "")
        if [ -n "$ssid" ]; then
            local signal=$(iwconfig "$WIFI_INTERFACE" 2>/dev/null | grep -oP 'Signal level=\K-?\d+' || echo "N/A")
            local ip=$(ip -4 addr show "$WIFI_INTERFACE" 2>/dev/null | grep -oP 'inet \K[\d.]+' || echo "N/A")
            echo "Connected to: $ssid"
            echo "Signal: ${signal} dBm"
            echo "IP: $ip"
        else
            echo "WiFi enabled, not connected"
        fi
    else
        echo "WiFi disabled"
    fi
}

# Enable WiFi
wifi_on() {
    log "Enabling WiFi..."

    # Unblock WiFi if blocked
    rfkill unblock wifi 2>/dev/null || true

    # Bring up interface
    ip link set "$WIFI_INTERFACE" up

    # Start wpa_supplicant if not running
    if ! pgrep -x wpa_supplicant &>/dev/null; then
        if [ -f "$WPA_CONF" ]; then
            wpa_supplicant -B -i "$WIFI_INTERFACE" -c "$WPA_CONF" -D nl80211,wext
        else
            log "Warning: No wpa_supplicant.conf found"
        fi
    fi

    # Request DHCP
    if command -v dhclient &>/dev/null; then
        dhclient -4 "$WIFI_INTERFACE" 2>/dev/null &
    elif command -v udhcpc &>/dev/null; then
        udhcpc -i "$WIFI_INTERFACE" -b -q &
    fi

    log "WiFi enabled"
}

# Disable WiFi
wifi_off() {
    log "Disabling WiFi..."

    # Kill DHCP client
    pkill -f "dhclient.*$WIFI_INTERFACE" 2>/dev/null || true
    pkill -f "udhcpc.*$WIFI_INTERFACE" 2>/dev/null || true

    # Disconnect from network
    wpa_cli -i "$WIFI_INTERFACE" disconnect 2>/dev/null || true

    # Bring down interface
    ip link set "$WIFI_INTERFACE" down 2>/dev/null || true

    log "WiFi disabled"
}

# Toggle WiFi
wifi_toggle() {
    if ip link show "$WIFI_INTERFACE" | grep -q "state UP"; then
        wifi_off
    else
        wifi_on
    fi
}

# Scan for networks
scan_networks() {
    log "Scanning for networks..."

    # Ensure interface is up
    ip link set "$WIFI_INTERFACE" up 2>/dev/null || true

    # Trigger scan
    iwlist "$WIFI_INTERFACE" scan 2>/dev/null | \
        grep -E "ESSID:|Quality=|Encryption:" | \
        sed 'N;N;s/\n/ /g' | \
        while read -r line; do
            local ssid=$(echo "$line" | grep -oP 'ESSID:"\K[^"]+')
            local quality=$(echo "$line" | grep -oP 'Quality=\K\d+')
            local encrypted=$(echo "$line" | grep -q "Encryption:on" && echo "🔒" || echo "🔓")

            if [ -n "$ssid" ]; then
                printf "%-30s %s %s%%\n" "$ssid" "$encrypted" "$quality"
            fi
        done
}

# Connect to a saved network
connect() {
    local ssid="$1"

    log "Connecting to $ssid..."

    # Find network ID
    local network_id=$(wpa_cli -i "$WIFI_INTERFACE" list_networks 2>/dev/null | \
                       grep -F "$ssid" | cut -f1)

    if [ -n "$network_id" ]; then
        wpa_cli -i "$WIFI_INTERFACE" select_network "$network_id"

        # Wait for connection
        for i in {1..10}; do
            sleep 1
            if iwgetid -r 2>/dev/null | grep -qF "$ssid"; then
                log "Connected to $ssid"

                # Request DHCP
                if command -v dhclient &>/dev/null; then
                    dhclient -4 "$WIFI_INTERFACE" 2>/dev/null &
                elif command -v udhcpc &>/dev/null; then
                    udhcpc -i "$WIFI_INTERFACE" -b -q &
                fi

                return 0
            fi
        done

        log "Connection timeout"
        return 1
    else
        log "Network not found in saved networks"
        return 1
    fi
}

# Add a new network
add_network() {
    local ssid="$1"
    local password="${2:-}"

    log "Adding network: $ssid"

    # Add network
    local network_id=$(wpa_cli -i "$WIFI_INTERFACE" add_network 2>/dev/null)

    wpa_cli -i "$WIFI_INTERFACE" set_network "$network_id" ssid "\"$ssid\""

    if [ -n "$password" ]; then
        wpa_cli -i "$WIFI_INTERFACE" set_network "$network_id" psk "\"$password\""
    else
        wpa_cli -i "$WIFI_INTERFACE" set_network "$network_id" key_mgmt NONE
    fi

    wpa_cli -i "$WIFI_INTERFACE" enable_network "$network_id"
    wpa_cli -i "$WIFI_INTERFACE" save_config

    log "Network added successfully"
}

# Remove a network
remove_network() {
    local ssid="$1"

    log "Removing network: $ssid"

    local network_id=$(wpa_cli -i "$WIFI_INTERFACE" list_networks 2>/dev/null | \
                       grep -F "$ssid" | cut -f1)

    if [ -n "$network_id" ]; then
        wpa_cli -i "$WIFI_INTERFACE" remove_network "$network_id"
        wpa_cli -i "$WIFI_INTERFACE" save_config
        log "Network removed"
    else
        log "Network not found"
    fi
}

# List saved networks
list_saved() {
    wpa_cli -i "$WIFI_INTERFACE" list_networks 2>/dev/null | tail -n +2
}

# Usage
usage() {
    echo "Usage: $0 <command> [options]"
    echo ""
    echo "Commands:"
    echo "  status        Show WiFi status"
    echo "  on            Enable WiFi"
    echo "  off           Disable WiFi"
    echo "  toggle        Toggle WiFi on/off"
    echo "  scan          Scan for available networks"
    echo "  connect       Connect to a saved network"
    echo "  add           Add a new network"
    echo "  remove        Remove a saved network"
    echo "  list          List saved networks"
    echo ""
    echo "Examples:"
    echo "  $0 toggle"
    echo "  $0 add \"MyNetwork\" \"password123\""
    echo "  $0 connect \"MyNetwork\""
}

main() {
    local command="${1:-status}"

    check_interface

    case "$command" in
        status)
            get_status
            ;;
        on)
            wifi_on
            ;;
        off)
            wifi_off
            ;;
        toggle)
            wifi_toggle
            ;;
        scan)
            scan_networks
            ;;
        connect)
            [ -z "${2:-}" ] && { usage; exit 1; }
            connect "$2"
            ;;
        add)
            [ -z "${2:-}" ] && { usage; exit 1; }
            add_network "$2" "${3:-}"
            ;;
        remove)
            [ -z "${2:-}" ] && { usage; exit 1; }
            remove_network "$2"
            ;;
        list)
            list_saved
            ;;
        *)
            usage
            exit 1
            ;;
    esac
}

main "$@"
