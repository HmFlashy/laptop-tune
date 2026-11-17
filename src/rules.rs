use std::fs;
use std::path::Path;

use crate::battery::detect_batteries_in;
use crate::daemon::enable_gpu_runtime_pm_in;
use crate::gpu::detect_gpus_in;

pub fn generate_udev_rules(pci_root: &Path) -> String {
    let gpus = detect_gpus_in(pci_root);
    let mut out = String::new();

    if let Some(igpu) = gpus
        .iter()
        .find(|g| g.vendor == "0x8086" || g.vendor == "0x1002")
    {
        let link = if igpu.vendor == "0x8086" {
            "dri/intel-igpu"
        } else {
            "dri/amd-igpu"
        };
        out.push_str(&format!(
            "KERNEL==\"card[0-9]*\", SUBSYSTEM==\"drm\", SUBSYSTEMS==\"pci\", KERNELS==\"{}\", SYMLINK+=\"{}\"\n",
            igpu.pci_slot, link
        ));
    }

    // Put any discrete GPU (2nd+ GPU: NVIDIA, AMD Radeon, or Intel Arc) and its companion PCI functions (.1 audio, .2/.3 USB-C) into runtime PM auto
    for dgpu in gpus.iter().skip(1) {
        let prefix = dgpu
            .pci_slot
            .rsplit_once('.')
            .map(|(p, _)| p)
            .unwrap_or(&dgpu.pci_slot);
        out.push_str(&format!(
            "ACTION==\"add|bind\", SUBSYSTEM==\"pci\", KERNELS==\"{prefix}.*\", TEST==\"power/control\", ATTR{{power/control}}=\"auto\"\n"
        ));
    }

    if gpus.iter().any(|g| g.vendor == "0x10de") {
        out.push_str("ACTION==\"add|bind\", SUBSYSTEM==\"pci\", ATTR{vendor}==\"0x10de\", ATTR{class}==\"0x030000\", TEST==\"power/control\", ATTR{power/control}=\"auto\"\n");
        out.push_str("ACTION==\"add|bind\", SUBSYSTEM==\"pci\", ATTR{vendor}==\"0x10de\", ATTR{class}==\"0x030200\", TEST==\"power/control\", ATTR{power/control}=\"auto\"\n");
        out.push_str("ACTION==\"add|bind\", SUBSYSTEM==\"pci\", ATTR{vendor}==\"0x10de\", ATTR{class}==\"0x040300\", TEST==\"power/control\", ATTR{power/control}=\"auto\"\n");
        out.push_str("ACTION==\"unbind\", SUBSYSTEM==\"pci\", ATTR{vendor}==\"0x10de\", ATTR{class}==\"0x030000\", TEST==\"power/control\", ATTR{power/control}=\"on\"\n");
        out.push_str("ACTION==\"unbind\", SUBSYSTEM==\"pci\", ATTR{vendor}==\"0x10de\", ATTR{class}==\"0x030200\", TEST==\"power/control\", ATTR{power/control}=\"on\"\n");
    }
    if gpus.len() > 1 && gpus.iter().any(|g| g.vendor == "0x1002") {
        out.push_str("ACTION==\"add|bind\", SUBSYSTEM==\"pci\", ATTR{vendor}==\"0x1002\", ATTR{class}==\"0x038000\", TEST==\"power/control\", ATTR{power/control}=\"auto\"\n");
    }
    out
}

pub fn generate_environment_d(pci_root: &Path) -> Option<String> {
    let gpus = detect_gpus_in(pci_root);
    if gpus.len() < 2 {
        return None;
    }
    let igpu = gpus
        .iter()
        .find(|g| g.vendor == "0x8086" || g.vendor == "0x1002")?;
    let dev = if igpu.vendor == "0x8086" {
        "/dev/dri/intel-igpu"
    } else {
        "/dev/dri/amd-igpu"
    };
    Some(format!(
        "# Pin Wayland compositors to the integrated GPU so the dedicated GPU stays in D3cold sleep\nKWIN_DRM_DEVICES={dev}\n"
    ))
}

pub fn generate_rules_text(pci_root: &Path, power_supply_root: &Path) -> String {
    let bats = detect_batteries_in(power_supply_root);
    let mut out = String::new();

    out.push_str("# === Generated udev rules (/etc/udev/rules.d/90-laptop-tune-gpu.rules) ===\n");
    out.push_str(&generate_udev_rules(pci_root));

    if let Some(env_d) = generate_environment_d(pci_root) {
        out.push_str(
            "\n# === Generated environment.d (/etc/environment.d/90-laptop-tune-igpu.conf) ===\n",
        );
        out.push_str(&env_d);
    }

    out.push_str("\n# === Generated TLP config (/etc/tlp.d/90-laptop-tune.conf) ===\n");
    out.push_str("CPU_ENERGY_PERF_POLICY_ON_AC=balance_performance\n");
    out.push_str("CPU_ENERGY_PERF_POLICY_ON_BAT=power\n");
    out.push_str("PLATFORM_PROFILE_ON_AC=balanced\n");
    out.push_str("PLATFORM_PROFILE_ON_BAT=quiet\n");
    out.push_str("CPU_BOOST_ON_AC=1\n");
    out.push_str("CPU_BOOST_ON_BAT=0\n");
    out.push_str("PCIE_ASPM_ON_AC=default\n");
    out.push_str("PCIE_ASPM_ON_BAT=powersupersave\n");

    let threshold_bats: Vec<_> = bats
        .into_iter()
        .filter(|b| b.join("charge_control_end_threshold").exists())
        .collect();
    if !threshold_bats.is_empty() {
        out.push_str("\n# === Battery thresholds (/etc/tlp.d/91-laptop-tune-battery.conf) ===\n");
        for bat in threshold_bats {
            let name = bat.file_name().unwrap_or_default().to_string_lossy();
            out.push_str(&format!("START_CHARGE_THRESH_{name}=50\n"));
            out.push_str(&format!("STOP_CHARGE_THRESH_{name}=80\n"));
        }
    }

    out
}

