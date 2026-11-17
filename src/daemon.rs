use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};

use crate::battery::detect_batteries_in;
use crate::config::{load_system_config_from, SystemConfig};
use crate::gpu::detect_gpus_in;
use crate::profile::apply_profile_in;
use crate::sysfs::{read_trimmed, write_value};
use crate::tray::notify_tray_refresh;

pub const SYSTEM_CONFIG_PATH: &str = "/etc/laptop-tune/config.conf";

pub fn is_on_ac_in(power_supply_root: &Path) -> bool {
    let Ok(entries) = fs::read_dir(power_supply_root) else {
        return true;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        let ps_type = read_trimmed(&p.join("type")).unwrap_or_default();
        if ps_type == "Mains" {
            if let Some(online) = read_trimmed(&p.join("online")) {
                if online == "1" {
                    return true;
                }
            }
        }
    }
    false
}

pub fn apply_battery_thresholds_in(
    power_supply_root: &Path,
    start: u8,
    stop: u8,
    dry_run: bool,
) -> Result<(), String> {
    if start >= stop {
        return Err(format!(
            "Invalid thresholds: start ({start}) must be < stop ({stop})"
        ));
    }
    for bat in detect_batteries_in(power_supply_root) {
        let end_path = bat.join("charge_control_end_threshold");
        let start_path = bat.join("charge_control_start_threshold");
        // Write end=100 first if raising, or start=0 first if lowering, to avoid kernel EINVAL.
        if end_path.exists() && start_path.exists() {
            let _ = write_value(&end_path, "100", dry_run);
            write_value(&start_path, &start.to_string(), dry_run)?;
            write_value(&end_path, &stop.to_string(), dry_run)?;
        } else if end_path.exists() {
            write_value(&end_path, &stop.to_string(), dry_run)?;
        }
    }
    Ok(())
}

pub fn enable_gpu_runtime_pm_in(pci_root: &Path, dry_run: bool) {
    let gpus = detect_gpus_in(pci_root);
    for (idx, gpu) in gpus.iter().enumerate() {
        if gpu.vendor == "0x10de" || (gpus.len() > 1 && idx > 0) {
            let ctrl = pci_root.join(&gpu.pci_slot).join("power/control");
            if ctrl.exists() {
                let _ = write_value(&ctrl, "auto", dry_run);
            }
        }
    }
}

pub fn apply_system_state_in(sys_root: &Path, cfg: &SystemConfig, dry_run: bool) -> bool {
    let ps_root = sys_root.join("class/power_supply");
    let pci_root = sys_root.join("bus/pci/devices");

    enable_gpu_runtime_pm_in(&pci_root, dry_run);
    if let Err(e) =
        apply_battery_thresholds_in(&ps_root, cfg.charge_start, cfg.charge_stop, dry_run)
    {
        eprintln!("[warn] thresholds: {e}");
    }

    let on_ac = is_on_ac_in(&ps_root);
    let target = if on_ac {
        &cfg.ac_profile
    } else {
        &cfg.bat_profile
    };
    if let Err(e) = apply_profile_in(sys_root, target, dry_run) {
        eprintln!("[warn] profile '{target}': {e}");
    }
    notify_tray_refresh();
    on_ac
}

pub fn run_daemon(once: bool) {
    let cfg = load_system_config_from(Path::new(SYSTEM_CONFIG_PATH));
    let sys_root = Path::new("/sys");
    let mut last_ac = apply_system_state_in(sys_root, &cfg, false);
    println!(
        "laptop-tune daemon: initial state {} -> profile '{}'",
        if last_ac { "AC" } else { "BAT" },
        if last_ac {
            &cfg.ac_profile
        } else {
            &cfg.bat_profile
        }
    );

    if once {
        return;
    }

    let Ok(mut child) = Command::new("udevadm")
        .args(["monitor", "--udev", "--subsystem-match=power_supply"])
        .stdout(Stdio::piped())
        .spawn()
    else {
        eprintln!("Failed to start udevadm monitor; exiting.");
        return;
    };

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            if line.contains("UDEV") && line.contains("power_supply") {
                let cfg = load_system_config_from(Path::new(SYSTEM_CONFIG_PATH));
                let on_ac = is_on_ac_in(&sys_root.join("class/power_supply"));
                if on_ac != last_ac {
                    last_ac = apply_system_state_in(sys_root, &cfg, false);
                    println!(
                        "laptop-tune daemon: switched to {} -> profile '{}'",
                        if last_ac { "AC" } else { "BAT" },
                        if last_ac {
                            &cfg.ac_profile
                        } else {
                            &cfg.bat_profile
                        }
                    );
                }
            }
        }
    }
    let _ = child.wait();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sysfs::test_utils::{write_mock, TempDir};

    #[test]
    fn test_apply_system_state_ac_vs_bat() {
        let tmp = TempDir::new("daemon");
        let sys = &tmp.0;

        write_mock(&sys.join("class/power_supply/AC/type"), "Mains\n");
        write_mock(&sys.join("class/power_supply/AC/online"), "1\n");
        write_mock(
            &sys.join("class/power_supply/BAT0/charge_control_start_threshold"),
            "0\n",
        );
        write_mock(
            &sys.join("class/power_supply/BAT0/charge_control_end_threshold"),
            "100\n",
        );
        write_mock(
            &sys.join("bus/pci/devices/0000:01:00.0/class"),
            "0x030200\n",
        );
        write_mock(&sys.join("bus/pci/devices/0000:01:00.0/vendor"), "0x10de\n");
        write_mock(
            &sys.join("bus/pci/devices/0000:01:00.0/power/control"),
            "on\n",
        );
        write_mock(
            &sys.join("firmware/acpi/platform_profile_choices"),
            "quiet balanced performance\n",
        );
        write_mock(&sys.join("firmware/acpi/platform_profile"), "balanced\n");

        let cfg = SystemConfig::default();
        assert!(apply_system_state_in(sys, &cfg, false));
        assert_eq!(
            read_trimmed(&sys.join("firmware/acpi/platform_profile")).as_deref(),
            Some("performance")
        );
        assert_eq!(
            read_trimmed(&sys.join("bus/pci/devices/0000:01:00.0/power/control")).as_deref(),
            Some("auto")
        );
        assert_eq!(
            read_trimmed(&sys.join("class/power_supply/BAT0/charge_control_start_threshold"))
                .as_deref(),
            Some("50")
        );
        assert_eq!(
            read_trimmed(&sys.join("class/power_supply/BAT0/charge_control_end_threshold"))
                .as_deref(),
            Some("80")
        );

        // Switch to battery
        write_mock(&sys.join("class/power_supply/AC/online"), "0\n");
        assert!(!apply_system_state_in(sys, &cfg, false));
        assert_eq!(
            read_trimmed(&sys.join("firmware/acpi/platform_profile")).as_deref(),
            Some("quiet")
        );
    }
}
