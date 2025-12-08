# RexOS Device Trees for Anbernic RG353 Series

This directory contains Device Tree Source (DTS) files for the Anbernic RG353
series of handheld gaming devices, all based on the Rockchip RK3566 SoC.

## File Structure

```
dts/
├── rk3566-anbernic-rgxx3.dtsi      # Base for all Anbernic RK3566 handhelds
├── rk3566-anbernic-rg353x.dtsi     # Base for RG353 series (display, joysticks)
├── rk3566-anbernic-rg353p.dts      # RG353P specific (dual analog, eMMC, touch)
├── rk3566-anbernic-rg353ps.dts     # RG353PS specific (single analog, eMMC)
└── README.md                        # This file
```

## Device Variants

| Model | Analog Sticks | Storage | Touchscreen | Display |
|-------|--------------|---------|-------------|---------|
| RG353P | 2 (with L3/R3) | eMMC + SD | Yes | 640x480 IPS |
| RG353PS | 1 (left only) | eMMC + SD | No | 640x480 IPS |
| RG353V | 2 (with L3/R3) | SD only | Yes | 640x480 IPS |
| RG353VS | 1 (left only) | SD only | No | 640x480 IPS |
| RG353M | 2 (with L3/R3) | SD only | No | 640x480 IPS |

## Usage

### Building for a Specific Variant

Edit the buildroot defconfig to select the correct DTS:

```bash
# For RG353PS (default)
BR2_LINUX_KERNEL_CUSTOM_DTS_PATH="$(BR2_EXTERNAL_REXOS_PATH)/board/rg353/dts/rk3566-anbernic-rg353ps.dts"

# For RG353P
BR2_LINUX_KERNEL_CUSTOM_DTS_PATH="$(BR2_EXTERNAL_REXOS_PATH)/board/rg353/dts/rk3566-anbernic-rg353p.dts"
```

### Mainline Kernel Compatibility

These device trees are based on the mainline Linux kernel device trees
(available since Linux 6.1+). If using a recent mainline kernel, you can
use the kernel's built-in device trees instead:

- `arch/arm64/boot/dts/rockchip/rk3566-anbernic-rg353p.dts`
- `arch/arm64/boot/dts/rockchip/rk3566-anbernic-rg353ps.dts`
- `arch/arm64/boot/dts/rockchip/rk3566-anbernic-rg353v.dts`
- `arch/arm64/boot/dts/rockchip/rk3566-anbernic-rg353vs.dts`

## Hardware Details

### Common Hardware (All Variants)
- Rockchip RK3566 SoC (4x Cortex-A55 @ 1.8GHz)
- Mali-G52 GPU (Panfrost driver)
- 2GB LPDDR4 RAM
- RK817 PMIC with integrated audio codec
- RTL8821CS WiFi/Bluetooth
- Mini HDMI output
- USB-C port (OTG)
- 3.5mm headphone jack with mic
- PWM LEDs (power, charging, status)
- Haptic vibration motor

### RG353P/PS Specific
- 32GB eMMC internal storage
- 3472mAh battery (same capacity)
- RG353P has Hynitron CST340 touchscreen

### Input Configuration
- D-pad: GPIO buttons
- Face buttons (A/B/X/Y): GPIO buttons
- Shoulder buttons (L1/R1/L2/R2): GPIO buttons
- Analog sticks: ADC via multiplexer
- Volume: GPIO buttons
- Mode button: ADC

## References

- [Mainline Linux DTS](https://github.com/torvalds/linux/tree/master/arch/arm64/boot/dts/rockchip)
- [ArkOS](https://github.com/christianhaitian/arkos) - Community firmware
- [Batocera Linux](https://github.com/batocera-linux/batocera.linux) - Gaming distro

## License

SPDX-License-Identifier: (GPL-2.0+ OR MIT)
