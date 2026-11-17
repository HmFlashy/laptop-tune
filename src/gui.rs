use std::env;
use std::process::Command;
use std::time::{Duration, Instant};

use eframe::egui;

use crate::config::{
    default_autostart_path, default_config_path, load_config_from, save_config_to,
    set_autostart_at, AppConfig,
};
use crate::gpu::vendor_name;
use crate::snapshot::{collect_snapshot, guess_active_profile, Snapshot};
use crate::tray::notify_tray_refresh;

const ACCENT_CYAN: egui::Color32 = egui::Color32::from_rgb(56, 189, 248);
const ACCENT_GREEN: egui::Color32 = egui::Color32::from_rgb(34, 197, 94);
const ACCENT_AMBER: egui::Color32 = egui::Color32::from_rgb(250, 204, 21);
const ACCENT_RED: egui::Color32 = egui::Color32::from_rgb(248, 113, 113);
const CARD_BG: egui::Color32 = egui::Color32::from_rgb(22, 30, 46);
const CARD_BORDER: egui::Color32 = egui::Color32::from_rgb(51, 65, 85);
const MUTED_TEXT: egui::Color32 = egui::Color32::from_rgb(148, 163, 184);

struct LaptopTuneApp {
    snapshot: Snapshot,
    last_refresh: Instant,
    status_msg: Option<(String, bool)>,
    styled: bool,
    config: AppConfig,
    autostart_enabled: bool,
    startup_applied: bool,
}

impl LaptopTuneApp {
    fn new() -> Self {
        let config = default_config_path()
            .map(|p| load_config_from(&p))
            .unwrap_or_default();
        let autostart_enabled = default_autostart_path().is_some_and(|p| p.exists());
        Self {
            snapshot: collect_snapshot(),
            last_refresh: Instant::now(),
            status_msg: None,
            styled: false,
            config,
            autostart_enabled,
            startup_applied: false,
        }
    }

    fn apply_theme(ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = egui::Color32::from_rgb(15, 23, 42);
        visuals.widgets.noninteractive.bg_fill = CARD_BG;
        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(30, 41, 59);
        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(51, 65, 85);
        visuals.widgets.active.bg_fill = egui::Color32::from_rgb(71, 85, 105);
        visuals.selection.bg_fill = ACCENT_CYAN.linear_multiply(0.35);
        ctx.set_visuals_of(egui::Theme::Dark, visuals);

        let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
        ctx.set_style_of(egui::Theme::Dark, style);
    }

    fn run_privileged(&mut self, args: &[&str], success_msg: String) -> bool {
        let exe = env::current_exe().unwrap_or_else(|_| "laptop-tune".into());
        let result = Command::new("pkexec").arg(&exe).args(args).output();

        match result {
            Ok(out) if out.status.success() => {
                self.status_msg = Some((success_msg, false));
                self.snapshot = collect_snapshot();
                notify_tray_refresh();
                true
            }
            Ok(out) => {
                let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
                let msg = if err.is_empty() {
                    "Action failed or cancelled.".into()
                } else {
                    err
                };
                self.status_msg = Some((msg, true));
                false
            }
            Err(e) => {
                self.status_msg = Some((format!("pkexec error: {e}"), true));
                false
            }
        }
    }

    fn save_config(&self) {
        if let Some(path) = default_config_path() {
            let _ = save_config_to(&path, &self.config);
        }
    }

    fn apply_profile(&mut self, name: &str) {
        self.run_privileged(&["profile", name], format!("Profile '{name}' applied."));
    }

    fn reset_defaults(&mut self) {
        self.run_privileged(&["reset"], "Defaults restored (balanced).".into());
    }

    fn active_profile_guess(&self) -> &'static str {
        guess_active_profile(&self.snapshot.profile)
    }
}

fn card_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(CARD_BG)
        .stroke(egui::Stroke::new(1.0_f32, CARD_BORDER))
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::same(12))
}

fn section_title(ui: &mut egui::Ui, title: &str) {
    ui.label(
        egui::RichText::new(title)
            .size(14.0)
            .strong()
            .color(ACCENT_CYAN),
    );
}

fn pill(ui: &mut egui::Ui, text: &str, color: egui::Color32) {
    egui::Frame::new()
        .fill(color.linear_multiply(0.16))
        .stroke(egui::Stroke::new(1.0_f32, color.linear_multiply(0.5)))
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(8, 3))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(text).size(11.5).strong().color(color));
        });
}

