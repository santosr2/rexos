# RexOS Base System Design

## Philosophy

RexOS is designed as a **purpose-built gaming OS**, not a general-purpose Linux distribution. This means:

- No package manager at runtime (all software is built into the image)
- Minimal base system (~20-50MB root filesystem)
- Fast boot times (<5 seconds to launcher)
- Maximum resources available for emulation

## Recommended Build Systems

### Primary: Buildroot

Buildroot is the recommended approach for RexOS because:

1. **Minimal footprint**: Only includes what you explicitly configure
2. **Fast builds**: Simpler than Yocto, faster iteration
3. **Well-suited for gaming devices**: Used by many similar projects (Batocera, Lakka, EmuELEC)
4. **Custom toolchain**: Optimized for target architecture

```
Target Image Size:
- Root filesystem: ~50MB (uncompressed)
- Boot partition: ~20MB
- Total SD card minimum: 2GB (rest for ROMs/saves)
```

### Alternative: Yocto/OpenEmbedded

Use Yocto if you need:
- Commercial support requirements
- BSP integration from SoC vendors (Rockchip provides Yocto layers)
- More complex package management

## Base System Components

### C Library: musl vs glibc

| Component | musl | glibc |
|-----------|------|-------|
| Size | ~1MB | ~10MB |
| Performance | Slightly faster startup | Better runtime perf |
| Compatibility | Some issues with proprietary blobs | Full compatibility |

**Recommendation**: Use **glibc** for RexOS due to:
- Better compatibility with RetroArch cores
- Required for some proprietary GPU drivers (Mali)
- More predictable behavior with emulators

### Init System

RexOS uses a **custom init** (`rexos-init`) instead of systemd/OpenRC:

```
Boot Sequence:
1. Kernel loads
2. rexos-init (PID 1) starts
3. Mount filesystems (proc, sys, dev, tmpfs)
4. Initialize hardware (display, input, audio)
5. Start minimal services (udev, dbus if needed)
6. Launch frontend (EmulationStation or rexos-launcher)

Target: Kernel → Launcher in <3 seconds
```

### Core Utilities: BusyBox

Use BusyBox for core utilities:
- Single binary provides: sh, mount, ls, cp, etc.
- Size: ~1MB vs ~50MB for GNU coreutils
- Sufficient for gaming OS needs

## Filesystem Layout

```
/
├── bin/                    # BusyBox symlinks + rexos binaries
├── lib/                    # Shared libraries (glibc, SDL2, etc.)
│   └── libretro/          # RetroArch cores
├── etc/                    # Minimal config
│   ├── init.d/            # Init scripts (if using OpenRC)
│   └── rexos/             # RexOS configuration
├── dev/                    # Device nodes (devtmpfs)
├── proc/                   # Procfs
├── sys/                    # Sysfs
├── tmp/                    # Tmpfs
├── var/                    # Runtime data (tmpfs or persistent)
│   ├── log/
│   └── run/
├── rexos/                  # RexOS application directory
│   ├── bin/               # RexOS binaries
│   ├── lib/               # RexOS libraries
│   ├── config/            # User configuration
│   └── data/              # Application data
└── roms/                   # Symlink to /mnt/ROMS or SD card
```

## Kernel Configuration

Minimal kernel config for RG353/RK3566:

```
# Required
CONFIG_MMC=y                    # SD card support
CONFIG_DRM=y                    # Display
CONFIG_INPUT_EVDEV=y            # Input devices
CONFIG_SND=y                    # Audio
CONFIG_USB=y                    # USB support
CONFIG_EXT4_FS=y               # Filesystem
CONFIG_VFAT_FS=y               # FAT for boot partition

# Performance
CONFIG_PREEMPT=y               # Lower latency
CONFIG_HZ_1000=y               # 1000Hz timer
CONFIG_NO_HZ_IDLE=y            # Tickless idle

# Disable (save space/boot time)
CONFIG_NETWORK_FILESYSTEMS=n   # No NFS, CIFS
CONFIG_DEBUG_KERNEL=n          # No debug
CONFIG_PRINTK=y                # Keep minimal logging
CONFIG_MODULES=y               # Loadable modules (for WiFi)

# Size optimization
CONFIG_CC_OPTIMIZE_FOR_SIZE=y
CONFIG_KERNEL_XZ=y             # XZ compressed kernel
```

## Build Configuration

### Buildroot defconfig

```makefile
# Target
BR2_aarch64=y
BR2_cortex_a55=y
BR2_ARM_FPU_NEON_FP_ARMV8=y

# Toolchain
BR2_TOOLCHAIN_BUILDROOT_GLIBC=y
BR2_TOOLCHAIN_BUILDROOT_CXX=y
BR2_GCC_VERSION_13_X=y

# System
BR2_INIT_NONE=y                # Custom init
BR2_ROOTFS_DEVICE_CREATION_DYNAMIC_EUDEV=y
BR2_TARGET_GENERIC_GETTY=n     # No login console
BR2_SYSTEM_BIN_SH_BUSYBOX=y

# Kernel
BR2_LINUX_KERNEL=y
BR2_LINUX_KERNEL_CUSTOM_GIT=y
BR2_LINUX_KERNEL_CUSTOM_REPO_URL="https://github.com/rockchip-linux/kernel.git"
BR2_LINUX_KERNEL_CUSTOM_REPO_VERSION="develop-5.10"

# Packages
BR2_PACKAGE_BUSYBOX=y
BR2_PACKAGE_SDL2=y
BR2_PACKAGE_SDL2_KMSDRM=y
BR2_PACKAGE_ALSA_LIB=y
BR2_PACKAGE_RETROARCH=y

# Filesystem
BR2_TARGET_ROOTFS_SQUASHFS=y   # Read-only root
BR2_TARGET_ROOTFS_SQUASHFS_XZ=y
```

## Performance Optimizations

### Boot Time Reduction

1. **No initramfs**: Boot directly to rootfs
2. **Parallel init**: Start services concurrently
3. **Lazy loading**: Defer non-critical initialization
4. **Kernel cmdline**: `quiet loglevel=0 fastboot`

### Runtime Performance

1. **CPU Governor**: Set to `performance` during gameplay
2. **GPU Driver**: Use Panfrost (open) or Mali blob
3. **Memory**: No swap, maximize for emulation
4. **I/O Scheduler**: `none` or `mq-deadline` for SD cards

### Power Management

1. **CPU idle states**: Enable C-states for battery life
2. **GPU DVFS**: Dynamic frequency scaling
3. **Display**: Automatic brightness, timeout to blank

## Comparison with Existing Projects

| Project | Base | Size | Boot Time | Notes |
|---------|------|------|-----------|-------|
| ArkOS | Debian | ~2GB | ~30s | Full apt, lots of tools |
| Batocera | Buildroot | ~500MB | ~15s | Mature, good reference |
| Lakka | LibreELEC | ~200MB | ~10s | RetroArch-only |
| EmuELEC | CoreELEC | ~500MB | ~15s | Amlogic focused |
| **RexOS** | Buildroot | ~100MB | <5s | Purpose-built, Rust core |

## Migration Path

For development, you can use:
1. **Debian/Ubuntu** for initial development and testing
2. **Docker** container with Buildroot for reproducible builds
3. **QEMU** for testing ARM images on x86

Production images should always use Buildroot for optimal performance.
