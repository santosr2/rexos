//! RexOS Launcher
//!
//! A TUI-based game launcher for RexOS, optimized for handheld devices.
//! Provides game browsing, launching, and settings management.
//!
//! # Input Support
//!
//! The launcher supports both keyboard and gamepad input:
//! - Keyboard: For development and SSH access
//! - Gamepad: Via HAL InputManager for actual device usage

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

use rexos_config::RexOSConfig;
use rexos_emulator::{EmulatorLauncher, LaunchConfig};
use rexos_hal::input::{Button, InputManager};
use rexos_library::{Game, GameDatabase, RomScanner};
use rexos_network::{NetworkConfig, NetworkManager};

/// Application state
struct App {
    /// Game database
    db: GameDatabase,

    /// Emulator launcher
    launcher: EmulatorLauncher,

    /// Configuration
    config: RexOSConfig,

    /// Gamepad input manager (optional - may not be available on dev machines)
    input: Option<InputManager>,

    /// Network manager (optional - may not be available)
    network: Option<NetworkManager>,

    /// Current view
    view: View,

    /// Systems list state
    systems_state: ListState,

    /// Games list state
    games_state: ListState,

    /// Settings list state (for navigating settings)
    settings_state: ListState,

    /// Available systems
    systems: Vec<(String, i64)>,

    /// Current games list
    games: Vec<Game>,

    /// Selected system
    selected_system: Option<String>,

    /// Status message
    status: String,

    /// Should quit
    should_quit: bool,

    /// Settings items for editing
    settings_items: Vec<SettingItem>,

    /// Whether we're currently editing a setting
    editing_setting: bool,
}

/// A setting that can be edited
#[derive(Debug, Clone)]
struct SettingItem {
    name: &'static str,
    kind: SettingKind,
}

/// Type of setting value
#[derive(Debug, Clone)]
enum SettingKind {
    /// Percentage value (0-100)
    Percentage { value: u8, step: u8 },
    /// Boolean toggle
    Toggle { value: bool },
    /// Selection from options
    Select { options: Vec<&'static str>, current: usize },
}

/// Current view/screen
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum View {
    Systems,
    Games,
    GameInfo,
    Settings,
}

impl App {
    /// Get ROM directory from environment or default
    fn get_roms_dir() -> PathBuf {
        std::env::var("REXOS_ROMS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/roms"))
    }

    /// Create new application
    fn new() -> Result<Self> {
        let roms_dir = Self::get_roms_dir();

        // Open game database
        let db_path = roms_dir.join(".rexos/games.db");

        // Create directory if needed
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let db = GameDatabase::open(&db_path)?;

        // Load configuration
        let config = RexOSConfig::load_default()?;

        // Create launcher
        let launcher = EmulatorLauncher::new();

        // Initialize gamepad input (optional - may fail on dev machines)
        let input = match InputManager::new() {
            Ok(mgr) => {
                info!("Gamepad input initialized with {} devices", mgr.devices().len());
                Some(mgr)
            }
            Err(e) => {
                warn!("Gamepad input not available: {} (keyboard-only mode)", e);
                None
            }
        };

        // Initialize network manager (optional)
        let network = match NetworkManager::new(NetworkConfig::default()) {
            Ok(mgr) => {
                info!("Network manager initialized");
                Some(mgr)
            }
            Err(e) => {
                warn!("Network manager not available: {}", e);
                None
            }
        };

        // Get systems
        let systems = db.get_systems()?;

        // Build settings items from current config
        let settings_items = Self::build_settings_items(&config);

        let mut app = Self {
            db,
            launcher,
            config,
            input,
            network,
            view: View::Systems,
            systems_state: ListState::default(),
            games_state: ListState::default(),
            settings_state: ListState::default(),
            systems,
            games: Vec::new(),
            selected_system: None,
            status: "Ready".to_string(),
            should_quit: false,
            settings_items,
            editing_setting: false,
        };

        // Select first system if available
        if !app.systems.is_empty() {
            app.systems_state.select(Some(0));
        }

        Ok(app)
    }

