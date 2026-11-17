use std::env;
use std::process::Command;
use std::time::{Duration, Instant};

use eframe::egui;

use crate::config::{
    default_autostart_path, default_config_path, load_config_from, save_config_to,
    set_autostart_at, AppConfig,
};
use crate::gpu::vendor_name;
use crate::i18n::tr;
use crate::snapshot::{collect_snapshot, guess_active_profile, Snapshot};
use crate::tray::notify_tray_refresh;

const ACCENT_GREEN: egui::Color32 = egui::Color32::from_rgb(39, 174, 96);
const ACCENT_AMBER: egui::Color32 = egui::Color32::from_rgb(230, 126, 34);
const ACCENT_RED: egui::Color32 = egui::Color32::from_rgb(220, 68, 55);

struct LaptopTuneApp {
    snapshot: Snapshot,
    last_refresh: Instant,
    status_msg: Option<(String, bool, Instant)>,
    config: AppConfig,
    autostart_enabled: bool,
    thresh_start: u8,
    thresh_stop: u8,
}

impl LaptopTuneApp {
    fn new() -> Self {
        let config = default_config_path()
            .map(|p| load_config_from(&p))
            .unwrap_or_default();
        let autostart_enabled = default_autostart_path().is_some_and(|p| p.exists());
        let snapshot = collect_snapshot();
        let thresh_start = snapshot
            .batteries
            .first()
            .and_then(|b| b.start_threshold.as_deref())
            .and_then(|s| s.parse::<u8>().ok())
            .unwrap_or(50);
        let thresh_stop = snapshot
            .batteries
            .first()
            .and_then(|b| b.stop_threshold.as_deref())
            .and_then(|s| s.parse::<u8>().ok())
            .unwrap_or(80);
        Self {
            snapshot,
            last_refresh: Instant::now(),
            status_msg: None,
            config,
            autostart_enabled,
            thresh_start,
            thresh_stop,
        }
    }

    fn configure_spacing(ctx: &egui::Context) {
        for theme in [egui::Theme::Dark, egui::Theme::Light] {
            let mut style = (*ctx.style_of(theme)).clone();
            style.spacing.item_spacing = egui::vec2(8.0, 8.0);
            style.spacing.button_padding = egui::vec2(12.0, 6.0);
            style.spacing.interact_size.y = 26.0;
            ctx.set_style_of(theme, style);
        }
    }

    fn run_privileged(&mut self, args: &[&str], success_msg: String) -> bool {
        let exe = env::current_exe().unwrap_or_else(|_| "laptop-tune".into());
        let result = Command::new("pkexec").arg(&exe).args(args).output();

        match result {
            Ok(out) if out.status.success() => {
                self.status_msg = Some((success_msg, false, Instant::now()));
                self.snapshot = collect_snapshot();
                notify_tray_refresh();
                true
            }
            Ok(out) => {
                let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
                let msg = if err.is_empty() {
                    tr().action_failed.into()
                } else {
                    err
                };
                self.status_msg = Some((msg, true, Instant::now()));
                false
            }
            Err(e) => {
                self.status_msg = Some((format!("pkexec error: {e}"), true, Instant::now()));
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
        let t = tr();
        self.run_privileged(
            &["profile", name],
            format!("{} {}", t.profile_applied_prefix, t.mode_label(name)),
        );
    }

    fn reset_defaults(&mut self) {
        self.run_privileged(&["reset"], tr().defaults_restored.into());
    }

    fn apply_thresholds(&mut self, start: u8, stop: u8) {
        let start = start.min(stop.saturating_sub(5).max(1));
        let stop = stop.clamp(start + 1, 100);
        self.thresh_start = start;
        self.thresh_stop = stop;
        let s_str = start.to_string();
        let e_str = stop.to_string();
        let t = tr();
        self.run_privileged(
            &["thresholds", &s_str, &e_str],
            format!("{}: {start}% / {stop}%", t.charge_thresholds),
        );
    }

    fn active_profile_guess(&self) -> &'static str {
        guess_active_profile(&self.snapshot.profile)
    }
}

fn card_frame(ui: &egui::Ui) -> egui::Frame {
    let visuals = ui.visuals();
    egui::Frame::new()
        .fill(visuals.faint_bg_color)
        .stroke(visuals.widgets.noninteractive.bg_stroke)
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::same(12))
}

fn section_title(ui: &mut egui::Ui, title: &str) {
    let accent = ui.visuals().hyperlink_color;
    ui.label(egui::RichText::new(title).size(14.0).strong().color(accent));
}

fn pill(ui: &mut egui::Ui, text: &str, color: egui::Color32) {
    egui::Frame::new()
        .fill(color.linear_multiply(0.15))
        .stroke(egui::Stroke::new(1.0_f32, color.linear_multiply(0.45)))
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(8, 3))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(text).size(11.5).strong().color(color));
        });
}

