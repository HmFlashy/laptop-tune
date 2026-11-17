use std::env;
use std::process::{Child, Command};
use std::sync::mpsc::{self, Receiver, Sender};

use ksni::blocking::{Handle, TrayMethods};

pub enum TrayEvent {
    ToggleWindow,
    ApplyProfile(String),
    Quit,
}

struct LaptopTuneTray {
    tx: Sender<TrayEvent>,
}

impl ksni::Tray for LaptopTuneTray {
    fn id(&self) -> String {
        "laptop-tune".into()
    }

    fn title(&self) -> String {
        "laptop-tune".into()
    }

    fn icon_name(&self) -> String {
        "laptop-tune".into()
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.tx.send(TrayEvent::ToggleWindow);
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        vec![
            StandardItem {
                label: "Show / Hide".into(),
                activate: Box::new(|this: &mut Self| {
                    let _ = this.tx.send(TrayEvent::ToggleWindow);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Profile: Eco".into(),
                activate: Box::new(|this: &mut Self| {
                    let _ = this.tx.send(TrayEvent::ApplyProfile("eco".into()));
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Profile: Balanced".into(),
                activate: Box::new(|this: &mut Self| {
                    let _ = this.tx.send(TrayEvent::ApplyProfile("balanced".into()));
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Profile: Performance".into(),
                activate: Box::new(|this: &mut Self| {
                    let _ = this.tx.send(TrayEvent::ApplyProfile("performance".into()));
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Profile: Gaming".into(),
                activate: Box::new(|this: &mut Self| {
                    let _ = this.tx.send(TrayEvent::ApplyProfile("gaming".into()));
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit".into(),
                activate: Box::new(|this: &mut Self| {
                    let _ = this.tx.send(TrayEvent::Quit);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

pub struct TrayHandle {
    pub rx: Receiver<TrayEvent>,
    _handle: Handle<LaptopTuneTray>,
}

pub fn spawn_tray() -> Option<TrayHandle> {
    let (tx, rx) = mpsc::channel();
    let tray = LaptopTuneTray { tx };
    match tray.disable_dbus_name(true).spawn() {
        Ok(handle) => Some(TrayHandle {
            rx,
            _handle: handle,
        }),
        Err(e) => {
            eprintln!("Tray disabled ({e})");
            None
        }
    }
}

fn spawn_gui_window(exe: &std::path::Path) -> Option<Child> {
    Command::new(exe).arg("gui").spawn().ok()
}

/// Runs a lightweight system-tray host that toggles the native GUI window on click.
/// Works reliably on both Wayland and X11 where window hide/unminimize is restricted.
pub fn run_tray_host() {
    let Some(tray) = spawn_tray() else {
        crate::gui::run_gui();
        return;
    };

    let exe = env::current_exe().unwrap_or_else(|_| "laptop-tune".into());
    let mut child: Option<Child> = spawn_gui_window(&exe);

    while let Ok(ev) = tray.rx.recv() {
        match ev {
            TrayEvent::ToggleWindow => {
                let is_running = child
                    .as_mut()
                    .and_then(|c| c.try_wait().ok().flatten())
                    .is_none()
                    && child.is_some();
                if is_running {
                    if let Some(mut c) = child.take() {
                        let _ = c.kill();
                        let _ = c.wait();
                    }
                } else {
                    child = spawn_gui_window(&exe);
                }
            }
            TrayEvent::ApplyProfile(name) => {
                let _ = Command::new("pkexec")
                    .arg(&exe)
                    .arg("profile")
                    .arg(&name)
                    .status();
            }
            TrayEvent::Quit => {
                if let Some(mut c) = child.take() {
                    let _ = c.kill();
                    let _ = c.wait();
                }
                break;
            }
        }
    }
}
