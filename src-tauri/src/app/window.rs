#[cfg(target_os = "windows")]
use std::thread;
#[cfg(target_os = "windows")]
use std::time::Duration;

use tauri::Emitter;
use tauri::Manager;

#[tauri::command]
pub(crate) fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

#[cfg(target_os = "windows")]
fn center_window_on_current_monitor(window: &tauri::WebviewWindow) {
    let monitor = window
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| window.primary_monitor().ok().flatten());
    let Some(monitor) = monitor else {
        return;
    };

    let Ok(window_size) = window.outer_size() else {
        return;
    };
    let monitor_position = monitor.position();
    let monitor_size = monitor.size();

    let centered_x =
        monitor_position.x + ((monitor_size.width as i32 - window_size.width as i32).max(0) / 2);
    let centered_y =
        monitor_position.y + ((monitor_size.height as i32 - window_size.height as i32).max(0) / 2);

    let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition::new(
        centered_x, centered_y,
    )));
}

#[tauri::command]
pub(crate) fn start_main_window_drag(app: tauri::AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "main window not found".to_string())?;
    window.start_dragging().map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn toggle_main_window_maximize(app: tauri::AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "main window not found".to_string())?;
    let is_maximized = window.is_maximized().map_err(|error| error.to_string())?;
    if is_maximized {
        window.unmaximize().map_err(|error| error.to_string())?;
    } else {
        window.maximize().map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub(crate) fn toggle_main_window_visibility(app: &tauri::AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };

    let is_visible = window.is_visible().unwrap_or(true);
    if is_visible {
        let _ = window.hide();
        return;
    }

    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
}

pub(crate) fn open_main_chat_panel(app: &tauri::AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };

    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
    let _ = window.emit("dragonclaw://chat-open", "main");
}

#[cfg(target_os = "windows")]
fn reinforce_main_window_overlay(app: tauri::AppHandle) {
    thread::spawn(move || {
        for delay_ms in [300u64] {
            thread::sleep(Duration::from_millis(delay_ms));

            let Some(window) = app.get_webview_window("main") else {
                return;
            };

            let _ = window.set_decorations(false);
            let _ = window.set_shadow(false);
            let _ = window.set_always_on_top(true);
            let _ = window.set_resizable(false);
            center_window_on_current_monitor(&window);

            unsafe {
                use windows::Win32::Foundation::HWND;
                use windows::Win32::Graphics::Dwm::DwmExtendFrameIntoClientArea;
                use windows::Win32::UI::Controls::MARGINS;
                use windows::Win32::UI::WindowsAndMessaging::*;

                if let Ok(raw) = window.hwnd() {
                    let hwnd = HWND(raw.0 as *mut _);

                    let style = GetWindowLongW(hwnd, GWL_STYLE);
                    SetWindowLongW(
                        hwnd,
                        GWL_STYLE,
                        style & !(WS_CAPTION.0 as i32) & !(WS_THICKFRAME.0 as i32),
                    );

                    let margins = MARGINS {
                        cxLeftWidth: -1,
                        cxRightWidth: -1,
                        cyTopHeight: -1,
                        cyBottomHeight: -1,
                    };
                    let _ = DwmExtendFrameIntoClientArea(hwnd, &margins);

                    let _ = SetWindowPos(
                        hwnd,
                        None,
                        0,
                        0,
                        0,
                        0,
                        SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
                    );
                }
            }

            let _ = window.with_webview(|webview| unsafe {
                use webview2_com::Microsoft::Web::WebView2::Win32::*;
                use windows::core::Interface;

                let controller = webview.controller();
                if let Ok(c2) = controller.cast::<ICoreWebView2Controller2>() {
                    let _ = c2.SetDefaultBackgroundColor(COREWEBVIEW2_COLOR {
                        A: 0,
                        R: 0,
                        G: 0,
                        B: 0,
                    });
                }
            });
        }
    });
}
