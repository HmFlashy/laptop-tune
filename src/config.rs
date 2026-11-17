use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, PartialEq)]
pub struct AppConfig {
    pub last_profile: Option<String>,
    pub apply_on_startup: bool,
}

fn config_dir() -> Option<PathBuf> {
    if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
        if !xdg.is_empty() {
            return Some(PathBuf::from(xdg));
        }
    }
    env::var("HOME")
        .ok()
        .map(|h| PathBuf::from(h).join(".config"))
}

pub fn default_config_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("laptop-tune/config.conf"))
}

pub fn default_autostart_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("autostart/laptop-tune.desktop"))
}

pub fn load_config_from(path: &Path) -> AppConfig {
    let Ok(content) = fs::read_to_string(path) else {
        return AppConfig::default();
    };
    let mut cfg = AppConfig::default();
    for line in content.lines() {
        let line = line.trim();
        if let Some((k, v)) = line.split_once('=') {
            match k.trim() {
                "last_profile" if !v.trim().is_empty() => {
                    cfg.last_profile = Some(v.trim().to_string());
                }
                "apply_on_startup" => {
                    cfg.apply_on_startup = v.trim() == "true";
                }
                _ => {}
            }
        }
    }
    cfg
}

pub fn save_config_to(path: &Path, cfg: &AppConfig) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    let last = cfg.last_profile.as_deref().unwrap_or("");
    let content = format!(
        "last_profile={last}\napply_on_startup={}\n",
        cfg.apply_on_startup
    );
    fs::write(path, content).map_err(|e| format!("{}: {e}", path.display()))
}

pub fn set_autostart_at(path: &Path, exe_path: &Path, enabled: bool) -> Result<(), String> {
    if enabled {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        let desktop = format!(
            "[Desktop Entry]\nType=Application\nName=Laptop Tune\nComment=Monitor battery, GPU runtime PM, thermals, and switch power profiles\nExec={} gui --tray\nIcon=laptop-tune\nTerminal=false\nCategories=System;Settings;HardwareSettings;\nX-GNOME-Autostart-enabled=true\n",
            exe_path.display()
        );
        fs::write(path, desktop).map_err(|e| format!("{}: {e}", path.display()))
    } else if path.exists() {
        fs::remove_file(path).map_err(|e| format!("{}: {e}", path.display()))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sysfs::test_utils::TempDir;

    #[test]
    fn test_save_and_load_config() {
        let tmp = TempDir::new("config");
        let path = tmp.0.join("laptop-tune/config.conf");

        let cfg = AppConfig {
            last_profile: Some("eco".into()),
            apply_on_startup: true,
        };
        save_config_to(&path, &cfg).unwrap();

        let loaded = load_config_from(&path);
        assert_eq!(loaded, cfg);
    }

    #[test]
    fn test_set_autostart_creates_and_removes_desktop_entry() {
        let tmp = TempDir::new("autostart");
        let desktop = tmp.0.join("autostart/laptop-tune.desktop");
        let exe = Path::new("/usr/bin/laptop-tune");

        set_autostart_at(&desktop, exe, true).unwrap();
        assert!(desktop.exists());
        let content = fs::read_to_string(&desktop).unwrap();
        assert!(content.contains("Exec=/usr/bin/laptop-tune gui --tray"));

        set_autostart_at(&desktop, exe, false).unwrap();
        assert!(!desktop.exists());
    }
}
