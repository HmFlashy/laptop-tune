use std::env;
use std::fs;
use std::io::{BufRead, BufReader};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use ksni::blocking::{Handle, TrayMethods};

use crate::config::{default_config_path, load_config_from};
use crate::daemon::is_on_ac_in;
use crate::i18n::tr;
use crate::snapshot::{guess_active_profile, read_profile_state_in};

pub enum TrayEvent {
    ToggleWindow,
    ApplyProfile(String),
    PowerChanged,
    Refresh,
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
        let state = read_profile_state_in(Path::new("/sys"));
        let active = guess_active_profile(&state);
        if active.is_empty() {
            "laptop-tune".into()
        } else {
            format!("laptop-tune ({active})")
        }
    }

    fn icon_name(&self) -> String {
        "laptop-tune".into()
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.tx.send(TrayEvent::ToggleWindow);
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        let t = tr();
        let state = read_profile_state_in(Path::new("/sys"));
        let active = guess_active_profile(&state);

        vec![
            StandardItem {
                label: t.show_hide.into(),
                activate: Box::new(|this: &mut Self| {
                    let _ = this.tx.send(TrayEvent::ToggleWindow);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            CheckmarkItem {
                label: t.mode_eco.into(),
                checked: active == "eco",
                activate: Box::new(|this: &mut Self| {
                    let _ = this.tx.send(TrayEvent::ApplyProfile("eco".into()));
                }),
                ..Default::default()
            }
            .into(),
            CheckmarkItem {
                label: t.mode_balanced.into(),
                checked: active == "balanced",
                activate: Box::new(|this: &mut Self| {
                    let _ = this.tx.send(TrayEvent::ApplyProfile("balanced".into()));
                }),
                ..Default::default()
            }
            .into(),
            CheckmarkItem {
                label: t.mode_performance.into(),
                checked: active == "performance",
                activate: Box::new(|this: &mut Self| {
                    let _ = this.tx.send(TrayEvent::ApplyProfile("performance".into()));
                }),
                ..Default::default()
            }
            .into(),
            CheckmarkItem {
                label: t.mode_max.into(),
                checked: active == "max",
                activate: Box::new(|this: &mut Self| {
                    let _ = this.tx.send(TrayEvent::ApplyProfile("max".into()));
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: t.quit.into(),
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
    tx: Sender<TrayEvent>,
    _handle: Handle<LaptopTuneTray>,
}

pub fn spawn_tray() -> Option<TrayHandle> {
    let (tx, rx) = mpsc::channel();
    let tray = LaptopTuneTray { tx: tx.clone() };
    match tray.disable_dbus_name(true).spawn() {
        Ok(handle) => Some(TrayHandle {
            rx,
            tx,
            _handle: handle,
        }),
        Err(e) => {
            eprintln!("Tray disabled ({e})");
            None
        }
    }
}

fn tray_socket_path() -> Option<PathBuf> {
    if let Ok(runtime) = env::var("XDG_RUNTIME_DIR") {
        if !runtime.is_empty() {
            return Some(PathBuf::from(runtime).join("laptop-tune-tray.sock"));
        }
    }
    None
}

pub fn notify_tray_refresh() {
    if let Some(sock) = tray_socket_path() {
        let _ = UnixStream::connect(sock);
    }
}

fn spawn_socket_listener(tx: Sender<TrayEvent>) -> Option<PathBuf> {
    let sock_path = tray_socket_path()?;
    let _ = fs::remove_file(&sock_path);
    let listener = UnixListener::bind(&sock_path).ok()?;
    thread::spawn(move || {
        for stream in listener.incoming() {
            if stream.is_ok() && tx.send(TrayEvent::Refresh).is_err() {
                break;
            }
        }
    });
    Some(sock_path)
}

fn spawn_gui_window(exe: &Path) -> Option<Child> {
    Command::new(exe).arg("gui").spawn().ok()
}

fn spawn_power_watcher(tx: Sender<TrayEvent>) {
    thread::spawn(move || {
        let Ok(mut child) = Command::new("udevadm")
            .args(["monitor", "--udev", "--subsystem-match=power_supply"])
            .stdout(Stdio::piped())
            .spawn()
        else {
            return;
        };
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                if line.contains("UDEV")
                    && line.contains("power_supply")
                    && tx.send(TrayEvent::PowerChanged).is_err()
                {
                    break;
                }
            }
        }
        let _ = child.kill();
        let _ = child.wait();
    });
}

/// Runs a lightweight event-driven system-tray host (zero polling).
pub fn run_tray_host() {
    let Some(tray) = spawn_tray() else {
        crate::gui::run_gui();
        return;
    };

    let sock_path = spawn_socket_listener(tray.tx.clone());
    spawn_power_watcher(tray.tx.clone());
    let exe = env::current_exe().unwrap_or_else(|_| "laptop-tune".into());
    let mut child: Option<Child> = spawn_gui_window(&exe);
    let mut last_ac = is_on_ac_in(Path::new("/sys/class/power_supply"));

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
                tray._handle.update(|_| {});
            }
            TrayEvent::PowerChanged => {
                let on_ac = is_on_ac_in(Path::new("/sys/class/power_supply"));
                if on_ac != last_ac {
                    last_ac = on_ac;
                    let cfg = default_config_path()
                        .map(|p| load_config_from(&p))
                        .unwrap_or_default();
                    if cfg.auto_switch_ac_bat {
                        let target = if on_ac {
                            &cfg.ac_profile
                        } else {
                            &cfg.bat_profile
                        };
                        let _ = Command::new("pkexec")
                            .arg(&exe)
                            .arg("profile")
                            .arg(target)
                            .status();
                    }
                    tray._handle.update(|_| {});
                }
            }
            TrayEvent::Refresh => {
                tray._handle.update(|_| {});
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

    if let Some(path) = sock_path {
        let _ = fs::remove_file(path);
    }
}
