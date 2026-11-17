mod battery;
mod config;
mod gpu;
mod gui;
mod profile;
mod rules;
mod snapshot;
mod sysfs;
mod tray;

use std::env;
use std::path::Path;
use std::process;

use gpu::vendor_name;
use profile::{apply_profile_in, reset_defaults_in};
use rules::generate_rules_text;
use snapshot::collect_snapshot;

fn cmd_status() {
    let snap = collect_snapshot();
    println!("=== laptop-tune status ===");

    if snap.batteries.is_empty() {
        println!("\n[Battery] No BAT* device found.");
    } else {
        for info in &snap.batteries {
            println!("\n[Battery {}]", info.name);
            println!("  State     : {} ({}%)", info.status, info.capacity);
            if let Some(w) = info.power_w {
                println!("  Power     : {w:.2} W");
            }
            match (&info.start_threshold, &info.stop_threshold) {
                (Some(s), Some(e)) => println!("  Thresholds: start={s}% / stop={e}%"),
                _ => println!("  Thresholds: not exposed by kernel"),
            }
        }
    }

    println!("\n[GPUs]");
    if snap.gpus.is_empty() {
        println!("  No PCI GPU devices found.");
    } else {
        for gpu in &snap.gpus {
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
    println!("  mem_sleep : {}", snap.mem_sleep);

    println!("\n[Profile]");
    println!("  platform  : {}", snap.profile.platform);
    println!("  cpu_epp   : {}", snap.profile.cpu_epp);
    println!("  turbo     : {}", snap.profile.turbo);

    println!("\n[Thermals]");
    for (name, deg_c) in &snap.thermals {
        println!("  {name:<16}: {deg_c}°C");
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
    println!(
        "  gui [--tray]                Launch the native desktop GUI (optionally minimize to tray)"
    );
    println!("  profile <name> [--dry-run]  Apply profile (eco | balanced | performance | gaming)");
    println!("  reset [--dry-run]           Restore safe default (balanced) kernel settings");
    println!("  generate-rules              Detect hardware and print udev + TLP configs");
    println!("  help                        Show this help message");
}

fn cmd_reset(dry_run: bool) {
    println!(
        "Resetting to default (balanced) settings{}...",
        if dry_run { " (dry-run)" } else { "" }
    );
    if let Err(e) = reset_defaults_in(Path::new("/sys"), dry_run) {
        eprintln!("{e}");
        process::exit(1);
    }
    println!("Done.");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("status") | None => cmd_status(),
        Some("gui") => {
            if args.iter().any(|a| a == "--tray") {
                tray::run_tray_host();
            } else {
                gui::run_gui();
            }
        }
        Some("profile") => {
            let Some(name) = args.get(2) else {
                eprintln!("Missing profile name. Use: eco | balanced | performance | gaming");
                process::exit(1);
            };
            let dry_run = args.iter().any(|a| a == "--dry-run");
            cmd_profile(name, dry_run);
        }
        Some("reset") => {
            let dry_run = args.iter().any(|a| a == "--dry-run");
            cmd_reset(dry_run);
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
