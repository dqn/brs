// Audio device enumeration via cpal.
//
// Provides output device listing for the launcher's device selection dropdown.

use cpal::traits::{DeviceTrait, HostTrait};

/// Information about an available audio output device.
#[derive(Debug, Clone)]
pub struct AudioDeviceInfo {
    /// Human-readable device name.
    pub name: String,
    /// Whether this is the system default output device.
    pub is_default: bool,
}

/// Enumerate available audio output devices.
///
/// Returns a list of output devices with their names and default status.
/// If enumeration fails, returns an empty list.
pub fn enumerate_output_devices() -> Vec<AudioDeviceInfo> {
    let host = cpal::default_host();

    let default_name = host.default_output_device().and_then(|d| d.name().ok());

    host.output_devices()
        .map(|devices| {
            devices
                .filter_map(|d| {
                    let name = d.name().ok()?;
                    Some(AudioDeviceInfo {
                        is_default: default_name.as_deref() == Some(&name),
                        name,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enumerate_returns_at_least_empty() {
        // On CI or headless environments, there may be no devices.
        // Just verify it doesn't panic.
        let devices = enumerate_output_devices();
        // If there are devices, at most one should be default
        let default_count = devices.iter().filter(|d| d.is_default).count();
        assert!(default_count <= 1);
    }
}
