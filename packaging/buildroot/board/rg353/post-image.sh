#!/bin/bash
# RexOS Post-Image Script for RG353
# Creates the final SD card image

set -e

BOARD_DIR="$(dirname $0)"
BOARD_NAME="rg353"
GENIMAGE_CFG="${BOARD_DIR}/genimage.cfg"
GENIMAGE_TMP="${BUILD_DIR}/genimage.tmp"

echo "RexOS post-image: Preparing boot files..."

# Copy extlinux.conf to binaries directory for genimage
cp "${BOARD_DIR}/extlinux.conf" "${BINARIES_DIR}/"

# Ensure DTB is available with expected name
# The kernel build may produce it with a different name
if [ -f "${BINARIES_DIR}/rk3566-anbernic-rg353ps.dtb" ]; then
    echo "  DTB: rk3566-anbernic-rg353ps.dtb found"
elif [ -f "${BINARIES_DIR}/rk3566-rg353.dtb" ]; then
    echo "  Renaming DTB to match extlinux.conf..."
    cp "${BINARIES_DIR}/rk3566-rg353.dtb" "${BINARIES_DIR}/rk3566-anbernic-rg353ps.dtb"
else
    # Try to find any RG353 DTB
    DTB_FILE=$(find "${BINARIES_DIR}" -name "*rg353*.dtb" -o -name "*rk3566*.dtb" 2>/dev/null | head -1)
    if [ -n "$DTB_FILE" ]; then
        echo "  Copying $(basename $DTB_FILE) to rk3566-anbernic-rg353ps.dtb"
        cp "$DTB_FILE" "${BINARIES_DIR}/rk3566-anbernic-rg353ps.dtb"
    else
        echo "Warning: No DTB file found! The image may not boot."
    fi
fi

# Verify kernel image exists
if [ ! -f "${BINARIES_DIR}/Image" ]; then
    echo "Error: Kernel Image not found!"
    exit 1
fi

# Check for U-Boot files
if [ ! -f "${BINARIES_DIR}/idbloader.img" ]; then
    echo "Warning: idbloader.img not found. U-Boot may need to be built separately."
fi
if [ ! -f "${BINARIES_DIR}/u-boot.itb" ]; then
    echo "Warning: u-boot.itb not found. U-Boot may need to be built separately."
fi

# Clean up temporary directory
rm -rf "${GENIMAGE_TMP}"

echo "RexOS post-image: Generating SD card image..."

# Generate the SD card image
genimage \
    --rootpath "${TARGET_DIR}" \
    --tmppath "${GENIMAGE_TMP}" \
    --inputpath "${BINARIES_DIR}" \
    --outputpath "${BINARIES_DIR}" \
    --config "${GENIMAGE_CFG}"

# Create compressed image
if [ -f "${BINARIES_DIR}/rexos-${BOARD_NAME}.img" ]; then
    echo "Compressing SD card image..."
    gzip -f -k "${BINARIES_DIR}/rexos-${BOARD_NAME}.img"

    # Create checksums
    cd "${BINARIES_DIR}"
    sha256sum "rexos-${BOARD_NAME}.img" > "rexos-${BOARD_NAME}.img.sha256"
    sha256sum "rexos-${BOARD_NAME}.img.gz" > "rexos-${BOARD_NAME}.img.gz.sha256"

    # Calculate image size
    IMG_SIZE=$(du -h "rexos-${BOARD_NAME}.img.gz" | cut -f1)

    echo ""
    echo "============================================"
    echo "RexOS image created successfully!"
    echo "============================================"
    echo ""
    echo "Image: ${BINARIES_DIR}/rexos-${BOARD_NAME}.img.gz ($IMG_SIZE)"
    echo ""
    echo "To flash to SD card:"
    echo "  1. Insert SD card and find its device (e.g., /dev/sdb)"
    echo "     lsblk"
    echo ""
    echo "  2. Flash the image (replace /dev/sdX with your device)"
    echo "     gunzip -c rexos-${BOARD_NAME}.img.gz | sudo dd of=/dev/sdX bs=4M status=progress"
    echo "     sync"
    echo ""
    echo "  3. Insert SD card into RG353 and power on"
    echo ""
fi
