# RexOS Roadmap

This document outlines the features and improvements planned for RexOS to differentiate it from existing solutions like ArkOS, muOS, and KNULLI.

## Vision

RexOS aims to be the fastest, most reliable, and most feature-rich operating system for retro gaming handhelds. Built with Rust for performance and safety, RexOS addresses the pain points of existing shell-based solutions while introducing modern gaming features.

## Competitive Analysis

### Current Landscape

| OS | Strengths | Weaknesses |
|----|-----------|------------|
| **ArkOS** | Mature, stable, wide device support | Shell-based (slow), remote services reset on reboot, dual SD issues |
| **muOS** | Fast, good PortMaster support | Limited device support (H700 only), WIP |
| **KNULLI** | EmulationStation UI, Batocera-based | EXT4 filesystem (Windows-unfriendly), shader bugs |
| **MinUI** | Clean/simple, best sleep/resume | No WiFi, limited emulator support |

### RexOS Advantages

- **Rust-based**: Faster boot, lower memory, safer code
- **Single binary services**: No shell script overhead
- **Modern architecture**: Async services, live config reload, proper IPC

---

## Phase 1: Core Foundation (MVP)

Essential features for a functional, competitive OS.

### 1.1 Fast Boot System
- [ ] Target boot time: <5 seconds (vs ArkOS ~15-20s)
- [ ] Parallel service initialization
- [ ] Lazy loading of non-critical components
- [ ] Splash screen with progress indicator

### 1.2 Session Recall / Quick Resume
- [ ] Auto-save state on sleep/exit
- [ ] Instant resume to last game
- [ ] Per-game save state slots (10 per game)
- [ ] Save state thumbnails with timestamps

### 1.3 Smart Power Management
- [ ] Intelligent power profiles
  - Performance: Max CPU/GPU, full brightness
  - Balanced: Dynamic scaling based on emulator load
  - Power Saver: Aggressive throttling, dimmed display
  - Auto: ML-based learning from usage patterns
- [ ] Per-emulator power hints
- [ ] Battery time remaining estimates
- [ ] Low battery warnings with auto-save

### 1.4 Reliable Storage Management
- [ ] Graceful single/dual SD card detection
- [ ] Automatic fallback if secondary SD missing
- [ ] exFAT support for Windows compatibility
- [ ] Filesystem integrity checks on boot
- [ ] TRIM support for SD card longevity

### 1.5 Input System
- [ ] Hot-pluggable controller support
- [ ] Per-game button remapping
- [ ] Combo hotkeys (e.g., Select+Start for menu)
- [ ] Analog stick calibration tool
- [ ] Rumble/haptic feedback support

---

## UI & Frontend Architecture

The current launcher (`rexos-launcher`) is a basic TUI using ratatui. This section outlines the missing UI components and the path to a polished user experience.

### Current State Analysis

**What exists:**
- Basic TUI with ratatui/crossterm
- System list and game list views
- Simple game info panel
- Placeholder settings screen
- Favorites toggle

**What's missing:**

#### Launcher UI (Primary Interface)

| Feature | Priority | Description |
|---------|----------|-------------|
| **Box art display** | High | Show game covers/screenshots in list |
| **Metadata scraping** | High | Auto-download artwork from ScreenScraper/IGDB |
| **Video previews** | Medium | Play video snaps on hover (like ES) |
| **Search/filter** | High | Quick search across all games |
| **Sort options** | High | By name, date, playtime, rating |
| **Grid view** | Medium | Alternative to list view |
| **Animations** | Low | Smooth transitions between views |
| **Sound effects** | Low | Navigation sounds, launch jingles |

#### Theme System

- [ ] **Theme engine** - JSON/TOML-based theme definitions
- [ ] **Color schemes** - Light/dark/custom palettes
- [ ] **Font support** - Custom fonts for different languages
- [ ] **Layout templates** - Different UI arrangements
- [ ] **Theme marketplace** - Download community themes
- [ ] **Per-system themes** - Different look for each console
- [ ] **Animated backgrounds** - Video/shader backgrounds

#### Quick Menu (In-Game Overlay)

