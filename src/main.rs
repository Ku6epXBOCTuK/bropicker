#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use std::cell::RefCell;
use std::rc::Rc;

use i_slint_backend_winit::WinitWindowAccessor;
use slint::Model;

mod config;
mod winit;

use config::{BrowserEntry, Config};

slint::include_modules!();

fn split_flags(flags: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    for c in flags.chars() {
        match c {
            '"' => in_quotes = !in_quotes,
            ' ' if !in_quotes => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            _ => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn load_icon(rel: &str) -> slint::Image {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let p = dir.join(rel);
            if p.exists() {
                return slint::Image::load_from_path(&p).unwrap_or_default();
            }
        }
    }
    slint::Image::load_from_path(std::path::Path::new(rel)).unwrap_or_default()
}

fn to_ui_config(entry: &BrowserEntry) -> BrowserConfig {
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

fn launch(entry: &BrowserEntry, url: &str) -> std::io::Result<()> {
    let mut cmd = std::process::Command::new(&entry.path);
    if !entry.flags.is_empty() {
        cmd.args(split_flags(&entry.flags));
    }
    cmd.arg(url).spawn().map(|_| ())
}

fn refresh_models(
    browser_model: &slint::VecModel<BrowserConfig>,
    remembered_model: &slint::VecModel<RememberedEntry>,
    cfg: &Config,
) {
    let rows: Vec<BrowserConfig> = cfg.browsers.iter().map(to_ui_config).collect();
    browser_model.set_vec(rows);
    let rows: Vec<RememberedEntry> = cfg
        .remembered
        .iter()
        .map(|(d, b)| RememberedEntry {
            domain: d.clone().into(),
            browser: b.clone().into(),
        })
        .collect();
    remembered_model.set_vec(rows);
}

fn init(cfg: Config, url: Option<String>) -> State {
    let cfg = Rc::new(RefCell::new(cfg));

    let ui_browsers: Vec<BrowserConfig> = cfg
        .borrow()
        .browsers
        .iter()
        .map(to_ui_config)
        .collect();
    let browser_model = Rc::new(slint::VecModel::<BrowserConfig>::from(ui_browsers));
    let remembered_model = Rc::new(slint::VecModel::<RememberedEntry>::from(vec![]));

    let main_window = MainWindow::new().unwrap();

    if std::env::var("BP_THEME").as_deref() == Ok("light") {
        main_window.global::<Theme>().set_is_dark(false);
    }

    refresh_models(&browser_model, &remembered_model, &cfg.borrow());

    let settings_window = SettingsWindow::new().unwrap();
    settings_window.set_browsers(browser_model.clone().into());
    settings_window.set_remembered(remembered_model.clone().into());

    main_window.set_current_url(
        url.clone().unwrap_or_else(|| "ku6epxboctuk.github.io".into()).into(),
    );
    if let Some(u) = &url {
        main_window.set_current_domain(config::domain_of(u).into());
    }
    main_window.set_remember_choice(cfg.borrow().settings.remember_choice);
    main_window.set_always_ask(cfg.borrow().settings.always_ask);

    main_window.on_launch_browser({
        let main_window = main_window.clone_strong();
        let browser_model = browser_model.clone();
        let cfg = cfg.clone();
        move |browser: BrowserConfig| {
            let index = (0..browser_model.row_count())
                .find(|&i| {
                    browser_model
                        .row_data(i)
                        .is_some_and(|b| b.name == browser.name)
                })
                .unwrap_or(0);
            main_window.set_selected_index(index as i32);

            let entry = browser_model.row_data(index).expect("row exists");
            let url = main_window.get_current_url().to_string();

            let entry_ref = BrowserEntry {
                name: entry.name.to_string(),
                path: entry.path.to_string(),
                flags: entry.flags.to_string(),
                icon: String::new(),
                emoji: String::new(),
            };
            let url = main_window.get_current_url().to_string();

            config::log(&format!(
                "[bp] launching [{}]: {:?} {:?} {}",
                index, entry_ref.path, entry_ref.flags, url
            ));

            match launch(&entry_ref, &url) {
                Ok(_) => {
                    let domain = config::domain_of(&url);
                    if main_window.get_remember_choice() && !domain.is_empty() {
                        cfg.borrow_mut()
                            .remembered
                            .insert(domain, entry_ref.name.clone());
                        config::save(&cfg.borrow());
                    }
                    let _ = main_window.window().hide();
                }
                Err(e) => config::log(&format!("[bp] launch failed: {e}")),
            }
        }
    });

    main_window.on_open_clicked({
        let main_window = main_window.clone_strong();
        let browser_model = browser_model.clone();
        let cfg = cfg.clone();
        move || {
            let index = main_window.get_selected_index().max(0) as usize;
            let Some(ui_entry) = browser_model.row_data(index) else {
                return;
            };
            let entry = BrowserEntry {
                name: ui_entry.name.to_string(),
                path: ui_entry.path.to_string(),
                flags: ui_entry.flags.to_string(),
                icon: String::new(),
                emoji: String::new(),
            };
            let url = main_window.get_current_url().to_string();

            config::log(&format!(
                "[bp] launching [{}]: {:?} {:?} {}",
                index, entry.path, entry.flags, url
            ));

            match launch(&entry, &url) {
                Ok(_) => {
                    let domain = config::domain_of(&url);
                    if main_window.get_remember_choice() && !domain.is_empty() {
                        cfg.borrow_mut()
                            .remembered
                            .insert(domain, entry.name.clone());
                        config::save(&cfg.borrow());
                    }
                    let _ = main_window.window().hide();
                }
                Err(e) => config::log(&format!("[bp] launch failed: {e}")),
            }
        }
    });

    main_window.on_toggle_remember({
        let cfg = cfg.clone();
        move |checked| {
            cfg.borrow_mut().settings.remember_choice = checked;
            config::save(&cfg.borrow());
        }
    });

    main_window.on_toggle_always_ask({
        let cfg = cfg.clone();
        move |checked| {
            cfg.borrow_mut().settings.always_ask = checked;
            config::save(&cfg.borrow());
        }
    });

    main_window.on_settings_clicked(move || {
        println!("Settings clicked");
    });

    main_window.on_request_drag({
        let main_window = main_window.clone_strong();
        move || {
            main_window.window().with_winit_window(|winit_window| {
                winit_window.drag_window().ok();
            });
        }
    });

    main_window.on_settings_clicked({
        move || {
            let path = config::config_path();
            if !path.exists() {
                let mut c = config::load();
                config::ensure_browsers(&mut c);
            }
            let editor = std::process::Command::new("subl")
                .arg(&path)
                .spawn()
                .or_else(|_| std::process::Command::new("notepad").arg(&path).spawn());
            if let Err(e) = editor {
                config::log(&format!("[bp] cannot open editor: {e}"));
            }
        }
    });

    settings_window.on_remove_browser({
        let browser_model = browser_model.clone();
        let remembered_model = remembered_model.clone();
        let cfg = cfg.clone();
        move |name| {
            let name = name.to_string();
            cfg.borrow_mut().browsers.retain(|b| b.name != name);
            config::save(&cfg.borrow());
            refresh_models(&browser_model, &remembered_model, &cfg.borrow());
            println!("Removed browser: {name}");
        }
    });

    settings_window.on_remove_remembered({
        let browser_model = browser_model.clone();
        let remembered_model = remembered_model.clone();
        let cfg = cfg.clone();
        move |domain| {
            let domain = domain.to_string();
            cfg.borrow_mut().remembered.remove(&domain);
            config::save(&cfg.borrow());
            refresh_models(&browser_model, &remembered_model, &cfg.borrow());
            println!("Removed remembered domain: {domain}");
        }
    });

    settings_window.on_redetect({
        let browser_model = browser_model.clone();
        let remembered_model = remembered_model.clone();
        let cfg = cfg.clone();
        move || {
            let detected = config::detect_browsers();
            let mut cfg_ref = cfg.borrow_mut();
            for entry in detected {
                if !cfg_ref
                    .browsers
                    .iter()
                    .any(|b| b.path.eq_ignore_ascii_case(&entry.path))
                {
                    cfg_ref.browsers.push(entry);
                }
            }
            config::save(&cfg_ref);
            drop(cfg_ref);
            refresh_models(&browser_model, &remembered_model, &cfg.borrow());
            println!("Re-detected browsers");
        }
    });

    settings_window.on_esc_pressed({
        let settings_window = settings_window.clone_strong();
        let main_window = main_window.clone_strong();
        move || {
            let _ = settings_window.window().hide();
            main_window.invoke_restore_focus();
        }
    });

    main_window.on_esc_pressed({
        let main_window = main_window.clone_strong();
        move || {
            let _ = main_window.window().hide();
        }
    });

    settings_window.window().on_close_requested({
        let main_window = main_window.clone_strong();
        move || {
            main_window.invoke_restore_focus();
            slint::CloseRequestResponse::HideWindow
        }
    });

    main_window.set_browser_model(browser_model.clone().into());
    State {
        main_window,
        settings_window,
        browser_model,
        remembered_model,
        config: cfg,
    }
}

pub struct State {
    pub main_window: MainWindow,
    pub settings_window: SettingsWindow,
    pub browser_model: Rc<slint::VecModel<BrowserConfig>>,
    pub remembered_model: Rc<slint::VecModel<RememberedEntry>>,
    pub config: Rc<RefCell<Config>>,
}

pub fn main() {
    let start = std::time::Instant::now();

    let mut cfg = config::load();
    config::ensure_browsers(&mut cfg);

    let url = std::env::args().nth(1);

    if let Some(url) = &url {
        let domain = config::domain_of(url);
        if let Some(name) = cfg.remembered.get(&domain).cloned() {
            if cfg.settings.always_ask {
                config::log(&format!("[bp] '{domain}' remembered -> {name}, but always_ask is on"));
            } else if let Some(entry) = cfg.browsers.iter().find(|b| b.name == name) {
                config::log(&format!("[bp] '{domain}' remembered -> launching {name}"));
                let _ = launch(entry, url);
                return;
            }
        }
    }

    let state = init(cfg, url);

    if std::env::var("BP_VIEW").as_deref() == Ok("settings") {
        state.settings_window.show().unwrap();
        let weak = state.settings_window.as_weak();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(sw) = weak.upgrade() {
                winit::center_window(sw.window());
                sw.invoke_restore_focus();
            }
        });
        state.settings_window.run().unwrap();
    } else {
        let main_window = state.main_window.clone_strong();
        main_window.show().unwrap();

        let weak = main_window.as_weak();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(mw) = weak.upgrade() {
                winit::center_window(mw.window());
                mw.invoke_restore_focus();
            }
            config::log(&format!(
                "[bp] cold start -> event loop: {:?}",
                start.elapsed()
            ));
        });

        state.main_window.run().unwrap();
    }
}
