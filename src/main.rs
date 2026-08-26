#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app;
mod config;
mod launcher;
mod mapping;
mod winit;

slint::include_modules!();

pub fn main() {
    let start = std::time::Instant::now();

    let mut cfg = config::load();
    config::ensure_browsers(&mut cfg);

    let url = std::env::args().nth(1);

    if let Some(url) = &url {
        let domain = config::domain_of(url);
        if let Some(name) = cfg.remembered.get(&domain).cloned() {
            if cfg.settings.always_ask {
                config::log(&format!(
                    "'{domain}' remembered -> {name}, but always_ask is on"
                ));
            } else if let Some(entry) = cfg.browsers.iter().find(|b| b.name == name) {
                config::log(&format!("'{domain}' remembered -> launching {name}"));
                let _ = launcher::launch(entry, url);
                return;
            }
        }
    }

    let app = app::App::new(cfg, url);
    app.wire();
    app.run(start);
}
