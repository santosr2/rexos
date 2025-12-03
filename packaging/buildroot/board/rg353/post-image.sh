#!/bin/bash
# RexOS Post-Image Script for RG353
# Creates the final SD card image

set -e

BOARD_DIR="$(dirname $0)"
BOARD_NAME="rg353"
GENIMAGE_CFG="${BOARD_DIR}/genimage.cfg"
GENIMAGE_TMP="${BUILD_DIR}/genimage.tmp"

# Clean up temporary directory
rm -rf "${GENIMAGE_TMP}"

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

    echo ""
    echo "============================================"
    echo "RexOS image created successfully!"
    echo "============================================"
    echo "Image: ${BINARIES_DIR}/rexos-${BOARD_NAME}.img.gz"
    echo ""
    echo "To flash:"
    echo "  gunzip -c rexos-${BOARD_NAME}.img.gz | sudo dd of=/dev/sdX bs=4M status=progress"
    echo ""
fi