Accessible via hotkey (e.g., Select+Start) while in-game:

```
┌─────────────────────────────────────┐
│         RexOS Quick Menu            │
├─────────────────────────────────────┤
│  ► Resume Game                      │
│    Save State                       │
│    Load State                       │
│    ───────────────────              │
│    Screenshot                       │
│    ───────────────────              │
│    Brightness    [████████░░] 80%   │
│    Volume        [██████░░░░] 60%   │
│    ───────────────────              │
│    Shader        [CRT-Royale    ▼]  │
│    Aspect Ratio  [4:3           ▼]  │
│    ───────────────────              │
│    RetroArch Menu                   │
│    Exit Game                        │
└─────────────────────────────────────┘
```

Features:
- [ ] Brightness/volume sliders
- [ ] Save/load state with thumbnails
- [ ] Quick screenshot
- [ ] Shader preset switcher
- [ ] Aspect ratio toggle
- [ ] FPS counter toggle
- [ ] Exit to launcher

#### Settings UI

Complete settings interface (currently placeholder):

```
Settings Categories:
├── Display
│   ├── Brightness
│   ├── Color temperature
│   ├── Screen rotation
│   └── Overscan adjustment
├── Audio
│   ├── Volume
│   ├── Output device
│   └── Audio latency
├── Controls
│   ├── Button mapping
│   ├── Analog deadzone
│   ├── Hotkey configuration
│   └── Rumble intensity
├── Network
│   ├── WiFi setup
│   ├── Bluetooth devices
│   ├── Remote services
│   └── Cloud sync
├── Storage
│   ├── SD card info
│   ├── Scraper settings
│   └── Backup/restore
├── Power
│   ├── Power profile
│   ├── Auto-sleep timer
│   ├── Battery stats
│   └── LED control
├── System
│   ├── Language
│   ├── Timezone
│   ├── Updates
│   └── About
└── Advanced
    ├── RetroArch settings
    ├── Debug logs
    └── Developer options
```

#### Notifications & Toasts

- [ ] Achievement unlocked popups
- [ ] Low battery warnings
- [ ] Download progress
- [ ] Controller connected/disconnected
- [ ] Save sync status
- [ ] Error messages with actions

#### Onboarding / First Run

- [ ] Welcome wizard
- [ ] WiFi setup prompt
- [ ] SD card format options
- [ ] ROM folder structure explanation
- [ ] Control scheme tutorial
- [ ] Theme selection

#### Accessibility

- [ ] High contrast mode
- [ ] Large text option
- [ ] Color blind modes
- [ ] Screen reader support
- [ ] Remappable navigation

### Emulator UI Integration

#### RetroArch Customization

- [ ] **Custom RetroArch config** - Pre-configured for RexOS
- [ ] **Unified menu theme** - Match launcher aesthetics
- [ ] **Hotkey consistency** - Same shortcuts in all emulators
- [ ] **Auto core download** - Fetch missing cores automatically
- [ ] **Per-game overrides** - Visual editor for game-specific settings

#### Standalone Emulator UIs

For emulators not using RetroArch (DraStic, PPSSPP, etc.):

- [ ] **Wrapper scripts** - Consistent launch/exit behavior
- [ ] **Config sync** - Unified controller mapping
- [ ] **Save location redirect** - All saves in one place
- [ ] **Exit handling** - Clean return to launcher

### Boot & Loading Screens

#### Boot Animation

- [ ] **Splash screen** - RexOS logo animation
- [ ] **Progress bar** - Show boot stage
- [ ] **Device info** - Model, firmware version
- [ ] **Custom boot videos** - User-replaceable

#### Loading States

- [ ] **Game loading** - System-themed loading screen
- [ ] **Scraping progress** - Per-game progress
- [ ] **Update download** - Progress with ETA
- [ ] **System scan** - ROM detection progress

### UI Technology Stack

**Current:**
- `ratatui` + `crossterm` for TUI

**Recommended evolution:**

