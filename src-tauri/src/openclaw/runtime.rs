use reqwest::header::ACCEPT;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

pub(crate) fn find_command_paths(command_name: &str) -> Vec<String> {
    #[cfg(target_os = "windows")]
    let output = {
        let mut command = Command::new("where");
        crate::suppress_windows_command_window(&mut command);
        match command.arg(command_name).output() {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        }
    };

    #[cfg(not(target_os = "windows"))]
    let output = {
        let mut with_all = Command::new("which");
        crate::suppress_windows_command_window(&mut with_all);
        if let Ok(value) = with_all.arg("-a").arg(command_name).output() {
            if value.status.success() {
                value
            } else {
                let mut fallback = Command::new("which");
                crate::suppress_windows_command_window(&mut fallback);
                match fallback.arg(command_name).output() {
                    Ok(value) => value,
                    Err(_) => return Vec::new(),
                }
            }
        } else {
            let mut fallback = Command::new("which");
            crate::suppress_windows_command_window(&mut fallback);
            match fallback.arg(command_name).output() {
                Ok(value) => value,
                Err(_) => return Vec::new(),
            }
        }
    };

    if !output.status.success() {
        return Vec::new();
    }

    let mut dedup = std::collections::HashSet::new();
    let mut output_paths = Vec::new();
    for line in String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
    {
        if line.is_empty() {
            continue;
        }
        if dedup.insert(line.to_string()) {
            output_paths.push(line.to_string());
        }
    }
    output_paths
}

pub(crate) fn resolve_project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

pub(crate) fn resolve_resource_root_candidates() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Some(path) = crate::app_resource_dir() {
        roots.push(path.clone());
        roots.push(path.join("resources"));
        roots.push(path.join("src-tauri").join("resources"));
    }

    roots.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources"));

    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(exe_dir) = current_exe.parent() {
            roots.push(exe_dir.join("resources"));
            roots.push(exe_dir.join("../resources"));
        }
    }

    let mut dedup = std::collections::HashSet::new();
    let mut output = Vec::new();
    for root in roots {
        let key = root.display().to_string();
        if dedup.insert(key) {
            output.push(root);
        }
    }

    output
}

#[allow(dead_code)]
fn resolve_openclaw_runtime_dir() -> Option<PathBuf> {
    if let Ok(explicit) = std::env::var("OPENCLAW_DIR") {
        let trimmed = explicit.trim();
        if !trimmed.is_empty() {
            let candidate = PathBuf::from(trimmed);
            if candidate.join("openclaw.mjs").exists() && candidate.join("package.json").exists() {
                return Some(candidate);
            }
        }
    }

    let project_root = resolve_project_root();
    let mut candidates = Vec::new();
    for root in resolve_resource_root_candidates() {
        candidates.push(root.join("openclaw"));
        candidates.push(root.join("resources").join("openclaw"));
        candidates.push(root.join("build").join("openclaw"));
        candidates.push(root.join("resources").join("build").join("openclaw"));
    }
    candidates.push(project_root.join("build").join("openclaw"));
    candidates.push(project_root.join("node_modules").join("openclaw"));

    candidates.into_iter().find(|candidate| {
        candidate.join("openclaw.mjs").exists() && candidate.join("package.json").exists()
    })
}

fn collect_node_binary_candidates() -> Vec<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(explicit) = std::env::var("OPENCLAW_NODE_PATH") {
        let trimmed = explicit.trim();
        if !trimmed.is_empty() {
            let candidate = PathBuf::from(trimmed);
            if candidate.exists() {
                candidates.push(candidate);
            }
        }
    }

    for path in find_command_paths("node") {
        let candidate = PathBuf::from(path);
        if candidate.exists() {
            candidates.push(candidate);
        }
    }

    let mut dedup = std::collections::HashSet::new();
    let mut output = Vec::new();
    for candidate in candidates {
        let key = candidate.display().to_string();
        if dedup.insert(key) {
            output.push(candidate);
        }
    }
    output
}

