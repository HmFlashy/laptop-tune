use std::fs;
use std::path::Path;

use crate::sysfs::{read_trimmed, write_value};

pub struct ProfileSpec {
    pub name: &'static str,
    pub platform_candidates: &'static [&'static str],
    pub epp: &'static str,
    pub turbo: bool,
    pub aspm: &'static str,
}

pub const PROFILES: &[ProfileSpec] = &[
    ProfileSpec {
        name: "eco",
        platform_candidates: &["quiet", "low-power", "cool"],
        epp: "power",
        turbo: false,
        aspm: "powersupersave",
    },
    ProfileSpec {
        name: "balanced",
        platform_candidates: &["balanced"],
        epp: "balance_performance",
        turbo: true,
        aspm: "default",
    },
    ProfileSpec {
        name: "performance",
        platform_candidates: &["performance"],
        epp: "performance",
        turbo: true,
        aspm: "default",
    },
    ProfileSpec {
        name: "max",
        platform_candidates: &["performance"],
        epp: "performance",
        turbo: true,
        aspm: "performance",
    },
];

pub fn choose_platform_profile<'a>(spec: &ProfileSpec, choices: &str) -> Option<&'a str> {
    spec.platform_candidates
        .iter()
        .copied()
        .find(|c| choices.split_whitespace().any(|avail| avail == *c))
}

pub fn reset_defaults_in(sys_root: &Path, dry_run: bool) -> Result<(), String> {
    apply_profile_in(sys_root, "balanced", dry_run)
}

