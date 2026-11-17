use std::env;
use std::process::Command;
use std::time::{Duration, Instant};

use eframe::egui;

use crate::gpu::vendor_name;
use crate::snapshot::{collect_snapshot, Snapshot};
use crate::tray::{spawn_tray, TrayEvent, TrayHandle};

struct LaptopTuneApp {
    snapshot: Snapshot,
    last_refresh: Instant,
    status_msg: Option<String>,
    minimize_to_tray: bool,
    window_visible: bool,
    tray: Option<TrayHandle>,
    force_quit: bool,
}

impl LaptopTuneApp {
    fn new(minimize_to_tray: bool) -> Self {
        let tray = spawn_tray();
        Self {
            snapshot: collect_snapshot(),
            last_refresh: Instant::now(),
            status_msg: None,
            minimize_to_tray: minimize_to_tray && tray.is_some(),
            window_visible: true,
            tray,
            force_quit: false,
        }
    }

    fn run_privileged(&mut self, args: &[&str], success_msg: String) {
        let exe = env::current_exe().unwrap_or_else(|_| "laptop-tune".into());
        let result = Command::new("pkexec").arg(&exe).args(args).output();

        match result {
            Ok(out) if out.status.success() => {
                self.status_msg = Some(success_msg);
                self.snapshot = collect_snapshot();
            }
            Ok(out) => {
                let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
                self.status_msg = Some(if err.is_empty() {
                    "Action failed or cancelled.".into()
                } else {
                    err
                });
            }
            Err(e) => {
                self.status_msg = Some(format!("pkexec error: {e}"));
            }
        }
    }

    fn apply_profile(&mut self, name: &str) {
        self.run_privileged(&["profile", name], format!("Profile '{name}' applied."));
    }

    fn reset_defaults(&mut self) {
        self.run_privileged(&["reset"], "Defaults restored (balanced).".into());
    }

    fn handle_tray_events(&mut self, ctx: &egui::Context) {
        let mut events = Vec::new();
        if let Some(tray) = &self.tray {
            while let Ok(ev) = tray.rx.try_recv() {
                events.push(ev);
            }
        }
        for ev in events {
            match ev {
                TrayEvent::ToggleWindow => {
                    self.window_visible = !self.window_visible;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(self.window_visible));
                    if self.window_visible {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                    }
                }
                TrayEvent::ApplyProfile(name) => {
                    self.apply_profile(&name);
                }
                TrayEvent::Quit => {
                    self.force_quit = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }
}

impl eframe::App for LaptopTuneApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_tray_events(ctx);

        if ctx.input(|i| i.viewport().close_requested())
            && self.minimize_to_tray
            && !self.force_quit
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.window_visible = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }

        if self.last_refresh.elapsed() >= Duration::from_millis(1500) {
            self.snapshot = collect_snapshot();
            self.last_refresh = Instant::now();
        }
        ctx.request_repaint_after(Duration::from_millis(1500));

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("laptop-tune");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.tray.is_some() {
                        ui.checkbox(&mut self.minimize_to_tray, "Minimize to tray");
                    }
                });
            });
            ui.add_space(6.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                // Profiles
                ui.group(|ui| {
                    ui.label(egui::RichText::new("Power Profile").strong());
                    ui.label(format!(
                        "Platform: {}  |  EPP: {}  |  Turbo: {}",
                        self.snapshot.profile.platform,
                        self.snapshot.profile.cpu_epp,
                        self.snapshot.profile.turbo
                    ));
                    ui.add_space(4.0);
                    let mut clicked: Option<&str> = None;
                    ui.horizontal(|ui| {
                        for prof in ["eco", "balanced", "performance", "gaming"] {
                            if ui.button(prof).clicked() {
                                clicked = Some(prof);
                            }
                        }
                        if ui.button("Reset Default").clicked() {
                            clicked = Some("__reset__");
                        }
                    });
                    if let Some(p) = clicked {
                        if p == "__reset__" {
                            self.reset_defaults();
                        } else {
                            self.apply_profile(p);
                        }
                    }
                    if let Some(msg) = &self.status_msg {
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new(msg).small());
                    }
                });

                ui.add_space(6.0);

                // Batteries
                ui.group(|ui| {
                    ui.label(egui::RichText::new("Battery").strong());
                    if self.snapshot.batteries.is_empty() {
                        ui.label("No battery detected.");
                    } else {
                        for bat in &self.snapshot.batteries {
                            let pct = bat.capacity.parse::<f32>().unwrap_or(0.0) / 100.0;
                            let power_str = bat
                                .power_w
                                .map(|w| format!(" | {w:.2} W"))
                                .unwrap_or_default();
                            ui.label(format!(
                                "{}: {} ({}%){}",
                                bat.name, bat.status, bat.capacity, power_str
                            ));
                            ui.add(egui::ProgressBar::new(pct).show_percentage());
                            if let (Some(s), Some(e)) = (&bat.start_threshold, &bat.stop_threshold)
                            {
                                ui.label(format!("Thresholds: start {s}% / stop {e}%"));
                            }
                        }
                    }
                });

                ui.add_space(6.0);

                // GPUs
                ui.group(|ui| {
                    ui.label(egui::RichText::new("GPUs & Runtime PM").strong());
                    for gpu in &self.snapshot.gpus {
                        let color = if gpu.runtime_status == "suspended" {
                            egui::Color32::LIGHT_GREEN
                        } else {
                            egui::Color32::KHAKI
                        };
                        ui.horizontal(|ui| {
                            ui.label(format!(
                                "{} ({} / {})",
                                gpu.pci_slot,
                                vendor_name(&gpu.vendor),
                                gpu.driver
                            ));
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} (control={})",
                                    gpu.runtime_status, gpu.power_control
                                ))
                                .color(color),
                            );
                        });
                    }
                    ui.label(format!("Sleep mode: {}", self.snapshot.mem_sleep));
                });

                ui.add_space(6.0);

                // Thermals
                ui.group(|ui| {
                    ui.label(egui::RichText::new("Thermals").strong());
                    egui::Grid::new("thermals_grid")
                        .num_columns(2)
                        .spacing([20.0, 4.0])
                        .show(ui, |ui| {
                            for (zone, deg) in &self.snapshot.thermals {
                                let color = if *deg >= 85 {
                                    egui::Color32::LIGHT_RED
                                } else if *deg >= 68 {
                                    egui::Color32::KHAKI
                                } else {
                                    egui::Color32::LIGHT_GREEN
                                };
                                ui.label(zone);
                                ui.label(egui::RichText::new(format!("{deg}°C")).color(color));
                                ui.end_row();
                            }
                        });
                });
            });
        });
    }
}

pub fn run_gui(minimize_to_tray: bool) {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([480.0, 560.0])
            .with_min_inner_size([380.0, 420.0]),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "laptop-tune",
        options,
        Box::new(move |_cc| Ok(Box::new(LaptopTuneApp::new(minimize_to_tray)))),
    ) {
        eprintln!("Failed to launch GUI: {e}");
        std::process::exit(1);
    }
}
