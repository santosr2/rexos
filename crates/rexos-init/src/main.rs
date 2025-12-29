//! RexOS Init System
//!
//! Lightweight init system optimized for fast boot on retro gaming handhelds.
//! Handles system initialization, service management, and shutdown.
//!
//! Boot sequence:
//! 1. Mount essential filesystems
//! 2. Initialize hardware (display, input, audio)
//! 3. Start system services
//! 4. Launch frontend (EmulationStation or custom launcher)

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// Global flag to signal shutdown
static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Global flag to signal reboot
static REBOOT_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Boot stages for timing
#[derive(Debug, Clone, Copy)]
enum BootStage {
    Filesystems,
    Hardware,
    Services,
    Frontend,
}

impl BootStage {
    fn name(&self) -> &'static str {
        match self {
            BootStage::Filesystems => "filesystems",
            BootStage::Hardware => "hardware",
            BootStage::Services => "services",
            BootStage::Frontend => "frontend",
        }
    }
}

fn main() -> Result<()> {
    let boot_start = Instant::now();

    // Setup logging
    setup_logging();

    info!("RexOS Init starting...");

    // Check if we're running as PID 1
    let pid = std::process::id();
    if pid != 1 {
        warn!(
            "Not running as PID 1 (pid={}), some features may not work",
            pid
        );
    }

    // Install signal handlers
    setup_signal_handlers()?;

    // Stage 1: Mount filesystems
    let stage_start = Instant::now();
    if let Err(e) = mount_filesystems() {
        error!("CRITICAL: Failed to mount filesystems: {}", e);
        display_boot_error(&format!("Filesystem mount failed: {}", e));
        // Continue anyway - some mounts may have succeeded
    }
    log_stage_complete(BootStage::Filesystems, stage_start);

    // Stage 2: Initialize hardware
    let stage_start = Instant::now();
    if let Err(e) = initialize_hardware() {
        error!("Hardware initialization failed: {}", e);
        display_boot_error(&format!("Hardware init failed: {}", e));
        // Continue - device may still be usable
    }
    log_stage_complete(BootStage::Hardware, stage_start);

    // Stage 3: Start services
    let stage_start = Instant::now();
    if let Err(e) = start_services() {
        error!("Service startup failed: {}", e);
        // Continue - frontend may still work
    }
    log_stage_complete(BootStage::Services, stage_start);

    // Stage 4: Launch frontend
    let stage_start = Instant::now();
    let frontend_child = match launch_frontend() {
        Ok(child) => child,
        Err(e) => {
            error!("CRITICAL: Frontend launch failed: {}", e);
            display_boot_error(&format!("Frontend launch failed: {}", e));
            None
        }
    };
    log_stage_complete(BootStage::Frontend, stage_start);

    info!("Boot complete in {:?}", boot_start.elapsed());

    // Write boot time to file for monitoring
    let _ = write_boot_time(boot_start.elapsed());

    // Enter main loop (handle signals, reap zombies, watchdog frontend)
    main_loop(frontend_child)
}

/// Setup logging to console and file
fn setup_logging() {
    use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(false).with_ansi(false))
        .init();
}

/// Setup signal handlers for graceful shutdown
fn setup_signal_handlers() -> Result<()> {
    use nix::sys::signal::{SaFlags, SigAction, SigHandler, SigSet, Signal, sigaction};

    let action = SigAction::new(
        SigHandler::Handler(handle_signal),
        SaFlags::empty(),
        SigSet::empty(),
    );

    unsafe {
        sigaction(Signal::SIGTERM, &action)?;
        sigaction(Signal::SIGINT, &action)?;
        sigaction(Signal::SIGUSR1, &action)?;
        sigaction(Signal::SIGUSR2, &action)?;
    }

    // Handle SIGCHLD to reap zombies
    let sigchld_action = SigAction::new(
        SigHandler::Handler(handle_sigchld),
        SaFlags::SA_NOCLDSTOP,
        SigSet::empty(),
    );

    unsafe {
        sigaction(Signal::SIGCHLD, &sigchld_action)?;
    }

    Ok(())
}