impl eframe::App for LaptopTuneApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if !self.styled {
            Self::apply_theme(&ctx);
            self.styled = true;
        }
        if !self.startup_applied {
            self.startup_applied = true;
            if self.config.apply_on_startup {
                if let Some(startup) = self.config.startup_profile.clone() {
                    self.apply_profile(&startup);
                }
            }
        }

        if self.last_refresh.elapsed() >= Duration::from_millis(1500) {
            self.snapshot = collect_snapshot();
            self.last_refresh = Instant::now();
        }
        ctx.request_repaint_after(Duration::from_millis(1500));

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(15, 23, 42))
                    .inner_margin(14.0),
            )
            .show(ui, |ui| {
                // Header
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("laptop-tune")
                            .size(20.0)
                            .strong()
                            .color(egui::Color32::WHITE),
                    );
                    pill(ui, &format!("v{}", env!("CARGO_PKG_VERSION")), ACCENT_CYAN);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Close").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.button("Refresh").clicked() {
                            self.snapshot = collect_snapshot();
                            self.last_refresh = Instant::now();
                        }
                    });
                });
                ui.add_space(10.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    // 1. Power Profile Card
                    card_frame().show(ui, |ui| {
                        ui.horizontal(|ui| {
                            section_title(ui, "Power Profile");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new("Reset Default").size(11.5),
                                            )
                                            .corner_radius(6.0),
                                        )
                                        .clicked()
                                    {
                                        self.reset_defaults();
                                    }
                                },
                            );
                        });
                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            pill(
                                ui,
                                &format!("Platform: {}", self.snapshot.profile.platform),
                                MUTED_TEXT,
                            );
                            pill(
                                ui,
                                &format!("EPP: {}", self.snapshot.profile.cpu_epp),
                                MUTED_TEXT,
                            );
                            let turbo_col = if self.snapshot.profile.turbo == "on" {
                                ACCENT_GREEN
                            } else {
                                MUTED_TEXT
                            };
                            pill(
                                ui,
                                &format!("Turbo: {}", self.snapshot.profile.turbo),
                                turbo_col,
                            );
                        });
                        ui.add_space(8.0);

                        let active = self.active_profile_guess();
                        let mut clicked: Option<&str> = None;
                        ui.horizontal(|ui| {
                            for (id, label) in [
                                ("eco", "Eco"),
                                ("balanced", "Balanced"),
                                ("performance", "Performance"),
                                ("gaming", "Gaming"),
                            ] {
                                let is_active = active == id;
                                let btn = if is_active {
                                    egui::Button::new(
                                        egui::RichText::new(label)
                                            .strong()
                                            .color(egui::Color32::from_rgb(15, 23, 42)),
                                    )
                                    .fill(ACCENT_CYAN)
                                    .corner_radius(6.0)
                                } else {
                                    egui::Button::new(label).corner_radius(6.0)
                                };
                                if ui.add(btn).clicked() {
                                    clicked = Some(id);
                                }
                            }
                        });
                        if let Some(p) = clicked {
                            self.apply_profile(p);
                        }
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            if ui
                                .checkbox(&mut self.autostart_enabled, "Start on login")
                                .changed()
                            {
                                if let Some(path) = default_autostart_path() {
                                    let exe =
                                        env::current_exe().unwrap_or_else(|_| "laptop-tune".into());
                                    if let Err(e) =
                                        set_autostart_at(&path, &exe, self.autostart_enabled)
                                    {
                                        self.status_msg = Some((e, true));
                                    }
                                }
                            }
                            if ui
                                .checkbox(&mut self.config.apply_on_startup, "Apply on startup:")
                                .changed()
                            {
                                if self.config.startup_profile.is_none() {
                                    self.config.startup_profile = Some("balanced".into());
                                }
                                self.save_config();
                            }
                            let selected = self
                                .config
                                .startup_profile
                                .get_or_insert_with(|| "balanced".into())
                                .clone();
                            let mut next_selected = selected.clone();
                            egui::ComboBox::from_id_salt("startup_profile_combo")
                                .selected_text(&selected)
                                .width(105.0)
                                .show_ui(ui, |ui| {
                                    for prof in ["eco", "balanced", "performance", "gaming"] {
                                        ui.selectable_value(&mut next_selected, prof.into(), prof);
                                    }
                                });
                            if next_selected != selected {
                                self.config.startup_profile = Some(next_selected);
                                self.save_config();
                            }
                        });
                        if let Some((msg, is_err)) = &self.status_msg {
                            ui.add_space(4.0);
                            let col = if *is_err { ACCENT_RED } else { ACCENT_GREEN };
                            ui.label(egui::RichText::new(msg).size(12.0).color(col));
                        }
                    });

                    ui.add_space(10.0);

                    // 2. Battery Card
                    card_frame().show(ui, |ui| {
                        section_title(ui, "Battery");
                        ui.add_space(4.0);
                        if self.snapshot.batteries.is_empty() {
                            ui.label(egui::RichText::new("No battery detected.").color(MUTED_TEXT));
                        } else {
                            for bat in &self.snapshot.batteries {
                                let pct_num = bat.capacity.parse::<f32>().unwrap_or(0.0);
                                let pct = (pct_num / 100.0).clamp(0.0, 1.0);
                                let bar_col = if pct_num >= 50.0 {
                                    ACCENT_GREEN
                                } else if pct_num >= 20.0 {
                                    ACCENT_AMBER
                                } else {
                                    ACCENT_RED
                                };
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{} — {} ({}%)",
                                            bat.name, bat.status, bat.capacity
                                        ))
                                        .strong(),
                                    );
                                    if let Some(w) = bat.power_w {
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                pill(ui, &format!("{w:.2} W"), ACCENT_CYAN);
                                            },
                                        );
                                    }
                                });
                                ui.add_space(4.0);
                                ui.add(
                                    egui::ProgressBar::new(pct)
                                        .fill(bar_col)
                                        .corner_radius(6.0)
                                        .show_percentage(),
                                );
                                if let (Some(s), Some(e)) =
                                    (&bat.start_threshold, &bat.stop_threshold)
                                {
                                    ui.add_space(4.0);
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "Charge thresholds: start {s}% / stop {e}%"
                                        ))
                                        .size(12.0)
                                        .color(MUTED_TEXT),
                                    );
                                }
                            }
                        }
                    });

                    ui.add_space(10.0);

                    // 3. GPUs & Sleep Card
                    card_frame().show(ui, |ui| {
                        ui.horizontal(|ui| {
                            section_title(ui, "GPUs & Sleep");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    pill(
                                        ui,
                                        &format!("sleep: {}", self.snapshot.mem_sleep),
                                        MUTED_TEXT,
                                    );
                                },
                            );
                        });
                        ui.add_space(4.0);
                        for gpu in &self.snapshot.gpus {
                            let (status_text, color) = if gpu.runtime_status == "suspended" {
                                ("D3 SUSPENDED", ACCENT_GREEN)
                            } else {
                                ("ACTIVE", ACCENT_AMBER)
                            };
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{} ({} · {})",
                                        vendor_name(&gpu.vendor),
                                        gpu.driver,
                                        gpu.pci_slot
                                    ))
                                    .size(13.0),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        pill(
                                            ui,
                                            &format!("{status_text} · {}", gpu.power_control),
                                            color,
                                        );
                                    },
                                );
                            });
                        }
                    });

                    ui.add_space(10.0);

                    // 4. Thermals Card
                    card_frame().show(ui, |ui| {
                        section_title(ui, "Thermals");
                        ui.add_space(4.0);
                        egui::Grid::new("thermals_grid")
                            .num_columns(2)
                            .spacing([24.0, 6.0])
                            .show(ui, |ui| {
                                for (zone, deg) in &self.snapshot.thermals {
                                    let color = if *deg >= 85 {
                                        ACCENT_RED
                                    } else if *deg >= 68 {
                                        ACCENT_AMBER
                                    } else {
                                        ACCENT_GREEN
                                    };
                                    ui.label(egui::RichText::new(zone).color(MUTED_TEXT));
                                    pill(ui, &format!("{deg}°C"), color);
                                    ui.end_row();
                                }
                            });
                    });
                });
            });
    }
}

pub fn run_gui() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([520.0, 620.0])
            .with_min_inner_size([400.0, 460.0]),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "laptop-tune",
        options,
        Box::new(|_cc| Ok(Box::new(LaptopTuneApp::new()))),
    ) {
        eprintln!("Failed to launch GUI: {e}");
        std::process::exit(1);
    }
}
