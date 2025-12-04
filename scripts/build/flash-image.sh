#!/bin/bash
# RexOS Image Flash Script
# Creates and flashes SD card images for target devices

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
IMAGES_DIR="$PROJECT_ROOT/images"
WORK_DIR="/tmp/rexos-image-$$"

# Default values
DEVICE=""
IMAGE_SIZE="4G"
DEVICE_PROFILE=""

log() {
    echo "[FLASH] $1"
}

error() {
    echo "[ERROR] $1" >&2
    exit 1
}

cleanup() {
    log "Cleaning up..."
    sync
    umount "$WORK_DIR/boot" 2>/dev/null || true
    umount "$WORK_DIR/root" 2>/dev/null || true
    losetup -d "$LOOP_DEV" 2>/dev/null || true
    rm -rf "$WORK_DIR"
}
trap cleanup EXIT

# Check if running as root
check_root() {
    if [ "$(id -u)" -ne 0 ]; then
        error "This script must be run as root"
    fi
}

# Check dependencies
check_deps() {
    local deps=("losetup" "parted" "mkfs.fat" "mkfs.ext4" "dd")

    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &>/dev/null; then
            error "Missing dependency: $dep"
        fi
    done
}

# List available devices
list_devices() {
    echo "Available block devices:"
    echo ""
    lsblk -d -o NAME,SIZE,MODEL,TRAN | grep -E "sd|mmcblk" || true
    echo ""
    echo "WARNING: Be VERY careful to select the correct device!"
    echo "Selecting the wrong device could destroy your system!"
}

# Create blank image file
create_image() {
    local image_path="$1"
    local size="$2"

    log "Creating image file: $image_path ($size)"

    # Create sparse file
    truncate -s "$size" "$image_path"

    log "Image file created"
}

# Partition the image
partition_image() {
    local image="$1"

    log "Partitioning image..."

    # Create partition table
    parted -s "$image" mklabel msdos

    # Create boot partition (256MB FAT32)
    parted -s "$image" mkpart primary fat32 1MiB 257MiB
    parted -s "$image" set 1 boot on

    # Create root partition (rest of the space, ext4)
    parted -s "$image" mkpart primary ext4 257MiB 100%

    log "Partitioning complete"
}

# Format partitions
format_partitions() {
    log "Formatting partitions..."

    # Set up loop device
    LOOP_DEV=$(losetup --find --show --partscan "$IMAGE_PATH")
    log "Loop device: $LOOP_DEV"

    sleep 1  # Wait for partitions to appear

    # Format boot partition
    mkfs.fat -F 32 -n REXOS_BOOT "${LOOP_DEV}p1"

    # Format root partition
    mkfs.ext4 -L REXOS_ROOT "${LOOP_DEV}p2"

    log "Formatting complete"
}

# Mount partitions
mount_partitions() {
    log "Mounting partitions..."

    mkdir -p "$WORK_DIR/boot"
    mkdir -p "$WORK_DIR/root"

    mount "${LOOP_DEV}p1" "$WORK_DIR/boot"
    mount "${LOOP_DEV}p2" "$WORK_DIR/root"

    log "Partitions mounted"
}

# Install boot files (device specific)
install_boot() {
    local profile="$1"

    log "Installing boot files for $profile..."

    local boot_dir="$WORK_DIR/boot"

    # This would normally copy device-specific bootloader files
    # For now, create placeholder structure

    mkdir -p "$boot_dir/extlinux"

    # Create extlinux config
    cat > "$boot_dir/extlinux/extlinux.conf" << 'EOF'
LABEL RexOS
    LINUX /Image
    FDT /dtbs/device.dtb
    APPEND root=LABEL=REXOS_ROOT rootwait quiet
EOF

    log "Boot files installed"
}