/// Signal handler
extern "C" fn handle_signal(sig: i32) {
    match sig {
        libc::SIGTERM | libc::SIGINT => {
            // Set shutdown flag - actual shutdown happens in main loop
            SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
        }
        libc::SIGUSR1 => {
            // Reload configuration - just log it, config is reloaded on next access
        }
        libc::SIGUSR2 => {
            // Set reboot flag - actual reboot happens in main loop
            REBOOT_REQUESTED.store(true, Ordering::SeqCst);
        }
        _ => {}
    }
}

/// SIGCHLD handler to reap zombie processes
extern "C" fn handle_sigchld(_sig: i32) {
    use nix::sys::wait::{WaitPidFlag, WaitStatus, waitpid};

    loop {
        match waitpid(nix::unistd::Pid::from_raw(-1), Some(WaitPidFlag::WNOHANG)) {
            Ok(WaitStatus::Exited(pid, status)) => {
                debug!("Child {} exited with status {}", pid, status);
            }
            Ok(WaitStatus::Signaled(pid, signal, _)) => {
                debug!("Child {} killed by signal {:?}", pid, signal);
            }
            Ok(WaitStatus::StillAlive) | Err(_) => break,
            _ => continue,
        }
    }
}

/// Mount essential filesystems
fn mount_filesystems() -> Result<()> {
    info!("Mounting filesystems...");

    // These may already be mounted by the kernel, but ensure they exist
    let mounts = [
        ("/proc", "proc", "proc"),
        ("/sys", "sysfs", "sysfs"),
        ("/dev", "devtmpfs", "devtmpfs"),
        ("/dev/pts", "devpts", "devpts"),
        ("/dev/shm", "tmpfs", "tmpfs"),
        ("/run", "tmpfs", "tmpfs"),
        ("/tmp", "tmpfs", "tmpfs"),
    ];

    for (mount_point, fstype, device) in &mounts {
        if !is_mounted(mount_point) {
            do_mount(device, mount_point, fstype)?;
        }
    }

    // Mount config partition if available (typically second partition on SD)
    mount_roms_partition()?;

    Ok(())
}

/// Check if a path is mounted
fn is_mounted(path: &str) -> bool {
    fs::read_to_string("/proc/mounts")
        .map(|mounts| {
            mounts
                .lines()
                .any(|line| line.split_whitespace().nth(1) == Some(path))
        })
        .unwrap_or(false)
}

/// Mount a filesystem
fn do_mount(source: &str, target: &str, fstype: &str) -> Result<()> {
    // Create mount point if it doesn't exist
    fs::create_dir_all(target)?;

    // Use the mount command for compatibility
    let status = Command::new("mount")
        .args(["-t", fstype, source, target])
        .output()
        .with_context(|| format!("Failed to execute mount for {} at {}", source, target))?;

    if !status.status.success() {
        return Err(anyhow::anyhow!(
            "Mount failed: {}",
            String::from_utf8_lossy(&status.stderr)
        ));
    }

    debug!("Mounted {} at {} ({})", source, target, fstype);
    Ok(())
}

/// Mount ROMs partition (external SD or second partition)
fn mount_roms_partition() -> Result<()> {
    let roms_mount = "/roms";

    if is_mounted(roms_mount) {
        return Ok(());
    }

    fs::create_dir_all(roms_mount)?;

    // Try common ROM partition locations
    let candidates = [
        "/dev/mmcblk1p1", // External SD card
        "/dev/mmcblk0p3", // Third partition on internal
        "/dev/sda1",      // USB drive
    ];

    for device in &candidates {
        if Path::new(device).exists() {
            // Try to mount with auto-detect filesystem
            let result = Command::new("mount")
                .args(["-o", "rw,noatime", device, roms_mount])
                .output();

            if result.map(|o| o.status.success()).unwrap_or(false) {
                info!("Mounted ROMs partition from {}", device);
                return Ok(());
            }
        }
    }

    warn!("No ROMs partition found, using internal storage");
    Ok(())
}

