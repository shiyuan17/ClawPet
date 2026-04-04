use tauri::Manager;

fn refresh_main_window<R: tauri::Runtime>(app: &tauri::App<R>) {
    if let Some(window) = app.get_webview_window("main") {
        if let Some(icon) = app.default_window_icon().cloned() {
            if let Err(error) = window.set_icon(icon) {
                eprintln!("[dragonclaw] failed to refresh main window icon: {error}");
            }
        }
        let _ = window.set_decorations(false);
        let _ = window.set_always_on_top(false);
        let _ = window.set_shadow(false);
        let _ = window.set_skip_taskbar(false);
        let _ = window.set_resizable(true);
        let _ = window.set_maximizable(true);
        let _ = window.set_minimizable(true);
    }
}

pub(crate) fn setup_app<R: tauri::Runtime>(
    app: &mut tauri::App<R>,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(resource_dir) = app.path().resource_dir() {
        crate::openclaw::state::set_app_resource_dir(resource_dir);
    }

    std::thread::spawn(|| {
        if let Err(error) = crate::openclaw::lobster::bootstrap_openclaw_runtime(true) {
            eprintln!("[dragonclaw] openclaw bootstrap skipped: {error}");
        }
    });

    #[cfg(desktop)]
    app.handle()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None::<Vec<&str>>,
        ))
        .map_err(Box::<dyn std::error::Error>::from)?;

    refresh_main_window(app);
    Ok(())
}