# Install RexOS root filesystem
install_rootfs() {
    log "Installing RexOS root filesystem..."

    local root_dir="$WORK_DIR/root"

    # Create basic directory structure
    mkdir -p "$root_dir"/{bin,sbin,lib,lib64,etc,home,mnt,opt,proc,run,sys,tmp,usr,var}
    mkdir -p "$root_dir"/usr/{bin,sbin,lib,share}
    mkdir -p "$root_dir"/var/{log,tmp,cache}
    mkdir -p "$root_dir"/home/rexos
    mkdir -p "$root_dir"/rexos

    # Copy RexOS distribution
    local dist_dir="$PROJECT_ROOT/dist/rexos"
    if [ -d "$dist_dir" ]; then
        cp -r "$dist_dir"/* "$root_dir/rexos/"
        log "RexOS distribution copied"
    else
        log "Warning: No distribution found at $dist_dir"
        log "Run 'scripts/build/build.sh' first"
    fi

    # Create fstab
    cat > "$root_dir/etc/fstab" << 'EOF'
# RexOS fstab
LABEL=REXOS_ROOT    /           ext4    defaults,noatime    0   1
LABEL=REXOS_BOOT    /boot       vfat    defaults            0   2
tmpfs               /tmp        tmpfs   defaults,nosuid     0   0
tmpfs               /var/tmp    tmpfs   defaults,nosuid     0   0
EOF

    # Create hostname
    echo "rexos" > "$root_dir/etc/hostname"

    # Set permissions
    chmod 755 "$root_dir"
    chmod 1777 "$root_dir/tmp"
    chmod 1777 "$root_dir/var/tmp"

    log "Root filesystem installed"
}

# Finalize image
finalize_image() {
    log "Finalizing image..."

    sync

    # Unmount
    umount "$WORK_DIR/boot"
    umount "$WORK_DIR/root"

    # Detach loop device
    losetup -d "$LOOP_DEV"

    # Compress image
    log "Compressing image..."
    gzip -k "$IMAGE_PATH"

    # Create checksum
    sha256sum "$IMAGE_PATH" > "${IMAGE_PATH}.sha256"
    sha256sum "${IMAGE_PATH}.gz" > "${IMAGE_PATH}.gz.sha256"

    log "Image finalized"
}

# Flash image to device
flash_to_device() {
    local image="$1"
    local device="$2"

    log "Flashing image to $device..."

    echo ""
    echo "WARNING: This will DESTROY all data on $device!"
    echo "Device info:"
    lsblk "$device" 2>/dev/null || true
    echo ""
    read -p "Type 'YES' to confirm: " confirm

    if [ "$confirm" != "YES" ]; then
        error "Aborted by user"
    fi

    # Unmount any mounted partitions
    umount "${device}"* 2>/dev/null || true

    # Flash image
    if [ "${image##*.}" = "gz" ]; then
        log "Decompressing and flashing..."
        gunzip -c "$image" | dd of="$device" bs=4M status=progress conv=fsync
    else
        log "Flashing..."
        dd if="$image" of="$device" bs=4M status=progress conv=fsync
    fi

    sync

    log "Flash complete!"
    log "You can now remove the SD card and boot your device"
}

# Create full image
create_full_image() {
    local profile="$1"
    local output="$2"

    IMAGE_PATH="$output"

    log "Creating RexOS image for $profile"
    log "Output: $IMAGE_PATH"

    mkdir -p "$(dirname "$IMAGE_PATH")"
    mkdir -p "$WORK_DIR"

    create_image "$IMAGE_PATH" "$IMAGE_SIZE"
    partition_image "$IMAGE_PATH"
    format_partitions
    mount_partitions
    install_boot "$profile"
    install_rootfs
    finalize_image

    log ""
    log "=========================================="
    log "Image creation complete!"
    log "=========================================="
    log "Image: $IMAGE_PATH"
    log "Compressed: ${IMAGE_PATH}.gz"
    log ""
}

# Usage
usage() {
    echo "Usage: $0 <command> [options]"
    echo ""
    echo "Commands:"
    echo "  create <profile> <output>  Create SD card image"
    echo "  flash <image> <device>     Flash image to SD card"
    echo "  list                       List available block devices"
    echo ""
    echo "Profiles:"
    echo "  rg353    Anbernic RG353 series"
    echo "  rg35xx   Anbernic RG35XX series"
    echo "  generic  Generic ARM64"
    echo ""
    echo "Options:"
    echo "  --size <size>  Image size (default: 4G)"
    echo ""
    echo "Examples:"
    echo "  $0 create rg353 rexos-rg353.img"
    echo "  $0 flash rexos-rg353.img.gz /dev/sdb"
    echo "  $0 list"
}

main() {
    local command="${1:-}"

    case "$command" in
        create)
            check_root
            check_deps
            [ -z "${2:-}" ] || [ -z "${3:-}" ] && { usage; exit 1; }
            create_full_image "$2" "$3"
            ;;
        flash)
            check_root
            [ -z "${2:-}" ] || [ -z "${3:-}" ] && { usage; exit 1; }
            flash_to_device "$2" "$3"
            ;;
        list)
            list_devices
            ;;
        -h|--help|help)
            usage
            ;;
        *)
            usage
            exit 1
            ;;
    esac
}

main "$@"
