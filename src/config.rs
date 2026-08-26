use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub fn log(msg: &str) {
    use std::io::Write;
    let Ok(appdata) = std::env::var("APPDATA") else {
        return;
    };
    let dir = Path::new(&appdata).join("bropicker");
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let path = dir.join("bropicker.log");
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(f, "[{ts}] {msg}");
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BrowserEntry {
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub flags: String,
    #[serde(default)]
    pub icon: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    #[serde(default = "default_true")]
    pub remember_choice: bool,
    #[serde(default = "default_true")]
    pub always_ask: bool,
}

fn default_true() -> bool {
    true
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            remember_choice: true,
            always_ask: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Config {
    #[serde(default)]
    pub browsers: Vec<BrowserEntry>,
    #[serde(default)]
    pub remembered: HashMap<String, String>,
    #[serde(default)]
    pub settings: Settings,
}

pub fn config_path() -> PathBuf {
    if let Ok(p) = std::env::var("BP_CONFIG") {
        return PathBuf::from(p);
    }
    std::env::var("APPDATA")
        .map(|d| PathBuf::from(d).join("bropicker").join("config.toml"))
        .unwrap_or_else(|_| PathBuf::from("config.toml"))
}

pub fn load() -> Config {
    let path = config_path();
    match std::fs::read_to_string(&path) {
        Ok(s) => toml::from_str(&s).unwrap_or_else(|e| {
            crate::config::log(&format!("[bp] config parse error ({path:?}): {e}"));
            Config::default()
        }),
        Err(_) => Config::default(),
    }
}

pub fn save(cfg: &Config) {
    let path = config_path();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    match toml::to_string_pretty(cfg) {
        Ok(s) => {
            if let Err(e) = std::fs::write(&path, s) {
                log(&format!("[bp] config save failed ({path:?}): {e}"));
            }
        }
        Err(e) => log(&format!("[bp] config serialize failed: {e}")),
    }
}

pub fn ensure_browsers(cfg: &mut Config) {
    if cfg.browsers.is_empty() {
        cfg.browsers = detect_browsers();
        log(&format!("[bp] autodetected {} browsers", cfg.browsers.len()));
        if !cfg.browsers.is_empty() {
            save(cfg);
        }
    }
}

pub fn domain_of(url: &str) -> String {
    let rest = url.split("://").nth(1).unwrap_or(url);
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    let host = host.split('@').next().unwrap_or(host);
    let host = host.split(':').next().unwrap_or(host);
    host.trim_start_matches("www.").to_lowercase()
}

fn extract_exe(cmd: &str) -> Option<String> {
    let t = cmd.trim();
    if t.starts_with('"') {
        t.split('"').nth(1).map(String::from)
    } else {
        t.split_whitespace().next().map(String::from)
    }
}

pub fn detect_browsers() -> Vec<BrowserEntry> {
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
    use winreg::RegKey;

    let mut out: Vec<BrowserEntry> = Vec::new();

    for hive in [HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER] {
        let Ok(clients) =
            RegKey::predef(hive).open_subkey("SOFTWARE\\Clients\\StartMenuInternet")
        else {
            continue;
        };
        for key_name in clients.enum_keys().flatten() {
            let Ok(client) = clients.open_subkey(&key_name) else {
                continue;
            };
            let friendly: String = client
                .get_value("")
                .unwrap_or_else(|_| key_name.clone());
            let command: String = client
                .open_subkey("shell\\open\\command")
                .and_then(|k| k.get_value(""))
                .unwrap_or_default();
            let Some(path) = extract_exe(&command) else {
                continue;
            };
            if !Path::new(&path).exists() {
                continue;
            }
            if out
                .iter()
                .any(|b: &BrowserEntry| b.path.eq_ignore_ascii_case(&path))
            {
                continue;
            }
            out.push(BrowserEntry {
                name: friendly,
                path,
                flags: String::new(),
                icon: String::new(),
            });
        }
    }

    out
}

pub fn icon_for(name: &str) -> &'static str {
    let n = name.to_lowercase();
    if n.contains("firefox") {
        "logos/firefox_48x48.png"
    } else if n.contains("chrome") {
        "logos/chrome_48x48.png"
    } else if n.contains("opera") {
        "logos/opera_48x48.png"
    } else if n.contains("brave") {
        "logos/brave_48x48.png"
    } else if n.contains("СЏРЅРґРµРєСЃ") || n.contains("yandex") {
        "logos/yandex_48x48.png"
    } else {
        "icons/globe.svg"
    }
}
