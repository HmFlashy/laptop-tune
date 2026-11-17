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
        "battery".into()
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
