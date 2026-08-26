use i_slint_backend_winit::WinitWindowAccessor;
use i_slint_backend_winit::winit::dpi::PhysicalPosition;
use i_slint_backend_winit::winit::monitor::MonitorHandle;
use i_slint_backend_winit::winit::window::Window;
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

fn get_cursor_monitor(window: &Window) -> Option<MonitorHandle> {
    let mut point = POINT { x: 0, y: 0 };
    let ok = unsafe { GetCursorPos(&mut point) };
    if ok.is_err() {
        return None;
    }

    for monitor in window.available_monitors() {
        let pos = monitor.position();
        let size = monitor.size();
        if point.x >= pos.x
            && point.x < pos.x + size.width as i32
            && point.y >= pos.y
            && point.y < pos.y + size.height as i32
        {
            return Some(monitor);
        }
    }
    None
}

pub fn center_window(window: &slint::Window) {
    if window.has_winit_window() {
        window.with_winit_window(|window: &Window| {
            let monitor = get_cursor_monitor(window)
                .or_else(|| window.current_monitor());
            if let Some(monitor) = monitor {
                set_centered(window, &monitor);
            }

            None as Option<()>
        });
    }
}

fn set_centered(window: &Window, monitor: &MonitorHandle) {
    let window_size = window.outer_size();

    let monitor_size = monitor.size();
    let monitor_position = monitor.position();

    let mut monitor_window_position = PhysicalPosition { x: 0, y: 0 };

    monitor_window_position.x = (monitor_position.x as f32 + (monitor_size.width as f32 * 0.5)
        - (window_size.width as f32 * 0.5)) as i32;

    monitor_window_position.y = (monitor_position.y as f32 + (monitor_size.height as f32 * 0.5)
        - (window_size.height as f32 * 0.5)) as i32;

    window.set_outer_position(monitor_window_position);
}
