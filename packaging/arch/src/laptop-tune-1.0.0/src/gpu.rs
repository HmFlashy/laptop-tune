use std::fs;
use std::path::Path;

use crate::sysfs::read_trimmed;

#[derive(Debug, PartialEq)]
pub struct GpuDevice {
    pub pci_slot: String,
    pub vendor: String,
    pub class: String,
    pub driver: String,
    pub runtime_status: String,
    pub power_control: String,
}

pub fn vendor_name(vendor: &str) -> &'static str {
    match vendor {
        "0x8086" => "Intel",
        "0x10de" => "NVIDIA",
        "0x1002" => "AMD",
        _ => "Unknown",
    }
}

pub fn detect_gpus_in(pci_root: &Path) -> Vec<GpuDevice> {
    let mut gpus = Vec::new();
    let Ok(entries) = fs::read_dir(pci_root) else {
        return gpus;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(class) = read_trimmed(&path.join("class")) else {
            continue;
        };
        if !class.starts_with("0x03") {
            continue;
        }
        let pci_slot = entry.file_name().to_string_lossy().into_owned();
        let vendor = read_trimmed(&path.join("vendor")).unwrap_or_else(|| "unknown".into());
        let driver = fs::read_link(path.join("driver"))
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "none".into());
        let runtime_status =
            read_trimmed(&path.join("power/runtime_status")).unwrap_or_else(|| "unknown".into());
        let power_control =
            read_trimmed(&path.join("power/control")).unwrap_or_else(|| "unknown".into());

        gpus.push(GpuDevice {
            pci_slot,
            vendor,
            class,
            driver,
            runtime_status,
            power_control,
        });
    }

    gpus.sort_by(|a, b| a.pci_slot.cmp(&b.pci_slot));
    gpus
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sysfs::test_utils::{write_mock, TempDir};

    #[test]
    fn test_vendor_name() {
        assert_eq!(vendor_name("0x8086"), "Intel");
        assert_eq!(vendor_name("0x10de"), "NVIDIA");
        assert_eq!(vendor_name("0x1002"), "AMD");
        assert_eq!(vendor_name("0x9999"), "Unknown");
    }

    #[test]
    fn test_detect_gpus_filters_non_display_and_sorts() {
        let tmp = TempDir::new("gpus");
        let pci = tmp.0.join("pci");

        write_mock(&pci.join("0000:00:14.3/class"), "0x028000\n");
        write_mock(&pci.join("0000:00:14.3/vendor"), "0x8086\n");

        write_mock(&pci.join("0000:01:00.0/class"), "0x030200\n");
        write_mock(&pci.join("0000:01:00.0/vendor"), "0x10de\n");
        write_mock(
            &pci.join("0000:01:00.0/power/runtime_status"),
            "suspended\n",
        );
        write_mock(&pci.join("0000:01:00.0/power/control"), "auto\n");

        write_mock(&pci.join("0000:00:02.0/class"), "0x030000\n");
        write_mock(&pci.join("0000:00:02.0/vendor"), "0x8086\n");
        write_mock(&pci.join("0000:00:02.0/power/runtime_status"), "active\n");
        write_mock(&pci.join("0000:00:02.0/power/control"), "on\n");

        let gpus = detect_gpus_in(&pci);
        assert_eq!(gpus.len(), 2);
        assert_eq!(gpus[0].pci_slot, "0000:00:02.0");
        assert_eq!(gpus[0].vendor, "0x8086");
        assert_eq!(gpus[1].pci_slot, "0000:01:00.0");
        assert_eq!(gpus[1].runtime_status, "suspended");
    }
}
