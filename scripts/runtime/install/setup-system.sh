#!/bin/bash
# RexOS System Setup Script
# Initial system configuration for fresh installations

set -euo pipefail

REXOS_ROOT="/rexos"
CONFIG_DIR="$REXOS_ROOT/config"
DATA_DIR="$REXOS_ROOT/data"
LOG_FILE="/var/log/rexos-setup.log"

log() {
    local msg="[SETUP] $1"
    echo "$msg"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $msg" >> "$LOG_FILE"
}

error() {
    log "ERROR: $1"
    exit 1
}

# Detect device type
detect_device() {
    log "Detecting device..."

    local device_model=""

    # Try to read device model from device tree
    if [ -f /proc/device-tree/model ]; then
        device_model=$(tr -d '\0' < /proc/device-tree/model)
    fi

    # Check for specific devices
    if echo "$device_model" | grep -qi "rg353"; then
        echo "rg353"
    elif echo "$device_model" | grep -qi "rg35xx"; then
        echo "rg35xx"
    elif echo "$device_model" | grep -qi "rk3566"; then
        echo "rk3566-generic"
    elif [ -f /sys/firmware/devicetree/base/compatible ]; then
        local compatible=$(tr '\0' '\n' < /sys/firmware/devicetree/base/compatible | head -1)
        echo "${compatible:-unknown}"
    else
        echo "unknown"
    fi
}

# Create directory structure
create_directories() {
    log "Creating directory structure..."

    local dirs=(
        "$REXOS_ROOT/bin"
        "$REXOS_ROOT/lib"
        "$CONFIG_DIR"
        "$CONFIG_DIR/retroarch"
        "$CONFIG_DIR/emulators"
        "$CONFIG_DIR/themes"
        "$CONFIG_DIR/networks"
        "$DATA_DIR"
        "$DATA_DIR/library.db"
        "$REXOS_ROOT/roms"
        "$REXOS_ROOT/bios"
        "$REXOS_ROOT/saves"
        "$REXOS_ROOT/states"
        "$REXOS_ROOT/screenshots"
        "$REXOS_ROOT/updates"
        "$REXOS_ROOT/backups"
        "$REXOS_ROOT/logs"
        "$REXOS_ROOT/keys"
    )

    for dir in "${dirs[@]}"; do
        mkdir -p "$dir"
    done

    log "Directories created"
}

# Create ROM system directories
create_rom_directories() {
    log "Creating ROM system directories..."

    local systems=(
        "gba:Game Boy Advance"
        "gbc:Game Boy Color"
        "gb:Game Boy"
        "nes:Nintendo Entertainment System"
        "snes:Super Nintendo"
        "n64:Nintendo 64"
        "nds:Nintendo DS"
        "psx:PlayStation"
        "psp:PlayStation Portable"
        "sega-md:Sega Genesis/Mega Drive"
        "sega-cd:Sega CD"
        "sega-32x:Sega 32X"
        "sega-gg:Sega Game Gear"
        "sega-ms:Sega Master System"
        "sega-saturn:Sega Saturn"
        "dreamcast:Sega Dreamcast"
        "arcade:Arcade"
        "mame:MAME"
        "fbneo:FinalBurn Neo"
        "neogeo:Neo Geo"
        "pce:PC Engine/TurboGrafx-16"
        "pcecd:PC Engine CD"
        "ngp:Neo Geo Pocket"
        "ngpc:Neo Geo Pocket Color"
        "wswan:WonderSwan"
        "wswanc:WonderSwan Color"
        "atari2600:Atari 2600"
        "atari5200:Atari 5200"
        "atari7800:Atari 7800"
        "lynx:Atari Lynx"
        "jaguar:Atari Jaguar"
        "coleco:ColecoVision"
        "intellivision:Intellivision"
        "msx:MSX"
        "zxspectrum:ZX Spectrum"
        "amiga:Amiga"
        "c64:Commodore 64"
        "dos:MS-DOS"
        "scummvm:ScummVM"
        "ports:Ports"
    )

    for entry in "${systems[@]}"; do
        local dir="${entry%%:*}"
        local name="${entry#*:}"

        mkdir -p "$REXOS_ROOT/roms/$dir"

        # Create info file
        echo "$name" > "$REXOS_ROOT/roms/$dir/.system_name"
    done

    log "ROM directories created"
}

# Generate default configuration
generate_default_config() {
    log "Generating default configuration..."

    local device=$(detect_device)

    # System config
    cat > "$CONFIG_DIR/system.toml" << EOF
# RexOS System Configuration
# Generated on $(date)

[system]
device = "$device"
timezone = "UTC"
language = "en"

[display]
brightness = 70
auto_brightness = false
screen_timeout = 300

[audio]
volume = 80
mute = false

[power]
sleep_timeout = 600
auto_poweroff = 0
low_battery_warning = 15
critical_battery = 5

[performance]
default_governor = "ondemand"
game_governor = "performance"
EOF

    # RetroArch base config
    cat > "$CONFIG_DIR/retroarch/retroarch.cfg" << 'EOF'
# RexOS RetroArch Configuration

# Video
video_fullscreen = "true"
video_vsync = "true"
video_max_swapchain_images = "2"
video_threaded = "true"

# Audio
audio_driver = "alsa"
audio_latency = "64"
audio_sync = "true"

# Input
input_autodetect_enable = "true"
input_enable_hotkey_btn = "6"
input_exit_emulator_btn = "7"
input_save_state_btn = "5"
input_load_state_btn = "4"
input_menu_toggle_btn = "3"

# Save states
savestate_auto_save = "false"
savestate_auto_load = "false"

# Paths
savefile_directory = "/rexos/saves"
savestate_directory = "/rexos/states"
screenshot_directory = "/rexos/screenshots"
system_directory = "/rexos/bios"

# UI
menu_driver = "ozone"
menu_show_advanced_settings = "false"
EOF

    log "Default configuration generated"
}

