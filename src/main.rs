#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::cell::RefCell;
use std::rc::Rc;

use i_slint_backend_winit::WinitWindowAccessor;
use slint::Model;

mod config;
mod launcher;
mod winit;

use config::{BrowserEntry, Config};

slint::include_modules!();

const DEFAULT_URL: &str = "ku6epxboctuk.github.io";
const ENV_THEME: &str = "BP_THEME";
const ENV_VIEW: &str = "BP_VIEW";
const VIEW_SETTINGS: &str = "settings";

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

fn load_icon(rel: &str) -> slint::Image {
    let from_exe = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join(rel)));
    let path = from_exe
        .filter(|p| p.exists())
        .unwrap_or_else(|| std::path::PathBuf::from(rel));
    slint::Image::load_from_path(&path).unwrap_or_default()
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

fn launch_index(
    main_window: &MainWindow,
    browser_model: &slint::VecModel<BrowserConfig>,
    cfg: &Rc<RefCell<Config>>,
    index: usize,
) {
    let Some(ui_entry) = browser_model.row_data(index) else {
        return;
    };
    main_window.set_selected_index(index as i32);

    let entry = BrowserEntry::from(&ui_entry);
    let url = main_window.get_current_url().to_string();
    let remember = main_window.get_remember_choice();

    if launcher::launch_and_remember(cfg, &entry, &url, remember) {
        let _ = main_window.window().hide();
    }
}

fn open_config_in_editor() {
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
        config::log(&format!("cannot open editor: {e}"));
    }
}

fn init(cfg: Config, url: Option<String>) -> (MainWindow, SettingsWindow) {
    let cfg = Rc::new(RefCell::new(cfg));
    let browser_model = Rc::new(slint::VecModel::<BrowserConfig>::from(vec![]));
    let remembered_model = Rc::new(slint::VecModel::<RememberedEntry>::from(vec![]));

    let main_window = MainWindow::new().unwrap();
    let settings_window = SettingsWindow::new().unwrap();

    if std::env::var(ENV_THEME).as_deref() == Ok("light") {
        main_window.global::<Theme>().set_is_dark(false);
    }

    refresh_models(&browser_model, &remembered_model, &cfg.borrow());

    settings_window.set_browsers(browser_model.clone().into());
    settings_window.set_remembered(remembered_model.clone().into());
    main_window.set_browser_model(browser_model.clone().into());

    main_window.set_current_url(
        url.clone()
            .unwrap_or_else(|| DEFAULT_URL.to_string())
            .into(),
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
        move |index: i32| {
            launch_index(&main_window, &browser_model, &cfg, index.max(0) as usize);
        }
    });

    main_window.on_open_clicked({
        let main_window = main_window.clone_strong();
        let browser_model = browser_model.clone();
        let cfg = cfg.clone();
        move || {
            let index = main_window.get_selected_index().max(0) as usize;
            launch_index(&main_window, &browser_model, &cfg, index);
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
        open_config_in_editor();
    });

    main_window.on_request_drag({
        let main_window = main_window.clone_strong();
        move || {
            main_window.window().with_winit_window(|winit_window| {
                winit_window.drag_window().ok();
            });
        }
    });

    main_window.on_esc_pressed({
        let main_window = main_window.clone_strong();
        move || {
            let _ = main_window.window().hide();
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

    settings_window.window().on_close_requested({
        let main_window = main_window.clone_strong();
        move || {
            main_window.invoke_restore_focus();
            slint::CloseRequestResponse::HideWindow
        }
    });

    (main_window, settings_window)
}

fn show_and_run(
    main_window: MainWindow,
    settings_window: SettingsWindow,
    start: std::time::Instant,
) {
    let settings_only = std::env::var(ENV_VIEW).as_deref() == Ok(VIEW_SETTINGS);

    if settings_only {
        settings_window.show().unwrap();
        let weak = settings_window.as_weak();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(sw) = weak.upgrade() {
                winit::center_window(sw.window());
                sw.invoke_restore_focus();
            }
            config::log(&format!("cold start -> event loop: {:?}", start.elapsed()));
        });
        settings_window.run().unwrap();
    } else {
        main_window.show().unwrap();
        let weak = main_window.as_weak();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(mw) = weak.upgrade() {
                winit::center_window(mw.window());
                mw.invoke_restore_focus();
            }
            config::log(&format!("cold start -> event loop: {:?}", start.elapsed()));
        });
        main_window.run().unwrap();
    }
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

    let (main_window, settings_window) = init(cfg, url);
    show_and_run(main_window, settings_window, start);
}