pub fn apply_gpu_setup_in(sys_root: &Path, etc_root: &Path, dry_run: bool) -> Result<(), String> {
    let pci_root = sys_root.join("bus/pci/devices");
    let udev_rules = generate_udev_rules(&pci_root);
    let udev_dir = etc_root.join("udev/rules.d");
    let udev_file = udev_dir.join("90-laptop-tune-gpu.rules");
    if dry_run {
        println!("  [dry-run] write {}", udev_file.display());
    } else {
        fs::create_dir_all(&udev_dir).map_err(|e| format!("{}: {e}", udev_dir.display()))?;
        fs::write(&udev_file, udev_rules).map_err(|e| format!("{}: {e}", udev_file.display()))?;
    }

    let modprobe_dir = etc_root.join("modprobe.d");
    let modprobe_file = modprobe_dir.join("90-laptop-tune-gpu.conf");
    let modprobe_content = "# Enable deep D3cold runtime power management for hybrid dedicated GPUs\noptions nvidia NVreg_DynamicPowerManagement=0x02\noptions amdgpu runpm=1\n";
    if dry_run {
        println!("  [dry-run] write {}", modprobe_file.display());
    } else {
        fs::create_dir_all(&modprobe_dir)
            .map_err(|e| format!("{}: {e}", modprobe_dir.display()))?;
        fs::write(&modprobe_file, modprobe_content)
            .map_err(|e| format!("{}: {e}", modprobe_file.display()))?;
    }

    if let Some(env_d) = generate_environment_d(&pci_root) {
        let env_dir = etc_root.join("environment.d");
        let env_file = env_dir.join("90-laptop-tune-igpu.conf");
        if dry_run {
            println!("  [dry-run] write {}", env_file.display());
        } else {
            fs::create_dir_all(&env_dir).map_err(|e| format!("{}: {e}", env_dir.display()))?;
            fs::write(&env_file, env_d).map_err(|e| format!("{}: {e}", env_file.display()))?;
        }
    }

    enable_gpu_runtime_pm_in(&pci_root, dry_run);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sysfs::test_utils::{write_mock, TempDir};

    #[test]
    fn test_generate_rules_text() {
        let tmp = TempDir::new("rules");
        let pci = tmp.0.join("pci");
        let ps = tmp.0.join("power_supply");

        write_mock(&pci.join("0000:00:02.0/class"), "0x030000\n");
        write_mock(&pci.join("0000:00:02.0/vendor"), "0x8086\n");
        write_mock(&pci.join("0000:01:00.0/class"), "0x030200\n");
        write_mock(&pci.join("0000:01:00.0/vendor"), "0x10de\n");
        write_mock(&ps.join("BAT0/charge_control_end_threshold"), "80\n");

        let rules = generate_rules_text(&pci, &ps);
        assert!(rules.contains("KERNELS==\"0000:00:02.0\""));
        assert!(rules.contains("ATTR{vendor}==\"0x10de\""));
        assert!(rules.contains("KWIN_DRM_DEVICES=/dev/dri/intel-igpu"));
        assert!(rules.contains("START_CHARGE_THRESH_BAT0=50"));
        assert!(rules.contains("STOP_CHARGE_THRESH_BAT0=80"));
    }

    #[test]
    fn test_apply_gpu_setup_in_writes_configs_and_sets_auto_pm() {
        let tmp = TempDir::new("gpu_setup");
        let sys = tmp.0.join("sys");
        let etc = tmp.0.join("etc");
        let pci = sys.join("bus/pci/devices");

        write_mock(&pci.join("0000:00:02.0/class"), "0x030000\n");
        write_mock(&pci.join("0000:00:02.0/vendor"), "0x1002\n");
        write_mock(&pci.join("0000:01:00.0/class"), "0x030200\n");
        write_mock(&pci.join("0000:01:00.0/vendor"), "0x10de\n");
        write_mock(&pci.join("0000:01:00.0/power/control"), "on\n");
        write_mock(&pci.join("0000:01:00.1/power/control"), "on\n");

        apply_gpu_setup_in(&sys, &etc, false).unwrap();

        assert!(etc.join("udev/rules.d/90-laptop-tune-gpu.rules").exists());
        assert!(etc.join("modprobe.d/90-laptop-tune-gpu.conf").exists());
        let env_d = fs::read_to_string(etc.join("environment.d/90-laptop-tune-igpu.conf")).unwrap();
        assert!(env_d.contains("KWIN_DRM_DEVICES=/dev/dri/amd-igpu"));
        assert_eq!(
            fs::read_to_string(pci.join("0000:01:00.0/power/control"))
                .unwrap()
                .trim(),
            "auto"
        );
        assert_eq!(
            fs::read_to_string(pci.join("0000:01:00.1/power/control"))
                .unwrap()
                .trim(),
            "auto"
        );
    }
}