impl eframe::App for LaptopTuneApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        if self.last_refresh.elapsed() >= Duration::from_millis(1500) {
            self.snapshot = collect_snapshot();
            self.last_refresh = Instant::now();
        }
        ctx.request_repaint_after(Duration::from_millis(1500));

        let t = tr();
        let accent = ui.visuals().hyperlink_color;
        let muted = ui.visuals().weak_text_color();
        let strong = ui.visuals().strong_text_color();

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(ui.visuals().panel_fill)
                    .inner_margin(14.0),
            )
            .show(ui, |ui| {
                // Header
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("laptop-tune")
                            .size(20.0)
                            .strong()
                            .color(strong),
                    );
                    pill(ui, &format!("v{}", env!("CARGO_PKG_VERSION")), accent);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(t.close).clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.button(t.refresh).clicked() {
                            self.snapshot = collect_snapshot();
                            self.last_refresh = Instant::now();
                        }
                    });
                });
                ui.add_space(10.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    // 1. Power Profile Card
                    card_frame(ui).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            section_title(ui, t.power_profile);
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new(t.reset_default).size(11.5),
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
                                &format!("{}: {}", t.platform, self.snapshot.profile.platform),
                                muted,
                            );
                            pill(
                                ui,
                                &format!("EPP: {}", self.snapshot.profile.cpu_epp),
                                muted,
                            );
                            let (turbo_txt, turbo_col) = if self.snapshot.profile.turbo == "on" {
                                (t.turbo_on, ACCENT_GREEN)
                            } else {
                                (t.turbo_off, muted)
                            };
                            pill(ui, turbo_txt, turbo_col);
                        });
                        ui.add_space(8.0);

                        let active = self.active_profile_guess();
                        let mut clicked: Option<&str> = None;
                        ui.horizontal(|ui| {
                            for id in ["eco", "balanced", "performance", "max"] {
                                let label = t.mode_label(id);
                                let is_active = active == id;
                                let btn = if is_active {
                                    egui::Button::new(
                                        egui::RichText::new(label)
                                            .strong()
                                            .color(ui.visuals().selection.stroke.color),
                                    )
                                    .fill(ui.visuals().selection.bg_fill)
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
                                .checkbox(&mut self.autostart_enabled, t.start_on_login)
                                .changed()
                            {
                                if let Some(path) = default_autostart_path() {
                                    let exe =
                                        env::current_exe().unwrap_or_else(|_| "laptop-tune".into());
                                    if let Err(e) =
                                        set_autostart_at(&path, &exe, self.autostart_enabled)
                                    {
                                        self.status_msg = Some((e, true, Instant::now()));
                                    }
                                }
                            }
                            if ui
                                .checkbox(&mut self.config.auto_switch_ac_bat, t.auto_switch)
                                .changed()
                            {
                                self.save_config();
                            }
                        });
                        ui.add_enabled_ui(self.config.auto_switch_ac_bat, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(t.on_battery).color(muted));
                                let mut bat = self.config.bat_profile.clone();
                                egui::ComboBox::from_id_salt("bat_profile_combo")
                                    .selected_text(t.mode_label(&bat))
                                    .width(110.0)
                                    .show_ui(ui, |ui| {
                                        for prof in ["eco", "balanced", "performance", "max"] {
                                            ui.selectable_value(
                                                &mut bat,
                                                prof.into(),
                                                t.mode_label(prof),
                                            );
                                        }
                                    });
                                if bat != self.config.bat_profile {
                                    self.config.bat_profile = bat;
                                    self.save_config();
                                }
                                ui.add_space(8.0);
                                ui.label(egui::RichText::new(t.on_ac).color(muted));
                                let mut ac = self.config.ac_profile.clone();
                                egui::ComboBox::from_id_salt("ac_profile_combo")
                                    .selected_text(t.mode_label(&ac))
                                    .width(110.0)
                                    .show_ui(ui, |ui| {
                                        for prof in ["eco", "balanced", "performance", "max"] {
                                            ui.selectable_value(
                                                &mut ac,
                                                prof.into(),
                                                t.mode_label(prof),
                                            );
                                        }
                                    });
                                if ac != self.config.ac_profile {
                                    self.config.ac_profile = ac;
                                    self.save_config();
                                }
                            });
                        });
                    });

                    ui.add_space(10.0);

                    // 2. Battery Card
                    let mut pending_thresholds: Option<(u8, u8)> = None;
                    card_frame(ui).show(ui, |ui| {
                        section_title(ui, t.battery);
                        ui.add_space(4.0);
                        if self.snapshot.batteries.is_empty() {
                            ui.label(egui::RichText::new(t.no_battery).color(muted));
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
                                            bat.name,
                                            t.battery_status(&bat.status),
                                            bat.capacity
                                        ))
                                        .strong(),
                                    );
                                    if let Some(w) = bat.power_w {
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                pill(ui, &format!("{w:.2} W"), accent);
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
                                    let cur_s = s.parse::<u8>().unwrap_or(self.thresh_start);
                                    let cur_e = e.parse::<u8>().unwrap_or(self.thresh_stop);
                                    let mut sel_s = cur_s;
                                    let mut sel_e = cur_e;
                                    ui.add_space(6.0);
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "{} :",
                                                t.charge_thresholds
                                            ))
                                            .color(muted),
                                        );
                                        ui.label(egui::RichText::new(t.start_label).color(muted));
                                        egui::ComboBox::from_id_salt("bat_start_thresh")
                                            .selected_text(format!("{sel_s}%"))
                                            .width(68.0)
                                            .show_ui(ui, |ui| {
                                                for val in [50u8, 60, 70, 80] {
                                                    ui.selectable_value(
                                                        &mut sel_s,
                                                        val,
                                                        format!("{val}%"),
                                                    );
                                                }
                                            });
                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new(t.stop_label).color(muted));
                                        egui::ComboBox::from_id_salt("bat_stop_thresh")
                                            .selected_text(format!("{sel_e}%"))
                                            .width(68.0)
                                            .show_ui(ui, |ui| {
                                                for val in [60u8, 70, 80, 90, 100] {
                                                    ui.selectable_value(
                                                        &mut sel_e,
                                                        val,
                                                        format!("{val}%"),
                                                    );
                                                }
                                            });
                                    });
                                    if sel_s != cur_s {
                                        if sel_s >= sel_e {
                                            sel_e = (sel_s + 10).min(100);
                                        }
                                        pending_thresholds = Some((sel_s, sel_e));
                                    } else if sel_e != cur_e {
                                        if sel_e <= sel_s {
                                            sel_s = sel_e.saturating_sub(10).max(50);
                                        }
                                        pending_thresholds = Some((sel_s, sel_e));
                                    }
                                }
                            }
                        }
                    });
                    if let Some((ps, pe)) = pending_thresholds {
                        self.apply_thresholds(ps, pe);
                    }

                    ui.add_space(10.0);

                    // 3. GPUs & Sleep Card
                    card_frame(ui).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            section_title(ui, t.gpus_and_sleep);
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    pill(
                                        ui,
                                        &format!("{}: {}", t.sleep_label, self.snapshot.mem_sleep),
                                        muted,
                                    );
                                },
                            );
                        });
                        ui.add_space(4.0);
                        for gpu in &self.snapshot.gpus {
                            let (status_text, color) = if gpu.runtime_status == "suspended" {
                                (t.gpu_suspended, ACCENT_GREEN)
                            } else {
                                (t.gpu_active, ACCENT_AMBER)
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
                    card_frame(ui).show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        section_title(ui, t.thermals);
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
                                    ui.label(egui::RichText::new(zone).color(muted));
                                    pill(ui, &format!("{deg}°C"), color);
                                    ui.end_row();
                                }
                            });
                    });
                });

                if self
                    .status_msg
                    .as_ref()
                    .is_some_and(|(_, _, ts)| ts.elapsed() >= Duration::from_secs(3))
                {
                    self.status_msg = None;
                }
                if let Some((msg, is_err, _)) = &self.status_msg {
                    let col = if *is_err { ACCENT_RED } else { ACCENT_GREEN };
                    let bg = ui.visuals().extreme_bg_color;
                    egui::Area::new(ui.make_persistent_id("toast_notification"))
                        .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -14.0))
                        .interactable(false)
                        .order(egui::Order::Foreground)
                        .show(&ctx, |ui| {
                            egui::Frame::new()
                                .fill(bg)
                                .stroke(egui::Stroke::new(1.2, col))
                                .corner_radius(egui::CornerRadius::same(8))
                                .inner_margin(egui::Margin::symmetric(14, 7))
                                .show(ui, |ui| {
                                    ui.label(
                                        egui::RichText::new(msg).size(12.5).strong().color(col),
                                    );
                                });
                        });
                }
            });
    }
}

pub fn run_gui() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([520.0, 665.0])
            .with_min_inner_size([400.0, 460.0]),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "laptop-tune",
        options,
        Box::new(|cc| {
            LaptopTuneApp::configure_spacing(&cc.egui_ctx);
            Ok(Box::new(LaptopTuneApp::new()))
        }),
    ) {
        eprintln!("Failed to launch GUI: {e}");
        std::process::exit(1);
    }
}