| Layer | Technology | Purpose |
|-------|------------|---------|
| **GPU UI** | `slint` or `iced` | Hardware-accelerated UI for better visuals |
| **Framebuffer** | `fbdev` / `drm` | Direct rendering for low-level control |
| **Overlay** | Custom OpenGL layer | In-game quick menu |
| **Web UI** | `axum` + HTMX/React | Remote management |

### UI Implementation Phases

**Phase 1 - Functional TUI:**
- [ ] Complete settings screens
- [ ] Search and filter
- [ ] Metadata display
- [ ] Notifications

**Phase 2 - Enhanced TUI:**
- [ ] Box art (sixel/kitty graphics protocol)
- [ ] Color themes
- [ ] Animations
- [ ] Sound effects

**Phase 3 - GPU Frontend (Optional):**
- [ ] Migrate to `slint` or `iced` for GPU rendering
- [ ] Full EmulationStation-style interface
- [ ] Video previews
- [ ] Shader backgrounds

---

## Phase 2: User Experience Polish

Features that make daily use delightful.

### 2.1 Cloud Save Sync
- [ ] Dropbox integration
- [ ] Google Drive integration
- [ ] WebDAV support (self-hosted)
- [ ] Conflict resolution UI
- [ ] Sync on WiFi only option
- [ ] Per-game sync enable/disable

### 2.2 Game Time Tracking
- [ ] Per-game playtime statistics
- [ ] Daily/weekly/monthly play summaries
- [ ] Visual graphs and charts
- [ ] "Year in Review" feature
- [ ] Export statistics to JSON/CSV

### 2.3 Achievement System
- [ ] RetroAchievements integration
- [ ] On-screen achievement notifications
- [ ] Achievement progress tracking
- [ ] Hardcore mode support
- [ ] Offline achievement queue

### 2.4 Web File Manager
- [ ] Modern responsive UI (better than FileBrowser)
- [ ] Drag-and-drop ROM uploads
- [ ] ROM scraping from web interface
- [ ] Save file management
- [ ] Screenshot gallery
- [ ] System configuration editor

### 2.5 Smart Collections
- [ ] Recently Played (last 20 games)
- [ ] Most Played (by time)
- [ ] Favorites with sorting
- [ ] By Genre (auto-detected from metadata)
- [ ] By Year
- [ ] Custom user collections
- [ ] Random game picker

### 2.6 Screenshot & Recording
- [ ] Hotkey for instant screenshots
- [ ] Screenshot gallery with viewer
- [ ] Video recording (last 30 seconds buffer)
- [ ] Share to cloud storage
- [ ] Automatic screenshot on achievement

---

## Phase 3: Advanced Features

Differentiating features that set RexOS apart.

### 3.1 Network Features
- [ ] Persistent remote services (user choice, not reset on reboot)
- [ ] Auto-enable services on trusted networks
- [ ] Zero-config device discovery (mDNS)
- [ ] SFTP server with proper permissions
- [ ] SMB/Samba shares
- [ ] SSH with key authentication

### 3.2 Bluetooth Audio
- [ ] Low-latency audio mode (aptX LL)
- [ ] Auto-reconnect to paired devices
- [ ] Audio device switcher in quick menu
- [ ] Volume sync with BT headphones

### 3.3 Multi-User Profiles
- [ ] Multiple user profiles
- [ ] Separate save files per user
- [ ] Profile-specific settings
- [ ] Parental controls per profile
- [ ] Time limits and play schedules
- [ ] Content filtering by rating

### 3.4 Migration Wizard
- [ ] One-click import from ArkOS
- [ ] One-click import from muOS
- [ ] One-click import from KNULLI
- [ ] Preserve saves, configs, and scrapes
- [ ] Rollback option if migration fails

### 3.5 Plugin System
- [ ] Lua scripting for extensions
- [ ] WASM plugin support
- [ ] Plugin marketplace/repository
- [ ] Hot-reload plugins without restart
- [ ] Sandboxed execution for security

### 3.6 Game Recommendations
- [ ] "If you liked X, try Y" suggestions
- [ ] Based on play history and ratings
- [ ] Discover hidden gems in library
- [ ] Community-powered recommendations

---

## Phase 4: Platform Ecosystem