    /// Build settings items from configuration
    fn build_settings_items(config: &RexOSConfig) -> Vec<SettingItem> {
        vec![
            SettingItem {
                name: "Brightness",
                kind: SettingKind::Percentage {
                    value: (config.system.brightness as f32 / 255.0 * 100.0) as u8,
                    step: 10,
                },
            },
            SettingItem {
                name: "Volume",
                kind: SettingKind::Percentage {
                    value: config.system.volume,
                    step: 10,
                },
            },
            SettingItem {
                name: "Performance Mode",
                kind: SettingKind::Select {
                    options: vec!["powersave", "balanced", "performance"],
                    current: match config.system.performance {
                        rexos_config::PerformanceProfile::Powersave => 0,
                        rexos_config::PerformanceProfile::Balanced => 1,
                        rexos_config::PerformanceProfile::Performance => 2,
                    },
                },
            },
            SettingItem {
                name: "WiFi",
                kind: SettingKind::Toggle {
                    value: config.system.network.wifi_enabled,
                },
            },
            SettingItem {
                name: "SSH",
                kind: SettingKind::Toggle {
                    value: config.system.network.ssh_enabled,
                },
            },
            SettingItem {
                name: "Auto-suspend",
                kind: SettingKind::Select {
                    options: vec!["Disabled", "5 min", "10 min", "15 min", "30 min"],
                    current: match config.system.suspend_timeout {
                        0 => 0,
                        5 => 1,
                        10 => 2,
                        15 => 3,
                        _ => 4,
                    },
                },
            },
        ]
    }

    /// Apply setting change and update config
    fn apply_setting(&mut self, index: usize) -> Result<()> {
        if index >= self.settings_items.len() {
            return Ok(());
        }

        let item = &self.settings_items[index];
        match (&item.kind, item.name) {
            (SettingKind::Percentage { value, .. }, "Brightness") => {
                self.config.system.brightness = (*value as f32 / 100.0 * 255.0) as u8;
                // Apply immediately via HAL if available
                debug!("Setting brightness to {}", self.config.system.brightness);
            }
            (SettingKind::Percentage { value, .. }, "Volume") => {
                self.config.system.volume = *value;
                // Apply via amixer
                let _ = std::process::Command::new("amixer")
                    .args(["sset", "Master", &format!("{}%", value)])
                    .output();
                debug!("Setting volume to {}%", value);
            }
            (SettingKind::Select { current, .. }, "Performance Mode") => {
                self.config.system.performance = match current {
                    0 => rexos_config::PerformanceProfile::Powersave,
                    1 => rexos_config::PerformanceProfile::Balanced,
                    _ => rexos_config::PerformanceProfile::Performance,
                };
            }
            (SettingKind::Toggle { value }, "WiFi") => {
                self.config.system.network.wifi_enabled = *value;
                // Toggle WiFi via network manager
                if let Some(ref mut net) = self.network {
                    if *value {
                        let _ = net.wifi().enable();
                    } else {
                        let _ = net.wifi().disable();
                    }
                }
            }
            (SettingKind::Toggle { value }, "SSH") => {
                self.config.system.network.ssh_enabled = *value;
                // Toggle SSH service
                let cmd = if *value { "start" } else { "stop" };
                let _ = std::process::Command::new("systemctl")
                    .args([cmd, "sshd"])
                    .output();
            }
            (SettingKind::Select { options, current }, "Auto-suspend") => {
                self.config.system.suspend_timeout = match current {
                    0 => 0,
                    1 => 5,
                    2 => 10,
                    3 => 15,
                    _ => 30,
                };
                let _ = options; // silence unused warning
            }
            _ => {}
        }

        // Save config to file
        self.config.save_default()?;
        self.status = format!("{} updated", item.name);
        Ok(())
    }