const OPENCLAW_MIN_NODE_MAJOR: u32 = 22;
const OPENCLAW_MIN_NODE_MINOR: u32 = 16;

fn parse_node_version_component(raw: &str) -> Option<u32> {
    let digits = raw
        .trim()
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<u32>().ok()
}

fn parse_node_version_triplet(raw: &str) -> Option<(u32, u32, u32)> {
    let cleaned = raw.trim().trim_start_matches('v');
    let mut parts = cleaned.split('.');
    let major = parts.next().and_then(parse_node_version_component)?;
    let minor = parts
        .next()
        .and_then(parse_node_version_component)
        .unwrap_or(0);
    let patch = parts
        .next()
        .and_then(parse_node_version_component)
        .unwrap_or(0);
    Some((major, minor, patch))
}

fn openclaw_required_node_version_label() -> String {
    format!("{OPENCLAW_MIN_NODE_MAJOR}.{OPENCLAW_MIN_NODE_MINOR}.0")
}

fn check_node_version_supported(raw: &str) -> bool {
    let Some((major, minor, _patch)) = parse_node_version_triplet(raw) else {
        return false;
    };
    major > OPENCLAW_MIN_NODE_MAJOR
        || (major == OPENCLAW_MIN_NODE_MAJOR && minor >= OPENCLAW_MIN_NODE_MINOR)
}

