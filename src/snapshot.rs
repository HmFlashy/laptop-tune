use std::fs;
use std::path::Path;

use crate::battery::{detect_batteries_in, read_battery_info, BatteryInfo};
use crate::gpu::{detect_gpus_in, GpuDevice};
use crate::sysfs::read_trimmed;

pub struct ProfileState {
    pub platform: String,
    pub cpu_epp: String,
    pub turbo: String,
    pub aspm: String,
}

pub fn parse_active_bracketed(raw: &str) -> String {
    raw.split_whitespace()
        .find(|tok| tok.starts_with('[') && tok.ends_with(']'))
        .map(|tok| tok[1..tok.len() - 1].to_string())
        .unwrap_or_else(|| raw.trim().to_string())
}

pub fn guess_active_profile(p: &ProfileState) -> &'static str {
    if p.cpu_epp == "power" || p.platform == "quiet" || p.platform == "cool" {
        "eco"
    } else if p.cpu_epp == "balance_power" {
        "balanced"
    } else if p.cpu_epp == "balance_performance" {
        "performance"
    } else if p.platform == "performance" || p.cpu_epp == "performance" || p.aspm == "performance" {
        "max"
    } else if p.platform == "balanced" {
        "balanced"
    } else {
        ""
    }
}

pub struct Snapshot {
    pub batteries: Vec<BatteryInfo>,
    pub gpus: Vec<GpuDevice>,
    pub mem_sleep: String,
    pub profile: ProfileState,
    pub thermals: Vec<(String, i64)>,
}

pub fn read_profile_state_in(sys_root: &Path) -> ProfileState {
    let platform = read_trimmed(&sys_root.join("firmware/acpi/platform_profile"))
        .unwrap_or_else(|| "unavailable".into());
    let cpu_epp = read_trimmed(
        &sys_root.join("devices/system/cpu/cpu0/cpufreq/energy_performance_preference"),
    )
    .unwrap_or_else(|| "unavailable".into());
    let turbo = if let Some(no_turbo) =
        read_trimmed(&sys_root.join("devices/system/cpu/intel_pstate/no_turbo"))
    {
        if no_turbo == "0" {
            "on".into()
        } else {
            "off".into()
        }
    } else if let Some(boost) = read_trimmed(&sys_root.join("devices/system/cpu/cpufreq/boost")) {
        if boost == "1" {
            "on".into()
        } else {
            "off".into()
        }
    } else {
        "unavailable".into()
    };

    let aspm = read_trimmed(&sys_root.join("module/pcie_aspm/parameters/policy"))
        .map(|raw| parse_active_bracketed(&raw))
        .unwrap_or_else(|| "unavailable".into());

    ProfileState {
        platform,
        cpu_epp,
        turbo,
        aspm,
    }
}

pub fn read_thermals_in(thermal_root: &Path) -> Vec<(String, i64)> {
    let mut thermals = Vec::new();
    if let Ok(entries) = fs::read_dir(thermal_root) {
        for entry in entries.flatten() {
            let p = entry.path();
            if !entry
                .file_name()
                .to_string_lossy()
                .starts_with("thermal_zone")
            {
                continue;
            }
            if let (Some(t), Some(temp_mc)) = (
                read_trimmed(&p.join("type")),
                read_trimmed(&p.join("temp")).and_then(|v| v.parse::<i64>().ok()),
            ) {
                thermals.push((t, temp_mc / 1000));
            }
        }
    }
    thermals.sort_by(|a, b| a.0.cmp(&b.0));
    thermals
}

pub fn collect_snapshot() -> Snapshot {
    let batteries = detect_batteries_in(Path::new("/sys/class/power_supply"))
        .into_iter()
        .map(|p| read_battery_info(&p))
        .collect();
    let gpus = detect_gpus_in(Path::new("/sys/bus/pci/devices"));
    let mem_sleep =
        read_trimmed(Path::new("/sys/power/mem_sleep")).unwrap_or_else(|| "unavailable".into());
    let profile = read_profile_state_in(Path::new("/sys"));
    let thermals = read_thermals_in(Path::new("/sys/class/thermal"));

    Snapshot {
        batteries,
        gpus,
        mem_sleep,
        profile,
        thermals,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sysfs::test_utils::{write_mock, TempDir};

    #[test]
    fn test_read_profile_state_and_thermals() {
        let tmp = TempDir::new("snapshot");
        let sys = &tmp.0;

        write_mock(&sys.join("firmware/acpi/platform_profile"), "quiet\n");
        write_mock(
            &sys.join("devices/system/cpu/cpu0/cpufreq/energy_performance_preference"),
            "power\n",
        );
        write_mock(&sys.join("devices/system/cpu/intel_pstate/no_turbo"), "1\n");
        write_mock(
            &sys.join("module/pcie_aspm/parameters/policy"),
            "default performance [powersupersave]\n",
        );

        let state = read_profile_state_in(sys);
        assert_eq!(state.platform, "quiet");
        assert_eq!(state.cpu_epp, "power");
        assert_eq!(state.turbo, "off");
        assert_eq!(state.aspm, "powersupersave");
        assert_eq!(guess_active_profile(&state), "eco");

        let max_prof = ProfileState {
            platform: "performance".into(),
            cpu_epp: "performance".into(),
            turbo: "on".into(),
            aspm: "performance".into(),
        };
        assert_eq!(guess_active_profile(&max_prof), "max");

        let perf = ProfileState {
            platform: "balanced".into(),
            cpu_epp: "balance_performance".into(),
            turbo: "on".into(),
            aspm: "default".into(),
        };
        assert_eq!(guess_active_profile(&perf), "performance");

        let bal = ProfileState {
            platform: "balanced".into(),
            cpu_epp: "balance_power".into(),
            turbo: "on".into(),
            aspm: "default".into(),
        };
        assert_eq!(guess_active_profile(&bal), "balanced");

        let therm = sys.join("class/thermal");
        write_mock(&therm.join("thermal_zone0/type"), "TCPU\n");
        write_mock(&therm.join("thermal_zone0/temp"), "68000\n");
        write_mock(&therm.join("thermal_zone1/type"), "TSKN\n");
        write_mock(&therm.join("thermal_zone1/temp"), "45000\n");

        let zones = read_thermals_in(&therm);
        assert_eq!(zones, vec![("TCPU".into(), 68), ("TSKN".into(), 45)]);
    }
}