/// Initialize hardware
fn initialize_hardware() -> Result<()> {
    info!("Initializing hardware...");

    // Load device profile
    let device = rexos_hal::Device::detect()?;
    info!(
        "Detected device: {} ({})",
        device.profile().name,
        device.profile().chipset
    );

    // Initialize display
    init_display(&device)?;

    // Initialize input
    init_input(&device)?;

    // Initialize audio
    init_audio(&device)?;

    // Initialize power management
    init_power(&device)?;

    Ok(())
}

/// Initialize display
fn init_display(_device: &rexos_hal::Device) -> Result<()> {
    // Set initial brightness
    let config = rexos_config::RexOSConfig::load_default()?;
    let brightness = config.system.brightness;

    // Create display manager and set brightness
    let display_config = rexos_hal::DisplayConfig::default();
    match rexos_hal::Display::new(display_config) {
        Ok(mut display) => {
            if let Err(e) = display.set_brightness(brightness) {
                warn!("Failed to set display brightness: {}", e);
            }
        }
        Err(e) => {
            warn!("Failed to initialize display: {}", e);
        }
    }

    // Show splash screen if enabled
    if config.system.splash_screen {
        show_splash_screen()?;
    }

    debug!("Display initialized");
    Ok(())
}

/// Show boot splash screen
fn show_splash_screen() -> Result<()> {
    // Look for splash image
    let splash_paths = ["/etc/rexos/splash.png", "/usr/share/rexos/splash.png"];

    for path in &splash_paths {
        if Path::new(path).exists() {
            // Use fbset or similar to display splash
            let _ = Command::new("fbsplash")
                .args(["-s", path, "-d", "/dev/fb0"])
                .spawn();
            break;
        }
    }

    Ok(())
}

/// Initialize input
fn init_input(_device: &rexos_hal::Device) -> Result<()> {
    // Input is typically handled by Linux input subsystem
    // Just verify common input devices exist

    let input_paths = ["/dev/input/event0", "/dev/input/event1", "/dev/input/js0"];

    let mut found = false;
    for path in &input_paths {
        if Path::new(path).exists() {
            debug!("Input device found: {}", path);
            found = true;
            break;
        }
    }

    if !found {
        warn!("No input devices found in /dev/input/");
    }

    Ok(())
}

/// Initialize audio
fn init_audio(_device: &rexos_hal::Device) -> Result<()> {
    // Set initial volume
    let config = rexos_config::RexOSConfig::load_default()?;

    // Use amixer to set volume
    let _ = Command::new("amixer")
        .args(["sset", "Master", &format!("{}%", config.system.volume)])
        .output();

    debug!("Audio initialized");
    Ok(())
}

/// Initialize power management
fn init_power(_device: &rexos_hal::Device) -> Result<()> {
    let config = rexos_config::RexOSConfig::load_default()?;

    // Set CPU governor based on performance profile
    let governor = match config.system.performance {
        rexos_config::PerformanceProfile::Powersave => "powersave",
        rexos_config::PerformanceProfile::Balanced => "schedutil",
        rexos_config::PerformanceProfile::Performance => "performance",
    };

    // Set governor for all CPUs
    let governor_path = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor";
    if Path::new(governor_path).exists() {
        let _ = fs::write(governor_path, governor);
        debug!("CPU governor set to {}", governor);
    } else {
        debug!("CPU governor path not found, skipping");
    }

    Ok(())
}

/// Start system services
fn start_services() -> Result<()> {
    info!("Starting services...");

    // Use the services module to start essential services
    services::start_essential();

    // Trigger udev to populate /dev
    let _ = Command::new("udevadm").args(["trigger"]).output();
    let _ = Command::new("udevadm")
        .args(["settle", "--timeout=5"])
        .output();

    Ok(())
}

