//! RexOS Launcher
//!
//! A TUI-based game launcher for RexOS, optimized for handheld devices.
//! Provides game browsing, launching, and basic settings management.

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tracing::{info, error};

use rexos_library::{GameDatabase, Game, RomScanner};
use rexos_emulator::{EmulatorLauncher, LaunchConfig};
use rexos_config::RexOSConfig;

/// Application state
struct App {
    /// Game database
    db: GameDatabase,

    /// Emulator launcher
    launcher: EmulatorLauncher,

    /// Configuration
    config: RexOSConfig,

    /// Current view
    view: View,

    /// Systems list state
    systems_state: ListState,

    /// Games list state
    games_state: ListState,

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
    /// Create new application
    fn new() -> Result<Self> {
        // Open game database
        let db_path = PathBuf::from("/roms/.rexos/games.db");

        // Create directory if needed
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let db = GameDatabase::open(&db_path)?;

        // Load configuration
        let config = RexOSConfig::load_default()?;

        // Create launcher
        let launcher = EmulatorLauncher::new();

        // Get systems
        let systems = db.get_systems()?;

        let mut app = Self {
            db,
            launcher,
            config,
            view: View::Systems,
            systems_state: ListState::default(),
            games_state: ListState::default(),
            systems,
            games: Vec::new(),
            selected_system: None,
            status: "Ready".to_string(),
            should_quit: false,
        };

        // Select first system if available
        if !app.systems.is_empty() {
            app.systems_state.select(Some(0));
        }

        Ok(app)
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
        match key {
            KeyCode::Up | KeyCode::Char('w') => {
                self.select_prev_system();
            }
            KeyCode::Down | KeyCode::Char('s') => {
                self.select_next_system();
            }
            KeyCode::Enter | KeyCode::Char('a') => {
                self.enter_system()?;
            }
            KeyCode::Char('r') => {
                self.rescan_roms()?;
            }
            KeyCode::Tab => {
                self.view = View::Settings;
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.should_quit = true;
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle games view input
    fn handle_games_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Up | KeyCode::Char('w') => {
                self.select_prev_game();
            }
            KeyCode::Down | KeyCode::Char('s') => {
                self.select_next_game();
            }
            KeyCode::Enter | KeyCode::Char('a') => {
                self.launch_selected_game()?;
            }
            KeyCode::Char('x') => {
                self.show_game_info();
            }
            KeyCode::Char('f') => {
                self.toggle_favorite()?;
            }
            KeyCode::Esc | KeyCode::Char('b') | KeyCode::Backspace => {
                self.view = View::Systems;
                self.games.clear();
                self.games_state.select(None);
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle game info view input
    fn handle_game_info_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Enter | KeyCode::Char('a') => {
                self.launch_selected_game()?;
            }
            KeyCode::Esc | KeyCode::Char('b') | KeyCode::Backspace => {
                self.view = View::Games;
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle settings view input
    fn handle_settings_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Esc | KeyCode::Tab | KeyCode::Char('b') => {
                self.view = View::Systems;
            }
            _ => {}
        }
        Ok(())
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
        if let Some(i) = self.systems_state.selected() {
            if i < self.systems.len() {
                let system = &self.systems[i].0;
                self.selected_system = Some(system.clone());
                self.games = self.db.get_games_by_system(system)?;
                self.view = View::Games;

                if !self.games.is_empty() {
                    self.games_state.select(Some(0));
                }

                self.status = format!("{} games", self.games.len());
            }
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
        if let Some(i) = self.games_state.selected() {
            if i < self.games.len() {
                let game = &mut self.games[i];
                game.favorite = !game.favorite;
                self.db.set_favorite(game.id, game.favorite)?;

                self.status = if game.favorite {
                    "Added to favorites".to_string()
                } else {
                    "Removed from favorites".to_string()
                };
            }
        }
        Ok(())
    }

    /// Launch selected game
    fn launch_selected_game(&mut self) -> Result<()> {
        if let Some(i) = self.games_state.selected() {
            if i < self.games.len() {
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
        }
        Ok(())
    }

    /// Rescan ROMs
    fn rescan_roms(&mut self) -> Result<()> {
        self.status = "Scanning ROMs...".to_string();

        let scanner = RomScanner::new();
        let roms_dir = PathBuf::from("/roms");

        if let Ok(results) = scanner.scan_all(&roms_dir) {
            let mut total_games = 0;

            for (system, games) in results {
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
        self.games_state.selected()
            .and_then(|i| self.games.get(i))
    }
}

/// Draw the UI
fn draw_ui(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Main content
            Constraint::Length(3),  // Footer
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
        View::Games => &format!("RexOS - {}", app.selected_system.as_deref().unwrap_or("Games")),
        View::GameInfo => "RexOS - Game Info",
        View::Settings => "RexOS - Settings",
    };

    let header = Paragraph::new(title)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(header, area);
}

/// Draw systems view
fn draw_systems_view(frame: &mut Frame, area: Rect, app: &mut App) {
    let items: Vec<ListItem> = app.systems
        .iter()
        .map(|(name, count)| {
            let display = format!("{:<20} ({} games)", name, count);
            ListItem::new(display)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Systems"))
        .highlight_style(Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, &mut app.systems_state);
}

/// Draw games view
fn draw_games_view(frame: &mut Frame, area: Rect, app: &mut App) {
    let items: Vec<ListItem> = app.games
        .iter()
        .map(|game| {
            let prefix = if game.favorite { "★ " } else { "  " };
            let display = format!("{}{}", prefix, game.name);
            ListItem::new(display)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Games"))
        .highlight_style(Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, &mut app.games_state);
}

/// Draw game info view
fn draw_game_info_view(frame: &mut Frame, area: Rect, app: &App) {
    let content = if let Some(game) = app.selected_game() {
        let mut lines = vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&game.name),
            ]),
            Line::from(vec![
                Span::styled("System: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&game.system),
            ]),
            Line::from(vec![
                Span::styled("Path: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&game.path),
            ]),
        ];

        if let Some(ref desc) = game.description {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Description: ", Style::default().add_modifier(Modifier::BOLD)),
            ]));
            lines.push(Line::from(desc.as_str()));
        }

        if let Some(ref dev) = game.developer {
            lines.push(Line::from(vec![
                Span::styled("Developer: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(dev),
            ]));
        }

        if let Some(rating) = game.rating {
            lines.push(Line::from(vec![
                Span::styled("Rating: ", Style::default().add_modifier(Modifier::BOLD)),
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

/// Draw settings view
fn draw_settings_view(frame: &mut Frame, area: Rect, _app: &App) {
    let settings_text = vec![
        Line::from("Settings (coming soon)"),
        Line::from(""),
        Line::from("• Brightness"),
        Line::from("• Volume"),
        Line::from("• WiFi"),
        Line::from("• Bluetooth"),
        Line::from("• Updates"),
    ];

    let paragraph = Paragraph::new(settings_text)
        .block(Block::default().borders(Borders::ALL).title("Settings"));

    frame.render_widget(paragraph, area);
}

/// Draw footer
fn draw_footer(frame: &mut Frame, area: Rect, app: &App) {
    let help_text = match app.view {
        View::Systems => "[↑↓] Navigate  [Enter] Select  [R] Rescan  [Tab] Settings  [Q] Quit",
        View::Games => "[↑↓] Navigate  [Enter] Launch  [F] Favorite  [X] Info  [B] Back",
        View::GameInfo => "[Enter] Launch  [B] Back",
        View::Settings => "[Tab] Back",
    };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));

    let status = Paragraph::new(app.status.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(help, chunks[0]);
    frame.render_widget(status, chunks[1]);
}

fn main() -> Result<()> {
    // Setup logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

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
    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| draw_ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_input(key.code)?;
                }
            }
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
    //! UI components
}

mod input {
    //! Input handling
}

mod state {
    //! Application state
}