    /// Poll gamepad input and convert to key codes
    fn poll_gamepad(&mut self) -> Option<KeyCode> {
        let input = self.input.as_mut()?;

        // Poll for new events
        if input.poll().is_err() {
            return None;
        }

        // Map gamepad buttons to key codes
        if input.is_pressed(Button::Up) {
            return Some(KeyCode::Up);
        }
        if input.is_pressed(Button::Down) {
            return Some(KeyCode::Down);
        }
        if input.is_pressed(Button::Left) {
            return Some(KeyCode::Left);
        }
        if input.is_pressed(Button::Right) {
            return Some(KeyCode::Right);
        }
        if input.is_pressed(Button::A) {
            return Some(KeyCode::Enter);
        }
        if input.is_pressed(Button::B) {
            return Some(KeyCode::Esc);
        }
        if input.is_pressed(Button::X) {
            return Some(KeyCode::Char('x'));
        }
        if input.is_pressed(Button::Y) {
            return Some(KeyCode::Char('f')); // Favorite
        }
        if input.is_pressed(Button::Start) {
            return Some(KeyCode::Tab);
        }
        if input.is_pressed(Button::Select) {
            return Some(KeyCode::Char('r')); // Rescan
        }
        if input.is_pressed(Button::L1) {
            return Some(KeyCode::PageUp);
        }
        if input.is_pressed(Button::R1) {
            return Some(KeyCode::PageDown);
        }

        None
    }

    /// Handle input
    fn handle_input(&mut self, key: KeyCode) -> Result<()> {
        match self.view {
            View::Systems => self.handle_systems_input(key)?,
            View::Games => self.handle_games_input(key)?,
            View::GameInfo => self.handle_game_info_input(key)?,
            View::Settings => self.handle_settings_input(key)?,
        }
        Ok(())
    }

    /// Handle systems view input
    fn handle_systems_input(&mut self, key: KeyCode) -> Result<()> {
        if input::is_nav_up(key) {
            self.select_prev_system();
        } else if input::is_nav_down(key) {
            self.select_next_system();
        } else if input::is_select(key) {
            self.enter_system()?;
        } else if input::is_rescan(key) {
            self.rescan_roms()?;
        } else if input::is_tab(key) {
            self.view = View::Settings;
            // Select first setting if none selected
            if self.settings_state.selected().is_none() && !self.settings_items.is_empty() {
                self.settings_state.select(Some(0));
            }
        } else if input::is_quit(key) {
            self.should_quit = true;
        }
        Ok(())
    }

    /// Handle games view input
    fn handle_games_input(&mut self, key: KeyCode) -> Result<()> {
        if input::is_nav_up(key) {
            self.select_prev_game();
        } else if input::is_nav_down(key) {
            self.select_next_game();
        } else if input::is_select(key) {
            self.launch_selected_game()?;
        } else if input::is_info(key) {
            self.show_game_info();
        } else if input::is_favorite(key) {
            self.toggle_favorite()?;
        } else if input::is_back(key) {
            self.view = View::Systems;
            self.games.clear();
            self.games_state.select(None);
        }
        Ok(())
    }

    /// Handle game info view input
    fn handle_game_info_input(&mut self, key: KeyCode) -> Result<()> {
        if input::is_select(key) {
            self.launch_selected_game()?;
        } else if input::is_back(key) {
            self.view = View::Games;
        }
        Ok(())
    }

