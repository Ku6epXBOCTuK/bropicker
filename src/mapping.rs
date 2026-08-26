use crate::BrowserConfig;
use crate::config::{self, BrowserEntry};

impl From<&BrowserConfig> for BrowserEntry {
    fn from(b: &BrowserConfig) -> Self {
        Self {
            name: b.name.to_string(),
            path: b.path.to_string(),
            flags: b.flags.to_string(),
            icon: String::new(),
            emoji: b.emoji.to_string(),
        }
    }
}

pub fn load_icon(rel: &str) -> slint::Image {
    let from_exe = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join(rel)));
    let path = from_exe
        .filter(|p| p.exists())
        .unwrap_or_else(|| std::path::PathBuf::from(rel));
    slint::Image::load_from_path(&path).unwrap_or_default()
}

pub fn to_ui_config(entry: &BrowserEntry) -> BrowserConfig {
    let icon_rel = if entry.icon.is_empty() {
        config::icon_for(&entry.name)
    } else {
        &entry.icon
    };
    BrowserConfig {
        icon: load_icon(icon_rel),
        emoji: entry.emoji.clone().into(),
        icon_mask: entry.icon.is_empty()
            && entry.emoji.is_empty()
            && !config::is_known_browser(&entry.name),
        name: entry.name.clone().into(),
        path: entry.path.clone().into(),
        flags: entry.flags.clone().into(),
        is_default: false,
    }
}
