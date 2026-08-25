use std::cell::RefCell;
use std::rc::Rc;

use i_slint_backend_winit::WinitWindowAccessor;
use slint::Model;

mod config;
mod winit;

use config::{BrowserEntry, Config};

slint::include_modules!();

fn load_icon(rel: &str) -> slint::Image {
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
        name: entry.name.clone().into(),
        path: entry.path.clone().into(),
        flags: entry.flags.clone().into(),
        is_default: false,
    }
}

fn launch(entry: &BrowserEntry, url: &str) -> std::io::Result<()> {
    let mut cmd = std::process::Command::new(&entry.path);
    if !entry.flags.is_empty() {
        cmd.args(entry.flags.split_whitespace());
    }
    cmd.arg(url).spawn().map(|_| ())
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

    let main_window = MainWindow::new().unwrap();

    if std::env::var("BP_THEME").as_deref() == Ok("light") {
        main_window.global::<Theme>().set_is_dark(false);
    }

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

            println!(
                "Launching [{}]: {:?} {:?} {}",
                index, entry.path, entry.flags, url
            );

            let entry_ref = BrowserEntry {
                name: entry.name.to_string(),
                path: entry.path.to_string(),
                flags: entry.flags.to_string(),
                icon: String::new(),
            };

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
                Err(e) => eprintln!("launch failed: {e}"),
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
            };
            let url = main_window.get_current_url().to_string();

            println!(
                "Launching [{}]: {:?} {:?} {}",
                index, entry.path, entry.flags, url
            );

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
                Err(e) => eprintln!("launch failed: {e}"),
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

    main_window.on_esc_pressed({
        let main_window = main_window.clone_strong();
        move || {
            let _ = main_window.window().hide();
        }
    });

    main_window.set_browser_model(browser_model.clone().into());
    State {
        main_window,
        browser_model,
        config: cfg,
    }
}

pub struct State {
    pub main_window: MainWindow,
    pub browser_model: Rc<slint::VecModel<BrowserConfig>>,
    pub config: Rc<RefCell<Config>>,
}

pub fn main() {
    let mut cfg = config::load();
    config::ensure_browsers(&mut cfg);

    let url = std::env::args().nth(1);

    if let Some(url) = &url {
        let domain = config::domain_of(url);
        if let Some(name) = cfg.remembered.get(&domain).cloned() {
            if cfg.settings.always_ask {
                eprintln!("[bp] '{domain}' remembered -> {name}, but always_ask is on");
            } else if let Some(entry) = cfg.browsers.iter().find(|b| b.name == name) {
                eprintln!("[bp] '{domain}' remembered -> launching {name}");
                let _ = launch(entry, url);
                return;
            }
        }
    }

    let state = init(cfg, url);
    let main_window = state.main_window.clone_strong();
    main_window.show().unwrap();

    let weak = main_window.as_weak();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(mw) = weak.upgrade() {
            winit::center_window(mw.window());
        }
    });

    main_window.run().unwrap();
}