    /// Handle settings view input
    fn handle_settings_input(&mut self, key: KeyCode) -> Result<()> {
        if self.editing_setting {
            // Handle editing mode
            if let Some(i) = self.settings_state.selected()
                && i < self.settings_items.len()
            {
                match key {
                    KeyCode::Left => {
                        self.adjust_setting(i, false);
                        self.apply_setting(i)?;
                    }
                    KeyCode::Right => {
                        self.adjust_setting(i, true);
                        self.apply_setting(i)?;
                    }
                    KeyCode::Enter | KeyCode::Esc | KeyCode::Char('b') => {
                        self.editing_setting = false;
                        self.status = "Settings saved".to_string();
                    }
                    _ => {}
                }
            }
        } else {
            // Handle navigation mode
            match key {
                KeyCode::Up | KeyCode::Char('w') => {
                    self.select_prev_setting();
                }
                KeyCode::Down | KeyCode::Char('s') => {
                    self.select_next_setting();
                }
                KeyCode::Enter | KeyCode::Char('a') | KeyCode::Left | KeyCode::Right => {
                    // Enter editing mode for the selected setting
                    if self.settings_state.selected().is_some() {
                        self.editing_setting = true;
                        self.status = "[←→] Adjust  [Enter] Confirm".to_string();

                        // For toggles, immediately toggle on Enter
                        if let Some(i) = self.settings_state.selected()
                            && i < self.settings_items.len()
                        {
                            if let SettingKind::Toggle { .. } = self.settings_items[i].kind {
                                self.adjust_setting(i, true); // Toggle
                                self.apply_setting(i)?;
                                self.editing_setting = false;
                            } else if key == KeyCode::Left || key == KeyCode::Right {
                                self.adjust_setting(i, key == KeyCode::Right);
                                self.apply_setting(i)?;
                            }
                        }
                    }
                }
                KeyCode::Esc | KeyCode::Tab | KeyCode::Char('b') => {
                    self.view = View::Systems;
                    self.settings_state.select(None);
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Select previous setting
    fn select_prev_setting(&mut self) {
        if self.settings_items.is_empty() {
            return;
        }

        let i = match self.settings_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.settings_items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.settings_state.select(Some(i));
    }

    /// Select next setting
    fn select_next_setting(&mut self) {
        if self.settings_items.is_empty() {
            return;
        }

        let i = match self.settings_state.selected() {
            Some(i) => {
                if i >= self.settings_items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.settings_state.select(Some(i));
    }

    /// Adjust a setting value
    fn adjust_setting(&mut self, index: usize, increase: bool) {
        if index >= self.settings_items.len() {
            return;
        }

        match &mut self.settings_items[index].kind {
            SettingKind::Percentage { value, step } => {
                if increase {
                    *value = (*value).saturating_add(*step).min(100);
                } else {
                    *value = (*value).saturating_sub(*step);
                }
            }
            SettingKind::Toggle { value } => {
                *value = !*value;
            }
            SettingKind::Select { options, current } => {
                if increase {
                    *current = (*current + 1) % options.len();
                } else {
                    *current = if *current == 0 {
                        options.len() - 1
                    } else {
                        *current - 1
                    };
                }
            }
        }
    }

    /// Select previous system
    fn select_prev_system(&mut self) {
        if self.systems.is_empty() {
            return;
        }

        let i = match self.systems_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.systems.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.systems_state.select(Some(i));
    }

    /// Select next system
    fn select_next_system(&mut self) {
        if self.systems.is_empty() {
            return;
        }

        let i = match self.systems_state.selected() {
            Some(i) => {
                if i >= self.systems.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.systems_state.select(Some(i));
    }

    /// Enter selected system
    fn enter_system(&mut self) -> Result<()> {
        if let Some(i) = self.systems_state.selected()
            && i < self.systems.len()
        {
            let system = &self.systems[i].0;
            self.selected_system = Some(system.clone());
            self.games = self.db.get_games_by_system(system)?;
            self.view = View::Games;

            if !self.games.is_empty() {
                self.games_state.select(Some(0));
            }

            self.status = format!("{} games", self.games.len());
        }
        Ok(())
    }

    /// Select previous game
    fn select_prev_game(&mut self) {
        if self.games.is_empty() {
            return;
        }

        let i = match self.games_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.games.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.games_state.select(Some(i));
    }

    /// Select next game
    fn select_next_game(&mut self) {
        if self.games.is_empty() {
            return;
        }

        let i = match self.games_state.selected() {
            Some(i) => {
                if i >= self.games.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.games_state.select(Some(i));
    }

    /// Show game info
    fn show_game_info(&mut self) {
        if self.games_state.selected().is_some() {
            self.view = View::GameInfo;
        }
    }

    /// Toggle favorite for selected game
    fn toggle_favorite(&mut self) -> Result<()> {
        if let Some(i) = self.games_state.selected()
            && i < self.games.len()
        {
            let game = &mut self.games[i];
            game.favorite = !game.favorite;
            self.db.set_favorite(game.id, game.favorite)?;

            self.status = if game.favorite {
                "Added to favorites".to_string()
            } else {
                "Removed from favorites".to_string()
            };
        }
        Ok(())
    }

    /// Launch selected game
    fn launch_selected_game(&mut self) -> Result<()> {
        if let Some(i) = self.games_state.selected()
            && i < self.games.len()
        {
            let game = &self.games[i];
            self.status = format!("Launching {}...", game.name);

            // Build launch config
            let config = LaunchConfig::for_rom(&game.path);

            // Launch game
            match self.launcher.launch(config) {
                Ok(result) => {
                    info!("Launched game with PID {}", result.pid);

                    // Wait for emulator to exit
                    let mut child = result.child;
                    let _ = child.wait();

                    // Update play stats
                    self.db.update_play_stats(game.id, 0)?;

                    self.status = "Ready".to_string();
                }
                Err(e) => {
                    error!("Failed to launch game: {}", e);
                    self.status = format!("Error: {}", e);
                }
            }
        }
        Ok(())
    }

    /// Rescan ROMs
    fn rescan_roms(&mut self) -> Result<()> {
        self.status = "Scanning ROMs...".to_string();

        let scanner = RomScanner::new();
        let roms_dir = Self::get_roms_dir();

        if let Ok(results) = scanner.scan_all(&roms_dir) {
            let mut total_games = 0;

            for (_system, games) in results {
                for game in games {
                    self.db.add_game(&game)?;
                    total_games += 1;
                }
            }

            self.status = format!("Found {} games", total_games);

            // Refresh systems list
            self.systems = self.db.get_systems()?;
        }

        Ok(())
    }

    /// Get selected game
    fn selected_game(&self) -> Option<&Game> {
        self.games_state.selected().and_then(|i| self.games.get(i))
    }
}

/// Draw the UI
fn draw_ui(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Footer
        ])
        .split(frame.size());

    // Draw header
    draw_header(frame, chunks[0], app);

    // Draw main content based on view
    match app.view {
        View::Systems => draw_systems_view(frame, chunks[1], app),
        View::Games => draw_games_view(frame, chunks[1], app),
        View::GameInfo => draw_game_info_view(frame, chunks[1], app),
        View::Settings => draw_settings_view(frame, chunks[1], app),
    }

    // Draw footer
    draw_footer(frame, chunks[2], app);
}

/// Draw header
fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let title = match app.view {
        View::Systems => "RexOS - Select System",
        View::Games => &format!(
            "RexOS - {}",
            app.selected_system.as_deref().unwrap_or("Games")
        ),
        View::GameInfo => "RexOS - Game Info",
        View::Settings => "RexOS - Settings",
    };

    let header = Paragraph::new(title)
        .style(ui::header_style())
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(header, area);
}

/// Draw systems view
fn draw_systems_view(frame: &mut Frame, area: Rect, app: &mut App) {
    let items: Vec<ListItem> = app
        .systems
        .iter()
        .map(|(name, count)| {
            let display = format!("{:<20} ({} games)", name, count);
            ListItem::new(display)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Systems"))
        .highlight_style(ui::highlight_style())
        .highlight_symbol(ui::SELECTION_SYMBOL);

    frame.render_stateful_widget(list, area, &mut app.systems_state);
}

/// Draw games view
fn draw_games_view(frame: &mut Frame, area: Rect, app: &mut App) {
    let items: Vec<ListItem> = app
        .games
        .iter()
        .map(|game| {
            let prefix = if game.favorite {
                ui::FAVORITE_PREFIX
            } else {
                ui::NORMAL_PREFIX
            };
            let display = format!("{}{}", prefix, game.name);
            ListItem::new(display)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Games"))
        .highlight_style(ui::highlight_style())
        .highlight_symbol(ui::SELECTION_SYMBOL);

    frame.render_stateful_widget(list, area, &mut app.games_state);
}

/// Draw game info view
fn draw_game_info_view(frame: &mut Frame, area: Rect, app: &App) {
    let content = if let Some(game) = app.selected_game() {
        let mut lines = vec![
            Line::from(vec![
                Span::styled("Name: ", ui::label_style()),
                Span::raw(&game.name),
            ]),
            Line::from(vec![
                Span::styled("System: ", ui::label_style()),
                Span::raw(&game.system),
            ]),
            Line::from(vec![
                Span::styled("Path: ", ui::label_style()),
                Span::raw(&game.path),
            ]),
        ];

        if let Some(ref desc) = game.description {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled("Description: ", ui::label_style())]));
            lines.push(Line::from(desc.as_str()));
        }

        if let Some(ref dev) = game.developer {
            lines.push(Line::from(vec![
                Span::styled("Developer: ", ui::label_style()),
                Span::raw(dev),
            ]));
        }

        if let Some(rating) = game.rating {
            lines.push(Line::from(vec![
                Span::styled("Rating: ", ui::label_style()),
                Span::raw(format!("{:.1}/5", rating)),
            ]));
        }

        Text::from(lines)
    } else {
        Text::raw("No game selected")
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("Game Info"))
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

/// Draw settings view - interactive settings list
fn draw_settings_view(frame: &mut Frame, area: Rect, app: &mut App) {
    let selected_idx = app.settings_state.selected();

    let items: Vec<ListItem> = app
        .settings_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let value_str = match &item.kind {
                SettingKind::Percentage { value, .. } => format!("{}%", value),
                SettingKind::Toggle { value } => {
                    if *value {
                        "Enabled".to_string()
                    } else {
                        "Disabled".to_string()
                    }
                }
                SettingKind::Select { options, current } => options[*current].to_string(),
            };

            // Show adjustment indicators for selected item
            let display = if Some(i) == selected_idx && app.editing_setting {
                format!("{:<16} < {} >", item.name, value_str)
            } else {
                format!("{:<16} {}", item.name, value_str)
            };

            ListItem::new(display)
        })
        .collect();

    let title = if app.editing_setting {
        "Settings (Editing - ←→ to adjust)"
    } else {
        "Settings (Enter to edit)"
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(ui::highlight_style())
        .highlight_symbol(ui::SELECTION_SYMBOL);

    frame.render_stateful_widget(list, area, &mut app.settings_state);
}

/// Draw footer
fn draw_footer(frame: &mut Frame, area: Rect, app: &App) {
    let help_text = match app.view {
        View::Systems => "[↑↓] Navigate  [Enter] Select  [R] Rescan  [Tab] Settings  [Q] Quit",
        View::Games => "[↑↓] Navigate  [Enter] Launch  [F] Favorite  [X] Info  [B] Back",
        View::GameInfo => "[Enter] Launch  [B] Back",
        View::Settings => {
            if app.editing_setting {
                "[←→] Adjust  [Enter] Confirm  [B] Cancel"
            } else {
                "[↑↓] Navigate  [Enter/←→] Edit  [Tab/B] Back"
            }
        }
    };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

    let help = Paragraph::new(help_text)
        .style(ui::help_style())
        .block(Block::default().borders(Borders::ALL));

    let status = Paragraph::new(app.status.as_str())
        .style(ui::status_style())
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(help, chunks[0]);
    frame.render_widget(status, chunks[1]);
}

fn main() -> Result<()> {
    // Setup logging
    tracing_subscriber::fmt().with_env_filter("info").init();

    info!("RexOS Launcher starting...");

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new()?;

    // Main loop
    let tick_rate = Duration::from_millis(50); // Faster for responsive gamepad input
    let mut last_tick = Instant::now();
    let mut last_gamepad_input = Instant::now();
    let gamepad_repeat_delay = Duration::from_millis(150); // Debounce gamepad

    loop {
        terminal.draw(|f| draw_ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        // Check keyboard input first
        if event::poll(timeout)?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            app.handle_input(key.code)?;
        }

        // Also check gamepad input (with debounce)
        if last_gamepad_input.elapsed() >= gamepad_repeat_delay
            && let Some(key) = app.poll_gamepad()
        {
            app.handle_input(key)?;
            last_gamepad_input = Instant::now();
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    info!("RexOS Launcher exiting");
    Ok(())
}

mod ui {
    //! UI components and rendering utilities
    //!
    //! This module contains reusable UI components for the TUI launcher.

    use ratatui::style::{Color, Modifier, Style};

    /// Default highlight style for selected items
    pub fn highlight_style() -> Style {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    }

    /// Style for bold labels
    pub fn label_style() -> Style {
        Style::default().add_modifier(Modifier::BOLD)
    }

    /// Style for help text in footer
    pub fn help_style() -> Style {
        Style::default().fg(Color::DarkGray)
    }

    /// Style for status messages
    pub fn status_style() -> Style {
        Style::default().fg(Color::Yellow)
    }

    /// Style for header/title
    pub fn header_style() -> Style {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    }

    /// Prefix for favorite games
    pub const FAVORITE_PREFIX: &str = "★ ";

    /// Prefix for non-favorite games
    pub const NORMAL_PREFIX: &str = "  ";

    /// Selection indicator
    pub const SELECTION_SYMBOL: &str = "> ";
}

mod input {
    //! Input handling and key mapping
    //!
    //! Defines key bindings and input handling logic for the launcher.

    use crossterm::event::KeyCode;

    /// Navigation direction (for future analog stick support)
    #[allow(dead_code)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum NavDirection {
        Up,
        Down,
    }

    /// Check if a key is a navigation key (up)
    pub fn is_nav_up(key: KeyCode) -> bool {
        matches!(key, KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('k'))
    }

    /// Check if a key is a navigation key (down)
    pub fn is_nav_down(key: KeyCode) -> bool {
        matches!(key, KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('j'))
    }

    /// Check if a key is a select/confirm key
    pub fn is_select(key: KeyCode) -> bool {
        matches!(key, KeyCode::Enter | KeyCode::Char('a'))
    }

    /// Check if a key is a back/cancel key
    pub fn is_back(key: KeyCode) -> bool {
        matches!(key, KeyCode::Esc | KeyCode::Char('b') | KeyCode::Backspace)
    }

    /// Check if a key is the quit key
    pub fn is_quit(key: KeyCode) -> bool {
        matches!(key, KeyCode::Char('q'))
    }

    /// Check if a key triggers rescan
    pub fn is_rescan(key: KeyCode) -> bool {
        matches!(key, KeyCode::Char('r'))
    }

    /// Check if a key toggles favorite
    pub fn is_favorite(key: KeyCode) -> bool {
        matches!(key, KeyCode::Char('f'))
    }

    /// Check if a key shows info
    pub fn is_info(key: KeyCode) -> bool {
        matches!(key, KeyCode::Char('x'))
    }

    /// Check if a key switches view (tab)
    pub fn is_tab(key: KeyCode) -> bool {
        matches!(key, KeyCode::Tab)
    }
}

#[allow(dead_code)] // State utilities module - provides alternative/extended state types
mod state {
    //! Application state management
    //!
    //! Types and utilities for managing launcher state.
    //! This module provides an extended View enum with helper methods.
    //! The main app uses a simpler View enum directly.

    /// Current view/screen of the launcher (extended version with helpers)
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub enum View {
        /// System selection screen
        #[default]
        Systems,
        /// Games list for selected system
        Games,
        /// Detailed info for selected game
        GameInfo,
        /// Settings menu
        Settings,
    }

    impl View {
        /// Get the title for this view
        pub fn title(&self) -> &'static str {
            match self {
                View::Systems => "Select System",
                View::Games => "Games",
                View::GameInfo => "Game Info",
                View::Settings => "Settings",
            }
        }

        /// Get help text for this view
        pub fn help_text(&self) -> &'static str {
            match self {
                View::Systems => {
                    "[↑↓] Navigate  [Enter] Select  [R] Rescan  [Tab] Settings  [Q] Quit"
                }
                View::Games => "[↑↓] Navigate  [Enter] Launch  [F] Favorite  [X] Info  [B] Back",
                View::GameInfo => "[Enter] Launch  [B] Back",
                View::Settings => "[Tab] Back",
            }
        }
    }

    /// Navigation helper for list selection with wraparound
    pub fn navigate_list(current: Option<usize>, len: usize, up: bool) -> Option<usize> {
        if len == 0 {
            return None;
        }

        Some(match current {
            Some(i) => {
                if up {
                    if i == 0 { len - 1 } else { i - 1 }
                } else if i >= len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        })
    }
}
