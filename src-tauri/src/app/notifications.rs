#[cfg(target_os = "macos")]
use std::ffi::{CStr, CString};
#[cfg(target_os = "macos")]
use std::os::raw::{c_char, c_int};
#[cfg(target_os = "macos")]
use std::process::{Command, Stdio};

#[cfg(not(target_os = "macos"))]
use tauri_plugin_notification::NotificationExt;

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn dragonclaw_show_user_notification(
        title_utf8: *const c_char,
        body_utf8: *const c_char,
        error_out: *mut *mut c_char,
    ) -> c_int;
    fn dragonclaw_free_c_string(value: *mut c_char);
}

#[cfg(target_os = "macos")]
fn run_macos_notification_ffi(
    ffi: unsafe extern "C" fn(*const c_char, *const c_char, *mut *mut c_char) -> c_int,
    title: &str,
    body: Option<&str>,
) -> Result<(), String> {
    let title_c = CString::new(title).map_err(|_| "通知标题包含无效字符。".to_string())?;
    let body_c =
        CString::new(body.unwrap_or("")).map_err(|_| "通知正文包含无效字符。".to_string())?;
    let mut error_ptr: *mut c_char = std::ptr::null_mut();

    let result = unsafe { ffi(title_c.as_ptr(), body_c.as_ptr(), &mut error_ptr) };
    if result == 1 {
        return Ok(());
    }

    let message = if error_ptr.is_null() {
        "系统通知发送失败。".to_string()
    } else {
        let message = unsafe { CStr::from_ptr(error_ptr) }
            .to_string_lossy()
            .into_owned();
        unsafe {
            dragonclaw_free_c_string(error_ptr);
        }
        message
    };

    Err(message)
}

#[cfg(target_os = "macos")]
fn show_macos_user_notification(title: &str, body: Option<&str>) -> Result<(), String> {
    run_macos_notification_ffi(dragonclaw_show_user_notification, title, body)
}

#[cfg(target_os = "macos")]
fn is_running_from_macos_app_bundle() -> bool {
    std::env::current_exe()
        .ok()
        .map(|path| {
            path.ancestors().any(|ancestor| {
                ancestor
                    .extension()
                    .and_then(|value| value.to_str())
                    .map(|value| value.eq_ignore_ascii_case("app"))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

#[cfg(target_os = "macos")]
fn show_macos_dev_helper_notification(title: &str, body: Option<&str>) -> Result<(), String> {
    let helper_app_path = option_env!("DRAGONCLAW_DEV_NOTIFIER_APP")
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "未找到开发态通知助手。".to_string())?;

    let mut command = Command::new("open");
    command
        .arg("-g")
        .arg("-na")
        .arg(helper_app_path)
        .arg("--args")
        .arg(title);
    if let Some(body_text) = body {
        command.arg(body_text);
    }

    let status = command
        .status()
        .map_err(|error| format!("启动开发态通知助手失败：{error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("开发态通知助手退出异常：{status}"))
    }
}

#[cfg(target_os = "macos")]
fn show_macos_osascript_dialog(title: &str, body: Option<&str>) -> Result<(), String> {
    let body_text = body.unwrap_or("");
    let mut command = Command::new("osascript");
    command
        .arg("-e")
        .arg("on run argv")
        .arg("-e")
        .arg("set theTitle to item 1 of argv")
        .arg("-e")
        .arg("set theBody to \"\"")
        .arg("-e")
        .arg("if (count of argv) > 1 then set theBody to item 2 of argv")
        .arg("-e")
        .arg("display dialog theBody with title theTitle buttons {\"知道了\"} default button 1 giving up after 12 with icon note")
        .arg("-e")
        .arg("end run")
        .arg("--")
        .arg(title)
        .arg(body_text)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    command
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("调用 osascript 显示系统弹窗失败：{error}"))
}

#[cfg(target_os = "macos")]
fn show_macos_osascript_notification(title: &str, body: Option<&str>) -> Result<(), String> {
    let body_text = body.unwrap_or("");
    let status = Command::new("osascript")
        .arg("-e")
        .arg("on run argv")
        .arg("-e")
        .arg("display notification (item 2 of argv) with title (item 1 of argv)")
        .arg("-e")
        .arg("end run")
        .arg("--")
        .arg(title)
        .arg(body_text)
        .status()
        .map_err(|error| format!("调用 osascript 发送系统通知失败：{error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("osascript 发送系统通知失败，退出码：{status}"))
    }
}

#[cfg(target_os = "macos")]
fn show_platform_system_notification(
    _app: &tauri::AppHandle,
    title: &str,
    body: Option<&str>,
) -> Result<(), String> {
    if !is_running_from_macos_app_bundle() {
        eprintln!("[dragonclaw] using osascript dialog path in dev runtime");
        return show_macos_osascript_dialog(title, body)
            .or_else(|dialog_error| {
                eprintln!("[dragonclaw] osascript dialog failed: {dialog_error}");
                show_macos_dev_helper_notification(title, body)
            })
            .or_else(|helper_error| {
                eprintln!("[dragonclaw] dev helper notification failed: {helper_error}");
                show_macos_osascript_notification(title, body)
            });
    }

    eprintln!("[dragonclaw] using modern macOS notification path in app bundle runtime");
    show_macos_user_notification(title, body).or_else(|modern_error| {
        eprintln!("[dragonclaw] modern macOS notification failed: {modern_error}");
        show_macos_osascript_notification(title, body)
    })
}

#[cfg(not(target_os = "macos"))]
fn show_platform_system_notification(
    app: &tauri::AppHandle,
    title: &str,
    body: Option<&str>,
) -> Result<(), String> {
    let mut builder = app.notification().builder().title(title.to_string());
    if let Some(body_text) = body {
        builder = builder.body(body_text.to_string());
    }

    builder
        .show()
        .map_err(|error| format!("系统通知发送失败：{error}"))
}

#[tauri::command]
pub(crate) fn show_system_notification(
    app: tauri::AppHandle,
    title: String,
    body: Option<String>,
) -> Result<(), String> {
    let normalized_title = title.trim().to_string();
    if normalized_title.is_empty() {
        return Err("通知标题不能为空。".to_string());
    }

    let normalized_body = body
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    show_platform_system_notification(&app, &normalized_title, normalized_body.as_deref())
}