fn read_node_binary_version(node_path: &Path) -> Result<String, String> {
    let normalized_node_path = crate::normalize_windows_path_for_child_process(node_path);
    let mut command = Command::new(&normalized_node_path);
    crate::suppress_windows_command_window(&mut command);
    let output = command.arg("-v").output().map_err(|error| {
        format!(
            "执行 Node 版本检查失败（{}）: {error}",
            normalized_node_path.display()
        )
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!(
                "Node 版本检查失败（{}，exit: {}）。",
                node_path.display(),
                output.status.code().unwrap_or(-1)
            )
        } else {
            format!("Node 版本检查失败（{}）：{stderr}", node_path.display())
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let normalized = stdout.trim().trim_start_matches('v').to_string();
    if normalized.is_empty() {
        return Err(format!(
            "无法解析 Node 版本（{} 输出为空）。",
            node_path.display()
        ));
    }

    Ok(normalized)
}

pub(crate) fn resolve_openclaw_node_runtime() -> Result<(PathBuf, String), String> {
    let candidates = collect_node_binary_candidates();
    if candidates.is_empty() {
        return Err("未找到 Node.js 可执行文件（可通过 OPENCLAW_NODE_PATH 指定）。".to_string());
    }

    let mut diagnostics: Vec<String> = Vec::new();
    let mut inspected_versions: Vec<String> = Vec::new();
    let mut highest_supported_candidate: Option<(PathBuf, String, (u32, u32, u32))> = None;
    let mut highest_unsupported_candidate: Option<(PathBuf, String, (u32, u32, u32))> = None;

    for node in candidates {
        match read_node_binary_version(&node) {
            Ok(version) => {
                if check_node_version_supported(&version) {
                    let parsed = parse_node_version_triplet(&version).unwrap_or((0, 0, 0));
                    match &highest_supported_candidate {
                        Some((_best_path, _best_version, best_triplet))
                            if parsed <= *best_triplet => {}
                        _ => {
                            highest_supported_candidate = Some((node, version, parsed));
                        }
                    }
                    continue;
                }

                inspected_versions.push(format!("{}（{}）", version, node.display()));

                let parsed = parse_node_version_triplet(&version).unwrap_or((0, 0, 0));
                match &highest_unsupported_candidate {
                    Some((_best_path, _best_version, best_triplet)) if parsed <= *best_triplet => {}
                    _ => {
                        highest_unsupported_candidate = Some((node, version, parsed));
                    }
                }
            }
            Err(error) => diagnostics.push(error),
        }
    }

    if let Some((node, version, _)) = highest_supported_candidate {
        return Ok((node, version));
    }

    if let Some((node, version, _)) = highest_unsupported_candidate {
        let inspected = if inspected_versions.is_empty() {
            String::new()
        } else {
            format!(" 已检测到候选 Node：{}。", inspected_versions.join("；"))
        };
        let diagnostics_detail = if diagnostics.is_empty() {
            String::new()
        } else {
            format!(" 其他候选检查异常：{}。", diagnostics.join("；"))
        };
        return Err(format!(
            "OpenClaw 运行条件不满足：Node 版本过低（当前 {}，路径 {}），要求 >= {}。请升级 Node 后重试，或通过 OPENCLAW_NODE_PATH 指向符合要求的 Node。{}{}",
            version,
            node.display(),
            openclaw_required_node_version_label(),
            inspected,
            diagnostics_detail
        ));
    }

    Err(format!(
        "未找到可用 Node.js 运行时。{}",
        if diagnostics.is_empty() {
            "请安装 Node，或通过 OPENCLAW_NODE_PATH 指向可执行文件。".to_string()
        } else {
            diagnostics.join("；")
        }
    ))
}

#[allow(dead_code)]
fn read_json_version_field(path: &Path) -> Option<String> {
    let raw = std::fs::read_to_string(path).ok()?;
    let parsed = serde_json::from_str::<Value>(&raw).ok()?;
    parsed
        .get("version")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

#[cfg(test)]
fn parse_dotted_numeric_version(raw: &str) -> Option<Vec<u32>> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut segments = Vec::new();
    for segment in trimmed.split('.') {
        if segment.is_empty() {
            return None;
        }
        let parsed = segment.parse::<u32>().ok()?;
        segments.push(parsed);
    }
    if segments.is_empty() {
        None
    } else {
        Some(segments)
    }
}

#[cfg(test)]
fn compare_dotted_numeric_versions(left: &str, right: &str) -> Option<std::cmp::Ordering> {
    let left_segments = parse_dotted_numeric_version(left)?;
    let right_segments = parse_dotted_numeric_version(right)?;
    let max_len = left_segments.len().max(right_segments.len());
    for idx in 0..max_len {
        let left_value = *left_segments.get(idx).unwrap_or(&0);
        let right_value = *right_segments.get(idx).unwrap_or(&0);
        match left_value.cmp(&right_value) {
            std::cmp::Ordering::Equal => {}
            value => return Some(value),
        }
    }
    Some(std::cmp::Ordering::Equal)
}

#[cfg(test)]
pub(crate) fn is_openclaw_official_version_newer(official: &str, bundled: &str) -> bool {
    match compare_dotted_numeric_versions(official, bundled) {
        Some(std::cmp::Ordering::Greater) => true,
        Some(_) => false,
        None => official.trim() != bundled.trim(),
    }
}

#[allow(dead_code)]
pub(crate) fn fetch_openclaw_latest_official_version() -> Result<String, String> {
    tauri::async_runtime::block_on(async {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(4))
            .build()
            .map_err(|error| format!("创建官网版本检查客户端失败: {error}"))?;
        let response = client
            .get("https://registry.npmjs.org/openclaw/latest")
            .header(ACCEPT, "application/json")
            .send()
            .await
            .map_err(|error| format!("访问官网版本接口失败: {error}"))?;
        let status = response.status();
        if !status.is_success() {
            return Err(format!("官网版本接口返回异常状态：{status}"));
        }
        let payload = response
            .json::<Value>()
            .await
            .map_err(|error| format!("解析官网版本响应失败: {error}"))?;
        let version = payload
            .get("version")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or("官网版本响应未包含有效 version 字段。".to_string())?;
        Ok(version.to_string())
    })
}

#[allow(dead_code)]
pub(crate) fn read_bundled_openclaw_runtime_version() -> Option<String> {
    let runtime_dir = resolve_openclaw_runtime_dir()?;
    read_json_version_field(&runtime_dir.join("package.json"))
}

#[allow(dead_code)]
fn resolve_openclaw_cli_wrapper_source() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    for root in resolve_resource_root_candidates() {
        #[cfg(target_os = "windows")]
        {
            candidates.push(root.join("cli").join("openclaw.cmd"));
            candidates.push(root.join("cli").join("win32").join("openclaw.cmd"));
            candidates.push(root.join("resources").join("cli").join("openclaw.cmd"));
            candidates.push(
                root.join("resources")
                    .join("cli")
                    .join("win32")
                    .join("openclaw.cmd"),
            );
        }
        #[cfg(not(target_os = "windows"))]
        {
            candidates.push(root.join("cli").join("openclaw"));
            candidates.push(root.join("cli").join("posix").join("openclaw"));
            candidates.push(root.join("resources").join("cli").join("openclaw"));
            candidates.push(
                root.join("resources")
                    .join("cli")
                    .join("posix")
                    .join("openclaw"),
            );
        }
    }

    candidates
        .into_iter()
        .find(|path| path.exists() && path.is_file())
}

