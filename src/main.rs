use std::rc::Rc;

use i_slint_backend_winit::WinitWindowAccessor;
use slint::Model;

use crate::winit::center_window;
mod winit;

slint::include_modules!();

fn load_icon(rel: &str) -> slint::Image {
    slint::Image::load_from_path(std::path::Path::new(rel)).unwrap_or_default()
}

fn init() -> State {
    let browsers = vec![
        BrowserConfig {
            icon: load_icon("logos/firefox_48x48.png"),
            name: "Work browser".into(),
            path: "firefox.exe".into(),
            flags: "--profile work".into(),
            is_default: true,
        },
        BrowserConfig {
            icon: load_icon("logos/chrome_48x48.png"),
            name: "Ku6epXBOCTuK".into(),
            path: "chrome.exe".into(),
            flags: "--profile personal".into(),
            is_default: false,
        },
        BrowserConfig {
            icon: load_icon("logos/opera_48x48.png"),
            name: "Chrome".into(),
            path: "chrome.exe".into(),
            flags: "".into(),
            is_default: false,
        },
        BrowserConfig {
            icon: load_icon("icons/globe.svg"),
            name: "Zen Browser".into(),
            path: "zen.exe".into(),
            flags: "".into(),
            is_default: false,
        },
    ];

    let default_index = browsers
        .iter()
        .position(|b| b.is_default)
        .unwrap_or_default();

    let browser_model = Rc::new(slint::VecModel::<BrowserConfig>::from(browsers));

    let main_window = MainWindow::new().unwrap();

    if std::env::var("BP_THEME").as_deref() == Ok("light") {
        main_window.global::<Theme>().set_is_dark(false);
    }

    let url = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "ku6epxboctuk.github.io".into());
    main_window.set_current_url(url.into());
    main_window.set_selected_index(default_index as i32);
    main_window.set_remember_choice(true);
    main_window.set_always_ask(true);

    main_window.on_launch_browser({
        let main_window = main_window.clone_strong();
        let browser_model = browser_model.clone();
        move |browser: BrowserConfig| {
            let index = (0..browser_model.row_count())
                .find(|&i| {
                    browser_model
                        .row_data(i)
                        .is_some_and(|b| b.name == browser.name)
                })
                .unwrap_or(0);
            main_window.set_selected_index(index as i32);

            let url = main_window.get_current_url().to_string();
            println!(
                "Launching [{}]: {:?} {:?} {}",
                index, browser.path, browser.flags, url
            );

            let mut cmd = std::process::Command::new(browser.path.to_string());
            if !browser.flags.is_empty() {
                cmd.args(browser.flags.split_whitespace());
            }

            match cmd.arg(url).spawn() {
                Ok(_) => {
                    let _ = main_window.window().hide();
                }
                Err(e) => eprintln!("launch failed: {e}"),
            }
        }
    });

    main_window.on_open_clicked({
        let main_window = main_window.clone_strong();
        let browser_model = browser_model.clone();
        move || {
            let index = main_window.get_selected_index().max(0) as usize;
            let Some(browser) = browser_model.row_data(index) else {
                return;
            };
            let url = main_window.get_current_url().to_string();

            println!(
                "Launching [{}]: {:?} {:?} {}",
                index, browser.path, browser.flags, url
            );

            let mut cmd = std::process::Command::new(browser.path.to_string());
            if !browser.flags.is_empty() {
                cmd.args(browser.flags.split_whitespace());
            }

            match cmd.arg(url).spawn() {
                Ok(_) => {
                    let _ = main_window.window().hide();
                }
                Err(e) => eprintln!("launch failed: {e}"),
            }
        }
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
    }
}

pub struct State {
    pub main_window: MainWindow,
    pub browser_model: Rc<slint::VecModel<BrowserConfig>>,
}

pub fn main() {
    // TODO: how to enable skia? not panics
    // let _selector = BackendSelector::new()
    //     .backend_name("renderer_skia".into())
    //     .select()
    //     .unwrap();

    let state = init();
    let main_window = state.main_window.clone_strong();
    main_window.show().unwrap();

    let weak = main_window.as_weak();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(mw) = weak.upgrade() {
            center_window(mw.window());
        }
    });

    main_window.run().unwrap();
}