# Set up permissions
setup_permissions() {
    log "Setting up permissions..."

    # Make scripts executable
    find "$REXOS_ROOT/scripts" -name "*.sh" -exec chmod +x {} \; 2>/dev/null || true

    # Make binaries executable
    chmod +x "$REXOS_ROOT/bin"/* 2>/dev/null || true

    # Set appropriate ownership (run as root during install)
    if [ "$(id -u)" -eq 0 ]; then
        # Create rexos user if doesn't exist
        if ! id -u rexos &>/dev/null; then
            useradd -m -G audio,video,input,dialout rexos 2>/dev/null || true
        fi

        # Set ownership
        chown -R rexos:rexos "$REXOS_ROOT" 2>/dev/null || true

        # Writable directories
        chmod 755 "$REXOS_ROOT"
        chmod 775 "$REXOS_ROOT/roms"
        chmod 775 "$REXOS_ROOT/saves"
        chmod 775 "$REXOS_ROOT/states"
        chmod 775 "$REXOS_ROOT/screenshots"
    fi

    log "Permissions configured"
}

# Install RetroArch cores
install_cores() {
    log "Setting up RetroArch core directories..."

    local cores_dir="$REXOS_ROOT/lib/libretro"
    mkdir -p "$cores_dir"
    mkdir -p "$CONFIG_DIR/retroarch/cores"

    # Create core info directory
    mkdir -p "$REXOS_ROOT/lib/libretro/info"

    log "Core directories ready (cores need to be installed separately)"
}

# Set up auto-start
setup_autostart() {
    log "Setting up auto-start..."

    # Create systemd service if systemd is available
    if command -v systemctl &>/dev/null && [ -d /etc/systemd/system ]; then
        cat > /etc/systemd/system/rexos.service << 'EOF'
[Unit]
Description=RexOS Frontend
After=local-fs.target

[Service]
Type=simple
User=rexos
Environment=HOME=/home/rexos
WorkingDirectory=/rexos
ExecStart=/rexos/bin/rexos-launcher
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

        systemctl daemon-reload
        systemctl enable rexos.service

        log "Systemd service installed"
    else
        # Fallback to init.d script
        if [ -d /etc/init.d ]; then
            cat > /etc/init.d/rexos << 'EOF'
#!/bin/sh
### BEGIN INIT INFO
# Provides:          rexos
# Required-Start:    $local_fs
# Required-Stop:     $local_fs
# Default-Start:     2 3 4 5
# Default-Stop:      0 1 6
# Description:       RexOS Frontend
### END INIT INFO

case "$1" in
    start)
        /rexos/scripts/system/startup.sh &
        ;;
    stop)
        /rexos/scripts/system/shutdown.sh
        ;;
    restart)
        $0 stop
        $0 start
        ;;
esac
EOF
            chmod +x /etc/init.d/rexos

            # Enable on boot
            update-rc.d rexos defaults 2>/dev/null || true
        fi

        log "Init script installed"
    fi
}

# Verify installation
verify_installation() {
    log "Verifying installation..."

    local errors=0

    # Check directories
    for dir in bin lib config roms bios saves states; do
        if [ ! -d "$REXOS_ROOT/$dir" ]; then
            log "  Missing: $REXOS_ROOT/$dir"
            ((errors++))
        fi
    done

    # Check config files
    for file in system.toml retroarch/retroarch.cfg; do
        if [ ! -f "$CONFIG_DIR/$file" ]; then
            log "  Missing: $CONFIG_DIR/$file"
            ((errors++))
        fi
    done

    if [ $errors -eq 0 ]; then
        log "Installation verified successfully"
        return 0
    else
        log "Installation has $errors errors"
        return 1
    fi
}

# Full setup
full_setup() {
    log "Starting RexOS system setup..."
    log "Device: $(detect_device)"

    create_directories
    create_rom_directories
    generate_default_config
    setup_permissions
    install_cores

    # Only setup autostart if running as root
    if [ "$(id -u)" -eq 0 ]; then
        setup_autostart
    fi

    verify_installation

    log ""
    log "=========================================="
    log "RexOS setup complete!"
    log "=========================================="
    log ""
    log "Next steps:"
    log "  1. Add BIOS files to /rexos/bios"
    log "  2. Add ROM files to /rexos/roms/<system>"
    log "  3. Install RetroArch cores"
    log "  4. Reboot to start RexOS"
    log ""
}

# Usage
usage() {
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  full       Full system setup (default)"
    echo "  dirs       Create directory structure only"
    echo "  config     Generate default configuration only"
    echo "  verify     Verify installation"
    echo "  detect     Detect device type"
    echo ""
}

main() {
    local command="${1:-full}"

    case "$command" in
        full)
            full_setup
            ;;
        dirs)
            create_directories
            create_rom_directories
            ;;
        config)
            generate_default_config
            ;;
        verify)
            verify_installation
            ;;
        detect)
            detect_device
            ;;
        *)
            usage
            exit 1
            ;;
    esac
}

main "$@"
