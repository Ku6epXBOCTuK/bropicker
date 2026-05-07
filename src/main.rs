use std::rc::Rc;

slint::include_modules!();

fn init() -> State {
    let browser_model = Rc::new(slint::VecModel::<BrowserConfig>::from(vec![
        BrowserConfig {
            icon: "chrome".into(),
            path: "chrome.exe".into(),
            flags: "profile 1".into(),
        },
        BrowserConfig {
            icon: "chrome".into(),
            path: "chrome.exe".into(),
            flags: "profile 2".into(),
        },
        BrowserConfig {
            icon: "chrome".into(),
            path: "zen.exe".into(),
            flags: "".into(),
        },
        BrowserConfig {
            icon: "chrome".into(),
            path: "edge.exe".into(),
            flags: "".into(),
        },
    ]));

    let main_window = MainWindow::new().unwrap();

    {
        main_window.window().on_close_requested(move || {
            todo!();
        });
    }

    main_window.set_browser_model(browser_model.clone().into());
    State {
        main_window,
        browser_model: browser_model,
    }
}

pub struct State {
    pub main_window: MainWindow,
    pub browser_model: Rc<slint::VecModel<BrowserConfig>>,
}

pub fn main() {
    let state = init();
    let main_window = state.main_window.clone_strong();
    #[cfg(target_os = "android")]
    STATE.with(|ui| *ui.borrow_mut() = Some(state));
    main_window.run().unwrap();
}