/// Launch the frontend (EmulationStation or custom launcher)
/// Returns the child process handle for watchdog monitoring
fn launch_frontend() -> Result<Option<Child>> {
    info!("Launching frontend...");

    let config = rexos_config::RexOSConfig::load_default()?;

    let frontend = match config.system.frontend.as_str() {
        "emulationstation" => "/usr/bin/emulationstation",
        "rexos-launcher" => "/usr/bin/rexos-launcher",
        custom => custom,
    };

    if !Path::new(frontend).exists() {
        error!("Frontend not found: {} - BOOT WILL FAIL", frontend);
        // Display error on screen if possible
        display_boot_error(&format!("Frontend not found: {}", frontend));
        return Ok(None);
    }

    // Launch frontend
    let child = Command::new(frontend)
        .stdin(Stdio::null())
        .spawn()
        .with_context(|| format!("Failed to launch {}", frontend))?;

    info!("Frontend launched: {} (PID {})", frontend, child.id());
    Ok(Some(child))
}

/// Display a boot error on screen for user visibility
fn display_boot_error(message: &str) {
    error!("BOOT ERROR: {}", message);

    // Try to write to console
    let _ = fs::write(
        "/dev/console",
        format!("\n\n*** REXOS BOOT ERROR ***\n{}\n\n", message),
    );

    // Also try fbset text mode if available
    let _ = Command::new("fbset").args(["-depth", "8"]).output();
}

/// Log stage completion with timing
fn log_stage_complete(stage: BootStage, start: Instant) {
    info!("Stage {} complete in {:?}", stage.name(), start.elapsed());
}

/// Write boot time to file for monitoring
fn write_boot_time(duration: std::time::Duration) -> Result<()> {
    let path = "/run/rexos-boot-time";
    fs::write(path, format!("{}", duration.as_millis()))?;
    Ok(())
}

/// Main loop - handle signals, reap zombies, and watchdog frontend
fn main_loop(mut frontend_child: Option<Child>) -> Result<()> {
    use std::thread;

    const WATCHDOG_INTERVAL: Duration = Duration::from_secs(5);
    const MAX_FRONTEND_RESTARTS: u32 = 3;
    const RESTART_COOLDOWN: Duration = Duration::from_secs(30);

    let mut restart_count = 0u32;
    let mut last_restart = Instant::now();

    info!("Entering main loop (watchdog active)");

    loop {
        // Check shutdown/reboot flags (set by signal handlers)
        if SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
            info!("Shutdown requested via signal");
            shutdown::shutdown();
            return Ok(());
        }

        if REBOOT_REQUESTED.load(Ordering::SeqCst) {
            info!("Reboot requested via signal");
            shutdown::reboot();
            return Ok(());
        }

        // Watchdog: Check if frontend is still running
        if let Some(ref mut child) = frontend_child {
            match child.try_wait() {
                Ok(Some(status)) => {
                    // Frontend exited
                    if status.success() {
                        info!("Frontend exited normally");
                        // User may have requested shutdown via frontend
                        if SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
                            shutdown::shutdown();
                            return Ok(());
                        }
                    } else {
                        error!("Frontend crashed with status: {:?}", status);
                    }

                    // Attempt restart with rate limiting
                    if last_restart.elapsed() > RESTART_COOLDOWN {
                        restart_count = 0;
                    }

                    if restart_count < MAX_FRONTEND_RESTARTS {
                        warn!(
                            "Restarting frontend (attempt {}/{})",
                            restart_count + 1,
                            MAX_FRONTEND_RESTARTS
                        );

                        // Small delay before restart
                        thread::sleep(Duration::from_secs(2));

                        match launch_frontend() {
                            Ok(new_child) => {
                                frontend_child = new_child;
                                restart_count += 1;
                                last_restart = Instant::now();
                                info!("Frontend restarted successfully");
                            }
                            Err(e) => {
                                error!("Failed to restart frontend: {}", e);
                                display_boot_error(&format!("Frontend restart failed: {}", e));
                            }
                        }
                    } else {
                        error!(
                            "Frontend crashed {} times in {}s - giving up",
                            MAX_FRONTEND_RESTARTS,
                            RESTART_COOLDOWN.as_secs()
                        );
                        display_boot_error("Frontend keeps crashing - system may be unstable");
                        frontend_child = None;
                    }
                }
                Ok(None) => {
                    // Still running - good
                }
                Err(e) => {
                    warn!("Failed to check frontend status: {}", e);
                }
            }
        }

        // Sleep and let signal handlers do their work
        thread::sleep(WATCHDOG_INTERVAL);
    }
}