pub(crate) fn prepend_global_openclaw_cli_to_command_path(command: &mut Command) -> Option<String> {
    let cli_candidate = collect_openclaw_cli_command_candidates()
        .into_iter()
        .next()?;
    let cli_dir = cli_candidate.parent()?.to_path_buf();
    let preferred_dir = crate::normalize_windows_path_for_child_process(&cli_dir);

    let mut path_entries = vec![preferred_dir.clone()];
    if let Some(existing_path) = std::env::var_os("PATH") {
        path_entries.extend(std::env::split_paths(&existing_path));
    }

    let joined_path = std::env::join_paths(path_entries).ok()?;
    command.env("PATH", joined_path);
    Some(preferred_dir.display().to_string())
}

fn apply_openclaw_child_process_env(command: &mut Command) {
    let openclaw_state_dir =
        crate::normalize_windows_path_for_child_process(&crate::resolve_openclaw_home_path());
    let openclaw_config =
        crate::normalize_windows_path_for_child_process(&crate::resolve_openclaw_config_path());
    command
        .env("OPENCLAW_NO_RESPAWN", "1")
        .env("OPENCLAW_EMBEDDED_IN", "DragonClaw")
        .env("OPENCLAW_STATE_DIR", openclaw_state_dir.clone())
        .env("CLAWDBOT_STATE_DIR", openclaw_state_dir)
        .env("OPENCLAW_CONFIG_PATH", openclaw_config);
    command.env_remove("OPENCLAW_HOME");

    if let Some(token) = crate::resolve_openclaw_gateway_token() {
        command.env("OPENCLAW_GATEWAY_TOKEN", token);
    }
}

#[allow(dead_code)]
fn collect_npm_command_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::<PathBuf>::new();

    if let Ok(explicit) = std::env::var("OPENCLAW_NPM_PATH") {
        let trimmed = explicit.trim();
        if !trimmed.is_empty() {
            let candidate = PathBuf::from(trimmed);
            if candidate.exists() {
                candidates.push(candidate);
            }
        }
    }

    if let Ok((node_path, _)) = resolve_openclaw_node_runtime() {
        if let Some(node_dir) = node_path.parent() {
            #[cfg(target_os = "windows")]
            let local_npm_candidates = [
                node_dir.join("npm.cmd"),
                node_dir.join("npm.exe"),
                node_dir.join("npm"),
            ];
            #[cfg(not(target_os = "windows"))]
            let local_npm_candidates = [node_dir.join("npm")];

            for candidate in local_npm_candidates {
                if candidate.exists() {
                    candidates.push(candidate);
                }
            }
        }
    }

    for path in find_command_paths("npm") {
        let candidate = PathBuf::from(path);
        if candidate.exists() {
            candidates.push(candidate);
        }
    }

    let mut dedup = std::collections::HashSet::new();
    let mut output = Vec::new();
    for candidate in candidates {
        let key = candidate.display().to_string();
        if dedup.insert(key) {
            output.push(candidate);
        }
    }
    output
}

