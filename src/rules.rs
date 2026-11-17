use std::path::Path;

use crate::battery::detect_batteries_in;
use crate::gpu::detect_gpus_in;

pub fn generate_rules_text(pci_root: &Path, power_supply_root: &Path) -> String {
    let gpus = detect_gpus_in(pci_root);
    let bats = detect_batteries_in(power_supply_root);
    let mut out = String::new();

    out.push_str("# === Generated udev rules (/etc/udev/rules.d/90-laptop-tune-gpu.rules) ===\n");
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
    if gpus.iter().any(|g| g.vendor == "0x10de") {
        out.push_str("ACTION==\"bind\", SUBSYSTEM==\"pci\", ATTR{vendor}==\"0x10de\", ATTR{class}==\"0x030000\", TEST==\"power/control\", ATTR{power/control}=\"auto\"\n");
        out.push_str("ACTION==\"bind\", SUBSYSTEM==\"pci\", ATTR{vendor}==\"0x10de\", ATTR{class}==\"0x030200\", TEST==\"power/control\", ATTR{power/control}=\"auto\"\n");
        out.push_str("ACTION==\"unbind\", SUBSYSTEM==\"pci\", ATTR{vendor}==\"0x10de\", ATTR{class}==\"0x030000\", TEST==\"power/control\", ATTR{power/control}=\"on\"\n");
        out.push_str("ACTION==\"unbind\", SUBSYSTEM==\"pci\", ATTR{vendor}==\"0x10de\", ATTR{class}==\"0x030200\", TEST==\"power/control\", ATTR{power/control}=\"on\"\n");
    }
    if gpus.len() > 1 && gpus.iter().any(|g| g.vendor == "0x1002") {
        out.push_str("ACTION==\"bind\", SUBSYSTEM==\"pci\", ATTR{vendor}==\"0x1002\", ATTR{class}==\"0x038000\", TEST==\"power/control\", ATTR{power/control}=\"auto\"\n");
    }

    out.push_str("\n# === Generated TLP config (/etc/tlp.d/90-laptop-tune.conf) ===\n");
    out.push_str("CPU_ENERGY_PERF_POLICY_ON_AC=performance\n");
    out.push_str("CPU_ENERGY_PERF_POLICY_ON_BAT=power\n");
    out.push_str("PLATFORM_PROFILE_ON_AC=performance\n");
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
        assert!(rules.contains("START_CHARGE_THRESH_BAT0=50"));
        assert!(rules.contains("STOP_CHARGE_THRESH_BAT0=80"));
    }
}
