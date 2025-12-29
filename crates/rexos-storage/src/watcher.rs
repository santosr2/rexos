//! Storage event watcher for hotplug detection

use crate::StorageError;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::time::Duration;

/// Events that can occur on storage devices
#[derive(Debug, Clone)]
pub enum StorageEvent {
    /// A new device was connected
    DeviceAdded { device: PathBuf },
    /// A device was disconnected
    DeviceRemoved { device: PathBuf },
    /// A partition was mounted
    Mounted {
        device: PathBuf,
        mount_point: PathBuf,
    },
    /// A partition was unmounted
    Unmounted { mount_point: PathBuf },
}

/// Watches for storage device changes
pub struct StorageWatcher {
    tx: Sender<StorageEvent>,
    rx: Receiver<StorageEvent>,
    running: bool,
}

impl StorageWatcher {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            tx,
            rx,
            running: false,
        }
    }

    /// Start watching for storage events
    pub fn start(&mut self) -> Result<(), StorageError> {
        if self.running {
            return Ok(());
        }

        self.running = true;
        let tx = self.tx.clone();

        // Spawn thread to watch /dev for changes
        // In production, this would use udev or inotify
        thread::spawn(move || {
            tracing::info!("Storage watcher started");

            // Simple polling approach - production would use udev
            let mut known_devices: std::collections::HashSet<PathBuf> =
                std::collections::HashSet::new();

            loop {
                // Check for mmcblk and sd devices
                if let Ok(entries) = std::fs::read_dir("/dev") {
                    let current_devices: std::collections::HashSet<PathBuf> = entries
                        .filter_map(|e| e.ok())
                        .filter(|e| {
                            let name = e.file_name().to_string_lossy().to_string();
                            (name.starts_with("mmcblk") || name.starts_with("sd"))
                                && !name.contains('p')
                        })
                        .map(|e| e.path())
                        .collect();

                    // Check for new devices
                    for device in current_devices.difference(&known_devices) {
                        let _ = tx.send(StorageEvent::DeviceAdded {
                            device: device.clone(),
                        });
                    }

                    // Check for removed devices
                    for device in known_devices.difference(&current_devices) {
                        let _ = tx.send(StorageEvent::DeviceRemoved {
                            device: device.clone(),
                        });
                    }

                    known_devices = current_devices;
                }

                thread::sleep(Duration::from_secs(2));
            }
        });

        Ok(())
    }

    /// Get the event receiver
    pub fn events(&self) -> &Receiver<StorageEvent> {
        &self.rx
    }

    /// Try to receive an event without blocking
    pub fn try_recv(&self) -> Option<StorageEvent> {
        self.rx.try_recv().ok()
    }

    /// Wait for the next event
    pub fn recv(&self) -> Option<StorageEvent> {
        self.rx.recv().ok()
    }

    /// Wait for an event with timeout
    pub fn recv_timeout(&self, timeout: Duration) -> Option<StorageEvent> {
        self.rx.recv_timeout(timeout).ok()
    }
}

impl Default for StorageWatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watcher_creation() {
        let watcher = StorageWatcher::new();
        assert!(!watcher.running);
    }

    #[test]
    fn test_watcher_default() {
        let watcher = StorageWatcher::default();
        assert!(!watcher.running);
    }

    #[test]
    fn test_storage_event_device_added() {
        let event = StorageEvent::DeviceAdded {
            device: PathBuf::from("/dev/sda"),
        };
        if let StorageEvent::DeviceAdded { device } = event {
            assert_eq!(device, PathBuf::from("/dev/sda"));
        } else {
            panic!("Expected DeviceAdded event");
        }
    }

    #[test]
    fn test_storage_event_device_removed() {
        let event = StorageEvent::DeviceRemoved {
            device: PathBuf::from("/dev/sdb"),
        };
        if let StorageEvent::DeviceRemoved { device } = event {
            assert_eq!(device, PathBuf::from("/dev/sdb"));
        } else {
            panic!("Expected DeviceRemoved event");
        }
    }

    #[test]
    fn test_storage_event_mounted() {
        let event = StorageEvent::Mounted {
            device: PathBuf::from("/dev/sda1"),
            mount_point: PathBuf::from("/mnt/usb"),
        };
        if let StorageEvent::Mounted {
            device,
            mount_point,
        } = event
        {
            assert_eq!(device, PathBuf::from("/dev/sda1"));
            assert_eq!(mount_point, PathBuf::from("/mnt/usb"));
        } else {
            panic!("Expected Mounted event");
        }
    }

    #[test]
    fn test_storage_event_unmounted() {
        let event = StorageEvent::Unmounted {
            mount_point: PathBuf::from("/mnt/usb"),
        };
        if let StorageEvent::Unmounted { mount_point } = event {
            assert_eq!(mount_point, PathBuf::from("/mnt/usb"));
        } else {
            panic!("Expected Unmounted event");
        }
    }

    #[test]
    fn test_try_recv_empty() {
        let watcher = StorageWatcher::new();
        // Should return None when no events are queued
        assert!(watcher.try_recv().is_none());
    }

    #[test]
    fn test_recv_timeout_empty() {
        let watcher = StorageWatcher::new();
        // Should return None when no events within timeout
        let result = watcher.recv_timeout(Duration::from_millis(10));
        assert!(result.is_none());
    }

    #[test]
    fn test_events_accessor() {
        let watcher = StorageWatcher::new();
        let _rx = watcher.events();
        // Just verify we can access the receiver
    }

    #[test]
    fn test_storage_event_clone() {
        let event = StorageEvent::DeviceAdded {
            device: PathBuf::from("/dev/sda"),
        };
        let cloned = event.clone();
        if let StorageEvent::DeviceAdded { device } = cloned {
            assert_eq!(device, PathBuf::from("/dev/sda"));
        }
    }

    #[test]
    fn test_storage_event_debug() {
        let event = StorageEvent::DeviceAdded {
            device: PathBuf::from("/dev/sda"),
        };
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("DeviceAdded"));
        assert!(debug_str.contains("sda"));
    }
}
