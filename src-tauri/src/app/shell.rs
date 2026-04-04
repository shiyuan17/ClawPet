#[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
use std::path::Path;
use std::process::Command;

#[tauri::command]
pub(crate) fn open_external_url(url: String) -> Result<(), String> {
    let trimmed = url.trim();
    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        return Err("仅支持打开 http 或 https 链接。".to_string());
    }

    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = Command::new("open");
        command.arg(trimmed);
        command
    };

    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = Command::new("cmd");
        crate::suppress_windows_command_window(&mut command);
        command.args(["/C", "start", "", trimmed]);
        command
    };

    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    let mut command = {
        let mut command = Command::new("xdg-open");
        command.arg(trimmed);
        command
    };

    command
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("打开外部浏览器失败：{error}"))
}

#[tauri::command]
pub(crate) fn open_local_path_in_folder(path: String) -> Result<String, String> {
    let normalized = path.trim();
    if normalized.is_empty() {
        return Err("本地路径不能为空。".to_string());
    }

    let expanded = crate::expand_home_path(normalized);
    let resolved = if expanded.exists() {
        std::fs::canonicalize(&expanded).unwrap_or(expanded)
    } else {
        expanded
    };
    if !resolved.exists() {
        return Err(format!("目标路径不存在：{}", resolved.display()));
    }

    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = Command::new("open");
        if resolved.is_file() {
            command.arg("-R").arg(&resolved);
        } else {
            command.arg(&resolved);
        }
        command
    };

    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = Command::new("explorer");
        crate::suppress_windows_command_window(&mut command);
        if resolved.is_file() {
            command.arg("/select,").arg(&resolved);
        } else {
            command.arg(&resolved);
        }
        command
    };

    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    let mut command = {
        let mut command = Command::new("xdg-open");
        let target_dir = if resolved.is_dir() {
            resolved.clone()
        } else {
            resolved
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| resolved.clone())
        };
        command.arg(target_dir);
        command
    };

    command
        .spawn()
        .map_err(|error| format!("打开文件夹失败：{error}"))?;
    Ok(resolved.display().to_string())
}
