use std::rc::Rc;

use i_slint_backend_winit::WinitWindowAccessor;

use crate::winit::center_window;
mod winit;

slint::include_modules!();

fn init() -> State {
    let browser_model = Rc::new(slint::VecModel::<BrowserConfig>::from(vec![
        BrowserConfig {
            icon: "🦊".into(),
            name: "Work browser".into(),
            path: "firefox.exe".into(),
            flags: "--profile work".into(),
            is_default: true,
        },
        BrowserConfig {
            icon: "🔥".into(),
            name: "Ku6epXBOCTuK".into(),
            path: "chrome.exe".into(),
            flags: "--profile personal".into(),
            is_default: false,
        },
        BrowserConfig {
            icon: "🌐".into(),
            name: "Chrome".into(),
            path: "chrome.exe".into(),
            flags: "".into(),
            is_default: false,
        },
        BrowserConfig {
            icon: "🧘".into(),
            name: "Zen Browser".into(),
            path: "zen.exe".into(),
            flags: "".into(),
            is_default: false,
        },
    ]));

    let main_window = MainWindow::new().unwrap();

    if std::env::var("BP_THEME").as_deref() == Ok("light") {
        main_window.global::<Theme>().set_is_dark(false);
    }

    main_window.set_current_url("ku6epxboctuk.github.io".into());
    main_window.set_remember_choice(true);
    main_window.set_always_ask(true);

    main_window.on_browser_selected({
        move |browser: BrowserConfig| {
            println!("Selected browser: {:?} {:?}", browser.path, browser.flags);
        }
    });

    main_window.on_toggle_remember(move |checked| {
        println!("Remember choice: {}", checked);
    });

    main_window.on_toggle_always_ask(move |checked| {
        println!("Always ask: {}", checked);
    });

    main_window.on_settings_clicked(move || {
        println!("Settings clicked");
    });

    main_window.on_open_clicked(move || {
        println!("Open clicked");
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