#[allow(dead_code)]
pub(crate) fn run_openclaw_cli_via_npm_exec(
    package_spec: &str,
    args: &[String],
) -> Result<(String, std::process::Output), String> {
    let npm_candidates = collect_npm_command_candidates();
    if npm_candidates.is_empty() {
        return Err("未找到 npm 可执行文件（请确认 PATH 或 OPENCLAW_NPM_PATH）。".to_string());
    }

    let args_text = args.join(" ");
    let mut launch_errors = Vec::new();
    for npm_raw in npm_candidates {
        let npm = crate::normalize_windows_path_for_child_process(&npm_raw);
        let mut command = Command::new(&npm);
        crate::suppress_windows_command_window(&mut command);
        command
            .arg("exec")
            .arg("--yes")
            .arg(package_spec)
            .arg("--")
            .args(args)
            .current_dir(resolve_project_root());
        apply_openclaw_child_process_env(&mut command);

        let command_display = format!(
            "{} exec --yes {} -- {}",
            npm.display(),
            package_spec,
            args_text
        );
        match command.output() {
            Ok(output) => return Ok((command_display, output)),
            Err(error) => launch_errors.push(format!("{}: {}", npm.display(), error)),
        }
    }

    Err(format!(
        "调用 npm exec 失败（{}）。",
        launch_errors.join("；")
    ))
}

pub(crate) fn collect_openclaw_cli_command_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::<PathBuf>::new();

    if let Ok(explicit) = std::env::var("OPENCLAW_BIN_PATH") {
        let trimmed = explicit.trim();
        if !trimmed.is_empty() {
            let candidate = PathBuf::from(trimmed);
            if candidate.exists() {
                candidates.push(candidate);
            }
        }
    }

    if let Some(home_parent) = crate::resolve_default_openclaw_home_path().parent() {
        let petclaw_node_root = home_parent.join(".petclaw").join("node");
        #[cfg(target_os = "windows")]
        let petclaw_candidates = vec![
            petclaw_node_root.join("bin").join("openclaw.cmd"),
            petclaw_node_root.join("bin").join("openclaw.exe"),
            petclaw_node_root.join("openclaw.cmd"),
            petclaw_node_root.join("openclaw.exe"),
        ];
        #[cfg(not(target_os = "windows"))]
        let petclaw_candidates = vec![petclaw_node_root.join("bin").join("openclaw")];

        for candidate in petclaw_candidates {
            if candidate.exists() {
                candidates.push(candidate);
            }
        }
    }

    for path in find_command_paths("openclaw") {
        let candidate = PathBuf::from(path);
        if candidate.exists() {
            candidates.push(candidate);
        }
    }
    #[cfg(target_os = "windows")]
    for path in find_command_paths("openclaw.cmd") {
        let candidate = PathBuf::from(path);
        if candidate.exists() {
            candidates.push(candidate);
        }
    }

    let mut dedup = std::collections::HashSet::new();
    let mut output = Vec::new();
    for candidate in candidates {
        let key = candidate.display().to_string();
        if dedup.insert(key) {
            output.push(candidate);
        }
    }
    output
}

fn run_openclaw_cli_via_global_command(
    args: &[&str],
) -> Result<(String, std::process::Output), String> {
    let cli_candidates = collect_openclaw_cli_command_candidates();
    if cli_candidates.is_empty() {
        return Err(
            "未找到全局 OpenClaw CLI（openclaw）。请先安装到系统环境并确保 PATH 可见。".to_string(),
        );
    }

    let mut launch_errors = Vec::new();
    for cli_raw in cli_candidates {
        let cli = crate::normalize_windows_path_for_child_process(&cli_raw);
        let mut command = Command::new(&cli);
        crate::suppress_windows_command_window(&mut command);
        command.args(args).current_dir(resolve_project_root());
        apply_openclaw_child_process_env(&mut command);
        let command_display = format!("{} {}", cli.display(), args.join(" "));
        match command.output() {
            Ok(output) => return Ok((command_display, output)),
            Err(error) => launch_errors.push(format!("{}: {}", cli.display(), error)),
        }
    }

    Err(format!(
        "调用全局 OpenClaw CLI 失败（{}）。",
        launch_errors.join("；")
    ))
}