pub fn apply_profile_in(sys_root: &Path, name: &str, dry_run: bool) -> Result<(), String> {
    let spec = PROFILES
        .iter()
        .find(|p| p.name == name)
        .ok_or_else(|| format!("Unknown profile: {name}"))?;

    let mut errors = 0;

    let plat_path = sys_root.join("firmware/acpi/platform_profile");
    if plat_path.exists() {
        let choices = read_trimmed(&sys_root.join("firmware/acpi/platform_profile_choices"))
            .unwrap_or_default();
        if let Some(chosen) = choose_platform_profile(spec, &choices) {
            if let Err(e) = write_value(&plat_path, chosen, dry_run) {
                eprintln!("  [warn] {e}");
                errors += 1;
            }
        }
    }

    let gov_candidates: &[&str] = match spec.name {
        "eco" => &["powersave", "conservative", "schedutil"],
        "balanced" => &["schedutil", "ondemand", "powersave"],
        _ => &["performance", "schedutil"],
    };

    let cpu_root = sys_root.join("devices/system/cpu");
    if let Ok(entries) = fs::read_dir(&cpu_root) {
        for entry in entries.flatten() {
            let fname = entry.file_name();
            let fname = fname.to_string_lossy();
            if !fname.starts_with("cpu") || !fname[3..].chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            let cpufreq = entry.path().join("cpufreq");
            let epp_path = cpufreq.join("energy_performance_preference");
            if epp_path.exists() {
                if let Err(e) = write_value(&epp_path, spec.epp, dry_run) {
                    eprintln!("  [warn] {e}");
                    errors += 1;
                    break;
                }
            } else {
                let gov_path = cpufreq.join("scaling_governor");
                let avail_path = cpufreq.join("scaling_available_governors");
                if gov_path.exists() {
                    let avail = read_trimmed(&avail_path).unwrap_or_default();
                    if let Some(gov) = gov_candidates
                        .iter()
                        .copied()
                        .find(|g| avail.split_whitespace().any(|a| a == *g))
                    {
                        if let Err(e) = write_value(&gov_path, gov, dry_run) {
                            eprintln!("  [warn] {e}");
                            errors += 1;
                            break;
                        }
                    }
                }
            }
        }
    }

    let intel_turbo = cpu_root.join("intel_pstate/no_turbo");
    let generic_boost = cpu_root.join("cpufreq/boost");
    if intel_turbo.exists() {
        let val = if spec.turbo { "0" } else { "1" };
        if let Err(e) = write_value(&intel_turbo, val, dry_run) {
            eprintln!("  [warn] {e}");
            errors += 1;
        }
    } else if generic_boost.exists() {
        let val = if spec.turbo { "1" } else { "0" };
        if let Err(e) = write_value(&generic_boost, val, dry_run) {
            eprintln!("  [warn] {e}");
            errors += 1;
        }
    }

    let aspm_path = sys_root.join("module/pcie_aspm/parameters/policy");
    if aspm_path.exists() {
        if let Err(e) = write_value(&aspm_path, spec.aspm, dry_run) {
            eprintln!("  [warn] {e}");
            errors += 1;
        }
    }

    if errors > 0 {
        Err("Some sysfs writes failed (try running with sudo).".into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sysfs::test_utils::{write_mock, TempDir};

    #[test]
    fn test_choose_platform_profile_prefers_first_match() {
        let eco = &PROFILES[0];
        assert_eq!(
            choose_platform_profile(eco, "cool quiet balanced performance"),
            Some("quiet")
        );
        assert_eq!(
            choose_platform_profile(eco, "cool balanced performance"),
            Some("cool")
        );
        assert_eq!(choose_platform_profile(eco, "balanced"), None);
    }

    #[test]
    fn test_apply_profile_in_mock_sysfs_and_dry_run() {
        let tmp = TempDir::new("profile");
        let sys = &tmp.0;

        write_mock(
            &sys.join("firmware/acpi/platform_profile_choices"),
            "cool quiet balanced performance\n",
        );
        write_mock(&sys.join("firmware/acpi/platform_profile"), "balanced\n");
        write_mock(
            &sys.join("devices/system/cpu/cpu0/cpufreq/energy_performance_preference"),
            "balance_performance\n",
        );
        write_mock(
            &sys.join("devices/system/cpu/cpu1/cpufreq/energy_performance_preference"),
            "balance_performance\n",
        );
        write_mock(&sys.join("devices/system/cpu/intel_pstate/no_turbo"), "0\n");
        write_mock(&sys.join("module/pcie_aspm/parameters/policy"), "default\n");

        apply_profile_in(sys, "eco", true).unwrap();
        assert_eq!(
            read_trimmed(&sys.join("firmware/acpi/platform_profile")).as_deref(),
            Some("balanced")
        );

        apply_profile_in(sys, "eco", false).unwrap();
        assert_eq!(
            read_trimmed(&sys.join("firmware/acpi/platform_profile")).as_deref(),
            Some("quiet")
        );
        assert_eq!(
            read_trimmed(
                &sys.join("devices/system/cpu/cpu0/cpufreq/energy_performance_preference")
            )
            .as_deref(),
            Some("power")
        );
        assert_eq!(
            read_trimmed(
                &sys.join("devices/system/cpu/cpu1/cpufreq/energy_performance_preference")
            )
            .as_deref(),
            Some("power")
        );
        assert_eq!(
            read_trimmed(&sys.join("devices/system/cpu/intel_pstate/no_turbo")).as_deref(),
            Some("1")
        );
        assert_eq!(
            read_trimmed(&sys.join("module/pcie_aspm/parameters/policy")).as_deref(),
            Some("powersupersave")
        );

        apply_profile_in(sys, "max", false).unwrap();
        assert_eq!(
            read_trimmed(&sys.join("firmware/acpi/platform_profile")).as_deref(),
            Some("performance")
        );
        assert_eq!(
            read_trimmed(&sys.join("devices/system/cpu/intel_pstate/no_turbo")).as_deref(),
            Some("0")
        );
        assert_eq!(
            read_trimmed(&sys.join("module/pcie_aspm/parameters/policy")).as_deref(),
            Some("performance")
        );
    }

    #[test]
    fn test_apply_profile_amd_or_non_epp_cpu_fallback() {
        let tmp = TempDir::new("amd_cpu");
        let sys = &tmp.0;

        write_mock(
            &sys.join("devices/system/cpu/cpu0/cpufreq/scaling_available_governors"),
            "conservative ondemand powersave performance schedutil\n",
        );
        write_mock(
            &sys.join("devices/system/cpu/cpu0/cpufreq/scaling_governor"),
            "schedutil\n",
        );
        write_mock(&sys.join("devices/system/cpu/cpufreq/boost"), "1\n");

        apply_profile_in(sys, "eco", false).unwrap();
        assert_eq!(
            read_trimmed(&sys.join("devices/system/cpu/cpu0/cpufreq/scaling_governor")).as_deref(),
            Some("powersave")
        );
        assert_eq!(
            read_trimmed(&sys.join("devices/system/cpu/cpufreq/boost")).as_deref(),
            Some("0")
        );

        apply_profile_in(sys, "performance", false).unwrap();
        assert_eq!(
            read_trimmed(&sys.join("devices/system/cpu/cpu0/cpufreq/scaling_governor")).as_deref(),
            Some("performance")
        );
        assert_eq!(
            read_trimmed(&sys.join("devices/system/cpu/cpufreq/boost")).as_deref(),
            Some("1")
        );
    }

    #[test]
    fn test_reset_defaults_restores_balanced() {
        let tmp = TempDir::new("reset");
        let sys = &tmp.0;

        write_mock(
            &sys.join("firmware/acpi/platform_profile_choices"),
            "cool quiet balanced performance\n",
        );
        write_mock(&sys.join("firmware/acpi/platform_profile"), "performance\n");
        write_mock(
            &sys.join("devices/system/cpu/cpu0/cpufreq/energy_performance_preference"),
            "power\n",
        );
        write_mock(&sys.join("devices/system/cpu/intel_pstate/no_turbo"), "1\n");
        write_mock(
            &sys.join("module/pcie_aspm/parameters/policy"),
            "powersupersave\n",
        );

        reset_defaults_in(sys, false).unwrap();

        assert_eq!(
            read_trimmed(&sys.join("firmware/acpi/platform_profile")).as_deref(),
            Some("balanced")
        );
        assert_eq!(
            read_trimmed(
                &sys.join("devices/system/cpu/cpu0/cpufreq/energy_performance_preference")
            )
            .as_deref(),
            Some("balance_performance")
        );
        assert_eq!(
            read_trimmed(&sys.join("devices/system/cpu/intel_pstate/no_turbo")).as_deref(),
            Some("0")
        );
        assert_eq!(
            read_trimmed(&sys.join("module/pcie_aspm/parameters/policy")).as_deref(),
            Some("default")
        );
    }
}
