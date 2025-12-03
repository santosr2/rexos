#!/bin/bash
# RexOS Bluetooth Manager Script
# Bluetooth controller and device management

set -euo pipefail

PAIRED_DEVICES_FILE="/rexos/config/bluetooth/paired_devices"

log() {
    echo "[BT] $1"
}

# Check if bluetooth service is available
check_bluetooth() {
    if ! command -v bluetoothctl &>/dev/null; then
        log "Error: bluetoothctl not found"
        exit 1
    fi

    # Check if bluetooth controller exists
    if ! bluetoothctl show &>/dev/null; then
        log "Error: No Bluetooth controller found"
        exit 1
    fi
}

# Get Bluetooth status
get_status() {
    local powered=$(bluetoothctl show 2>/dev/null | grep -oP 'Powered: \K\w+' || echo "no")
    local discoverable=$(bluetoothctl show 2>/dev/null | grep -oP 'Discoverable: \K\w+' || echo "no")

    echo "Bluetooth: $powered"
    echo "Discoverable: $discoverable"
    echo ""
    echo "Connected devices:"
    bluetoothctl devices Connected 2>/dev/null || echo "  None"
}

# Enable Bluetooth
bt_on() {
    log "Enabling Bluetooth..."

    # Unblock Bluetooth
    rfkill unblock bluetooth 2>/dev/null || true

    # Power on
    bluetoothctl power on

    # Enable agent
    bluetoothctl agent on
    bluetoothctl default-agent

    log "Bluetooth enabled"
}

# Disable Bluetooth
bt_off() {
    log "Disabling Bluetooth..."
    bluetoothctl power off
    log "Bluetooth disabled"
}

# Toggle Bluetooth
bt_toggle() {
    local powered=$(bluetoothctl show 2>/dev/null | grep -oP 'Powered: \K\w+' || echo "no")

    if [ "$powered" = "yes" ]; then
        bt_off
    else
        bt_on
    fi
}

# Scan for devices
scan_devices() {
    local duration="${1:-10}"

    log "Scanning for devices (${duration}s)..."

    # Start scanning
    bluetoothctl --timeout "$duration" scan on &
    local scan_pid=$!

    sleep "$duration"

    # List discovered devices
    echo ""
    echo "Discovered devices:"
    bluetoothctl devices 2>/dev/null | while read -r _ mac name; do
        local info=$(bluetoothctl info "$mac" 2>/dev/null)
        local icon=$(echo "$info" | grep -oP 'Icon: \K\w+' || echo "device")
        local paired=$(echo "$info" | grep -q "Paired: yes" && echo "✓" || echo " ")
        local connected=$(echo "$info" | grep -q "Connected: yes" && echo "●" || echo "○")

        # Determine device type
        local type="📱"
        case "$icon" in
            audio*) type="🎧" ;;
            input-gaming) type="🎮" ;;
            input-keyboard) type="⌨️" ;;
            input-mouse) type="🖱️" ;;
        esac

        printf "%s %s %s %-17s %s\n" "$connected" "$paired" "$type" "$mac" "$name"
    done

    kill $scan_pid 2>/dev/null || true
}

# Pair with a device
pair_device() {
    local mac="$1"

    log "Pairing with $mac..."

    # Trust and pair
    bluetoothctl trust "$mac"

    if bluetoothctl pair "$mac"; then
        log "Paired successfully"

        # Save to paired devices file
        mkdir -p "$(dirname "$PAIRED_DEVICES_FILE")"
        local name=$(bluetoothctl info "$mac" 2>/dev/null | grep -oP 'Name: \K.*' || echo "Unknown")
        echo "$mac $name" >> "$PAIRED_DEVICES_FILE"
        sort -u "$PAIRED_DEVICES_FILE" -o "$PAIRED_DEVICES_FILE"

        return 0
    else
        log "Pairing failed"
        return 1
    fi
}

# Connect to a paired device
connect_device() {
    local mac="$1"

    log "Connecting to $mac..."

    if bluetoothctl connect "$mac"; then
        log "Connected successfully"
        return 0
    else
        log "Connection failed"
        return 1
    fi
}

# Disconnect from a device
disconnect_device() {
    local mac="$1"

    log "Disconnecting from $mac..."
    bluetoothctl disconnect "$mac"
    log "Disconnected"
}

