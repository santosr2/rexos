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
use std::process::{Command, Stdio};
use std::time::Instant;
use tracing::{debug, error, info, warn};

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
    mount_filesystems()?;
    log_stage_complete(BootStage::Filesystems, stage_start);

    // Stage 2: Initialize hardware
    let stage_start = Instant::now();
    initialize_hardware()?;
    log_stage_complete(BootStage::Hardware, stage_start);

    // Stage 3: Start services
    let stage_start = Instant::now();
    start_services()?;
    log_stage_complete(BootStage::Services, stage_start);

    // Stage 4: Launch frontend
    let stage_start = Instant::now();
    launch_frontend()?;
    log_stage_complete(BootStage::Frontend, stage_start);

    info!("Boot complete in {:?}", boot_start.elapsed());

    // Write boot time to file for monitoring
    write_boot_time(boot_start.elapsed())?;

    // Enter main loop (handle signals, reap zombies)
    main_loop()
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
            // Initiate shutdown
            info!("Received shutdown signal");
            std::process::exit(0);
        }
        libc::SIGUSR1 => {
            // Reload configuration
            info!("Received reload signal");
        }
        libc::SIGUSR2 => {
            // Reserved for future use
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

    // Start essential services
    let services = [
        (
            "dbus",
            "/usr/bin/dbus-daemon",
            &["--system", "--nofork"][..],
        ),
        ("udev", "/sbin/udevd", &["--daemon"][..]),
    ];

    for (name, path, args) in &services {
        if Path::new(path).exists() {
            match start_service(name, path, args) {
                Ok(_) => debug!("Started service: {}", name),
                Err(e) => warn!("Failed to start {}: {}", name, e),
            }
        }
    }

    // Trigger udev to populate /dev
    let _ = Command::new("udevadm").args(["trigger"]).output();
    let _ = Command::new("udevadm")
        .args(["settle", "--timeout=5"])
        .output();

    Ok(())
}

/// Start a service
fn start_service(name: &str, path: &str, args: &[&str]) -> Result<()> {
    Command::new(path)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("Failed to start {}", name))?;

    Ok(())
}

/// Launch the frontend (EmulationStation or custom launcher)
fn launch_frontend() -> Result<()> {
    info!("Launching frontend...");

    let config = rexos_config::RexOSConfig::load_default()?;

    let frontend = match config.system.frontend.as_str() {
        "emulationstation" => "/usr/bin/emulationstation",
        "rexos-launcher" => "/usr/bin/rexos-launcher",
        custom => custom,
    };

    if !Path::new(frontend).exists() {
        error!("Frontend not found: {}", frontend);
        return Ok(());
    }

    // Launch frontend
    Command::new(frontend)
        .stdin(Stdio::null())
        .spawn()
        .with_context(|| format!("Failed to launch {}", frontend))?;

    info!("Frontend launched: {}", frontend);
    Ok(())
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

/// Main loop - handle signals and reap zombies
fn main_loop() -> Result<()> {
    use std::thread;
    use std::time::Duration;

    loop {
        // Sleep and let signal handlers do their work
        thread::sleep(Duration::from_secs(1));

        // Check if we should shutdown
        // This would be set by signal handler in production
    }
}

mod services {
    //! Service management
}

#[allow(dead_code)]
mod shutdown {
    //! Shutdown handling

    use std::process::Command;
    use tracing::info;

    /// Perform clean shutdown
    pub fn shutdown() {
        info!("Initiating shutdown...");

        // Stop services (in reverse order)
        stop_services();

        // Sync filesystems
        let _ = Command::new("sync").output();

        // Unmount filesystems
        unmount_all();

        // Power off
        let _ = Command::new("poweroff").output();
    }

    /// Perform reboot
    pub fn reboot() {
        info!("Initiating reboot...");

        stop_services();
        let _ = Command::new("sync").output();
        unmount_all();
        let _ = Command::new("reboot").output();
    }

    fn stop_services() {
        // Kill all non-essential processes
        let _ = Command::new("pkill")
            .args(["-TERM", "emulationstation"])
            .output();
        let _ = Command::new("pkill").args(["-TERM", "retroarch"]).output();

        // Wait for processes to exit
        std::thread::sleep(std::time::Duration::from_secs(2));
    }

    fn unmount_all() {
        // Unmount in reverse order
        let mounts = ["/roms", "/tmp", "/run"];

        for mount in &mounts {
            let _ = Command::new("umount").arg(mount).output();
        }
    }
}
