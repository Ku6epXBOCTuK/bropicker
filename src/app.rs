use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use i_slint_backend_winit::WinitWindowAccessor;
use slint::{ComponentHandle, Model};

use crate::config::{self, BrowserEntry, Config};
use crate::launcher;
use crate::mapping::to_ui_config;
use crate::winit;
use crate::{BrowserConfig, MainWindow, RememberedEntry, SettingsWindow, Theme};

const DEFAULT_URL: &str = "ku6epxboctuk.github.io";
const ENV_THEME: &str = "BP_THEME";
const ENV_VIEW: &str = "BP_VIEW";
const VIEW_SETTINGS: &str = "settings";

pub struct App {
    main_window: MainWindow,
    settings_window: SettingsWindow,
    browser_model: Rc<slint::VecModel<BrowserConfig>>,
    remembered_model: Rc<slint::VecModel<RememberedEntry>>,
    config: Rc<RefCell<Config>>,
}

impl App {
    pub fn new(cfg: Config, url: Option<String>) -> Self {
        let config = Rc::new(RefCell::new(cfg));
        let browser_model = Rc::new(slint::VecModel::<BrowserConfig>::from(vec![]));
        let remembered_model = Rc::new(slint::VecModel::<RememberedEntry>::from(vec![]));

        let main_window = MainWindow::new().unwrap();
        let settings_window = SettingsWindow::new().unwrap();

        if std::env::var(ENV_THEME).as_deref() == Ok("light") {
            main_window.global::<Theme>().set_is_dark(false);
        }

        Self::refresh_models(&browser_model, &remembered_model, &config.borrow());

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
        main_window.set_remember_choice(config.borrow().settings.remember_choice);
        main_window.set_always_ask(config.borrow().settings.always_ask);

        Self {
            main_window,
            settings_window,
            browser_model,
            remembered_model,
            config,
        }
    }

    pub fn wire(&self) {
        self.wire_launch();
        self.wire_config_editor();
        self.wire_settings_window();
        self.wire_window_chrome();
    }

    fn wire_launch(&self) {
        self.main_window.on_launch_browser({
            let main_window = self.main_window.clone_strong();
            let browser_model = self.browser_model.clone();
            let config = self.config.clone();
            move |index: i32| {
                let index = index.max(0) as usize;
                let Some(ui_entry) = browser_model.row_data(index) else {
                    return;
                };
                main_window.set_selected_index(index as i32);
                let entry = BrowserEntry::from(&ui_entry);
                let url = main_window.get_current_url().to_string();
                let remember = main_window.get_remember_choice();
                if launcher::launch_and_remember(&config, &entry, &url, remember) {
                    let _ = main_window.window().hide();
                }
            }
        });

        self.main_window.on_open_clicked({
            let main_window = self.main_window.clone_strong();
            let browser_model = self.browser_model.clone();
            let config = self.config.clone();
            move || {
                let index = main_window.get_selected_index().max(0) as usize;
                let Some(ui_entry) = browser_model.row_data(index) else {
                    return;
                };
                let entry = BrowserEntry::from(&ui_entry);
                let url = main_window.get_current_url().to_string();
                let remember = main_window.get_remember_choice();
                if launcher::launch_and_remember(&config, &entry, &url, remember) {
                    let _ = main_window.window().hide();
                }
            }
        });

        self.main_window.on_toggle_remember({
            let config = self.config.clone();
            move |checked| {
                config.borrow_mut().settings.remember_choice = checked;
                config::save(&config.borrow());
            }
        });

        self.main_window.on_toggle_always_ask({
            let config = self.config.clone();
            move |checked| {
                config.borrow_mut().settings.always_ask = checked;
                config::save(&config.borrow());
            }
        });
    }

    fn wire_config_editor(&self) {
        self.main_window.on_settings_clicked(move || {
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
        });
    }

    fn wire_settings_window(&self) {
        self.settings_window.on_remove_browser({
            let browser_model = self.browser_model.clone();
            let remembered_model = self.remembered_model.clone();
            let config = self.config.clone();
            move |name| {
                let name = name.to_string();
                config.borrow_mut().browsers.retain(|b| b.name != name);
                config::save(&config.borrow());
                Self::refresh_models(&browser_model, &remembered_model, &config.borrow());
            }
        });

        self.settings_window.on_remove_remembered({
            let browser_model = self.browser_model.clone();
            let remembered_model = self.remembered_model.clone();
            let config = self.config.clone();
            move |domain| {
                let domain = domain.to_string();
                config.borrow_mut().remembered.remove(&domain);
                config::save(&config.borrow());
                Self::refresh_models(&browser_model, &remembered_model, &config.borrow());
            }
        });

        self.settings_window.on_redetect({
            let browser_model = self.browser_model.clone();
            let remembered_model = self.remembered_model.clone();
            let config = self.config.clone();
            move || {
                let detected = config::detect_browsers();
                let mut cfg_ref = config.borrow_mut();
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
                Self::refresh_models(&browser_model, &remembered_model, &config.borrow());
            }
        });

        self.settings_window.on_esc_pressed({
            let settings_window = self.settings_window.clone_strong();
            let main_window = self.main_window.clone_strong();
            move || {
                let _ = settings_window.window().hide();
                main_window.invoke_restore_focus();
            }
        });

        self.settings_window.window().on_close_requested({
            let main_window = self.main_window.clone_strong();
            move || {
                main_window.invoke_restore_focus();
                slint::CloseRequestResponse::HideWindow
            }
        });
    }

    fn wire_window_chrome(&self) {
        self.main_window.on_request_drag({
            let main_window = self.main_window.clone_strong();
            move || {
                main_window.window().with_winit_window(|winit_window| {
                    winit_window.drag_window().ok();
                });
            }
        });

        self.main_window.on_esc_pressed({
            let main_window = self.main_window.clone_strong();
            move || {
                let _ = main_window.window().hide();
            }
        });
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

    pub fn run(self, start: Instant) {
        let settings_only = std::env::var(ENV_VIEW).as_deref() == Ok(VIEW_SETTINGS);

        if settings_only {
            self.settings_window.show().unwrap();
            let weak = self.settings_window.as_weak();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(sw) = weak.upgrade() {
                    winit::center_window(sw.window());
                    sw.invoke_restore_focus();
                }
                config::log(&format!("cold start -> event loop: {:?}", start.elapsed()));
            });
            self.settings_window.run().unwrap();
        } else {
            self.main_window.show().unwrap();
            let weak = self.main_window.as_weak();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(mw) = weak.upgrade() {
                    winit::center_window(mw.window());
                    mw.invoke_restore_focus();
                }
                config::log(&format!("cold start -> event loop: {:?}", start.elapsed()));
            });
            self.main_window.run().unwrap();
        }
    }
}