pub(crate) fn run_openclaw_cli_output(
    args: &[&str],
) -> Result<(String, std::process::Output), String> {
    run_openclaw_cli_via_global_command(args)
}

#[allow(dead_code)]
pub(crate) fn install_openclaw_cli_wrapper() -> Result<Option<String>, String> {
    #[cfg(target_os = "windows")]
    {
        let wrapper = resolve_openclaw_cli_wrapper_source()
            .ok_or("未找到内置 OpenClaw CLI 包装器（openclaw.cmd）。".to_string())?;
        let cli_dir = wrapper
            .parent()
            .ok_or("无法解析 CLI 包装器目录。".to_string())?;
        let helper_path = cli_dir.join("update-user-path.ps1");
        if !helper_path.exists() {
            return Ok(Some(cli_dir.display().to_string()));
        }

        let power_shell = std::env::var("SystemRoot")
            .map(|root| {
                PathBuf::from(root)
                    .join("System32")
                    .join("WindowsPowerShell")
                    .join("v1.0")
                    .join("powershell.exe")
            })
            .unwrap_or_else(|_| PathBuf::from("powershell.exe"));

        let mut command = Command::new(power_shell);
        crate::suppress_windows_command_window(&mut command);
        let output = command
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                helper_path.to_string_lossy().as_ref(),
                "-Action",
                "add",
                "-CliDir",
                cli_dir.to_string_lossy().as_ref(),
            ])
            .output()
            .map_err(|error| format!("更新 Windows PATH 失败: {error}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(if stderr.is_empty() {
                format!(
                    "更新 Windows PATH 失败（exit: {}）。",
                    output.status.code().unwrap_or(-1)
                )
            } else {
                format!("更新 Windows PATH 失败：{stderr}")
            });
        }

        return Ok(Some(cli_dir.display().to_string()));
    }

    #[cfg(not(target_os = "windows"))]
    {
        let source = resolve_openclaw_cli_wrapper_source()
            .ok_or("未找到内置 OpenClaw CLI 包装器。".to_string())?;
        let home = std::env::var("HOME").map_err(|_| "无法读取 HOME 环境变量。".to_string())?;
        let target_dir = PathBuf::from(home).join(".local").join("bin");
        std::fs::create_dir_all(&target_dir)
            .map_err(|error| format!("创建 CLI 目录失败（{}）: {error}", target_dir.display()))?;
        let target_path = target_dir.join("openclaw");

        if target_path.exists() {
            let _ = std::fs::remove_file(&target_path);
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::{symlink, PermissionsExt};
            symlink(&source, &target_path).map_err(|error| {
                format!(
                    "创建 CLI 软链接失败（{} -> {}）: {error}",
                    target_path.display(),
                    source.display()
                )
            })?;
            if let Ok(metadata) = std::fs::metadata(&source) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o755);
                let _ = std::fs::set_permissions(&source, perms);
            }
        }

        Ok(Some(target_path.display().to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::is_openclaw_official_version_newer;

    #[test]
    fn dotted_version_compare_detects_newer_official_version() {
        assert!(is_openclaw_official_version_newer("2026.3.28", "2026.3.13"));
        assert!(!is_openclaw_official_version_newer(
            "2026.3.13",
            "2026.3.28"
        ));
        assert!(!is_openclaw_official_version_newer(
            "2026.3.28",
            "2026.3.28"
        ));
    }

    #[test]
    fn dotted_version_compare_handles_length_differences() {
        assert!(is_openclaw_official_version_newer("2026.4", "2026.3.99"));
        assert!(!is_openclaw_official_version_newer("2026.3", "2026.3.0"));
    }
}
