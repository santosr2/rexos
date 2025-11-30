# Feature Specification

## Core Features

### 1. Fast Boot System ⚡
**Priority**: High  
**Target**: < 10 seconds from power-on to UI

#### Requirements
- Optimized U-Boot configuration
- Minimal kernel modules
- Lazy service initialization
- Cached library index
- Splash screen immediately on display

#### Implementation Notes
- Use initramfs for critical components
- Defer non-essential services
- Preload frequently used data

---

### 2. Hardware Abstraction Layer (HAL) 🔧
**Priority**: Critical  
**Status**: Design Phase

#### Supported Hardware
- **Input**: GPIO buttons, analog sticks, touch screens
- **Display**: RGB565/888 framebuffer, HDMI output
- **Audio**: I2S, analog output
- **Storage**: SD/eMMC, USB storage
- **Network**: WiFi (RTL8xxx, etc), Bluetooth
- **Power**: Battery monitoring, charging detection

#### Device Profiles
```toml
# Example: RG353M profile
[device.rg353m]
chipset = "RK3566"
display = { width = 640, height = 480, format = "RGB565" }
buttons = ["up", "down", "left", "right", "a", "b", "x", "y", "l1", "l2", "r1", "r2", "start", "select"]
analog_sticks = 2
battery_capacity = 3500
```

---

### 3. Game Library Manager 📚
**Priority**: High  
**Status**: Planning

#### Features
- **Fast Scanning**: Multi-threaded ROM scanning
- **Metadata**: Scraping from ScreenScraper, TheGamesDB
- **Collections**: Custom collections, favorites
- **Search**: Full-text search, filters
- **History**: Recently played, play counts

#### Database Schema
```sql
CREATE TABLE games (
    id INTEGER PRIMARY KEY,
    path TEXT NOT NULL,
    system TEXT NOT NULL,
    name TEXT,
    description TEXT,
    release_date DATE,
    developer TEXT,
    publisher TEXT,
    genre TEXT,
    players INTEGER,
    rating REAL,
    last_played TIMESTAMP,
    play_count INTEGER DEFAULT 0
);
```

---

### 4. Emulator Management 🎮
**Priority**: Critical  
**Status**: Design

#### Supported Emulators
- **RetroArch**: Primary emulation frontend
- **Standalone**: PPSSPP, DraStic, etc.

#### Core Selection
```yaml
# System-to-core mapping
gb:
  default: gambatte
  alternatives: [sameboy, tgbdual, gearboy]
  
gba:
  default: mgba
  alternatives: [vba-next, vba-m, gpsp]

n64:
  default: mupen64plus-next
  alternatives: [parallel-n64]
```

#### Per-Game Configuration
- Custom core selection
- Performance profiles
- Save state slots
- Shader presets

---

### 5. Input System 🕹️
**Priority**: Critical  
**Status**: Planning

#### Features
- Hotkey support (save/load states, exit, etc.)
- Per-emulator button mapping
- Analog stick calibration
- Dead zone configuration
- Turbo button support

#### Hotkey Examples
```toml
[hotkeys]
exit = "Select + Start"
save_state = "Select + R1"
load_state = "Select + L1"
fast_forward = "Select + R2"
screenshot = "Select + L2"
```

---

### 6. Display Management 📺
**Priority**: High  
**Status**: Design

#### Features
- Resolution switching
- Brightness control
- Color temperature adjustment
- Integer scaling option
- Aspect ratio correction
- Bezel/overlay support

---

### 7. Audio System 🔊
**Priority**: High  
**Status**: Planning

#### Features
- Volume control (master, per-app)
- Audio profiles (headphones, speaker)
- Sample rate switching
- Low-latency mode

---

### 8. Network Services 🌐
**Priority**: Medium  
**Status**: Future

#### Features
- **WiFi Manager**: Easy network configuration
- **File Sharing**: Samba/SMB for ROM transfer
- **SSH**: Remote access for debugging
- **Web UI**: Browser-based file manager
- **Cloud Sync**: Save state synchronization

---

### 9. Update System 🔄
**Priority**: High  
**Status**: Design

#### Features
- Automatic update checking
- Delta updates (bandwidth efficient)
- Rollback capability
- Update channels (stable, beta, nightly)
- Component updates (cores, themes, BIOS)

#### Update Process
```
Check for updates (background)
    ↓
Notify user
    ↓
Download (with progress)
    ↓
Verify signature
    ↓
Create recovery point
    ↓
Apply update
    ↓
Verify installation
    ↓
Reboot if needed
```

---

### 10. Power Management 🔋
**Priority**: High  
**Status**: Planning

#### Features
- Battery percentage display
- Low battery warnings
- Auto-suspend on idle
- Performance profiles:
  - **Powersave**: Extend battery life
  - **Balanced**: Default mode
  - **Performance**: Maximum speed

#### Estimated Battery Life (3500mAh)
- **Powersave**: 8-10 hours (2D games)
- **Balanced**: 5-7 hours (most games)
- **Performance**: 3-4 hours (demanding systems)

---

### 11. Save State Management 💾
**Priority**: High  
**Status**: Design

#### Features
- Quick save/load (per slot)
- Screenshot thumbnails
- State management UI
- Auto-save on exit
- Cloud backup (optional)

---

### 12. Theme Support 🎨
**Priority**: Medium  
**Status**: Future

#### Features
- EmulationStation themes compatibility
- Custom theme creation
- Dynamic theme switching
- Per-system themes

---

### 13. Scraper Integration 🖼️
**Priority**: Medium  
**Status**: Future

#### Supported Sources
- ScreenScraper
- TheGamesDB
- Local metadata

#### Scraped Data
- Box art
- Screenshots
- Descriptions
- Release dates
- Ratings

---

### 14. RetroAchievements 🏆
**Priority**: Low  
**Status**: Future

#### Features
- Achievement unlocking
- Leaderboards
- Rich presence
- Progress tracking

---

### 15. Advanced Features (Phase 3)
- Port Manager (like PortMaster)
- Netplay support
- Screenshot/video recording
- Custom shaders
- Cheat support
- Per-game notes
- Playlist management

---

## Feature Priority Matrix

| Feature | Priority | Complexity | Phase |
|---------|----------|------------|-------|
| HAL | Critical | High | 1 |
| Boot System | High | Medium | 1 |
| Input System | Critical | Medium | 1 |
| Display | High | Medium | 1 |
| Audio | High | Medium | 1 |
| Library Manager | High | High | 1-2 |
| Emulator Management | Critical | High | 1 |
| Power Management | High | Medium | 2 |
| Update System | High | High | 2 |
| Network Services | Medium | High | 2 |
| Themes | Medium | Low | 2 |
| Scraper | Medium | Medium | 2 |
| RetroAchievements | Low | Medium | 3 |
| Netplay | Low | High | 3 |

---

## Performance Targets

- **Boot time**: < 10 seconds
- **ROM scanning**: 1000 games/second
- **UI response**: < 100ms
- **Game launch**: < 3 seconds
- **Memory footprint**: < 200MB (system only)
- **Battery efficiency**: > 5 hours (typical use)
