# RexOS Quick Start Guide

Get your retro gaming device up and running with RexOS in minutes.

## Requirements

- Anbernic RG353M/V/VS or RG35XX series device
- MicroSD card (16GB or larger recommended)
- Computer with SD card reader
- USB cable for initial setup (optional)

## Installation

### Option 1: Pre-built Image (Recommended)

1. **Download the image** for your device from the [releases page](https://github.com/santosr2/rexos/releases):
   - RG353 series: `rexos-<version>-rg353.img.gz`
   - RG35XX series: `rexos-<version>-rg35xx.img.gz`

2. **Flash the image** to your SD card:

   **Using Balena Etcher (Windows/Mac/Linux):**
   - Download and install [Balena Etcher](https://www.balena.io/etcher/)
   - Select the downloaded `.img.gz` file
   - Select your SD card
   - Click "Flash!"

   **Using dd (Linux/Mac):**
   ```bash
   gunzip -c rexos-<version>-<device>.img.gz | sudo dd of=/dev/sdX bs=4M status=progress
   sync
   ```

3. **Insert the SD card** into your device and power on

### Option 2: Update from Existing OS

If you're running a compatible Linux-based OS:

```bash
# Download the update package
wget https://github.com/santosr2/rexos/releases/download/v<version>/rexos-<version>-<arch>.tar.gz

# Apply the update
sudo tar -xzf rexos-<version>-<arch>.tar.gz -C /

# Run setup
sudo /rexos/scripts/install/setup-system.sh

# Reboot
sudo reboot
```

## First Boot

1. **Initial Setup**: RexOS will automatically detect your device and configure settings
2. **Wait for Setup**: The first boot takes 1-2 minutes while the system initializes
3. **Game Browser**: You'll see the main game browser interface

## Adding Games

### Using SD Card

1. Power off your device
2. Remove the SD card and insert it into your computer
3. Copy ROM files to the appropriate folders:
   ```
   /rexos/roms/gba/       - Game Boy Advance
   /rexos/roms/snes/      - Super Nintendo
   /rexos/roms/psx/       - PlayStation
   /rexos/roms/arcade/    - Arcade games
   ... etc
   ```
4. Insert the SD card back into your device
5. Games will be automatically detected on next boot

### Using WiFi Transfer

1. Connect to WiFi (see below)
2. Note your device's IP address
3. Use SFTP/SCP to transfer files:
   ```bash
   scp game.gba rexos@<device-ip>:/rexos/roms/gba/
   ```

## Connecting to WiFi

1. Press **SELECT + X** to open the menu
2. Navigate to **Settings > Network > WiFi**
3. Select **Scan for Networks**
4. Choose your network and enter the password

Or use the command line:
```bash
/rexos/scripts/utils/wifi-toggle.sh add "MyNetwork" "password123"
/rexos/scripts/utils/wifi-toggle.sh connect "MyNetwork"
```

## Controls

### Default Hotkeys

All hotkeys use **SELECT** as the modifier button:

| Combination | Action |
|------------|--------|
| SELECT + START | Exit game |
| SELECT + R1 | Save state |
| SELECT + L1 | Load state |
| SELECT + X | Menu |
| SELECT + Y | Pause |
| SELECT + R2 | Fast forward |
| SELECT + L2 | Screenshot |
| SELECT + D-Pad Up | Volume up |
| SELECT + D-Pad Down | Volume down |
| SELECT + D-Pad Right | Brightness up |
| SELECT + D-Pad Left | Brightness down |

### Game Browser Navigation

| Button | Action |
|--------|--------|
| D-Pad | Navigate |
| A | Select/Launch |
| B | Back |
| X | Options |
| Y | Search |
| L1/R1 | Page up/down |

## BIOS Files

Some emulators require BIOS files. Place them in `/rexos/bios/`:

| System | File | Location |
|--------|------|----------|
| PlayStation | `scph1001.bin` | `/rexos/bios/` |
| GBA | `gba_bios.bin` | `/rexos/bios/` |
| Sega CD | `bios_CD_U.bin` | `/rexos/bios/` |

## Power Management

### Sleep Mode
- Short press the power button to sleep
- Press again to wake

### Power Off
- Long press the power button (3 seconds)
- Or: SELECT + Power button

### Battery Status
Battery level is shown in the top bar of the game browser.

## Performance Modes

Switch between performance profiles:

```bash
/rexos/scripts/utils/performance-mode.sh performance  # Maximum performance
/rexos/scripts/utils/performance-mode.sh balanced     # Balanced
/rexos/scripts/utils/performance-mode.sh powersave    # Maximum battery life
```

## Troubleshooting

### Device Won't Boot
- Ensure the SD card is properly inserted
- Try re-flashing the image
- Check if SD card is compatible (some cheap cards have issues)

### Games Not Showing
- Check that ROM files are in the correct folder
- Verify file extensions are supported
- Try running a library rescan from settings

### No Sound
- Check volume (SELECT + D-Pad Up)
- Ensure headphones are properly connected
- Try toggling mute (Settings > Audio)

### WiFi Not Working
- Verify your network name and password
- Try moving closer to your router
- Check if WiFi is enabled in settings

## Getting Updates

RexOS can update itself over WiFi:

1. Connect to WiFi
2. Go to **Settings > System > Check for Updates**
3. If an update is available, select **Install**
4. Wait for download and installation
5. Reboot when prompted

Or manually:
```bash
/rexos/scripts/update/check-updates.sh
/rexos/scripts/update/apply-update.sh /rexos/updates/rexos-<version>.tar.gz
```

## Backup and Restore

### Create Backup
```bash
/rexos/scripts/maintenance/backup.sh create
```

### Restore Backup
```bash
/rexos/scripts/maintenance/backup.sh restore
```

## Getting Help

- **Documentation**: See the `/rexos/docs` folder
- **GitHub Issues**: Report bugs at github.com/santosr2/rexos/issues
- **Community**: Join our Discord (coming soon)

Enjoy your retro gaming experience with RexOS!
