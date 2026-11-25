use std::fs;
use std::path::{Path, PathBuf};

use crate::snapshot::parse_active_bracketed;
use crate::sysfs::{read_trimmed, read_u64, write_value};

#[derive(Debug, PartialEq)]
pub struct BatteryInfo {
    pub name: String,
    pub status: String,
    pub capacity: String,
    pub power_w: Option<f64>,
    pub start_threshold: Option<String>,
    pub stop_threshold: Option<String>,
    pub charge_type: Option<String>,
    pub supports_fast_charge: bool,
}

pub fn detect_batteries_in(power_supply_root: &Path) -> Vec<PathBuf> {
    let mut bats = Vec::new();
    let Ok(entries) = fs::read_dir(power_supply_root) else {
        return bats;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with("BAT") {
            bats.push(entry.path());
        }
    }
    bats.sort();
    bats
}

pub fn read_battery_info(bat: &Path) -> BatteryInfo {
    let name = bat
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let status = read_trimmed(&bat.join("status")).unwrap_or_else(|| "Unknown".into());
    let capacity = read_trimmed(&bat.join("capacity")).unwrap_or_else(|| "?".into());
    let start_threshold = read_trimmed(&bat.join("charge_control_start_threshold"));
    let stop_threshold = read_trimmed(&bat.join("charge_control_end_threshold"));
    let charge_types_raw = read_trimmed(&bat.join("charge_types"));
    let supports_fast_charge = charge_types_raw.as_deref().is_some_and(|raw| {
        raw.split_whitespace()
            .any(|t| t.trim_matches(['[', ']']) == "Fast")
    });
    let charge_type = charge_types_raw.as_deref().map(parse_active_bracketed);

    let power_w = if let Some(pw) = read_u64(&bat.join("power_now")) {
        Some(pw as f64 / 1_000_000.0)
    } else if let (Some(curr), Some(volt)) = (
        read_u64(&bat.join("current_now")),
        read_u64(&bat.join("voltage_now")),
    ) {
        Some((curr as f64 / 1_000_000.0) * (volt as f64 / 1_000_000.0))
    } else {
        None
    };

    BatteryInfo {
        name,
        status,
        capacity,
        power_w,
        start_threshold,
        stop_threshold,
        charge_type,
        supports_fast_charge,
    }
}

pub fn set_fast_charge_in(
    power_supply_root: &Path,
    enabled: bool,
    dry_run: bool,
) -> Result<(), String> {
    let bats = detect_batteries_in(power_supply_root);
    let mut updated = false;
    for bat in bats {
        let ct_path = bat.join("charge_types");
        let Some(raw) = read_trimmed(&ct_path) else {
            continue;
        };
        let has = |name: &str| {
            raw.split_whitespace()
                .any(|tok| tok.trim_matches(['[', ']']) == name)
        };
        if !has("Fast") {
            continue;
        }
        let target = if enabled {
            "Fast"
        } else if has("Custom") {
            "Custom"
        } else if has("Standard") {
            "Standard"
        } else {
            "Adaptive"
        };
        write_value(&ct_path, target, dry_run)?;
        updated = true;
    }
    if !updated {
        return Err("Fast charge is not exposed by kernel (no charge_types with Fast).".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sysfs::test_utils::{write_mock, TempDir};

    #[test]
    fn test_read_battery_info_fallback_current_times_voltage() {
        let tmp = TempDir::new("bat");
        let bat = tmp.0.join("BAT0");
        write_mock(&bat.join("status"), "Discharging\n");
        write_mock(&bat.join("capacity"), "50\n");
        write_mock(&bat.join("current_now"), "2000000\n");
        write_mock(&bat.join("voltage_now"), "12500000\n");
        write_mock(&bat.join("charge_control_start_threshold"), "50\n");
        write_mock(&bat.join("charge_control_end_threshold"), "80\n");
        write_mock(
            &bat.join("charge_types"),
            "Trickle Fast Standard Adaptive [Custom]\n",
        );

        let info = read_battery_info(&bat);
        assert_eq!(info.name, "BAT0");
        assert_eq!(info.status, "Discharging");
        assert_eq!(info.capacity, "50");
        assert!((info.power_w.unwrap() - 25.0).abs() < 1e-6);
        assert_eq!(info.start_threshold.as_deref(), Some("50"));
        assert_eq!(info.stop_threshold.as_deref(), Some("80"));
        assert_eq!(info.charge_type.as_deref(), Some("Custom"));
        assert!(info.supports_fast_charge);

        set_fast_charge_in(&tmp.0, true, false).unwrap();
        assert_eq!(
            read_trimmed(&bat.join("charge_types")).as_deref(),
            Some("Fast")
        );
    }
}