mod services {
    //! Service management utilities
    //!
    //! Provides functions for starting, stopping, and managing system services.
    //! Some functions are marked as `#[allow(dead_code)]` as they provide a
    //! complete API for service management, even if not all are currently used.

    use std::path::Path;
    use std::process::{Command, Stdio};
    use tracing::{debug, warn};

    /// Service definition
    pub struct ServiceDef {
        pub name: &'static str,
        pub path: &'static str,
        pub args: &'static [&'static str],
    }

    /// Essential system services to start at boot
    pub const ESSENTIAL_SERVICES: &[ServiceDef] = &[
        ServiceDef {
            name: "dbus",
            path: "/usr/bin/dbus-daemon",
            args: &["--system", "--nofork"],
        },
        ServiceDef {
            name: "udev",
            path: "/sbin/udevd",
            args: &["--daemon"],
        },
    ];

    /// Start a service by name
    pub fn start(name: &str, path: &str, args: &[&str]) -> Result<u32, String> {
        if !Path::new(path).exists() {
            return Err(format!("Service binary not found: {}", path));
        }

        let child = Command::new(path)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to spawn {}: {}", name, e))?;

        let pid = child.id();
        debug!("Started service {} with PID {}", name, pid);
        Ok(pid)
    }

    /// Stop a service by name using pkill
    pub fn stop(name: &str) {
        let _ = Command::new("pkill").args(["-TERM", name]).output();
        debug!("Sent SIGTERM to {}", name);
    }

    /// Stop a service forcefully
    #[allow(dead_code)] // Part of service management API
    pub fn kill(name: &str) {
        let _ = Command::new("pkill").args(["-KILL", name]).output();
        warn!("Sent SIGKILL to {}", name);
    }

    /// Check if a service is running
    #[allow(dead_code)] // Part of service management API
    pub fn is_running(name: &str) -> bool {
        Command::new("pgrep")
            .arg(name)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Start all essential services
    pub fn start_essential() {
        for svc in ESSENTIAL_SERVICES {
            if Path::new(svc.path).exists() {
                match start(svc.name, svc.path, svc.args) {
                    Ok(pid) => debug!("Started {} (PID {})", svc.name, pid),
                    Err(e) => warn!("Failed to start {}: {}", svc.name, e),
                }
            }
        }
    }

    /// Stop all non-essential services (for shutdown)
    pub fn stop_all_nonessential() {
        // Stop user-facing services first
        let services = ["emulationstation", "retroarch", "rexos-launcher"];
        for svc in &services {
            stop(svc);
        }

        // Give them time to exit gracefully
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}

mod shutdown {
    //! Shutdown and reboot handling
    //!
    //! Provides clean shutdown and reboot procedures that properly
    //! stop services, sync filesystems, and unmount partitions.

    use super::services;
    use std::process::Command;
    use tracing::info;

    /// Perform clean shutdown
    pub fn shutdown() {
        info!("Initiating shutdown...");

        // Stop services (in reverse order)
        services::stop_all_nonessential();

        // Sync filesystems
        info!("Syncing filesystems...");
        let _ = Command::new("sync").output();

        // Unmount filesystems
        unmount_all();

        // Power off
        info!("Powering off...");
        let _ = Command::new("poweroff").output();

        // If poweroff fails, exit
        std::process::exit(0);
    }

    /// Perform reboot
    pub fn reboot() {
        info!("Initiating reboot...");

        services::stop_all_nonessential();

        info!("Syncing filesystems...");
        let _ = Command::new("sync").output();

        unmount_all();

        info!("Rebooting...");
        let _ = Command::new("reboot").output();

        // If reboot fails, exit
        std::process::exit(0);
    }

    fn unmount_all() {
        info!("Unmounting filesystems...");

        // Unmount in reverse order of mount priority
        let mounts = ["/roms", "/tmp", "/run", "/dev/shm", "/dev/pts"];

        for mount in &mounts {
            let _ = Command::new("umount").arg(mount).output();
        }
    }
}
