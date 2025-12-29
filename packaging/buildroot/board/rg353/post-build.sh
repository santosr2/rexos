#!/bin/bash
# RexOS Post-Build Script for RG353
# Run after building the root filesystem

set -e

BOARD_DIR="$(dirname $0)"
REXOS_ROOT="${TARGET_DIR}/rexos"

echo "RexOS post-build: Configuring system..."

# Create RexOS directory structure
mkdir -p "${REXOS_ROOT}"/{bin,lib,config,data,scripts}
mkdir -p "${REXOS_ROOT}"/lib/libretro/info
mkdir -p "${TARGET_DIR}"/rexos/{roms,bios,saves,states,screenshots}

# Copy default configuration
if [ -d "${BOARD_DIR}/../../../config" ]; then
    cp -r "${BOARD_DIR}/../../../config"/* "${REXOS_ROOT}/config/"
fi

# Create version file
VERSION=$(cat "${BOARD_DIR}/../../../VERSION" 2>/dev/null || echo "0.1.0-dev")
GIT_HASH=$(git -C "${BOARD_DIR}/../../.." rev-parse --short HEAD 2>/dev/null || echo "unknown")
echo "${VERSION}-${GIT_HASH}" > "${REXOS_ROOT}/version"

# Set up init system
# Create minimal /etc/inittab for BusyBox init (if using it as fallback)
cat > "${TARGET_DIR}/etc/inittab" << 'EOF'
# RexOS inittab
# Run RexOS init first, then first-boot script, then launcher
::sysinit:/rexos/bin/rexos-init
::wait:/rexos/scripts/runtime/install/first-boot.sh
::respawn:/rexos/bin/rexos-launcher
::shutdown:/rexos/scripts/runtime/system/shutdown.sh
::ctrlaltdel:/sbin/reboot
EOF

# Create symlinks for RexOS binaries
ln -sf /rexos/bin/rexos-init "${TARGET_DIR}/sbin/init" 2>/dev/null || true

# Copy scripts to correct location
if [ -d "${BOARD_DIR}/../../../scripts" ]; then
    mkdir -p "${REXOS_ROOT}/scripts"
    cp -r "${BOARD_DIR}/../../../scripts/runtime"/* "${REXOS_ROOT}/scripts/runtime/" 2>/dev/null || true
    find "${REXOS_ROOT}/scripts" -name "*.sh" -exec chmod +x {} \;
fi

# Set up udev rules for input devices
mkdir -p "${TARGET_DIR}/etc/udev/rules.d"
cat > "${TARGET_DIR}/etc/udev/rules.d/99-rexos-input.rules" << 'EOF'
# RexOS Input Device Rules

# Gamepads - allow user access
SUBSYSTEM=="input", ATTRS{name}=="*gamepad*", MODE="0666"
SUBSYSTEM=="input", ATTRS{name}=="*controller*", MODE="0666"
SUBSYSTEM=="input", ATTRS{name}=="*joystick*", MODE="0666"

# GPIO keys (device buttons)
SUBSYSTEM=="input", ATTRS{name}=="gpio-keys*", MODE="0666"

# ADC for analog sticks
SUBSYSTEM=="input", ATTRS{name}=="adc-keys*", MODE="0666"
EOF

# Set up audio configuration
mkdir -p "${TARGET_DIR}/etc/asound.conf.d"
cat > "${TARGET_DIR}/etc/asound.conf" << 'EOF'
# RexOS ALSA Configuration
pcm.!default {
    type hw
    card 0
}

ctl.!default {
    type hw
    card 0
}
EOF

# Create network configuration directory
mkdir -p "${TARGET_DIR}/etc/wpa_supplicant"
cat > "${TARGET_DIR}/etc/wpa_supplicant/wpa_supplicant.conf" << 'EOF'
ctrl_interface=/var/run/wpa_supplicant
ctrl_interface_group=0
update_config=1
country=US
EOF

# Set permissions
chmod 600 "${TARGET_DIR}/etc/wpa_supplicant/wpa_supplicant.conf"

# Create tmpfs mounts in fstab
cat > "${TARGET_DIR}/etc/fstab" << 'EOF'
# RexOS Filesystem Table
# <device>       <mount>           <type>   <options>                         <dump> <pass>
/dev/root        /                 auto     defaults,ro                       0      1
PARTLABEL=data   /rexos/data       ext4     defaults,noatime                  0      2
PARTLABEL=roms   /rexos/roms       exfat    defaults,noatime,uid=0,gid=0      0      0
tmpfs            /tmp              tmpfs    defaults,nosuid,nodev             0      0
tmpfs            /var/run          tmpfs    defaults,nosuid,nodev             0      0
tmpfs            /var/tmp          tmpfs    defaults,nosuid,nodev             0      0
tmpfs            /var/log          tmpfs    defaults,nosuid,nodev,size=16M    0      0
EOF

# Create mount points for data and roms partitions
mkdir -p "${TARGET_DIR}/rexos/data"
mkdir -p "${TARGET_DIR}/rexos/roms"

# Create symlinks for save data locations to data partition
mkdir -p "${TARGET_DIR}/rexos/saves"
mkdir -p "${TARGET_DIR}/rexos/states"
mkdir -p "${TARGET_DIR}/rexos/screenshots"
# Note: These will be symlinked to /rexos/data/ on first boot by init

# Set hostname
echo "rexos" > "${TARGET_DIR}/etc/hostname"

# Create hosts file
cat > "${TARGET_DIR}/etc/hosts" << 'EOF'
127.0.0.1       localhost
127.0.1.1       rexos
::1             localhost ip6-localhost ip6-loopback
EOF

# Create shell profile
cat > "${TARGET_DIR}/etc/profile" << 'EOF'
export PATH="/rexos/bin:/usr/bin:/bin:/sbin:/usr/sbin"
export HOME="/home/rexos"
export TERM="linux"
export PS1='\u@\h:\w\$ '

# RexOS environment
export REXOS_ROOT="/rexos"
export RETROARCH_CONFIG="/rexos/config/retroarch"
export LD_LIBRARY_PATH="/rexos/lib:/usr/lib"

# Aliases
alias ll='ls -la'
alias la='ls -A'
EOF

# Create rexos user home directory
mkdir -p "${TARGET_DIR}/home/rexos"

# Remove unnecessary files to reduce size
rm -rf "${TARGET_DIR}/usr/share/doc"
rm -rf "${TARGET_DIR}/usr/share/man"
rm -rf "${TARGET_DIR}/usr/share/info"
rm -rf "${TARGET_DIR}/usr/share/locale"

echo "RexOS post-build: Complete"
