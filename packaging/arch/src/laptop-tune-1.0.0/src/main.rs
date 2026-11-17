mod battery;
mod gpu;
mod profile;
mod rules;
mod sysfs;

use std::env;
use std::fs;
use std::path::Path;
use std::process;

use battery::{detect_batteries_in, read_battery_info};
use gpu::{detect_gpus_in, vendor_name};
use profile::apply_profile_in;
use rules::generate_rules_text;
use sysfs::read_trimmed;

fn cmd_status() {
    println!("=== laptop-tune status ===");

    let bats = detect_batteries_in(Path::new("/sys/class/power_supply"));
    if bats.is_empty() {
        println!("\n[Battery] No BAT* device found.");
    } else {
        for bat in &bats {
            let info = read_battery_info(bat);
            println!("\n[Battery {}]", info.name);
            println!("  State     : {} ({}%)", info.status, info.capacity);
            if let Some(w) = info.power_w {
                println!("  Power     : {w:.2} W");
            }
            match (info.start_threshold, info.stop_threshold) {
                (Some(s), Some(e)) => println!("  Thresholds: start={s}% / stop={e}%"),
                _ => println!("  Thresholds: not exposed by kernel"),
            }
        }
    }

    let gpus = detect_gpus_in(Path::new("/sys/bus/pci/devices"));
    println!("\n[GPUs]");
    if gpus.is_empty() {
        println!("  No PCI GPU devices found.");
    } else {
        for gpu in &gpus {
            let vname = vendor_name(&gpu.vendor);
            let ok_hint = if gpu.runtime_status == "suspended" && gpu.power_control == "auto" {
                " [OK: D3 suspended]"
            } else if gpu.vendor == "0x10de" && gpu.power_control != "auto" {
                " [WARN: power/control != auto]"
            } else {
                ""
            };
            println!(
                "  {} ({} / {} / {}): {} (control={}){}",
                gpu.pci_slot,
                vname,
                gpu.driver,
                gpu.class,
                gpu.runtime_status,
                gpu.power_control,
                ok_hint
            );
        }
    }

    println!("\n[Sleep]");
    if let Some(mem_sleep) = read_trimmed(Path::new("/sys/power/mem_sleep")) {
        println!("  mem_sleep : {mem_sleep}");
    } else {
        println!("  mem_sleep : unavailable");
    }

    println!("\n[Profile]");
    let plat = read_trimmed(Path::new("/sys/firmware/acpi/platform_profile"))
        .unwrap_or_else(|| "unavailable".into());
    let epp = read_trimmed(Path::new(
        "/sys/devices/system/cpu/cpu0/cpufreq/energy_performance_preference",
    ))
    .unwrap_or_else(|| "unavailable".into());
    let turbo = if let Some(no_turbo) =
        read_trimmed(Path::new("/sys/devices/system/cpu/intel_pstate/no_turbo"))
    {
        if no_turbo == "0" {
            "on"
        } else {
            "off"
        }
    } else if let Some(boost) = read_trimmed(Path::new("/sys/devices/system/cpu/cpufreq/boost")) {
        if boost == "1" {
            "on"
        } else {
            "off"
        }
    } else {
        "unavailable"
    };
    println!("  platform  : {plat}");
    println!("  cpu_epp   : {epp}");
    println!("  turbo     : {turbo}");

    println!("\n[Thermals]");
    if let Ok(entries) = fs::read_dir("/sys/class/thermal") {
        let mut zones: Vec<(String, i64)> = Vec::new();
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
                zones.push((t, temp_mc / 1000));
            }
        }
        zones.sort_by(|a, b| a.0.cmp(&b.0));
        for (name, deg_c) in zones {
            println!("  {name:<16}: {deg_c}°C");
        }
    }
}

fn cmd_generate_rules() {
    print!(
        "{}",
        generate_rules_text(
            Path::new("/sys/bus/pci/devices"),
            Path::new("/sys/class/power_supply")
        )
    );
}

fn cmd_profile(name: &str, dry_run: bool) {
    println!(
        "Applying profile '{}'{}...",
        name,
        if dry_run { " (dry-run)" } else { "" }
    );
    if let Err(e) = apply_profile_in(Path::new("/sys"), name, dry_run) {
        eprintln!("{e}");
        process::exit(1);
    }
    println!("Done.");
}

fn print_help() {
    println!("laptop-tune v{}", env!("CARGO_PKG_VERSION"));
    println!("Usage: laptop-tune <command>\n");
    println!("Commands:");
    println!("  status                      Show battery, GPU PM, profile, sleep, and thermals");
    println!("  profile <name> [--dry-run]  Apply profile (eco | balanced | performance | gaming)");
    println!("  generate-rules              Detect hardware and print udev + TLP configs");
    println!("  help                        Show this help message");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("status") | None => cmd_status(),
        Some("profile") => {
            let Some(name) = args.get(2) else {
                eprintln!("Missing profile name. Use: eco | balanced | performance | gaming");
                process::exit(1);
            };
            let dry_run = args.iter().any(|a| a == "--dry-run");
            cmd_profile(name, dry_run);
        }
        Some("generate-rules") => cmd_generate_rules(),
        Some("help") | Some("--help") | Some("-h") => print_help(),
        Some(other) => {
            eprintln!("Unknown command: {other}");
            print_help();
            process::exit(1);
        }
    }
}