# Remove/unpair a device
remove_device() {
    local mac="$1"

    log "Removing device $mac..."

    # Disconnect first
    bluetoothctl disconnect "$mac" 2>/dev/null || true

    # Remove
    bluetoothctl remove "$mac"

    # Remove from saved file
    if [ -f "$PAIRED_DEVICES_FILE" ]; then
        grep -v "$mac" "$PAIRED_DEVICES_FILE" > "${PAIRED_DEVICES_FILE}.tmp" 2>/dev/null || true
        mv "${PAIRED_DEVICES_FILE}.tmp" "$PAIRED_DEVICES_FILE"
    fi

    log "Device removed"
}

# List paired devices
list_paired() {
    echo "Paired devices:"
    bluetoothctl devices Paired 2>/dev/null | while read -r _ mac name; do
        local connected=$(bluetoothctl info "$mac" 2>/dev/null | grep -q "Connected: yes" && echo "●" || echo "○")
        printf "%s %-17s %s\n" "$connected" "$mac" "$name"
    done
}

# Auto-connect to known devices
auto_connect() {
    log "Auto-connecting to paired devices..."

    bluetoothctl devices Paired 2>/dev/null | while read -r _ mac name; do
        local trusted=$(bluetoothctl info "$mac" 2>/dev/null | grep -q "Trusted: yes" && echo "yes" || echo "no")

        if [ "$trusted" = "yes" ]; then
            log "Attempting to connect to $name..."
            bluetoothctl connect "$mac" &>/dev/null &
        fi
    done

    log "Auto-connect initiated"
}

# Make device discoverable for pairing
make_discoverable() {
    local duration="${1:-60}"

    log "Making device discoverable for ${duration}s..."

    bluetoothctl discoverable on
    bluetoothctl pairable on

    (
        sleep "$duration"
        bluetoothctl discoverable off
        bluetoothctl pairable off
    ) &

    log "Device is now discoverable"
}

# Set device name
set_name() {
    local name="$1"

    log "Setting Bluetooth name to: $name"
    bluetoothctl system-alias "$name"
}

# Usage
usage() {
    echo "Usage: $0 <command> [options]"
    echo ""
    echo "Commands:"
    echo "  status          Show Bluetooth status"
    echo "  on              Enable Bluetooth"
    echo "  off             Disable Bluetooth"
    echo "  toggle          Toggle Bluetooth on/off"
    echo "  scan [seconds]  Scan for devices (default: 10s)"
    echo "  pair <mac>      Pair with a device"
    echo "  connect <mac>   Connect to a paired device"
    echo "  disconnect <mac> Disconnect from a device"
    echo "  remove <mac>    Remove/unpair a device"
    echo "  list            List paired devices"
    echo "  auto            Auto-connect to trusted devices"
    echo "  discoverable [s] Make device discoverable"
    echo "  name <name>     Set Bluetooth device name"
    echo ""
    echo "Examples:"
    echo "  $0 scan 15"
    echo "  $0 pair AA:BB:CC:DD:EE:FF"
    echo "  $0 connect AA:BB:CC:DD:EE:FF"
}

main() {
    local command="${1:-status}"

    check_bluetooth

    case "$command" in
        status)
            get_status
            ;;
        on)
            bt_on
            ;;
        off)
            bt_off
            ;;
        toggle)
            bt_toggle
            ;;
        scan)
            scan_devices "${2:-10}"
            ;;
        pair)
            [ -z "${2:-}" ] && { usage; exit 1; }
            pair_device "$2"
            ;;
        connect)
            [ -z "${2:-}" ] && { usage; exit 1; }
            connect_device "$2"
            ;;
        disconnect)
            [ -z "${2:-}" ] && { usage; exit 1; }
            disconnect_device "$2"
            ;;
        remove)
            [ -z "${2:-}" ] && { usage; exit 1; }
            remove_device "$2"
            ;;
        list)
            list_paired
            ;;
        auto)
            auto_connect
            ;;
        discoverable)
            make_discoverable "${2:-60}"
            ;;
        name)
            [ -z "${2:-}" ] && { usage; exit 1; }
            set_name "$2"
            ;;
        *)
            usage
            exit 1
            ;;
    esac
}

main "$@"
