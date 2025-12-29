# Building RexOS

This guide explains how to build RexOS from source and create a bootable SD card image.

## Prerequisites

### Build Host Requirements

- **Operating System**: Linux (Ubuntu 22.04+, Debian 12+, Fedora 38+, or Arch Linux)
- **Disk Space**: ~15GB free space
- **RAM**: 4GB minimum, 8GB recommended
- **Internet**: Required for downloading sources

### Install Build Dependencies

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install build-essential git wget unzip rsync cpio bc python3 \
    libncurses-dev libssl-dev flex bison
```

**Fedora:**
```bash
sudo dnf install make gcc gcc-c++ git wget unzip rsync cpio bc python3 \
    ncurses-devel openssl-devel flex bison
```

**Arch Linux:**
```bash
sudo pacman -S base-devel git wget unzip rsync cpio bc python ncurses openssl flex bison
```

### Optional: Install Rust (for faster builds)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustup target add aarch64-unknown-linux-gnu
```

## Quick Start

### Build Complete Image

```bash
# Clone the repository
git clone https://github.com/santosr2/rexos.git
cd rexos

# Build the SD card image (takes 1-2 hours on first build)
./scripts/build/build-image.sh
```

The build output will be in `build/output/images/`.

### Flash to SD Card

```bash
# Find your SD card device (usually /dev/sdb or /dev/mmcblk0)
lsblk

# Flash the image (replace /dev/sdX with your SD card)
gunzip -c build/output/images/rexos-rg353.img.gz | sudo dd of=/dev/sdX bs=4M status=progress
sync
```

## Build Options

### Target Devices

```bash
# RG353 series (default) - RG353M, RG353V, RG353VS, RG353PS
./scripts/build/build-image.sh -d rg353

# RG35XX series (future) - RG35XX, RG35XX+, RG35XX-H
./scripts/build/build-image.sh -d rg35xx
```

### Build Commands

```bash
# Full build
./scripts/build/build-image.sh

# Clean and rebuild
./scripts/build/build-image.sh clean all

# Interactive configuration (Buildroot menuconfig)
./scripts/build/build-image.sh menuconfig

# Download sources only (for offline building later)
./scripts/build/build-image.sh download

# Download pre-built libretro cores
./scripts/build/build-image.sh cores

# Specify parallel jobs
./scripts/build/build-image.sh -j 8
```

### Environment Variables

```bash
# Custom Buildroot version
BUILDROOT_VERSION=2024.02.1 ./scripts/build/build-image.sh

# Custom build directory
BUILD_BASE=/path/to/build ./scripts/build/build-image.sh

# Specify parallel jobs
PARALLEL_JOBS=8 ./scripts/build/build-image.sh
```

## Building Components Separately

### Build Rust Components Only

```bash
# Native build (for testing on host)
cargo build --release

# Cross-compile for RG353 (aarch64)
cross build --target aarch64-unknown-linux-gnu --release

# Cross-compile for RG35XX (armv7)
cross build --target armv7-unknown-linux-gnueabihf --release
```

### Download Libretro Cores

```bash
# Download cores for aarch64 (RG353)
./scripts/build/download-cores.sh --arch aarch64

# Download cores for armv7 (RG35XX)
./scripts/build/download-cores.sh --arch armv7-neon-hf

# Download all cores including optional ones
./scripts/build/download-cores.sh --all
```

## SD Card Layout

The generated image has the following partition layout:

| Partition | Type     | Size   | Purpose                      |
|-----------|----------|--------|------------------------------|
| idbloader | raw      | 3.5MB  | First-stage bootloader       |
| uboot     | raw      | 4MB    | U-Boot bootloader            |
| boot      | FAT32    | 64MB   | Kernel, DTB, extlinux config |
| rootfs    | SquashFS | 256MB  | Read-only root filesystem    |
| data      | ext4     | 256MB  | User data (saves, config)    |
| roms      | exFAT    | Rest   | ROM files                    |

## Directory Structure on Device

