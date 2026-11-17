use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq)]
pub struct AppConfig {
    pub auto_switch_ac_bat: bool,
    pub ac_profile: String,
    pub bat_profile: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            auto_switch_ac_bat: false,
            ac_profile: "performance".into(),
            bat_profile: "eco".into(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SystemConfig {
    pub ac_profile: String,
    pub bat_profile: String,
    pub charge_start: u8,
    pub charge_stop: u8,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            ac_profile: "performance".into(),
            bat_profile: "eco".into(),
            charge_start: 50,
            charge_stop: 80,
        }
    }
}

pub fn load_system_config_from(path: &Path) -> SystemConfig {
    let mut cfg = SystemConfig::default();
    let Ok(content) = fs::read_to_string(path) else {
        return cfg;
    };
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let v = v.trim();
            match k.trim() {
                "ac_profile" if !v.is_empty() => cfg.ac_profile = v.to_string(),
                "bat_profile" if !v.is_empty() => cfg.bat_profile = v.to_string(),
                "charge_start" => {
                    if let Ok(n) = v.parse::<u8>() {
                        cfg.charge_start = n.min(99);
                    }
                }
                "charge_stop" => {
                    if let Ok(n) = v.parse::<u8>() {
                        cfg.charge_stop = n.clamp(1, 100);
                    }
                }
                _ => {}
            }
        }
    }
    cfg
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
                "auto_switch_ac_bat" => {
                    cfg.auto_switch_ac_bat = v.trim() == "true";
                }
                "ac_profile" if !v.trim().is_empty() => {
                    cfg.ac_profile = v.trim().to_string();
                }
                "bat_profile" if !v.trim().is_empty() => {
                    cfg.bat_profile = v.trim().to_string();
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
    let content = format!(
        "auto_switch_ac_bat={}\nac_profile={}\nbat_profile={}\n",
        cfg.auto_switch_ac_bat, cfg.ac_profile, cfg.bat_profile
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
            auto_switch_ac_bat: true,
            ac_profile: "gaming".into(),
            bat_profile: "eco".into(),
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

    #[test]
    fn test_load_system_config() {
        let tmp = TempDir::new("sysconf");
        let path = tmp.0.join("config.conf");
        fs::write(
            &path,
            "# comment\nac_profile=gaming\nbat_profile=eco\ncharge_start=40\ncharge_stop=85\n",
        )
        .unwrap();

        let cfg = load_system_config_from(&path);
        assert_eq!(cfg.ac_profile, "gaming");
        assert_eq!(cfg.bat_profile, "eco");
        assert_eq!(cfg.charge_start, 40);
        assert_eq!(cfg.charge_stop, 85);
    }
}
