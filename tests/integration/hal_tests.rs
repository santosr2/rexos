//! Integration tests for the Hardware Abstraction Layer

use rexos_hal::mock::{MockDevice, MockHal};
use rexos_hal::{DeviceProfile, DisplayConfig, PowerConfig};

#[test]
fn test_mock_device_profiles() {
    let mock = MockHal::new();

    // Test known device profiles
    let profiles = ["RG353M", "RG353V", "RG35XX", "RG503", "RGB30"];

    for profile_name in profiles {
        let profile = mock.profile_for_name(profile_name);
        assert!(profile.is_some(), "Profile {} should exist", profile_name);
    }
}

#[test]
fn test_mock_device_creation() {
    let device = MockDevice::rg353m();
    let profile = device.profile();

    assert_eq!(profile.name, "RG353M");
    assert_eq!(profile.chipset, "RK3566");
    assert_eq!(profile.display.width, 640);
    assert_eq!(profile.display.height, 480);
}

#[test]
fn test_mock_display_operations() {
    let device = MockDevice::rg353m();
    let mut display = device.display().unwrap();

    // Test brightness control
    assert!(display.set_brightness(50).is_ok());
    assert_eq!(display.brightness().unwrap(), 50);

    // Test bounds
    assert!(display.set_brightness(0).is_ok());
    assert!(display.set_brightness(100).is_ok());
}

#[test]
fn test_mock_input_polling() {
    let device = MockDevice::rg353m();
    let mut input = device.input().unwrap();

    // Poll should succeed on mock
    assert!(input.poll().is_ok());
}

#[test]
fn test_mock_power_management() {
    let device = MockDevice::rg353m();
    let power = device.power().unwrap();

    // Should be able to get battery level
    let level = power.battery_level();
    assert!(level.is_some());
    assert!((0..=100).contains(&level.unwrap()));
}

#[test]
fn test_device_profile_serialization() {
    let profile = DeviceProfile {
        name: "TestDevice".to_string(),
        chipset: "TestChip".to_string(),
        display: rexos_hal::DisplaySpec {
            width: 800,
            height: 600,
            dpi: 96,
            refresh_rate: 60,
        },
        has_wifi: true,
        has_bluetooth: true,
        has_analog_sticks: true,
        has_rumble: false,
    };

    // Serialize to JSON
    let json = serde_json::to_string(&profile).expect("Serialization failed");

    // Deserialize back
    let deserialized: DeviceProfile =
        serde_json::from_str(&json).expect("Deserialization failed");

    assert_eq!(profile.name, deserialized.name);
    assert_eq!(profile.chipset, deserialized.chipset);
    assert_eq!(profile.display.width, deserialized.display.width);
}

#[test]
fn test_display_config_defaults() {
    let config = DisplayConfig::default();

    // Should have sensible defaults
    assert!(config.brightness >= 0);
    assert!(config.brightness <= 100);
}

#[test]
fn test_power_config_defaults() {
    let config = PowerConfig::default();

    // Should have a valid governor
    assert!(!config.governor.is_empty());
}

#[test]
fn test_mock_qemu_device() {
    let device = MockDevice::qemu();
    let profile = device.profile();

    assert_eq!(profile.name, "QEMU");
    // QEMU device should have basic capabilities for testing
    assert!(!profile.has_wifi);
    assert!(!profile.has_bluetooth);
}

#[test]
fn test_all_mock_device_variants() {
    // Test that all device factory methods work
    let devices = [
        MockDevice::rg353m(),
        MockDevice::rg353v(),
        MockDevice::rg35xx(),
        MockDevice::rg503(),
        MockDevice::rgb30(),
        MockDevice::qemu(),
    ];

    for device in devices {
        let profile = device.profile();
        assert!(!profile.name.is_empty());
        assert!(!profile.chipset.is_empty());
        assert!(profile.display.width > 0);
        assert!(profile.display.height > 0);
    }
}

#[test]
fn test_input_button_names() {
    use rexos_hal::input::Button;

    let buttons = [
        (Button::A, "A"),
        (Button::B, "B"),
        (Button::X, "X"),
        (Button::Y, "Y"),
        (Button::Start, "Start"),
        (Button::Select, "Select"),
        (Button::Up, "Up"),
        (Button::Down, "Down"),
        (Button::Left, "Left"),
        (Button::Right, "Right"),
        (Button::L1, "L1"),
        (Button::R1, "R1"),
    ];

    for (button, expected_name) in buttons {
        assert_eq!(button.name(), expected_name);
    }
}

#[test]
fn test_analog_stick_values() {
    use rexos_hal::input::AnalogStick;

    let stick = AnalogStick::new(100, 50);

    assert_eq!(stick.x, 100);
    assert_eq!(stick.y, 50);

    // Test normalized values (assuming -32768 to 32767 range)
    let normalized = stick.normalized();
    assert!(normalized.0 >= -1.0 && normalized.0 <= 1.0);
    assert!(normalized.1 >= -1.0 && normalized.1 <= 1.0);
}

#[test]
fn test_display_rotation() {
    use rexos_hal::display::Rotation;

    assert_eq!(Rotation::Normal.degrees(), 0);
    assert_eq!(Rotation::Rotate90.degrees(), 90);
    assert_eq!(Rotation::Rotate180.degrees(), 180);
    assert_eq!(Rotation::Rotate270.degrees(), 270);
}