```
/
├── rexos/
│   ├── bin/           # RexOS binaries
│   ├── lib/
│   │   └── libretro/  # Libretro cores
│   ├── config/        # Configuration files
│   ├── data/          # Runtime data
│   ├── scripts/       # System scripts
│   ├── roms/          # Symlink to ROM partition
│   ├── bios/          # BIOS files
│   ├── saves/         # Save files
│   └── states/        # Save states
└── ...
```

## Customization

### Adding Custom Packages

1. Create a new package in `packaging/buildroot/package/`:
   ```
   packaging/buildroot/package/my-package/
   ├── Config.in
   └── my-package.mk
   ```

2. Add to `packaging/buildroot/Config.in`:
   ```
   source "$BR2_EXTERNAL_REXOS_PATH/package/my-package/Config.in"
   ```

3. Enable in defconfig:
   ```
   BR2_PACKAGE_MY_PACKAGE=y
   ```

### Modifying Kernel Config

```bash
# Start menuconfig for kernel
./scripts/build/build-image.sh menuconfig
# Navigate to: Kernel > Linux Kernel > Kernel configuration

# Or edit directly:
vi packaging/buildroot/board/rg353/linux.config
```

### Device Tree Modifications

Device tree sources are in `packaging/buildroot/board/rg353/dts/`:
- `rk3566-anbernic-rgxx3.dtsi` - Base config for all RG353
- `rk3566-anbernic-rg353x.dtsi` - RG353 specific features
- `rk3566-anbernic-rg353ps.dts` - RG353PS variant

## Troubleshooting

### Build Fails with "Out of Memory"

Reduce parallel jobs:
```bash
./scripts/build/build-image.sh -j 2
```

### Missing Dependencies

The build script checks for dependencies. If something is missing:
```bash
# Ubuntu/Debian
sudo apt install build-essential libncurses-dev libssl-dev

# Check what's missing
./scripts/build/build-image.sh 2>&1 | head -20
```

### Network Errors During Download

Use a VPN or mirror:
```bash
# Download sources first
./scripts/build/build-image.sh download

# Then build offline (may need additional setup)
./scripts/build/build-image.sh
```

### Device Won't Boot

1. Verify the image was written correctly:
   ```bash
   sha256sum build/output/images/rexos-rg353.img.gz
   # Compare with the .sha256 file
   ```

2. Check SD card health:
   ```bash
   sudo badblocks -v /dev/sdX
   ```

3. Try a different SD card (Class 10 or better recommended)

### RetroArch Cores Missing

Download cores manually:
```bash
./scripts/build/download-cores.sh --arch aarch64 --output build/cores
```

## Development Workflow

### Incremental Builds

After the initial build, subsequent builds are much faster:
```bash
# Rebuild only changed packages
./scripts/build/build-image.sh

# Rebuild specific package
make -C build/buildroot-*/output O=$(pwd)/build/output \
    BR2_EXTERNAL=$(pwd)/packaging/buildroot \
    rexos-core-rebuild
```

### Testing Changes Quickly

For rapid iteration, build Rust components natively and test:
```bash
# Build and test on host
cargo build --release
cargo test

# Then do a full image build for hardware testing
./scripts/build/build-image.sh
```

## Supported Devices

| Device        | SoC     | Status    | Config                     |
|---------------|---------|-----------|----------------------------|
| RG353M        | RK3566  | Supported | `rexos_rg353_defconfig`    |
| RG353V        | RK3566  | Supported | `rexos_rg353_defconfig`    |
| RG353VS       | RK3566  | Supported | `rexos_rg353_defconfig`    |
| RG353PS       | RK3566  | Supported | `rexos_rg353_defconfig`    |
| RG35XX        | H700    | Planned   | `rexos_rg35xx_defconfig`   |
| RG35XX Plus   | H700    | Planned   | `rexos_rg35xx_defconfig`   |

## Resources

- [Buildroot Manual](https://buildroot.org/downloads/manual/manual.html)
- [Rockchip RK3566 Documentation](https://opensource.rock-chips.com/)
- [LibRetro Documentation](https://docs.libretro.com/)
- [RetroArch Documentation](https://docs.libretro.com/guides/retroarch/)