Long-term vision for RexOS as a platform.

### 4.1 Remote Play
- [ ] Stream games to phone/tablet
- [ ] Stream to PC via web browser
- [ ] Moonlight protocol compatibility
- [ ] Touch controls overlay for mobile

### 4.2 P2P Multiplayer
- [ ] WiFi Direct game linking
- [ ] Netplay lobby system
- [ ] Friend list and invites
- [ ] Voice chat support

### 4.3 REST API
- [ ] Full system control via API
- [ ] Launch games remotely
- [ ] Query library and stats
- [ ] Webhook notifications
- [ ] Integration with Home Assistant

### 4.4 OTA Updates
- [ ] A/B partition scheme for safe updates
- [ ] Automatic rollback on boot failure
- [ ] Delta updates (download only changes)
- [ ] Update channels (stable/beta/nightly)
- [ ] Update scheduling (night-time only)

---

## Technical Architecture

### Core Services

```
┌─────────────────────────────────────────────────────────┐
│                    RexOS Architecture                    │
├─────────────────────────────────────────────────────────┤
│  UI Layer                                                │
│  ├── EmulationStation (default)                         │
│  ├── SimpleMenu (minimal mode)                          │
│  └── Web Interface                                       │
├─────────────────────────────────────────────────────────┤
│  System Services (Rust)                                  │
│  ├── rexos-launcher    - Game launching & session mgmt  │
│  ├── rexos-library     - ROM database & scraping        │
│  ├── rexos-emulator    - RetroArch/standalone control   │
│  ├── rexos-network     - WiFi, BT, remote services      │
│  ├── rexos-update      - OTA updates & verification     │
│  └── rexos-config      - Settings & profiles            │
├─────────────────────────────────────────────────────────┤
│  Core System (Rust)                                      │
│  ├── rexos-hal         - Hardware abstraction           │
│  ├── rexos-storage     - SD cards, mounting, watching   │
│  └── rexos-init        - PID 1, service supervision     │
├─────────────────────────────────────────────────────────┤
│  Linux Kernel (device-specific)                          │
├─────────────────────────────────────────────────────────┤
│  Hardware (Anbernic RG353, RG35XX, etc.)                │
└─────────────────────────────────────────────────────────┘
```

### Performance Targets

| Metric | ArkOS | RexOS Target |
|--------|-------|--------------|
| Boot time | ~15-20s | <5s |
| Memory at idle | ~150MB | <80MB |
| Game launch time | ~3-5s | <2s |
| Sleep/resume | ~2s | <0.5s |
| Config reload | Restart required | Live reload |

### Reliability Goals

- **99.9% boot success rate** - Comprehensive error handling
- **Zero data loss** - Atomic writes, journaled saves
- **Graceful degradation** - Boot to safe mode on errors
- **Automatic recovery** - Self-healing filesystem checks

---

## Device Support Roadmap

### Tier 1 (Primary Support)
- Anbernic RG353M / RG353V / RG353VS (RK3566)
- Anbernic RG353P / RG353PS (RK3566)

### Tier 2 (Full Support)
- Anbernic RG503 (RK3566)
- Powkiddy RGB30 (RK3566)
- Anbernic RG351V / RG351P / RG351MP (RK3326)

### Tier 3 (Community Support)
- Anbernic RG35XX / RG35XX Plus / RG35XX H (H700)
- Powkiddy RGB10 / RGB20S
- Other RK3566/RK3326 devices

---

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for how to contribute to RexOS development.

### Priority Areas for Contributors

1. **Emulator testing** - Test RetroArch cores on target devices
2. **Device profiles** - Add support for new devices
3. **Translations** - Localize the UI
4. **Themes** - Create beautiful themes
5. **Documentation** - Improve user guides

---

## References

- [ArkOS Wiki](https://github.com/christianhaitian/arkos/wiki)
- [KNULLI Documentation](https://knulli.org/)
- [muOS GitHub](https://github.com/MustardOS/muos)
- [Retro Game Corps Guides](https://retrogamecorps.com/)
- [SBC Gaming Reddit](https://reddit.com/r/SBCGaming)
